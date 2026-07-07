// markdown/badge.wgsl — tool 卡状态徽章(Plan 25 M2b/M4 / design §4.6):圆环/圆点/勾/叉 SDF。
//
// params: [from_shape, stroke(px), morph_t(0..1), to_shape](形状:0=pending 环,1=running 点,
// 2=done 勾,3=error 叉)。M4(Plan 10 相位 4 最小件):状态迁移 = **距离场 `mix` 形变**
// d = mix(d_from, d_to, t) —— SDF 值层的 MixT(0025)。稳态 from==to、t=1 恒等(AR3)。
// 颜色由实例 color 携带(状态→色在 core 决策)。全部分支用 select 收敛(无控制流 →
// fwidth 恒 top-level 均匀,见 widget.wgsl 注)。

fn badge_shape_d(local: vec2<f32>, r: f32, stroke: f32, shape: f32) -> f32 {
    let d_ring = abs(sd_circle(local, r)) - stroke * 0.5;
    let d_dot = sd_circle(local, r * 0.72);
    let d_check = min(
        sd_seg(local, vec2<f32>(-r * 0.70, 0.05 * r), vec2<f32>(-r * 0.12, r * 0.55)),
        sd_seg(local, vec2<f32>(-r * 0.12, r * 0.55), vec2<f32>(r * 0.75, -r * 0.50)),
    ) - stroke * 0.5;
    let d_cross = min(
        sd_seg(local, vec2<f32>(-r * 0.60, -r * 0.60), vec2<f32>(r * 0.60, r * 0.60)),
        sd_seg(local, vec2<f32>(-r * 0.60, r * 0.60), vec2<f32>(r * 0.60, -r * 0.60)),
    ) - stroke * 0.5;
    var d = d_ring;
    d = select(d, d_dot, shape > 0.5);
    d = select(d, d_check, shape > 1.5);
    d = select(d, d_cross, shape > 2.5);
    return d;
}

fn md_badge(
    local: vec2<f32>,
    halfsz: vec2<f32>,
    color: vec4<f32>,
    params: vec4<f32>,
) -> vec4<f32> {
    let r = min(halfsz.x, halfsz.y) * 0.78;
    let stroke = max(params.y, 1.0);
    let d_from = badge_shape_d(local, r, stroke, params.x);
    let d_to = badge_shape_d(local, r, stroke, params.w);
    let d = mix(d_from, d_to, clamp(params.z, 0.0, 1.0));
    let aa = max(fwidth(d), 0.0001);
    let cov = 1.0 - smoothstep(0.0, aa, d);
    return vec4<f32>(color.rgb, color.a * cov);
}
