//! motion(M9 / Plan 25 P0)— MotionTokens:动效节奏 + 间距令牌一张表
//! (设计 [design/chat-page-design.md §3.1];原则 5「一切皆数据」)。
//!
//! 所有组件动效/间距只引用 token,不写裸数——改观感 = 改表,不是改管线。生效路径(0021):
//! - **时长/交错**(渲染类):Engine 每帧把 `dur_glyph` 写进 `FrameData.fade_ms` 等 → setter
//!   改完**下一帧生效,无需重排**。
//! - **间距**(布局输入):`boxlayout::layout_chat` 每帧读 → 改完下帧重排(无需 refresh_fonts;
//!   字内几何不变,只有盒位变,0016 morph 平滑收口)。
//!
//! 曲线本体在 `glyph.wgsl::apply_curve`(GPU 求值,0025);这里只钉 **id 常量**(Rust/WGSL
//! 两侧同值对齐,快照测试锁数值)。CR1:纯数据,native 可测。

use serde::Deserialize;

/// 曲线 id:easeOutCubic(迁移前默认;`apply_curve` 同值)。
pub const CURVE_EASE_OUT_CUBIC: u32 = 0;
/// 曲线 id:easeOutBack(表头/整表 pop 回弹)。
pub const CURVE_EASE_OUT_BACK: u32 = 1;
/// 曲线 id:`ease.enter` ≈ cubic-bezier(0, 0, 0.38, 0.9)(Carbon entrance,decelerate;
/// 一切进场)。WGSL 侧解析近似 `1-(1-t)^1.9`(maxErr ≈ 0.02,拟合见 plan25_progress)。
pub const CURVE_ENTER: u32 = 2;
/// 曲线 id:`ease.exit` ≈ cubic-bezier(0.2, 0, 1, 0.9)(Carbon exit,accelerate;一切退场)。
/// WGSL 近似 `0.5·t + 0.5·t^2.1`(maxErr ≈ 0.019)。
pub const CURVE_EXIT: u32 = 3;
/// 曲线 id:`ease.expressive` ≈ cubic-bezier(0.4, 0.14, 0.3, 1)(骨架/面板生长、大过渡)。
/// WGSL 近似 `1-(1-t^1.7)^3.4`(maxErr ≈ 0.035)。
pub const CURVE_EXPRESSIVE: u32 = 4;

/// `ease.expressive` 的 CPU 求值(与 glyph.wgsl `ease_expressive` 同拟合式):骨架/面板生长。
/// 面板 wipe 是少量实例的 CPU 每帧参数(0018 面板参数本就 CPU 喂),非逐字(RD2 不涉)。
pub fn ease_expressive(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    1.0 - (1.0 - t.powf(1.7)).powf(3.4)
}

/// 节奏/间距令牌表。字段名即 JSON 键;`#[serde(default)]` → 缺字段用默认补,局部覆盖友好
/// (同 [`Theme`](crate::theme::Theme) 模式)。时长 ms、间距 px(world unit)。
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(default)]
pub struct MotionTokens {
    /// 单字形/词内元素进场(≤ 一次注视)。喂 `FrameData.fade_ms`(glyph 淡入)。
    pub dur_glyph: f32,
    /// 行内元素、徽章、光标态变。
    pub dur_element: f32,
    /// 块/卡片进场、折叠展开。
    pub dur_block: f32,
    /// 骨架/网格/大面板生长。
    pub dur_panel: f32,
    /// 退场 = 进场 × 此系数(不对称惯例)。
    pub exit_factor: f32,
    /// 词内字符交错(ms)。
    pub stagger_glyph: f32,
    /// 列表项/表格行/cell 组交错(ms)。
    pub stagger_item: f32,
    /// 0019 Stage 间交错(ms,如网格→表头→cell)。
    pub stagger_stage: f32,
    /// 回合间距(px;分组靠间距不靠线)。
    pub space_turn: f32,
    /// 同回合 part 间距(px)。
    pub space_part: f32,
    /// 块内间距补偿(px;现仅数学溢出补偿位点,真块间距由块间换行驱动——见 plan25 遗留)。
    pub space_block_gap: f32,
    /// 卡内边距(px;user 弱底面板 outset、卡片 inset)。
    pub space_inset: f32,
    /// 到达高亮强度(M2e / design §3.3:最新揭示词短暂提亮的幅度;0 = 关,默认弱 +8%)。
    pub arrive_boost: f32,
}

impl Default for MotionTokens {
    /// 设计 §3.1 默认值(survey §3.1,Carbon/M3 折中)。`space_block_gap` 例外:保持迁移前
    /// 几何(8;真块间距机制未数据化,defer)。改此处 = 改默认观感,快照测试会红。
    fn default() -> Self {
        Self {
            dur_glyph: 200.0,
            dur_element: 240.0,
            dur_block: 320.0,
            dur_panel: 420.0,
            exit_factor: 0.8,
            stagger_glyph: 25.0,
            stagger_item: 50.0,
            stagger_stage: 60.0,
            space_turn: 24.0, // Plan 28 R1:参考 session-turn-list gap 24px
            space_part: 12.0, // Plan 28 R1:参考 assistant-content gap 12px
            space_block_gap: 8.0,
            space_inset: 16.0,
            arrive_boost: 0.08,
        }
    }
}

impl MotionTokens {
    /// 从 JSON 解析(缺字段默认补 → 局部覆盖友好)。wasm `set_motion` 用;错误由调用方处理。
    pub fn from_json(json: &str) -> Result<MotionTokens, serde_json::Error> {
        serde_json::from_str(json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::float_cmp)] // reason: 快照比较的是未经运算的字面量默认值,精确相等即意图
    fn token_table_snapshot() {
        // Plan 25 M0 验收:token 表数值稳定性快照。改默认值必须是有意的(同步设计文档),
        // 本测试红 = 提醒「你在改默认观感」。
        let m = MotionTokens::default();
        assert_eq!(
            [m.dur_glyph, m.dur_element, m.dur_block, m.dur_panel],
            [200.0, 240.0, 320.0, 420.0]
        );
        assert_eq!(m.exit_factor, 0.8);
        assert_eq!(
            [m.stagger_glyph, m.stagger_item, m.stagger_stage],
            [25.0, 50.0, 60.0]
        );
        assert_eq!(
            [m.space_turn, m.space_part, m.space_block_gap, m.space_inset],
            // Plan 28 R1:turn 24 / part 12 = opencode 参考(session-turn.css gap)。
            [24.0, 12.0, 8.0, 16.0]
        );
        assert_eq!(m.arrive_boost, 0.08);
        // 曲线 id 与 WGSL apply_curve 分支同值(改任一侧必须同步另一侧 + 本快照)。
        assert_eq!(
            [
                CURVE_EASE_OUT_CUBIC,
                CURVE_EASE_OUT_BACK,
                CURVE_ENTER,
                CURVE_EXIT,
                CURVE_EXPRESSIVE
            ],
            [0, 1, 2, 3, 4]
        );
    }

    #[test]
    #[allow(clippy::float_cmp)] // reason: 同上,比较未经运算的字面量
    fn from_json_partial_override_keeps_defaults() {
        let m = MotionTokens::from_json(r#"{"space_turn": 48.0}"#).expect("合法 JSON 必解析");
        assert_eq!(m.space_turn, 48.0);
        assert_eq!(m.dur_glyph, 200.0); // 未覆盖字段 = 默认
    }
}

/// cubicPulse(IQ,Plan 36 N3 hit-flash 包络):中心 `c` 半宽 `w` 的三次脉冲;
/// 端点恒等(x 离 c 超 w → 0),峰值 1。纯函数(R8)。
#[must_use]
pub fn cubic_pulse(c: f32, w: f32, x: f32) -> f32 {
    let mut x = (x - c).abs();
    if x > w {
        return 0.0;
    }
    x /= w.max(1.0e-6);
    1.0 - x * x * (3.0 - 2.0 * x)
}

#[cfg(test)]
mod pulse_tests {
    use super::cubic_pulse;

    /// N3:包络端点恒等(AR3 数据面)+ 峰值 + 对称 + 确定性。
    #[test]
    #[allow(clippy::float_cmp)] // reason: 端点/确定性断言就是精确相等语义
    fn cubic_pulse_endpoints_identity_peak_center() {
        assert_eq!(cubic_pulse(0.5, 0.5, 0.0), 0.0, "起点 0");
        assert_eq!(cubic_pulse(0.5, 0.5, 1.0), 0.0, "终点 0");
        assert!((cubic_pulse(0.5, 0.5, 0.5) - 1.0).abs() < 1e-6, "峰值 1");
        assert!(
            (cubic_pulse(0.5, 0.5, 0.3) - cubic_pulse(0.5, 0.5, 0.7)).abs() < 1e-6,
            "对称"
        );
        assert_eq!(
            cubic_pulse(0.5, 0.5, 0.31),
            cubic_pulse(0.5, 0.5, 0.31),
            "确定性"
        );
    }
}
