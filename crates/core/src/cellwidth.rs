//! cellwidth(Plan 46 L0)—— TUI cell 网格的**逐字宽度表**(East Asian Width 语义)。
//!
//! 终端把每个 grapheme 量成整数 **cell**:半宽(拉丁/数字/ASCII)=1、宽/全角(CJK 统一表意/假名/
//! 谚文/全角/CJK 标点/emoji)=2、组合记号(Mn/Me)与零宽 =0。语义承 UAX#11 + `unicode-width`
//! 既定做法(codex 用 `unicode-width`、lipgloss 用 rune width),**不脑补**(§0-3)。表驱动纯函数
//! (CR1,native 全测);宽区间扩自 `web/src/layout-bridge.ts:246` 的 `isCJK`。

/// 一个字符占几个 cell(0/1/2)。控制字符/换行/零宽 → 0(不占格;`\n` 由布局器另作断行)。
#[must_use]
pub(crate) fn char_cells(ch: char) -> u8 {
    let cp = ch as u32;
    // 0 宽:控制、零宽、组合记号(近似 Mn/Me 常用区间)。
    if is_zero_width(cp) {
        return 0;
    }
    if is_wide(cp) {
        return 2;
    }
    1
}

/// 一段文本(按 char)的 cell 总宽。
#[must_use]
pub(crate) fn str_cells(s: &str) -> u32 {
    s.chars().map(|c| u32::from(char_cells(c))).sum()
}

/// 区间表(闭区间);线性查表。
fn in_ranges(cp: u32, ranges: &[(u32, u32)]) -> bool {
    ranges.iter().any(|&(lo, hi)| (lo..=hi).contains(&cp))
}

/// 零宽:C0/C1 控制、格式控制、零宽空格家族、组合记号(Mn/Me 常用区间近似;疑难容差豁免 §6)。
fn is_zero_width(cp: u32) -> bool {
    const Z: &[(u32, u32)] = &[
        (0x0000, 0x001F),
        (0x007F, 0x009F),
        (0x200B, 0x200F),
        (0x202A, 0x202E),
        (0x2060, 0x2064),
        (0xFEFF, 0xFEFF),
        (0x0300, 0x036F),
        (0x0483, 0x0489),
        (0x0591, 0x05BD),
        (0x0610, 0x061A),
        (0x064B, 0x065F),
        (0x0670, 0x0670),
        (0x1AB0, 0x1AFF),
        (0x1DC0, 0x1DFF),
        (0x20D0, 0x20FF),
        (0xFE00, 0xFE0F),
        (0xFE20, 0xFE2F),
    ];
    in_ranges(cp, Z)
}

/// 宽/全角(2 cell):CJK 家族(扩自 isCJK)+ Hangul Jamo + 全角 + emoji 面。
fn is_wide(cp: u32) -> bool {
    const W: &[(u32, u32)] = &[
        (0x1100, 0x115F),   // Hangul Jamo
        (0x2E80, 0x303E),   // CJK 部首/康熙/CJK 标点
        (0x3041, 0x33FF),   // 假名/注音/CJK 兼容
        (0x3400, 0x4DBF),   // CJK 扩展 A
        (0x4E00, 0x9FFF),   // CJK 统一表意
        (0xA000, 0xA4CF),   // 彝文
        (0xAC00, 0xD7A3),   // 谚文音节
        (0xF900, 0xFAFF),   // CJK 兼容表意
        (0xFE10, 0xFE19),   // 竖排标点
        (0xFE30, 0xFE6F),   // CJK 兼容形/小写变体
        (0xFF00, 0xFF60),   // 全角 ASCII
        (0xFFE0, 0xFFE6),   // 全角符号
        (0x1F000, 0x1F0FF), // 麻将/多米诺/扑克
        (0x1F300, 0x1FAFF), // emoji & 象形符号面
        (0x2600, 0x27BF),   // 杂项符号 & dingbats
        (0x20000, 0x3FFFD), // CJK 扩展 B–F
    ];
    in_ranges(cp, W)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn latin_and_digits_are_one_cell() {
        for ch in ['A', 'z', '0', '9', ' ', '!', '~', '@'] {
            assert_eq!(char_cells(ch), 1, "{ch:?} 应 1 cell");
        }
    }

    #[test]
    fn cjk_and_fullwidth_are_two_cells() {
        for ch in ['中', '文', 'あ', 'ア', '한', '，', '。', '　', 'Ａ', '１'] {
            assert_eq!(char_cells(ch), 2, "{ch:?} 应 2 cell");
        }
    }

    #[test]
    fn emoji_is_two_cells() {
        for ch in ['🌸', '✅', '🎉', '❤'] {
            assert_eq!(char_cells(ch), 2, "{ch:?} 应 2 cell(emoji)");
        }
    }

    #[test]
    fn combining_and_control_are_zero_cells() {
        // 组合尖音符(U+0301)、零宽空格(U+200B)、换行(U+000A)。
        assert_eq!(char_cells('\u{0301}'), 0);
        assert_eq!(char_cells('\u{200B}'), 0);
        assert_eq!(char_cells('\n'), 0);
    }

    #[test]
    fn mixed_str_cells_accumulate() {
        // "字A" = 2 + 1 = 3;"ab中" = 1+1+2 = 4。
        assert_eq!(str_cells("字A"), 3);
        assert_eq!(str_cells("ab中"), 4);
        assert_eq!(str_cells("hello"), 5);
    }
}
