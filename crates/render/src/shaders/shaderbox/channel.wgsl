// shaderbox/channel.wgsl — 输入纹理 channel0 + 溶解/扫光示例(Plan 16 ④)。
// channel0 纹理走 **group(1)**(复用 image 纹理 bind group:binding0=texture_2d, binding1=sampler)。
// 数据驱动(0021):`p0.x` = 进度(0=空,1=全显);`p0.y` = 模式(<0.5 噪声溶解 / ≥0.5 左→右扫光)。
// 输出 = 纹理色 × reveal(alpha);over 下层由 fs.wgsl 合成。

@group(1) @binding(0) var ch0_tex: texture_2d<f32>;
@group(1) @binding(1) var ch0_samp: sampler;

fn ch_hash(p: vec2<f32>) -> f32 {
    return fract(sin(dot(p, vec2<f32>(127.1, 311.7))) * 43758.5453);
}

fn shade(c: ShadeCtx) -> vec4<f32> {
    let tex = textureSample(ch0_tex, ch0_samp, c.uv);
    let progress = clamp(c.p0.x, 0.0, 1.0);
    var reveal: f32;
    if (c.p0.y >= 0.5) {
        // 扫光:左→右软边推进(progress 推进显示边界)。
        reveal = smoothstep(progress - 0.12, progress, 1.0 - c.uv.x);
    } else {
        // 噪声溶解:每像素(粗格)阈值 < progress → 显;软边过渡,避免硬跳。
        let thr = ch_hash(floor(c.uv * c.resolution * 0.5));
        reveal = smoothstep(thr - 0.08, thr + 0.08, progress);
    }
    return vec4<f32>(tex.rgb, tex.a * clamp(reveal, 0.0, 1.0));
}
