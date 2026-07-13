//! grapheme 切分(0039 下沉自 core::support):吐字/光栅化单位 = grapheme cluster(AR7)。

use unicode_segmentation::UnicodeSegmentation;

/// 文本 → grapheme cluster 列表(与 web 侧 `Intl.Segmenter` 同单位,BR8)。
#[must_use]
pub fn graphemes(s: &str) -> Vec<&str> {
    s.graphemes(true).collect()
}
