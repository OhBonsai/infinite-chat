// components.ts — Plan 44 / 0042 第五页:引擎 tile 墙(Agent 卡全组件 + Markdown 全类型)。
// 单 canvas 真渲染;布局 = manifest 数据(TileSpec)驱动。默认静态成品态,hover 才播(单 tile 单 part
// 替换重播,同时 ≤1);diff 卡可点展开。无 GPU → 网格幻灯降级。清单/尺寸真值见
// spec/design/component-tile-manifest.md(T0)。
import "./pages-theme.css";
import { mountNav } from "./pages-nav";
import { bootHero } from "../boot";

mountNav("components");

// ───────────── manifest(内容写死 = R8;title=标题条 glyph,content=组件实例)─────────────
type PartSpec = { kind: "text"; text: string } | { kind: "tool"; tool: string; state: unknown };
interface TileDef {
  id: string;
  span: [number, number];
  title: string;
  content: PartSpec[];
  section?: boolean; // 区标题带(4×1,视觉分隔)
}
const text = (t: string): PartSpec => ({ kind: "text", text: t });
const tool = (name: string, state: unknown): PartSpec => ({ kind: "tool", tool: name, state });

const TILES: TileDef[] = [
  // ── 区:Agent 会话卡 ──
  { id: "sec-agent", span: [4, 1], title: "", section: true, content: [text("## Agent 会话卡 · 九组件")] },
  { id: "a-user", span: [2, 1], title: "USER", content: [text("把首页六幕重构成组件墙,tile 要 SDF 真渲染。")] },
  { id: "a-asst", span: [2, 1], title: "ASSISTANT", content: [text("已拆成 **4 列基网格** + skyline 摆位。每格是真 panel 图元,缩放锐利。")] },
  { id: "a-reason", span: [2, 1], title: "REASONING", content: [text("先确认 span 设计表 → 再写布局器 → 恒等硬门先验。")] },
  { id: "a-read", span: [2, 1], title: "TOOL · READ", content: [tool("read", { status: "running", input: { path: "crates/core/src/tilelayout.rs" } })] },
  {
    id: "a-bash",
    span: [2, 2],
    title: "TOOL · BASH",
    content: [tool("bash", { status: "completed", input: { cmd: "cargo test -p core tile" }, metadata: { output: "running 8 tests\ntest result: ok. 8 passed" } })],
  },
  {
    id: "a-ask",
    span: [2, 2],
    title: "ASK",
    content: [tool("ask", { status: "pending", metadata: { question: "允许写入 tilelayout.rs?", options: ["允许", "拒绝"] } })],
  },
  {
    id: "a-diff",
    span: [4, 3],
    title: "TOOL · EDIT",
    content: [
      tool("edit", {
        status: "completed",
        input: { path: "crates/core/src/tilelayout.rs" },
        metadata: { filediff: "@@ -40,3 +40,6 @@\n ctx let col_w = grid(w);\n-place_naive(t);\n+let row = skyline_min(cols, sw);\n+for i in col..col+sw { skyline[i] = row + sh; }\n+rect(col, row, sw, sh) // 落网格线\n" },
      }),
    ],
  },
  { id: "a-compact", span: [4, 1], title: "COMPACTION", content: [text("— 上下文已压缩 · 保留 12 条 —")] },
  { id: "a-error", span: [2, 1], title: "ERROR", content: [tool("bash", { status: "error", input: { cmd: "curl api" }, error: "APIError: rate limited (429) — 已重试 3 次" })] },

  // ── 区:Markdown 全类型 ──
  { id: "sec-md", span: [4, 1], title: "", section: true, content: [text("## Markdown 全类型 · 十九型")] },
  { id: "m-head", span: [2, 2], title: "HEADINGS", content: [text("# H1 标题\n## H2\n### H3")] },
  { id: "m-emph", span: [2, 1], title: "EMPHASIS", content: [text("**粗** · *斜* · ~~删~~ · `行内码`")] },
  { id: "m-link", span: [1, 1], title: "LINK", content: [text("[GitHub ↗](https://github.com/OhBonsai/infinite-chat)")] },
  { id: "m-emoji", span: [1, 1], title: "EMOJI", content: [text("字 🌸 混排 ✅")] },
  { id: "m-ul", span: [2, 2], title: "LIST · UL", content: [text("- 一\n- 二\n  - 嵌套")] },
  { id: "m-ol", span: [2, 2], title: "LIST · OL", content: [text("1. 一\n2. 二\n3. 三")] },
  { id: "m-task", span: [2, 1], title: "TASKS", content: [text("- [x] 已完成\n- [ ] 待办")] },
  { id: "m-quote", span: [2, 1], title: "QUOTE", content: [text("> 引擎在跑,你看到的就是宣传片本身。")] },
  { id: "m-note", span: [2, 1], title: "ALERT · NOTE", content: [text("> [!NOTE]\n> SDF 渲染,任意缩放锐利。")] },
  { id: "m-warn", span: [2, 1], title: "ALERT · WARN", content: [text("> [!WARNING]\n> 前沿留 hold 区,不提交歧义字节。")] },
  { id: "m-hr", span: [4, 1], title: "RULE", content: [text("分节线\n\n---")] },
  {
    id: "m-code",
    span: [4, 2],
    title: "CODE BLOCK",
    content: [text("```rust\nfn place(t: &Tile, sky: &mut [u32]) -> Rect {\n    let (c, row) = skyline_min(sky, t.span[0]);\n    for i in c..c + t.span[0] { sky[i] = row + t.span[1]; }\n    Rect::grid(c, row, t.span) // 四边落网格线\n}\n```")],
  },
  {
    id: "m-codefold",
    span: [2, 2],
    title: "CODE · FOLD",
    content: [text("```rust\n" + Array.from({ length: 28 }, (_, i) => `let v${i} = ${i} * col_w;`).join("\n") + "\n```")],
  },
  {
    id: "m-table",
    span: [4, 2],
    title: "TABLE",
    content: [text("| 能力 | 手法 | 规模 |\n| --- | --- | --- |\n| 文字 | SDF / MSDF | 任意缩放锐利 |\n| 流式 | 逐字 reveal | 全程无跳变 |\n| 历史 | settled 派绘 | 100+ 轮丝滑 |")],
  },
  { id: "m-foot", span: [2, 1], title: "FOOTNOTE", content: [text("正文有脚注[^1]\n\n[^1]: 脚注内容。")] },
  { id: "m-imath", span: [1, 1], title: "MATH · IN", content: [text("质能 $E=mc^2$")] },
  {
    id: "m-bmath",
    span: [2, 2],
    title: "MATH · BLOCK",
    content: [text("$$\\sum_{i=1}^{n} i = \\frac{n(n+1)}{2}$$")],
  },
  { id: "m-svg", span: [2, 1], title: "SVG", content: [text("![流程](data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' width='120' height='40'><rect x='2' y='8' width='50' height='24' rx='4' fill='none' stroke='%23c8aa6e'/><text x='27' y='25' fill='%23c8aa6e' font-size='11' text-anchor='middle'>SDF</text></svg>)")] },
];

const SID = "cm";
const GAP = 16;

interface Api {
  push_event?: (raw: string) => void;
  set_tile_spec?: (json: string) => boolean;
  tile_hit?: (sx: number, sy: number) => string;
  set_reveal_cps?: (cps: number) => void;
  tap?: (sx: number, sy: number) => boolean;
  effect_preset_name?: () => string;
}

/** 单 tile 的 part 序列:标题作首行(落 header 色带),区标题无 eyebrow 前缀。 */
function tileParts(t: TileDef): PartSpec[] {
  const head: PartSpec[] = t.section ? [] : [text(`**${t.title}**`)];
  return [...head, ...t.content];
}

function pushPart(chat: Api, mid: string, pid: string, spec: PartSpec): void {
  const part =
    spec.kind === "text"
      ? { type: "text", id: pid, messageID: mid, sessionID: SID, text: spec.text }
      : { type: "tool", id: pid, messageID: mid, sessionID: SID, tool: spec.tool, state: spec.state };
  chat.push_event?.(JSON.stringify({ type: "message.part.updated", properties: { part, time: 1 } }));
}

async function main(): Promise<void> {
  const canvas = document.getElementById("cm-canvas") as HTMLCanvasElement | null;
  if (!canvas) return;
  const { chat, ok } = await bootHero({ canvasId: "cm-canvas", glyphMode: 1 });
  if (!ok || !chat) {
    await runFallback();
    return;
  }
  const api = chat as unknown as Api;

  // 引擎就绪门(同 markdown:effect_preset_name 非空即就绪)。
  const whenReady = (tries = 0): void => {
    const nm = api.effect_preset_name;
    if (!nm || nm.call(chat) === "") {
      if (tries < 600) requestAnimationFrame(() => whenReady(tries + 1));
      return;
    }
    build();
  };

  let seq = 0;
  const liveParts = new Map<string, string[]>(); // tile id → 本 tile 现存 part ids

  function build(): void {
    api.set_reveal_cps?.(1e9); // 成品态:全揭示(hover 才降速重播)
    for (const t of TILES) {
      chat.push_event?.(
        JSON.stringify({ type: "message.updated", properties: { info: { id: t.id, role: "assistant", sessionID: SID } } }),
      );
      const ids: string[] = [];
      for (const spec of tileParts(t)) {
        const pid = `${t.id}-p${seq++}`;
        pushPart(api, t.id, pid, spec);
        ids.push(pid);
      }
      liveParts.set(t.id, ids);
    }
    // manifest → TileSpec:每 tile = 一条消息(顺序 ↔ turn),span/title 写死。
    const spec = {
      cols: 4,
      gap: GAP,
      row_h: 150,
      pad: 13,
      title_h: 26,
      tiles: TILES.map((t) => ({ id: t.id, span: t.span, title: t.title })),
    };
    const okSpec = api.set_tile_spec?.(JSON.stringify(spec));
    if (!okSpec) console.warn("[components] tile spec 被拒(退单列)");
  }

  // hover 才播:命中 tile → 重播其 content(fresh id + 删旧),同时 ≤1。
  const dpr = window.devicePixelRatio || 1;
  let animating = "";
  let idleTimer = 0;
  const replayTile = (tid: string): void => {
    const t = TILES.find((x) => x.id === tid);
    if (!t || t.section || animating === tid) return;
    animating = tid;
    api.set_reveal_cps?.(42); // 可见流式速度
    const old = liveParts.get(tid) ?? [];
    const fresh: string[] = [];
    for (const spec of t.content) {
      // 只重播 content(标题首行留静)。
      const pid = `${tid}-r${seq++}`;
      pushPart(api, tid, pid, spec);
      fresh.push(pid);
    }
    // 删旧 content part(标题 part = old[0],留;其余替换)。
    const titleId = old[0];
    for (const pid of old.slice(t.section ? 0 : 1)) {
      chat.push_event?.(
        JSON.stringify({ type: "message.part.removed", properties: { sessionID: SID, messageID: tid, partID: pid } }),
      );
    }
    liveParts.set(tid, titleId ? [titleId, ...fresh] : fresh);
    window.clearTimeout(idleTimer);
    idleTimer = window.setTimeout(() => {
      api.set_reveal_cps?.(1e9); // 收敛回成品态
      animating = "";
    }, 2600);
  };

  canvas.addEventListener("pointermove", (e) => {
    const r = canvas.getBoundingClientRect();
    const sx = (e.clientX - r.left) * dpr;
    const sy = (e.clientY - r.top) * dpr;
    const tid = api.tile_hit?.(sx, sy) ?? "";
    // 同时 ≤1 tile 在动(硬约束):仅当无 tile 播中才起新的(旧的 2.6s 后自收敛)。
    if (tid && animating === "") replayTile(tid);
  });
  // 点击:命中 diff/折叠盒 → tap(既有 fold 路);链接白名单内打开走引擎回调。
  canvas.addEventListener("click", (e) => {
    const r = canvas.getBoundingClientRect();
    api.tap?.((e.clientX - r.left) * dpr, (e.clientY - r.top) * dpr);
  });

  whenReady();
}

async function runFallback(): Promise<void> {
  const canvas = document.getElementById("cm-canvas");
  if (canvas) canvas.style.display = "none";
  const { mountComponentsFallback } = await import("./components-fallback");
  mountComponentsFallback(document.getElementById("cm-fallback") as HTMLElement, import.meta.env.BASE_URL);
}

void main();
