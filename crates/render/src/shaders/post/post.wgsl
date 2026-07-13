// post/post.wgsl — 屏幕空间后处理(0040 / Plan 37 F2):单 shader 参数向量化。
// 全零 = 恒等直通(同 uv 采样原像素,位等 —— present 管线因此统一取代 blit)。
// vignette:中心距压暗;grain:noise.wgsl 帧 seed 颗粒(R8:seed=注入帧计数);
// chroma:径向 RGB 通道偏移(弱档,px 级)。noise.wgsl 由 backend 前置拼接。

struct PostParams {
    vignette: f32,
    grain: f32,
    chroma_px: f32,
    grain_seed: f32,
};

@group(0) @binding(0) var src_tex: texture_2d<f32>;
@group(0) @binding(1) var src_smp: sampler;
@group(0) @binding(2) var<uniform> pp: PostParams;

struct FsIn {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_fullscreen(@builtin(vertex_index) vid: u32) -> FsIn {
    var out: FsIn;
    let x = f32(i32(vid & 1u) * 4 - 1);
    let y = f32(i32(vid >> 1u) * 4 - 1);
    out.clip = vec4<f32>(x, y, 0.0, 1.0);
    out.uv = vec2<f32>(x, -y) * 0.5 + vec2<f32>(0.5, 0.5);
    return out;
}

@fragment
fn fs_post(in: FsIn) -> @location(0) vec4<f32> {
    let dim = vec2<f32>(textureDimensions(src_tex));
    let center = in.uv - vec2<f32>(0.5, 0.5);
    // 色差:R/B 沿径向反向偏移 chroma_px(0 → 同点采样,恒等)。
    let off = center * (pp.chroma_px / max(dim.x, 1.0));
    let r = textureSample(src_tex, src_smp, in.uv + off).r;
    let gb = textureSample(src_tex, src_smp, in.uv);
    let b = textureSample(src_tex, src_smp, in.uv - off).b;
    var rgb = vec3<f32>(r, gb.g, b);
    // vignette:1 − v·smoothstep(0.35, 0.9, |center|·2)(0 → 因子 1 恒等)。
    let d = length(center) * 2.0;
    rgb *= 1.0 - pp.vignette * smoothstep(0.35, 0.9, d);
    // grain:±0.5 噪声 × 强度(0 → 加 0 恒等);seed = 注入帧计数(确定可重放)。
    let n = nz_value(in.uv * dim / 3.0, u32(pp.grain_seed)) - 0.5;
    rgb += n * pp.grain;
    return vec4<f32>(clamp(rgb, vec3<f32>(0.0), vec3<f32>(1.0)), gb.a);
}
