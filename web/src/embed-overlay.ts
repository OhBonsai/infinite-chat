// embed-overlay.ts — Plan 14 ⑥ / §2.5:动图 DOM overlay。GIF/动画 SVG **永不 settle**(每帧变)→
// 不塞 canvas 纹理逐帧重传(毁块冻结 0025 §4);改在 canvas 上叠一层 DOM `<img>`,让**浏览器原生
// 播 GIF 循环 / SVG SMIL·CSS 动画**,canvas 那块仍冻结。这是 [0022 DOM overlay] 的最小切片(仅动图)。
//
// 每帧读 `chat.frame_embeds()`(JSON,设备像素屏幕坐标)→ 按 key 复用 `<img>`、按相机定位;滚出视口
// / 卸载的元素回收。坐标 ÷ DPR 转 CSS px(canvas world 用设备像素)。

import { syncLayerToCanvas } from "./canvas-rect";

interface EmbedHost {
  frame_embeds(): string;
}

interface ScreenEmbed {
  key: string;
  url: string;
  x: number;
  y: number;
  w: number;
  h: number;
}

let layer: HTMLDivElement | null = null;
const els = new Map<string, HTMLImageElement>();

function ensureLayer(): HTMLDivElement {
  if (!layer) {
    layer = document.createElement("div");
    // 对齐画布矩形(canvas-rect)、不挡画布交互(pointer-events:none),裁掉溢出。
    // z 在画布上、输入框/面板下。left/top/width/height 由 syncLayerToCanvas 维护。
    layer.style.cssText = "position:fixed;z-index:50;pointer-events:none;overflow:hidden";
    document.body.appendChild(layer);
  }
  return layer;
}

/** 一帧:把动图嵌入的 `<img>` 同步到当前相机位置(main rAF 调)。 */
export function pumpEmbedOverlay(host: EmbedHost): void {
  let embeds: ScreenEmbed[];
  try {
    embeds = JSON.parse(host.frame_embeds()) as ScreenEmbed[];
  } catch {
    return;
  }
  const root = ensureLayer();
  syncLayerToCanvas(root);
  const dpr = window.devicePixelRatio || 1;
  const seen = new Set<string>();
  for (const e of embeds) {
    seen.add(e.key);
    let img = els.get(e.key);
    if (!img) {
      img = document.createElement("img");
      img.src = e.url; // 浏览器原生解码 + 循环播放 GIF / SVG 动画
      img.style.cssText = "position:absolute;object-fit:contain;will-change:transform";
      root.appendChild(img);
      els.set(e.key, img);
    }
    // 设备像素 → CSS 像素(÷DPR)。
    img.style.left = `${e.x / dpr}px`;
    img.style.top = `${e.y / dpr}px`;
    img.style.width = `${e.w / dpr}px`;
    img.style.height = `${e.h / dpr}px`;
  }
  // 回收滚出视口 / 卸载的元素(§7 Open③:暂直接移除,不缓存)。
  for (const [k, img] of els) {
    if (!seen.has(k)) {
      img.remove();
      els.delete(k);
    }
  }
}
