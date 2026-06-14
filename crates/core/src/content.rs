//! content(M6)— markdown 语义化(Plan2 H,plan2 H1)。
//!
//! 解析交给 vendored 的 **jcode-render-core**(后端中立文档模型:`parse_markdown -> Document`,
//! 含标题/粗斜/行内与块代码/列表/引用/**表格**/数学等);本文件把它的 `Document` 适配成我们
//! 渲染管线的 [`StyledSpan`] + [`StyleRole`](决定字体/上色,render 侧按 `as_u32` 分桶取色)。
//!
//! 流式不闪烁:[`remend`] 在解析前对尾部"主动补全"未闭合的 `**`/`` ` ``/```` ``` ````
//! (0004 §5.1),避免半截语法字符瞬间字面显形(AR9 同族)。

use jcode_render_core::{
    parse_markdown as jcode_parse, Block, BlockKind, StyleRole as JRole, StyledSpan as JSpan,
    TextAttrs,
};

/// 样式角色(content→layout/render 契约,architecture §五.3)。role 决定字体(粗/斜/等宽)
/// 与上色;render 侧 atlas 按 (role, cluster) 分桶,glyph.wgsl 按 role 取色。数值稳定。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum StyleRole {
    #[default]
    Normal,
    Bold,
    Italic,
    BoldItalic,
    /// 行内代码。
    Code,
    /// 代码块内文本。
    CodeBlock,
    /// 标题(H1–H6 合并为一类,Plan2 不分级)。
    Heading,
    /// 链接文字。
    Link,
    /// 引用块 / 弱化文本(reasoning)。
    Quote,
    /// 列表/表格标记(dim)。
    ListMarker,
    /// 标题 H2–H6 分级(H1 = [`StyleRole::Heading`]);逐级字号在 layout/raster 侧按角色取。
    Heading2,
    Heading3,
    Heading4,
    Heading5,
    Heading6,
}

impl StyleRole {
    /// 给 render/atlas 用的稳定数值。
    pub fn as_u32(self) -> u32 {
        self as u32
    }

    /// markdown 标题级别(1–6)→ 角色。H1 复用 `Heading`,H2–H6 分级(4A3)。
    pub fn heading(level: u8) -> StyleRole {
        match level {
            0 | 1 => StyleRole::Heading,
            2 => StyleRole::Heading2,
            3 => StyleRole::Heading3,
            4 => StyleRole::Heading4,
            5 => StyleRole::Heading5,
            _ => StyleRole::Heading6,
        }
    }

    /// 是否标题角色(任意级别)。
    pub fn is_heading(self) -> bool {
        matches!(
            self,
            StyleRole::Heading
                | StyleRole::Heading2
                | StyleRole::Heading3
                | StyleRole::Heading4
                | StyleRole::Heading5
                | StyleRole::Heading6
        )
    }
}

/// 一段带样式角色的文本 run(content→layout)。
#[derive(Clone, Debug, PartialEq)]
pub struct StyledSpan {
    text: String,
    role: StyleRole,
}

impl StyledSpan {
    pub fn new(text: impl Into<String>, role: StyleRole) -> Self {
        Self {
            text: text.into(),
            role,
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn role(&self) -> StyleRole {
        self.role
    }
}

/// 纯文本直通:整段文本 → 单个 Normal span(给非 markdown 路径/测试用)。
pub fn plain(text: &str) -> Vec<StyledSpan> {
    if text.is_empty() {
        Vec::new()
    } else {
        vec![StyledSpan::new(text, StyleRole::Normal)]
    }
}

/// 解析 markdown 源 → 带角色的 span 序列。块/行间以 `\n` 分隔(零宽换行由 layout 处理)。
pub fn parse_markdown(src: &str) -> Vec<StyledSpan> {
    let patched = remend(src);
    let doc = jcode_parse(&patched);
    let mut out: Vec<StyledSpan> = Vec::new();
    for block in &doc.blocks {
        if !out.is_empty() {
            out.push(StyledSpan::new("\n", StyleRole::Normal)); // 块间换行
        }
        emit_block(&mut out, block);
    }
    out
}

/// 把一个 jcode Block 展平成我们的 span 序列(行间插 `\n`)。
fn emit_block(out: &mut Vec<StyledSpan>, block: &Block) {
    if matches!(block.kind, BlockKind::Table) && !block.table.is_empty() {
        // 表格:每行一行,单元格用 " │ " 分隔;表头加粗。列对齐(等宽)留后续。
        for (r, row) in block.table.iter().enumerate() {
            if r > 0 {
                out.push(StyledSpan::new("\n", StyleRole::Normal));
            }
            let role = if r == 0 {
                StyleRole::Heading
            } else {
                StyleRole::Normal
            };
            for (c, cell) in row.iter().enumerate() {
                if c > 0 {
                    out.push(StyledSpan::new(" │ ", StyleRole::ListMarker));
                }
                push_text(out, cell, role);
            }
        }
        return;
    }
    for (i, line) in block.lines.iter().enumerate() {
        if i > 0 {
            out.push(StyledSpan::new("\n", StyleRole::Normal));
        }
        for span in &line.spans {
            let role = map_role(span, &block.kind);
            push_text(out, &span.text, role);
        }
    }
}

/// jcode 的 (StyleRole + attrs + 块类型) → 我们的渲染角色。
fn map_role(span: &JSpan, kind: &BlockKind) -> StyleRole {
    // 块级覆盖优先。
    match kind {
        BlockKind::Heading { level } => return StyleRole::heading(*level),
        BlockKind::CodeBlock { .. } => return StyleRole::CodeBlock,
        _ => {}
    }
    match span.role {
        JRole::Code | JRole::Math => StyleRole::Code,
        JRole::Link => StyleRole::Link,
        JRole::Dim => StyleRole::ListMarker,
        JRole::Reasoning => StyleRole::Quote,
        JRole::Html => StyleRole::Normal,
        JRole::Strong | JRole::Text => {
            let bold = span.attrs.bold || matches!(span.role, JRole::Strong);
            emphasis(bold, span.attrs)
        }
    }
}

fn emphasis(bold: bool, attrs: TextAttrs) -> StyleRole {
    match (bold, attrs.italic) {
        (true, true) => StyleRole::BoldItalic,
        (true, false) => StyleRole::Bold,
        (false, true) => StyleRole::Italic,
        (false, false) => StyleRole::Normal,
    }
}

/// 把内部换行拆成 span 边界的 `\n`(零宽,layout 处理);其余直接成 span。
fn push_text(out: &mut Vec<StyledSpan>, text: &str, role: StyleRole) {
    let mut first = true;
    for line in text.split('\n') {
        if !first {
            out.push(StyledSpan::new("\n", StyleRole::Normal));
        }
        first = false;
        if !line.is_empty() {
            out.push(StyledSpan::new(line, role));
        }
    }
}

/// 尾部"主动补全"未闭合的行内/块语法,消除流式半截标记闪烁(0004 §5.1,AR9)。
/// 只补全成可解析的形态,不改变已闭合内容。
fn remend(src: &str) -> String {
    let fence_count = src.matches("```").count();
    let mut patched = src.to_string();
    if fence_count % 2 == 1 {
        if !patched.ends_with('\n') {
            patched.push('\n');
        }
        patched.push_str("```");
        return patched; // 围栏内不再处理行内标记
    }
    let last_line = patched.rsplit('\n').next().unwrap_or("");
    let mut suffix = String::new();
    if last_line.matches('`').count() % 2 == 1 {
        suffix.push('`');
    }
    let strong = last_line.matches("**").count();
    if strong % 2 == 1 {
        suffix.push_str("**");
    }
    let without_strong = last_line.replace("**", "");
    if without_strong.matches('*').count() % 2 == 1 {
        suffix.push('*');
    }
    patched.push_str(&suffix);
    patched
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render(spans: &[StyledSpan]) -> String {
        spans.iter().map(StyledSpan::text).collect()
    }

    fn role_of(spans: &[StyledSpan], needle: &str) -> Option<StyleRole> {
        spans
            .iter()
            .find(|s| s.text().contains(needle))
            .map(StyledSpan::role)
    }

    #[test]
    fn bold_italic_code_roles() {
        let spans = parse_markdown("a **b** c *d* `e`");
        assert_eq!(role_of(&spans, "b"), Some(StyleRole::Bold));
        assert_eq!(role_of(&spans, "d"), Some(StyleRole::Italic));
        assert_eq!(role_of(&spans, "e"), Some(StyleRole::Code));
        assert!(!render(&spans).contains('*'));
        assert!(!render(&spans).contains('`'));
    }

    #[test]
    fn heading_role_and_break() {
        let spans = parse_markdown("# Title\n\nbody");
        assert_eq!(role_of(&spans, "Title"), Some(StyleRole::Heading));
        assert!(render(&spans).contains('\n'), "标题与正文之间应有换行");
        assert!(!render(&spans).contains('#'));
    }

    #[test]
    fn heading_levels_distinct() {
        let spans = parse_markdown("# One\n\n## Two\n\n### Three");
        assert_eq!(role_of(&spans, "One"), Some(StyleRole::Heading)); // H1
        assert_eq!(role_of(&spans, "Two"), Some(StyleRole::Heading2));
        assert_eq!(role_of(&spans, "Three"), Some(StyleRole::Heading3));
        assert!(StyleRole::Heading3.is_heading());
        assert!(!StyleRole::Bold.is_heading());
    }

    #[test]
    fn code_block_role() {
        let spans = parse_markdown("```\nlet x = 1;\n```");
        assert_eq!(role_of(&spans, "let x"), Some(StyleRole::CodeBlock));
    }

    #[test]
    fn table_renders_rows_not_raw_pipes() {
        let md = "| A | B |\n|---|---|\n| 1 | 2 |";
        let spans = parse_markdown(md);
        let r = render(&spans);
        assert!(!r.contains('|'), "原始竖线不该显形: {r}");
        assert!(!r.contains("---"), "分隔行不该显形: {r}");
        assert!(r.contains('│'), "应有单元格分隔符: {r}");
        for cell in ["A", "B", "1", "2"] {
            assert!(r.contains(cell), "缺单元格 {cell}: {r}");
        }
        assert_eq!(role_of(&spans, "A"), Some(StyleRole::Heading)); // 表头加粗
    }

    #[test]
    fn remend_hides_unclosed_bold_no_flicker() {
        let spans = parse_markdown("**bol");
        assert!(
            !render(&spans).contains('*'),
            "未闭合 ** 不该显形: {:?}",
            render(&spans)
        );
        assert_eq!(role_of(&spans, "bol"), Some(StyleRole::Bold));
    }

    #[test]
    fn remend_closes_unbalanced_fence() {
        let spans = parse_markdown("```\ncode");
        assert_eq!(role_of(&spans, "code"), Some(StyleRole::CodeBlock));
        assert!(!render(&spans).contains('`'));
    }

    #[test]
    fn plain_passthrough_still_works() {
        let spans = plain("纯文本 🚀");
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].role(), StyleRole::Normal);
        assert_eq!(spans[0].text(), "纯文本 🚀");
    }

    #[test]
    fn comprehensive_markdown_syntax_coverage() {
        let md = r#"# Heading 1
## Heading 2
### Heading 3
#### Heading 4
##### Heading 5
###### Heading 6

This is a **bold** word and an *italic* word and ***bold italic*** and ~~strikethrough~~ text.

Inline `code` span, a [link](https://example.com), an ![image](img.png), and some $math$ inline.

> A blockquote line.
> Multiple blockquote lines.

- Unordered item
- Another item

1. First ordered
2. Second ordered

- Top level
  - Nested item

- [ ] Pending task
- [x] Done task

```
fn main() {
    let x = 42;
}
```

```rust
fn typed() -> &'static str {
    "hello"
}
```

| Name | Age |
|------|-----|
| Alice | 30 |
| Bob | 25 |

---

Footnote: some text[^1].

[^1]: This is a footnote definition.

Term
: Definition text.

Display math:
$$E = mc^2$$
"#;

        let spans = parse_markdown(md);
        let r = render(&spans);

        // Headings — H1 = Heading, H2–H6 分级(4A3)
        assert_eq!(role_of(&spans, "Heading 1"), Some(StyleRole::Heading));
        assert_eq!(role_of(&spans, "Heading 6"), Some(StyleRole::Heading6));
        assert!(!r.contains('#'), "raw # 不应出现在渲染文本中");

        // Inline styles
        assert_eq!(role_of(&spans, "bold"), Some(StyleRole::Bold));
        assert_eq!(role_of(&spans, "italic"), Some(StyleRole::Italic));
        assert_eq!(role_of(&spans, "bold italic"), Some(StyleRole::BoldItalic));
        assert!(!r.contains("~~"), "raw ~~ 不应出现");

        // Code
        assert_eq!(role_of(&spans, "code"), Some(StyleRole::Code));
        assert!(!r.contains('`'), "raw backtick 不应出现");

        // Link and image
        assert!(r.contains("link"), "链接文本应保留: {r}");
        assert!(r.contains("https://example.com"), "链接 URL 应保留");
        assert!(
            r.contains("[image:") || r.contains("img.png"),
            "图片应渲染为占位符: {r}"
        );

        // Math
        assert!(r.contains("math"), "行内数学应保留文本");

        // Blockquote — content preserved, role is Normal (BlockKind::BlockQuote not yet mapped to Quote)
        assert!(r.contains("blockquote line"), "引用块文本应保留: {r}");

        // Lists
        assert!(r.contains("Unordered"), "无序列表项应保留");
        assert!(r.contains("First"), "有序列表项应保留");
        assert!(r.contains("Nested"), "嵌套列表项应保留");

        // Task lists
        assert!(r.contains("Pending"), "待办项应保留");
        assert!(r.contains("Done"), "完成项应保留");

        // Code blocks
        assert_eq!(role_of(&spans, "fn main() {"), Some(StyleRole::CodeBlock));
        assert_eq!(role_of(&spans, "fn typed()"), Some(StyleRole::CodeBlock));

        // Table
        assert!(r.contains('│'), "表格应有单元格分隔符");
        assert!(!r.contains('|'), "原始竖线不应显形");

        // Thematic break
        assert!(r.contains('─'), "分隔线应渲染为横线字符");

        // Footnote
        assert!(r.to_lowercase().contains("footnote"), "脚注定义应保留: {r}");

        // Definition list
        assert!(r.contains("Definition"), "定义列表项应保留");

        // Display math
        assert!(r.contains("E = mc"), "显示数学应保留文本");
    }
}
