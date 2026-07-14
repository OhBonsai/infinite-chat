// home-fallback.ts — Plan 41 H3:无 GPU 降级幻灯。引擎起不来(WebGPU+WebGL2 皆无)时,
// 用 plan40 入库的 pages-assets 截图/短片替 canvas,**同幕结构、同 chrome、同字幕**。
// 幕→资产映射(每幕一张代表图/短片;S7 由入口浮层接管,无幻灯)。
const SCENE_ASSET: Record<string, string> = {
  title: "reveal-typing.webm",
  what: "canvas-zoom.png",
  features: "card-tool-states.png",
  conversation: "card-diff.png",
  markdown: "md-codeblock.png",
  effects: "fx-preset-switch.webm",
  // outro:无(入口浮层)
};

export interface FallbackSlides {
  show: (sceneId: string) => void;
}

/** 在 container 里建每幕一张幻灯(img/video 叠放,按 sceneId 切显);webm 进场自动播。 */
export function mountFallbackSlides(container: HTMLElement, base: string): FallbackSlides {
  container.style.display = "block";
  const slides = new Map<string, HTMLElement>();
  for (const [sceneId, file] of Object.entries(SCENE_ASSET)) {
    const src = `${base}pages-assets/${file}`;
    let el: HTMLElement;
    if (file.endsWith(".webm")) {
      const v = document.createElement("video");
      v.src = src;
      v.muted = true;
      v.loop = true;
      v.playsInline = true;
      v.preload = "auto";
      el = v;
    } else {
      const img = document.createElement("img");
      img.src = src;
      img.loading = "eager";
      el = img;
    }
    el.className = "fb-slide";
    container.appendChild(el);
    slides.set(sceneId, el);
  }
  return {
    show: (sceneId: string) => {
      for (const [id, el] of slides) {
        const on = id === sceneId;
        el.classList.toggle("on", on);
        if (el instanceof HTMLVideoElement) {
          if (on) void el.play().catch(() => {});
          else el.pause();
        }
      }
    },
  };
}
