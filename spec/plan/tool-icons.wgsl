// opencode 内置 tool 的 animated icon —— WGSL(plan16 §2.5 契约)
// - 工具箱(下半)实际应并入 shaders/base/sdf.wgsl(0026 拼接);此处自包含便于校验。
// - icon_id 50..65 = tool 图标,接在 deck 50 支(0..49)之后(plan16 §2.5c)。
// - 契约:uv = fragCoord/resolution(单 ShaderBox = 一个 icon 满格);t = time;
//   输出 vec4,1-bit 灰度,alpha=亮度 → over 背景(0028 §2.1)。
// - 16 个全 dynamic=true(均用 time)。

const PI: f32 = 3.14159;
const TAU: f32 = 6.283185;

// ============ icon 入口 ============
// id: 50..65;st: 盒内 [0,1]^2;t: time。返回 1-bit 亮度。
fn icon(st_in: vec2<f32>, id: i32, t: f32) -> f32 {
    let st = st_in;
    let S = 1.0 + sin(t) / 4.0;   // 呼吸
    let C = 1.0 + cos(t) / 4.0;
    var color = 0.0;

    switch (id) {

    case 50: { // read —— 纸张 + 扫描线下移
        let page = rectSDF(st, vec2<f32>(0.58, 0.78));
        let inside = fillf(page, 0.5);
        color = stroke(page, 0.5, 0.045);
        let scanY = 0.80 - 0.60 * fract(t * 0.6);
        color += stroke(st.y, scanY, 0.022) * inside;
    }
    case 51: { // write —— 纸张 + 笔尖横移留迹 + caret 闪
        let page = rectSDF(st, vec2<f32>(0.58, 0.78));
        let inside = fillf(page, 0.5);
        color = stroke(page, 0.5, 0.045);
        let tt = fract(t * 0.5);
        let penX = 0.30 + 0.40 * tt;
        let baseY = 0.40;
        color += stroke(st.y, baseY, 0.020) * step(st.x, penX) * step(0.28, st.x) * inside;
        let caret = fillf(rectSDF(offc(st, penX, baseY + 0.06), vec2<f32>(0.020, 0.14)), 0.5);
        color += caret * step(0.5, fract(t * 2.0)) * inside;
    }
    case 52: { // edit —— 文本行 + 高亮框滑动反相
        var lines = 0.0;
        lines += stroke(st.y, 0.64, 0.020) * step(0.24, st.x) * step(st.x, 0.76);
        lines += stroke(st.y, 0.50, 0.020) * step(0.24, st.x) * step(st.x, 0.68);
        lines += stroke(st.y, 0.36, 0.020) * step(0.24, st.x) * step(st.x, 0.72);
        let hcx = 0.34 + 0.30 * fract(t * 0.35);
        let hbox = rectSDF(offc(st, hcx, 0.50), vec2<f32>(0.17, 0.10));
        let ins = fillf(hbox, 0.5);
        color = flipf(lines, ins);
        color += stroke(hbox, 0.5, 0.03);
    }
    case 53: { // grep —— 放大镜 + 镜内文本上滚
        let lens = circleSDF(st);
        let ins = fillf(lens, 0.44);
        color = stroke(lens, 0.46, 0.05);
        let h = rotate(st, radians(-45.0));
        color += stroke(h.x, 0.5, 0.035) * step(0.50, h.y) * step(h.y, 0.80);
        for (var i: i32 = 0; i < 3; i = i + 1) {
            let yy = fract(0.20 + f32(i) * 0.30 + t * 0.25);
            let lineY = 0.30 + 0.40 * yy;
            color += stroke(st.y, lineY, 0.022) * ins * step(0.32, st.x) * step(st.x, 0.68);
        }
    }
    case 54: { // glob —— 星号 * 旋转 + 角落命中方块闪
        let c = rotate(st, t * 0.3);
        let rays = raysSDF(c, 6);
        let disc = circleSDF(st);
        color = stroke(rays, 0.5, 0.085) * fillf(disc, 0.66);
        color += fillf(disc, 0.12);
        for (var i: i32 = 0; i < 4; i = i + 1) {
            let a = f32(i) * TAU / 4.0 + PI / 4.0;
            let p = vec2<f32>(cos(a), sin(a)) * 0.40;
            let blink = step(0.5, fract(t * 0.6 + f32(i) * 0.25));
            color += fillf(rectSDF(st - p, vec2<f32>(0.06, 0.06)), 0.5) * blink * 0.9;
        }
    }
    case 55: { // shell —— ">" + 下划线 caret 闪
        let fU = (st.x + st.y) * 0.5;
        let fL = 0.5 + (st.y - st.x) * 0.5;
        let bx = step(0.28, st.x) * step(st.x, 0.66);
        let byU = step(0.50, st.y) * step(st.y, 0.74);
        let byL = step(0.26, st.y) * step(st.y, 0.50);
        color = stroke(fU, 0.58, 0.055) * bx * byU;
        color += stroke(fL, 0.42, 0.055) * bx * byL;
        color += stroke(st.y, 0.30, 0.030) * step(0.70, st.x) * step(st.x, 0.88)
                 * step(0.5, fract(t * 1.5));
    }
    case 56: { // task —— 枢纽 + 轨道 + 3 卫星环绕
        color = stroke(circleSDF(st), 0.66, 0.012);
        color += fillf(circleSDF(st), 0.17);
        for (var i: i32 = 0; i < 3; i = i + 1) {
            let a = t * 0.8 + f32(i) * TAU / 3.0;
            let p = vec2<f32>(cos(a), sin(a)) * 0.33;
            color += fillf(circleSDF(st - p), 0.11);
        }
    }
    case 57: { // webfetch —— 地球仪 + 点向心飞入
        let globe = circleSDF(st);
        let inside = fillf(globe, 0.62);
        color = stroke(globe, 0.62, 0.04);
        color += stroke(circleSDF(scale2(st, vec2<f32>(2.4, 1.0))), 0.62, 0.03) * inside;
        color += stroke(st.y, 0.5, 0.03) * inside;
        let tt = fract(t * 0.6);
        let dx = 0.5 + 0.45 * (1.0 - tt);
        color += fillf(circleSDF(offc(st, dx, 0.5)), 0.07) * step(0.52, dx);
    }
    case 58: { // websearch —— 声呐扩散环
        let r = circleSDF(st);
        for (var i: i32 = 0; i < 3; i = i + 1) {
            let phase = fract(t * 0.5 + f32(i) / 3.0);
            color += stroke(r, phase * 1.15, 0.035 * (1.0 - phase));
        }
        color += fillf(r, 0.11);
    }
    case 59: { // todowrite —— 复选框 + 顶行对勾描出
        for (var i: i32 = 0; i < 3; i = i + 1) {
            let fy = 0.66 - f32(i) * 0.18;
            let box = rectSDF(offc(st, 0.30, fy), vec2<f32>(0.11, 0.11));
            color += stroke(box, 0.5, 0.035);
            color += stroke(st.y, fy, 0.020) * step(0.46, st.x) * step(st.x, 0.80);
        }
        let tc = clamp(fract(t * 0.4) * 1.6, 0.0, 1.0);
        let a = vec2<f32>(0.255, 0.655);
        let b = vec2<f32>(0.295, 0.620);
        let d = vec2<f32>(0.350, 0.700);
        color += line2(st, a, mix(a, b, clamp(tc * 2.0, 0.0, 1.0)), 0.016);
        color += line2(st, b, mix(b, d, clamp(tc * 2.0 - 1.0, 0.0, 1.0)), 0.016) * step(0.5, tc);
    }
    case 60: { // skill —— 钥匙微转
        let k = rotate(st, sin(t * 0.8) * 0.12);
        let bow = circleSDF(offc(k, 0.30, 0.5));
        color = stroke(bow, 0.30, 0.05);
        color += stroke(k.y, 0.5, 0.045) * step(0.38, k.x) * step(k.x, 0.80);
        color += stroke(k.x, 0.66, 0.030) * step(0.40, k.y) * step(k.y, 0.50);
        color += stroke(k.x, 0.74, 0.030) * step(0.40, k.y) * step(k.y, 0.50);
    }
    case 61: { // apply_patch —— diff ± 交替加重
        let frame = rectSDF(st, vec2<f32>(0.66, 0.66));
        color = stroke(frame, 0.5, 0.030);
        let eU = step(0.5, fract(t * 0.6));
        let plus = crossSDF(offc(st, 0.5, 0.63), 1.0);
        color += fillf(plus, 0.15) * (0.4 + 0.6 * eU);
        color += stroke(st.y, 0.37, 0.045) * step(0.34, st.x) * step(st.x, 0.66)
                 * (0.4 + 0.6 * (1.0 - eU));
    }
    case 62: { // question —— "?" 呼吸
        let q = offc(st, 0.5, 0.60);
        let ring = circleSDF(q);
        let ang = atan2(q.y - 0.5, q.x - 0.5);
        let arcMask = step(-2.3, ang);
        color = stroke(ring, 0.30 * S, 0.050) * arcMask;
        color += stroke(st.x, 0.5, 0.050) * step(0.34, st.y) * step(st.y, 0.48);
        color += fillf(circleSDF(offc(st, 0.5, 0.26)), 0.055);
    }
    case 63: { // plan-enter —— 折线路由 draw-on
        let n0 = vec2<f32>(0.25, 0.36);
        let n1 = vec2<f32>(0.50, 0.64);
        let n2 = vec2<f32>(0.75, 0.40);
        let tt = fract(t * 0.4);
        let ta = clamp(tt * 2.0, 0.0, 1.0);
        let tb = clamp(tt * 2.0 - 1.0, 0.0, 1.0);
        color += line2(st, n0, mix(n0, n1, ta), 0.020);
        color += line2(st, n1, mix(n1, n2, tb), 0.020) * step(0.5, tt);
        color += fillf(circleSDF(offc(st, n0.x, n0.y)), 0.07);
        color += fillf(circleSDF(offc(st, n1.x, n1.y)), 0.07) * step(0.5, tt);
        color += fillf(circleSDF(offc(st, n2.x, n2.y)), 0.07) * step(0.96, tt);
    }
    case 64: { // plan-exit —— 路由已成 + 播放三角呼吸
        let n0 = vec2<f32>(0.25, 0.36);
        let n1 = vec2<f32>(0.50, 0.64);
        let n2 = vec2<f32>(0.75, 0.40);
        color += line2(st, n0, n1, 0.018);
        color += line2(st, n1, n2, 0.018);
        color += fillf(circleSDF(offc(st, n0.x, n0.y)), 0.06);
        color += fillf(circleSDF(offc(st, n2.x, n2.y)), 0.06);
        let pc = rotate(offc(st, n1.x, n1.y), radians(-90.0));
        color += fillf(triSDF(pc), 0.22 * S);
    }
    case 65: { // invalid —— 禁止符抖动
        let ring = circleSDF(st);
        color = stroke(ring, 0.60, 0.05);
        let dd = 0.5 + (st.x - st.y) * 0.5;
        color += stroke(dd, 0.5, 0.05) * fillf(ring, 0.60);
        let flick = 0.55 + 0.45 * step(0.5, fract(t * 6.0 + sin(t * 13.0)));
        color *= flick;
    }
    default: { color = 0.0; }
    }

    return clamp(color, 0.0, 1.0);
}

// shaderbox effect 入口(0028 §2.1):返回 over-背景 的 rgba。
fn shade(uv: vec2<f32>, icon_id: u32, t: f32) -> vec4<f32> {
    let c = icon(uv, i32(icon_id), t);
    return vec4<f32>(c, c, c, c);
}

// ============ SDF 工具箱(→ base/sdf.wgsl) ============
fn sstep(a: f32, b: f32) -> f32 { return smoothstep(a - 0.005, a + 0.005, b); }

fn stroke(x: f32, s: f32, w: f32) -> f32 {
    let d = sstep(s, x + w * 0.5) - sstep(s, x - w * 0.5);
    return clamp(d, 0.0, 1.0);
}
fn circleSDF(st: vec2<f32>) -> f32 { return length(st - 0.5) * 2.0; }
fn fillf(x: f32, size: f32) -> f32 { return 1.0 - sstep(size, x); }
fn rectSDF(st_in: vec2<f32>, s: vec2<f32>) -> f32 {
    let st = st_in * 2.0 - 1.0;
    return max(abs(st.x / s.x), abs(st.y / s.y));
}
fn crossSDF(st: vec2<f32>, s: f32) -> f32 {
    return min(rectSDF(st, vec2<f32>(0.25, s)), rectSDF(st, vec2<f32>(s, 0.25)));
}
fn flipf(v: f32, pct: f32) -> f32 { return mix(v, 1.0 - v, pct); }
fn triSDF(st_in: vec2<f32>) -> f32 {
    let st = (2.0 * st_in - 1.0) * 2.0;
    return max(abs(st.x) * 0.866025 + st.y * 0.5, -st.y * 0.5);
}
fn rotate(st: vec2<f32>, a: f32) -> vec2<f32> {
    let m = mat2x2<f32>(cos(a), -sin(a), sin(a), cos(a));
    return m * (st - 0.5) + 0.5;
}
fn raysSDF(st_in: vec2<f32>, n: i32) -> f32 {
    let st = st_in - 0.5;
    return fract(atan2(st.y, st.x) / TAU * f32(n));
}
fn scale2(st: vec2<f32>, s: vec2<f32>) -> vec2<f32> { return (st - 0.5) * s + 0.5; }
fn seg(p: vec2<f32>, a: vec2<f32>, b: vec2<f32>) -> f32 {
    let pa = p - a;
    let ba = b - a;
    let h = clamp(dot(pa, ba) / dot(ba, ba), 0.0, 1.0);
    return length(pa - ba * h);
}
fn offc(st: vec2<f32>, cx: f32, cy: f32) -> vec2<f32> { return st - vec2<f32>(cx - 0.5, cy - 0.5); }
fn line2(p: vec2<f32>, a: vec2<f32>, b: vec2<f32>, w: f32) -> f32 { return 1.0 - sstep(w, seg(p, a, b)); }
// 注:radians() / atan2() / smoothstep() / step() / fract() 均为 WGSL 内置,不再自定义。
