//! 渲染样式角色与 span(0039 下沉自 core::content):content→layout/render 的纯数据契约。

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
    /// 分隔线(`---`)锚点:发一个零墨空格,render 侧据此画整宽细线 rect(4B1)。
    Rule,
    /// GitHub Alert 标签行(`[!NOTE]` 等);承载告警类型,render 侧据此上色左条(4B1)。
    AlertLabel,
    /// 表格单元格(等宽对齐,0014 A);layout 用等宽字体 → 列对齐。
    TableCell,
    /// 表格表头单元格(等宽 + 表头色;render 侧据此画表头底 + 分隔线)。
    TableHeader,
    /// 表格体内强调(粗/链接文字):**等宽加粗**保列对齐 + 区分(0014 A,5E.1 #2)。
    TableStrong,
    /// 表格体内斜体:**等宽斜体**保列对齐(与 TableStrong 区分,5E.1 #2)。
    TableEm,
    /// 表格列分隔符(`│`):render 侧据其 x 画**全表高竖直网格线**(5E.1 #5);等宽 + 弱化色。
    TableSep,
    /// 任务复选框·未勾(GFM `- [ ]`,0026/Plan 11):零墨锚点字格,render 侧据此画 SDF 空框。
    /// **追加在末尾**(值 22),不移动既有数值 → 守 0001 契约"数值稳定"(shader/enter_profile_id 不动)。
    TaskUnchecked,
    /// 任务复选框·已勾(GFM `- [x]`):零墨锚点字格,render 侧画 SDF 框 + 对勾。值 23。
    TaskChecked,
    /// 脚注引用(`[^1]`,Plan 11 §4):行内小号 Link 色标记(去方括号);值 24。
    FootnoteRef,
    /// 脚注定义行首标记(`[^1]:`,Plan 11 §4):弱化小号 `1.`;值 25。
    FootnoteDef,
    // ── 数学(LaTeX)字形角色(Plan 12 / 0013 §8):值 = KaTeX 字族,web 据此选字体 + atlas 分桶。
    //    **追加在末尾**(值 26+),不移动既有数值 → 守 0001 数值稳定。RaTeX `GlyphPath.font` 字符串
    //    映射到这些(见 [`crate::math::font_role`])。
    /// `KaTeX_Main-Regular`:数字 / 运算符 / 直立符号。值 26。
    MathMain,
    /// `KaTeX_Main-Bold`。值 27。
    MathBold,
    /// `KaTeX_Main-Italic`。值 28。
    MathItalic,
    /// `KaTeX_Main-BoldItalic` / `KaTeX_Math-BoldItalic`。值 29。
    MathBoldItalic,
    /// `KaTeX_Math-Italic`:数学变量(斜体 a/b/x…)。值 30。
    MathVar,
    /// `KaTeX_AMS-Regular`:AMS 符号。值 31。
    MathAms,
    /// `KaTeX_Size1-Regular`:大号定界符/算符(∑∫√ 等)。值 32。
    MathSize1,
    /// `KaTeX_Size2-Regular`。值 33。
    MathSize2,
    /// `KaTeX_Size3-Regular`。值 34。
    MathSize3,
    /// `KaTeX_Size4-Regular`。值 35。
    MathSize4,
    /// `KaTeX_Caligraphic-Regular`。值 36。
    MathCal,
    /// `KaTeX_Fraktur-Regular|Bold`。值 37。
    MathFrak,
    /// `KaTeX_SansSerif-*`。值 38。
    MathSans,
    /// `KaTeX_Script-Regular`。值 39。
    MathScript,
    /// `KaTeX_Typewriter-Regular`。值 40。
    MathTt,
    /// 行内数学源(`$…$`,含 `$` 定界):**哨兵**——build_frame 识别该 run、剥 `$` 交 RaTeX 排版成
    /// 数学 SDF 字形(`display=false`);RaTeX 失败则按本角色当普通文本回退(原文 `$…$`)。值 41。
    MathTeX,
    /// 图片嵌入(`![alt](url)`,Plan 14):**哨兵**——span 文本打包 `url\u{1f}alt`,content 抽成
    /// [`Embed`] 描述 + `NodeKind::Embed` 节点;桌面端走纹理 quad,失败/未加载回退 alt 文本。值 42。
    Image,
    /// 代码块行号(Plan 15 ②,§2.4):每行前置右对齐行号 + 分隔,弱化(Dim 小号)。行窗随纵滚、横滚
    /// 固定(gutter)。值 43。
    CodeLineNum,
    // 代码语法高亮(research 路 A,8 色塌缩):scope → 8 个语义角色 → 0021 调色板,render 零改。
    // `CodePlain` = 复用 `CodeBlock`(值 5)。下列 7 个值 44–50,尾部追加不移位(守 0001)。
    /// 关键字(`fn`/`def`/`if`…)。值 44。
    CodeKeyword,
    /// 类型(`u32`/`String`/CamelCase…)。值 45。
    CodeType,
    /// 函数名(后随 `(`)。值 46。
    CodeFunc,
    /// 字符串字面量。值 47。
    CodeString,
    /// 注释。值 48。
    CodeComment,
    /// 数字字面量。值 49。
    CodeNumber,
    /// 标点 / 运算符。值 50。
    CodePunct,
    // ── part 渲染角色(Plan 23 / 0033):tool 卡 / reasoning / diff 的语义角色。
    //    **追加在末尾**(值 51+),不移动既有数值 → 守 0001 数值稳定(shader/atlas 分桶不动)。
    //    视觉(卡底面板 / 左条 / diff 行底)由 app.rs 按角色画装饰,本枚举只定"是什么文字"。
    /// 推理 / 思考区正文(弱化,对标 Quote 但语义独立 → 可单独折叠/上色)。值 51。
    Reasoning,
    /// 工具卡标题(工具名,稍亮 + 略强)。值 52。
    ToolTitle,
    /// 工具调用参数(args/input,弱化等宽)。值 53。
    ToolArg,
    /// 工具输出(output,正文中性)。值 54。
    ToolOutput,
    /// 工具状态徽章(`[running]`/`[done]`/`[error]`):render 侧据状态上色小标。值 55。
    ToolBadge,
    /// diff 新增行(`+`):render 侧据此画绿底。值 56。
    DiffAdded,
    /// diff 删除行(`-`):render 侧据此画红底。值 57。
    DiffRemoved,
    /// 流内 ask 按钮/所选 chip 字(Plan 27):`block_decorations` 按此角色 run 画按钮面板 +
    /// 产命中盒(pending permission),落定卡里作所选项 chip 高亮。值 58。
    AskButton,
    /// diff 上下文/hunk 行(Plan 28 遗留-2):文件卡 bbox 计入 + 弱化色。值 59。
    DiffCtx,
}

impl StyleRole {
    /// 是否**代码内容**字(代码块正文,含高亮各类;不含行号 gutter)。Plan 15 行窗/横裁/底框判据。
    pub fn is_code_text(self) -> bool {
        matches!(
            self,
            StyleRole::CodeBlock
                | StyleRole::CodeKeyword
                | StyleRole::CodeType
                | StyleRole::CodeFunc
                | StyleRole::CodeString
                | StyleRole::CodeComment
                | StyleRole::CodeNumber
                | StyleRole::CodePunct
        )
    }

    /// 同 [`is_code_text`](Self::is_code_text) 但吃角色数值(app.rs 持 `Vec<u32>` roles 用):
    /// `CodeBlock`(5)或高亮 7 角色(44–50)。
    pub fn is_code_text_u32(role: u32) -> bool {
        role == StyleRole::CodeBlock as u32 || (44..=50).contains(&role)
    }

    /// 是否数学字族角色(26–40,KaTeX 字体);`MathTeX`(41,行内源哨兵)不算(回退作文本)。
    pub fn is_math_font(self) -> bool {
        let v = self as u32;
        (26..=40).contains(&v)
    }
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

/// 一段带样式角色的文本 run(content→layout)。`strike` = 删除线(与 role 正交:粗/斜可叠删除线;
/// 删除线是**装饰**不是字体,render 侧在字中线画一条 → 不入 layout,故 layout 忽略此字段)。
#[derive(Clone, Debug, PartialEq)]
pub struct StyledSpan {
    text: String,
    role: StyleRole,
    strike: bool,
}

impl StyledSpan {
    pub fn new(text: impl Into<String>, role: StyleRole) -> Self {
        Self {
            text: text.into(),
            role,
            strike: false,
        }
    }

    /// 带删除线的 run(`~~…~~`)。
    pub fn styled(text: impl Into<String>, role: StyleRole, strike: bool) -> Self {
        Self {
            text: text.into(),
            role,
            strike,
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn role(&self) -> StyleRole {
        self.role
    }

    /// 是否删除线(render 在字中线画线;`~~…~~`)。
    pub fn is_struck(&self) -> bool {
        self.strike
    }
}

/// 纯文本直通:整段文本 → 单个 Normal span(非 markdown 路径/测试/Default parse 用)。
#[must_use]
pub fn plain(text: &str) -> Vec<StyledSpan> {
    if text.is_empty() {
        Vec::new()
    } else {
        vec![StyledSpan::new(text, StyleRole::Normal)]
    }
}
