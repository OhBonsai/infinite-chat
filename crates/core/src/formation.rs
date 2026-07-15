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

/// 黄金角(phyllotaxis 均匀填充,无方向伪影)。
const GOLDEN_ANGLE: f32 = 2.399_963_2;

fn bbox_center(centers: &[[f32; 2]]) -> [f32; 2] {
    let (mut left, mut top, mut right, mut bottom) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
    for c in centers {
        left = left.min(c[0]);
        top = top.min(c[1]);
        right = right.max(c[0]);
        bottom = bottom.max(c[1]);
    }
    [(left + right) * 0.5, (top + bottom) * 0.5]
}

fn frac(x: f32) -> f32 {
    x - x.floor()
}

/// k 瓣花填充:round-robin 把字均分到 k 个花瓣,每瓣是一枚沿花瓣轴的锥形瓣叶(base→tip 填,
/// 中段最宽两端收)→ 花瓣**清晰分离**(比 `|cos(kθ)|` 包络更像花)。`a`=花半径 px;`seed`=朝向相位。
fn rose_points(n: usize, center: [f32; 2], a: f32, k: u32, seed: u32) -> Vec<[f32; 2]> {
    let k = k.max(1);
    let per = (n as f32 / k as f32).ceil().max(1.0);
    let phase = (seed as f32) * 0.001;
    (0..n)
        .map(|i| {
            let petal = (i as u32 % k) as f32;
            let idx_in = (i as u32 / k) as f32;
            let along = 0.14 + 0.86 * (idx_in + 0.5) / per; // 0.14 起 → 花心留小空、瓣根分离
            let axis = core::f32::consts::TAU * petal / k as f32 + phase;
            let r = a * along;
            let halfw = a * 0.26 * (core::f32::consts::PI * along).sin(); // 窄瓣(相邻瓣不糊成一团)
            let side = (frac(i as f32 * 0.618_034) - 0.5) * 2.0; // 黄金比横向铺满瓣宽
            let (ca, sa) = (axis.cos(), axis.sin());
            [
                center[0] + ca * r - sa * side * halfw,
                center[1] + sa * r + ca * side * halfw,
            ]
        })
        .collect()
}

/// 心形填充点(备选形状,证「形状=数据」):经典心形参数式,phyllotaxis 角 + 径向填充。
fn heart_points(n: usize, center: [f32; 2], scale: f32) -> Vec<[f32; 2]> {
    let s = scale * 0.062;
    (0..n)
        .map(|i| {
            let fi = i as f32;
            let t = fi * GOLDEN_ANGLE;
            let rf = ((fi + 0.5) / n as f32).sqrt();
            let hx = 16.0 * t.sin().powi(3);
            let hy =
                -(13.0 * t.cos() - 5.0 * (2.0 * t).cos() - 2.0 * (3.0 * t).cos() - (4.0 * t).cos());
            [center[0] + hx * s * rf, center[1] + hy * s * rf]
        })
        .collect()
}

/// 角度桶贪心匹配:glyph 与 point 各按(方位角, 半径)排序,同序配对 → 保角、避轨迹大交叉。
/// 返回 `assign[glyph_i] = point_j`(双射,n==n)。O(n log n)。
fn match_by_angle(glyph_centers: &[[f32; 2]], points: &[[f32; 2]], center: [f32; 2]) -> Vec<usize> {
    let key = |p: &[f32; 2]| {
        let (dx, dy) = (p[0] - center[0], p[1] - center[1]);
        (dy.atan2(dx), dx.hypot(dy))
    };
    let cmp = |ka: (f32, f32), kb: (f32, f32)| ka.0.total_cmp(&kb.0).then(ka.1.total_cmp(&kb.1));
    let mut gi: Vec<usize> = (0..glyph_centers.len()).collect();
    let mut pj: Vec<usize> = (0..points.len()).collect();
    gi.sort_by(|&a, &b| cmp(key(&glyph_centers[a]), key(&glyph_centers[b])));
    pj.sort_by(|&a, &b| cmp(key(&points[a]), key(&points[b])));
    let mut assign = vec![0usize; glyph_centers.len()];
    for (g, p) in gi.iter().zip(pj.iter()) {
        assign[*g] = *p;
    }
    assign
}

/// 把 N 个形状点(与 glyph 数等）匹配回 glyph 序,并按目标半径给 stagger(外瓣后到 → 向外绽放)。
fn matched_targets(centers: &[[f32; 2]], points: &[[f32; 2]], center: [f32; 2]) -> Vec<Target> {
    let assign = match_by_angle(centers, points, center);
    let maxr = points
        .iter()
        .map(|p| (p[0] - center[0]).hypot(p[1] - center[1]))
        .fold(1.0_f32, f32::max);
    (0..centers.len())
        .map(|i| {
            let tp = points[assign[i]];
            let r = (tp[0] - center[0]).hypot(tp[1] - center[1]);
            Target {
                center: tp,
                stagger: (r / maxr).clamp(0.0, 1.0),
            }
        })
        .collect()
}

/// 花瓣落轨闭式(与 rect.wgsl mode 5 **同式**,勿漂移):`age`(秒)→ `[sway_x, fall_y]`(px)。
/// 端点 `age=0` 归原点(RD4:落定/未生前恒等);纯 f(seed,vy,age) 无时基外输入 → 定帧确定(R8)。
pub fn petal_offset(seed: f32, vy: f32, age: f32) -> [f32; 2] {
    let sway = 36.0 * ((age * 1.6 + seed).sin() - seed.sin());
    [sway, vy * age]
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
        Shape::Rose { a, k } => {
            let center = bbox_center(centers);
            let points = rose_points(n, center, *a, *k, spec.seed);
            matched_targets(centers, &points, center)
        }
        Shape::Heart { scale } => {
            let center = bbox_center(centers);
            let points = heart_points(n, center, *scale);
            matched_targets(centers, &points, center)
        }
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

    fn dist(a: [f32; 2], b: [f32; 2]) -> f32 {
        (a[0] - b[0]).hypot(a[1] - b[1])
    }

    #[test]
    fn rose_targets_bijection_and_shape_is_data() {
        let centers: Vec<[f32; 2]> = (0..60)
            .map(|i| [(i % 12) as f32 * 20.0, (i / 12) as f32 * 20.0])
            .collect();
        let rose = compute_targets(
            &centers,
            &parse_spec(r#"{"shape":"rose","a":200,"k":5,"seed":3}"#).expect("valid"),
        );
        assert_eq!(rose.len(), 60);
        // 双射:60 个 target 互不重复(匹配到 60 个互异玫瑰点)。
        for i in 0..rose.len() {
            for j in (i + 1)..rose.len() {
                assert!(
                    !approx(rose[i].center, rose[j].center),
                    "target {i}/{j} 重复"
                );
            }
        }
        // 形状=数据:换 heart → 目标集不同(非同一批点)。
        let heart = compute_targets(
            &centers,
            &parse_spec(r#"{"shape":"heart","scale":300}"#).expect("valid"),
        );
        assert_eq!(heart.len(), 60);
        assert!(
            !approx(rose[0].center, heart[0].center) || !approx(rose[30].center, heart[30].center)
        );
    }

    #[test]
    fn petal_fall_endpoints() {
        // age=0 → 原点(未落);age>0 → 落 y = vy·age,sway 有界(±72)。
        let at0 = petal_offset(1.3, 150.0, 0.0);
        assert!(approx(at0, [0.0, 0.0]));
        let at2 = petal_offset(1.3, 150.0, 2.0);
        assert!((at2[1] - 300.0).abs() < 1e-3, "落轨 y = vy·age");
        assert!(at2[0].abs() <= 72.0, "sway 有界");
    }

    #[test]
    fn angle_bucket_beats_arbitrary_matching() {
        // glyphs 环形分布;角度桶匹配总飞行距离应 ≤ 生成序(任意)匹配的 70%(基准入账)。
        let n = 120usize;
        let centers: Vec<[f32; 2]> = (0..n)
            .map(|i| {
                let a = i as f32 * 0.3;
                [300.0 + a.cos() * 150.0, 300.0 + a.sin() * 150.0]
            })
            .collect();
        let center = bbox_center(&centers);
        let points = rose_points(n, center, 200.0, 5, 0);
        let assign = match_by_angle(&centers, &points, center);
        let bucket: f32 = (0..n).map(|i| dist(centers[i], points[assign[i]])).sum();
        let arbitrary: f32 = (0..n).map(|i| dist(centers[i], points[i])).sum();
        assert!(
            bucket <= arbitrary * 0.7,
            "角度桶总距离 {bucket:.0} 应 ≤ 0.7×任意 {arbitrary:.0}"
        );
    }
}
