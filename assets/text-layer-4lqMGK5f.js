let l = null;
const u = /* @__PURE__ */ new Map(), g = typeof Intl < "u" && "Segmenter" in Intl ? new Intl.Segmenter(void 0, { granularity: "grapheme" }) : null;
function x(n, t) {
  if (t <= 0) return 0;
  if (!g) return Math.min(t, n.length);
  let o = 0;
  for (const c of g.segment(n)) {
    if (c.index >= t) break;
    o += 1;
  }
  return o;
}
function v(n) {
  if (l) return l;
  const t = document.createElement("style");
  return t.textContent = ".text-layer ::selection{background:transparent}.text-layer ::-moz-selection{background:transparent}", document.head.appendChild(t), l = document.createElement("div"), l.className = "text-layer", l.style.cssText = "position:fixed;inset:0;z-index:52;overflow:hidden;pointer-events:none;color-scheme:only light", l.addEventListener("wheel", (o) => {
    n.dispatchEvent(new WheelEvent("wheel", { deltaX: o.deltaX, deltaY: o.deltaY, ctrlKey: o.ctrlKey, clientX: o.clientX, clientY: o.clientY, bubbles: true, cancelable: true })), o.preventDefault();
  }, { passive: false }), document.body.appendChild(l), l;
}
function E(n, t) {
  let o;
  try {
    o = JSON.parse(n.visible_text_runs());
  } catch {
    return;
  }
  const c = v(t), s = window.devicePixelRatio || 1, a = /* @__PURE__ */ new Set();
  for (const e of o) {
    const i = `${e.block}:${e.char0}`;
    a.add(i);
    let r = u.get(i);
    r || (r = document.createElement("span"), r.dataset.block = String(e.block), r.dataset.char0 = String(e.char0), r.style.cssText = "position:absolute;white-space:pre;color:transparent;user-select:text;-webkit-user-select:text;pointer-events:auto;cursor:text;margin:0;padding:0;overflow:hidden", c.appendChild(r), u.set(i, r)), r.textContent !== e.text && (r.textContent = e.text), r.style.left = `${e.x / s}px`, r.style.top = `${e.y / s}px`, r.style.width = `${e.w / s}px`, r.style.height = `${e.h / s}px`, r.style.fontSize = `${e.h / s * 0.82}px`, r.style.lineHeight = `${e.h / s}px`;
  }
  for (const [e, i] of u) a.has(e) || (i.remove(), u.delete(e));
}
function b() {
  const n = window.getSelection();
  if (!n || n.isCollapsed || n.rangeCount === 0) return new Uint32Array(0);
  const t = n.getRangeAt(0), o = /* @__PURE__ */ new Map();
  for (const [, s] of u) {
    if (!t.intersectsNode(s)) continue;
    const a = Number(s.dataset.block), e = Number(s.dataset.char0), i = s.textContent ?? "", r = s.firstChild;
    let f = 0, h = i.length;
    r && t.startContainer === r && (f = t.startOffset), r && t.endContainer === r && (h = t.endOffset);
    const m = e + x(i, f), y = e + x(i, h), d = o.get(a);
    d ? (d.start = Math.min(d.start, m), d.end = Math.max(d.end, y)) : o.set(a, { start: m, end: y });
  }
  const c = [];
  for (const [s, { start: a, end: e }] of o) e > a && c.push(s, a, e);
  return new Uint32Array(c);
}
function C(n) {
  return n.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
}
function w(n) {
  return `<div>${n.split(`
`).map((o) => `<div>${C(o) || "<br>"}</div>`).join("")}</div>`;
}
function k(n) {
  const t = navigator.clipboard;
  if (t) try {
    const o = new ClipboardItem({ "text/plain": new Blob([n], { type: "text/plain" }), "text/html": new Blob([w(n)], { type: "text/html" }) });
    t.write([o]).catch(() => void t.writeText(n).catch(() => {
    }));
  } catch {
    t.writeText(n).catch(() => {
    });
  }
}
function p(n) {
  var _a;
  if (!n || n.rangeCount === 0) return false;
  const t = n.anchorNode;
  return !!((_a = t instanceof Element ? t : t == null ? void 0 : t.parentElement) == null ? void 0 : _a.closest(".text-layer"));
}
function S(n) {
  let t = false;
  const o = () => {
    t || (t = true, requestAnimationFrame(() => {
      t = false, p(window.getSelection()) && n.set_selection(b());
    }));
  }, c = (a) => {
    var _a, _b;
    const e = window.getSelection();
    if (!p(e) || !e || e.isCollapsed) return;
    const i = e.toString();
    (_a = a.clipboardData) == null ? void 0 : _a.setData("text/plain", i), (_b = a.clipboardData) == null ? void 0 : _b.setData("text/html", w(i)), a.preventDefault();
  }, s = (a) => {
    if (!(a.ctrlKey || a.metaKey) || a.key !== "c" && a.key !== "C") return;
    const e = window.getSelection();
    !p(e) || !e || e.isCollapsed || (a.preventDefault(), k(e.toString()));
  };
  return document.addEventListener("selectionchange", o), document.addEventListener("copy", c), document.addEventListener("keydown", s), () => {
    document.removeEventListener("selectionchange", o), document.removeEventListener("copy", c), document.removeEventListener("keydown", s);
  };
}
export {
  S as attachSelection,
  E as pumpTextLayer
};
