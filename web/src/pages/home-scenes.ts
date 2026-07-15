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
  bloom: {
    eyebrow: "上限秀",
    title: "每个字都是一等图元",
    note: "整段对话飞起聚成一朵花 · 花瓣飘落 · 指针可拨、可点绽放。",
  },
  outro: {}, // 入口浮层接管
};

// 段框取:一律用 scroll_to(view 序号,part 级、视口无关) —— 它「脱离锚底」故能定住位置;
// pan_by 会被 follow 态每帧拉回底覆盖(H2 实测)。view 校准(see /tmp probe,settle 后读):
// 0 = 自述问,1 = 自述答,2 = 功能问,3 = 功能答("全部富内容…",其下 tool/diff/表格/公式),
// 9 = markdown 问,10 = markdown 答。scroll_to 须在 enter 末(full/preset 之后)以免被 follow 覆盖。
const V_INTRO = 0;
const V_FEATURES = 3;
const V_MARKDOWN = 9;
const frameView = (c: FilmCtx, v: number): void => c.call("scroll_to", v);
// 幕 enter **不碰 reveal 节奏**:母文档在 boot 一次性全揭示(见 home.ts),故无 cps 变化 →
// 不触发 follow 回底覆盖 scroll_to(H2 实测:full() 会让 scroll_to 被 follow 每帧拉回)。
// 流式揭示由深链 /chat/、/markdown/ 页现场演示;首页幕靠 scroll 框取 + 效果预设 + 字幕叙事。

// 七幕(时长走 token,H0 定稿)。enter 只碰引擎(scroll/pace/preset),幂等;DOM 由 home.ts 驱动。
export const HOME_SCENES: Scene[] = [
  {
    id: "title",
    title: "标题",
    durationMs: 5000,
    enter: (c) => {
      c.call("set_effect_preset", "expressive");
      frameView(c, V_INTRO);
    },
  },
  {
    id: "what",
    title: "是什么",
    durationMs: 6000,
    enter: (c) => {
      c.call("set_effect_preset", "subtle");
      frameView(c, V_INTRO);
    },
  },
  {
    id: "features",
    title: "功能",
    durationMs: 8000,
    enter: (c) => {
      c.call("set_effect_preset", "subtle");
      frameView(c, V_FEATURES);
    },
    // 段内巡礼:向下推过 tool 卡 → diff → 表格(pan 增量,相机不改基准缩放)。
    cues: [
      { at: 2600, fn: (c) => c.call("pan_by", 0, 200) },
      { at: 5200, fn: (c) => c.call("pan_by", 0, 200) },
    ],
  },
  {
    id: "conversation",
    title: "完整对话",
    durationMs: 6000,
    enter: (c) => {
      c.call("set_effect_preset", "subtle");
      frameView(c, V_INTRO); // 从对话开头看整场
    },
  },
  {
    id: "markdown",
    title: "Markdown",
    durationMs: 6000,
    enter: (c) => {
      c.call("set_effect_preset", "subtle");
      frameView(c, V_MARKDOWN);
    },
  },
  {
    id: "effects",
    title: "效果",
    durationMs: 5000,
    enter: (c) => {
      frameView(c, V_MARKDOWN);
      c.call("set_effect_preset", "off");
    },
    cues: [
      { at: 1600, fn: (c) => c.call("set_effect_preset", "subtle") },
      { at: 3200, fn: (c) => c.call("set_effect_preset", "expressive") },
    ],
  },
  {
    // 高潮幕:字阵绽放(Plan 42)。enter 只做引擎框取/预设;formation 的推进/花瓣/风场由 home.ts
    // 的 BloomController 按幕内时间驱动(需每帧,scene enter 一次性喂不了)。
    id: "bloom",
    title: "绽放",
    durationMs: 10000,
    enter: (c) => {
      c.call("set_effect_preset", "expressive");
      frameView(c, V_FEATURES); // 密集内容 → 花更满
    },
  },
  {
    id: "outro",
    title: "入口",
    durationMs: 6000,
    enter: (c) => {
      c.call("set_effect_preset", "subtle");
      frameView(c, V_INTRO);
    },
  },
];

// —— 首页母文档:开机一次性载入(instant catch-up),各幕 scroll_to 框取 ——
// 三段 assistant 回合(真富内容:自述文本 / tool 卡+diff+表格+公式 / markdown 全类型)。
// 事件形状同真 opencode(text/tool part;tool state 含 status/input/output/metadata.filediff)。
const SID = "home";
type Api = { push_event?: (raw: string) => void };

function msg(api: Api, mid: string, role: "assistant" | "user" = "assistant"): void {
  api.push_event?.(
    JSON.stringify({ type: "message.updated", properties: { info: { id: mid, role, sessionID: SID } } }),
  );
}
// 用户回合(分隔 turn:每个 user 消息起一个新回合 → scroll_to 可按 turn 框段;也让文档成真对话)。
function userTurn(api: Api, mid: string, text: string): void {
  msg(api, mid, "user");
  textPart(api, mid, `${mid}-p`, text);
}
function textPart(api: Api, mid: string, pid: string, text: string): void {
  api.push_event?.(
    JSON.stringify({
      type: "message.part.updated",
      properties: { part: { type: "text", id: pid, messageID: mid, sessionID: SID, text }, time: 1 },
    }),
  );
}
function toolPart(api: Api, mid: string, pid: string, tool: string, state: unknown): void {
  api.push_event?.(
    JSON.stringify({
      type: "message.part.updated",
      properties: { part: { type: "tool", id: pid, messageID: mid, sessionID: SID, tool, state }, time: 1 },
    }),
  );
}

export function buildMasterDoc(chat: unknown): void {
  const api = chat as Api;

  // 段 0 · 自述(S1/S2)—— 一问一答成真对话(S4 用),user 消息分隔 turn 供 scroll_to 框段。
  userTurn(api, "home-u0", "这引擎能把 LLM 对话渲染成什么样?");
  msg(api, "home-m0");
  textPart(
    api,
    "home-m0",
    "home-p0",
    "用**游戏引擎**的思路渲染 LLM 对话:Rust + WebAssembly + WebGPU,`SDF` 文字图元任意缩放锐利,无限画布,流式**揭示**不是打印——是入场。",
  );

  // 段 1 · 功能卡(S3/S4):标题 → tool 卡三态 → diff 卡 → 表格 → 公式
  userTurn(api, "home-u1", "给我看看工具卡、diff、表格和公式。");
  msg(api, "home-m1");
  textPart(api, "home-m1", "home-p1", "全部富内容都走引擎图元实时渲染:");
  toolPart(api, "home-m1", "home-t1", "bash", {
    status: "completed",
    input: { command: "npm run build" },
    output: "done in 3.2s",
  });
  toolPart(api, "home-m1", "home-t2", "bash", { status: "running", input: { command: "npm test -- --watch" } });
  toolPart(api, "home-m1", "home-t3", "edit", { status: "pending", input: { path: "src/config.ts" } });
  toolPart(api, "home-m1", "home-td", "edit", {
    status: "completed",
    input: { path: "src/auth/login.ts" },
    metadata: {
      filediff:
        "@@ -12,3 +12,4 @@\n ctx const TIMEOUT = 1000;\n-for (let i = 0; i < 5; i++) { await fetchToken({ timeout: TIMEOUT }); }\n+for (let i = 0; i < 3; i++) {\n+  await fetchToken({ timeout: TIMEOUT * 2 ** i }); // 指数退避\n+}\n",
    },
  });
  textPart(
    api,
    "home-m1",
    "home-p1b",
    "| 能力 | 手法 | 规模 |\n| --- | --- | --- |\n| 文字 | SDF / MSDF | 任意缩放锐利 |\n| 流式 | 逐字 reveal | 全程无跳变 |\n| 历史 | settled 派绘 | 100+ 轮丝滑 |\n\n行内与块级数学:$E = mc^2$\n\n$$\\sum_{i=1}^{n} i = \\frac{n(n+1)}{2}$$",
  );

  // 段 2 · markdown 全类型(S5/S6)
  userTurn(api, "home-u2", "Markdown 全类型呢?");
  msg(api, "home-m2");
  textPart(
    api,
    "home-m2",
    "home-p2",
    "## Markdown 全类型\n\n**粗**、*斜*、~~删~~、`码`、[链接](https://github.com/OhBonsai/infinite-chat)。\n\n- 无序一\n- 无序二\n  - 嵌套\n\n> [!NOTE]\n> GitHub Alert 也走 SDF 渲染,任意缩放锐利。\n\n```rust\nfn render(f: &Frame) {\n    for n in f.visible() { n.draw(); }\n}\n```",
  );
}
