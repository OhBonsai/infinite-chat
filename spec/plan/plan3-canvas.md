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

## Phase K — Glyph atlas:双模(位图默认 + SDF 特效)

> 域:`scene`(atlas/instance)· `render`(glyph shader)· `layout`(glyph 光栅来源)。
> 目标:**双模文字图元**——位图(Canvas2D 覆盖率,带 hinting、小字锐利)为默认主力;SDF(距离场,无级缩放清晰、可叠特效)为 opt-in 特效模式。两者同一 quad 管线,仅片元按 `kind` 分支。

> **修订(2026-06-14)**:初版为"位图 → SDF、位图退役";真机验证纯 SDF 小字正文不如位图(无 hinting、偏软),改为**双模**:**位图不退役、设为默认**,SDF 仅特效用。详见 [0011 §3.5](../decision/0011-gpu-text-as-sdf-primitive.md) 与 [TODO「K′ 双模文字图元」](../../TODO.md)。

**背景**:现状 `crates/render/atlas.rs`、`web/src/glyph-raster.ts`、`shaders/glyph.wgsl`。K 加 SDF tile 与距离场采样,并以 `kind` 与位图覆盖率共存。

**任务**
- **K1 `scene`/atlas 表示**:tile = **R8 单通道 SDF**;**固定 tile 尺寸 + 字号分桶**(几档尺寸,GPU 缩放补差);**多页纹理数组 + LRU 淘汰**(0004 §7.4 / 0011 §3.3③)。glyph key =(font, cluster, size-bucket)。可见字形钉住不淘汰。
- **K2 `layout`/SDF 生成**:移植 **TinySDF/ESDT**(自 infinite-canvas-tutorial `packages/core/src/utils/glyph/{tiny-sdf,sdf-edt,sdf-esdt}.ts`)——逐字 Canvas2D `fillText` → alpha 位图 → ESDT 距离变换 → SDF tile。**浏览器整形(含 CJK 禁则)+ `@font-face` 自带字体**。先在 **JS 侧生成**(复用浏览器整形)跑通,是否下沉 wasm 后议。
- **K3 `render`/shader**:`glyph.wgsl` 改读距离场:`alpha = smoothstep(0.5 - aa, 0.5 + aa, sdf)`,`aa` 用屏幕导数 `fwidth` → 任意缩放清晰。**保留 `spawn_time` 淡入**(乘进 alpha)。
- **K4 双模(非退役)**:实例加 `kind`(0=位图覆盖率【默认】/1=SDF/2=RGBA emoji),atlas 统一 R8、tile 记 kind;片元按 kind 分支(bitmap `cov=tex.r` / SDF `smoothstep`)。**位图(Plan2 `rasterize`)保留为默认主力,不退役**;SDF 光栅仅对标记特效的少量 ASCII 生成。`tile_mode` 由 layout/角色决定(详见 [TODO K′](../../TODO.md))。
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
- **单消息 = 一个巨块,块级裁剪不够(已踩)**:`views` 每 part 一块,一条长消息是**一个巨块**,块级 AABB 永远与视口相交 → 每帧发射整篇 glyph + 逐字 `String::clone` → **主线程分配风暴 → 卡顿、滚动像冻**。必须**块内 glyph 级 y 裁剪**(已加)。后续可按 y 二分(placed 行序 y 单调)或把消息**按 md 块拆 view**让块冻结/块裁剪真正生效。
- **Rust 改动必须重建 wasm(易误判)**:scroll/相机/`build_frame`/atlas 都在 Rust;只 `npm run dev`(JS 热重载)看不到效果,易错判"滚不动/没生效"。改 Rust 后务必 `wasm-pack build` 再验。
- **小字偏软是单通道 SDF 的上限**:RADIUS/阈值调到位后仍比位图软是物理限制(无 hinting);再要锐只剩 **TILE_PX 提分辨率**(+Rust 同步+重建+atlas 内存↑)或 **MSDF**(0011 §6)。

## K/L 实测调优记录(2026-06-14)

- **字体(实时路径)**:接 `LXGWWenKaiMono-Light.ttf`(等宽,CJK+拉丁)→ `web/public/fonts/`;`main.ts` 用 FontFace **await 加载**再 `new ChatCanvas`(否则首帧用系统兜底出错字);`pretext-bridge` 字体族切到 `"LXGW WenKai Mono"`。等宽 → advance 稳定。
- **SDF 参数**:`RADIUS` 16→**8**(≈fontPx/6,细笔画峰值升到 ~0.69,饱满);shader 边缘阈值 0.5→**0.46**(浅字深底加粗找回字重);AA 带 **0.6×fwidth**(收窄更锐)。
- **quad 模型修正(关键)**:从"advance 矩形 quad + 整方 tile UV"(导致窄拉丁字**横向压扁**)改为 **方形 cell quad**(= tile footprint 缩放)+ **advance 仅推进笔位/折行**;`pos = 笔位 - tile 留白偏移`。`TILE_PX`/`SDF_BUFFER` 收为 `pretext-bridge` 单一来源、glyph-raster 复用(几何须一致)。
- **性能修正(关键)**:`build_frame` 加 **glyph 级 y 裁剪**(块内只发与视口相交的字),把每帧发射量从"整篇"降到"约一屏",根治长消息卡顿。
- **效果定性**:渲染观感已可用(markdown 结构/配色/CJK 正常);剩余小字软为单通道 SDF 上限(见上风险)。

## 验证点(K/L 收尾填,喂后续)

| 验证点 | 预期 | 实测 |
|---|---|---|
| 缩放清晰度 | 2×–8× 边缘锐利 | 架构落地:R8 SDF tile + `smoothstep(0.5±fwidth)` shader(naga 校验);浏览器锐利度待真机 |
| 拐角质量 | 可接受 / 需 MSDF | 单通道 SDF(从位图)拐角会圆;待真机看小字号是否可接受,否则登记 MSDF(0011 §6) |
| 2D 裁剪 fps/内存 | 随可见集、与总量无关 | 已落地:SpatialGrid broad-phase + AABB narrow-phase + 块冻结 → 出 glyph 只与可见块成正比(cull 测);浏览器 fps 曲线待真机 |
| 相机平移缩放手感 | ≥60fps、像素对齐 | Camera2D(pan/zoom-at-point)+ ctrl-滚轮接好;数学 native 测(锚点不动/缩放/平移);手感待真机 |
| 块冻结回归 | 字节级稳定 | ✓ counting-layout 测仍过(settled 块不重排) |
| 字形比例(不变形) | 窄字不被横向压扁 | ✓ 改方形 cell quad + advance 推进笔位后修复;真机确认 |
| 自带字体接入 | Canvas2D 测量/光栅用 LXGW 等宽 | ✓ FontFace await 加载 + 族切换;CJK+拉丁正常 |
| 长消息滚动/性能 | 每帧发射 ≈ 一屏,滚动顺 | ✓ 加 glyph 级 y 裁剪;**须重建 wasm 验**(之前单巨块每帧发整篇致卡) |

> 完整落地记录见 [plan3_progress.md](./plan3_progress.md)。
