// components-fallback.ts — Plan 44 T2:无 GPU 降级。引擎起不来(WebGPU+WebGL2 皆无)时,用 CSS grid
// 复刻同设计表布局的**静态网格**(4 列 span,同对齐),纯 DOM 兜底(内容取 manifest 标题 + 一行摘要)。
// 非降级路径零 DOM 内容层(SDF 纯度硬门只约束引擎路径)。

interface FbTile {
  title: string;
  span: [number, number];
  summary: string;
  section?: boolean;
}

// 与 components.ts TILES 同序同 span(降级只给结构感,不复刻富渲染)。
const FB: FbTile[] = [
  { title: "Agent 会话卡 · 九组件", span: [4, 1], summary: "", section: true },
  { title: "USER", span: [2, 1], summary: "user 气泡" },
  { title: "ASSISTANT", span: [2, 1], summary: "assistant 正文(markdown)" },
  { title: "REASONING", span: [2, 1], summary: "推理卡" },
  { title: "TOOL · READ", span: [2, 1], summary: "running + shimmer" },
  { title: "TOOL · BASH", span: [2, 2], summary: "completed + 输出" },
  { title: "ASK", span: [2, 2], summary: "permission 真按钮" },
  { title: "TOOL · EDIT", span: [4, 2], summary: "diff 卡(折叠可展开)" },
  { title: "COMPACTION", span: [4, 1], summary: "上下文压缩分隔" },
  { title: "ERROR", span: [2, 1], summary: "错误兜底卡" },
  { title: "Markdown 全类型 · 十九型", span: [4, 1], summary: "", section: true },
  { title: "HEADINGS", span: [2, 1], summary: "H1/H2/H3" },
  { title: "EMPHASIS", span: [2, 1], summary: "粗斜删 + 行内码" },
  { title: "LINK", span: [1, 1], summary: "可点链接" },
  { title: "EMOJI", span: [1, 1], summary: "emoji + CJK" },
  { title: "LIST · UL", span: [2, 1], summary: "无序 + 嵌套" },
  { title: "LIST · OL", span: [2, 1], summary: "有序" },
  { title: "TASKS", span: [2, 1], summary: "任务列表" },
  { title: "QUOTE", span: [2, 1], summary: "引用" },
  { title: "ALERT · NOTE", span: [2, 1], summary: "GitHub Alert" },
  { title: "ALERT · WARN", span: [2, 1], summary: "GitHub Alert" },
  { title: "RULE", span: [4, 1], summary: "分节线" },
  { title: "CODE BLOCK", span: [4, 2], summary: "高亮 + 行号" },
  { title: "CODE · FOLD", span: [2, 2], summary: "折叠行窗" },
  { title: "TABLE", span: [4, 2], summary: "对齐 + CJK" },
  { title: "FOOTNOTE", span: [2, 1], summary: "脚注" },
  { title: "MATH · IN", span: [1, 1], summary: "行内公式" },
  { title: "MATH · BLOCK", span: [2, 2], summary: "块级公式" },
  { title: "SVG", span: [2, 1], summary: "矢量" },
];

/** DOM 网格幻灯降级:4 列 CSS grid,span 同设计表;section 带整行标题。 */
export function mountComponentsFallback(container: HTMLElement, _base: string): void {
  container.style.display = "block";
  container.style.cssText +=
    ";display:grid;grid-template-columns:repeat(4,1fr);gap:16px;padding:16px;box-sizing:border-box;overflow:auto;height:100%";
  for (const t of FB) {
    const el = document.createElement("div");
    el.style.gridColumn = `span ${t.span[0]}`;
    el.style.gridRow = `span ${t.span[1]}`;
    if (t.section) {
      el.style.cssText +=
        ";grid-column:1 / -1;color:var(--gold);font:600 15px/1.4 var(--font-display);letter-spacing:0.08em;padding:14px 4px 2px;border:0";
      el.textContent = t.title;
    } else {
      el.style.cssText +=
        `;grid-column:span ${t.span[0]};grid-row:span ${t.span[1]};background:var(--surface-base,#141a24);border:1px solid var(--gold-dim);border-radius:10px;padding:12px 14px;box-sizing:border-box;min-height:${t.span[1] * 116}px`;
      const h = document.createElement("div");
      h.textContent = t.title;
      h.style.cssText = "color:var(--gold);font:600 11px/1.4 var(--font-mono);letter-spacing:0.12em";
      const p = document.createElement("div");
      p.textContent = t.summary;
      p.style.cssText = "color:var(--ink-dim);font:13px/1.6 var(--font-cjk,sans-serif);margin-top:8px";
      el.append(h, p);
    }
    container.appendChild(el);
  }
}
