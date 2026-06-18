// markdown/box.wgsl — 复选/确认框组件(0026/Plan 11 §3)。
//
// 纯组件函数,无 entry/binding;由 markdown/widget.wgsl 的 fs_main 按 component id 调用。
// 依赖 base/sdf.wgsl 的 sd_round_box / sd_seg(backend.rs 前置拼接 sdf→box→widget)。
//
// params:.x=圆角半径 px  .y=描边宽 px  .z=勾选进度(0..1)  .w=保留。
// 画法:圆角方框描边环(SDF outline)+ 勾选时叠 SDF 对勾(两段线 sd_seg,归一到框半尺寸空间)。

fn md_box(local: vec2<f32>, half: vec2<f32>, color: vec4<f32>, params: vec4<f32>) -> vec4<f32> {
    let radius = params.x;
    let stroke = max(params.y, 1.0);
    let check = clamp(params.z, 0.0, 1.0);
    let r = min(radius, min(half.x, half.y));

    // 框边环:实心覆盖 inside 减去内陷 stroke 的 inner。
    let d = sd_round_box(local, half, r);
    let aa = max(fwidth(d), 0.0001);
    let inside = 1.0 - smoothstep(-aa, aa, d);
    let inner = 1.0 - smoothstep(-aa, aa, d + stroke);
    let ring = clamp(inside - inner, 0.0, 1.0);

    // 勾选对勾(归一到 [-1,1],y 向下):短段下行 + 长段上行;线半宽 cw。
    // ⚠ 导数(fwidth)必须在**均匀控制流**里求(WebGPU/Tint 硬性要求)→ 不放进 `if check>0`,
    // 而是无条件算出 dc/ca,再用 check 作系数(未勾 check=0 → 勾自然不显)。否则 Chrome 判 shader
    // 非法 → 画 widget 即整 pass 失败黑屏(naga 较宽松不报,故本地 cargo test 看不出)。
    let n = local / max(half, vec2<f32>(1.0, 1.0));
    let dc = min(
        sd_seg(n, vec2<f32>(-0.45, 0.05), vec2<f32>(-0.1, 0.4)),
        sd_seg(n, vec2<f32>(-0.1, 0.4), vec2<f32>(0.5, -0.45)),
    );
    let cw = 0.14;
    let ca = fwidth(dc) + 0.001;
    let checkcov = (1.0 - smoothstep(cw - ca, cw + ca, dc)) * check;

    let cov = max(ring, checkcov);
    return vec4<f32>(color.rgb, color.a * cov);
}
