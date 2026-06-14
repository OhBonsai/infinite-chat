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

export function msdfLoaded(): boolean {
  return loaded;
}

/// 拉取 + 解码 + 灌入 wasm。重复调用复用同一 Promise;成功后幂等返回。
export function loadMsdf(chat: ChatCanvas, base = "/fonts/lxgw-msdf"): Promise<void> {
  if (loaded) return Promise.resolve();
  if (loading) return loading;
  const dir = base.replace(/[^/]+$/, ""); // pages 是裸文件名,相对 json 目录
  loading = (async () => {
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
    loaded = true;
    console.info(`[msdf] loaded ${chars.length} glyphs / ${font.pages.length} pages`);
  })();
  return loading;
}
