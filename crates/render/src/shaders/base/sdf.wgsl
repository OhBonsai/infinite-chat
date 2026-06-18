// base/sdf.wgsl — 共享 SDF 形状/算子函数库(0026)。
//
// **无 entry point、无 binding**,纯函数。build 期由 backend.rs 前置拼接到需要的 pipeline
// 源前(rect/panel/markdown widget),WGSL 要求"声明先于使用",故本库置于调用方之前。
// 唯一出处:形状函数在此定义一份,各 pipeline 复用(0026 共识①:复用底层 SDF 函数,不复用上层图元)。

/// 圆角矩形有符号距离场。`p` 为以矩形中心为原点的点,`b` 为半尺寸,`r` 为圆角内陷(均世界 px)。
fn sd_round_box(p: vec2<f32>, b: vec2<f32>, r: f32) -> f32 {
    let q = abs(p) - b + vec2<f32>(r, r);
    return min(max(q.x, q.y), 0.0) + length(max(q, vec2<f32>(0.0, 0.0))) - r;
}

/// 点 `p` 到线段 `a→b` 的距离(用于勾选对勾、下划线等线形)。
fn sd_seg(p: vec2<f32>, a: vec2<f32>, b: vec2<f32>) -> f32 {
    let pa = p - a;
    let ba = b - a;
    let h = clamp(dot(pa, ba) / max(dot(ba, ba), 1.0e-6), 0.0, 1.0);
    return length(pa - ba * h);
}

/// 圆/点 SDF(半径 r)。
fn sd_circle(p: vec2<f32>, r: f32) -> f32 {
    return length(p) - r;
}

/// 描边算子:把实心 SDF 变成宽 `w` 的环(|d| - w/2)。
fn op_outline(d: f32, w: f32) -> f32 {
    return abs(d) - w * 0.5;
}

/// 平滑并(smin,polynomial,0025/Plan 10 §4 备用):k 越大融合越圆。
fn op_smin(a: f32, b: f32, k: f32) -> f32 {
    let h = clamp(0.5 + 0.5 * (b - a) / max(k, 1.0e-6), 0.0, 1.0);
    return mix(b, a, h) - k * h * (1.0 - h);
}
