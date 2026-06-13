// glyph.wgsl — 实例化字形 + 着色器淡入(M9/M10)。
//
// 一个 instance = 一个 grapheme quad(triangle-strip 4 顶点)。淡入完全靠
// `time_ms - spawn_time` 在 GPU 算,CPU 零参与(0002 §5)。fade_ms=0 即关闭(参数置零,
// 非分支,满足 AR3 恒等收敛)。

struct Globals {
    viewport: vec2<f32>,   // 画布像素尺寸
    time_ms: f32,          // 当前帧时间
    fade_ms: f32,          // 淡入时长;0 = 不淡入
};

@group(0) @binding(0) var<uniform> globals: Globals;
@group(0) @binding(1) var atlas_tex: texture_2d<f32>;
@group(0) @binding(2) var atlas_samp: sampler;

struct InstanceIn {
    @location(0) pos: vec2<f32>,       // 左上角 world px
    @location(1) size: vec2<f32>,      // 字形宽高 px
    @location(2) uv: vec4<f32>,        // atlas: u0,v0,u1,v1
    @location(3) spawn_time: f32,
    @location(4) style: u32,           // StyleRole 数值
};

struct VsOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) alpha: f32,
    @location(2) tint: vec3<f32>,
};

// StyleRole → 上色(对应 content::StyleRole 的数值)。文字位图为白色,tint 直接相乘;
// emoji 走 Normal(tint=白)不被染色。
fn style_color(s: u32) -> vec3<f32> {
    switch s {
        case 1u: { return vec3<f32>(1.0, 1.0, 1.0); }        // Bold
        case 2u: { return vec3<f32>(0.85, 0.85, 0.90); }     // Italic
        case 3u: { return vec3<f32>(1.0, 1.0, 1.0); }        // BoldItalic
        case 4u, 5u: { return vec3<f32>(0.60, 0.85, 0.70); } // Code / CodeBlock
        case 6u: { return vec3<f32>(0.55, 0.78, 1.0); }      // Heading
        case 7u: { return vec3<f32>(0.45, 0.70, 1.0); }      // Link
        case 8u, 9u: { return vec3<f32>(0.62, 0.62, 0.68); } // Quote / ListMarker
        default: { return vec3<f32>(0.90, 0.90, 0.92); }     // Normal
    }
}

@vertex
fn vs_main(@builtin(vertex_index) vid: u32, inst: InstanceIn) -> VsOut {
    // strip 角点:0=(0,0) 1=(1,0) 2=(0,1) 3=(1,1)
    var corners = array<vec2<f32>, 4>(
        vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 0.0),
        vec2<f32>(0.0, 1.0), vec2<f32>(1.0, 1.0),
    );
    let c = corners[vid];
    let px = inst.pos + c * inst.size;
    // world px(左上原点)→ NDC(中心原点,y 向上)
    let ndc = vec2<f32>(
        px.x / globals.viewport.x * 2.0 - 1.0,
        1.0 - px.y / globals.viewport.y * 2.0,
    );
    var out: VsOut;
    out.clip = vec4<f32>(ndc, 0.0, 1.0);
    out.uv = vec2<f32>(mix(inst.uv.x, inst.uv.z, c.x), mix(inst.uv.y, inst.uv.w, c.y));
    let age = globals.time_ms - inst.spawn_time;
    // fade_ms<=0 时 alpha 直接为 1(max 保护除零)。
    out.alpha = select(clamp(age / max(globals.fade_ms, 1.0), 0.0, 1.0), 1.0, globals.fade_ms <= 0.0);
    out.tint = style_color(inst.style);
    return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let texel = textureSample(atlas_tex, atlas_samp, in.uv);
    // 文字位图为白色 → tint 上色;emoji(Normal,tint≈白)保留本色。coverage 取 a。
    return vec4<f32>(texel.rgb * in.tint, texel.a * in.alpha);
}
