# 研究:AI Elements 与 shadcn chat components 对本引擎的可落地借鉴

- 日期:2026-06-30
- 类型:research(对照 + 落地建议)
- 对象:
  - **AI Elements**(Vercel)<https://elements.ai-sdk.dev/> —— 建在 shadcn/ui 之上、深绑 AI SDK 的 AI-native 组件注册表。
  - **shadcn chat components**(2026-06 phase 1)<https://ui.shadcn.com/docs/changelog/2026-06-chat-components> —— `MessageScroller / Message / Bubble / Attachment / Marker`,行为下沉到无样式 headless 包 `@shadcn/react`。
- 立场:本引擎是 Rust→WASM→WebGPU 的**无边画布对话渲染引擎**(SDF 文字、GPU 实例化、块冻结、infinite session),与这两个 React/DOM 库**技术栈不同**。因此借鉴**取观念、弃引擎**——抽其接口契约 / 状态模型 / 流式正确性规则,映射到本项目既有 ADR,不照搬 React 实现。
- 分级:🔵 高优先(应纳入近期 plan) · 🟡 中(值得排期/试做) · 🔴 仅参考(边界/不照搬)

---

## 0. 一句话结论

两库与本引擎在**深层设计上高度同源**(行为/状态 与 视觉/样式 分离、part 判别联合 + 兜底、流式正确性是一等公民、a11y 必须配 DOM 语义层),这反过来**交叉验证了本项目 0001/0021/0033/0030/0016-0019 的方向是对的**。真正的增量借鉴集中在三处:① shadcn 把"优秀流式滚动"显式拆成 **15 条规则**,本引擎的滚动/锚定还只覆盖其中一部分;② AI Elements 的 **Streamdown `parseIncompleteMarkdown`** 正是本项目 0017 §10 自陈短板("结构块先闪 raw")的成熟答案;③ 两库的**接口契约写法**(headless Provider + styled frame、part switch、prefix 安全白名单、live-region a11y 细节)可直接收敛进本引擎的 wasm 公开 API。

---

## 1. 两库速描(六维)

| 维度 | AI Elements | shadcn chat components |
|---|---|---|
| 接口设计 | 组合式小部件;props 对齐 AI SDK `UIMessage` 类型;`message.parts` 判别联合 switch 渲染;render-props(children 接 context) | **Provider(headless 状态)+ 多个具名 slot 部件**;行为全在 `@shadcn/react`,样式件只是壳;hooks 暴露命令/可见性/滚动态 |
| 组件设计 | Conversation / Message / Response / Reasoning / Tool / Sources / Branch…每件"做一件事";`Simplicity: when in doubt leave it out` | `MessageScroller`(滚动)/ `Message`(行)/ `Bubble`(面)/ `Attachment` / `Marker`(状态/系统/日期分隔/工具活动);"故意做小,靠组合" |
| 组件实现 | `Conversation` 基于 `use-stick-to-bottom`;`Response`=Streamdown;`Reasoning` 基于 Collapsible | 滚动热路径**绕开 React state**:命令式跟踪 + `data-*` 镜像;`content-visibility:auto` + `contain-intrinsic-size`;**虚拟化刻意不进 primitive**(留给 TanStack Virtual) |
| 性能调优 | "流式中最小重渲染、高效 DOM 更新、tree-shake、无运行时 CSS-in-JS" | "滚动/流式不触发任意 transcript row 重渲染";可见性追踪 **pay-for-what-you-use**(无订阅=零成本) |
| 兼容性 | 注册表 copy-paste,代码进你仓库;沿用你的 shadcn 主题/Tailwind;Apache-2.0、无锁定 | 同 headless 行为包**同时支持 Radix 与 Base UI**;CSS 工具(`scroll-fade`/`shimmer`)随 `shadcn/tailwind.css` 出货 |
| 稳定性 | a11y 内建(语义 HTML/ARIA/键盘/读屏);`parseIncompleteMarkdown` 抗半截语法;图片/链接 prefix 白名单 | 显式 **15 条流式滚动规则**("绝不违背读者意图移动");`scrollToMessage` 可在挂载前排队、挂载后缺失返回 `false` 不猜;`role=log`+`aria-relevant=additions`+`aria-busy` |

---

## 2. 逐维对照与借鉴

### 2.1 接口设计

**两库共识 = 本引擎 0021 划界 + 0001 契约的同构再现。**

- shadcn 把职责切得极干净:`MessageScrollerProvider` **只拥有"滚动视口"的状态**(打开位置、锚定、跟随、prepend 保位、可见性、滚动命令),**明确不拥有** messages / AI state / transport / persistence / branching / model state——"你来组合内容渲染器"。这与本引擎"wasm 引擎只管画布 + 少量配置,前端框架管外围控件"(README 三条定调①、0001)是同一条缝,但 shadcn 把**缝的两侧各拥有什么**列得更细。
  - 🔵 **建议 R1**:照此把本引擎 wasm 公开 API 的"拥有/不拥有"清单写进 0021。明确 `ChatCanvas` **拥有**:相机/视口裁剪、块冻结、reveal 编排、字形工作集;**不拥有**:消息真相源、transport、持久化、分支。让接入方(React/Vue)清楚边界,避免把 store 责任漏进引擎。
- AI Elements 的 `message.parts.map(switch part.type)` 判别联合,与本引擎 **0033 `PartKind` 分派注册表 + `fallback_render` 兜底**是同一模式(0033 §1 已自陈这是 Vercel/assistant-ui/CopilotKit 的收敛模式)。**交叉验证通过**。增量点:AI Elements 把"未知 part 默认 `return null`",而本引擎选择**未知必兜底显示、绝不静默吞**(README 原则 10)——本引擎此处**更稳更对**,应在 0033 显式标注此差异为有意决策。
- 接口"props 对齐协议类型":AI Elements props 直接吃 AI SDK `UIMessage`。本引擎 `RenderPart`(0033)已与 protocol Part 解耦(投影层),这点比直接吃协议类型更抗协议漂移,**保持**。

### 2.2 组件设计

- shadcn 的 **`Marker`**(统一承载:流式状态、工具活动、系统注记、日期/分隔、join/leave)是个干净抽象。本引擎已有 **TurnDivider**(0005:中断/compaction 插分隔线),但"流式中状态条 / 工具活动 / 日期分隔"目前散落。
  - 🟡 **建议 R2**:把这些收敛成一个 **`Marker` 类 part/装饰原语**(走 0018 SDF 面板装饰 + 0033 注册表),让"加一种标记 = 注册一行",而非散在各渲染器。
- `Reasoning` 组件:**流式时自动展开、结束自动收起**,并把模型可能吐的**多个 reasoning part 合并成一个块**(避免多个 "Thinking…")。本引擎有 thinking 标签(0006)与 reveal 编排(0019),但"多 reasoning part 合并 + 流末自动折叠"是具体可借的交互细节。
  - 🟡 **建议 R3**:在 0019 的门控/编排表里增加 reasoning 的 style 行:`gate=reasoning 段出现`、`choreography=展开→流末延迟收起`;合并逻辑放投影层(0033 `RenderPart`),不进机制。
- `ConversationDownload` / `messagesToMarkdown`:一键把会话导出 markdown。本引擎画布无 DOM 文本,导出需依赖 0030 的 DOM 镜像或一个 store→markdown 序列化器。
  - 🟡 **建议 R4**:给 `Store` 加一个确定性 `to_markdown()`(纯函数,CR1),既服务导出,又能作为 0030 a11y 镜像与测试快照的副产物。

### 2.3 组件实现 / 性能调优(本节最具增量)

**核心共鸣:两库都把"高频热路径移出框架的 reactive 系统",与本引擎"移出 CPU/主线程交给 GPU"是同一性能哲学。**

- shadcn 性能目标原文:"keep the scroll hot path **outside of React state**:no rerenders for rows, no forced layout on every scroll";滚动/锚定/跟随**命令式**追踪,经 `data-*` 属性镜像到 root/viewport。→ 等价于本引擎"块冻结 + settled O(1) 跳过 + 每帧一次跨界(0016 AR10)"。**交叉验证通过**。
- shadcn 的 **`content-visibility:auto` + `contain-intrinsic-size`**:屏外行留在 DOM(可选中/复制/Cmd+F/SSR/读屏),但浏览器跳过渲染。这是 DOM 世界对本引擎 **0029 四级工作集(Hot/Warm/Cold/FrozenFar)** 的"穷人版";而本引擎走得更远(连 CPU 侧几何/文本都分级释放,内存与可见一屏成正比)。
  - 🔵 **建议 R5**:把 0030 的 DOM 透明文本层**也套用 `content-visibility:auto` + `contain-intrinsic-size`**。本引擎 0030 已决定 DOM 文本层"仅可见块、虚拟化",但显式用这两个 CSS 属性,可在"略大于可见区"的缓冲带上省掉渲染成本而不丢选区/读屏/Cmd+F——直接补强 0030 §3 的 ① 层。
- shadcn **可见性追踪 pay-for-what-you-use**:无人订阅 `useMessageScrollerVisibility` 就零成本。
  - 🟡 **建议 R6**:本引擎的 `?debug` 帧统计 / 可见块上报(TODO 可观测性)采同款惰性:无订阅者时不计算 `visibleMessageIds`/`currentAnchorId`。
- **虚拟化的取舍差异(重要边界)**:shadcn **故意把虚拟化排除在 primitive 外**,理由是真实 DOM 行到"数千轮"仍够快,多数会话不需要;需要时才接 TanStack Virtual。本引擎相反——**虚拟化是北极星硬需求**(0029),因为 SDF 字形/几何的内存与历史线性增长,不分级就爆。
  - 🔴 **R7(边界)**:不要被 shadcn "虚拟化非必需"误导。两者结论不同是因为**成本结构不同**(DOM 行 vs GPU 字形工作集)。0029 的方向不变;但可借 shadcn 的"**把虚拟化做成可选层、不污染核心 primitive**"的封装姿态——本引擎应让 0029 工作集逻辑与渲染机制(0016)保持可分、可测、可关。

### 2.4 兼容性

- **headless 行为包 + 样式壳**:shadcn 把滚动行为放进 `@shadcn/react`,样式件只是壳,因此同一行为**同时支持 Radix 与 Base UI**。这正是本引擎"wasm 引擎(行为)+ DOM 壳(样式/控件)+ React/Vue 无关"(README ①、0021)的同构。**交叉验证通过**。
  - 🔵 **建议 R8**:把这条上升为本引擎的兼容性卖点与测试矩阵——**同一份 wasm,React harness 与 Vue harness 各跑一遍 smoke**,证明"行为不绑框架"。当前 `web/` 只有一个 harness;补一个最小 Vue 接入即可证伪框架耦合(对应 0021 划界的回归保护)。
- `scroll-fade`(滚动感知边缘渐隐,**无需 scroll listener、无 overlay**,纯 CSS mask)与 `shimmer`("Thinking…"微光)。
  - 🟡 **建议 R9**:本引擎边缘渐隐应**在 shader 里做**(视口上下边 alpha 衰减,近乎零成本),比 CSS mask 更自然;`shimmer` 微光等价于本引擎 `spawn_time` 淡入/GPU 节奏,已有能力,补一个"等待态微光"style 即可(0019 一行配置)。
- 分发模式:两库均 copy-paste/注册表、代码进用户仓库、无锁定。本引擎是 npm wasm 包(README 交付形态),路线不同但都强调**轻包体、无重依赖**(本引擎更硬:否决重型文字栈,0011 §2)。**保持**。

### 2.5 稳定性 / 正确性(第二增量重点)

**shadcn 的"优秀流式滚动 15 条"应作为本引擎滚动/锚定模块的验收清单。** 摘其与本引擎相关且可能尚未完整覆盖者:

1. **绝不违背读者意图移动**;auto-scroll 永不为默认——仅当读者已在底边才跟随。
2. **每个交互都是信号**:不止滚轮——选中文本、键盘、打开链接、搜索都应**停止跟随**。←(本引擎自动跟随的释放条件是否覆盖"选区/键盘/链接"?需核。)
3. **新回合锚到视口近顶部**,并保留上一条的一道 **peek**(`scrollPreviousItemPeek`,默认 64px),让新回合"长进屏幕"而非把旧内容推走。
4. **新内容可在屏外到达**,不改变读者正在看的位置;并**提示屏外有流式/新消息**。
5. **重开已存会话停在 `last-anchor`(最后一个有意义回合,通常是用户最后一条),不是绝对底部**;无锚则回退到 end。
6. **prepend 历史时保持读者位置**(`preserveScrollOnPrepend` + 稳定 `messageId`)。
7. **布局变化(图片加载、markdown 展开、代码块渲染、上方插旧消息)不得丢失读者位置。**
8. **中断/重试/重生成/分支/错误不得偷移位置。**
9. **`scrollToMessage` 可在 row 挂载前排队**(覆盖 permalink 先到、transcript 后挂载);挂载后缺失返回 `false`,**不进入猜测式重试循环**。

对照本引擎:第 3/4/6/7 条与 0016/0017/0029 的"几何变化不 snap、prepend、保位"高度一致;但 **第 2、5、8、9 条更偏"读者意图状态机"**,本引擎的 ADR 主要从**渲染机制**视角覆盖,从**滚动意图 FSM** 视角的覆盖较薄。

- 🔵 **建议 R10**:在本引擎新增/扩写一条 ADR(或并入 0031 event-fsm-resilience),把"**滚动跟随 FSM**"显式化:状态 = `Following / Released / Anchoring`;释放信号 = 滚轮/触摸/键盘滚动键/拖滚动条/选区/打开链接/显式 jump;`scrollToEnd`/jump-to-latest 重新进入 Following。这正是 shadcn `autoScroll` + `data-autoscrolling` 的语义,本引擎可在画布相机层实现同一状态机。把上面 9 条做成 native 测试/确定性重放用例(testing-and-benchmark)。
- 🔵 **建议 R11(对应 0017 §10 自陈短板)**:AI Elements `MessageResponse`(Streamdown)的 **`parseIncompleteMarkdown`** = 在渲染前**补全未闭合的 markdown 语法**,从根上避免"结构块先闪 raw 源(`| a | b |` 再 snap 成表)"——而 0017 §10 正把这列为当前不足。Streamdown 的做法(闭合补全/和解)与本引擎 0017 §5 的"remend / 保守预测 + 和解"同向。**借鉴**:把本引擎的"活跃块补全(remend)"提升为默认,并参照 Streamdown 对**表格/代码围栏/强调对**的半截补全规则,结合 0019 的 gate(结构块整行/整表就绪才揭示)消灭 raw 闪烁。这条同时呼应本项目北极星(结构块不闪 raw、先骨架再填字)。
- 🔵 **建议 R12(安全,几乎零成本即可补)**:Streamdown 暴露 `allowedImagePrefixes / allowedLinkPrefixes / defaultOrigin` —— 对模型吐出的图片/链接做 **prefix 白名单**。本引擎 README 原则 10 + 0006 已"标签一律当数据、不执行";但**图片/链接的 URL 白名单**是更具体的一层,应在 0004/0007(嵌入)与 0006 落一个**可配置 URL 白名单 + 默认拒未知协议/外链**的策略项。
- 🟡 **建议 R13(a11y live-region 细节)**:shadcn 的 `MessageScrollerContent` 用 `role="log"` + `aria-relevant="additions"`,流式时 `aria-busy=true`——**让读屏按"完成的消息行"播报,而非逐 token 噪音播报**;viewport `role=region`/`aria-label`/`tabindex=0`;滚动按钮无目标时 `inert`+`tabIndex=-1`。本引擎 0030 主攻选区/复制,**对"流式播报节奏"着墨少**。把这几条加入 0030 的 DOM 镜像规范,可直接消除"逐字播报"这一最常见的 canvas-a11y 翻车点。

---

## 3. 落地建议汇总(按优先级)

| # | 建议 | 关联 ADR/Plan | 级别 |
|---|---|---|---|
| R1 | wasm API 写明"拥有/不拥有"清单(对齐 shadcn Provider 边界) | 0021 / 0001 | 🔵 |
| R5 | DOM 透明文本层套 `content-visibility:auto`+`contain-intrinsic-size` | 0030 / 0029 | 🔵 |
| R8 | 补最小 Vue harness,证明"行为不绑框架"(兼容性回归) | 0021 / README① | 🔵 |
| R10 | 显式"滚动跟随 FSM"+ shadcn 15 条做成测试用例 | 0031 / 0016 / testing | 🔵 |
| R11 | remend 默认化 + 参照 Streamdown 补全规则,灭 raw 闪烁 | 0017 §10 / 0019 | 🔵 |
| R12 | 图片/链接 URL prefix 白名单(默认拒未知) | 0006 / 0004 / 0007 | 🔵 |
| R2 | 收敛统一 `Marker` 装饰原语(状态/工具/系统/日期分隔) | 0018 / 0033 / 0005 | 🟡 |
| R3 | reasoning:多 part 合并 + 流末自动折叠(投影层做合并) | 0019 / 0033 / 0006 | 🟡 |
| R4 | `Store::to_markdown()` 确定性序列化(导出 + a11y + 测试快照) | 0030 / CR1 | 🟡 |
| R6 | 可见性/调试统计 pay-for-what-you-use(无订阅零成本) | TODO 可观测性 | 🟡 |
| R9 | 边缘渐隐放 shader、等待态微光复用 spawn_time | 0011 / 0019 | 🟡 |
| R13 | 0030 补 live-region 规范(role=log/aria-busy,避免逐 token 播报) | 0030 | 🟡 |
| R7 | 边界:别被"虚拟化非必需"误导,0029 仍是硬需求 | 0029 | 🔴 |

---

## 4. 不该照搬的边界

- **不照搬虚拟化取舍**:DOM 行 vs GPU 字形工作集成本结构不同(见 R7)。
- **不照搬"未知 part return null"**:本引擎"未知必兜底显示"更稳(README 10 / 0033),保持。
- **不照搬整套 React 组件树/render-props**:本引擎是 SDF 世界、非组件框架(0032);借的是**状态/契约/规则**,不是 JSX 组合方式。
- **不引入重依赖**:`use-stick-to-bottom`/Streamdown/Radix 都是 DOM 库,本引擎只取其**算法与规则**,渲染仍走 wgpu(对齐 0011 §2"借算法不借框架")。

---

## 来源

- AI Elements 概览:<https://elements.ai-sdk.dev/>
- AI Elements Philosophy:<https://elements.ai-sdk.dev/docs/philosophy>
- AI Elements Conversation:<https://elements.ai-sdk.dev/components/conversation>
- AI Elements Message(含 `MessageResponse`/Streamdown/Branch props):<https://elements.ai-sdk.dev/components/message>
- AI Elements Reasoning:<https://elements.ai-sdk.dev/components/reasoning>
- Streamdown:<https://streamdown.ai/>
- shadcn 2026-06 chat components changelog:<https://ui.shadcn.com/docs/changelog/2026-06-chat-components>
- shadcn MessageScroller(15 条规则 / 性能 / 虚拟化 / a11y):<https://ui.shadcn.com/docs/components/message-scroller>
