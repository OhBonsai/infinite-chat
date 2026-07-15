//! formation — Plan 42 字阵编队:目标形状采样 + glyph→target 匹配(**纯函数,native 可测**)。
//!
//! 形状=数据(§0-3):目标图案是参数(shape + seed),不是硬编码编排。编队全程只在 VS 位移
//! (presentation,AR2/AR3);本模块只算「每个 glyph 的目标中心 + 起飞相位」,不碰布局/命中。

use serde::Deserialize;

/// 编队目标形状(serde tag = "shape";坏数据整拒 AR12)。
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(tag = "shape", rename_all = "lowercase")]
pub enum Shape {
    /// 方格(G1 通道验证):`cols` 列,`cell` 间距 px。
    Grid { cols: u32, cell: f32 },
    /// 玫瑰线(G2 花):`r = a·cos(k·θ)`,`k` = 花瓣数;`a` = 幅值 px。
    Rose { a: f32, k: u32 },
    /// 心形(G2 备选,证明「形状=数据」):`scale` px。
    Heart { scale: f32 },
}

/// 一次编队的完整规格(shape + 采样 seed)。
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct FormationSpec {
    #[serde(flatten)]
    pub shape: Shape,
    #[serde(default)]
    pub seed: u32,
}

/// 单 glyph 的编队目标:中心(世界 px)+ 起飞相位 stagger(0..1)。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Target {
    pub center: [f32; 2],
    pub stagger: f32,
}

/// 解析 shape JSON;坏数据整份拒绝(AR12,不 panic)。
pub fn parse_spec(json: &str) -> Result<FormationSpec, String> {
    serde_json::from_str(json).map_err(|e| e.to_string())
}

/// 计算每个 glyph 的目标(与输入 `centers` **同序**返回)。花心 = 输入中心的包围盒中心。
/// G1 grid:按输入序铺格(调用方保证输入序稳定 → 确定性);G2 花在此扩玫瑰线 + 角度桶匹配。
pub fn compute_targets(centers: &[[f32; 2]], spec: &FormationSpec) -> Vec<Target> {
    let n = centers.len();
    if n == 0 {
        return Vec::new();
    }
    let (mut left, mut top, mut right, mut bottom) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
    for c in centers {
        left = left.min(c[0]);
        top = top.min(c[1]);
        right = right.max(c[0]);
        bottom = bottom.max(c[1]);
    }
    let (cx, cy) = ((left + right) * 0.5, (top + bottom) * 0.5);
    match &spec.shape {
        Shape::Grid { cols, cell } => {
            let cols = (*cols).max(1) as usize;
            let cell = *cell;
            let rows = n.div_ceil(cols);
            let w = (cols.saturating_sub(1)) as f32 * cell;
            let h = (rows.saturating_sub(1)) as f32 * cell;
            (0..n)
                .map(|i| {
                    let (r, c) = (i / cols, i % cols);
                    Target {
                        center: [
                            cx - w * 0.5 + c as f32 * cell,
                            cy - h * 0.5 + r as f32 * cell,
                        ],
                        stagger: i as f32 / n as f32, // 顺序波(G2 换环形/角度桶)
                    }
                })
                .collect()
        }
        // G2 实现:玫瑰线采样 + 角度桶贪心匹配;G1 先恒等占位(不改位)。
        Shape::Rose { .. } | Shape::Heart { .. } => centers
            .iter()
            .map(|c| Target {
                center: *c,
                stagger: 0.0,
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_rejects_bad_json() {
        assert!(parse_spec("not json").is_err());
        assert!(parse_spec(r#"{"shape":"unknown"}"#).is_err());
        assert!(parse_spec(r#"{"shape":"grid","cols":4,"cell":20}"#).is_ok());
    }

    fn approx(a: [f32; 2], b: [f32; 2]) -> bool {
        (a[0] - b[0]).abs() < 1e-3 && (a[1] - b[1]).abs() < 1e-3
    }

    #[test]
    fn grid_targets_deterministic_and_centered() {
        let centers = vec![[0.0, 0.0], [100.0, 0.0], [0.0, 100.0], [100.0, 100.0]];
        let spec = parse_spec(r#"{"shape":"grid","cols":2,"cell":10,"seed":7}"#).expect("valid");
        let a = compute_targets(&centers, &spec);
        let b = compute_targets(&centers, &spec);
        assert_eq!(a, b, "同输入 → 逐字节确定");
        assert_eq!(a.len(), 4);
        // 花心 = 包围盒中心 (50,50);2×2 格、cell 10 → 角点 (45,45)..(55,55)。
        assert!(approx(a[0].center, [45.0, 45.0]));
        assert!(approx(a[3].center, [55.0, 55.0]));
        // stagger 递增 [0,.25,.5,.75]
        assert!(a[0].stagger < a[3].stagger);
    }

    #[test]
    fn empty_input_empty_output() {
        let spec = parse_spec(r#"{"shape":"grid","cols":2,"cell":10}"#).expect("valid");
        assert!(compute_targets(&[], &spec).is_empty());
    }
}
