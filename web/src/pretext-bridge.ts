// pretext-bridge(M7)— StyledSpan 文本 → 平铺位置数组 [x,y,w,h]*N。
//
// Plan1 用 Canvas2D measureText + Intl.Segmenter(grapheme)做单段排版,自洽可跑、零
// 字体打包(BR5,用系统字体)。pretext 是 Plan2 的排版引擎,接口不变(text+maxWidth →
// 每 grapheme 位置),届时只换本文件实现。
//
// 关键契约:**一个 grapheme 一组 [x,y,w,h]**,顺序须与 Rust 侧 unicode-segmentation 的
// grapheme 顺序一致(含换行的零宽占位),app 据此回填 spawn_time。

// HiDPI:所有几何都按设备像素算(canvas 后备缓冲也设成 设备像素,见 main.ts),
// 这样在 Retina 上 1:1 映射物理像素,文字锐利不发虚。
const DPR = typeof window !== "undefined" ? window.devicePixelRatio || 1 : 1;
const BASE_FONT_CSS_PX = 16;

export const FONT_SIZE = Math.round(BASE_FONT_CSS_PX * DPR);
export const LINE_HEIGHT = Math.ceil(FONT_SIZE * 1.4);

const SANS = `ui-sans-serif, system-ui, "PingFang SC", "Noto Sans CJK SC", sans-serif`;
const MONO = `ui-monospace, "SF Mono", Menlo, Consolas, "Noto Sans Mono CJK SC", monospace`;

/// 排版测量用的 body 字体(layout 不分角色,用 body 度量;粗/斜/code 的精确度量留 Plan2.5)。
export const FONT = `${FONT_SIZE}px ${SANS}`;

/// StyleRole 数值 → CSS 字体(与 Rust content::StyleRole as_u32 对应)。光栅化按此选字体。
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
    case 6: // Heading
      return `bold ${FONT_SIZE}px ${SANS}`;
    case 8: // Quote
      return `italic ${FONT_SIZE}px ${SANS}`;
    default: // Normal / Link / ListMarker
      return `${FONT_SIZE}px ${SANS}`;
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
  measureCtx.font = FONT;
  return measureCtx;
}

/** grapheme 宽度(px,向上取整,至少 1)。 */
export function measureCluster(cluster: string): number {
  return Math.max(1, Math.ceil(ctx().measureText(cluster).width));
}

/** 把整段文本排版成平铺 [x,y,w,h]*N(每 grapheme 一组)。 */
export function layout(text: string, maxWidth: number): Float32Array {
  const out: number[] = [];
  let x = 0;
  let y = 0;
  for (const { segment } of segmenter.segment(text)) {
    if (segment === "\n") {
      out.push(x, y, 0, LINE_HEIGHT); // 零宽占位,保持 1:1
      x = 0;
      y += LINE_HEIGHT;
      continue;
    }
    const w = measureCluster(segment);
    if (x + w > maxWidth && x > 0) {
      x = 0;
      y += LINE_HEIGHT;
    }
    out.push(x, y, w, LINE_HEIGHT);
    x += w;
  }
  return new Float32Array(out);
}
