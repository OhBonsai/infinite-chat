// base/noise.wgsl — 噪声函数库(Plan 36 N1;0026 共享库:无 entry point、无 binding,
// backend.rs 前置拼接到消费 pipeline 源前)。
//
// **hash 选型 = PCG 整数 hash**(jcgt.org/published/0009/03/02,catalog §1):
// 整数运算跨 WebGPU/WebGL2(naga→GLSL)**位稳定** —— 浮点 hash(Hoskins sin-free)在
// GLSL 精度档下可能漂移,直接选整数路预答 DoD-2 的精度条款(记 plan36_progress N1)。
// 确定性(R8):全部函数 = 纯函数 of (seed, 坐标);时间由调用方经注入时钟折进坐标。

/// PCG 单轮 32 位混淆(jcgt PCG hash;输入输出全 u32,位稳定)。
fn pcg(v: u32) -> u32 {
    let state = v * 747796405u + 2891336453u;
    let word = ((state >> ((state >> 28u) + 4u)) ^ state) * 277803737u;
    return (word >> 22u) ^ word;
}

/// 2D 格点 + seed → [0,1) 均匀标量(经 pcg 两轮折叠;u32 → f32 除以 2^32)。
fn nz_hash21(p: vec2<i32>, seed: u32) -> f32 {
    let h = pcg(pcg(bitcast<u32>(p.x) + seed) + bitcast<u32>(p.y));
    return f32(h) * 2.3283064e-10; // 1/2^32
}

/// 2D 格点 → 单位梯度向量(gradient noise 用;角度自 hash)。
fn nz_grad2(p: vec2<i32>, seed: u32) -> vec2<f32> {
    let a = nz_hash21(p, seed) * 6.2831853;
    return vec2<f32>(cos(a), sin(a));
}

/// value noise(双线性 + smoothstep 权重):返回 [0,1)。
fn nz_value(uv: vec2<f32>, seed: u32) -> f32 {
    let i = vec2<i32>(floor(uv));
    let f = fract(uv);
    let w = f * f * (3.0 - 2.0 * f);
    let a = nz_hash21(i, seed);
    let b = nz_hash21(i + vec2<i32>(1, 0), seed);
    let c = nz_hash21(i + vec2<i32>(0, 1), seed);
    let d = nz_hash21(i + vec2<i32>(1, 1), seed);
    return mix(mix(a, b, w.x), mix(c, d, w.x), w.y);
}

/// gradient(Perlin 型)noise:返回约 [-1,1] → 归一到 [0,1)。
fn nz_gradient(uv: vec2<f32>, seed: u32) -> f32 {
    let i = vec2<i32>(floor(uv));
    let f = fract(uv);
    let w = f * f * (3.0 - 2.0 * f);
    let ga = dot(nz_grad2(i, seed), f);
    let gb = dot(nz_grad2(i + vec2<i32>(1, 0), seed), f - vec2<f32>(1.0, 0.0));
    let gc = dot(nz_grad2(i + vec2<i32>(0, 1), seed), f - vec2<f32>(0.0, 1.0));
    let gd = dot(nz_grad2(i + vec2<i32>(1, 1), seed), f - vec2<f32>(1.0, 1.0));
    let n = mix(mix(ga, gb, w.x), mix(gc, gd, w.x), w.y);
    return n * 0.7071 + 0.5; // ≈[-0.707,0.707] → [0,1)
}

/// fbm(分形布朗运动):octaves ∈ [1,4](参数化,DoD-1);lacunarity 2 / gain 0.5。
fn nz_fbm(uv: vec2<f32>, seed: u32, octaves: u32) -> f32 {
    var sum = 0.0;
    var amp = 0.5;
    var p = uv;
    var total = 0.0;
    for (var o = 0u; o < 4u; o = o + 1u) {
        if o >= octaves {
            break;
        }
        sum += amp * nz_gradient(p, seed + o);
        total += amp;
        p = p * 2.0;
        amp *= 0.5;
    }
    return sum / max(total, 1.0e-6);
}

/// voronoi F1(最近特征点距离,[0,~1]):九宫格扫描;特征点位置自 hash。
fn nz_voronoi_f1(uv: vec2<f32>, seed: u32) -> f32 {
    let i = vec2<i32>(floor(uv));
    let f = fract(uv);
    var best = 8.0;
    for (var y = -1; y <= 1; y = y + 1) {
        for (var x = -1; x <= 1; x = x + 1) {
            let cell = vec2<i32>(x, y);
            let jx = nz_hash21(i + cell, seed);
            let jy = nz_hash21(i + cell, seed ^ 0x9e3779b9u);
            let d = vec2<f32>(cell) + vec2<f32>(jx, jy) - f;
            best = min(best, dot(d, d));
        }
    }
    return sqrt(best);
}
