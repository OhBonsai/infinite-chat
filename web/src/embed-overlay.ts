// embed-overlay.ts — 世界定位 DOM 叠加层(0022;Plan 14 最小切片 → Plan 31 R3 升级)。
//
// 三层分工(0022 §3 / 0007 §3,world = 设备 px 共享相机):
// - **世界矩形**(布局产物):变化走 0016 同款补间(120ms cubic-out,JS 侧对世界坐标 lerp——
//   与 core Scene 同式;差异记 progress:FrameEmbed 不经 Scene,视觉等效);
// - **相机**(pan/zoom):每帧直乘,零过渡(否则快滚跟随迟滞 = 脱节);
// - **定位**:`transform: translate3d + scale`(GPU 合成,0022 §47),不再 left/top 触发重排。
//
// EmbedKind 宿主渲染器注册表(0022 §89):`registerEmbedRenderer(kind, r)`;内置 "image"
// (动图 <img>,浏览器原生播 GIF/SVG 动画,canvas 块保持冻结)。钩子:mount/transform/
// visibility/unmount。未知 kind 兜底 "image"(AR12)。

import { syncLayerToCanvas } from "./canvas-rect";

interface EmbedHost {
  frame_embeds(): string;
}

interface WireEmbed {
  key: string;
  url: string;
  kind?: string;
  wx: number;
  wy: number;
  ww: number;
  wh: number;
}

interface WirePayload {
  cam: { px: number; py: number; zoom: number };
  embeds: WireEmbed[];
}

/** 宿主渲染器(0022 §89 钩子)。transform 缺省由框架设 translate3d+scale。 */
export interface EmbedRenderer {
  mount(e: WireEmbed): HTMLElement;
  onTransform?(el: HTMLElement, cssX: number, cssY: number, cssW: number, cssH: number): void;
  onVisibilityChange?(el: HTMLElement, visible: boolean): void;
  unmount?(el: HTMLElement): void;
}

const renderers = new Map<string, EmbedRenderer>();

/** 注册某 EmbedKind 的宿主渲染器(0022 §89;重复注册覆盖)。 */
export function registerEmbedRenderer(kind: string, r: EmbedRenderer): void {
  renderers.set(kind, r);
}

// 内置:动图 <img>(Plan 14 语义)。基准尺寸取首帧,后续尺寸变化经 scale(免重排)。
registerEmbedRenderer("image", {
  mount(e) {
    const img = document.createElement("img");
    img.src = e.url;
    img.style.cssText =
      "position:absolute;left:0;top:0;object-fit:contain;will-change:transform;transform-origin:0 0";
    return img;
  },
});

interface Live {
  el: HTMLElement;
  kind: string;
  baseW: number; // mount 时 CSS 基准宽高(transform scale 相对它)
  baseH: number;
  // 0016 补间态(世界坐标):布局变化时 from→to,相机不参与。
  from: [number, number, number, number];
  to: [number, number, number, number];
  t0: number; // 补间起点 ms(performance.now)
  visible: boolean;
}

let layer: HTMLDivElement | null = null;
const els = new Map<string, Live>();
const TWEEN_MS = 120; // 0016 同 dur

function ensureLayer(): HTMLDivElement {
  if (!layer) {
    layer = document.createElement("div");
    layer.style.cssText = "position:fixed;z-index:50;pointer-events:none;overflow:hidden";
    document.body.appendChild(layer);
  }
  return layer;
}

const easeCubicOut = (t: number) => 1 - (1 - t) ** 3;

/** 一帧:世界矩形补间 + 相机直乘 + transform 定位(main rAF 调)。 */
export function pumpEmbedOverlay(host: EmbedHost): void {
  let payload: WirePayload;
  try {
    payload = JSON.parse(host.frame_embeds()) as WirePayload;
  } catch {
    return;
  }
  const root = ensureLayer();
  syncLayerToCanvas(root);
  const dpr = window.devicePixelRatio || 1;
  const now = performance.now();
  const { px, py, zoom } = payload.cam;
  const seen = new Set<string>();
  for (const e of payload.embeds) {
    seen.add(e.key);
    let live = els.get(e.key);
    if (!live) {
      const kind = e.kind ?? "image";
      const r = renderers.get(kind) ?? renderers.get("image")!;
      const el = r.mount(e);
      root.appendChild(el);
      live = {
        el,
        kind,
        baseW: e.ww / dpr,
        baseH: e.wh / dpr,
        from: [e.wx, e.wy, e.ww, e.wh],
        to: [e.wx, e.wy, e.ww, e.wh],
        t0: now - TWEEN_MS, // 初始即到位
        visible: true,
      };
      el.style.width = `${live.baseW}px`;
      el.style.height = `${live.baseH}px`;
      els.set(e.key, live);
    }
    // 世界矩形变化 → 起补间(from = 当前显示态,可打断不回跳,0016 §4.4)。
    const target: [number, number, number, number] = [e.wx, e.wy, e.ww, e.wh];
    if (target.some((v, i) => Math.abs(v - live.to[i]) > 0.01)) {
      const t = Math.min(1, (now - live.t0) / TWEEN_MS);
      const k = easeCubicOut(t);
      live.from = live.from.map((f, i) => f + (live.to[i] - f) * k) as typeof live.from;
      live.to = target;
      live.t0 = now;
    }
    const t = Math.min(1, (now - live.t0) / TWEEN_MS);
    const k = easeCubicOut(t);
    const wx = live.from[0] + (live.to[0] - live.from[0]) * k;
    const wy = live.from[1] + (live.to[1] - live.from[1]) * k;
    const ww = live.from[2] + (live.to[2] - live.from[2]) * k;
    const wh = live.from[3] + (live.to[3] - live.from[3]) * k;
    // 相机直乘(零过渡)→ CSS px;transform 定位(GPU 合成)。
    const cssX = ((wx - px) * zoom) / dpr;
    const cssY = ((wy - py) * zoom) / dpr;
    const cssW = (ww * zoom) / dpr;
    const cssH = (wh * zoom) / dpr;
    const r = renderers.get(live.kind) ?? renderers.get("image")!;
    if (r.onTransform) {
      r.onTransform(live.el, cssX, cssY, cssW, cssH);
    } else {
      const sx = live.baseW > 0 ? cssW / live.baseW : 1;
      const sy = live.baseH > 0 ? cssH / live.baseH : 1;
      live.el.style.transform = `translate3d(${cssX}px,${cssY}px,0) scale(${sx},${sy})`;
    }
    if (!live.visible) {
      live.visible = true;
      live.el.style.display = "";
      r.onVisibilityChange?.(live.el, true);
    }
  }
  // 视口外/卸载:回收元素(0022 §84;display:none 缓存策略待需求)。
  for (const [k, live] of els) {
    if (!seen.has(k)) {
      const r = renderers.get(live.kind);
      r?.onVisibilityChange?.(live.el, false);
      r?.unmount?.(live.el);
      live.el.remove();
      els.delete(k);
    }
  }
}
