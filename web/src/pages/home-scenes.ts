// home-scenes.ts — Plan 41 首页 film 幕表 + 字幕文案 + 母文档构建。
// 架构(见 plan41_progress H0):单份「首页母文档」上编排,幕 enter() 幂等 = scroll_to 段 +
// reveal 节奏 + 效果预设(引擎 zoom 恒 1.0,S1 dolly 走 CSS)。字幕/dolly/outro 由 home.ts 按幕 index
// 驱动(非在 enter 里碰 DOM)。内容复用 showcase 段 + plan40 markdown 样例 + 一段自述。
import type { Scene, FilmCtx } from "../film/director";

/** 每幕字幕(DOM 字幕层用;S7 留空 → 入口浮层接管)。 */
export interface HomeSub {
  eyebrow?: string;
  title?: string;
  note?: string;
  link?: { href: string; text: string }; // 深链(base 前缀由 home.ts 补)
}

export const SUBTITLES: Record<string, HomeSub> = {
  title: {
    eyebrow: "RUST · WASM · WEBGPU · SDF",
    title: "INFINITE CHAT",
    note: "用游戏引擎的思路,渲染 LLM 对话。",
  },
  what: {
    eyebrow: "是什么",
    title: "对话渲染引擎",
    note: "Rust + wasm + WebGPU · SDF 文字任意缩放锐利 · 无限画布 · 流式揭示",
  },
  features: {
    eyebrow: "功能巡礼",
    title: "工具卡 · diff · 表格 · 公式",
    note: "全部富内容都走引擎图元实时渲染。",
  },
  conversation: {
    eyebrow: "完整对话",
    title: "从头到尾,可看可玩",
    link: { href: "chat/", text: "进入完整版 →" },
  },
  markdown: {
    eyebrow: "Markdown 全类型",
    title: "流式揭示 + 实时 Playground",
    link: { href: "markdown/", text: "打开 playground →" },
  },
  effects: {
    eyebrow: "效果系统",
    title: "off → subtle → expressive",
    note: "dissolve · 微光 glow · 滚动拖尾 —— 一键换档。",
  },
  outro: {}, // 入口浮层接管
};

// 母文档分段 → view/回合序号(scroll_to 目标)。H1 三段占位,H2 换真富内容(卡/diff/表)。
export const SECTION = { intro: 0, features: 1, markdown: 2 } as const;

const reveal = (c: FilmCtx, cps: number): void => {
  c.call("set_reveal_cps", cps);
  c.call("restart_reveal");
};

// 七幕(时长走 token,H0 定稿)。enter 只碰引擎(scroll/pace/preset),幂等;DOM 由 home.ts 驱动。
export const HOME_SCENES: Scene[] = [
  {
    id: "title",
    title: "标题",
    durationMs: 5000,
    enter: (c) => {
      c.call("set_effect_preset", "expressive");
      c.call("scroll_to", SECTION.intro);
      reveal(c, 22);
    },
  },
  {
    id: "what",
    title: "是什么",
    durationMs: 6000,
    enter: (c) => {
      c.call("set_effect_preset", "subtle");
      c.call("scroll_to", SECTION.intro);
      reveal(c, 20);
    },
  },
  {
    id: "features",
    title: "功能",
    durationMs: 8000,
    enter: (c) => {
      c.call("set_effect_preset", "subtle");
      c.call("scroll_to", SECTION.features);
      reveal(c, 26);
    },
  },
  {
    id: "conversation",
    title: "完整对话",
    durationMs: 6000,
    enter: (c) => {
      c.call("set_effect_preset", "subtle");
      c.call("scroll_to", SECTION.features);
      reveal(c, 1e9); // 快放感(全显)
    },
  },
  {
    id: "markdown",
    title: "Markdown",
    durationMs: 6000,
    enter: (c) => {
      c.call("set_effect_preset", "subtle");
      c.call("scroll_to", SECTION.markdown);
      reveal(c, 22);
    },
  },
  {
    id: "effects",
    title: "效果",
    durationMs: 5000,
    enter: (c) => {
      c.call("scroll_to", SECTION.markdown);
      c.call("set_effect_preset", "off");
      reveal(c, 24);
    },
    cues: [
      { at: 1600, fn: (c) => c.call("set_effect_preset", "subtle") },
      { at: 3200, fn: (c) => c.call("set_effect_preset", "expressive") },
    ],
  },
  {
    id: "outro",
    title: "入口",
    durationMs: 6000,
    enter: (c) => {
      c.call("set_effect_preset", "subtle");
      c.call("scroll_to", SECTION.intro);
    },
  },
];

// —— 首页母文档:开机一次性载入(instant catch-up),各幕 scroll_to 框取 ——
// 三段 assistant 回合(H1 文本占位;H2 换 tool 卡/diff/表格真富内容)。事件形状同真 opencode。
const SID = "home";
function pushTurn(chat: unknown, i: number, text: string): void {
  const api = chat as { push_event?: (raw: string) => void };
  const mid = `home-m${i}`;
  api.push_event?.(
    JSON.stringify({ type: "message.updated", properties: { info: { id: mid, role: "assistant", sessionID: SID } } }),
  );
  api.push_event?.(
    JSON.stringify({
      type: "message.part.updated",
      properties: { part: { type: "text", id: `home-p${i}`, messageID: mid, sessionID: SID, text }, time: 1 },
    }),
  );
}

export function buildMasterDoc(chat: unknown): void {
  pushTurn(
    chat,
    0,
    "# INFINITE CHAT\n\n用**游戏引擎**的思路渲染 LLM 对话:Rust + WebAssembly + WebGPU,`SDF` 文字图元任意缩放锐利,无限画布,流式**揭示**不是打印——是入场。",
  );
  pushTurn(
    chat,
    1,
    "## 功能\n\n- **工具卡**四态:pending / running / done / error\n- **diff 卡**三层视觉 + 上下文折叠\n- **表格** SDF 网格与文字严丝合缝\n- 行内与块级**数学公式** $E = mc^2$",
  );
  pushTurn(
    chat,
    2,
    "## Markdown 全类型\n\n**粗**、*斜*、`码`、[链接](https://github.com/OhBonsai/infinite-chat)。\n\n> [!NOTE]\n> GitHub Alert 也走 SDF 渲染。\n\n```rust\nfn render(f: &Frame) { for n in f.visible() { n.draw(); } }\n```",
  );
}
