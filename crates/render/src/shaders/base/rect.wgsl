// rect.wgsl — 矩形/圆角图元(Plan 4B 装饰底/条 + 4C3 调试框)。
//
// 与 glyph.wgsl 共用 Globals(同相机/视口),独立管线(无 atlas 采样)。圆角/描边用
// 圆角矩形 SDF + fwidth 抗锯齿,故任意缩放边缘锐利。文字**之前**绘制 → 作背景。

struct Globals {
    viewport: vec2<f32>,
    time_ms: f32,
    fade_ms: f32,
    cam_pan: vec2<f32>,
    cam_zoom: f32,
    _pad: f32,
};

@group(0) @binding(0) var<uniform> globals: Globals;

struct InstanceIn {
    @location(0) pos: vec2<f32>,    // 左上角世界 px
    @location(1) size: vec2<f32>,   // 宽高 px
    @location(2) color: vec4<f32>,  // RGBA
    @location(3) radius: f32,       // 圆角半径 px
    @location(4) stroke: f32,       // 描边宽 px;0 = 实心填充
    @location(5) gloop: vec4<f32>,  // [prev_dx, prev_w, next_dx, next_w];w<=0 = 无邻接
};

struct VsOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) local: vec2<f32>,                 // 以矩形中心为原点的世界 px
    @location(1) @interpolate(flat) halfsz: vec2<f32>,
    @location(2) @interpolate(flat) color: vec4<f32>,
    @location(3) @interpolate(flat) radius: f32,
    @location(4) @interpolate(flat) stroke: f32,
    @location(5) @interpolate(flat) gloop: vec4<f32>,
};

// 圆角矩形 SDF `sd_round_box` 由 base/sdf.wgsl 提供(backend.rs 前置拼接,0026)。

@vertex
fn vs_main(@builtin(vertex_index) vid: u32, inst: InstanceIn) -> VsOut {
    var corners = array<vec2<f32>, 4>(
        vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 0.0),
        vec2<f32>(0.0, 1.0), vec2<f32>(1.0, 1.0),
    );
    let c = corners[vid];
    let world = inst.pos + c * inst.size;
    let screen = (world - globals.cam_pan) * globals.cam_zoom;
    let ndc = vec2<f32>(
        screen.x / globals.viewport.x * 2.0 - 1.0,
        1.0 - screen.y / globals.viewport.y * 2.0,
    );
    var out: VsOut;
    out.clip = vec4<f32>(ndc, 0.0, 1.0);
    out.local = (c - vec2<f32>(0.5, 0.5)) * inst.size;
    out.halfsz = inst.size * 0.5;
    out.color = inst.color;
    out.radius = min(inst.radius, min(out.halfsz.x, out.halfsz.y));
    out.stroke = inst.stroke;
    out.gloop = inst.gloop;
    return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    // gloop(Plan 32 D3,makepad draw_selection 手法):对上/下邻行盒(视为同高,y 平移
    // ±一行)与本盒做 smooth-union(op_smin,sdf.wgsl)→ 相邻同色行底融成连续圆角块,
    // 接缝角被 smin 场填平;两邻皆无(w<=0)时 d 不变 = 独立圆角(退化正确,像素不变)。
    // k = 2×radius:k=radius 时接缝角点(场值 ≈ 0.414r,smin 只压 k/4)残缺口,2r 才闭合。
    // 邻盒凸出本 quad 的部分被 quad 天然裁掉 —— 等宽带(diff 行底)无凸出;异宽留待选区租户。
    var d = sd_round_box(in.local, in.halfsz, in.radius);
    if in.gloop.y > 0.0 || in.gloop.w > 0.0 {
        let k = max(in.radius * 2.0, 1.0);
        if in.gloop.y > 0.0 {
            let c = vec2<f32>(in.gloop.x + in.gloop.y * 0.5 - in.halfsz.x, -2.0 * in.halfsz.y);
            let dp = sd_round_box(in.local - c, vec2<f32>(in.gloop.y * 0.5, in.halfsz.y), in.radius);
            d = op_smin(d, dp, k);
        }
        if in.gloop.w > 0.0 {
            let c = vec2<f32>(in.gloop.z + in.gloop.w * 0.5 - in.halfsz.x, 2.0 * in.halfsz.y);
            let dn = sd_round_box(in.local - c, vec2<f32>(in.gloop.w * 0.5, in.halfsz.y), in.radius);
            d = op_smin(d, dn, k);
        }
    }
    let aa = max(fwidth(d), 0.0001);
    let inside = 1.0 - smoothstep(-aa, aa, d);
    // stroke>0:仅内边一圈(outer 减去内陷 stroke 的实心)→ 调试框/边。
    let inner = 1.0 - smoothstep(-aa, aa, d + in.stroke);
    let cov = select(inside, inside - inner, in.stroke > 0.0);
    return vec4<f32>(in.color.rgb, in.color.a * cov);
}
