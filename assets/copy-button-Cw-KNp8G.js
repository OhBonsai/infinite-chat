import { syncLayerToCanvas as C } from "./canvas-rect-1-4iAlsh.js";
let a = null;
const d = /* @__PURE__ */ new Map(), u = /* @__PURE__ */ new Map();
function k() {
  return a || (a = document.createElement("div"), a.style.cssText = "position:fixed;z-index:55;pointer-events:none;overflow:hidden", document.body.appendChild(a)), a;
}
function S(r, c) {
  const n = document.createElement("button");
  return n.className = "copy-btn", n.type = "button", n.textContent = "\u590D\u5236", n.style.cssText = "position:absolute;pointer-events:auto;cursor:pointer;font-size:11px;line-height:1;padding:3px 7px;border-radius:6px;border:1px solid rgba(255,255,255,0.18);background:rgba(40,44,54,0.78);color:#cdd3df;backdrop-filter:blur(4px);opacity:0;transition:opacity 0.24s ease 0.06s;will-change:transform", n.addEventListener("mouseenter", () => n.style.opacity = "1"), n.addEventListener("click", () => {
    const s = u.get(r) ?? "";
    navigator.clipboard.writeText(s).then(() => b(n, "\u5DF2\u590D\u5236 \u2713"), () => b(n, "\u590D\u5236\u5931\u8D25"));
  }), n;
}
function b(r, c) {
  r.textContent = c, r.style.opacity = "1", window.setTimeout(() => {
    r.textContent = "\u590D\u5236";
  }, 1100);
}
let y = -1, f = -1;
typeof window < "u" && window.addEventListener("pointermove", (r) => {
  y = r.clientX, f = r.clientY;
});
function L(r) {
  let c;
  try {
    c = JSON.parse(r.visible_turns());
  } catch {
    return;
  }
  const n = k();
  C(n);
  const s = window.devicePixelRatio || 1, p = /* @__PURE__ */ new Map();
  for (const t of c) {
    const o = `${t.turn}:${t.role}`, e = p.get(o);
    e ? (e.parts.push(t), e.allSettled = e.allSettled && t.settled, t.y + t.h > e.anchor.y + e.anchor.h && (e.anchor = t)) : p.set(o, { anchor: t, parts: [t], allSettled: t.settled });
  }
  const h = /* @__PURE__ */ new Set();
  for (const [t, o] of p) {
    if (!o.allSettled) continue;
    h.add(t), u.set(t, o.parts.sort((i, l) => i.id - l.id).map((i) => i.text).join(`

`));
    let e = d.get(t);
    e || (e = S(t, o.anchor.role === "user"), e.dataset.turnId = String(o.anchor.id), n.appendChild(e), d.set(t, e));
    const g = (o.anchor.x + o.anchor.w) / s, w = (o.anchor.y + o.anchor.h) / s;
    e.style.left = `${g - 52}px`, e.style.top = `${w + 4}px`;
    const x = n.getBoundingClientRect(), v = o.parts.some((i) => {
      const l = x.left + i.x / s, m = x.top + i.y / s;
      return y >= l && y <= l + i.w / s && f >= m && f <= m + i.h / s + 28;
    });
    document.activeElement !== e && (e.style.opacity = v ? "1" : "0");
  }
  for (const [t, o] of d) h.has(t) || (o.remove(), d.delete(t), u.delete(t));
}
export {
  L as pumpCopyButtons
};
