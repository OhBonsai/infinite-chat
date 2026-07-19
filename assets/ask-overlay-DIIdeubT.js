import { syncLayerToCanvas as y } from "./canvas-rect-1-4iAlsh.js";
let p = null, n = null, u = -1, m = false, f = false, d = null;
function b() {
  return p || (p = document.createElement("div"), p.className = "ask-layer", p.style.cssText = "position:fixed;z-index:56;pointer-events:none;overflow:hidden", document.body.appendChild(p)), p;
}
function g(a, s) {
  const e = document.createElement("form");
  e.className = "ask-form", e.setAttribute("role", "form"), e.setAttribute("aria-label", s.prompt || "\u52A9\u624B\u63D0\u95EE"), e.style.cssText = "position:absolute;pointer-events:auto;display:flex;flex-direction:column;gap:8px;background:rgba(24,28,38,0.96);border:1px solid rgba(120,150,255,0.35);border-radius:12px;padding:12px 14px;font:13px/1.5 system-ui,sans-serif;color:#e6e9f0;box-shadow:0 6px 24px rgba(0,0,0,0.45)";
  const r = document.createElement("div");
  r.className = "ask-prompt", r.textContent = s.prompt, r.style.cssText = "font-weight:600", e.appendChild(r);
  for (let l = 0; l < s.options.length; l++) {
    const c = document.createElement("label");
    c.style.cssText = "display:flex;gap:8px;align-items:center;cursor:pointer";
    const i = document.createElement("input");
    i.type = "checkbox", i.className = "ask-option", i.value = String(l);
    const x = document.createElement("span");
    x.textContent = s.options[l], c.append(i, x), e.appendChild(c);
  }
  const t = document.createElement("input");
  t.type = "text", t.className = "ask-text", t.placeholder = "\u8865\u5145\u8BF4\u660E(\u53EF\u9009)\u2026", t.style.cssText = "padding:6px 9px;border-radius:7px;border:1px solid rgba(255,255,255,0.15);background:rgba(0,0,0,0.3);color:#e8e8ea;outline:none;font:13px system-ui,sans-serif", e.appendChild(t);
  const o = document.createElement("button");
  return o.type = "submit", o.className = "ask-submit", o.textContent = "\u786E\u8BA4", o.style.cssText = "align-self:flex-end;padding:6px 16px;border-radius:7px;border:none;cursor:pointer;color:#0a0e14;background:#c8aa6e;font:600 13px system-ui,sans-serif", e.appendChild(o), e.addEventListener("submit", (l) => {
    l.preventDefault();
    const c = [...e.querySelectorAll(".ask-option:checked")].map((i) => Number(i.value));
    a.reply_question_with(JSON.stringify({ options: c, text: t.value.trim() }));
  }), e;
}
function k(a) {
  var _a;
  let s = null;
  try {
    s = JSON.parse(a.ask_state());
  } catch {
    s = null;
  }
  const e = s && s.kind === "question" ? s : null;
  if ((!e || n && u !== e.block) && (n && (n.remove(), n = null, u = -1, m = false, f = false, (d == null ? void 0 : d.isConnected) && d.focus(), d = null), !e)) return;
  const r = b();
  y(r), n || (n = g(a, e), u = e.block, r.appendChild(n), d = document.activeElement);
  let t = null;
  try {
    t = JSON.parse(a.ask_rect());
  } catch {
    t = null;
  }
  if (t) {
    const o = window.devicePixelRatio || 1;
    if (n.style.left = `${t.x / o}px`, n.style.top = `${t.y / o}px`, n.style.width = `${Math.max(220, t.w / o - 28)}px`, n.style.display = "", f || (f = true, (_a = n.querySelector("input")) == null ? void 0 : _a.focus()), !m) {
      const l = n.offsetHeight * o;
      l > t.h + 4 && (a.set_ask_height(l + 12 * o), m = true);
    }
  } else n.style.display = "none";
}
export {
  k as pumpAskOverlay
};
