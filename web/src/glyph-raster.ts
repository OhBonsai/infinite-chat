// glyph-raster(M8)— 单个 grapheme → RGBA 位图(OffscreenCanvas 2D fillText)。
//
// 与 pretext-bridge 共用字号/行高;按 StyleRole 选粗/斜/等宽字体并上色(粗体看起来粗)。
// 文字用白色描边(深色画布可见,着色器再按 role 染色),emoji 自带彩色。
//
// 返回 { data: Uint8Array(RGBA), width, height },长度 = width*height*4。

import { FONT_SIZE, LINE_HEIGHT, fontForRole } from "./pretext-bridge";

export interface GlyphBitmap {
  data: Uint8Array;
  width: number;
  height: number;
}

let rasterCtx: OffscreenCanvasRenderingContext2D | null = null;

export function rasterize(cluster: string, style: number): GlyphBitmap {
  const font = fontForRole(style);
  // 用同一字体测宽,保证位图尺寸与排版一致。
  if (!rasterCtx) {
    const c = new OffscreenCanvas(8, 8).getContext("2d");
    if (!c) throw new Error("无法创建 2D 光栅化上下文");
    rasterCtx = c;
  }
  rasterCtx.font = font;
  const width = Math.max(1, Math.ceil(rasterCtx.measureText(cluster).width));
  const height = LINE_HEIGHT;

  const canvas = new OffscreenCanvas(width, height);
  const c = canvas.getContext("2d");
  if (!c) throw new Error("无法创建 2D 光栅化上下文");
  c.font = font;
  c.textBaseline = "top";
  c.fillStyle = "#ffffff";
  c.fillText(cluster, 0, FONT_SIZE * 0.1);
  const img = c.getImageData(0, 0, width, height);
  return { data: new Uint8Array(img.data.buffer), width, height };
}
