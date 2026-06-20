// shaderbox/common.wgsl — ShaderBox 公共底座(Plan 16 / 0028)。Globals + 实例解包 + 顶点 + **SDF
// 工具箱**(PixelSpiritDeck/LYGIA,GLSL→WGSL 直译,见 plan16 §2.5a / 附录 A)+ ShadeCtx。
// 每个 effect 文件接着定义 `fn shade(c: ShadeCtx) -> vec4<f32>`,末尾 fs.wgsl 提供 fs_main 调用它。
// 拼接顺序(backend with_sdf):base/sdf.wgsl + common.wgsl + <effect>.wgsl + fs.wgsl(声明先于使用)。

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
    @location(0) pos: vec2<f32>,      // 左上角世界 px
    @location(1) size: vec2<f32>,     // 宽高 px(= resolution)
    @location(2) params0: vec4<f32>,  // params[0..4](p0.x 常 = icon_id)
    @location(3) params1: vec4<f32>,  // params[4..8]
    @location(4) bg: vec4<f32>,       // 背景 RGBA
    @location(5) time: f32,           // shader time(秒)
};

struct VsOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>,                       // 0..1(左上原点)
    @location(1) @interpolate(flat) resolution: vec2<f32>,
    @location(2) @interpolate(flat) p0: vec4<f32>,
    @location(3) @interpolate(flat) p1: vec4<f32>,
    @location(4) @interpolate(flat) bg: vec4<f32>,
    @location(5) @interpolate(flat) time: f32,
};

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
    out.uv = c;
    out.resolution = inst.size;
    out.p0 = inst.params0;
    out.p1 = inst.params1;
    out.bg = inst.bg;
    out.time = inst.time;
    return out;
}

/// 片元上下文(传给各 effect 的 `shade`)。`st`/`uv` = 0..1;`time` 秒;`p0/p1` = params。
struct ShadeCtx {
    uv: vec2<f32>,
    resolution: vec2<f32>,
    time: f32,
    p0: vec4<f32>,
    p1: vec4<f32>,
};

// ── 常量 ──
const SB_PI: f32 = 3.14159265;
const SB_TAU: f32 = 6.2831853;
const SB_QTR_PI: f32 = 0.78539816;

// 呼吸宏 S/C(源 `#define S (1.+sin(iTime)/4.)`)。
fn sb_S(t: f32) -> f32 { return 1.0 + sin(t) / 4.0; }
fn sb_C(t: f32) -> f32 { return 1.0 + cos(t) / 4.0; }

// ── SDF 工具箱(括号 = 源卡编号;GLSL→WGSL 直译,plan16 §2.5a)──
fn sstep(a: f32, b: f32) -> f32 { return smoothstep(a - 0.005, a + 0.005, b); } // AA step
fn sb_stroke(x: f32, s: f32, w: f32) -> f32 { // 04
    let d = sstep(s, x + w / 2.0) - sstep(s, x - w / 2.0);
    return clamp(d, 0.0, 1.0);
}
fn circleSDF(st: vec2<f32>) -> f32 { return length(st - 0.5) * 2.0; } // 08
fn sb_fill(x: f32, size: f32) -> f32 { return 1.0 - sstep(size, x); } // 09
fn rectSDF(st: vec2<f32>, s: vec2<f32>) -> f32 { // 10
    let p = st * 2.0 - 1.0;
    return max(abs(p.x / s.x), abs(p.y / s.y));
}
fn crossSDF(st: vec2<f32>, s: f32) -> f32 { // 11
    let size = vec2<f32>(0.25, s);
    return min(rectSDF(st, size.xy), rectSDF(st, size.yx));
}
fn sb_flip(v: f32, pct: f32) -> f32 { return mix(v, 1.0 - v, pct); } // 12
fn vesicaSDF(st: vec2<f32>, w: f32) -> f32 { // 14
    let offset = vec2<f32>(w * 0.5, 0.0);
    return max(circleSDF(st - offset), circleSDF(st + offset));
}
fn triSDF(st: vec2<f32>) -> f32 { // 16
    let p = (2.0 * st - 1.0) * 2.0;
    return max(abs(p.x) * 0.866025 + p.y * 0.5, -p.y * 0.5);
}
fn rhombSDF(st: vec2<f32>) -> f32 { // 17
    return max(triSDF(st), triSDF(vec2<f32>(st.x, 1.0 - st.y)));
}
fn sb_rotate(st: vec2<f32>, a: f32) -> vec2<f32> { // 19
    let m = mat2x2<f32>(cos(a), -sin(a), sin(a), cos(a));
    return m * (st - 0.5) + 0.5;
}
fn polySDF(st: vec2<f32>, v: i32) -> f32 { // 26
    let p = st * 2.0 - 1.0;
    let a = atan2(p.x, p.y) + SB_PI;
    let r = length(p);
    let seg = SB_TAU / f32(v);
    return cos(floor(0.5 + a / seg) * seg - a) * r;
}
fn hexSDF(st: vec2<f32>) -> f32 { // 27
    let p = abs(st * 2.0 - 1.0);
    return max(abs(p.y), p.x * 0.866025 + p.y * 0.5);
}
fn starSDF(st: vec2<f32>, v: i32, s: f32) -> f32 { // 28
    let p = st * 4.0 - 2.0;
    var a = atan2(p.y, p.x) / SB_TAU;
    let seg = a * f32(v);
    a = ((floor(seg) + 0.5) / f32(v) + mix(s, -s, step(0.5, fract(seg)))) * SB_TAU;
    return abs(dot(vec2<f32>(cos(a), sin(a)), p));
}
fn raysSDF(st: vec2<f32>, n: i32) -> f32 { // 30
    let p = st - 0.5;
    return fract(atan2(p.y, p.x) / SB_TAU * f32(n));
}
fn heartSDF(st: vec2<f32>) -> f32 { // 34
    let q = st - vec2<f32>(0.5, 0.8);
    let r = length(q) * 5.0;
    let n = normalize(q);
    return r - ((n.y * pow(abs(n.x), 0.67)) / (n.y + 1.5) - 2.0 * n.y + 1.26);
}
fn sb_bridge(c: f32, d: f32, s: f32, w: f32) -> f32 { // 35
    var cc = c * (1.0 - sb_stroke(d, s, w * 2.0));
    return cc + sb_stroke(d, s, w);
}
fn spiralSDF(st: vec2<f32>, t: f32) -> f32 { // 47
    let p = st - 0.5;
    let r = dot(p, p);
    let a = atan2(p.y, p.x);
    return abs(sin(fract(log(r) * t + a * 0.159)));
}
fn sb_scale(st: vec2<f32>, s: vec2<f32>) -> vec2<f32> { return (st - 0.5) * s + 0.5; }
fn flowerSDF(st: vec2<f32>, n: i32) -> f32 {
    let p = st * 2.0 - 1.0;
    let r = length(p) * 2.0;
    let a = atan2(p.y, p.x);
    let v = f32(n) * 0.5;
    return 1.0 - (abs(cos(a * v)) * 0.5 + 0.5) / r;
}
