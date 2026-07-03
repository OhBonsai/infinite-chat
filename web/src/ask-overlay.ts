// ask-overlay.ts — Plan 27 B 路:question 活跃期 = DOM overlay 锚定表单(checkbox 多选 + 自由输入)。
//
// 为什么 DOM:自由文本输入必须真 `<input>`(IME 组合输入 canvas 自绘是深坑,0030 已论证)。
// 锚定:容器对齐画布(canvas-rect),表单每帧按 `ask_rect()`(pending ask 块的屏幕矩形)摆位——
// 与 embed-overlay 同款相机同步 → 随滚动跟块走。表单实测高比块高 → `set_ask_height` 单向回报
// 一次(块补白,防遮挡下文;§3)。提交 → `reply_question_with` → 表单卸载,落定卡原位接管
// (状态先行:core 已把块渲染成答案卡,DOM 后撤 → 间隙无闪)。
//
// permission 不走这里(A 路 wasm 原生按钮);表单天生读屏可达(26② announcer 沿用)。

import { syncLayerToCanvas } from "./canvas-rect";

interface AskHost {
  ask_state(): string;
  ask_rect(): string;
  reply_question_with(json: string): boolean;
  set_ask_height(px: number): void;
}

interface AskState {
  kind: string;
  prompt: string;
  options: string[];
  block: number;
}

let layer: HTMLDivElement | null = null;
let form: HTMLFormElement | null = null;
let formForBlock = -1; // 当前表单对应的 ask 块(换块/落定 → 重建/卸载)
let heightReported = false; // §3:单向回报一次,不做双向反馈环(防抖动)
let focusedOnce = false; // 首次**可见**时聚焦(建号当帧块矩形可能未就绪 → display:none 会丢焦点)
let prevFocus: HTMLElement | null = null;

function ensureLayer(): HTMLDivElement {
  if (!layer) {
    layer = document.createElement("div");
    layer.className = "ask-layer";
    layer.style.cssText = "position:fixed;z-index:56;pointer-events:none;overflow:hidden";
    document.body.appendChild(layer);
  }
  return layer;
}

function buildForm(host: AskHost, st: AskState): HTMLFormElement {
  const f = document.createElement("form");
  f.className = "ask-form";
  f.setAttribute("role", "form");
  f.setAttribute("aria-label", st.prompt || "助手提问");
  f.style.cssText =
    "position:absolute;pointer-events:auto;display:flex;flex-direction:column;gap:8px;" +
    "background:rgba(24,28,38,0.96);border:1px solid rgba(120,150,255,0.35);border-radius:12px;" +
    "padding:12px 14px;font:13px/1.5 system-ui,sans-serif;color:#e6e9f0;" +
    "box-shadow:0 6px 24px rgba(0,0,0,0.45)";

  const prompt = document.createElement("div");
  prompt.className = "ask-prompt";
  prompt.textContent = st.prompt;
  prompt.style.cssText = "font-weight:600";
  f.appendChild(prompt);

  for (let i = 0; i < st.options.length; i++) {
    const label = document.createElement("label");
    label.style.cssText = "display:flex;gap:8px;align-items:center;cursor:pointer";
    const cb = document.createElement("input");
    cb.type = "checkbox";
    cb.className = "ask-option";
    cb.value = String(i);
    const span = document.createElement("span");
    span.textContent = st.options[i];
    label.append(cb, span);
    f.appendChild(label);
  }

  const text = document.createElement("input");
  text.type = "text";
  text.className = "ask-text";
  text.placeholder = "补充说明(可选)…";
  text.style.cssText =
    "padding:6px 9px;border-radius:7px;border:1px solid rgba(255,255,255,0.15);" +
    "background:rgba(0,0,0,0.3);color:#e8e8ea;outline:none;font:13px system-ui,sans-serif";
  f.appendChild(text);

  const submit = document.createElement("button");
  submit.type = "submit";
  submit.className = "ask-submit";
  submit.textContent = "确认";
  submit.style.cssText =
    "align-self:flex-end;padding:6px 16px;border-radius:7px;border:none;cursor:pointer;" +
    "color:#fff;background:#3b6fe0;font:600 13px system-ui,sans-serif";
  f.appendChild(submit);

  f.addEventListener("submit", (e) => {
    e.preventDefault();
    const options = [...f.querySelectorAll<HTMLInputElement>(".ask-option:checked")].map((c) =>
      Number(c.value),
    );
    host.reply_question_with(JSON.stringify({ options, text: text.value.trim() }));
    // 落定卡在 core 侧同帧已渲出(状态先行);表单由下一次 pump 检测到无 pending 后撤。
  });
  return f;
}

/** 一帧:按 pending question 建/摆/撤表单(boot 的 rAF 泵调)。 */
export function pumpAskOverlay(host: AskHost): void {
  let st: AskState | null = null;
  try {
    st = JSON.parse(host.ask_state()) as AskState | null;
  } catch {
    st = null;
  }
  const pendingQ = st && st.kind === "question" ? st : null;

  // 撤:无 pending question(落定/换 permission)→ 卸表单 + 焦点还原(26② 语义沿用)。
  if (!pendingQ || (form && formForBlock !== pendingQ.block)) {
    if (form) {
      form.remove();
      form = null;
      formForBlock = -1;
      heightReported = false;
      focusedOnce = false;
      if (prevFocus?.isConnected) prevFocus.focus();
      prevFocus = null;
    }
    if (!pendingQ) return;
  }

  const root = ensureLayer();
  syncLayerToCanvas(root);

  if (!form) {
    form = buildForm(host, pendingQ);
    formForBlock = pendingQ.block;
    root.appendChild(form);
    prevFocus = document.activeElement as HTMLElement | null; // 焦点来处(提交/撤时还原)
  }

  // 摆位:每帧按块屏幕矩形(设备像素 ÷ DPR;canvas-rect 已把容器对齐画布)。
  interface R {
    x: number;
    y: number;
    w: number;
    h: number;
  }
  let rect: R | null = null;
  try {
    rect = JSON.parse(host.ask_rect()) as R | null;
  } catch {
    rect = null;
  }
  if (rect) {
    const dpr = window.devicePixelRatio || 1;
    form.style.left = `${rect.x / dpr}px`;
    form.style.top = `${rect.y / dpr}px`;
    form.style.width = `${Math.max(220, rect.w / dpr - 28)}px`;
    form.style.display = "";
    if (!focusedOnce) {
      // 首次可见才入焦(alertdialog 同款语义;display:none 期间聚焦会被浏览器丢弃)。
      focusedOnce = true;
      form.querySelector<HTMLElement>("input")?.focus();
    }
    // §3:表单实测高 > 块高 → 回报一次(core 补白,块高微调,防遮挡下文)。
    if (!heightReported) {
      const need = form.offsetHeight * dpr;
      if (need > rect.h + 4) {
        host.set_ask_height(need + 12 * dpr);
        heightReported = true;
      }
    }
  } else {
    form.style.display = "none"; // 块滚出视野:表单隐藏(FSM 仍阻塞;"跳回问题"引导属 P2)
  }
}
