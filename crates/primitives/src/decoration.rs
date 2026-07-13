//! 声明式装饰契约(0037;0039 下沉自 core::partrender):纯数据指令,app 解析摆放。

use crate::style::StyledSpan;

// ───────────────────────── 0037:声明式装饰契约(additive) ─────────────────────────

/// 装饰种类(0037 §2):app 按 [`DecorationOp::span`] 解析成世界矩形(摆放规则 0037 §3)。
/// 未知种类兜底不画(AR12)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecorKind {
    /// 整宽行底(区间按行分段;同 `group` 相邻行 gloop 融合,Plan 32 D3)。
    Band,
    /// 左缘细条(区间行范围;宽 = floor(0.275×行高),Zed 语义)。
    Gutter,
    /// 字符级底(词级高亮;区间内逐段 glyph AABB)。
    CharBg,
    /// 折叠汇总行(Plan 32 D4):span = 「⋯ n unchanged lines」行,`group` = 折叠区序号。
    /// app 画 Zed 式强圆角 gutter marker(宽 floor(0.35×行高)、圆角 = 行高)+ 登记 tap 命中盒。
    FoldSummary,
    /// 已展开折叠区(Plan 32 D4):span = 展开的 Context 行,`group` = 折叠区序号。
    /// 不画;app 据此对展开动画中的行施加 scale 指数逼近包络(makepad 范式,收敛恒等 AR3)。
    FoldRegion,
}

/// 语义色槽(0021:色值 emit 时经 theme 解析,不进缓存)。首租全为 Diff 域;
/// plan33 迁其它 part 时自然扩非 Diff 槽(同前缀 lint 此期放行)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(clippy::enum_variant_names)]
pub enum DecorSlot {
    DiffAddBand,
    DiffDelBand,
    DiffAddGutter,
    DiffDelGutter,
    DiffModGutter,
    DiffAddWord,
    DiffDelWord,
}

/// 一条声明式装饰指令(0037):锚定 = 渲染后 display glyph 区间 [start,end)(与 clusters 对齐)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DecorationOp {
    pub kind: DecorKind,
    pub span: (u32, u32),
    pub slot: DecorSlot,
    pub group: u32,
}

/// full 渲染输出(0037):spans + 装饰指令。旧 [`RenderFn`] 自动包装(decorations 空)。
#[derive(Debug, Clone, PartialEq, Default)]
pub struct RenderOutput {
    pub spans: Vec<StyledSpan>,
    pub decorations: Vec<DecorationOp>,
}

impl RenderOutput {
    #[must_use]
    pub fn spans_only(spans: Vec<StyledSpan>) -> Self {
        Self {
            spans,
            decorations: Vec::new(),
        }
    }
}
