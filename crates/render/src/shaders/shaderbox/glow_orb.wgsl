// shaderbox/glow_orb.wgsl — Agent 回复 logo(Plan 16 §2.6):noise 调制的发光环(**自写** + 通用
// hash 噪声,非 verbatim 抄 shadertoy)。环色/脉冲走 params(0021 数据驱动):
//   p0.xyz = 环色(0 → 默认蓝);p0.w = 脉冲速度(0 → 1.0);p1.x = 内半径(0 → 0.6)。

fn sb_hash21(p: vec2<f32>) -> f32 {
    var q = fract(p * vec2<f32>(123.34, 345.45));
    q += dot(q, q + 34.345);
    return fract(q.x * q.y);
}

// 值噪声(双线性插值 hash;LYGIA 同范式,通用实现)。
fn sb_vnoise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);
    let a = sb_hash21(i);
    let b = sb_hash21(i + vec2<f32>(1.0, 0.0));
    let cc = sb_hash21(i + vec2<f32>(0.0, 1.0));
    let d = sb_hash21(i + vec2<f32>(1.0, 1.0));
    return mix(mix(a, b, u.x), mix(cc, d, u.x), u.y);
}

fn shade(c: ShadeCtx) -> vec4<f32> {
    let p = (c.uv - 0.5) * 2.0; // -1..1
    let r = length(p);
    let ang = atan2(p.y, p.x);

    let speed = select(1.0, c.p0.w, c.p0.w > 0.0);
    let inner = select(0.6, c.p1.x, c.p1.x > 0.0);
    let t = c.time * speed;

    // 噪声调制环半径 + 脉冲(呼吸)。
    let pulse = 0.5 + 0.5 * sin(t * 2.0);
    let wobble = (sb_vnoise(vec2<f32>(ang * 1.5, t)) - 0.5) * 0.12;
    let ring_r = inner + wobble;
    let ring = smoothstep(0.16, 0.0, abs(r - ring_r));

    // 角向高光(一道亮点绕环转)+ 径向辉光衰减。
    let hi = pow(max(0.0, cos(ang - t * 1.7)), 6.0) * ring;
    let glow = smoothstep(1.0, 0.0, r) * 0.35 * (0.7 + 0.3 * pulse);

    let intensity = clamp(ring * (0.6 + 0.4 * pulse) + hi * 0.8 + glow, 0.0, 1.0);
    var base = vec3<f32>(0.30, 0.55, 1.0);
    if (c.p0.x + c.p0.y + c.p0.z > 0.0) {
        base = c.p0.rgb;
    }
    let col = mix(base, vec3<f32>(0.9, 0.95, 1.0), hi + pulse * 0.3);
    return vec4<f32>(col, intensity);
}
