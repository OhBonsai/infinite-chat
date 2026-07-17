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
  // S3 证据一(Plan 44 T3:Markdown 墙局部框取)→ 字幕深链改指全量组件墙。
  table: { eyebrow: "MARKDOWN 墙 · 局部", link: { href: "components/", text: "全部组件 →" } },
  // S4 证据二(Plan 44 T3:Agent 卡墙局部,diff 自动展开)→ 深链指全量组件墙。
  card: { eyebrow: "AGENT 卡墙 · 局部", link: { href: "components/", text: "全部组件 →" } },
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
      // keynote:三行大字**明快流式**(typewriter + 38cps)—— reader 5.5cps 一屏内揭不完,反失焦。
      c.call("set_reveal_preset", "typewriter");
      c.call("set_reveal_cps", 38);
    },
  },
  {
    id: "table",
    title: "表格",
    durationMs: 7000,
    enter: (c) => {
      c.call("set_effect_preset", "subtle");
      c.call("set_reveal_preset", "typewriter"); // 骨架先行 + 明快揭示(表 80 字须 ~2s 内落屏)
      c.call("set_reveal_cps", 44);
    },
  },
  {
    id: "card",
    title: "卡片",
    durationMs: 7000,
    enter: (c) => {
      c.call("set_effect_preset", "subtle");
      c.call("set_reveal_preset", "typewriter");
      c.call("set_reveal_cps", 60); // diff 卡文本多 → 更快落屏,留时间给折叠动画
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
