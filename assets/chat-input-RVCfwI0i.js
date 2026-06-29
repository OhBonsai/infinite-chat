function w(t) {
  const i = t.indexOf("/");
  return i < 0 ? { providerID: "", modelID: t } : { providerID: t.slice(0, i), modelID: t.slice(i + 1) };
}
async function g(t, i) {
  const s = await fetch(`${t}/session`, { method: "POST", headers: { "content-type": "application/json" }, body: "{}" });
  if (!s.ok) throw new Error(`\u5EFA\u4F1A\u8BDD\u5931\u8D25 ${s.status} ${await s.text()}`);
  const n = await s.json();
  if (!n.id) throw new Error("\u5EFA\u4F1A\u8BDD\u54CD\u5E94\u7F3A id");
  return n.id;
}
const u = "ic_pending_send";
function E(t) {
  let i = t.sessionId;
  const s = document.createElement("div");
  s.style.cssText = "position:fixed;left:0;right:0;bottom:0;z-index:9000;display:flex;gap:8px;align-items:flex-end;padding:10px 12px;background:rgba(20,22,28,0.82);backdrop-filter:blur(6px);border-top:1px solid rgba(255,255,255,0.08)";
  const n = document.createElement("textarea");
  n.placeholder = "\u8F93\u5165\u6D88\u606F,Enter \u53D1\u9001 / Shift+Enter \u6362\u884C\u2026", n.rows = 1, n.style.cssText = "flex:1;resize:none;max-height:30vh;min-height:22px;padding:8px 10px;border-radius:8px;border:1px solid rgba(255,255,255,0.12);background:rgba(0,0,0,0.35);color:#e8e8ea;font:14px/1.4 system-ui,sans-serif;outline:none";
  const r = document.createElement("button");
  r.textContent = "\u53D1\u9001", r.style.cssText = "padding:8px 16px;border-radius:8px;border:none;cursor:pointer;color:#fff;background:#3b6fe0;font:600 14px system-ui,sans-serif";
  const a = document.createElement("div");
  a.style.cssText = "position:fixed;left:12px;right:12px;bottom:64px;z-index:9001;color:#ffb4b4;background:rgba(60,16,16,0.92);border:1px solid #7a2a2a;border-radius:8px;padding:8px 12px;font:13px/1.45 system-ui,sans-serif;white-space:pre-wrap;display:none";
  const l = (e) => {
    a.textContent = e, a.style.display = "block";
  }, y = () => {
    a.style.display = "none";
  }, c = () => {
    n.style.height = "auto", n.style.height = `${n.scrollHeight}px`;
  }, x = (e) => e instanceof TypeError ? `\u8FDE\u4E0D\u4E0A opencode (${t.serverUrl})\u3002\u5148\u8D77\u670D\u52A1\u7AEF:node scripts/serve.mjs,\u6216 ?server= \u6307\u5B9A\u5730\u5740\u3002` : String(e);
  let d = false;
  const p = async () => {
    const e = n.value.trim();
    if (!e || d) return;
    d = true, n.disabled = true, r.disabled = true;
    const b = r.textContent;
    if (y(), !t.canvasLive) {
      r.textContent = "\u8FDE\u63A5\u4E2D\u2026", console.info("[chat-input] \u753B\u5E03\u672A\u8FDE\u670D\u52A1\u7AEF,\u91CD\u8FDE\u540E\u7EED\u53D1", { serverUrl: t.serverUrl });
      try {
        const o = i ?? await g(t.serverUrl);
        sessionStorage.setItem(u, e);
        const f = new URL(location.href);
        f.searchParams.set("server", t.serverUrl), f.searchParams.set("session", o), location.assign(f.toString());
      } catch (o) {
        l(`\u65E0\u6CD5\u8FDE\u63A5:${x(o)}`), console.error("[chat-input] \u8FDE\u63A5\u5931\u8D25", o), d = false, n.disabled = false, r.disabled = false, r.textContent = b;
      }
      return;
    }
    r.textContent = "\u53D1\u9001\u4E2D\u2026", console.info("[chat-input] \u53D1\u9001", { serverUrl: t.serverUrl, session: i, text: e });
    try {
      i || (i = await g(t.serverUrl));
      const o = await fetch(`${t.serverUrl}/session/${i}/message`, { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify({ parts: [{ type: "text", text: e }], model: t.model }) });
      o.ok ? (n.value = "", c()) : l(`\u53D1\u9001\u5931\u8D25 ${o.status}: ${(await o.text()).slice(0, 300)}`);
    } catch (o) {
      l(`\u53D1\u9001\u5931\u8D25:${x(o)}`), console.error("[chat-input] \u53D1\u9001\u5931\u8D25", o);
    } finally {
      d = false, n.disabled = false, r.disabled = false, r.textContent = b, n.focus();
    }
  };
  n.addEventListener("input", c), n.addEventListener("keydown", (e) => {
    e.isComposing || e.keyCode === 229 || e.key === "Enter" && !e.shiftKey && (e.preventDefault(), p());
  }), r.addEventListener("click", () => void p()), s.appendChild(n), s.appendChild(r), t.parent.appendChild(s), t.parent.appendChild(a);
  let h = -1;
  const m = () => {
    const e = s.offsetHeight;
    document.documentElement.style.setProperty("--input-h", `${e}px`), e !== h && (h = e, window.dispatchEvent(new Event("resize")));
  }, v = new ResizeObserver(m);
  v.observe(s), m();
  for (const e of [300, 1200]) setTimeout(() => window.dispatchEvent(new Event("resize")), e);
  if (t.canvasLive) {
    const e = sessionStorage.getItem(u);
    e && (sessionStorage.removeItem(u), n.value = e, c(), setTimeout(() => void p(), 150));
  }
  return () => {
    v.disconnect(), document.documentElement.style.setProperty("--input-h", "0px"), window.dispatchEvent(new Event("resize")), t.parent.removeChild(s), t.parent.removeChild(a);
  };
}
export {
  g as ensureSession,
  E as mountChatInput,
  w as parseModel
};
