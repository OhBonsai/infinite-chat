//! codeblock(M15 / Plan 15 / 0027)— 代码块**行窗视口**的纯逻辑:固定 6 行盒高、流式 tail、边缘
//! 淡入淡出 alpha、双向滚动 clamp。纯函数(native 可测,CR1);几何/字形发射在 [`crate::app`] 集成,
//! 视觉(淡入淡出 / 滚动手感 / 横裁)须人工 GPU。
//!
//! 行窗(§2.1):代码块超 `MAX_LINES` 行只露窗口那几行,盒高稳定(不顶下文,plan13 §4.5 锚底友好);
//! 软裁剪(§2.2):窗外字 cull,窗内**仅"还有更多"那侧**按 [`FADE_BAND`] 淡;tail(§2.3):活动块
//! scrollY 自动跟最新窗。

/// 行窗最大行数(0027 §2.1;0021 可配,v1 固定 6)。
pub(crate) const MAX_LINES: usize = 6;
/// 边缘淡入淡出带宽(× lineH;§2.2)。
pub(crate) const FADE_BAND: f32 = 0.75;

/// 行窗盒高(§2.1):`min(N, MAX_LINES) · lineH`。N≤窗 → 实高;超窗 → 钉死窗高(不顶下文)。
pub(crate) fn window_height(n_lines: usize, line_h: f32) -> f32 {
    n_lines.min(MAX_LINES) as f32 * line_h.max(0.0)
}

/// 最大可滚行数(§2.5 clamp 上界):`max(0, N − MAX_LINES)`。
pub(crate) fn max_scroll_lines(n_lines: usize) -> i32 {
    i32::try_from(n_lines.saturating_sub(MAX_LINES)).unwrap_or(i32::MAX)
}

/// 流式 tail 的 scrollY(§2.3):跟最新 `MAX_LINES` 行 = `max(0, N − MAX_LINES)`。
pub(crate) fn tail_scroll(n_lines: usize) -> i32 {
    max_scroll_lines(n_lines)
}

/// scrollY(行)clamp 到 `[0, max_scroll_lines]`(§2.5)。手动滚动(Plan 15 ④)用。
#[allow(dead_code)] // reason: Plan 15 ④ 手动滚动接入前先备(已有单测覆盖)
pub(crate) fn clamp_scroll_y(scroll_y: i32, n_lines: usize) -> i32 {
    scroll_y.clamp(0, max_scroll_lines(n_lines))
}

/// scrollX(px)clamp 到 `[0, max(0, max_line_w − code_view_w)]`(§2.5 横向)。手动横滚(④)用。
#[allow(dead_code)] // reason: Plan 15 ④ 横向滚动接入前先备
pub(crate) fn clamp_scroll_x(scroll_x: f32, max_line_w: f32, code_view_w: f32) -> f32 {
    scroll_x.clamp(0.0, (max_line_w - code_view_w).max(0.0))
}

/// 边缘淡入淡出 alpha 系数(§2.2)。`y_in_view` = 字在窗内 y(px,0=窗顶,`view_h`=窗底);`scroll_y`/
/// `max_scroll` 决定哪侧"还有更多"才淡(顶端 scrollY=0 不淡顶、底端到底不淡底)。返回 `[0,1]` 乘到字 alpha。
pub(crate) fn edge_fade(
    y_in_view: f32,
    view_h: f32,
    line_h: f32,
    scroll_y: i32,
    max_scroll: i32,
) -> f32 {
    let fb = (FADE_BAND * line_h).max(1.0);
    // 上方还有内容(scrollY>0)→ 顶端淡出;否则顶端实(=1)。
    let a_top = if scroll_y > 0 {
        (y_in_view / fb).clamp(0.0, 1.0)
    } else {
        1.0
    };
    // 下方还有内容(未到底)→ 底端淡入;否则底端实。
    let a_bottom = if scroll_y < max_scroll {
        ((view_h - y_in_view) / fb).clamp(0.0, 1.0)
    } else {
        1.0
    };
    a_top.min(a_bottom)
}

/// 字是否在行窗外(§2.2 cull):`y_in_view < 0`(滚出顶)或 `≥ view_h`(滚出底)→ 不发射。
pub(crate) fn culled(y_in_view: f32, view_h: f32) -> bool {
    y_in_view < 0.0 || y_in_view >= view_h
}

/// 行号 gutter 宽(§2.4):`digits(N) · char_w + pad`。Plan 15 ② 行号 gutter 用。
#[allow(dead_code)] // reason: Plan 15 ② 行号 gutter 接入前先备
pub(crate) fn gutter_width(n_lines: usize, char_w: f32, pad: f32) -> f32 {
    let digits = n_lines.max(1).to_string().len();
    digits as f32 * char_w + pad
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn box_height_clamps_at_max_lines() {
        // N≤窗 → 实高;超窗 → 钉死 6 行(plan §4 用例)。
        assert!((window_height(3, 20.0) - 60.0).abs() < 1e-6, "3 行 = 3·20");
        assert!((window_height(6, 20.0) - 120.0).abs() < 1e-6, "6 行 = 满窗");
        assert!(
            (window_height(100, 20.0) - 120.0).abs() < 1e-6,
            "超窗钉死 6 行"
        );
    }

    #[test]
    fn tail_and_clamp() {
        assert_eq!(tail_scroll(3), 0, "不足窗不滚");
        assert_eq!(tail_scroll(10), 4, "10 行 tail = 10-6");
        assert_eq!(clamp_scroll_y(99, 10), 4, "上界夹");
        assert_eq!(clamp_scroll_y(-3, 10), 0, "下界夹");
        assert!((clamp_scroll_x(999.0, 500.0, 200.0) - 300.0).abs() < 1e-6);
        assert!(
            clamp_scroll_x(50.0, 100.0, 200.0).abs() < 1e-6,
            "行不超视口不横滚"
        );
    }

    #[test]
    fn edge_fade_top_and_bottom() {
        let (view_h, lh) = (120.0, 20.0); // 6 行窗,fb = 0.75·20 = 15
                                          // 顶端 scrollY=0:不淡顶(a_top=1);底端未到底:底淡入。
        assert!(
            (edge_fade(0.0, view_h, lh, 0, 4) - 1.0).abs() < 1e-6,
            "顶端 scrollY=0 不淡"
        );
        assert!(
            edge_fade(view_h, view_h, lh, 0, 4).abs() < 1e-6,
            "最底沿淡到 0(下方还有)"
        );
        // 滚到中间(两侧都有):顶端淡出。
        assert!(
            edge_fade(0.0, view_h, lh, 2, 4).abs() < 1e-6,
            "滚动中顶端淡到 0"
        );
        // 滚到底:底端不淡(a_bottom=1)。
        assert!(
            (edge_fade(view_h, view_h, lh, 4, 4) - 1.0).abs() < 1e-6,
            "到底不淡底"
        );
        // 窗内中段:满 alpha。
        assert!(
            (edge_fade(60.0, view_h, lh, 2, 4) - 1.0).abs() < 1e-6,
            "中段满 alpha"
        );
    }

    #[test]
    fn cull_outside_window() {
        assert!(culled(-0.1, 120.0), "滚出顶");
        assert!(culled(120.0, 120.0), "滚出底(含端)");
        assert!(!culled(0.0, 120.0), "窗顶在内");
        assert!(!culled(119.0, 120.0), "窗内");
    }

    #[test]
    fn gutter_width_by_digits() {
        assert!((gutter_width(9, 10.0, 8.0) - 18.0).abs() < 1e-6, "1 位");
        assert!(
            (gutter_width(120, 10.0, 8.0) - 38.0).abs() < 1e-6,
            "3 位 = 30+8"
        );
    }
}
