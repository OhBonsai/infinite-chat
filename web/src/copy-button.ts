// copy-button.ts — Plan 21 P1(0030 步骤 1):每条**可见消息**一个"复制"按钮。
//
// 每帧读 `chat.visible_turns()`(JSON,设备像素屏幕坐标)→ 按 `id` 复用一个浮层 `<button>`,摆在该
// 消息盒**右下缘外**(meta 行位,Plan 25:不再压消息文字);点击 → `navigator.clipboard.writeText(text)`
// (用户手势,满足 Clipboard 约束)→ 短暂"已复制 ✓"。滚出视口 / 卸载的按钮回收(同 `embed-overlay`
// 习惯)。**按钮数 ∝ 可见消息**(虚拟化,core 只吐 Hot 可见块)→ 不随历史增长。
//
// Plan 25 P0(design §4.2)meta 延迟显:未 `settled`(揭示未结算 = 流式中)的消息不显 copy
// (Complete 态才现);首次出现走 CSS 渐显(dur.element=240ms,延迟 stagger.stage=60ms ——
// 镜像 core MotionTokens 默认)。

import { syncLayerToCanvas } from "./canvas-rect";

interface CopyHost {
  visible_turns(): string;
}

interface ScreenTurn {
  id: number;
  turn: number;
  role: string;
  x: number;
  y: number;
  w: number;
  h: number;
  text: string;
  settled: boolean;
}

let layer: HTMLDivElement | null = null;
// Plan 25:按钮**每「回合×角色」一个**(meta 行语义),不逐 part —— key = `${turn}:${role}`。
const btns = new Map<string, HTMLButtonElement>();
const texts = new Map<string, string>(); // key → 该组全部可见 part 文本(点击时取)

function ensureLayer(): HTMLDivElement {
  if (!layer) {
    layer = document.createElement("div");
    // 对齐画布矩形(canvas-rect)、容器本身不挡画布(pointer-events:none),按钮单独开 auto。
    // z 在画布上、面板下。left/top/width/height 由 syncLayerToCanvas 维护。
    layer.style.cssText = "position:fixed;z-index:55;pointer-events:none;overflow:hidden";
    document.body.appendChild(layer);
  }
  return layer;
}

function makeButton(key: string, hoverOnly: boolean): HTMLButtonElement {
  const b = document.createElement("button");
  b.className = "copy-btn";
  b.type = "button";
  b.textContent = "复制";
  b.style.cssText =
    "position:absolute;pointer-events:auto;cursor:pointer;font-size:11px;line-height:1;" +
    "padding:3px 7px;border-radius:6px;border:1px solid rgba(255,255,255,0.18);" +
    "background:rgba(40,44,54,0.78);color:#cdd3df;backdrop-filter:blur(4px);" +
    // Plan 25:初始透明,下一帧渐显(dur.element 240ms,延迟 stagger.stage 60ms)= meta 延迟显。
    "opacity:0;transition:opacity 0.24s ease 0.06s;will-change:transform";
  // user 消息 meta = hover 才显(design §4.1);assistant meta = Complete 后常显(半透明)。
  const rest = hoverOnly ? "0" : "0.55";
  requestAnimationFrame(() => (b.style.opacity = rest));
  b.addEventListener("mouseenter", () => (b.style.opacity = "1"));
  b.addEventListener("mouseleave", () => (b.style.opacity = rest));
  b.addEventListener("click", () => {
    const text = texts.get(key) ?? "";
    void navigator.clipboard.writeText(text).then(
      () => flash(b, "已复制 ✓"),
      () => flash(b, "复制失败"),
    );
  });
  return b;
}

function flash(b: HTMLButtonElement, msg: string): void {
  b.textContent = msg;
  b.style.opacity = "1";
  window.setTimeout(() => {
    b.textContent = "复制";
    b.style.opacity = "0.55";
  }, 1100);
}

/** 一帧:把复制按钮同步到当前可见消息(main rAF 调,同 embed-overlay)。 */
export function pumpCopyButtons(host: CopyHost): void {
  let turns: ScreenTurn[];
  try {
    turns = JSON.parse(host.visible_turns()) as ScreenTurn[];
  } catch {
    return;
  }
  const root = ensureLayer();
  syncLayerToCanvas(root);
  const dpr = window.devicePixelRatio || 1;
  // Plan 25:按「回合×角色」聚合(meta 行语义)。组内全部 settled 才现(Complete 态);
  // 锚 = 组内最底 part 盒;文本 = 组内可见 part 按文档序拼接。
  interface Group {
    anchor: ScreenTurn;
    parts: ScreenTurn[];
    allSettled: boolean;
  }
  const groups = new Map<string, Group>();
  for (const t of turns) {
    const key = `${t.turn}:${t.role}`;
    const g = groups.get(key);
    if (!g) {
      groups.set(key, { anchor: t, parts: [t], allSettled: t.settled });
    } else {
      g.parts.push(t);
      g.allSettled = g.allSettled && t.settled;
      if (t.y + t.h > g.anchor.y + g.anchor.h) g.anchor = t;
    }
  }
  const seen = new Set<string>();
  for (const [key, g] of groups) {
    if (!g.allSettled) continue; // 流式中 → meta 不现
    seen.add(key);
    texts.set(
      key,
      g.parts
        .sort((a, b) => a.id - b.id)
        .map((p) => p.text)
        .join("\n\n"),
    );
    let b = btns.get(key);
    if (!b) {
      b = makeButton(key, g.anchor.role === "user");
      // 稳定身份 → 跨帧/测试定位:data-turn-id = 锚 part 的 view 下标(单 part 组即该 part)。
      b.dataset.turnId = String(g.anchor.id);
      root.appendChild(b);
      btns.set(key, b);
    }
    // 设备像素 → CSS 像素(÷DPR)。摆组底 part 盒**右下缘外 4px**(meta 行位,不压文字)。
    const right = (g.anchor.x + g.anchor.w) / dpr;
    const bottom = (g.anchor.y + g.anchor.h) / dpr;
    b.style.left = `${right - 52}px`;
    b.style.top = `${bottom + 4}px`;
  }
  // 回收滚出视口 / 卸载的按钮。
  for (const [key, b] of btns) {
    if (!seen.has(key)) {
      b.remove();
      btns.delete(key);
      texts.delete(key);
    }
  }
}
