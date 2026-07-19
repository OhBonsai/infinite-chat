const m = [{ title: "Agent \u4F1A\u8BDD\u5361 \xB7 \u4E5D\u7EC4\u4EF6", span: [4, 1], summary: "", section: true }, { title: "USER", span: [2, 1], summary: "user \u6C14\u6CE1" }, { title: "ASSISTANT", span: [2, 1], summary: "assistant \u6B63\u6587(markdown)" }, { title: "REASONING", span: [2, 1], summary: "\u63A8\u7406\u5361" }, { title: "TOOL \xB7 READ", span: [2, 1], summary: "running + shimmer" }, { title: "TOOL \xB7 BASH", span: [2, 2], summary: "completed + \u8F93\u51FA" }, { title: "ASK", span: [2, 2], summary: "permission \u771F\u6309\u94AE" }, { title: "TOOL \xB7 EDIT", span: [4, 2], summary: "diff \u5361(\u6298\u53E0\u53EF\u5C55\u5F00)" }, { title: "COMPACTION", span: [4, 1], summary: "\u4E0A\u4E0B\u6587\u538B\u7F29\u5206\u9694" }, { title: "ERROR", span: [2, 1], summary: "\u9519\u8BEF\u515C\u5E95\u5361" }, { title: "Markdown \u5168\u7C7B\u578B \xB7 \u5341\u4E5D\u578B", span: [4, 1], summary: "", section: true }, { title: "HEADINGS", span: [2, 1], summary: "H1/H2/H3" }, { title: "EMPHASIS", span: [2, 1], summary: "\u7C97\u659C\u5220 + \u884C\u5185\u7801" }, { title: "LINK", span: [1, 1], summary: "\u53EF\u70B9\u94FE\u63A5" }, { title: "EMOJI", span: [1, 1], summary: "emoji + CJK" }, { title: "LIST \xB7 UL", span: [2, 1], summary: "\u65E0\u5E8F + \u5D4C\u5957" }, { title: "LIST \xB7 OL", span: [2, 1], summary: "\u6709\u5E8F" }, { title: "TASKS", span: [2, 1], summary: "\u4EFB\u52A1\u5217\u8868" }, { title: "QUOTE", span: [2, 1], summary: "\u5F15\u7528" }, { title: "ALERT \xB7 NOTE", span: [2, 1], summary: "GitHub Alert" }, { title: "ALERT \xB7 WARN", span: [2, 1], summary: "GitHub Alert" }, { title: "RULE", span: [4, 1], summary: "\u5206\u8282\u7EBF" }, { title: "CODE BLOCK", span: [4, 2], summary: "\u9AD8\u4EAE + \u884C\u53F7" }, { title: "CODE \xB7 FOLD", span: [2, 2], summary: "\u6298\u53E0\u884C\u7A97" }, { title: "TABLE", span: [4, 2], summary: "\u5BF9\u9F50 + CJK" }, { title: "FOOTNOTE", span: [2, 1], summary: "\u811A\u6CE8" }, { title: "MATH \xB7 IN", span: [1, 1], summary: "\u884C\u5185\u516C\u5F0F" }, { title: "MATH \xB7 BLOCK", span: [2, 2], summary: "\u5757\u7EA7\u516C\u5F0F" }, { title: "SVG", span: [2, 1], summary: "\u77E2\u91CF" }];
function i(e, r) {
  e.style.display = "block", e.style.cssText += ";display:grid;grid-template-columns:repeat(4,1fr);gap:16px;padding:16px;box-sizing:border-box;overflow:auto;height:100%";
  for (const t of m) {
    const s = document.createElement("div");
    if (s.style.gridColumn = `span ${t.span[0]}`, s.style.gridRow = `span ${t.span[1]}`, t.section) s.style.cssText += ";grid-column:1 / -1;color:var(--gold);font:600 15px/1.4 var(--font-display);letter-spacing:0.08em;padding:14px 4px 2px;border:0", s.textContent = t.title;
    else {
      s.style.cssText += `;grid-column:span ${t.span[0]};grid-row:span ${t.span[1]};background:var(--surface-base,#141a24);border:1px solid var(--gold-dim);border-radius:10px;padding:12px 14px;box-sizing:border-box;min-height:${t.span[1] * 116}px`;
      const a = document.createElement("div");
      a.textContent = t.title, a.style.cssText = "color:var(--gold);font:600 11px/1.4 var(--font-mono);letter-spacing:0.12em";
      const n = document.createElement("div");
      n.textContent = t.summary, n.style.cssText = "color:var(--ink-dim);font:13px/1.6 var(--font-cjk,sans-serif);margin-top:8px", s.append(a, n);
    }
    e.appendChild(s);
  }
}
export {
  i as mountComponentsFallback
};
