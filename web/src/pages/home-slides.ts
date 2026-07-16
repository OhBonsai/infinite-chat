// home-slides.ts — Plan 43:替换式画面内容(废 plan41 母文档)。每幕 = 一份**设计过的画面**;
// 幕切 = 画布内容替换(先 upsert 本幕新 id → 再 remove 现存)。幂等:任意直跳 → 画布只余本幕内容,
// 终态一致(DoD-2)。居中/放大交引擎(set_focus_valign 垂直 + 实际墨迹水平居中 + per-幕 set_focus_zoom),
// host 侧不再 pan 追帧(免 layout 竞态)。导演文档:spec/design/homepage-director-notes.md(六镜四问 + 文案真值)。

interface Api {
  push_event?: (raw: string) => void;
  set_focus_valign?: (frac: number) => void;
  set_focus_zoom?: (z: number) => void;
}

// 每幕 keynote 缩放(引擎大字/表格放大成主角;宽内容小放大免溢屏,bloom 保 1.0 交 plan42 编队框取)。
const ZOOM: Record<string, number> = {
  title: 1.0, // S1 hero 是 DOM 大标题,引擎只渲氛围行 → 不放大
  claim: 1.45, // S2 三行短大字 → 放大成主角
  table: 1.18, // S3 表格宽,小放大免横向溢屏
  card: 1.12, // S4 diff 卡
  bloom: 1.0, // S5 字阵编队自带世界空间框取
  door: 1.0, // S6 尾幕 DOM 入口浮层承载
};

const SID = "home";
const MID = "home-slide"; // 单 assistant 消息,parts 随幕替换

/** 一片 part 规格:文本(markdown)或 tool 卡。 */
type PartSpec = { kind: "text"; text: string } | { kind: "tool"; tool: string; state: unknown };

// 六幕内容(文案写死 = 确定性 R8;导演文档为真值)。密度:每幕一个焦点、大字、少字。
export const SLIDES: Record<string, PartSpec[]> = {
  // S1 hero:引擎渲一行氛围短诗(≤14 字);DOM 字幕层出主标题 INFINITE CHAT。
  title: [{ kind: "text", text: "# 文字,是活的图元。" }],
  // S2 主张:三行大字(单 part 三行 → 可见 part=1,连续流式;引擎亲口说世界观)。
  claim: [{ kind: "text", text: "# 每个字都是一等图元\n\n## GPU 上的 SDF 实例 —— 可揭示 · 可调度 · 可飞行\n\n## fps 与内存,只随可见的一屏" }],
  // S3 证据一:单块流式 markdown(表格 + 行内公式),骨架先行。
  table: [
    {
      kind: "text",
      text: "## 先骨架,再文字\n\n| 能力 | 手法 | 规模 |\n| --- | --- | --- |\n| 文字 | SDF / MSDF | 任意缩放锐利 |\n| 流式 | 逐字 reveal | 全程无跳变 |\n| 历史 | settled 派绘 | 100+ 轮丝滑 |\n\n质能方程 $E = mc^2$",
    },
  ],
  // S4 证据二:单张 diff 卡(edit 工具 + filediff),折叠展开一次(cue 在 home.ts)。
  card: [
    {
      kind: "tool",
      tool: "edit",
      state: {
        status: "completed",
        input: { path: "src/auth/login.ts" },
        metadata: {
          filediff:
            "@@ -12,3 +12,4 @@\n ctx const TIMEOUT = 1000;\n-for (let i = 0; i < 5; i++) { await fetchToken({ timeout: TIMEOUT }); }\n+for (let i = 0; i < 3; i++) {\n+  await fetchToken({ timeout: TIMEOUT * 2 ** i }); // 指数退避\n+}\n",
        },
      },
    },
  ],
  // S5 高潮:复用 S2 三行大字(密集 → 花更满);BloomController 做编队。单 part 三行,同 claim。
  bloom: [{ kind: "text", text: "# 每个字都是一等图元\n\n## GPU 上的 SDF 实例 —— 可揭示 · 可调度 · 可飞行\n\n## fps 与内存,只随可见的一屏" }],
  // S6 尾幕:画布静场(引擎内容清空,免与 DOM 入口四卡浮层重叠);花瓣余韵飘落 + 浮层承载全部。
  door: [],
};

/** 替换式内容台:build(sceneId) 清旧 part + 推本幕 part + 居中框取。 */
export class SlideDeck {
  private live: string[] = [];
  private seq = 0;
  private built = false;

  constructor(private chat: Api) {
    // 幕式短内容垂直居中到焦点带(略高于正中);引擎默认顶锚,只此 keynote 开(见 app.rs focus_valign)。
    this.chat.set_focus_valign?.(0.42);
  }

  build(sceneId: string): void {
    if (!this.built) {
      // 建一次 assistant 消息(parts 随幕替换,不再建新消息)。
      this.chat.push_event?.(
        JSON.stringify({ type: "message.updated", properties: { info: { id: MID, role: "assistant", sessionID: SID } } }),
      );
      this.built = true;
    }
    // 顺序承 markdown.ts 已验证替换术:**先推新 part,再删旧 part**(始终≥1 part,消息不被清空塌陷)。
    const stale = this.live;
    // 1) 推本幕 part(fresh id → 干净重播;plan42 坑:同 id upsert 沿用已揭示态、不重播)。
    const specs = SLIDES[sceneId] ?? [];
    const fresh: string[] = [];
    for (const spec of specs) {
      const pid = `s${this.seq++}`;
      const part =
        spec.kind === "text"
          ? { type: "text", id: pid, messageID: MID, sessionID: SID, text: spec.text }
          : { type: "tool", id: pid, messageID: MID, sessionID: SID, tool: spec.tool, state: spec.state };
      this.chat.push_event?.(
        JSON.stringify({ type: "message.part.updated", properties: { part, time: 1 } }),
      );
      fresh.push(pid);
    }
    // 2) 删旧 part(→ 画布只余本幕内容 = 幂等终态 DoD-2)。
    for (const pid of stale) {
      this.chat.push_event?.(
        JSON.stringify({
          type: "message.part.removed",
          properties: { sessionID: SID, messageID: MID, partID: pid },
        }),
      );
    }
    this.live = fresh;
    // 放大成幕主角(绝对缩放,幂等);垂直居中由引擎 focus_valign 每帧兜(免 host pan 追帧竞态)。
    this.chat.set_focus_zoom?.(ZOOM[sceneId] ?? 1.0);
  }
}
