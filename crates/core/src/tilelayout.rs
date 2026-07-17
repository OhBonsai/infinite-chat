//! Tile 网格布局器(0042 §2.2):`TileSpec` + 列宽内 4 列基网格 → 每 tile 世界矩形。
//!
//! **skyline first-fit** 摆位:确定性、整数格坐标 → 每 tile 四边**恒落网格线**(对齐是摆位的必然,
//! 非事后校正),可 native 精确断言、可重放。纯计算(CR1;无平台依赖)。
//!
//! 输出的 `TileRect` 供 [`crate::app`] 在 tile 模式替换 `boxlayout` 的 `boxpos`:内容块 origin =
//! tile 内容区左上,`box_w` = tile 内宽;chrome = 每 tile 一个 panel。

use infinite_chat_primitives::tile::TileSpec;

/// 一个 tile 的世界矩形 + 网格坐标(0042 §2.1)。`(x,y)` = 左上 world(设备 px);`(w,h)` = 含 span 的像素尺寸。
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct TileRect {
    pub(crate) col: u32,
    pub(crate) row: u32,
    pub(crate) span: [u32; 2],
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) w: f32,
    pub(crate) h: f32,
}

/// 布局结果:每 tile(与 `spec.tiles` 同序)矩形 + 网格步长(断言四边落线用)+ 墙总高。
pub(crate) struct TileLayout {
    pub(crate) rects: Vec<TileRect>,
    pub(crate) col_w: f32,
    // reason: 网格步长供 native 对齐断言读(四边落线校验);运行期布局只需 rects/col_w/total_h。
    #[allow(dead_code)]
    pub(crate) col_step: f32, // col_w + gap(列网格线步长)
    #[allow(dead_code)]
    pub(crate) row_step: f32, // row_h + gap(行网格线步长)
    pub(crate) total_h: f32, // 墙外接盒高(相机 max_pan_y 用)
}

/// 在总宽 `w`(引擎 `max_width`)内按 `spec.cols` 列基网格 skyline 摆位(0042 §2.2)。
///
/// 每 tile:找**最左**的连续 `span_w` 列、放置行 = 这几列 skyline 的 **max**(顶对齐)→ 取 row 最小者、
/// 同 row 取最左;落位后把这几列 skyline 抬到 `row + span_h`。坏 spec 交调用侧 `is_valid` 先挡(此处防御性夹取)。
pub(crate) fn layout_tiles(spec: &TileSpec, w: f32) -> TileLayout {
    let cols = spec.cols.max(1);
    let gap = spec.gap.max(0.0);
    let row_h = spec.row_h.max(1.0);
    let col_w = ((w - (cols as f32 - 1.0) * gap) / cols as f32).max(1.0);
    let col_step = col_w + gap;
    let row_step = row_h + gap;

    let mut skyline = vec![0u32; cols as usize]; // 每列已填到的行号
    let mut rects = Vec::with_capacity(spec.tiles.len());
    for t in &spec.tiles {
        let sw = t.span[0].clamp(1, cols);
        let sh = t.span[1].max(1);
        // 候选:每个可放的最左列 c,其行 = c..c+sw 列 skyline 的 max(顶对齐);取行最小、同行最左。
        let mut best_col = 0u32;
        let mut best_row = u32::MAX;
        for c in 0..=(cols - sw) {
            let row = (c..c + sw).map(|i| skyline[i as usize]).max().unwrap_or(0);
            if row < best_row {
                best_row = row;
                best_col = c;
            }
        }
        let row = if best_row == u32::MAX { 0 } else { best_row };
        for i in best_col..best_col + sw {
            skyline[i as usize] = row + sh;
        }
        let x = best_col as f32 * col_step;
        let y = row as f32 * row_step;
        let tw = sw as f32 * col_w + (sw as f32 - 1.0) * gap;
        let th = sh as f32 * row_h + (sh as f32 - 1.0) * gap;
        rects.push(TileRect {
            col: best_col,
            row,
            span: [sw, sh],
            x,
            y,
            w: tw,
            h: th,
        });
    }
    let max_row = skyline.iter().copied().max().unwrap_or(0);
    let total_h = if max_row == 0 {
        0.0
    } else {
        max_row as f32 * row_step - gap // 末行不含尾 gap
    };
    TileLayout {
        rects,
        col_w,
        col_step,
        row_step,
        total_h,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use infinite_chat_primitives::tile::TilePlacement;

    fn tile(id: &str, w: u32, h: u32) -> TilePlacement {
        TilePlacement {
            id: id.into(),
            span: [w, h],
            title: String::new(),
        }
    }

    fn spec(tiles: Vec<TilePlacement>) -> TileSpec {
        TileSpec {
            cols: 4,
            gap: 16.0,
            row_h: 140.0,
            pad: 14.0,
            title_h: 26.0,
            tiles,
        }
    }

    /// 硬断言(0042 §3 对齐维):**每 tile 四边落网格线**。左/上 = k*step;右/下 = (k+span)*step - gap。
    #[test]
    fn every_edge_on_grid_line() {
        let s = spec(vec![
            tile("a", 2, 1),
            tile("b", 2, 1),
            tile("c", 4, 2),
            tile("d", 1, 1),
            tile("e", 2, 2),
        ]);
        let lay = layout_tiles(&s, 1200.0);
        for r in &lay.rects {
            // 左缘 = col * col_step。
            let lx = r.col as f32 * lay.col_step;
            assert!((r.x - lx).abs() < 1e-3, "左缘不在网格线: {} vs {lx}", r.x);
            // 上缘 = row * row_step。
            let ly = r.row as f32 * lay.row_step;
            assert!((r.y - ly).abs() < 1e-3, "上缘不在网格线: {} vs {ly}", r.y);
            // 右缘 = (col+span_w)*col_step - gap。
            let rx = (r.col + r.span[0]) as f32 * lay.col_step - s.gap;
            assert!(
                (r.x + r.w - rx).abs() < 1e-3,
                "右缘不在网格线: {} vs {rx}",
                r.x + r.w
            );
            // 下缘 = (row+span_h)*row_step - gap。
            let by = (r.row + r.span[1]) as f32 * lay.row_step - s.gap;
            assert!(
                (r.y + r.h - by).abs() < 1e-3,
                "下缘不在网格线: {} vs {by}",
                r.y + r.h
            );
        }
    }

    /// skyline 摆位:满行(2+2)后换行;宽 tile(4×2)独占;顶对齐同行。
    #[test]
    fn skyline_packs_rows_top_aligned() {
        let s = spec(vec![
            tile("a", 2, 1), // r0 c0
            tile("b", 2, 1), // r0 c2(填满 row0)
            tile("c", 4, 2), // r1 c0(独占 2 行)
            tile("d", 2, 1), // r3 c0
        ]);
        let lay = layout_tiles(&s, 1200.0);
        assert_eq!((lay.rects[0].col, lay.rects[0].row), (0, 0));
        assert_eq!((lay.rects[1].col, lay.rects[1].row), (2, 0));
        assert_eq!((lay.rects[2].col, lay.rects[2].row), (0, 1));
        // c(4×2) 抬所有列 skyline 到 row3 → d 落 row3。
        assert_eq!((lay.rects[3].col, lay.rects[3].row), (0, 3));
        // 同行顶对齐:a、b 同 y。
        assert!((lay.rects[0].y - lay.rects[1].y).abs() < 1e-3);
    }

    /// 1×1 填缝:窄 tile 落最左空列。
    #[test]
    fn small_tile_fills_leftmost_gap() {
        let s = spec(vec![
            tile("wide", 3, 1),  // r0 c0-2
            tile("small", 1, 1), // r0 c3(填右缝,非换行)
        ]);
        let lay = layout_tiles(&s, 1200.0);
        assert_eq!((lay.rects[1].col, lay.rects[1].row), (3, 0));
    }

    /// 列宽 = (W - (cols-1)*gap)/cols;gap 全等。
    #[test]
    fn column_width_and_uniform_gap() {
        let s = spec(vec![tile("a", 1, 1)]);
        let lay = layout_tiles(&s, 1200.0);
        let expect_col_w = (1200.0 - 3.0 * 16.0) / 4.0;
        assert!((lay.col_w - expect_col_w).abs() < 1e-3);
        assert!((lay.col_step - (expect_col_w + 16.0)).abs() < 1e-3);
    }
}
