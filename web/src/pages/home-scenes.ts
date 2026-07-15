// home-scenes.ts — Plan 43:六幕 keynote(每幕一份设计画面,替换式内容见 home-slides.ts)。
// 幕表只管**时长 + 效果/节奏预设**(FilmDirector 消费);内容替换/居中/字幕/花瓣由 home.ts 按幕 id 驱动。
// 导演文档 spec/design/homepage-director-notes.md 为幕单/文案/时长真值。plan41 母文档 + scroll_to 框取术退役。
import type { Scene } from "../film/director";

/** 每幕字幕(DOM 字幕层;瘦身:eyebrow + ≤12 字标题 + 可选深链;S6 留空 → 入口浮层接管)。 */
export interface HomeSub {
  eyebrow?: string;
  title?: string;
  note?: string;
  link?: { href: string; text: string }; // 深链(base 前缀由 home.ts 补)
}

export const SUBTITLES: Record<string, HomeSub> = {
  // S1 hero:主标题在字幕层(Cinzel);引擎渲一行氛围字。
  title: {
    eyebrow: "RUST · WASM · WEBGPU · SDF",
    title: "INFINITE CHAT",
    note: "文字即图元的对话渲染引擎",
  },
  // S2 主张:引擎大字即标题 → 字幕让位,只留 eyebrow。
  claim: { eyebrow: "是什么 · WHAT" },
  // S3 证据一:引擎渲的表格自己就是主角(标题=表首行)→ 字幕只留 eyebrow + 深链,不与引擎抢标题。
  table: { eyebrow: "流式 MARKDOWN · 骨架先行", link: { href: "markdown/", text: "全类型 →" } },
  // S4 证据二:diff 卡是主角 → 字幕只留 eyebrow + chat 深链(全片唯一 chat 钩子)。
  card: { eyebrow: "AGENT 会话卡", link: { href: "chat/", text: "完整对话 →" } },
  // S5 高潮:回扣主张(与物证同框)。
  bloom: { eyebrow: "上限秀", title: "每个字都是一等图元" },
  // S6 尾幕:入口浮层接管。
  door: {},
};

// 六幕(时长承导演文档 timeline;enter 只设效果/节奏预设 —— 内容替换由 home.ts SlideDeck 驱动)。
export const HOME_SCENES: Scene[] = [
  {
    id: "title",
    title: "标题",
    durationMs: 6000,
    enter: (c) => {
      c.call("set_effect_preset", "expressive"); // dissolve-in + 微 glow
      c.call("set_reveal_preset", "reader");
    },
  },
  {
    id: "claim",
    title: "主张",
    durationMs: 6000,
    enter: (c) => {
      c.call("set_effect_preset", "subtle");
      c.call("set_reveal_preset", "reader"); // 三行按句读节奏揭示 = 流式 demo 本身
    },
  },
  {
    id: "table",
    title: "表格",
    durationMs: 7000,
    enter: (c) => {
      c.call("set_effect_preset", "subtle");
      c.call("set_reveal_preset", "reader"); // 骨架先行 = 引擎默认行为
    },
  },
  {
    id: "card",
    title: "卡片",
    durationMs: 7000,
    enter: (c) => {
      c.call("set_effect_preset", "subtle");
    },
    // diff 卡自动折叠展开一次(plan32 D4 动画);home.ts 建卡后命中折叠盒。
    cues: [
      { at: 2000, fn: (c) => c.call("tap_fold_first") },
      { at: 4500, fn: (c) => c.call("tap_fold_first") },
    ],
  },
  {
    // 高潮:字阵绽放(plan42)。内容 = claim 三行(BloomController 驱动 formation/petals/wind)。
    id: "bloom",
    title: "绽放",
    durationMs: 10000,
    enter: (c) => {
      c.call("set_effect_preset", "expressive");
    },
  },
  {
    id: "door",
    title: "入口",
    durationMs: 6000,
    enter: (c) => {
      c.call("set_effect_preset", "subtle");
    },
  },
];
