// glyph.wgsl — SDF 文字图元(Plan 3 K)。
//
// atlas tile 是 R8 单通道**距离场**(0.5 = 字形边缘)。覆盖率用 smoothstep + 屏幕导数
// fwidth 抗锯齿 → **任意缩放清晰**(无边画布硬需求)。保留 spawn_time GPU 淡入、StyleRole
// 上色。富特效(发光/描边/溶解,0007)在此片元层加几行即可(本期不做)。

struct Globals {
    viewport: vec2<f32>,   // 画布像素尺寸
    time_ms: f32,          // 当前帧时间
    fade_ms: f32,          // 淡入时长;0 = 不淡入
    cam_pan: vec2<f32>,    // 相机:屏幕左上角对应的世界坐标
    cam_zoom: f32,         // 相机缩放
    arrive_boost: f32, // M2e 到达高亮(0=关)
    form_progress: f32,    // Plan 42 编队进度(0=恒等)
    // 对齐填充(3×f32,各 align 4)→ wind 落在 offset 48,与 Rust [f32;3] 一致(勿用 vec3,其 align 16 会错位)。
    _form_pad0: f32,
    _form_pad1: f32,
    _form_pad2: f32,
    wind: vec4<f32>,       // Plan 42 指针风场 [pos.xy, radius, strength]
};

@group(0) @binding(0) var<uniform> globals: Globals;
@group(0) @binding(1) var atlas_tex: texture_2d_array<f32>;
@group(0) @binding(2) var atlas_samp: sampler;
@group(0) @binding(3) var msdf_tex: texture_2d_array<f32>;  // 离线烘焙 MSDF(0015,RGBA 页)

struct InstanceIn {
    @location(0) pos: vec2<f32>,       // 左上角世界 px
    @location(1) size: vec2<f32>,      // 宽高 px
    @location(2) uv: vec4<f32>,        // tile UV: u0,v0,u1,v1
    @location(3) spawn_time: f32,
    @location(4) style: u32,           // StyleRole
    @location(5) layer: u32,           // atlas 页(纹理数组层)
    @location(6) kind: u32,            // 字形源:0=位图覆盖率 1=TinySDF 2=MSDF 3=RGBA
    @location(7) anim: u32,            // 进场动画 profile id(0025/Plan10 §3b;core 据角色+reveal风格选)
    @location(8) alpha: f32,           // 静态 alpha 乘子(Plan 15:代码块行窗边缘淡入淡出;默认 1)
    @location(9) exit_time: f32,       // 退场 dissolve 起点 ms(Plan 36 N3;0=无,恒等)
    @location(10) form_target: vec2<f32>, // Plan 42 编队目标世界坐标(默认 0;form_progress>0 时用)
    @location(11) form_pack: u32,      // Plan 42 编队打包(stagger 低 16 + seed 高 16;默认 0=恒等)
};

struct VsOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) alpha: f32,
    @location(2) tint: vec3<f32>,
    @location(3) @interpolate(flat) layer: u32,
    @location(4) @interpolate(flat) kind: u32,
    @location(5) world: vec2<f32>,                 // 世界坐标(N3 dissolve 的空间噪声域)
    @location(6) @interpolate(flat) exit_time: f32,
};

fn style_color(s: u32) -> vec3<f32> {
    // Plan 28 R1:opencode dark 文字色系(层级靠亮度:strong #EDEDED / base #A0A0A0 /
    // weak #707070 / weaker #505050;数值 = tokens.live-dark.json,依据 NOTES.md §2)。
    switch s {
        case 1u: { return vec3<f32>(1.0, 1.0, 1.0); }        // Bold(单字重字体无法加粗 → 以纯白近似 medium,ΔE≈4)
        case 2u: { return vec3<f32>(0.929, 0.929, 0.929); }  // Italic(同正文 strong,靠斜体区分)
        case 3u: { return vec3<f32>(1.0, 1.0, 1.0); }        // BoldItalic(同 Bold 近似)
        case 4u: { return vec3<f32>(0.0, 0.808, 0.726); }    // 行内 code(--syntax-string #00ceb9,markdown.css :not(pre)>code)
        case 5u: { return vec3<f32>(0.929, 0.929, 0.929); }  // CodeBlock 素文字(shiki 缺省 = strong)
        case 6u, 10u, 11u, 12u, 13u, 14u: { return vec3<f32>(0.929, 0.929, 0.929); } // Heading(白,层级靠字重/字号)
        case 7u: { return vec3<f32>(0.753, 0.831, 0.984); }  // Link(--text-interactive-base #c0d4fb)
        case 8u, 9u: { return vec3<f32>(0.439, 0.439, 0.439); } // Quote / ListMarker(text-weak #707070)
        case 16u: { return vec3<f32>(0.929, 0.929, 0.929); } // AlertLabel(类型色靠左条;文字 strong)
        case 17u: { return vec3<f32>(0.929, 0.929, 0.929); } // TableCell(表体继承 markdown strong)
        case 18u: { return vec3<f32>(0.929, 0.929, 0.929); } // TableHeader(strong)
        case 19u: { return vec3<f32>(1.0, 1.0, 1.0); }       // TableStrong(Bold 同款近似)
        case 20u: { return vec3<f32>(0.929, 0.929, 0.929); } // TableEm(同表体,靠斜体区分)
        case 21u: { return vec3<f32>(0.314, 0.314, 0.314); } // TableSep(text-weaker)
        case 24u: { return vec3<f32>(0.753, 0.831, 0.984); } // FootnoteRef(Link 色)
        case 25u: { return vec3<f32>(0.439, 0.439, 0.439); } // FootnoteDef(weak)
        case 43u: { return vec3<f32>(0.314, 0.314, 0.314); } // CodeLineNum(text-weaker)
        // 代码语法高亮:opencode --syntax-* 色板(live-dark)。
        case 44u: { return vec3<f32>(0.929, 0.698, 0.945); } // CodeKeyword(--syntax-keyword #edb2f1)
        case 45u: { return vec3<f32>(0.988, 0.835, 0.228); } // CodeType(--syntax-type #fcd53a)
        case 46u: { return vec3<f32>(0.549, 0.690, 1.000); } // CodeFunc(--syntax-primitive #8cb0ff)
        case 47u: { return vec3<f32>(0.000, 0.808, 0.726); } // CodeString(--syntax-string #00ceb9)
        case 48u: { return vec3<f32>(0.561, 0.561, 0.561); } // CodeComment(--syntax-comment #8f8f8f)
        case 49u: { return vec3<f32>(0.576, 0.914, 0.965); } // CodeNumber(--syntax-constant #93e9f6)
        case 50u: { return vec3<f32>(0.439, 0.439, 0.439); } // CodePunct(--syntax-punctuation #707070)
        // part 渲染角色(Plan 23 / 0033):tool 卡 / reasoning / diff。
        case 51u: { return vec3<f32>(0.439, 0.439, 0.439); } // Reasoning(text-weak,NOTES §3)
        case 52u: { return vec3<f32>(0.929, 0.929, 0.929); } // ToolTitle(strong,NOTES §4)
        case 53u: { return vec3<f32>(0.314, 0.314, 0.314); } // ToolArg(text-weaker)
        case 54u: { return vec3<f32>(0.628, 0.628, 0.628); } // ToolOutput(正文 base)
        case 55u: { return vec3<f32>(0.439, 0.439, 0.439); } // ToolBadge(中性;状态色靠 app rect)
        case 56u: { return vec3<f32>(0.769, 1.000, 0.753); } // DiffAdded(--text-diff-add-base #c4ffc0)
        case 57u: { return vec3<f32>(0.926, 0.184, 0.078); } // DiffRemoved(--text-diff-delete-base #ec2f14)
        case 58u: { return vec3<f32>(0.098, 0.098, 0.098); } // AskButton(primary 亮底上的深字,NOTES §6)
        case 59u: { return vec3<f32>(0.628, 0.628, 0.628); } // DiffCtx(diff 上下文行,--text-base)
        default: { return vec3<f32>(0.929, 0.929, 0.929); }  // Normal(markdown 正文 = --text-strong #EDEDED)/ Rule
    }
}

// ── SDF 节点动画(0025 / Plan 10 §3,相位 3:per-element profile by StyleRole)──
// 进场 = 缓动 alpha + 绕字心 scale-in(+translate 钩子),由 spawn_time + fade_ms 驱动;settled(e=1)零影响、
// 瞬显(fade<=0)跳过。**per-element**:按字的 StyleRole 查 profile —— 表头/标题(pop+回弹+稍慢)≠ 正文。
// 用现有 instance.style 区分 → 零 buffer 改动;若要按 reveal 上下文(整表/行框)再分 = 相位 3b(per-instance 字段)。
// Plan 25 P0(design §4.3):正文进场收敛为「Alpha + 上浮 6px + Scale 0.96→1」——克制的
// 微动效(≤ 一次注视),替代旧 0.1→1 的大缩放 pop(过强,近似"弹出")。
const ANIM_ENTER_SCALE: f32 = 0.96; // 正文进场起始缩放(<1 由小放大);1.0 关闭
const ANIM_ENTER_RISE: f32 = 6.0;   // 进场上浮 px(>0 由下而上);0 关闭

struct EnterProfile { scale_from: f32, rise: f32, dur_mul: f32, curve: u32 }

fn ease_out_cubic(t: f32) -> f32 { let u = 1.0 - t; return 1.0 - u * u * u; }
fn ease_out_back(t: f32) -> f32 { let c1 = 1.70158; let c3 = c1 + 1.0; let u = t - 1.0; return 1.0 + c3 * u * u * u + c1 * u * u; }
// Plan 25 P0 / 0025 CurveId 扩三条(id 与 core motion.rs CURVE_* 同值对齐,快照测试锁):
// 2 = ease.enter ≈ cubic-bezier(0,0,.38,.9)(Carbon entrance;拟合 1-(1-t)^1.9,maxErr≈.02)
// 3 = ease.exit ≈ cubic-bezier(.2,0,1,.9)(Carbon exit;拟合 .5t+.5t^2.1,maxErr≈.019)
// 4 = ease.expressive ≈ cubic-bezier(.4,.14,.3,1)(拟合 1-(1-t^1.7)^3.4,maxErr≈.035)
// t=1 时三条均恒 1(RD4 恒等收敛)。
fn ease_enter(t: f32) -> f32 { return 1.0 - pow(1.0 - t, 1.9); }
fn ease_exit(t: f32) -> f32 { return 0.5 * t + 0.5 * pow(t, 2.1); }
fn ease_expressive(t: f32) -> f32 { return 1.0 - pow(1.0 - pow(t, 1.7), 3.4); }
// 5 = Spring 闭式(Plan 36 N4):1 − e^{−kt}·cos(wt),k=6/w=12(保守单次过冲);
// t≥1 直接 1(收敛恒等 AR3;闭式值 ≈0.998,snap 补齐)。与 motion::spring 同源。
fn ease_spring(t: f32) -> f32 {
    if (t >= 1.0) { return 1.0; }
    return 1.0 - exp(-6.0 * t) * cos(12.0 * t);
}
fn apply_curve(curve: u32, t: f32) -> f32 {
    if (curve == 1u) { return ease_out_back(t); }
    if (curve == 2u) { return ease_enter(t); }
    if (curve == 3u) { return ease_exit(t); }
    if (curve == 4u) { return ease_expressive(t); }
    if (curve == 5u) { return ease_spring(t); }
    return ease_out_cubic(t);
}

// 进场 profile 表(按 core 给的 id 查;id 与 app::enter_profile_id 对齐,3b 数据驱动)。
// 0=正文逐字(ease.enter);1=表头/标题 pop(0.4→1 回弹,1.5×);2=整表风格的表头(0.2→1 回弹,2× 更大更慢)。
fn enter_profile_by_id(id: u32) -> EnterProfile {
    if (id == 5u) { return EnterProfile(ANIM_ENTER_SCALE, ANIM_ENTER_RISE, 1.0, 5u); } // N4 spring 进场(可选)
    if (id == 2u) { return EnterProfile(0.2, 0.0, 2.0, 1u); }
    if (id == 1u) { return EnterProfile(0.4, 0.0, 1.5, 1u); }
    return EnterProfile(ANIM_ENTER_SCALE, ANIM_ENTER_RISE, 1.0, 2u);
}

@vertex
fn vs_main(@builtin(vertex_index) vid: u32, inst: InstanceIn) -> VsOut {
    var corners = array<vec2<f32>, 4>(
        vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 0.0),
        vec2<f32>(0.0, 1.0), vec2<f32>(1.0, 1.0),
    );
    let c = corners[vid];
    // per-element 进场(按 core 给的 profile id 查表,3b);e = 缓动 (now-spawn)/(fade*dur_mul);fade<=0 → e=1 跳过。
    let prof = enter_profile_by_id(inst.anim);
    let age = globals.time_ms - inst.spawn_time;
    let dur = max(globals.fade_ms * prof.dur_mul, 1.0);
    let e = select(apply_curve(prof.curve, clamp(age / dur, 0.0, 1.0)), 1.0, globals.fade_ms <= 0.0);
    // 绕字心 scale-in(+上浮);settled(e=1)还原 inst.pos + c*size。回弹曲线 e 可略 >1 → scale 过冲再落定(pop)。
    let half = vec2<f32>(0.5, 0.5);
    let s = mix(prof.scale_from, 1.0, e);
    let corner = inst.size * (half + (c - half) * s); // 角相对左上角的偏移
    // Plan 42 字阵编队:base(左上角)从布局位飞到 form_target(保字形大小);form_progress=0 恒等直通。
    var base = inst.pos;
    if (globals.form_progress > 0.0) {
        // stagger 错峰起飞(低 16 位定点 0..1);k=stagger 占比,progress=1 时全部到位。
        let stagger = f32(inst.form_pack & 0xffffu) / 65535.0;
        let seedf = f32(inst.form_pack >> 16u);
        let k = 0.45;
        let tf = clamp((globals.form_progress - stagger * k) / (1.0 - k), 0.0, 1.0);
        let fe = ease_expressive(tf);
        // 贝塞尔弧飞行:控制点在飞行线中点法向偏移(seed 定弧向/幅)→ 弧线不呆板;fe=1 落定 form_target。
        let p0 = inst.pos;
        let p2 = inst.form_target;
        let d = p2 - p0;
        let dlen = max(length(d), 1.0);
        let perp = vec2<f32>(-d.y, d.x) / dlen;
        let arc_sign = select(-1.0, 1.0, (inst.form_pack & 0x10000u) != 0u);
        let arc_mag = dlen * (0.12 + 0.18 * fract(seedf * 0.017));
        let ctrl = (p0 + p2) * 0.5 + perp * arc_sign * arc_mag;
        let u = 1.0 - fe;
        let bez = u * u * p0 + 2.0 * u * fe * ctrl + fe * fe * p2;
        // 飞行中噪声扰动(fenv 两端归 0 → 落定恒等 RD4;纯 f(seed,fe) 无时基 → 定帧确定)。
        let fenv = fe * (1.0 - fe) * 4.0;
        let wob = vec2<f32>(sin(seedf * 1.7 + fe * 6.0), cos(seedf * 2.3 + fe * 5.0)) * (fenv * dlen * 0.03);
        base = bez + wob;
    }
    let world = base + corner + vec2<f32>(0.0, (1.0 - e) * prof.rise);
    // 世界坐标 → 相机 → 屏幕 px → NDC(Plan 3 L)。
    let screen = (world - globals.cam_pan) * globals.cam_zoom;
    let ndc = vec2<f32>(
        screen.x / globals.viewport.x * 2.0 - 1.0,
        1.0 - screen.y / globals.viewport.y * 2.0,
    );
    var out: VsOut;
    out.clip = vec4<f32>(ndc, 0.0, 1.0);
    out.uv = vec2<f32>(mix(inst.uv.x, inst.uv.z, c.x), mix(inst.uv.y, inst.uv.w, c.y));
    out.alpha = clamp(e, 0.0, 1.0) * inst.alpha; // 缓动淡入 × 静态 alpha(Plan 15 行窗边缘淡);emoji(kind3)同走
    out.world = world;
    out.exit_time = inst.exit_time;
    // M2e 到达高亮(design §3.3 karaoke 读头):enter 期短暂提亮,随 e→1 衰减归位(AR3)。
    // Plan 28 R4:标题 shimmer(style 高位 1<<31;参考 TextShimmer:base --text-weak →
    // peak --text-strong,~1.2s 周期沿 x 扫过)。GPU 相位 = f(time, world.x) → 冻结块零重传。
    let role = inst.style & 0x7fffffffu;
    var tint = style_color(role);
    if ((inst.style & 0x80000000u) != 0u) {
        let phase = fract(globals.time_ms / 1200.0);
        let u = fract(world.x * 0.004 - phase); // 波长 ~250 world px,向右扫
        let sweep = smoothstep(0.72, 0.92, u) * (1.0 - smoothstep(0.92, 1.0, u));
        tint = mix(vec3<f32>(0.439, 0.439, 0.439), vec3<f32>(0.929, 0.929, 0.929), sweep);
    }
    out.tint = tint * (1.0 + globals.arrive_boost * (1.0 - clamp(e, 0.0, 1.0)));
    out.layer = inst.layer;
    out.kind = inst.kind;
    return out;
}

// 单通道距离场覆盖率(TinySDF):smoothstep + fwidth 屏幕空间梯度 → 任意缩放锐利。
// ③ 阈值下移到 0.46:浅字深底会"视觉变细",下移加粗找回字重。
// ② AA 带收窄到 0.6×fwidth:整 fwidth 当半宽约 2px 过渡偏软,收窄更锐。
fn sdf_coverage(d: f32) -> f32 {
    let aa = max(fwidth(d), 0.0001);
    return smoothstep(0.46 - 0.6 * aa, 0.46 + 0.6 * aa, d);
}

fn median3(c: vec3<f32>) -> f32 {
    return max(min(c.r, c.g), min(max(c.r, c.g), c.b));
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    // 两源都在统一控制流里采(textureSample/fwidth 需 uniform control flow),再按 kind 选;
    // 非 MSDF 实例的 msdf 采样作废(dummy 1×1 或越界层被 clamp,安全)。
    let r8 = textureSample(atlas_tex, atlas_samp, in.uv, i32(in.layer));
    let m = textureSample(msdf_tex, atlas_samp, in.uv, i32(in.layer)).rgb;
    let cov_sdf = sdf_coverage(r8.r);
    let cov_msdf = sdf_coverage(median3(m));
    var cov: f32;
    switch in.kind {
        case 0u: { cov = r8.r; }              // 位图覆盖率:.r 直采
        case 2u: { cov = cov_msdf; }          // MSDF:median 距离场
        case 3u: { return vec4<f32>(r8.rgb, r8.a * in.alpha); } // RGBA 彩字(emoji):直采真彩,fade 走 alpha(0015 §7.2)
        default: { cov = cov_sdf; }           // TinySDF
    }
    var rgb = in.tint;
    var a = cov * in.alpha;
    // N3 退场 dissolve(Febucci 配方,catalog §4):噪声阈值裁剪 + 阈值窄带发光边。
    // exit_time==0(默认)恒等直通;时长复用 fade_ms(0 时兜 400ms)。空间噪声域 = 世界
    // 坐标 / 6(字级颗粒),纯函数 of 位置 → 双跑一致(R8)。
    if in.exit_time > 0.0 {
        let prog = clamp(in.exit_time, 0.0, 1.0); // core 已折算进度(0=off 不进此支)
        let n = nz_value(in.world / 6.0, 54u);
        let keep = smoothstep(prog - 0.04, prog + 0.04, n + 0.001);
        let band = smoothstep(prog - 0.14, prog, n) - smoothstep(prog, prog + 0.02, n);
        rgb = mix(rgb, vec3<f32>(0.95, 0.62, 0.2), clamp(band, 0.0, 1.0) * step(0.001, prog));
        a = a * keep;
    }
    return vec4<f32>(rgb, a);
}
