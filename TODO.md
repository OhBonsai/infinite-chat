# TODO —— 产品相位 backlog(Plan 3 K/L 之后)

- **近期可做项已拆出** → [plan4-polish](spec/plan/plan4-polish.md)(排版收口 + markdown 观感 + 基础调试器):含原 M(折行/分级/装饰/表格/令牌/pretext 清理)、调试器 P1 + 数据通道、per-role 度量。
- **愿景/上限层** → [TODO2](TODO2.md):效果系统(原 N)、画布产品化、极致规模、交互深度。
- 本文件 = **剩余产品相位 + Plan 2 欠账 + 决策锚点**;一条 ≈ 一个 Phase/PR,完成后上提到正式 plan。

---

## 产品相位(依赖 L 的图元管线 / plan4 的装饰与调试器)

### K′ — 双模/三源文字图元:位图(默认)+ SDF(特效)+ 离线 MSDF(文字当图片)

> 现状代码是初版"全 SDF";本相位落地三源。背景/性能账见 [0011 §3.5](spec/decision/0011-gpu-text-as-sdf-primitive.md)。

- [ ] **实例加 `kind: u32`**(0=位图覆盖率, 1=单通道 SDF, 2=MSDF, 3=RGBA emoji);片元按 kind 分支:bitmap `cov=tex.r` / SDF `smoothstep` / MSDF `median(r,g,b)` 再 smoothstep / RGBA 直采。
- [ ] **atlas 分页按通道**:R8(位图/SDF)、RGB(MSDF)、RGBA(emoji);tile 记 kind;key 带 kind。
- [ ] **光栅分流**:位图 = Plan2 `rasterize`(**默认,不退役**);SDF = 运行时 TinySDF(有限 ASCII);MSDF = 离线 msdf-atlas-gen 烘 atlas+metrics(启动加载、运行时只采)。
- [ ] **谁走哪种**:layout/instance 加 `tile_mode`(默认 Bitmap);特效 run → SDF;"文字当图片"实体 → MSDF。不进 markdown 语义。
- [ ] **位图缩放策略**(拍板):①(推荐)按缩放档重栅;②接受放大糊;③超阈值升 SDF。
- [ ] **离线 MSDF 资源管线**:小装饰字集 → `atlas.png(RGB)+metrics.json` 放 `web/public/`,直接采样不经 Canvas2D。

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
- [ ] **CPU 基础盒模型**(0011 §3.3④)做命中/选区/复制——不回读 GPU、不用正在动的 SDF
- [ ] 可点超链接(借 warp `hyperlink + Action`,0010 §5);脚注/引用跳转
- [ ] 选区跨折行、复制保真
- 参考:[0011 §3.3④](spec/decision/0011-gpu-text-as-sdf-primitive.md)、[0010](spec/decision/0010-markdown-parsing-strategy.md)

### R — 无障碍 DOM 镜像 + 渲染降级
- [ ] 可见内容 **DOM 镜像**(屏幕阅读器)——**可嵌入组件硬需求,别拖到最后**;兼作"无 WebGPU 也无 WebGL2"极端兜底
- [ ] **WebGL2 路专测**(已通过 `Backends::GL` 启用、自动兜底,未测);处理其限制:无 compute → 逐字 compute 特效降级 vertex+fragment(0011 §3.4)
- [ ] **Canvas2D 不做**(`RenderBackend` trait 留缝但不实现)
- 参考:[0003 §5](spec/decision/0003-fault-tolerance.md)、[0011 §3.4](spec/decision/0011-gpu-text-as-sdf-primitive.md)

### S — 公共 API + React/Vue 封装 + npm 打包
- [ ] 命令式 API / props / 事件 / 主题(`api` 模块)
- [ ] React、Vue 薄封装;`npm i` 即用
- [ ] **产物体积守门**(守"轻包体"原则)
- 参考:[0000](spec/decision/0000-overview.md)、README「交付形态」

---

## 可观测性(运行时;P1 + 数据通道已入 plan4 4C)

- [x] **节流帧统计**(`?debug`):每秒一行 `tracing target=perf` —— fps / 帧耗时 / 发射 vs 总 glyph / 可见 vs 总块 / atlas 占用·容量·淘汰。
- [ ] `performance.measure`(build_frame / draw)进 devtools Performance 面板。
- [ ] 关键路径计时 span(layout / rasterize / atlas alloc / draw)。
- [ ] **调试器 P2 质量组**:画布 glyph 包围盒/baseline/行框/按 role-kind 上色/atlas 纹理查看器;DOM 点选某字 → cluster/role/pos/size。
- [ ] **调试器 P3 效果上限组**:DOM 特效参数滑杆热调 + cps + FSM 徽章 + 事件日志;画布按 spawn_time 上色 / 生长尾高亮。
- 参考:[0012](spec/decision/0012-debugger-gui-html-vs-egui.md)。

## Plan 2 欠账(可插空)

- [ ] **语法高亮**(H5):tree-sitter / syntect-fancy-regex → 接颜色管线(GitHub `.pl-*` 调色板可抄)
- [ ] **Turn 完整分组投影(AR11)+ 折叠 tool/reasoning**(I5)
- [ ] **10k 行真机 fps/内存 benchmark**(一直挂着的待验项)
- [ ] 可视滚动条 + 块内 glyph 级裁剪细化(G 推迟项)
- [ ] 显式心跳 backoff 强制重连(J2,当前用周期 resync + 自动重连覆盖)

---

## 已决策、勿重开(背景锚点)

- 解析沿用 **pulldown-cmark**,不手写 nom、不上 comrak;自定义语法走标签层不动解析器([0010](spec/decision/0010-markdown-parsing-strategy.md))
- 文字 = **quad 图元**,自有引擎、借算法(TinySDF)不借框架(否决 AntV G / egui / cosmic-text / glyphon)([0011](spec/decision/0011-gpu-text-as-sdf-primitive.md))
- **正文用浏览器系统字体栈**(零打包,小包体);固定字形仅离线 MSDF([0009](spec/decision/0009-text-rendering-engine.md)→0011)
- **不引 pretext**,手搓 layout;**BiDi/RTL 非目标**([0001 §2.2 修订](spec/decision/0001-canvas-architecture.md))
- 调试器 = **DOM 面板 + 引擎自绘几何,否决 egui**([0012](spec/decision/0012-debugger-gui-html-vs-egui.md))
