// shaderbox/fs.wgsl — ShaderBox 片元入口(拼接顺序末尾;调前面 effect 的 `shade`)。
// fragment 色 over 背景(alpha 混合,0028 §2.1)。

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    var ctx: ShadeCtx;
    ctx.uv = in.uv;
    ctx.resolution = in.resolution;
    ctx.time = in.time;
    ctx.p0 = in.p0;
    ctx.p1 = in.p1;
    let s = shade(ctx);
    let a = clamp(s.a, 0.0, 1.0);
    let rgb = mix(in.bg.rgb, s.rgb, a);
    return vec4<f32>(rgb, max(in.bg.a, a));
}
