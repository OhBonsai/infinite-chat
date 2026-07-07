// markdown/pulse.wgsl — 呼吸圆角条(Plan 25 M2a / design §4.4 待机生命感)。
//
// 行内光标(文字流末尾小方块)与回合左缘指示条共用:圆角盒 SDF × 慢呼吸 alpha。
// params: [radius(px), base_alpha(0..1), freq_hz(~0.15), amp(呼吸幅度,如 0.05 = ±5%)]。
// **不是 blink**(1s ≤3 次且更柔,WCAG);time 由 globals.time_ms 驱动(注入时钟,seek 确定)。
// 恒等收敛(RD4)由发射侧保证:只在流式中(未 settled)发射,idle 一律退场 → 页面回全冻结。
//
// ⚠ fwidth 必须 top-level(非均匀控制流禁求导,见 widget.wgsl 注)。

fn md_pulse(
    local: vec2<f32>,
    halfsz: vec2<f32>,
    color: vec4<f32>,
    params: vec4<f32>,
    time_ms: f32,
) -> vec4<f32> {
    let d = sd_round_box(local, halfsz, params.x);
    let aa = max(fwidth(d), 0.0001);
    let cov = 1.0 - smoothstep(0.0, aa, d);
    // 慢呼吸:base ×(1 ± amp·sin)。0.15Hz → 周期 ~6.7s,幅度小(克制,原则 4)。
    let breath = 1.0 + params.w * sin(6.2831853 * params.z * time_ms / 1000.0);
    let a = clamp(color.a * params.y * breath, 0.0, 1.0);
    return vec4<f32>(color.rgb, a * cov);
}
