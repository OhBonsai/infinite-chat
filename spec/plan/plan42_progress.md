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

- **core formation 模块**(`crates/core/src/formation.rs`,纯函数 native 可测):`Shape`(grid/rose/heart,
  serde tag=shape,坏数据整拒 AR12)+ `FormationSpec{shape,seed}` + `parse_spec` + `compute_targets
  (centers, spec) -> Vec<Target{center,stagger}>`(与输入同序;花心=包围盒中心)。G1 grid 铺格 + 顺序
  stagger;rose/heart 先恒等占位(G2 填)。native 测 3:坏 JSON 拒 / grid 确定居中 / 空入空出。
- **wasm 编队态**:`GpuSink.formation: Option<FormationState{spec,progress}>`;`formation_begin(json)`
  (解析+置态,坏 JSON→false)· `formation_set_progress(p)` · `formation_end()`。**submit 内**:激活时取
  可见集 centers → `compute_targets` → 写 `sample.form_target`(目标 **top-left** = center−size/2,保字形
  大小)+ `form_pack`(stagger 低16 定点 + 逐字 seed 高16)+ `backend.set_form_progress`;否则喂 0。
  ChatCanvas 暴露 `formation_begin/formation_progress/formation_end`(全页 __chat/__hero 可调)。
- **glyph.wgsl VS 编队段**:重构为 `base(左上角) + corner(角偏移) + rise`;`form_progress>0` 时
  `base = mix(inst.pos, inst.form_target, ease_expressive(tf))`,`tf = clamp((progress − stagger·k)/(1−k))`
  (k=0.45 错峰起飞);progress=0 **不进支** → 恒等直通(golden 不变)。G1 直线飞行(G2 叠贝塞尔弧+噪声)。
- **验证**:目视首页 grid 编队(字飞成阵)+ 散场还原;e2e 2(编队显著改渲染 dFormed>2% + 散场 dAfter<
  0.35·dFormed;坏 JSON 整拒)。native/clippy(-D warnings:bbox 名去相似 / 测用 expect 非 unwrap /
  float 容差比较)/fmt 通过。**踩坑**:e2e 严格双拍/绝对恒等阈值受待机 pulse(时基)影响脆 → 改**相对差
  比**断言(散场明显靠回 base),帧字节确定由 native + 注入时钟架构保证,不靠 GPU 逐像素 golden。

## G2 · 花形 + 匹配 + 弧线扰动 + 备选形状

- **花形采样(formation.rs)**:
  - **rose(k 瓣花)**:round-robin 均分字到 k 瓣;每瓣 = 沿花瓣轴的**锥形瓣叶**(`along` 0.14→1 花心留空、
    瓣根分离;`halfw = 0.26a·sin(π·along)` 窄瓣不糊;黄金比 `side` 横向铺满)。比 `|cos(kθ)|` 包络更像花。
    `a`=花半径 px,`k`=瓣数,`seed`=朝向相位。
  - **heart(备选,证形状=数据)**:经典心形参数式 + phyllotaxis 角 + 径向填充 → 清晰心形轮廓。
- **角度桶贪心匹配** `match_by_angle`:glyph 与 point 各按(方位角,半径)`total_cmp` 排序,同序配对 →
  保角、避轨迹大交叉;双射 O(n log n)。`matched_targets` 回 glyph 序 + 按目标半径给 stagger(外瓣后到=
  向外绽放)。
- **VS 贝塞尔弧飞行 + 扰动**(glyph.wgsl):`base = bezier(p0, ctrl, form_target, fe)`,控制点在飞行线
  中点法向偏移(`seed` 定弧向/弧幅)→ 弧线不呆板;`wob = f(seed, fe)` 飞行中扰动,`fenv=4fe(1-fe)` 两端
  归 0 → **落定 form_target 恒等**(RD4);全 `f(seed,fe)` **无时基** → 定帧确定。
- **native 测(+2,共 5)**:rose 目标双射(60 target 互异)+ 形状=数据(rose≠heart)· 角度桶总飞行距离
  ≤ 0.7×任意(生成序)匹配(**基准数值断言**,DoD-2)。
- **验证**:目视首页 rose k=5 成放射瓣叶花 + heart 成清晰心形(shape=data);字节确定/退场恒等由 native +
  无时基 VS 保证。**遗留 → G4**:首页密集大字(CJK/公式)下花瓣仍偏挤,最终 a/k/内容集在 G4 幕接入时调
  (形状=数据,只换参数)。「成花定帧 golden」同 G1 取相对差 e2e + native 基准(GPU 逐像素 golden 跨机脆)。

## G3 · 花瓣层 + 风场 + 绽放脉冲

(未开始)

## G4 · 首页高潮幕 + 真 zoom + celebrate 槽 + 收口

(未开始)

## DoD 对账

(未开始)
