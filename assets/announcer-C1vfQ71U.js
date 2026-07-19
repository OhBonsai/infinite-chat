function p(t, e) {
  return t === e ? null : e === "awaiting" ? "\u5DF2\u53D1\u9001,\u7B49\u5F85\u56DE\u590D" : e === "streaming" ? t === "idle" || t === "awaiting" ? "\u6B63\u5728\u56DE\u590D" : null : e === "blocked:permission" ? "\u5DE5\u5177\u8BF7\u6C42\u6743\u9650,\u9700\u8981\u786E\u8BA4" : e === "blocked:question" ? "\u52A9\u624B\u6709\u4E00\u4E2A\u95EE\u9898,\u9700\u8981\u56DE\u7B54" : e === "errored" ? "\u51FA\u9519\u4E86" : e === "stopped" ? "\u5DF2\u505C\u6B62" : e === "idle" && t !== "" && t !== "idle" ? "\u56DE\u590D\u5B8C\u6210" : null;
}
function A(t) {
  return t.startsWith("blocked:") ? "assertive" : "polite";
}
function b(t, e, i, n = 1500) {
  return e === t.lastMsg && i - t.lastAt < n ? [false, t] : [true, { lastMsg: e, lastAt: i }];
}
function g(t, e = document.body) {
  const i = document.createElement("div");
  i.className = "sr-announcer", i.setAttribute("aria-live", "polite"), i.setAttribute("aria-atomic", "true"), i.style.cssText = "position:absolute;width:1px;height:1px;margin:-1px;padding:0;overflow:hidden;clip:rect(0 0 0 0);white-space:nowrap;border:0", e.appendChild(i);
  const n = document.createElement("div");
  n.className = "sr-feed", n.setAttribute("role", "log"), n.setAttribute("aria-relevant", "additions"), n.setAttribute("aria-label", "\u6D88\u606F\u64AD\u62A5"), n.style.cssText = i.style.cssText, e.appendChild(n);
  const f = 30;
  let c = "", d = { lastMsg: "", lastAt: 0 }, a = 0;
  const m = () => {
    var _a;
    const s = t.session_status(), o = p(c, s);
    if (o) {
      const [l, r] = b(d, o, performance.now());
      l && (d = r, i.setAttribute("aria-live", A(s)), i.textContent = o);
    }
    if (c = s, n.setAttribute("aria-busy", s === "streaming" ? "true" : "false"), t.take_settle_announcements) try {
      const l = JSON.parse(t.take_settle_announcements());
      for (const r of l) {
        if (!r.text.trim()) continue;
        const u = document.createElement("p");
        for (u.dataset.role = r.role, u.textContent = r.text, n.appendChild(u); n.childElementCount > f; ) (_a = n.firstElementChild) == null ? void 0 : _a.remove();
      }
    } catch {
    }
    a = requestAnimationFrame(m);
  };
  return a = requestAnimationFrame(m), () => {
    cancelAnimationFrame(a), i.remove(), n.remove();
  };
}
export {
  A as livenessOf,
  g as mountAnnouncer,
  b as shouldEmit,
  p as statusAnnouncement
};
