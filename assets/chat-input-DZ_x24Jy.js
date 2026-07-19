function w(t) {
  const s = t.indexOf("/");
  return s < 0 ? { providerID: "", modelID: t } : { providerID: t.slice(0, s), modelID: t.slice(s + 1) };
}
async function y(t, s) {
  const n = await fetch(`${t}/session`, { method: "POST", headers: { "content-type": "application/json" }, body: "{}" });
  if (!n.ok) throw new Error(`\u5EFA\u4F1A\u8BDD\u5931\u8D25 ${n.status} ${await n.text()}`);
  const e = await n.json();
  if (!e.id) throw new Error("\u5EFA\u4F1A\u8BDD\u54CD\u5E94\u7F3A id");
  return e.id;
}
function g(t = false) {
  const s = document.createElement("div");
  s.className = "chat-input-bar";
  const n = document.createElement("textarea");
  n.placeholder = "\u8F93\u5165\u6D88\u606F,Enter \u53D1\u9001 / Shift+Enter \u6362\u884C\u2026", n.rows = 1;
  const e = document.createElement("button");
  if (t) {
    s.style.cssText = "display:flex;flex-direction:column;gap:6px;margin:10px;padding:10px 12px 8px;background:rgba(255,255,255,0.045);border:1px solid rgba(255,255,255,0.10);border-radius:14px", n.style.cssText = "width:100%;box-sizing:border-box;resize:none;max-height:30vh;min-height:22px;padding:2px;border:none;background:transparent;color:#e8e8ea;font:14px/1.5 system-ui,sans-serif;outline:none", e.textContent = "\u2191", e.style.cssText = "width:30px;height:30px;border-radius:9px;border:none;cursor:pointer;color:#0a0e14;background:#c8aa6e;font:600 15px/1 system-ui,sans-serif";
    const o = document.createElement("div");
    o.style.cssText = "display:flex;justify-content:flex-end;align-items:center", o.appendChild(e), s.append(n, o);
  } else s.style.cssText = "position:fixed;left:0;right:0;bottom:0;z-index:9000;display:flex;gap:8px;align-items:flex-end;padding:10px 12px;background:rgba(20,22,28,0.82);backdrop-filter:blur(6px);border-top:1px solid rgba(255,255,255,0.08)", n.style.cssText = "flex:1;resize:none;max-height:30vh;min-height:22px;padding:8px 10px;border-radius:8px;border:1px solid rgba(255,255,255,0.12);background:rgba(0,0,0,0.35);color:#e8e8ea;font:14px/1.4 system-ui,sans-serif;outline:none", e.textContent = "\u53D1\u9001", e.style.cssText = "padding:8px 16px;border-radius:8px;border:none;cursor:pointer;color:#0a0e14;background:#c8aa6e;font:600 14px system-ui,sans-serif", s.append(n, e);
  return { bar: s, ta: n, btn: e };
}
function v(t) {
  let s = -1;
  const n = () => {
    const o = t.offsetHeight;
    document.documentElement.style.setProperty("--input-h", `${o}px`), o !== s && (s = o, window.dispatchEvent(new Event("resize")));
  }, e = new ResizeObserver(n);
  e.observe(t), n();
  for (const o of [300, 1200]) setTimeout(() => window.dispatchEvent(new Event("resize")), o);
  return () => {
    e.disconnect(), document.documentElement.style.setProperty("--input-h", "0px"), window.dispatchEvent(new Event("resize"));
  };
}
const m = "ic_pending_send";
function E(t) {
  let s = t.sessionId;
  const { bar: n, ta: e, btn: o } = g(), a = document.createElement("div");
  a.style.cssText = "position:fixed;left:12px;right:12px;bottom:64px;z-index:9001;color:#ffb4b4;background:rgba(60,16,16,0.92);border:1px solid #7a2a2a;border-radius:8px;padding:8px 12px;font:13px/1.45 system-ui,sans-serif;white-space:pre-wrap;display:none";
  const d = (r) => {
    a.textContent = r, a.style.display = "block";
  }, b = () => {
    a.style.display = "none";
  }, u = () => {
    e.style.height = "auto", e.style.height = `${e.scrollHeight}px`;
  }, l = (r) => r instanceof TypeError ? `\u8FDE\u4E0D\u4E0A opencode (${t.serverUrl})\u3002\u5148\u8D77\u670D\u52A1\u7AEF:node scripts/serve.mjs,\u6216 ?server= \u6307\u5B9A\u5730\u5740\u3002` : String(r);
  let c = false;
  const p = async () => {
    const r = e.value.trim();
    if (!r || c) return;
    c = true, e.disabled = true, o.disabled = true;
    const f = o.textContent;
    if (b(), !t.canvasLive) {
      o.textContent = "\u8FDE\u63A5\u4E2D\u2026", console.info("[chat-input] \u753B\u5E03\u672A\u8FDE\u670D\u52A1\u7AEF,\u91CD\u8FDE\u540E\u7EED\u53D1", { serverUrl: t.serverUrl });
      try {
        const i = s ?? await y(t.serverUrl);
        sessionStorage.setItem(m, r);
        const h = new URL(location.href);
        h.searchParams.set("server", t.serverUrl), h.searchParams.set("session", i), location.assign(h.toString());
      } catch (i) {
        d(`\u65E0\u6CD5\u8FDE\u63A5:${l(i)}`), console.error("[chat-input] \u8FDE\u63A5\u5931\u8D25", i), c = false, e.disabled = false, o.disabled = false, o.textContent = f;
      }
      return;
    }
    o.textContent = "\u53D1\u9001\u4E2D\u2026", console.info("[chat-input] \u53D1\u9001", { serverUrl: t.serverUrl, session: s, text: r });
    try {
      s || (s = await y(t.serverUrl));
      const i = await fetch(`${t.serverUrl}/session/${s}/message`, { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify({ parts: [{ type: "text", text: r }], model: t.model }) });
      i.ok ? (e.value = "", u()) : d(`\u53D1\u9001\u5931\u8D25 ${i.status}: ${(await i.text()).slice(0, 300)}`);
    } catch (i) {
      d(`\u53D1\u9001\u5931\u8D25:${l(i)}`), console.error("[chat-input] \u53D1\u9001\u5931\u8D25", i);
    } finally {
      c = false, e.disabled = false, o.disabled = false, o.textContent = f, e.focus();
    }
  };
  e.addEventListener("input", u), e.addEventListener("keydown", (r) => {
    r.isComposing || r.keyCode === 229 || r.key === "Enter" && !r.shiftKey && (r.preventDefault(), p());
  }), o.addEventListener("click", () => void p()), t.parent.appendChild(n), t.parent.appendChild(a);
  const x = v(n);
  if (t.canvasLive) {
    const r = sessionStorage.getItem(m);
    r && (sessionStorage.removeItem(m), e.value = r, u(), setTimeout(() => void p(), 150));
  }
  return () => {
    x(), t.parent.removeChild(n), t.parent.removeChild(a);
  };
}
function I(t) {
  const { bar: s, ta: n, btn: e } = g(t !== document.body);
  n.readOnly = true, n.placeholder = "", t.appendChild(s);
  const o = v(s), a = () => {
    n.style.height = "auto", n.style.height = `${n.scrollHeight}px`;
  };
  let d = 0;
  return { typeText: (l, c, p = false) => {
    if (window.clearInterval(d), p || c <= 0) return n.value = l, a(), Promise.resolve();
    const x = [...l];
    n.value = "", n.focus();
    let r = 0;
    return new Promise((f) => {
      d = window.setInterval(() => {
        r += 1, n.value = x.slice(0, r).join(""), a(), r >= x.length && (window.clearInterval(d), f());
      }, 1e3 / c);
    });
  }, flashSend: () => {
    const l = e.style.background;
    e.style.background = "#e2cd92", e.style.transform = "scale(0.94)", window.setTimeout(() => {
      e.style.background = l, e.style.transform = "", n.value = "", a();
    }, 160);
  }, unmount: () => {
    window.clearInterval(d), o(), t.removeChild(s);
  } };
}
export {
  y as ensureSession,
  E as mountChatInput,
  I as mountScriptedInput,
  w as parseModel
};
