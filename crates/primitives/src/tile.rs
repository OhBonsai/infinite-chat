//! Tile 网格布局数据契约(0042)。manifest 驱动的 Metro 式组件墙:**布局 = 数据**。
//!
//! 纯数据(serde;JSON 可加载),零平台依赖(CR1);core `tilelayout` 消费它算每 tile 世界矩形。
//! 换墙 / 改 span **零代码**——只改 manifest。

use serde::Deserialize;

/// 一个 tile 的声明(0042 §2.1):内容归属 id + 网格 span + 标题。
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct TilePlacement {
    /// = store 里该 tile 内容的 `messageID`(块归属靠它反查)。
    pub id: String,
    /// `(span_w 列, span_h 行)`;异构**只在此**,不在偏移。
    pub span: [u32; 2],
    /// 标题条 glyph 文案(Mono eyebrow 风);缺省空。
    #[serde(default)]
    pub title: String,
}

/// 一面墙的 tile 规格(0042 §2.1)。字段名即 JSON 键;`#[serde(default)]` → 局部覆盖友好。
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(default)]
pub struct TileSpec {
    /// 基网格列数(设计表 = 4)。
    pub cols: u32,
    /// 统一 gap(格间 + 行间;单一 token)。
    pub gap: f32,
    /// 基行高(1 格的像素高;设备 px)。
    pub row_h: f32,
    /// tile 内衬(panel 边 → 内容)。
    pub pad: f32,
    /// 标题条高(panel header)。
    pub title_h: f32,
    /// tile 列表(manifest 顺序 = skyline 摆位顺序)。
    pub tiles: Vec<TilePlacement>,
}

impl Default for TileSpec {
    fn default() -> Self {
        Self {
            cols: 4,
            gap: 16.0,
            row_h: 140.0,
            pad: 14.0,
            title_h: 26.0,
            tiles: Vec::new(),
        }
    }
}

impl TileSpec {
    /// 解析 manifest JSON。坏 JSON → `Err`(host 侧整拒退单列,AR12)。
    ///
    /// # Errors
    /// JSON 语法/类型不符时返回 `serde_json::Error`。
    pub fn from_json(json: &str) -> Result<TileSpec, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// 合法性(0042 §2.5):`cols>0`、几何量非负、`row_h>0`、非空、每 tile `span` 两维 ≥1 且 `span_w ≤ cols`。
    /// 坏 → `false`(整拒退 `Column`,不 panic)。
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.cols > 0
            && self.gap >= 0.0
            && self.row_h > 0.0
            && self.pad >= 0.0
            && self.title_h >= 0.0
            && !self.tiles.is_empty()
            && self
                .tiles
                .iter()
                .all(|t| t.span[0] >= 1 && t.span[1] >= 1 && t.span[0] <= self.cols)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_manifest_json() {
        let json = r#"{"cols":4,"gap":16,"row_h":140,"tiles":[
            {"id":"a","span":[2,1],"title":"USER"},
            {"id":"b","span":[4,2],"title":"DIFF"}
        ]}"#;
        let spec = TileSpec::from_json(json).expect("valid manifest");
        assert_eq!(spec.cols, 4);
        assert_eq!(spec.tiles.len(), 2);
        assert_eq!(spec.tiles[1].span, [4, 2]);
        assert!(spec.is_valid());
    }

    #[test]
    fn rejects_bad_span() {
        // span_w=5 > cols=4 → 非法(AR12 整拒)。
        let spec = TileSpec {
            tiles: vec![TilePlacement {
                id: "x".into(),
                span: [5, 1],
                title: String::new(),
            }],
            ..TileSpec::default()
        };
        assert!(!spec.is_valid());
    }

    #[test]
    fn rejects_zero_span_and_empty() {
        let zero = TileSpec {
            tiles: vec![TilePlacement {
                id: "x".into(),
                span: [0, 1],
                title: String::new(),
            }],
            ..TileSpec::default()
        };
        assert!(!zero.is_valid());
        assert!(!TileSpec::default().is_valid(), "空 tiles → 非法");
    }
}
