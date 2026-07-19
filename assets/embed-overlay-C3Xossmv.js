import { syncLayerToCanvas as z } from "./canvas-rect-1-4iAlsh.js";
const a = /* @__PURE__ */ new Map();
function L(s, o) {
  a.set(s, o);
}
L("image", { mount(s) {
  const o = document.createElement("img");
  return o.src = s.url, o.style.cssText = "position:absolute;left:0;top:0;object-fit:contain;will-change:transform;transform-origin:0 0", o;
} });
let m = null;
const p = /* @__PURE__ */ new Map(), u = 120;
function N() {
  return m || (m = document.createElement("div"), m.style.cssText = "position:fixed;z-index:50;pointer-events:none;overflow:hidden", document.body.appendChild(m)), m;
}
const C = (s) => 1 - (1 - s) ** 3;
function V(s) {
  var _a, _b, _c;
  let o;
  try {
    o = JSON.parse(s.frame_embeds());
  } catch {
    return;
  }
  const b = N();
  z(b);
  const i = window.devicePixelRatio || 1, f = performance.now(), { px: E, py: M, zoom: d } = o.cam, g = /* @__PURE__ */ new Set();
  for (const t of o.embeds) {
    g.add(t.key);
    let e = p.get(t.key);
    if (!e) {
      const n = t.kind ?? "image", r = (a.get(n) ?? a.get("image")).mount(t);
      b.appendChild(r), e = { el: r, kind: n, baseW: t.ww / i, baseH: t.wh / i, from: [t.wx, t.wy, t.ww, t.wh], to: [t.wx, t.wy, t.ww, t.wh], t0: f - u, visible: true }, r.style.width = `${e.baseW}px`, r.style.height = `${e.baseH}px`, p.set(t.key, e);
    }
    const l = [t.wx, t.wy, t.ww, t.wh];
    if (l.some((n, c) => Math.abs(n - e.to[c]) > 0.01)) {
      const n = Math.min(1, (f - e.t0) / u), c = C(n);
      e.from = e.from.map((r, S) => r + (e.to[S] - r) * c), e.to = l, e.t0 = f;
    }
    const T = Math.min(1, (f - e.t0) / u), w = C(T), W = e.from[0] + (e.to[0] - e.from[0]) * w, $ = e.from[1] + (e.to[1] - e.from[1]) * w, H = e.from[2] + (e.to[2] - e.from[2]) * w, O = e.from[3] + (e.to[3] - e.from[3]) * w, h = (W - E) * d / i, x = ($ - M) * d / i, v = H * d / i, k = O * d / i, y = a.get(e.kind) ?? a.get("image");
    if (y.onTransform) y.onTransform(e.el, h, x, v, k);
    else {
      const n = e.baseW > 0 ? v / e.baseW : 1, c = e.baseH > 0 ? k / e.baseH : 1;
      e.el.style.transform = `translate3d(${h}px,${x}px,0) scale(${n},${c})`;
    }
    e.visible || (e.visible = true, e.el.style.display = "", (_a = y.onVisibilityChange) == null ? void 0 : _a.call(y, e.el, true));
  }
  for (const [t, e] of p) if (!g.has(t)) {
    const l = a.get(e.kind);
    (_b = l == null ? void 0 : l.onVisibilityChange) == null ? void 0 : _b.call(l, e.el, false), (_c = l == null ? void 0 : l.unmount) == null ? void 0 : _c.call(l, e.el), e.el.remove(), p.delete(t);
  }
}
export {
  V as pumpEmbedOverlay,
  L as registerEmbedRenderer
};
