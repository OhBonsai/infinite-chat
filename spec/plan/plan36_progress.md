# Plan 36 · 进度账本(噪声库 + 距离带效果)

> 逐 milestone 记录:做了什么 / 参考依据(file:行号 / catalog 链接)/ 数值来源 / 遗留。
> GOAL:[plan36-noise-and-distance-effects-goal](./plan36-noise-and-distance-effects-goal.md)

## N1 · noise.wgsl 库 + 可视化

- **hash 选型 = PCG 整数 hash**(jcgt 0009/03/02,catalog §1):整数运算跨 WebGPU/WebGL2
  位稳定,直接预答 DoD-2 精度条款(浮点 Hoskins 在 GLSL 精度档可漂移 → 不选,依据记此)。
- **`base/noise.wgsl`**(0026 共享库):`pcg` / `nz_hash21(seed)` / `nz_grad2` / `nz_value` /
  `nz_gradient` / `nz_fbm(octaves≤4 参数化)` / `nz_voronoi_f1`;全签名带 seed;时间由调用方
  经注入时钟折进坐标(R8)。
- **可视化**:`ShaderId::NoiseDebug = 4` + `shaderbox/noise_debug.wgsl`(四象限
  value/gradient/fbm4/voronoi;p0.x=seed;static → time 恒 0);`?gallery` 第 69 格;
  backend 第五条 shaderbox pipeline(noise.wgsl 前置拼接);gallery 计数测试 68→69 同步。
- **测试**:naga 校验;e2e 双次独立加载 noise 格像素**逐字节一致**(DoD-2)+
  `noise-quadrants.png` golden(maxDiffPixelRatio 0,即永久双跑锁)。
- **遗留**:真 WebGL2 路实拍对拍(headless 单 WebGPU;整数 hash 按构造同值)记 N4。

## N2 · 距离带三件(glow / 解析 shadow / 条纹带)

(未开始)

## N3 · dissolve 退场 + hit-flash

(未开始)

## N4 · Spring 曲线 + 面板试衣间

(未开始)

## DoD 对账

(未开始)
