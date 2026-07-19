// celllayout (Plan 46 L1/L2) -- TUI-flavor cell-grid per-grapheme layouter (pure fn, CR1, native-testable).
//
// Input: spans + wrap_cols (cell budget) + CellMetrics -> LayoutResult (same shape as seam.rs:
// one PlacedGlyph per grapheme + block_height). x = col*cell_w, y = line*cell_h, col/line integer
// -> lands on the integer cell grid (DoD-3). Wrap = word-preferred + CJK-breakable + head-ban
// (codex live_wrap / opencode HEAD_BAN semantics, web/src/layout-bridge.ts:260). R8: cell size is
// an injected constant, zero measurement at layout time.
//
// Pure bypass: only called by ensure_layouts when tui_cell_active() (tui flavor + Column +
// calibrated); rich/tile/uncalibrated -> original JS per-glyph path, byte-identical (iron law 0-1).
//
// NOTE: comments here are ASCII-only on purpose -- CJK in this file's comments tripped a
// clippy/rustc span ICE (rustc_span multi-byte assertion) under the workspace pedantic lints.

use crate::app::CellMetrics;
use crate::cellwidth::{char_cells, str_cells};
use crate::seam::{LayoutResult, PlacedGlyph};
use infinite_chat_primitives::style::StyledSpan;
use infinite_chat_primitives::text::graphemes;

// Head-ban set (must not start a line -> hangs onto the previous line's end).
// Ported from layout-bridge.ts:260 HEAD_BAN; codepoint array to avoid literal CJK in source.
const HEAD_BAN: &[char] = &[
    '\u{3002}', '\u{ff0c}', '\u{3001}', '\u{ff1b}', '\u{ff1a}', '\u{ff1f}', '\u{ff01}', '\u{ff09}',
    '\u{300d}', '\u{300f}', '\u{3011}', '\u{300b}', '\u{3009}', '\u{201c}', '\u{201d}', '\u{2019}',
    '%', '\u{2026}', '\u{00b7}', '.', ',', ';', ':', '?', '!',
];

fn char_is_head_ban(ch: char) -> bool {
    HEAD_BAN.contains(&ch)
}

// A placed cell: integer col/line grid position + cluster cell width.
struct Cell {
    cells: u32,
    col: u32,
    line: u32,
}

// CJK / wide (2 cells) = per-char break opportunity; approximated by the first char being 2 cells.
fn is_break_after(cluster: &str) -> bool {
    cluster == " " || cluster.chars().next().is_some_and(|c| char_cells(c) == 2)
}

fn is_head_ban(cluster: &str) -> bool {
    cluster.chars().next().is_some_and(char_is_head_ban)
}

// Cell-grid layout (DoD-3). wrap_cols = per-line cell budget (>=1); m = cell size.
#[must_use]
pub(crate) fn layout_cells(spans: &[StyledSpan], wrap_cols: u32, m: CellMetrics) -> LayoutResult {
    let cols = wrap_cols.max(1);
    let mut cells: Vec<Cell> = Vec::new();
    let mut col: u32 = 0;
    let mut line: u32 = 0;
    // Index of the last break opportunity (space/CJK) on the current line.
    let mut last_break: Option<usize> = None;

    for span in spans {
        for g in graphemes(span.text()) {
            if g == "\n" {
                cells.push(Cell {
                    cells: 0,
                    col,
                    line,
                });
                col = 0;
                line += 1;
                last_break = None;
                continue;
            }
            let w = str_cells(g);
            // Need to wrap: doesn't fit, line already has content, and this char isn't head-ban
            // (a head-ban char hangs on the current line, allowed to exceed the budget).
            if col + w > cols && col > 0 && !is_head_ban(g) {
                if let Some(bi) = last_break.filter(|&bi| bi + 1 < cells.len()) {
                    // Word-preferred: move the trailing partial word (after the break) to the next line.
                    line += 1;
                    let mut c = 0u32;
                    for cell in &mut cells[bi + 1..] {
                        cell.line = line;
                        cell.col = c;
                        c += cell.cells;
                    }
                    col = c;
                } else {
                    // No break point (word longer than a line) -> newline (long word degrades to
                    // per-cell breaking, same as a terminal).
                    line += 1;
                    col = 0;
                }
                last_break = None;
            }
            cells.push(Cell {
                cells: w,
                col,
                line,
            });
            col += w;
            if is_break_after(g) {
                last_break = Some(cells.len() - 1);
            }
        }
    }

    let glyphs: Vec<PlacedGlyph> = cells
        .iter()
        .map(|c| PlacedGlyph {
            pos: [c.col as f32 * m.cell_w, c.line as f32 * m.cell_h],
            size: [c.cells as f32 * m.cell_w, m.cell_h],
        })
        .collect();
    let block_height = if cells.is_empty() {
        0.0
    } else {
        (line + 1) as f32 * m.cell_h
    };
    LayoutResult {
        glyphs,
        block_height,
        table_panels: Vec::new(),
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp)] // reason: cell positions are exact (col*cell_w integer math) -- exact cmp is the assertion
mod tests {
    use super::*;
    use infinite_chat_primitives::style::StyleRole;

    fn m() -> CellMetrics {
        CellMetrics {
            cell_w: 8.0,
            cell_h: 18.0,
        }
    }
    fn span(t: &str) -> StyledSpan {
        StyledSpan::new(t, StyleRole::Normal)
    }

    // Latin: 1 cell each, x on the integer cell grid (col*cell_w), y=0 (single line).
    #[test]
    fn latin_places_on_integer_cell_grid() {
        let r = layout_cells(&[span("abc")], 80, m());
        assert_eq!(r.glyphs.len(), 3);
        for (i, g) in r.glyphs.iter().enumerate() {
            assert_eq!(g.pos[0], i as f32 * 8.0, "glyph {i} x = col*cell_w");
            assert_eq!(g.pos[1], 0.0);
            assert_eq!(g.size, [8.0, 18.0], "1-cell box");
        }
        assert_eq!(r.block_height, 18.0, "single line height = cell_h");
    }

    // CJK = 2 cells: mixed CJK/Latin x accumulates (a=1 -> cjk=2 -> b=1).
    #[test]
    fn cjk_two_cells_and_mixed_advance() {
        let r = layout_cells(&[span("a\u{4e2d}b")], 80, m()); // "a中b"
        assert_eq!(r.glyphs[0].pos[0], 0.0);
        assert_eq!(r.glyphs[0].size[0], 8.0, "a = 1 cell");
        assert_eq!(r.glyphs[1].pos[0], 8.0);
        assert_eq!(r.glyphs[1].size[0], 16.0, "cjk = 2 cells");
        assert_eq!(r.glyphs[2].pos[0], 24.0, "b after cjk (1+2)");
    }

    // Word-preferred wrap: break at the space, trailing word moves down.
    #[test]
    fn word_preferred_wrap_at_space() {
        // wrap_cols=5: "ab cd ef" -> "ab cd" (5 cells) full, "ef" next line.
        let r = layout_cells(&[span("ab cd ef")], 5, m());
        let last2: Vec<u32> = r
            .glyphs
            .iter()
            .rev()
            .take(2)
            .map(|g| (g.pos[1] / 18.0) as u32)
            .collect();
        assert!(last2.iter().all(|&l| l == 1), "ef on line 2");
        assert_eq!(r.glyphs[0].pos[1], 0.0);
    }

    // Head-ban: a full-width period does not start a line, hangs on the prev line (may exceed budget).
    #[test]
    fn head_ban_hangs_to_prev_line() {
        // wrap_cols=3: "abc。" -> the period would wrap but is head-ban -> hangs on line 0 (x=3 cells).
        let r = layout_cells(&[span("abc\u{3002}")], 3, m());
        let dot = r.glyphs.last().expect("has glyphs");
        assert_eq!(dot.pos[1], 0.0, "period hangs on line 0 (not line start)");
        assert_eq!(dot.pos[0], 24.0, "period after abc (col=3)");
    }

    // Newline grapheme: col resets, line+1; glyph count includes \n (1:1 with grapheme order).
    #[test]
    fn newline_resets_col_advances_line() {
        let r = layout_cells(&[span("a\nb")], 80, m());
        assert_eq!(r.glyphs.len(), 3, "a \\n b = 3 graphemes");
        assert_eq!(r.glyphs[2].pos[1], 18.0, "b on line 2");
        assert_eq!(r.glyphs[2].pos[0], 0.0, "b at line start");
        assert_eq!(r.block_height, 36.0, "two lines");
    }
}
