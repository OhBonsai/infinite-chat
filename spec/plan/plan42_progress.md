# Plan 42 · 进度账本(字阵 Glyph Bloom)

> 逐 milestone 记录:字段/stride 记账 / 匹配基准数值 / 定帧 golden / 遗留。
> GOAL:[plan42-glyph-formation-goal](./plan42-glyph-formation-goal.md)

## G0 · 设计钉板(字段布局 / shape schema / 匹配选型 / 恒等硬门)

### instance 字段(Sample / GpuInstance 各 +2;stride 记账)
- `form_target: [f32; 2]`(location 10,Float32x2)—— 编队目标世界坐标;默认 [0,0]。
- `form_pack: u32`(location 11,Uint32)—— 打包:**stagger 低 16 位**(定点 0..1 = 值/65535,起飞相位)
  + **seed 高 16 位**(噪声/自转)。默认 0。
- GpuInstance stride:60 → **72 字节**(+12);顶点属性 10→12 个;instance step。带宽 +20%(仅文字管线;
  perf 门把关,默认态零行为)。构造点全默认 0:wasm/lib.rs ×2(282/343)+ morph.rs GpuInstance(219)+
  morph 测 ×2。Sample 全默认 0 → 恒等。

### Globals uniform(+form_progress + wind;64 字节)
| 字段 | 偏移 | 说明 |
|---|---|---|
| viewport/time/fade/cam_pan/cam_zoom/arrive_boost | 0–31 | **不动**(其它 shader 读前缀,零影响) |
| form_progress: f32 | 32 | 编队进度 0=恒等 / 1=成形(注入时钟推进,可反向) |
| form_pad: [f32;3] | 36–47 | 16 字节对齐填充(WGSL 用 **3×f32**,勿用 vec3——其 align 16 会把 wind 顶到 64 错位) |
| wind: vec4 | 48–63 | 指针风场 [pos.xy(世界), radius, strength];默认 0=无风 |
- 共享 `globals_buf`(glyph/rect/panel/widget/image/shaderbox 六 shader)只**追加**尾字段 → 前缀偏移不变,
  其余 shader 各读自己较短的 Globals 前缀,零影响(buffer 32→64,as_entire_binding,64≥各 struct)。
- backend:`WebGpuBackend { form_progress, wind }` 字段 + `set_form_progress/set_wind` setter(仿 arrive_boost);
  draw() 构 Globals 时喂。

### API 面(core,G1 实现)
`formation_begin(shape_json) → 采样目标 + 匹配写 form_target/form_pack` · `formation_set_progress(p)`(或注入时钟
推进)· `formation_reverse()`(1→0 散场)· `formation_end()`(清 target/progress 归零)· `set_wind(x,y,r,s)` ·
`formation_pulse()`(点击绽放 impulse,G3)。progress/落轨走注入时钟(R8)。

### shape JSON schema(primitives,serde,坏数据整拒 AR12)
`{ "shape": "rose"|"heart", "params": { rose: {a:f32, k:u32(花瓣数)} | heart: {scale:f32} }, "seed": u32 }`。
形状=数据(§0-3):换图案 = 换 JSON,不改编排。

### 匹配算法选型(G2 实现 + native 基准)
**角度桶贪心**:按可见 glyph 相对花心的方位角分桶(桶数 = 花瓣数×M),桶内按半径排序,就近配采样点 →
避免轨迹大交叉;O(n log n),1e4 字毫秒级。基准:总飞行距离 ≤ 随机匹配 70%(native 断言,数值入账 G2)。

### 恒等硬门(G0 先立)
VS **零行为改动**(form_target/form_pack/form_progress/wind 全声明未用;progress 默认 0)→ 默认态 golden
全集应逐字节。native(render/core/wasm)+ clippy(-D warnings,form_pad 去下划线避 unused-public)+ fmt 通过;
门验证 golden 逐字节(见下)。

## G1 · 编队通道 v0(直线飞行 + 方格目标)

(未开始)

## G2 · 花形 + 匹配 + 弧线扰动 + 备选形状

(未开始)

## G3 · 花瓣层 + 风场 + 绽放脉冲

(未开始)

## G4 · 首页高潮幕 + 真 zoom + celebrate 槽 + 收口

(未开始)

## DoD 对账

(未开始)
