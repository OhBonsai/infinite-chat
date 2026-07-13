// shaderbox/noise_debug.wgsl — 噪声库四象限可视化(Plan 36 N1 debug case)。
// 象限:左上 value / 右上 gradient / 左下 fbm(4 oct)/ 右下 voronoi F1。
// p0.x = seed(f32 → u32);静态(不吃 time → 帧间恒等,golden 可逐字节)。
// 前置拼接:common.wgsl + noise.wgsl 在本文件之前(backend.rs)。

fn shade(c: ShadeCtx) -> vec4<f32> {
    let seed = u32(max(c.p0.x, 0.0));
    let q = c.uv * 2.0; // [0,2)^2 → 四象限
    let cell = vec2<u32>(u32(q.x), u32(q.y));
    let local = fract(q) * 8.0; // 每象限 8×8 个噪声周期
    var v = 0.0;
    if cell.x == 0u && cell.y == 0u {
        v = nz_value(local, seed);
    } else if cell.x == 1u && cell.y == 0u {
        v = nz_gradient(local, seed);
    } else if cell.x == 0u && cell.y == 1u {
        v = nz_fbm(local, seed, 4u);
    } else {
        v = clamp(nz_voronoi_f1(local, seed), 0.0, 1.0);
    }
    // 象限分隔细线(可视化辅助)。
    let line = step(fract(q.x), 0.01) + step(fract(q.y), 0.01);
    let g = clamp(v, 0.0, 1.0);
    return vec4<f32>(mix(vec3<f32>(g), vec3<f32>(0.3, 0.5, 0.9), clamp(line, 0.0, 1.0) * 0.6), 1.0);
}
