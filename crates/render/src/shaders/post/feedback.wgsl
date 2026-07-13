// post/feedback.wgsl — 反馈通道(0040 / Plan 37 F1)。两个入口:
//   fs_prev:全屏采样上一帧 accum × decay(叠进本帧 RT,配 Max 混合 —— 静止内容恒等:
//           max(scene, scene·decay) == scene;运动/消失留衰减残影)。
//   fs_blit:RT → surface 直通上屏。
// 全屏三角形 trick(无顶点缓冲);RGBA8 + 标准 sampler(WebGL2 无特性依赖)。

struct FbParams {
    decay: f32,
    _p0: f32,
    _p1: f32,
    _p2: f32,
};

@group(0) @binding(0) var src_tex: texture_2d<f32>;
@group(0) @binding(1) var src_smp: sampler;
@group(0) @binding(2) var<uniform> fb: FbParams;

struct FsIn {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_fullscreen(@builtin(vertex_index) vid: u32) -> FsIn {
    // 3 顶点覆盖全屏(超出裁剪);uv 同步。
    var out: FsIn;
    let x = f32(i32(vid & 1u) * 4 - 1);
    let y = f32(i32(vid >> 1u) * 4 - 1);
    out.clip = vec4<f32>(x, y, 0.0, 1.0);
    out.uv = vec2<f32>(x, -y) * 0.5 + vec2<f32>(0.5, 0.5);
    return out;
}

@fragment
fn fs_prev(in: FsIn) -> @location(0) vec4<f32> {
    let c = textureSample(src_tex, src_smp, in.uv);
    return vec4<f32>(c.rgb * fb.decay, c.a * fb.decay);
}

@fragment
fn fs_blit(in: FsIn) -> @location(0) vec4<f32> {
    return textureSample(src_tex, src_smp, in.uv);
}
