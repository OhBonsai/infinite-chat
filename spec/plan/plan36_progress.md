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

- **参数通道**:`FrameRect`/`RectInstance` += `fx:[mode,p1,p2,_]` + `fx_color`(attr 6/7);
  **mode 0 = 恒等直通**(默认观感零变化,fragment 原路径原样);发射端负责预膨胀 quad,
  fx.z 携外扩量,shader 对内缩盒求 d。
- **三件**(rect.wgsl,自写):①glow `exp(-k·d)`(catalog §2;k=fx.y);②解析 drop
  shadow —— Evan Wallace 闭式思路的 **SDF×erf 单 pass 变体**(erf 用 Abramowitz–Stegun
  7.1.26 近似,自写;直边精确、圆角近似,偏差肉眼不可辨,记选型);③等距条纹
  `fract(-d/L) < duty`(L=fx.y,duty=fx.z)。
- **演示/验收面**:`?gallery` 下发射三演示条(默认视图零发射);e2e golden
  `fx-distance-band.png`(maxDiffPixelRatio 0)+ 验收帧
  `test/results/plan36/n2-distance-band-dpr{1,2}.png`。
- **native**:`fx_demo_rects_only_under_gallery_and_default_is_identity` ——
  默认路径全 rect fx.mode==0(参数恒等数据面)+ gallery 下 mode 1/2/3 各恰一。
- **遗留**:panel.wgsl 侧 fx 参数(0018 storage 通道)未扩 —— rect 图元已覆盖 0037 装饰
  租户;panel 容器需要 glow/shadow 时再扩(防过度参数化);DecorSlot 新槽位随 N3
  hit-flash 一并接。

## N3 · dissolve 退场 + hit-flash

- **dissolve(0016 exit 首租,noise 消费者第一号)**:`FrameGlyph/GpuInstance/Sample` +=
  `exit_time`(0=off 恒等);glyph.wgsl 前置拼接 noise.wgsl,片元按
  `(time−exit_time)/fade_ms` 推进 —— `nz_value(world/6, seed)` 阈值裁剪 +
  **Febucci 窄带发光边**(catalog §4;阈上 0.14 带混暖橙)。空间噪声域 = 世界坐标
  (纯函数 of 位置,双跑一致 R8;morph 身份迁移时 exit_time 随 Sample 保持)。
- **exit 通道**(clear_orphan_views):`Engine.exit_dissolve_ms`(默认 0 = 孤儿即时清除
  == 旧行为恒等);>0 → 孤儿先置 `PartView.exiting`(cache/渲染态原样,几何不动 AR3),
  glyph 携 exit_time 交 shader,到期真清除。普通淡出 = dissolve 的退化(强度即窄带宽,
  按 GOAL「dissolve 是 exit 淡出的超集」记账)。wasm `set_exit_dissolve_ms`。
- **hit-flash**:rect.wgsl fx mode 4 —— cubicPulse(IQ)时间包络混色(fx.y=起点 ms,
  fx.z=时长);起点前/过峰后包络 0 = 恒等(AR3)。`motion::cubic_pulse` 纯函数入
  primitives(N4 试衣间/plan38 预设的 CPU 侧同源)。
- **native**:`exit_dissolve_defers_orphan_wipe_then_clears`(off 即时清/on 溶解中全 glyph
  携 exit_time/到期清除/双跑一致)+ `cubic_pulse_endpoints_identity_peak_center`
  (端点恒等/峰值/对称/确定性)。
- **遗留**:hit-flash 的真实触发器(DecorSlot 语义槽消费)留 plan38 预设库场景表接线;
  dissolve 中间帧 golden 留 N4 试衣间(可控 progress 才可定帧)。

## N4 · Spring 曲线 + 面板试衣间

(未开始)

## DoD 对账

(未开始)
