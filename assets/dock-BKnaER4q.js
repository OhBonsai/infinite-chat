let n = null, l = "";
function u() {
  return n || (n = document.createElement("div"), n.className = "session-dock", n.style.cssText = "position:fixed;left:50%;bottom:24px;transform:translateX(-50%);z-index:9997;display:none;gap:10px;align-items:center;background:rgba(28,31,40,0.96);border:1px solid rgba(255,255,255,0.18);border-radius:10px;padding:10px 14px;backdrop-filter:blur(6px);font-size:13px;color:#e6e9f0;box-shadow:0 6px 24px rgba(0,0,0,0.4)", document.body.appendChild(n), n);
}
function p(o, t) {
  const e = u();
  e.innerHTML = "";
  const r = document.createElement("span");
  r.className = "dock-label", r.textContent = t === "permission" ? "\u5DE5\u5177\u8BF7\u6C42\u6743\u9650" : "\u52A9\u624B\u6709\u4E00\u4E2A\u95EE\u9898", e.appendChild(r);
  const i = (a, d, c) => {
    const s = document.createElement("button");
    return s.type = "button", s.className = d ? "dock-allow" : "dock-deny", s.textContent = a, s.style.cssText = "cursor:pointer;font-size:12px;padding:5px 12px;border-radius:6px;border:1px solid " + (d ? "rgba(90,150,255,0.6);background:rgba(60,110,220,0.85)" : "rgba(255,255,255,0.18);background:rgba(50,54,64,0.85)") + ";color:#fff", s.addEventListener("click", c), s;
  };
  t === "permission" ? (e.appendChild(i("\u5141\u8BB8", true, () => o.reply_permission())), e.appendChild(i("\u62D2\u7EDD", false, () => o.reply_permission()))) : e.appendChild(i("\u56DE\u7B54", true, () => o.reply_question())), e.style.display = "flex";
}
function b(o) {
  const t = o.session_status(), e = t.startsWith("blocked:") ? t : "";
  e !== l && (l = e, e === "blocked:permission" ? p(o, "permission") : e === "blocked:question" ? p(o, "question") : n && (n.style.display = "none"));
}
export {
  b as pumpDock
};
