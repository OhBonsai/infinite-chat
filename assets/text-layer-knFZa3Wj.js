import { syncLayerToCanvas as C } from "./canvas-rect-1-4iAlsh.js";
let p = null;
const w = /* @__PURE__ */ new Map(), b = typeof Intl < "u" && "Segmenter" in Intl ? new Intl.Segmenter(void 0, { granularity: "grapheme" }) : null;
function v(e, t) {
  if (t <= 0) return 0;
  if (!b) return Math.min(t, e.length);
  let n = 0;
  for (const a of b.segment(e)) {
    if (a.index >= t) break;
    n += 1;
  }
  return n;
}
function E(e) {
  if (p) return p;
  const t = document.createElement("style");
  return t.textContent = ".text-layer ::selection{background:transparent}.text-layer ::-moz-selection{background:transparent}", document.head.appendChild(t), p = document.createElement("div"), p.className = "text-layer", p.setAttribute("role", "log"), p.setAttribute("aria-label", "\u5BF9\u8BDD"), p.setAttribute("aria-live", "off"), p.style.cssText = "position:fixed;z-index:52;overflow:hidden;pointer-events:none;color-scheme:only light", p.addEventListener("wheel", (n) => {
    e.dispatchEvent(new WheelEvent("wheel", { deltaX: n.deltaX, deltaY: n.deltaY, ctrlKey: n.ctrlKey, clientX: n.clientX, clientY: n.clientY, bubbles: true, cancelable: true })), n.preventDefault();
  }, { passive: false }), document.body.appendChild(p), p;
}
const g = /* @__PURE__ */ new Map();
function S(e, t) {
  let n = g.get(t);
  return n || (n = document.createElement("div"), n.style.display = "contents", n.setAttribute("role", "article"), n.dataset.ablock = String(t), e.appendChild(n), g.set(t, n)), n;
}
let x = false;
function A(e, t, n) {
  if (x || !t.tap) return;
  x = true;
  let a = null;
  e.addEventListener("pointerdown", (o) => {
    o.button === 0 && (a = [o.clientX, o.clientY]);
  }), e.addEventListener("pointerup", (o) => {
    var _a;
    if (!a) return;
    const i = Math.hypot(o.clientX - a[0], o.clientY - a[1]);
    if (a = null, i > 4) return;
    const s = n.getBoundingClientRect(), c = window.devicePixelRatio || 1;
    (_a = t.tap) == null ? void 0 : _a.call(t, (o.clientX - s.left) * c, (o.clientY - s.top) * c);
  }), e.addEventListener("pointermove", (o) => {
    var _a, _b;
    const i = n.getBoundingClientRect(), s = window.devicePixelRatio || 1, c = (o.clientX - i.left) * s, r = (o.clientY - i.top) * s;
    (_a = t.set_pointer) == null ? void 0 : _a.call(t, c, r);
    const d = o.target;
    d && d !== e && (d.style.cursor = ((_b = t.link_hit_at) == null ? void 0 : _b.call(t, c, r)) ? "pointer" : "text");
  });
}
function B(e, t) {
  var _a;
  let n;
  try {
    n = JSON.parse(e.visible_text_runs());
  } catch {
    return;
  }
  const a = E(t);
  A(a, e, t), C(a);
  const o = window.devicePixelRatio || 1;
  let i = -1;
  try {
    const r = e.ask_state ? JSON.parse(e.ask_state()) : null;
    (r == null ? void 0 : r.kind) === "permission" && (i = r.block ?? -1);
  } catch {
    i = -1;
  }
  const s = /* @__PURE__ */ new Set(), c = /* @__PURE__ */ new Set();
  for (const r of n) {
    const d = `${r.block}:${r.char0}`;
    s.add(d), c.add(r.block);
    let l = w.get(d);
    l || (l = document.createElement("span"), l.dataset.block = String(r.block), l.dataset.char0 = String(r.char0), l.style.cssText = "position:absolute;white-space:pre;color:transparent;user-select:text;-webkit-user-select:text;pointer-events:auto;cursor:text;margin:0;padding:0;overflow:hidden", S(a, r.block).appendChild(l), w.set(d, l)), l.textContent !== r.text && (l.textContent = r.text), l.style.left = `${r.x / o}px`, l.style.top = `${r.y / o}px`, l.style.width = `${r.w / o}px`, l.style.height = `${r.h / o}px`, l.style.fontSize = `${r.h / o * 0.82}px`, l.style.lineHeight = `${r.h / o}px`, l.style.pointerEvents = r.block === i ? "none" : "auto";
  }
  for (const [r, d] of w) s.has(r) || (d.remove(), w.delete(r));
  for (const [r, d] of g) c.has(r) || (d.remove(), g.delete(r));
  if (T(e, a), e.visible_turns) try {
    const r = JSON.parse(e.visible_turns()), d = new Map(r.map((f) => [f.id, f.role])), l = ((_a = e.stats) == null ? void 0 : _a.call(e).retainedViews) ?? -1;
    for (const [f, y] of g) {
      const m = d.get(f);
      y.setAttribute("aria-roledescription", m === "user" ? "\u7528\u6237\u6D88\u606F" : "\u52A9\u624B\u6D88\u606F"), y.setAttribute("aria-posinset", String(f + 1)), y.setAttribute("aria-setsize", String(l));
    }
  } catch {
  }
}
function L() {
  const e = window.getSelection();
  if (!e || e.isCollapsed || e.rangeCount === 0) return new Uint32Array(0);
  const t = e.getRangeAt(0), n = /* @__PURE__ */ new Map();
  for (const [, o] of w) {
    if (!t.intersectsNode(o)) continue;
    const i = Number(o.dataset.block), s = Number(o.dataset.char0), c = o.textContent ?? "", r = o.firstChild;
    let d = 0, l = c.length;
    r && t.startContainer === r && (d = t.startOffset), r && t.endContainer === r && (l = t.endOffset);
    const f = s + v(c, d), y = s + v(c, l), m = n.get(i);
    m ? (m.start = Math.min(m.start, f), m.end = Math.max(m.end, y)) : n.set(i, { start: f, end: y });
  }
  const a = [];
  for (const [o, { start: i, end: s }] of n) s > i && a.push(o, i, s);
  return new Uint32Array(a);
}
function _(e) {
  return e.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
}
function k(e) {
  return `<div>${e.split(`
`).map((n) => `<div>${_(n) || "<br>"}</div>`).join("")}</div>`;
}
function N(e) {
  const t = navigator.clipboard;
  if (t) try {
    const n = new ClipboardItem({ "text/plain": new Blob([e], { type: "text/plain" }), "text/html": new Blob([k(e)], { type: "text/html" }) });
    t.write([n]).catch(() => void t.writeText(e).catch(() => {
    }));
  } catch {
    t.writeText(e).catch(() => {
    });
  }
}
function h(e) {
  var _a;
  if (!e || e.rangeCount === 0) return false;
  const t = e.anchorNode;
  return !!((_a = t instanceof Element ? t : t == null ? void 0 : t.parentElement) == null ? void 0 : _a.closest(".text-layer"));
}
function M(e) {
  let t = false;
  const n = () => {
    t || (t = true, requestAnimationFrame(() => {
      t = false, h(window.getSelection()) && e.set_selection(L());
    }));
  }, a = (i) => {
    var _a, _b;
    const s = window.getSelection();
    if (!h(s) || !s || s.isCollapsed) return;
    const c = s.toString();
    (_a = i.clipboardData) == null ? void 0 : _a.setData("text/plain", c), (_b = i.clipboardData) == null ? void 0 : _b.setData("text/html", k(c)), i.preventDefault();
  }, o = (i) => {
    if (!(i.ctrlKey || i.metaKey) || i.key !== "c" && i.key !== "C") return;
    const s = window.getSelection();
    !h(s) || !s || s.isCollapsed || (i.preventDefault(), N(s.toString()));
  };
  return document.addEventListener("selectionchange", n), document.addEventListener("copy", a), document.addEventListener("keydown", o), () => {
    document.removeEventListener("selectionchange", n), document.removeEventListener("copy", a), document.removeEventListener("keydown", o);
  };
}
let u = null;
function T(e, t) {
  if (!e.ask_state || !e.reply_permission) return;
  let n = false, a = "";
  try {
    const i = JSON.parse(e.ask_state());
    n = (i == null ? void 0 : i.kind) === "permission", a = (i == null ? void 0 : i.prompt) ?? "";
  } catch {
    n = false;
  }
  if (!n) {
    u == null ? void 0 : u.remove(), u = null;
    return;
  }
  if (u) return;
  u = document.createElement("div"), u.className = "ask-shadow", u.setAttribute("role", "group"), u.setAttribute("aria-label", a || "\u6743\u9650\u8BF7\u6C42"), u.style.cssText = "position:absolute;width:1px;height:1px;overflow:hidden;clip:rect(0 0 0 0);pointer-events:auto";
  const o = (i, s, c) => {
    const r = document.createElement("button");
    return r.type = "button", r.className = c, r.textContent = i, r.addEventListener("click", () => e.reply_permission(s)), r;
  };
  u.append(o("\u5141\u8BB8", true, "ask-shadow-allow"), o("\u62D2\u7EDD", false, "ask-shadow-deny")), t.appendChild(u);
}
export {
  M as attachSelection,
  B as pumpTextLayer
};
