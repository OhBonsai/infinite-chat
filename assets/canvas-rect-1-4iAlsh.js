let e = { left: 0, top: 0, width: 0, height: 0 }, h = 0, n = null;
function i() {
  if (!n) return;
  const t = n.getBoundingClientRect();
  (t.left !== e.left || t.top !== e.top || t.width !== e.width || t.height !== e.height) && (e = { left: t.left, top: t.top, width: t.width, height: t.height }, h += 1);
}
function d(t) {
  n = t, i();
  let r = t.clientWidth, l = t.clientHeight;
  typeof ResizeObserver < "u" && new ResizeObserver(() => {
    i(), (t.clientWidth !== r || t.clientHeight !== l) && (r = t.clientWidth, l = t.clientHeight, window.dispatchEvent(new Event("resize")));
  }).observe(t), window.addEventListener("resize", i);
}
function o(t) {
  t.dataset.rectV !== String(h) && (t.dataset.rectV = String(h), t.style.left = `${e.left}px`, t.style.top = `${e.top}px`, t.style.width = `${e.width}px`, t.style.height = `${e.height}px`);
}
export {
  o as syncLayerToCanvas,
  d as watchCanvasRect
};
