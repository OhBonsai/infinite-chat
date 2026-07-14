// chat/main.ts — Plan 25:/chat 剧本回放页入口。载剧本 → 共享装配(boot)→ driver(打字机/
// push_event/Dock 点击)→ 调度器 + 播放器 chrome。**引擎零特判**:一切内容经公开接口进引擎。
//
// URL 参数:?script=<name>(默认 showcase-full)· ?speed=<x> 倍速 · ?at=<ms> 启动即快放到该点
// (后向 seek 的着陆方式,见 player-chrome)· ?w=<px> 画布列宽(默认 600,可拖右下手柄 resize)。

import { bootCanvas } from "../boot";
import { mountNav } from "../pages/pages-nav";
import "../pages/pages-theme.css";
import { mountScriptedInput } from "../chat-input";
import { parseScript } from "./script";
import { ScriptedPlayer, type ScriptDriver } from "./scripted-player";
import { mountPlayerChrome } from "./player-chrome";

const DEFAULT_CPS = 14; // 打字机默认字速(字/秒)
const DEFAULT_HOLD_MS = 400; // 打完到"发送"的默认停顿

function fatal(msg: string): void {
  const el = document.createElement("div");
  el.style.cssText =
    "position:fixed;inset:0;display:flex;align-items:center;justify-content:center;" +
    "color:#ffb4b4;font:14px/1.6 system-ui;white-space:pre-wrap;padding:24px;";
  el.textContent = msg;
  document.body.appendChild(el);
}

async function main() {
  mountNav("chat"); // Plan 40:统一导航(金线当前态)
  const params = new URLSearchParams(location.search);
  const name = params.get("script") ?? "showcase-full";
  const speed = Number(params.get("speed") ?? "") || 1;
  const startAt = Number(params.get("at") ?? "") || 0;

  // 画布列宽:?w= 覆盖默认 600(#stage);运行中调宽走右面板滑块(下方接线),
  // 尺寸变化经 canvas-rect 桥到 wasm 重配(surface/max_width)+ overlay 跟随。
  // 必须在 boot 前设好——boot 按 clientWidth 定后备缓冲。
  const stage = document.getElementById("stage");
  const w = Number(params.get("w") ?? "");
  if (w > 0 && stage) stage.style.width = `${Math.round(w)}px`;

  // 右面板「画布」控制器:宽度滑块 ↔ #stage(初值随 ?w=)。
  {
    const slider = document.getElementById("wslider") as HTMLInputElement | null;
    const wval = document.getElementById("wval");
    if (slider && wval && stage) {
      const cur = Math.round(stage.getBoundingClientRect().width) || 600;
      slider.value = String(cur);
      wval.textContent = `${cur}px`;
      slider.addEventListener("input", () => {
        stage.style.width = `${slider.value}px`;
        wval.textContent = `${slider.value}px`;
      });
    }
  }

  // 1) 载剧本(校验失败 → 指向条目下标的可读错误)。
  const base = import.meta.env.BASE_URL;
  let rawJson: unknown;
  try {
    const r = await fetch(`${base}chats/${name}.json`);
    if (!r.ok) throw new Error(`HTTP ${r.status}`);
    rawJson = await r.json();
  } catch (e) {
    fatal(`剧本载入失败: chats/${name}.json\n${String(e)}`);
    return;
  }
  const parsed = parseScript(rawJson);
  if (!parsed.ok) {
    fatal(`剧本非法(条目 ${parsed.index}): ${parsed.error}`);
    return;
  }
  const script = parsed.script;
  if (script.meta.title) document.title = `${script.meta.title} · infinite-chat`;

  // 2) 共享装配(canvas/wasm/渲染泵/字体)。**空 replay** = 静默 Player 连接(否则无 replay/server
  //    会落到合成演示流,污染剧本内容与 FSM);剧本内容全靠 push_event 注入。
  //    Plan 25 M1:剧本页同走产品默认节奏档 reader(design §3.2)——驱动器打字机只是**到达**
  //    节奏,呈现节奏由引擎调度(节奏与 token 解耦的展示面);e2e 照旧 set_reveal_cps(1e9) 覆盖。
  const { chat } = await bootCanvas({ replay: [], rhythmPreset: "reader" });

  // Plan 26①:剧本可指定主题(themes/<name>.json 或内联 token)。set_theme 下一帧生效。
  if (script.meta.theme) {
    try {
      const t = script.meta.theme;
      const json =
        typeof t === "string"
          ? await (await fetch(`${base}themes/${t}.json`)).text()
          : JSON.stringify(t);
      chat.set_theme(json);
    } catch (e) {
      console.warn("[chat] 主题载入失败,用默认", e);
    }
  }

  // 3) 剧本模式输入框:挂进对话列(#stage)底部 → 内嵌形态(canvas 在上、输入盒在下,同宽)。
  //    canvas flex:1 自动让高;尺寸变化经 canvas-rect RO 桥到 wasm 重配。
  const input = mountScriptedInput(stage ?? document.body);

  // 4) driver:三种指令 → 真实代码路径(Plan 27:ask 走真交互 —— permission tap 真按钮矩形,
  //    question 填真 DOM 表单 —— 不直调 reply)。目标晚指令几帧出现 → 重试(~2s 放弃并告警)。
  const tapPermission = (allow: boolean, tries = 0) => {
    try {
      const targets = JSON.parse(chat.ask_hit_targets()) as {
        id: string;
        x: number;
        y: number;
        w: number;
        h: number;
      }[];
      const t = targets.find((x) => x.id === (allow ? "allow" : "deny"));
      if (t) {
        chat.tap(t.x + t.w / 2, t.y + t.h / 2); // 按钮矩形中心,真命中路径
        return;
      }
    } catch {
      /* 引擎未就绪:走重试 */
    }
    if (tries < 120) requestAnimationFrame(() => tapPermission(allow, tries + 1));
    else console.warn("[chat] permission 按钮未出现");
  };
  const fillQuestion = (options: number[], text: string, tries = 0) => {
    const form = document.querySelector<HTMLFormElement>(".ask-form");
    if (form) {
      for (const idx of options) {
        const cb = form.querySelector<HTMLInputElement>(`.ask-option[value="${idx}"]`);
        if (cb) cb.checked = true;
      }
      const input = form.querySelector<HTMLInputElement>(".ask-text");
      if (input) input.value = text;
      form.querySelector<HTMLButtonElement>(".ask-submit")?.click();
      return;
    }
    if (tries < 120) requestAnimationFrame(() => fillQuestion(options, text, tries + 1));
    else console.warn("[chat] question 表单未出现");
  };
  const driver: ScriptDriver = {
    // 返回 Promise → player 冻结时间轴直到打字+发送完成(打字门:用户气泡必然晚于发送,任何
    // 倍速/负载下顺序确定;之前 fire-and-forget 会让 note_send 落到 idle 之后,复活 FSM)。
    typeUser: async (u, instant) => {
      const cps = u.cps ?? DEFAULT_CPS;
      const hold = u.holdMs ?? DEFAULT_HOLD_MS;
      await input.typeText(u.text, cps * speed, instant);
      if (!instant) await new Promise((r) => setTimeout(r, hold / speed));
      input.flashSend();
      chat.note_send(); // 真实路径语义:发送 → FSM AwaitingAck(活跃指示)
    },
    pushEvent: (raw) => chat.push_event(raw),
    ask: (a) => {
      if (typeof a.allow === "boolean") tapPermission(a.allow);
      else fillQuestion(a.options ?? [], a.text ?? "");
    },
  };

  // 5) 调度器 + chrome + rAF 泵。
  const player = new ScriptedPlayer(script, driver);
  player.setSpeed(speed);
  // 播放控制器停靠右面板(无 #panel-player 时退回底部悬浮条,防手工打开旧壳)。
  mountPlayerChrome(player, document.getElementById("panel-player") ?? document.body);
  (window as unknown as { __player: unknown }).__player = player;

  if (startAt > 0) {
    // 后向 seek 着陆:从头**快放**到目标点 —— 到点前指令全部立即触发(user 秒填),
    // 且吐字/揭示临时不限速让积压瞬间上屏,再恢复正常节奏。
    chat.set_stream_rate(1e9);
    player.seekForward(startAt);
    setTimeout(() => chat.set_stream_rate(200), 500); // 积压清空后恢复默认吐字节奏
  }
  player.play(); // 纯自动播片:进页即播

  const loop = (now: number) => {
    player.tick(now);
    requestAnimationFrame(loop);
  };
  requestAnimationFrame(loop);

  console.info(`[chat] 剧本 "${name}" · ${script.track.length} 条指令 · ${Math.round(player.duration() / 1000)}s`);
}

void main();
