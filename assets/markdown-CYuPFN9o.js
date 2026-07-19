import { e as I } from "./boot-B4nH8qNC.js";
import { m as D } from "./pages-nav-BiSBOohm.js";
D("markdown");
const v = [{ label: "\u884C\u5185\u4E0E\u5F3A\u8C03", md: `# \u884C\u5185\u6837\u5F0F

**\u7C97\u4F53**\u3001*\u659C\u4F53*\u3001~~\u5220\u9664\u7EBF~~\u3001\`\u884C\u5185\u4EE3\u7801\`,\u4EE5\u53CA [\u94FE\u63A5](https://github.com/OhBonsai/infinite-chat)\u3002

\u4E2D\u82F1\u6DF7\u6392\u4E5F\u4E25\u4E1D\u5408\u7F1D:SDF \u6587\u5B57\u56FE\u5143,\u4EFB\u610F\u7F29\u653E\u9510\u5229\u3002` }, { label: "\u5217\u8868\u4E0E\u4EFB\u52A1", md: `# \u5217\u8868\u4E0E\u4EFB\u52A1\u6846

- \u65E0\u5E8F\u4E00
- \u65E0\u5E8F\u4E8C
  - \u5D4C\u5957\u5B50\u9879

1. \u6709\u5E8F\u4E00
2. \u6709\u5E8F\u4E8C

- [x] \u5DF2\u5B8C\u6210\u7684\u4EFB\u52A1
- [ ] \u5F85\u529E\u4EFB\u52A1` }, { label: "\u4EE3\u7801\u5757", md: `# \u4EE3\u7801\u5757(\u8BED\u6CD5\u9AD8\u4EAE)

\`\`\`rust
fn render(frame: &Frame) {
    for node in frame.visible() {
        node.draw(); // GPU \u5B9E\u4F8B\u5316 + \u89C6\u53E3\u88C1\u526A
    }
}
\`\`\`

\u8D85\u957F\u4EE3\u7801\u6298\u53E0\u6210\u53EF\u6EDA\u52A8\u884C\u7A97,\u53F3\u4E0A\u89D2\u6709 copy \u6309\u94AE\u3002` }, { label: "\u8868\u683C", md: `# \u8868\u683C(SDF \u7F51\u683C\u4E25\u4E1D\u5408\u7F1D)

| \u80FD\u529B | \u624B\u6CD5 | \u89C4\u6A21 |
| --- | --- | --- |
| \u6587\u5B57 | SDF / MSDF | \u4EFB\u610F\u7F29\u653E\u9510\u5229 |
| \u6D41\u5F0F | \u9010\u5B57 reveal | \u5168\u7A0B\u65E0\u8DF3\u53D8 |
| \u5386\u53F2 | settled \u6D3E\u7ED8 | 100+ \u8F6E\u56DE\u65E7\u4E1D\u6ED1 |` }, { label: "\u6570\u5B66\u516C\u5F0F", md: `# \u6570\u5B66\u516C\u5F0F(SDF \u6392\u7248)

\u884C\u5185:\u8D28\u80FD $E = mc^2$,\u6B27\u62C9\u6052\u7B49\u5F0F $e^{i\\pi} + 1 = 0$\u3002

\u5757\u7EA7\u6C42\u548C:

$$\\sum_{i=1}^{n} i = \\frac{n(n+1)}{2}$$` }, { label: "\u5F15\u7528\u4E0E Alert", md: `# \u5F15\u7528\u5757\u4E0E GitHub Alert

> \u666E\u901A\u5F15\u7528:\u5F15\u64CE\u5728\u8DD1,\u4F60\u770B\u5230\u7684\u5C31\u662F\u5BA3\u4F20\u7247\u672C\u8EAB\u3002

> [!NOTE]
> GitHub Alert \u4E5F\u8D70 SDF \u6E32\u67D3,\u4EFB\u610F\u7F29\u653E\u90FD\u9510\u5229\u3002

> [!WARNING]
> \u6D41\u5F0F\u524D\u6CBF\u7559 hold \u533A,\u4E0D\u63D0\u4EA4\u6B67\u4E49\u8BED\u6CD5\u5B57\u8282\u3002` }];
function S(i) {
  const n = document.querySelector(".md-canvas-wrap");
  if (!n) return;
  const t = document.createElement("div");
  t.style.cssText = "margin:auto;padding:24px;color:var(--ink-dim);font:13px/1.7 system-ui;text-align:center;max-width:30em", t.textContent = i, n.appendChild(t);
}
async function b() {
  var _a;
  const i = document.getElementById("md-canvas");
  if (!i) return;
  const { chat: n, ok: t } = await I({ canvasId: "md-canvas", rhythmPreset: "reader", glyphMode: 1 });
  if (!t || !n) {
    i.style.display = "none", S("\u6B64\u6D4F\u89C8\u5668\u65E0\u6CD5\u542F\u52A8 WebGPU/WebGL2 \u2014\u2014 Markdown \u6F14\u793A\u9700\u8981\u5F15\u64CE\u3002\u7528 Chrome \u6253\u5F00\u53EF\u770B\u5168\u7C7B\u578B\u6D41\u5F0F\u63ED\u793A\u3002");
    return;
  }
  const r = "md", a = "md-msg";
  let m = false, h = 0, d = "";
  const c = (e) => {
    m || (n.push_event(JSON.stringify({ type: "message.updated", properties: { info: { id: a, role: "assistant", sessionID: r } } })), m = true);
    const f = `md-p-${h++}`;
    n.push_event(JSON.stringify({ type: "message.part.updated", properties: { part: { type: "text", id: f, messageID: a, sessionID: r, text: e }, time: 1 } })), d && n.push_event(JSON.stringify({ type: "message.part.removed", properties: { sessionID: r, messageID: a, partID: d } })), d = f;
  }, y = 7e3;
  let s = 0, l = 0;
  const o = () => {
    s && window.clearInterval(s), s = 0;
  }, p = () => {
    o();
    const e = () => {
      document.hidden || (c(v[l % v.length].md), l++);
    };
    e(), s = window.setInterval(e, y);
  }, u = (e = 0) => {
    if (!n.set_effect_preset("expressive")) {
      e < 600 && requestAnimationFrame(() => u(e + 1));
      return;
    }
    p();
  };
  u();
  const g = document.getElementById("md-input");
  (_a = document.getElementById("md-run")) == null ? void 0 : _a.addEventListener("click", () => {
    o();
    const e = (g == null ? void 0 : g.value) ?? "";
    e.trim() && c(e);
  }), document.addEventListener("visibilitychange", () => {
    n.set_paused(document.hidden), document.hidden ? o() : s || p();
  });
}
b();
