//! content(M6)— markdown 语义化(Plan2 H)。
//!
//! 把一段 markdown 源文本解析成带**样式角色**的 `StyledSpan` 序列(隐藏语法标记,块间插
//! `\n`)。用纯 Rust 的 `pulldown-cmark`(wasm 安全)。语法高亮(syntect)留作独立子项。
//!
//! 流式不闪烁:[`remend`] 在解析前对尾部"主动补全"未闭合的 `**`/`` ` ``/```` ``` ````
//! (0004 §5.1),避免半截语法字符瞬间字面显形(AR9 同族)。

use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};

/// 样式角色(content→layout/render 契约,architecture §五.3)。role 决定字体(粗/斜/等宽)
/// 与上色;render 侧 atlas 按 (role, cluster) 分桶,glyph.wgsl 按 role 取色。
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
    /// 引用块。
    Quote,
    /// 列表项标记("• ")。
    ListMarker,
}

impl StyleRole {
    /// 给 render/atlas 用的稳定数值。
    pub fn as_u32(self) -> u32 {
        self as u32
    }

    fn from_emphasis(bold: u32, italic: u32) -> Self {
        match (bold > 0, italic > 0) {
            (true, true) => StyleRole::BoldItalic,
            (true, false) => StyleRole::Bold,
            (false, true) => StyleRole::Italic,
            (false, false) => StyleRole::Normal,
        }
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

/// 解析 markdown 源 → 带角色的 span 序列。块间以 `\n` 分隔(零宽换行由 layout 处理)。
pub fn parse_markdown(src: &str) -> Vec<StyledSpan> {
    let patched = remend(src);
    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_STRIKETHROUGH);
    let parser = Parser::new_ext(&patched, opts);

    let mut out: Vec<StyledSpan> = Vec::new();
    let mut bold = 0u32;
    let mut italic = 0u32;
    let mut in_code_block = false;
    let mut in_heading = false;
    let mut in_quote = false;
    let mut in_link = false;
    let mut pending_break = false; // 块结束后,下一块前插一个 \n

    // 在块开始时把待插的换行落实(避免末尾多余换行)。
    let flush_break = |out: &mut Vec<StyledSpan>, pending: &mut bool| {
        if *pending && !out.is_empty() {
            out.push(StyledSpan::new("\n", StyleRole::Normal));
        }
        *pending = false;
    };

    for ev in parser {
        match ev {
            Event::Start(Tag::Strong) => bold += 1,
            Event::End(TagEnd::Strong) => bold = bold.saturating_sub(1),
            Event::Start(Tag::Emphasis) => italic += 1,
            Event::End(TagEnd::Emphasis) => italic = italic.saturating_sub(1),
            Event::Start(Tag::Heading { .. }) => {
                flush_break(&mut out, &mut pending_break);
                in_heading = true;
            }
            Event::End(TagEnd::Heading(_)) => {
                in_heading = false;
                pending_break = true;
            }
            Event::Start(Tag::CodeBlock(_)) => {
                flush_break(&mut out, &mut pending_break);
                in_code_block = true;
            }
            Event::End(TagEnd::CodeBlock) => {
                in_code_block = false;
                pending_break = true;
            }
            Event::Start(Tag::BlockQuote(_)) => in_quote = true,
            Event::End(TagEnd::BlockQuote(_)) => {
                in_quote = false;
                pending_break = true;
            }
            Event::Start(Tag::Item) => {
                flush_break(&mut out, &mut pending_break);
                out.push(StyledSpan::new("• ", StyleRole::ListMarker));
            }
            Event::Start(Tag::Paragraph) => flush_break(&mut out, &mut pending_break),
            Event::End(TagEnd::Item | TagEnd::Paragraph) => pending_break = true,
            Event::Start(Tag::Link { .. }) => in_link = true,
            Event::End(TagEnd::Link) => in_link = false,
            Event::Text(t) => {
                flush_break(&mut out, &mut pending_break);
                let role = if in_code_block {
                    StyleRole::CodeBlock
                } else if in_heading {
                    StyleRole::Heading
                } else if in_link {
                    StyleRole::Link
                } else if in_quote {
                    StyleRole::Quote
                } else {
                    StyleRole::from_emphasis(bold, italic)
                };
                push_text(&mut out, &t, role);
            }
            Event::Code(t) => {
                flush_break(&mut out, &mut pending_break);
                push_text(&mut out, &t, StyleRole::Code);
            }
            Event::SoftBreak | Event::HardBreak => {
                push_text(&mut out, " ", StyleRole::from_emphasis(bold, italic));
            }
            _ => {}
        }
    }
    out
}

/// 把代码块里的内部换行拆成 span 边界的 `\n`(零宽,layout 处理);其余直接成 span。
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
    // 代码围栏 ``` 的奇偶:奇数个 → 补一行收尾。
    let fence_count = src.matches("```").count();
    let mut patched = src.to_string();
    if fence_count % 2 == 1 {
        if !patched.ends_with('\n') {
            patched.push('\n');
        }
        patched.push_str("```");
        return patched; // 围栏内不再处理行内标记
    }
    // 行内:对最后一行补未闭合的 ** / * / `(成对补全)。
    let last_line = patched.rsplit('\n').next().unwrap_or("");
    let mut suffix = String::new();
    if last_line.matches('`').count() % 2 == 1 {
        suffix.push('`');
    }
    // ** 先于 *:统计去掉 ** 后的剩余 * 奇偶。
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
        // 语法标记 ** * ` 被隐藏。
        assert!(!render(&spans).contains('*'));
        assert!(!render(&spans).contains('`'));
    }

    #[test]
    fn heading_and_paragraphs_break() {
        let spans = parse_markdown("# Title\n\nbody");
        assert_eq!(role_of(&spans, "Title"), Some(StyleRole::Heading));
        assert!(render(&spans).contains('\n'), "标题与正文之间应有块换行");
        assert!(!render(&spans).contains('#'), "# 标记隐藏");
    }

    #[test]
    fn list_items_get_markers() {
        let spans = parse_markdown("- one\n- two");
        let r = render(&spans);
        assert!(r.contains("• one"));
        assert!(r.contains("• two"));
    }

    #[test]
    fn code_block_role() {
        let spans = parse_markdown("```\nlet x = 1;\n```");
        assert_eq!(role_of(&spans, "let x"), Some(StyleRole::CodeBlock));
    }

    #[test]
    fn remend_hides_unclosed_bold_no_flicker() {
        // 流式中途:"**bol" 未闭合 → 不应字面显示 **。
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
}
