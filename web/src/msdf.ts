// msdf(0015 §2.3)— 加载离线烘焙的 BMFont MSDF:json metrics → 紧凑 typed array,
// PNG 页 → RGBA 像素,交 wasm `ChatCanvas.load_msdf`。懒加载(SDF 模式首次需要时拉)。

import type { ChatCanvas } from "../pkg/infinite_chat_wasm.js";

interface BMChar {
  id: number;
  x: number;
  y: number;
  width: number;
  height: number;
  xoffset: number;
  yoffset: number;
  xadvance: number;
  page: number;
}
interface BMFont {
  common: { scaleW: number; scaleH: number };
  info: { size: number };
  pages: string[];
  chars: BMChar[];
}

let loaded = false;
let loading: Promise<void> | null = null;

// LXGW 度量(0015 §2.5):codepoint → baked xadvance(baked 像素);layout 用它给 MSDF 命中字量宽,
// 保证"量宽字体 == 渲染字体"。bakedSize = json.info.size,用于缩放到显示尺寸。
const advances = new Map<number, number>();
let bakedSize = 0;

export function msdfLoaded(): boolean {
  return loaded;
}

/// 该 codepoint 是否在 MSDF 烘集内(coverage)。layout 与 render 共用此判定(0015 §2.5 ①)。
export function msdfHas(cp: number): boolean {
  return loaded && advances.has(cp);
}

/// MSDF 命中字的 advance(显示像素)= baked xadvance × 显示/baked;未命中返回 null(让 layout 退 measureText)。
export function msdfAdvancePx(cp: number, displayFontSize: number): number | null {
  if (!loaded || bakedSize === 0) return null;
  const a = advances.get(cp);
  return a == null ? null : (a * displayFontSize) / bakedSize;
}

/// 拉取 + 解码 + 灌入 wasm。重复调用复用同一 Promise;成功后幂等返回。
export function loadMsdf(chat: ChatCanvas, base = "/fonts/lxgw-msdf"): Promise<void> {
  if (loaded) return Promise.resolve();
  if (loading) return loading;
  const dir = base.replace(/[^/]+$/, ""); // pages 是裸文件名,相对 json 目录
  loading = (async () => {
    const font: BMFont = await fetch(`${base}.json`).then((r) => r.json());
    const chars = font.chars;
    bakedSize = font.info.size; // 供 msdfAdvancePx 缩放(0015 §2.5)
    const ids = new Uint32Array(chars.length);
    const cells = new Float32Array(chars.length * 7);
    chars.forEach((c, i) => {
      ids[i] = c.id;
      cells.set([c.x, c.y, c.width, c.height, c.xoffset, c.yoffset, c.page], i * 7);
      advances.set(c.id, c.xadvance); // LXGW 步进 → layout 量宽用(逐源一致)
    });

    const pixels: Uint8Array[] = [];
    for (const page of font.pages) {
      const blob = await fetch(`${dir}${page}`).then((r) => r.blob());
      const bmp = await createImageBitmap(blob);
      const cv = new OffscreenCanvas(bmp.width, bmp.height);
      const cx = cv.getContext("2d");
      if (!cx) throw new Error("MSDF 解码:无 2D 上下文");
      cx.drawImage(bmp, 0, 0);
      const data = cx.getImageData(0, 0, bmp.width, bmp.height).data;
      pixels.push(new Uint8Array(data.buffer.slice(0)));
    }

    chat.load_msdf({
      atlasW: font.common.scaleW,
      atlasH: font.common.scaleH,
      fontSize: font.info.size,
      ids,
      cells,
      pixels,
    });
    loaded = true;
    console.info(`[msdf] loaded ${chars.length} glyphs / ${font.pages.length} pages`);
  })();
  return loading;
}

let mathLoading: Promise<void> | null = null;
let mathLoaded = false;
export function mathMsdfLoaded(): boolean {
  return mathLoaded;
}

/// 加载 **KaTeX 数学 MSDF**(Plan 12 ④):`web/public/fonts/katex-msdf.{json,.0.png}`(合成键
/// `role*0x110000+codepoint`)灌入 wasm `load_msdf`(同 MSDF 绑定)→ 数学字形锐利缩放。**独立**于
/// 正文 lxgw MSDF(共用单绑定:加载本 atlas 则正文走 TinySDF、数学走 MSDF;两者不混键、不污染
/// 正文 `advances`)。完成后宜 `chat.refresh_fonts()` 让已出现公式重栅为 MSDF。
export function loadMathMsdf(chat: ChatCanvas, base = "/fonts/katex-msdf"): Promise<void> {
  if (mathLoaded) return Promise.resolve();
  if (mathLoading) return mathLoading;
  const dir = base.replace(/[^/]+$/, "");
  mathLoading = (async () => {
    const font: BMFont = await fetch(`${base}.json`).then((r) => r.json());
    const chars = font.chars;
    const ids = new Uint32Array(chars.length);
    const cells = new Float32Array(chars.length * 7);
    chars.forEach((c, i) => {
      ids[i] = c.id;
      cells.set([c.x, c.y, c.width, c.height, c.xoffset, c.yoffset, c.page], i * 7);
    });
    const pixels: Uint8Array[] = [];
    for (const page of font.pages) {
      const blob = await fetch(`${dir}${page}`).then((r) => r.blob());
      const bmp = await createImageBitmap(blob);
      const cv = new OffscreenCanvas(bmp.width, bmp.height);
      const cx = cv.getContext("2d");
      if (!cx) throw new Error("MSDF 解码:无 2D 上下文");
      cx.drawImage(bmp, 0, 0);
      const data = cx.getImageData(0, 0, bmp.width, bmp.height).data;
      pixels.push(new Uint8Array(data.buffer.slice(0)));
    }
    chat.load_msdf({
      atlasW: font.common.scaleW,
      atlasH: font.common.scaleH,
      fontSize: font.info.size,
      ids,
      cells,
      pixels,
    });
    mathLoaded = true;
    console.info(`[math-msdf] loaded ${chars.length} glyphs`);
  })();
  return mathLoading;
}
