import { F as v } from "./director-CgSS3eI1.js";
const w = (t) => {
  const e = Math.floor(t / 1e3);
  return `${Math.floor(e / 60)}:${String(e % 60).padStart(2, "0")}`;
};
function M(t, e = document.body) {
  const i = document.createElement("div");
  i.style.cssText = "position:fixed;left:0;right:0;bottom:0;z-index:9997;display:flex;align-items:center;gap:14px;padding:12px 18px;background:linear-gradient(#0d0f1700,#0d0f17ee);font:12px/1 'JetBrains Mono',monospace;color:#cdd3e0;user-select:none;";
  const s = document.createElement("button");
  s.style.cssText = "all:unset;cursor:pointer;width:30px;height:30px;border-radius:50%;border:1px solid #3df5d066;color:#3df5d0;text-align:center;line-height:30px;flex:0 0 auto;", s.textContent = "\u25B6";
  const a = document.createElement("div");
  a.style.cssText = "position:relative;flex:1;height:18px;cursor:pointer;display:flex;align-items:center;";
  const l = document.createElement("div");
  l.style.cssText = "position:absolute;left:0;right:0;height:3px;background:#222838;border-radius:2px;";
  const r = document.createElement("div");
  r.style.cssText = "position:absolute;left:0;height:3px;width:0;background:#3df5d0;border-radius:2px;";
  const p = document.createElement("div");
  p.style.cssText = "position:absolute;width:11px;height:11px;border-radius:50%;background:#3df5d0;left:0;transform:translateX(-50%);", a.append(l, r, p);
  for (const o of t.marks) {
    const n = document.createElement("div"), d = o.startMs / t.totalMs * 100;
    n.title = o.title, n.style.cssText = `position:absolute;left:${d}%;width:2px;height:9px;background:#7c8499;opacity:.6;transform:translateX(-50%);`, a.appendChild(n);
  }
  const c = document.createElement("span");
  c.style.cssText = "flex:0 0 auto;opacity:.7;min-width:74px;text-align:right;";
  const f = document.createElement("div");
  f.style.cssText = "display:flex;gap:6px;flex:0 0 auto;";
  const m = {};
  for (const o of [0.5, 1, 2]) {
    const n = document.createElement("button");
    n.style.cssText = "all:unset;cursor:pointer;padding:3px 7px;border-radius:5px;font-size:11px;color:#9aa3b5;border:1px solid #ffffff14;", n.textContent = `${o}\xD7`, n.onclick = () => t.setRate(o), m[o] = n, f.appendChild(n);
  }
  i.append(s, a, c, f), e.appendChild(i), s.onclick = () => t.toggle();
  const b = (o) => {
    const n = a.getBoundingClientRect(), d = Math.max(0, Math.min(1, (o.clientX - n.left) / n.width));
    t.seek(d * t.totalMs);
  };
  let x = false;
  a.addEventListener("mousedown", (o) => {
    x = true, b(o);
  }), window.addEventListener("mousemove", (o) => {
    x && b(o);
  }), window.addEventListener("mouseup", () => {
    x = false;
  }), t.onUpdate = (o) => {
    const n = o.total ? o.clock / o.total : 0;
    r.style.width = `${n * 100}%`, p.style.left = `${n * 100}%`, s.textContent = o.playing ? "\u23F8" : "\u25B6", c.textContent = `${w(o.clock)} / ${w(o.total)}`;
    for (const d of [0.5, 1, 2]) {
      const g = o.rate === d;
      m[d].style.color = g ? "#3df5d0" : "#9aa3b5", m[d].style.borderColor = g ? "#3df5d066" : "#ffffff14";
    }
  };
}
function k(t = document.body) {
  const e = document.createElement("div");
  e.style.cssText = "position:fixed;left:50%;top:46%;transform:translate(-50%,-50%) scale(0.96);z-index:9990;min-width:260px;max-width:360px;padding:18px 20px;border:1px solid #3df5d055;border-radius:12px;background:#1a1040ee;color:#e8eaf2;font:14px/1.5 system-ui,sans-serif;opacity:0;transition:opacity .35s ease,transform .35s cubic-bezier(0.34,1.56,0.64,1);pointer-events:none;", e.innerHTML = `<div style="font:700 10px/1 'JetBrains Mono',monospace;letter-spacing:.24em;color:#3df5d0;margin-bottom:8px">COMING SOON</div><div class="t-title" style="font-size:16px;font-weight:600;margin-bottom:6px"></div><div class="t-body" style="color:#bfeee2;opacity:.85"></div>`, t.appendChild(e);
  const i = e.querySelector(".t-title"), s = e.querySelector(".t-body");
  return { show(a, l) {
    i.textContent = a, s.textContent = l, e.style.opacity = "1", e.style.transform = "translate(-50%,-50%) scale(1)";
  }, hide() {
    e.style.opacity = "0", e.style.transform = "translate(-50%,-50%) scale(0.96)";
  } };
}
const h = (t) => new Promise((e) => setTimeout(e, t));
async function y(t, e, i = 900) {
  const s = window.devicePixelRatio || 1, a = window.innerWidth * s / 2, l = window.innerHeight * s * 0.42, r = 26, p = Math.pow(e, 1 / r);
  for (let c = 0; c < r; c++) t.call("zoom_at", p, a, l), await h(20);
  await h(i);
  for (let c = 0; c < r; c++) t.call("zoom_at", 1 / p, a, l), await h(20);
}
const u = (t, e, i) => {
  t.call("set_reveal_cps", e), t.call("set_reveal_slow", i), t.call("restart_reveal");
}, E = [{ id: "open", title: "\u5F00\u573A", durationMs: 4e3, enter: (t) => u(t, 12, 1) }, { id: "scale-glimpse", title: "\u89C4\u6A21\xB7\u60CA\u9E3F", durationMs: 4e3, cues: [{ at: 400, fn: (t) => void y(t, 0.5, 1400) }] }, { id: "stream", title: "\u6D41\u5F0F", durationMs: 5e3, enter: (t) => u(t, 9, 0.5), cues: [{ at: 2600, fn: (t) => u(t, 46, 1) }] }, { id: "sdf", title: "SDF\xB7\u6570\u5B66", durationMs: 5e3, enter: (t) => u(t, 28, 1), cues: [{ at: 700, fn: (t) => void y(t, 3, 1300) }] }, { id: "rich", title: "\u5BCC\u5185\u5BB9", durationMs: 5e3, enter: (t) => u(t, 24, 1) }, { id: "fx", title: "\u7279\u6548\xB7\u8DEF\u7EBF\u56FE", durationMs: 5e3, enter: (t) => t.call("set_shaderbox_gallery", true), cues: [{ at: 1600, fn: (t) => {
  t.call("set_shaderbox_gallery", false), t.teaser.show("Agent \u53D1\u5149\u8EAB\u4EFD", "assistant \u7684 glow-orb \u8EAB\u4EFD\u73AF \xB7 \u6D41\u5F0F\u65F6\u52A0\u901F\u8109\u51B2(plan16 \xA72.6)");
} }, { at: 3e3, fn: (t) => t.teaser.show("3D \u76F8\u673A \xB7 raymarch SDF", "\u5149\u6805 SDF + raymarch \u6DF7\u5408\u6E32\u67D3,\u6587\u5B57\u8D70\u8FDB\u4E09\u7EF4(\u51B3\u7B56 0024)") }, { at: 4200, fn: (t) => t.teaser.show("\u591A\u5B9E\u4F8B\u540C\u6B65 \xB7 a11y \u955C\u50CF", "\u591A\u6807\u7B7E\u9875\u72B6\u6001\u540C\u6B65(0008)\xB7 canvas \u7684\u65E0\u969C\u788D DOM \u955C\u50CF") }] }, { id: "resilient", title: "\u5BB9\u9519", durationMs: 5e3, enter: (t) => t.teaser.show("\u65AD\u7F51\u4E0D\u614C \xB7 \u5237\u65B0\u4E0D\u4E22\u5386\u53F2", "\u5F31\u7F51\u4E22\u5305:\u5DF2\u4E0A\u5C4F\u5185\u5BB9\u7EB9\u4E1D\u4E0D\u52A8;EventSource \u81EA\u52A8\u91CD\u8FDE + \u5FEB\u7167 resync \u5BF9\u8D26(\u51B3\u7B56 0003)") }, { id: "scale-peak", title: "\u89C4\u6A21\xB7\u5168\u7206\u53D1", durationMs: 4e3, cues: [{ at: 300, fn: (t) => void y(t, 0.32, 1800) }] }, { id: "outro", title: "\u6536\u5C3E \xB7 Chat", durationMs: 3e3, enter: (t) => {
  u(t, 30, 1), t.teaser.show("\u4E00\u6761\u6C38\u4E0D\u7ED3\u675F\u7684\u5BF9\u8BDD", "infinite-chat \xB7 Rust \xB7 WebAssembly \xB7 WebGPU \xB7 React/Vue import");
} }];
function T(t) {
  const e = k(), i = new v(E, t, e);
  M(i);
  let s = performance.now(), a = true;
  const l = (r) => {
    a && (i.tick(r - s), s = r, requestAnimationFrame(l));
  };
  return requestAnimationFrame((r) => {
    s = r, requestAnimationFrame(l);
  }), i.play(), () => {
    a = false;
  };
}
export {
  E as SCENES,
  T as mountFilm
};
