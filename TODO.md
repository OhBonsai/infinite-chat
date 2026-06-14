# TODO —— Plan 3 待办积压(K/L 之后)

- 维护说明:Plan 3 的 **K、L 已细化** 在 [spec/plan/plan3-canvas.md](spec/plan/plan3-canvas.md);本文件收**其余相位(M–S)+ Plan 2 欠账**,后续逐步细化/上提到正式 plan。
- 编号续 K、L → **M–S**;一个条目大致 ≈ 一个 Phase/PR。完成后从这里划走、补进 plan 文档。

---

## 相位积压(依赖:M–N 需 K/L 完成;O–S 需 L 的图元管线)

### M — 块装饰 quad + 表格对齐 + 词折行(追平 markdown 观感)
- [ ] 矩形/圆角图元:代码块底、行内码 chip、引用左条、表格网格+斑马+表头、hr、H1/H2 细线、GitHub Alert 左色条
- [ ] **拆 H1–H6 分级**(`content.rs` 现合并为一类 Heading)→ 逐级字号
- [ ] 表格**两趟列宽对齐**(max-content,padding 6px/13px,边框)
- [ ] **词边界折行 + CJK 禁则**(替换 `pretext-bridge` 逐 grapheme 贪婪折行)
- [ ] 固化 **github-theme 设计令牌**(字号/间距/色,见会话实测值)→ 喂 `StyleRole` 映射
- 参考:[0004](spec/decision/0004-markdown-and-embeds.md)、GitHub 令牌(`github-markdown-css`)
- 取舍:**自带字体 vs GitHub 字形一致 互斥**(0009/0011)——只追结构/间距/配色

### N — 富 shader 特效(依赖 SDF)
- [ ] SDF 发光 / 描边 / 溶解(fragment)
- [ ] 逐字 compute/vertex 动效(0011 §3.2);无状态时间效果走纯 VS(兼容冻结),有状态物理才 compute(先裁剪)
- 参考:[0007](spec/decision/0007-rich-media-embeds.md)、[0011 §3.2](spec/decision/0011-gpu-text-as-sdf-primitive.md)

### K′ — 双模文字图元:Canvas 位图(默认)+ SDF(特效)

> 背景:纯 SDF 渲染普通小字正文观感不如 Canvas2D 位图(无 hinting、固有偏软);决定**位图当默认/主力**(CJK+ASCII 正文、小字锐利),**SDF 仅 opt-in 用于特殊场景**。文字图元因此是**三种 tile 来源 × 同一 quad 管线**(顶点/相机/位置/shader 增强共用,只片元采样不同):

> 1. **位图(运行时 Canvas2D 覆盖率)= 默认主力** —— 正文、CJK+ASCII、小字锐利。
> 2. **SDF(运行时 TinySDF)= opt-in 特效** —— 任意 ASCII 加发光/描边/溶解/超大缩放。
> 3. **MSDF(离线预烘)= "文字当图片"** —— 固定的装饰/展示字集(logo、标语、特殊大字),离线烘最高质量、运行时零生成、拐角锐利、可叠特效。**这条用户明确需要**。

- [ ] **实例加 `kind: u32`**(0=位图覆盖率, 1=单通道 SDF, 2=**MSDF**, 3=Color/RGBA emoji);片元按 kind 分支:bitmap `cov=tex.r`;SDF `smoothstep`;**MSDF `median(r,g,b)` 再 smoothstep**(拐角锐);RGBA 直采。
- [ ] **atlas 分页按通道**:R8 页(位图/单通道 SDF)、**RGB 页(MSDF)**、RGBA 页(emoji);tile 记 kind;glyph key 带 kind。
- [ ] **光栅/来源分流**:位图 = 保留 Plan2 `rasterize`(**别退役,设为默认**);SDF = 运行时 TinySDF(有限 ASCII);**MSDF = 离线 msdf-atlas-gen 烘 atlas+metrics,启动加载、运行时只采不生成**。
- [ ] **谁走哪种**:layout/instance 加 `tile_mode`(默认 Bitmap);特效 run → SDF;**"文字当图片"实体 →（查离线 MSDF 字集,命中则)MSDF**(主题/角色/`<glow>` 类 0006 标签或专用实体类型)。属呈现选择,**不进 markdown 语义**。
- [ ] **位图在缩放下**(必须拍板):①(推荐)**按缩放档重栅**——浏览器光栅便宜,正文任意缩放都锐利;②接受放大糊(画布基本 1× 时最省);③超阈值自动升 SDF。
- [ ] **离线 MSDF 资源管线**:工具(msdf-atlas-gen / fdsm)烘"文字当图片"字集 → `atlas.png(RGB) + metrics.json`;放 `web/public/`,FontFace 无关(MSDF 直接采样,不经 Canvas2D)。**字集要小**(装饰字,不是全文)→ 体积可控。
- [ ] 文档已对齐:[0011 §3.5](spec/decision/0011-gpu-text-as-sdf-primitive.md)、[plan3-canvas Phase K](spec/plan/plan3-canvas.md)(双模→三源)。
- 性能账(已评估):每帧渲染三者基本相等;差别在**来源**——位图/SDF 运行时生成(SDF 多 EDT,别全量 CJK)、**MSDF 离线零运行时生成但要下载+常驻**(故仅用于小的"文字当图片"字集)。

### O — 嵌入块(图片 → mermaid → 卡片)
- [ ] 图片:浏览器解码 → 纹理 quad;mermaid:SVG → 浏览器光栅 → 纹理
- [ ] embed FSM:Placeholder → Loading → Ready → Failed;占位高度防 reflow;像素对齐
- [ ] wasm 只持元数据(尺寸/位置),重活交浏览器
- 参考:[0004 §7](spec/decision/0004-markdown-and-embeds.md)、[0007](spec/decision/0007-rich-media-embeds.md)

### P — 标签层 + 自定义语法
- [ ] pre-markdown segmenter + 标签注册表(hold 区、未知标签默认 Literal)
- [ ] `:::` 容器开启符(0006 §5.1);`<thinking>`/citation 区域 FSM
- [ ] 行内 chip:`@提及` / 引用角标(parse 后 span 后处理,0006 §5.2);`[^1]` 用 pulldown `ENABLE_FOOTNOTES`
- [ ] 安全:标签当数据,绝不当 HTML 执行
- 参考:[0006](spec/decision/0006-inline-tags-and-extensibility.md)、[0010 §5.1](spec/decision/0010-markdown-parsing-strategy.md)

### Q — input / 选区 / hit-test / 可点链接
- [ ] **CPU 基础盒模型**(§3.3④)做命中/选区/复制——不回读 GPU、不用正在动的 SDF
- [ ] 可点超链接(借 warp `hyperlink + Action`,0010 §5);脚注/引用跳转
- [ ] 选区跨折行、复制保真
- 参考:[0011 §3.3④](spec/decision/0011-gpu-text-as-sdf-primitive.md)、[0010](spec/decision/0010-markdown-parsing-strategy.md)

### R — 无障碍 DOM 镜像 + 渲染降级
- [ ] 可见内容 **DOM 镜像**(屏幕阅读器)——**可嵌入组件的硬需求,别拖到最后**;兼作"无 WebGPU 也无 WebGL2"的极端兜底
- [ ] **WebGL2 路专测**(已通过 `Backends::GL` 启用、自动兜底,但未测);处理其限制:**无 compute → 逐字 compute 特效降级为 vertex+fragment**(见 0011 §3.4)
- [ ] **Canvas2D 不做**(SDF/shader/compete 在 Canvas2D 上无意义;`RenderBackend` trait 留缝但不实现)
- 参考:[0003 §5](spec/decision/0003-fault-tolerance.md)、[0011 §3.4](spec/decision/0011-gpu-text-as-sdf-primitive.md)、`api` 模块(无障碍镜像)

### S — 公共 API + React/Vue 封装 + npm 打包
- [ ] 命令式 API / props / 事件 / 主题(`api` 模块)
- [ ] React、Vue 薄封装;`npm i` 即用
- [ ] **产物体积守门**(守"轻包体"原则)
- 参考:[0000](spec/decision/0000-overview.md)、README「交付形态」

---

## 可观测性(运行时,2026-06-14 起)

> 教训:K/L 调试期全靠截图 + 读代码反推(单巨块每帧发整篇),因为**只有 panic/生命周期日志,无性能可观测**。运行时、数据相关、GPU/主线程的退化,测试期严谨抓不到。

- [x] **节流帧统计**(`?debug`):每秒一行 `tracing info target=perf` —— fps / 帧耗时(avg+max)/ 丢帧 / **发射 glyph vs 总 glyph** / 可见块 vs 总块 / **atlas 占用·容量·累计淘汰**。emit≈total 即"每帧发整篇";evict 持续增长即 thrash。
- [ ] `performance.measure`(build_frame / draw)进 devtools Performance 面板。
- [ ] 关键路径计时 span(layout / rasterize / atlas alloc / draw)。

### 调试器(DOM 面板 + 引擎自绘几何,**不引 egui**,见 [0012](spec/decision/0012-debugger-gui-html-vs-egui.md))

> 分工:对齐世界/相机坐标的 → 画布(引擎自绘);数字/图表/控件 → DOM。3 条 dev-only 通道:状态快照拉取(`stats()`)/ 控制下发(setter)/ 点选查询(hit-test)。全挂 `?debug`,prod 零成本。

**数据通道(先打通)**
- [ ] `ChatCanvas.stats() -> JsValue`:一帧快照(FrameStats + atlas used/cap/evict + frame_ms + camera + smoother 水位 + FSM 状态);面板每 ~200ms 拉。
- [ ] 控制 setter:debug flags(overlay 开关)/ 特效参数(→ uniform/profile)/ cps / 相机操作。
- [ ] 点选 hit-test(用 0011 §3.3④ CPU 盒模型,不回读 GPU)→ inspector 读数;FSM/对账事件走环形缓冲。

**P1 流畅组**(目标第一优先)
- [ ] DOM:性能数字 + **sparkline**(fps/frame_ms)、暂停/单步;atlas used/cap/evict + thrash 警告。
- [ ] 画布:**块 AABB 裁剪框、SpatialGrid 格、visible_rect**(flag 开关)。

**P2 质量组**
- [ ] 画布:**glyph 包围盒、baseline、行框、按 role/kind 上色、atlas 纹理查看器**。
- [ ] DOM:点选某字 → cluster/role/pos/size 读数。

**P3 效果上限组**
- [ ] DOM:**SDF/特效参数滑杆热调**(RADIUS/edge/band/fade/glow/描边/溶解/compute 开关)、cps 滑杆、FSM 徽章 + 事件日志、smoother 水位计。
- [ ] 画布:**按 spawn_time 年龄上色、生长尾高亮**;(可选)单字放大预览、LOD 阈值线。

## Plan 2 欠账(可插空,喂上面相位)

- [ ] **真 pretext per-role 精确度量**(Plan2.5):粗/斜/code 排版宽度(measureText 现按 body 度量)→ 随 M 的 SDF 度量一并解决
- [ ] **语法高亮**(H5):tree-sitter / syntect-fancy-regex → 接 M 的颜色管线(GitHub `.pl-*` 调色板可直接抄)
- [ ] **Turn 完整分组投影(AR11)+ 折叠 tool/reasoning**(I5)
- [ ] **10k 行真机 fps/内存 benchmark**(§3 一直挂着的待验项)
- [ ] 可视滚动条 + 块内 glyph 级裁剪细化(G 推迟项)
- [ ] 显式心跳 backoff 强制重连(J2,当前用周期 resync + 自动重连覆盖)

---

## 已决策、勿重开(背景锚点)

- 解析沿用 **pulldown-cmark**,不手写 nom、不上 comrak;自定义语法走标签层不动解析器([0010](spec/decision/0010-markdown-parsing-strategy.md))
- 文字 = **SDF 图元**,自有引擎、借算法(TinySDF)不借框架(否决 AntV G / egui / cosmic-text / glyphon)([0011](spec/decision/0011-gpu-text-as-sdf-primitive.md))
- **字体打包自带**(放宽 BR5),浏览器固定主战场([0009](spec/decision/0009-text-rendering-engine.md)→0011)
