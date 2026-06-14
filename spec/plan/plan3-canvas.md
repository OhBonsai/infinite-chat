# Plan 3(画布化)—— K/L:SDF 文字图元 + 无边画布底座

- 日期:2026-06-14
- 范围:**仅细化 K、L 两个相位**(引擎底座);M–S 及 Plan2 欠账见 [TODO.md](../../TODO.md)
- 前置:Plan 2(F–J)已完成([plan2_progress](./plan2_progress.md));决策 [0011](../decision/0011-gpu-text-as-sdf-primitive.md)(自有引擎 / SDF 文字图元 / 数据结构层)、[0001 §2.2](../decision/0001-canvas-architecture.md)(content→layout→render 契约)、[0007](../decision/0007-rich-media-embeds.md)(像素对齐相机)
- 相位编号续 F–J → **K、L**;一个 Phase ≈ 一个 PR,末尾 `/dev-wrap` 过卡口。

## 0. 主轴

把 Plan 2 的"一维滚动文字列"升级成 **SDF 文字图元 + 相机 + 空间索引的无边画布底座**。
**K = 文字表示换成 SDF**(位图→距离场),**L = 把渲染搬到 2D 相机 + 空间索引**。
两者都在 **content→layout→render 契约之下**做,`crates/core` 大脑一行不动,旧路保留一版可回退。

**In(K/L 做)**:SDF tile atlas、TinySDF 生成、距离场 shader、2D 相机平移缩放、空间索引、统一 quad 实例管线。
**Out(留 TODO)**:块装饰/表格/词折行(M)、富特效(N)、embed(O)、标签层(P)、input/选区(Q)、a11y/降级(R)、打包(S)、Plan2 欠账。

**全程铁律**:AR1 渲染只读状态 / AR10 每帧一次跨界 / CR1 core 零平台依赖 / 契约 0001 §2.2 不动 / R8–R9 确定性。

---

## Phase K — Glyph atlas:位图 → SDF 表示

> 域:`scene`(atlas/instance)· `render`(glyph shader)· `layout`(glyph 光栅来源)。
> 目标:文字成为**无级缩放清晰、可叠 shader 特效**的 SDF 图元,替换 Plan2 的位图 atlas 路径。

**背景**:现状 `crates/render/atlas.rs` 单页位图、`web/src/glyph-raster.ts` 用 Canvas2D `fillText` 出 alpha、`shaders/glyph.wgsl` 采样覆盖率。K 把 tile 换成 SDF。

**任务**
- **K1 `scene`/atlas 表示**:tile = **R8 单通道 SDF**;**固定 tile 尺寸 + 字号分桶**(几档尺寸,GPU 缩放补差);**多页纹理数组 + LRU 淘汰**(0004 §7.4 / 0011 §3.3③)。glyph key =(font, cluster, size-bucket)。可见字形钉住不淘汰。
- **K2 `layout`/SDF 生成**:移植 **TinySDF/ESDT**(自 infinite-canvas-tutorial `packages/core/src/utils/glyph/{tiny-sdf,sdf-edt,sdf-esdt}.ts`)——逐字 Canvas2D `fillText` → alpha 位图 → ESDT 距离变换 → SDF tile。**浏览器整形(含 CJK 禁则)+ `@font-face` 自带字体**。先在 **JS 侧生成**(复用浏览器整形)跑通,是否下沉 wasm 后议。
- **K3 `render`/shader**:`glyph.wgsl` 改读距离场:`alpha = smoothstep(0.5 - aa, 0.5 + aa, sdf)`,`aa` 用屏幕导数 `fwidth` → 任意缩放清晰。**保留 `spawn_time` 淡入**(乘进 alpha)。
- **K4 接口抽象**:把"atlas tile 表示"抽成接口(位图 ↔ SDF 可切),契约不变;`glyph_bridge`/`scene` 只换实现。**位图路径标注退役、保留一版可回退**。
- **K5 字体**:打包自选字体经 `@font-face` 注入测量/光栅上下文;CJK 回退字体**懒加载**(不压首包)。

**DoD**
- 同一字形单 tile,2×/4×/8× 缩放边缘清晰不糊;
- 记录拐角质量(单通道 SDF 圆角程度);小字号正文若不可接受 → 登记 **MSDF 触发条件**(0011 §6),本期不做;
- `spawn_time` 淡入仍工作;
- naga 校验新 shader;atlas **分配 / 淘汰 / 命中** 单测;
- 体积:新增依赖(若有)与产物增量记录(守"轻包体"原则)。

---

## Phase L — 相机 + 无边画布场景 + 空间索引

> 域:`render`(相机)· `scene`(世界坐标 + 2D 裁剪 + GPU 实例)· `input`(相机控制)· `app`(编排)。
> 目标:从 Plan2 的 1D `scroll_offset` 升到 **2D 相机 + 空间索引**,所有图元世界坐标统一管线。

**背景**:Plan2 (Phase G) 是 1D `scroll_offset` + glyph 按 y 二分裁剪。L 升到 2D 相机 + 空间索引,并统一 quad 图元。

**任务**
- **L1 `render`/相机**:2D 相机(pan x/y + zoom),view/proj 作 uniform;所有 quad 顶点走**世界坐标 → 相机变换**。HiDPI / 像素对齐(0007 §3)。
- **L2 `input`/相机控制**:**滚动 = 相机平移**(复用 G 的 wheel/touch),新增**缩放**(ctrl+wheel / 双指 pinch / 双击);Plan2 的 `scroll_offset` 收敛进相机状态;锚底改为相机跟随策略。
- **L3 `scene`/CPU 空间索引**:对象(消息/块/卡片/embed)入 **quadtree 或 AABB 网格**;视口查询 → 可见对象集(替代 G 的纯 y 二分,扩到 2D)。**脏区失效复用同结构**(数据结构 §3.3②:CPU 树)。
- **L4 `scene`/GPU 扁平采样结构**:可见对象 → **扁平实例 storage buffer**(GPU O(1) 取),每帧从 CPU 索引重生成可见集;**块冻结仍在**(settled 对象实例缓存)。CPU 树 / GPU 扁平网格分工(§3.3②)。
- **L5 图元统一**:**文字 quad / 矩形 quad / 图片 quad 同一实例管线 + 同相机 + 同裁剪**;本期给矩形/图片图元**最小占位实现**(为 M/O 铺路)。
- **L6 `app`/编排**:`build_frame` 改为"相机 → 空间索引查可见 → 取冻结实例 + 重排尾部 → 提交";保持 **每帧一次跨界(AR10)**。

**DoD**
- 画布可**平移 + 缩放**,≥60fps;像素对齐不糊;
- 2D 视口裁剪正确(平移到任意位置只构建可见对象);
- 10k+ 行 / 大量对象:fps/内存随**可见集**而非总量(benchmark,testing §3.4);
- 块冻结仍字节级稳定;Plan2 滚动行为无回归;
- 单测:空间索引查询正确性、可见集增量、相机变换数学。

---

## 风险与回退

- **K/L 是大改**:守 content→layout→render 契约,`core` 不动;**保留位图 + 1D 滚动旧路一版**,任一相位不通即回退(同 Plan1 §8)。
- **TinySDF JS↔wasm 边界**:先 JS 生成 SDF(复用浏览器整形)跑通,再议是否下沉 wasm。
- **缩放下文字 LOD**:远到 sub-pixel → 占位框,本期**只留 hook,不做全 LOD**(归 M/后续)。
- **K 与 L 顺序**:K 先(文字表示),L 后(相机/场景);L5 的矩形/图片图元只做占位,真装饰归 M、真 embed 归 O。

## 验证点(K/L 收尾填,喂后续)

| 验证点 | 预期 | 实测 |
|---|---|---|
| 缩放清晰度 | 2×–8× 边缘锐利 | 架构落地:R8 SDF tile + `smoothstep(0.5±fwidth)` shader(naga 校验);浏览器锐利度待真机 |
| 拐角质量 | 可接受 / 需 MSDF | 单通道 SDF(从位图)拐角会圆;待真机看小字号是否可接受,否则登记 MSDF(0011 §6) |
| 2D 裁剪 fps/内存 | 随可见集、与总量无关 | 已落地:SpatialGrid broad-phase + AABB narrow-phase + 块冻结 → 出 glyph 只与可见块成正比(cull 测);浏览器 fps 曲线待真机 |
| 相机平移缩放手感 | ≥60fps、像素对齐 | Camera2D(pan/zoom-at-point)+ ctrl-滚轮接好;数学 native 测(锚点不动/缩放/平移);手感待真机 |
| 块冻结回归 | 字节级稳定 | ✓ counting-layout 测仍过(settled 块不重排) |

> 完整落地记录见 [plan3_progress.md](./plan3_progress.md)。
