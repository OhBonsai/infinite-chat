# Plan 31 · GOAL:「不只是文字」——math 一等公民 + magic-move + DOM overlay + streaming-svg(agent 直跑,无人介入)

- 日期:2026-07-08
- 状态:**GOAL(可执行任务书)** —— 给 Claude Code 的完整执行合同,读完即可自主跑到 DoD。
- 优先级:**P2**(初心第五节「不只是文字」的能力扩展 + 差异化点)。
- 前置:
  - [decision/0013-math-latex-rendering](../decision/0013-math-latex-rendering.md) + [research/math-latex-sdf-rendering §6](../research/math-latex-sdf-rendering.md)(**翻转 0013:RaTeX 作 v1**——原生吐"定位字形 + 横线"扁平 display list,1:1 映射 FrameGlyph/SDF atlas,每字可走 0025 逐实例动画 = Manim 级)
  - [design/thinking.md §2](../design/thinking.md)(字块 move / magic-move / keyed reflow——差异化点,依赖数学字形进 quad 管线)
  - [decision/0022-dom-overlay-layer](../decision/0022-dom-overlay-layer.md) + [research/dom-overlay-layer](../research/dom-overlay-layer.md)(世界定位 DOM 叠加:tool 卡交互 / html embed / 图片交互半边)
  - [research/image-rendering-warp-jcode §5](../research/image-rendering-warp-jcode.md)(**streaming-svg 须路 B 矢量原生**,可变纹理做不到逐元素流式/缩放锐利/可 morph;建议新开 ADR 翻 0007 流式 SVG 条)
  - [decision/0016-streaming-morph-render-model](../decision/0016-streaming-morph-render-model.md)(reflow/embed 补间同一套 past→current)· [plan12-math-sdf-first-class](./plan12-math-sdf-first-class.md) · [plan14-image-embed](./plan14-image-embed.md)(位图链已落地)· [0020 节点树](../decision/0020-content-node-identity-model.md)(`NodeKind::Embed` 占位已留)

---

## 0. 内部顺序(本 GOAL 有依赖链,单 worktree 内串行)

`R1 math RaTeX → R2 magic-move`(magic-move 需数学字形先进 quad 管线);`R3 DOM overlay` 与 R1/R2 **文件基本不相交,可穿插并行**;`R4 streaming-svg` 承 R1 的矢量/atlas 经验,排最后。每完成一个 R 跑一次四层门。

## 1. GOAL 与完成定义(DoD)

**GOAL**:让引擎渲染**结构化非文本**且保持"每图元皆 SDF、可逐实例动画"的护城河——(R1) 数学公式经 RaTeX 原生 display list → 每字进 atlas 成 SDF 实例;(R2) magic-move:上下标/列宽变化时旧字形**平滑移到新位**(keyed reflow,非重建);(R3) DOM overlay:世界定位、随相机 pan/zoom 跟随的交互层(tool 卡折叠、html embed);(R4) streaming-svg:矢量原生路 B v1。

**DoD(全满足才算完)**:

1. **新 ADR 就位**(append-only,0009+ 编号,承 dev-practices §二):`00XX-svg-vector-native-vs-raster`(翻 0007 流式 SVG 条,承 0013 范式);若 RaTeX 决策与 0013 冲突,写 `00XX` superseding 记翻转(不改 0013 原文)。
2. **R1 math**:`$…$`/`$$…$$` → RaTeX display list → FrameGlyph/横线进 atlas;公式每字是 SDF 实例、可挂 0025 逐实例动画;基线/上下标/分数位置对(与 KaTeX 参考并排 ≤ 容差)。流式中公式不闪 raw TeX(骨架 hold 到可闭合再 typeset,对接 plan30 的骨架先行接口)。
3. **R2 magic-move**:`$x$`→`$x^2$`(x 抬为上标)、表格列变宽顶开——旧字形按**内容偏移/逻辑身份** keyed diff 匹配,`target_pos` lerp 平滑移动(0016 §3.2 机制),不"消失→重写"。峰值位移速度落在可追随区(接 [perception §5] <20–40°/s 硬边界)。
4. **R3 DOM overlay**(0022 §84-90):叠加容器 + billboard(`world_to_screen` + translate/scale 跟随相机、视口剔除 `display:none`);content 产 `NodeKind::Embed` + 世界矩形留位、layout 围绕它流;`reportSize` → 更新 world → relayout;embed 世界矩形走 0016 补间;主题随 0021 源切换。tool 卡折叠交互走 overlay 透明热区(opencode 调研 §4 路线 a:视觉 GPU / 交互 DOM)。
5. **R4 streaming-svg**(image-rendering §5 路 B):SVG 解析成矢量图元 → 复用 SDF/quad 管线 + 0016 补间;逐元素流式揭示(接 plan30 骨架先行)、缩放锐利;PNG 位图仍走 plan14 纹理路(分流)。
6. `node test/run.mjs` 四层全绿;每个 R **+1 native 测试**(math 布局定位 / magic-move keyed diff 稳定性 / embed 变换跟随 / svg 矢量解析)红→绿;golden 帧新增各 R 一帧,双跑一致。
7. `spec/plan/plan31_progress.md` 逐 R 记录(做了什么 / 参考 file:行号 / 容差结果 / 遗留)。
8. **全程不 git commit / 不 push**;工作区即交付。

**启动**:`claude --permission-mode acceptEdits "读 spec/plan/plan31-rich-content-goal.md,按其执行到 DoD 全满足"`(中断重跑同句,读 plan31_progress 续跑)。

## 2. Milestones = R1→R4(见 §0 顺序)

- **R1 · math RaTeX 一等公民**([math-latex-sdf §6]):**接入前必读** `crates/ratex-types` + `ratex-wasm` 的 `DisplayItem` 确切字段(§116 核实备注);`content` 产 `NodeKind::Math`(0020)承载 display list;字形/横线进 FrameGlyph/SDF atlas;KaTeX 字体入 atlas。**KaTeX/MathJax 不用于取坐标**(§43 结论)。
- **R2 · magic-move**(thinking §2):keyed diff(按内容偏移/cluster 身份,非屏幕位置,§80);`spawn_time` 旁加 `target_pos`+`move_start`,shader 两位置 lerp(与淡入同机制,0011 §3.2);先文本 append 前缀稳定,再 reflow(列宽/上下标)逻辑身份匹配;只在尾部活动块做,settled 不动。
- **R3 · DOM overlay**(0022 分阶段 §84-90):最小 billboard(手填锚点)→ Embed 节点 + 世界矩形 → reportSize 重排 → 接 0016 补间 → 主题联动 → 钩子(`onMount/onTransform/onVisibilityChange`,按 `EmbedKind` 注册宿主渲染器,与 0007 embed FSM 合流)。
- **R4 · streaming-svg**(image-rendering §5 路 B):矢量解析 → 图元表达(复用现有 SDF/quad + 0016);逐元素揭示;与 PNG 位图分流(§72)。**先开 §1 的 ADR 记路 A→路 B 翻转**再实现。

## 3. 客观判定

| 维度 | 判据 |
|---|---|
| math 定位 | 与 KaTeX 参考渲染并排:基线/上下标/分数盒 ≤ 容差(布局盒 ≤4px 或 ≤3%) |
| magic-move 稳定 | 同一 reflow 双跑:keyed diff 匹配对 byte 级一致;无"闪一下重建"(读图 + 断言旧 glyph id 复用) |
| 位移速度上限 | reflow 补间峰值位移速度 ≤ 可追随区(perception §5) |
| embed 跟随 | pan/zoom 时 overlay 世界矩形与 GPU 内容零脱节(共享相机 world=px,0007 §3) |
| svg 锐利 | 缩放 ×4 无栅格模糊(矢量原生证据) |
| 分流正确 | PNG→纹理路、SVG→矢量路,各走各的不混 |

## 4. 铁律(承 AGENTS.md / dev-practices §4)

- **AR9** 公式/图片流式前沿 hold 到可闭合(不闪 raw)· **AR1/AR3** 渲染只读、settled 恒等(magic-move 只在活动块,settled 不动)· **AR10** 每帧一次跨界 · **AR12** 未知 embed 类型 → 兜底不 panic。
- **CR1** core 零 wasm/web-sys/wgpu(RaTeX 若走 wasm crate,坐标/display list 解析放 core、平台调用走 seam,CR2)· **R8/R9** 确定性(magic-move 的 keyed diff 纯函数、无 `now()`/裸 rand)。
- **CR3** render 后端走 trait 不用 cfg 堆 · **T8** 视觉 off-profile golden · **T9** 先复现再改。
- **新 ADR 而非改旧**:0013/0007 的翻转写 superseding ADR,append-only,不改原文(AGENTS.md §8)。
- **0→1 一次做对**:math 直接走 RaTeX 目标形态(每字 SDF 实例),不先做"整张纹理"再重构;streaming-svg 直接路 B,不留路 A 可变纹理双轨。
- **不 commit / 不 push**;改 M6/M8 前过 AR 清单;改 render 先 `Skill(render-write)`;改 `web/` overlay 先 `Skill(bridge-write)`。

## 5. 文件归属(worktree 并行边界)

- **主战场**:`crates/core/src/math.rs`、`content.rs` 的 **Embed/Math 节点产出部分**、effects 的 `target_pos`/keyed diff、`render` 的 glyph/panel shader(magic-move lerp)、`web/src` 的 overlay 层、api 的 embed 盒子上报。
- **禁区(留给 plan29/30)**:`spatial.rs`/tier(plan29)、`reveal.rs` 节奏函数(plan30)。**骨架先行的"揭示时序"由 plan30 定**,本 GOAL 只提供公式/图片/svg 的**真身**(typeset/纹理/矢量),对 plan30 的骨架接口交付真身端点。
- content.rs 与 plan30 潜在重叠:plan30 碰 gate/reveal-unit,本 GOAL 碰 Math/Embed 节点产出;各在 progress 记一行触碰范围,合并时以节点产出(本 GOAL)+ gate(plan30)分层不冲突。

## 6. 风险与预案

| 风险 | 预案 |
|---|---|
| RaTeX `DisplayItem` 字段与调研假设不符 | 接入前先读源钉字段(§116);不符则回填 knowledge + 调整映射,不硬套 |
| magic-move keying 在插入处打乱前缀匹配 | 前缀稳定优先;reflow 用逻辑身份(cluster/内容偏移)非屏幕位;proptest 乱序/插入 |
| DOM overlay 与 GPU 快滚脱节 | 共享相机 world=px(0007 §3),overlay 与画布同一滚动量平移,非"追" |
| streaming-svg 矢量范围过大 | v1 只覆盖常见图元(path/rect/circle/text/line),复杂特性降级为整图纹理(记遗留) |
| headless WebGPU 起不来 | 唯一允许求助人的情形;native 逻辑(display list/keyed diff/矢量解析)先推进 |

> 一句话:**math 让每个公式字都是可动画的 SDF,magic-move 让流式变形平滑不重建,overlay/svg 让富内容进得来又缩得清**——把"不只是文字"这条初心补齐,同时守住"每图元皆 SDF"的护城河。
