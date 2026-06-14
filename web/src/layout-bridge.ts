// layout-bridge(M7)— 带角色的文本 run → 平铺位置数组 [x,y,w,h]*N(每 grapheme 一组)。
//
// Canvas2D measureText + Intl.Segmenter,自洽可跑、零字体打包(用浏览器系统字体栈)。
// Plan 4A:词边界折行(拉丁不断词)+ CJK 每字可断 + 轻量禁则 + 按角色度量(粗/斜/code/标题分级)。
// 范围 = LTR(中英),不做 BiDi(0001 §2.2)。
//
// 契约:输入 runTexts[]/runRoles[](来自 Rust StyledSpan,与其 grapheme 顺序一致);
// 输出每 grapheme 一组 [x,y,w,h](含换行的零面积占位),app 据此回填 spawn_time。

const DPR = typeof window !== "undefined" ? window.devicePixelRatio || 1 : 1;
const BASE_FONT_CSS_PX = 16;

export const FONT_SIZE = Math.round(BASE_FONT_CSS_PX * DPR);
export const LINE_HEIGHT = Math.ceil(FONT_SIZE * 1.4);

// SDF tile 几何(单一来源,glyph-raster 复用;须与 Rust render::atlas::TILE_PX 一致)。
export const TILE_PX = 64;
export const SDF_BUFFER = 8;
export const FONT_PX = TILE_PX - 2 * SDF_BUFFER;

// 正文用浏览器系统字体栈(零打包)。Canvas2D 逐字形按优先级 fallback(Latin→中文→emoji)。
// 可在运行时切换字体族(调试器用,Plan 4C);切换后须 bump atlas 代 + 重排(见 ChatCanvas.refresh_fonts)。
export type FontPreset = "system" | "serif" | "rounded" | "mono";

const PRESETS: Record<FontPreset, { sans: string; mono: string }> = {
  system: {
    sans: `ui-sans-serif, system-ui, -apple-system, "Segoe UI", "PingFang SC", "Microsoft YaHei", "Noto Sans CJK SC", sans-serif`,
    mono: `ui-monospace, SFMono-Regular, "SF Mono", Menlo, Consolas, "Liberation Mono", "Noto Sans Mono CJK SC", monospace`,
  },
  serif: {
    sans: `ui-serif, Georgia, Cambria, "Times New Roman", "Songti SC", "SimSun", "Noto Serif CJK SC", serif`,
    mono: `ui-monospace, SFMono-Regular, Menlo, "Noto Sans Mono CJK SC", monospace`,
  },
  rounded: {
    sans: `"SF Pro Rounded", ui-rounded, "Hiragino Maru Gothic ProN", "Yuanti SC", system-ui, sans-serif`,
    mono: `ui-monospace, SFMono-Regular, Menlo, "Noto Sans Mono CJK SC", monospace`,
  },
  mono: {
    // 全等宽:正文也走等宽栈(终端风)。
    sans: `ui-monospace, SFMono-Regular, "SF Mono", Menlo, Consolas, "Noto Sans Mono CJK SC", monospace`,
    mono: `ui-monospace, SFMono-Regular, "SF Mono", Menlo, Consolas, "Noto Sans Mono CJK SC", monospace`,
  },
};

let active: FontPreset = "system";
let SANS = PRESETS.system.sans;
let MONO = PRESETS.system.mono;

/// 切换字体预设(layout 与 raster 同走 `fontForRole`,故一处生效两处)。返回是否变更。
export function setFontPreset(name: FontPreset): boolean {
  const p = PRESETS[name];
  if (!p || name === active) return false;
  active = name;
  SANS = p.sans;
  MONO = p.mono;
  return true;
}

/// 当前预设。
export function currentFontPreset(): FontPreset {
  return active;
}

/// 所有可选预设(供调试器枚举)。
export function fontPresets(): FontPreset[] {
  return Object.keys(PRESETS) as FontPreset[];
}

export const FONT = `${FONT_SIZE}px ${SANS}`;

/// StyleRole 数值 → CSS 字体(family + 粗细/斜体,FONT_SIZE 基准)。光栅化与度量按此选字体(4A4)。
export function fontForRole(role: number): string {
  switch (role) {
    case 1: // Bold
      return `bold ${FONT_SIZE}px ${SANS}`;
    case 2: // Italic
      return `italic ${FONT_SIZE}px ${SANS}`;
    case 3: // BoldItalic
      return `bold italic ${FONT_SIZE}px ${SANS}`;
    case 4: // Code
    case 5: // CodeBlock
      return `${FONT_SIZE}px ${MONO}`;
    case 6: // Heading (H1)
    case 10: // Heading2..6
    case 11:
    case 12:
    case 13:
    case 14:
      return `bold ${FONT_SIZE}px ${SANS}`;
    case 8: // Quote
      return `italic ${FONT_SIZE}px ${SANS}`;
    case 16: // AlertLabel(告警标签:加粗)
      return `bold ${FONT_SIZE}px ${SANS}`;
    default: // Normal / Link / ListMarker / Rule(零墨)
      return `${FONT_SIZE}px ${SANS}`;
  }
}

/// 标题分级字号倍率(4A3):H1=2.0 … H6=0.9;非标题 1.0。
function roleScale(role: number): number {
  switch (role) {
    case 6:
      return 2.0; // H1
    case 10:
      return 1.6; // H2
    case 11:
      return 1.3; // H3
    case 12:
      return 1.15; // H4
    case 13:
      return 1.0; // H5
    case 14:
      return 0.9; // H6
    default:
      return 1.0;
  }
}

const segmenter = new Intl.Segmenter(undefined, { granularity: "grapheme" });

let measureCtx: OffscreenCanvasRenderingContext2D | null = null;
function ctx(): OffscreenCanvasRenderingContext2D {
  if (!measureCtx) {
    const c = new OffscreenCanvas(8, 8).getContext("2d");
    if (!c) throw new Error("无法创建 2D 测量上下文");
    measureCtx = c;
  }
  return measureCtx;
}

/// 按角色度量 advance(px,4A4:粗/斜/code 用各自字体;标题乘分级倍率)。
function advanceFor(cluster: string, role: number): number {
  const c = ctx();
  c.font = fontForRole(role);
  return Math.max(1, c.measureText(cluster).width) * roleScale(role);
}

// CJK / 假名 / 谚文 / 全角:每字可断点。
function isCJK(ch: string): boolean {
  const cp = ch.codePointAt(0) ?? 0;
  return (
    (cp >= 0x4e00 && cp <= 0x9fff) || // CJK 统一
    (cp >= 0x3040 && cp <= 0x30ff) || // 平/片假名
    (cp >= 0xac00 && cp <= 0xd7a3) || // 谚文
    (cp >= 0xff00 && cp <= 0xffef) || // 全角
    (cp >= 0x3000 && cp <= 0x303f) // CJK 标点
  );
}
function isSpace(ch: string): boolean {
  return /\s/.test(ch);
}
// 行首禁则(不该出现在行首)→ 悬挂到上一行末(4A2 轻量)。
const HEAD_BAN = "。，、；：？！）」』】》〉”’%…·.,;:?!)]}";
function isHeadBan(ch: string): boolean {
  return HEAD_BAN.includes(ch);
}

interface G {
  cluster: string;
  role: number;
  adv: number;
  cell: number;
  off: number;
  lineH: number;
  nl: boolean;
  cjk: boolean;
  space: boolean;
}

/// 排版:runTexts[i] 文本、runRoles[i] 角色;返回每 grapheme 一组 [x,y,w,h]。
export function layout(
  runTexts: string[],
  runRoles: Uint32Array,
  maxWidth: number,
): Float32Array {
  const S = FONT_SIZE / FONT_PX; // tile px → 显示 px
  // 1) 展平成带度量的 grapheme 列表(与 Rust grapheme 顺序一致)。
  const gs: G[] = [];
  for (let r = 0; r < runTexts.length; r++) {
    const role = runRoles[r] ?? 0;
    const scale = roleScale(role);
    for (const { segment } of segmenter.segment(runTexts[r])) {
      const nl = segment === "\n";
      gs.push({
        cluster: segment,
        role,
        adv: nl ? 0 : advanceFor(segment, role),
        cell: TILE_PX * S * scale,
        off: SDF_BUFFER * S * scale,
        lineH: Math.ceil(LINE_HEIGHT * scale),
        nl,
        cjk: !nl && isCJK(segment),
        space: !nl && isSpace(segment),
      });
    }
  }

  // 2) 词级折行(拉丁不断词,CJK 每字可断,轻量禁则)。
  const out = new Float32Array(gs.length * 4);
  let penX = 0;
  let lineY = 0;
  let lineH = LINE_HEIGHT;
  const place = (g: G, idx: number) => {
    out[idx * 4] = penX - g.off;
    out[idx * 4 + 1] = lineY - g.off;
    out[idx * 4 + 2] = g.cell;
    out[idx * 4 + 3] = g.cell;
    penX += g.adv;
    if (g.lineH > lineH) lineH = g.lineH;
  };
  let i = 0;
  while (i < gs.length) {
    const g = gs[i];
    if (g.nl) {
      out[i * 4] = penX - g.off;
      out[i * 4 + 1] = lineY - g.off;
      out[i * 4 + 2] = 0;
      out[i * 4 + 3] = 0;
      penX = 0;
      lineY += lineH;
      lineH = LINE_HEIGHT;
      i++;
      continue;
    }
    // 取一个"词":CJK 单字;否则累加到空格(含)或下一个 CJK / 换行。
    let j = i;
    let wordW = 0;
    if (g.cjk) {
      j = i + 1;
      wordW = g.adv;
    } else {
      while (j < gs.length && !gs[j].nl && !gs[j].cjk) {
        wordW += gs[j].adv;
        j++;
        if (gs[j - 1].space) break; // 词以空格收尾
      }
    }
    // 折行:词溢出且不在行首;行首禁则字则悬挂(不折)。
    if (penX + wordW > maxWidth && penX > 0 && !isHeadBan(g.cluster)) {
      penX = 0;
      lineY += lineH;
      lineH = LINE_HEIGHT;
    }
    for (let k = i; k < j; k++) place(gs[k], k);
    i = j;
  }
  return out;
}
