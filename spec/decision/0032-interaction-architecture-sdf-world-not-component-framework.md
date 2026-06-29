# 决策记录 0032:交互复杂度的归宿 —— SDF 世界 + 命中层 + DOM 逃生口,不引组件框架

- 日期:2026-06-24
- 状态:**提议中(草案,待评审)**
- 前置:[0001](0001-canvas-architecture.md)(React 管控件 + GPU 画布)、[0007](0007-rich-media-embeds.md)(否决 egui 做内容)、[0011 §3.3④](0011-gpu-text-as-sdf-primitive.md)(CPU 基础盒模型 hit-test)、[0012](0012-debugger-gui-html-vs-egui.md)(否决 egui 做调试器)、[0018](0018-sdf-panel-decoration-primitive.md)(SDF 面板)、[0020](0020-content-node-identity-model.md)(节点身份/树)、[0022](0022-dom-overlay-layer.md)(DOM overlay)、[0025](0025-sdf-node-animation-system.md)(SDF 效果)、[0030](0030-text-copy-selection-and-a11y-mirror.md)(DOM 管交互/GPU 管视觉)、[0031](0031-event-fsm-resilience-and-js-rust-boundary.md)(FSM `Blocked` 态)、[agent-ui 业界调研](../research/agent-ui-industry-survey.md)(Zed/GPUI 对照)
- 定位:**一次性钉死一个反复纠结的问题**——交互越来越复杂(子 agent、dock、折叠、表单)时,要不要引入 GPUI / egui 这类组件框架?**结论:不引。** 交互复杂度不靠渲染框架解决,靠**三层模型**:SDF 世界(核心)+ 命中/交互层(Rust 薄层)+ DOM 逃生口(仅真 HTML)。本篇是 0007/0012「否决 egui」的**泛化与收口**:把「调试器/内容」两个特例升级成「所有复杂交互」的总纲。

---

## 1. 触发:复杂交互让人误以为缺一个组件框架

子 agent 卡、permission/question dock、工具卡折叠、todo 勾选、diff per-hunk……交互越来越多,直觉是"该上个 GPUI / egui 来写组件了"。**这个直觉混淆了两件事:**

- **渲染框架**(GPUI / egui):自带 renderer + layout 引擎 + 事件循环 + 字体栈 + 输入系统。
- **交互模型**:"pointer → 命中谁 → 做什么" + 控件状态。这是**路由 + 状态**,不是"怎么画像素"。

**真实痛点是后者(缺一层交互),不是前者(渲染没问题)。** 你的 SDF 世界画卡片/面板/文字/diff 底色都没问题;卡住的是"点击进子 agent""展开折叠""勾选"——这些是几何命中 + 事件分发 + 控件态,几百行的事,不是一个框架。

## 2. 为什么 GPUI / egui 都不引(都是"第二个渲染世界")

| | egui | GPUI |
|---|---|---|
| 范式冲突 | 即时模式每帧 CPU 重建 ↔ **块冻结对冲**(0012/0011 已述) | 保留/即时混合,但**自带整套 element 树 + Taffy + 事件循环**,是另一个世界 |
| 可嵌入性 | paint-callback 可叠,但带第二套输入/字体 | **不可嵌入**:OWNS window/surface/事件循环,不是能 compose 进你 wgpu scene 的库;wasm 弱 |
| 包体 | 数百 KB(0012/0009 否决理由) | 更大;违背"可嵌入组件要轻"(0011/0009) |
| **与 SDF 世界的关系** | 第二渲染世界 → "文字 SDF、控件框架" | 同上 |
| **你最值钱的能力** | 框架控件**进不了 tier 虚拟化、不能被相机 zoom、不能 morph、不吃 spawn_time 特效** | 同上 |

**核心反对**:引入任一框架 = 在 SDF 世界旁边立**第二个渲染世界**(两个 compositor / 两套 layout / 两套输入 / 两套字体)。那一刻"SDF 世界是核心"退化成"SDF 是配角"。子 agent 卡若是框架组件,它**进不了你的虚拟化、缩放、morph、特效**——你为之建了 0016/0018/0025/0029 的全部能力它都用不上。**这违背项目根立场,且 0007/0012 否决 egui 的理由对 GPUI 同样(甚至更)成立。**

> 业界印证([调研](../research/agent-ui-industry-survey.md)):Zed 用 GPUI **从零自建** SDF primitive + glyph atlas + List 虚拟化,**没有**在 GPUI 之上再叠一个组件框架——它自己就是那层。你已经有等价的 SDF 世界(0011/0018/0029),再叠 GPUI 是叠床架屋。

## 3. 决策:三层交互模型(SDF 永远是核心)

```
① SDF 世界(core,护城河)      ── 所有内容/卡片/面板都是「块」:可缩放 / 可 morph / 被 tier 虚拟化 / spawn 特效
② 命中 / 交互层(Rust core,薄) ── CPU 基础盒(0011 §3.3④)+ 节点树(0020):pointer → 节点 → 动作路由 + 控件态
③ DOM 逃生口(0022,仅真 HTML) ── 表单/输入/IME/a11y(dock)、富嵌入(iframe/artifact);world_to_screen 同步
```

**分派规则(谁去哪层):**
- **能 SDF 画、要 zoom/特效/虚拟化 → ①**:卡片视觉、状态徽章、diff 底色、分隔线、reasoning 区。
- **点击/展开/拖拽/勾选/选区 → ②**:纯几何命中 + 事件路由 + 折叠态(by node id);**这是本篇新增的一层**,建在已规划的 0011 §3.3④ CPU 盒 + 0020 节点树上,不是框架。
- **真表单/真输入法/真无障碍/任意 HTML → ③**:dock(0031 `Blocked`)、artifact、富嵌入。

GPUI/egui 想统包的事被**正交切成三层**:视觉归 SDF、命中归节点树、真控件归 DOM。**没有第二个渲染世界。**

### 3.1 命中/交互层(②)的形态(本篇新增最小件)

- **命中**:复用 `code_block_at_screen` 已有的 screen→world 换算 + [0020] 节点树查询 → `hit_node(world_x, world_y) -> NodeId`(基于 CPU 基础盒,不回读 GPU、不用正在动的 SDF)。
- **路由**:`on_pointer(event)` → 命中节点 → 查该节点的**交互角色**(可点卡/折叠把手/复选框/链接)→ 发动作(enter-subagent / toggle / check / open-link)。
- **控件态**:折叠/选中态 by node id(append-only,与几何正交),是 **presentation 输入**(同相机/选区),**不进 reveal、不破 R8 重放**(同 0030/0031)。
- **与 0030 一致**:视觉交 GPU,交互的**命中**在 core(纯几何可测),交互的**真表单**在 DOM。

## 4. Worked example:子 agent(零框架)

opencode = 可点跳转的 task 卡;Zed = 行内卡 + 导航。**你的无限画布有独有解法,且全在现有体系内:**

| 维度 | 落点 | 复用 |
|---|---|---|
| **数据** | `SessionCreated{parentID}` → 登记子会话;task tool part = 一个卡 | 0031 |
| **渲染(SDF 块)** | task 卡 = SDF 面板块:agent 名 + 状态徽章(running 用 spawn 微光)+「(background)」 | 0018/0025 |
| **命中(②)** | pointer 命中卡 AABB → 路由"进入子 agent" | 0011 §3.3④ + 0020 |
| **空间形态(差异化)** | 别人只能 navigate/tab/nested;**你可"缩放进子 agent"**:子会话消息 = 同一 SDF 世界里的更多块,相机飞入 / 盒布局排成子区域簇 | 相机 + boxlayout(TODO2 B) |
| **要权限时** | permission dock = **DOM overlay**(屏幕固定、阻塞 turn);真 radio/输入框/IME | 0022 + 0031 `Blocked` |

**子 agent = SDF 卡(视觉)+ 命中(进入)+ 相机/盒布局(空间嵌套)+ DOM dock(若需表单)。一个框架都不用,且每层都让 SDF 世界更核心。** 尤其"缩放进子 agent"是组件框架**给不了**的能力——它只能在自己的矩形里画一个 panel,不能让子会话变成无边画布上的一片可缩放空间。

## 5. "canvas 里算 HTML 位置并保持同步"——已具备,边界明确

机制已有(`embed-overlay.ts`:`frame_embeds()` 每帧返回屏幕矩形 = world→screen÷DPR,DOM 按相机定位)。**经验法则:**

- **dock 类(permission/question/todo)→ 屏幕固定定位**,不跟内容滚 → **零同步缝**。这是交互控件的正确做法。
- **内容锚定 DOM(罕见:内嵌 iframe/artifact)→ 跟相机走**,惯性滚动滞后一帧是已知代价(0022/0007 §2.3);CSS transform 跟 camera 缓解;且必须随可见窗虚拟化(0029)。

即:**交互控件用屏幕固定 DOM(无缝);只有"必须贴某内容块"的富嵌入才进 content-anchored overlay(有缝、要虚拟化)。** 子 agent 卡是 SDF(本就跟相机),其 dock 是屏幕固定 DOM(无缝)——两边都不踩坑。

## 6. 决策与不决定的

**采纳**:**交互复杂度走三层模型**(SDF 世界 ① / 命中交互层 ② / DOM 逃生口 ③),**不引入 GPUI / egui 或任何组件渲染框架**。本篇新增最小件 = **② 命中/交互层**(CPU 盒命中 + 节点路由 + 控件态,建在 0011 §3.3④ + 0020 上,纯 core 可测)。子 agent / dock / 折叠 / 勾选 / diff 交互全部按三层分派落地。

**理由**:渲染框架是第二个渲染世界,稀释 SDF 核心、丢掉虚拟化/缩放/morph/特效、违背小包体与可嵌入(0007/0009/0011/0012 一致);交互痛点本质是缺一层薄命中路由,几百行即可,且让 SDF 世界更核心而非边缘;"缩放进子 agent"等差异化能力只有自有 SDF 世界给得了。

**不决定的(留实现/后续)**:命中层的事件协议(回调 vs 命令队列);折叠态持久化粒度;子 agent 的空间形态 v1(navigate)vs v2(空间嵌套,TODO2 B 产品决策);per-hunk diff 交互(若做可写 agent);命中层是否暴露给 web 做"透明热区"(与 0030 文本层热区合流)。

**重新评估触发条件**(承 0012 §6 精神):若项目从"可嵌入 wasm 组件"转为**独立桌面/原生 app**(包体顾虑消失)**且**需要海量字段级原生控件 → 可重评 GPUI(届时它替代的是整个渲染层,而非叠加)。当前形态下不重开。

## 7. 与既有决策的关系

- **泛化 0007/0012**:那两篇否决 egui 于"内容/调试器"两个特例;本篇升级为"所有复杂交互"的总纲,并补上 0012 当年没给的**正解(② 命中层)**。
- **0030 同构**:0030 说"DOM 管交互/GPU 管视觉"——本篇把"交互"再拆成"命中(core)"与"真表单(DOM)",更精确。
- **0022 是 ③**:DOM overlay 是本三层的第三层,不是替代 SDF。
- **0011 §3.3④ / 0020 是 ② 的地基**:CPU 盒 + 节点树早已规划,本篇把它们正式扶正为"交互层"。

---

参考:[0007](0007-rich-media-embeds.md)/[0012](0012-debugger-gui-html-vs-egui.md)(否决 egui,本篇泛化)· [0011 §3.3④](0011-gpu-text-as-sdf-primitive.md)/[0020](0020-content-node-identity-model.md)(命中层地基)· [0018](0018-sdf-panel-decoration-primitive.md)/[0025](0025-sdf-node-animation-system.md)(SDF 卡/特效)· [0022](0022-dom-overlay-layer.md)(DOM 逃生口)· [0030](0030-text-copy-selection-and-a11y-mirror.md)/[0031](0031-event-fsm-resilience-and-js-rust-boundary.md)(交互/状态)· [agent-ui 业界调研 §1](../research/agent-ui-industry-survey.md)(Zed/GPUI 自建 SDF、不叠框架)· [opencode 调研](../research/opencode-desktop-part-rendering-and-interaction.md)(dock/sub-agent 交互形态)。

> 可信度:Zed 自建 SDF 不叠框架、GPUI 不可嵌入为业界调研核实;其余为本项目既有决策的延伸。
