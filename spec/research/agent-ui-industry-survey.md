# 业界开源 agent UI 对照研究(技术 + 设计)

- 日期:2026-06-24
- 状态:研究 / 设计依据(把 [opencode 调研](./opencode-desktop-part-rendering-and-interaction.md) 放进行业坐标;补强 Plan 22/23、0031)
- 范围:跨 ~20 个开源/文档化 agent 实现,提取**技术(渲染/数据模型/事件模型)+ 设计(交互/折叠/审批/流式)**,聚焦对**本项目(GPU SDF 对话引擎)**可迁移的结论。
- 方法:5 路并行 web 调研(IDE 扩展 / 事件平台 / UI SDK / GPU·TUI / 跨切设计模式),引用见各节。可信度:官方文档/源码为高,设计观点/二手已标注。

---

## 0. TL;DR(对本项目最重要的 6 条)

1. **Zed/GPUI = 你的技术最近邻,逐条验证你的架构**:Rust + GPU、**专用 SDF/shader 画圆角矩形/阴影**、**glyph atlas + shaping 缓存 + 单次 instanced draw**、**List+SumTree 可见性虚拟化**——你 0011/0018/0029 的每个核心选择 Zed 都做了同样的事。**你不是孤例,有一个生产级 GPU 编辑器走在同一条路上。**
2. **数据模型已收敛,验证 Plan 22/23**:`message parts 数组 + tool 按名注册表 + tool-call 四态(pending→running→completed→error)`是 Vercel AI SDK / assistant-ui / CopilotKit / opencode **共同抽象**。你的两级分派(`part.type→tool`)+ `ToolState` 正在主干上。
3. **非 DOM 渲染有成熟经验可抄**:Claude Code fullscreen「只渲染可见消息→内存恒定」、opentui「shadow buffer + cell diff」、Crush「两阶段渲染 + 按宽度缓存 + 动画在缓存外」——全部映射到你的 GPU「retained scene + 帧间 diff + 块缓存 + 动画分层」。
4. **验证你的北极星(再一次)**:业界把「skeleton-then-fill + debounce 防闪 + 结构块原子渲染 + 平滑 cadence 优先于吞吐」当流式共识(CHI 2026 JIT pacing 把 boundary-aligned pacing 当一等调度目标)。你的 reveal cadence 是被学界/工业双重背书的方向。
5. **HITL 是一条光谱**:从最轻(Vercel `approval-requested` + `addToolApprovalResponse`)到最全(LangGraph Agent Inbox 四动作 `accept/edit/respond/ignore`)。你 0031 的 `Blocked` FSM + DOM dock 落在中段,可借 Agent Inbox 的四动作 schema。
6. **涌现的设计语言**(用户已默认预期):progressive disclosure(默认折叠+一键展开+error 默认展开)、reasoning 完成后 auto-collapse、**per-hunk diff accept/reject**、plan-then-execute、graduated autonomy(read 自动/危险硬拦/allowlist 渐进)、history 与 context 解耦。

---

## 1. 技术最近邻:Zed Agent Panel + GPUI

> **这是全行业里和你架构最像的项目**,值得单列。Zed = Rust 编辑器,自有 GPU UI 框架 GPUI(非 DOM)。

**渲染管线(= 你的思路)**:每帧 `Layout(Taffy Flexbox)→ Prepaint(hitbox/滚动)→ Paint(生成 Scene:quad/glyph/path/shadow primitive)→ GPU Submit(Metal/wgpu)`。primitive 都是**专用 SDF/shader**:圆角矩形 = 矩形 SDF;阴影 = Evan Wallace 的 erf 高斯近似(不采样邻域);**文本 = OS shaping(CoreText)缓存 + 只存 alpha 的 glyph atlas(etagere bin-pack)+ 单次 instanced draw,颜色靠 alpha×color 乘法(同字不同色不重复存)**。绘制顺序固定 `shadow→rect→glyph→icon→image`。([Zed: Leveraging Rust and the GPU](https://zed.dev/blog/videogame))

> **对照你**:你的 0018 SDF 面板 = Zed 圆角矩形 SDF;你的 atlas + TinySDF = Zed glyph atlas(它甚至也是 alpha-only + tint);你的绘制顺序 panel→rect→…→glyph = Zed 的 shadow→rect→glyph。**几乎同构。** 一个可借的优化:Zed 明确「shaping 结果按 text-font 缓存,流式时只 reshape 变化行」——和你"块冻结只动尾块"同源,可确认你的 layout 缓存也缓存了 shaping。

**虚拟化(= 你的 0029)**:`UniformList`(等高,O(1))/ `List`(变高,**SumTree 索引高度,O(log N)**)。**Agent Panel 的 `ThreadView` 用 `List`**,只渲染可见 entry + `entry_view_state` 跟踪每块展开态。([Zed Rendering Pipeline — DeepWiki](https://deepwiki.com/zed-industries/zed/2.2-rendering-pipeline-and-layout))

> **对照你**:你 0029 Hot/Warm tier ≈ Zed List 可见性渲染。**可借**:Zed 用 SumTree 做变高项 O(log N) 高度索引——你现在 SpatialGrid 每帧 O(总块) 重建(Plan 19 已识别),**SumTree/前缀和是更优的高度索引**,可作 TODO2 C quadtree 升级的替代方案。

**Agent 对话渲染(`agent_ui`)**:`ThreadView` 用 `list()` 渲 entry(`UserMessage`/`AssistantMessage`/`ToolCall`);**tool-call 行内带状态指示器 + 权限粒度下拉(`PermissionSelection`)**(审批是内联控件,非弹窗);**`AgentDiffPane` multi-buffer 逐 hunk accept/reject**(复用编辑器 diff 能力);thinking 专用块;sidebar 线程状态图标 `Running(spinner)/WaitingForConfirmation/Error/Completed`。([Zed Agent Panel — DeepWiki](https://deepwiki.com/zed-industries/zed/8.1-agent-panel-and-ui))

> **对照你**:Zed 把**审批做成行内下拉**而非模态 dock——和 opencode 的 dock 不同路线。你可二选一:轻交互行内(Zed)/ 阻塞 dock(opencode)。Zed 的 **diff 复用编辑器 hunk accept/reject** 提示你:diff 块应支持 per-hunk 接受(Plan 23 R3 可加)。

---

## 2. 收敛的数据模型(验证 Plan 22/23)

**三件套已成业界共识**:① message = parts 数组(非单一 content)② tool 名 → 渲染器注册表 ③ tool-call 四态生命周期。

**tool-call 四态对照**(各家命名不同,语义一一对应):

| 阶段 | Vercel AI SDK v5 | assistant-ui | CopilotKit | **opencode / 你** |
|---|---|---|---|---|
| 参数流式 | `input-streaming` | `running`(argsText 部分) | `inProgress` | `pending` |
| 输入就绪/执行中 | `input-available` | `running` | `executing` | `running` |
| 完成 | `output-available` | `complete` | `complete` | `completed` |
| 出错 | `output-error` | `incomplete(error)` | — | `error` |
| 待批准 | `approval-requested` | `requires-action` | HITL respond | (你 0031 Blocked) |

**注册表对照**:Vercel(纯 `switch(part.type)`,无注册表)< CopilotKit(`useRenderTool({name})` + `"*"` fallback + 内置默认)< assistant-ui(`Tools().by_name` map + `ToolFallback`,最显式)≈ **opencode `ToolRegistry`(by-name + `GenericTool` fallback)≈ 你 Plan 23**。([Vercel UIMessage](https://ai-sdk.dev/docs/reference/ai-sdk-core/ui-message)、[assistant-ui Tools](https://www.assistant-ui.com/docs/guides/tools)、[CopilotKit tool-rendering](https://docs.copilotkit.ai/generative-ui/tool-rendering))

> **对 Plan 22/23 的修正/补强**:
> - **补 fallback renderer**(对标 `ToolFallback`/`useDefaultRenderTool`):未注册工具开箱可画(你 Plan 23 已提 GenericTool,确认保留)。
> - **可选细分 `pending`**:Vercel 把"参数流式(`input-streaming`,partial args)"与"输入就绪(`input-available`)"拆两态。流式工具卡若要显示"正在拼参数"vs"参数齐了在执行",可把你的 `pending` 拆 `streaming/ready`。**非必须**,opencode 用单 `pending` 也活得好。
> - **parts 词汇对齐**:业界 part 词汇 `text/reasoning/tool/file/source/data/step-start` 高度一致;你 Plan 22 的扩展集对齐即可(注意 `source`=引用来源,你可作行内角标)。

---

## 3. 非 DOM 渲染经验(TUI/GPU → 你的引擎)

> 终端 cell 约束 ≈ GPU 块约束。这批经验最可直接搬。

| 经验 | 出处 | 映射到你的 GPU 引擎 |
|---|---|---|
| **只渲染可见消息 → 内存恒定** | Claude Code fullscreen(alt-screen,render tree 只留可见项);opentui 只算可见;Zed List | = 你 0029 tier(已做);确认工具卡/diff 也只为可见块生成几何 |
| **shadow buffer + cell diff(只 flush 变更 cell)** | opentui(Zig);Claude Code 闪烁根因=全屏清屏 | = retained scene + 帧间 diff;流式 token 只更新受影响 glyph run,不重建整树(你块冻结已是此) |
| **两阶段渲染 + 按宽度缓存** | Crush `RawRender(无样式,缓存)`/`Render(套样式)`,width 为 key | = 你"content 解析→layout 块"缓存,宽度变才失效(你 BlockCache 已按 (block,width) 缓存) |
| **动画放在缓存层之外** | Crush spinner 绕过缓存;Zed 流式靠"帧间文本几乎不变" | = spinner/进场动画走 GPU spawn_time(零 CPU,你已是);别让动画使整块重排 |
| **固定输入框 + 流式不上移** | Claude Code fullscreen | = 输入框 DOM overlay 固定底部;画布锚底跟随(你已有) |
| **tool 卡=有状态可折叠块,按类型分派** | Crush(每 tool 一 renderer,折叠 bool,子 agent 用 lipgloss/tree 树形 compact) | = Plan 23 的 SDF 卡 + 折叠 + 按工具注册;子 agent 可树形 |
| **大输出:截断+展开/外部打开** | Claude Code(thinking 10 行截断、tool 结果折叠点击展开、`[` 写回 scrollback、`v` 外部编辑器) | = 折叠态 + 命中展开 + "在新视图打开完整内容"逃生口 |
| **markdown/高亮纯函数式缓存** | glamour(Goldmark AST→ANSI,Chroma 高亮,刻意 pure 便于缓存) | = 你 content/highlight 已纯函数 + 按 hash 缓存 |

> **一条新启发(SumTree/cell-diff)**:opentui 的 cell-diff 和 Zed 的 SumTree 都指向同一件事——**长会话渲染的命门是"只算/只画变化的部分"**。你已做了块冻结 + tier;下一个杠杆是把 SpatialGrid 每帧 O(总块) 重建换成增量高度索引(SumTree/前缀和),呼应 Plan 19 的发现。

---

## 4. 事件/状态模型对照(补强 0031)

- **OpenHands(事件模型最成熟)**:一切是 **Event**,非 `Action`(agent 决定做什么,带 `thought/reasoning/thinking_blocks`)即 `Observation`(结果);**不可变 append-only log = agent 记忆 + 可 replay/pause-resume**;`source(user/agent/environment)` 与 LLM `role` **故意解耦**;并行调用按 `llm_response_id` 分组;两类错误(`AgentErrorEvent` LLM 可见可恢复 / `ConversationErrorEvent` 终态)。([OpenHands Events](https://docs.openhands.dev/sdk/arch/events))
  > **对你 0031**:① append-only event log + replay = 你 R8 录像重放(你已有,更强);② **source≠role 解耦**值得借:判断"是否真用户输入"看 source,不从 role 推(你的 protocol 可加 source 归因);③ 两类错误区分 = 你 F3(ghost-abort 可恢复)vs F8(终态注卡)。
- **LangGraph `stream_mode` 多投影**:`updates`(state diff 增量)/`values`(全量)/`messages`(token)/`custom`(进度)——同一事件流按消费者切分。([LangGraph Streaming](https://docs.langchain.com/oss/python/langgraph/streaming))
  > **对你**:= 你"同一事件流喂 reveal(token)/store(state)/render(块)";LangGraph 把它做成显式 API,印证你的管线分相。
- **Aider edit formats**:`diff(SEARCH/REPLACE)/diff-fenced/whole/udiff` + architect/editor 双模型分离。([Aider edit-formats](https://aider.chat/docs/more/edit-formats.html))
  > **对你**:diff 渲染源是 LLM 输出契约(opencode 用 tool metadata.filediff);你只需消费已解析的 filediff,不必关心 edit format。

---

## 5. 涌现的设计语言(用户已默认预期 → 该采纳的)

| 模式 | 共识做法 | 路线分歧 | 你该做 |
|---|---|---|---|
| **tool 卡** | 默认折叠 + 一行可验证 summary(做了什么)+ 一键展开;**error 默认展开** | — | Plan 23 R2 采纳:折叠态 header + 命中展开;error 卡默认展开 |
| **reasoning** | 运行时流式展开 + 完成 **auto-collapse** + dimmed + 留 heading | **raw CoT 是否暴露**:OpenAI 隐(只 summary)/ DeepSeek 全显 / Anthropic 折中(summarized + 折叠) | Plan 23 R1:思考区默认弱化;完成折叠成"Thought…"一行;raw 显隐作配置 |
| **diff 评审** | inline diff + syntax + **per-hunk accept/reject** + "N files changed" + file-tree | Copilot 已落地 per-hunk;Claude Code/Codex 仍是 feature request | Plan 23 R3 加 per-hunk accept/reject(若你做可写 agent;纯展示可先省) |
| **HITL 审批** | 危险/不可逆/付费动作插 checkpoint;`approve/deny/always`;allowlist 渐进;YOLO 仅沙箱 | 模型自判 `requires_approval`(Cline)vs 显式 allow/deny prefix(Roo) | 0031 `Blocked` + dock;借 Agent Inbox 四动作 `accept/edit/respond/ignore` schema |
| **plan/todo** | todo 三态 `pending/in_progress/completed` 实时勾选;≥3 步才用;**plan-then-execute**(改前看计划) | — | todo → DOM dock(Plan 23 R6);plan mode 是产品决策(可后置) |
| **sub-agent** | nested(递归内联)/ tabs / **navigate(进子会话)** 三路线 | opencode/Zed 用 navigate | 你无限画布有独特优势:**可"缩放进子会话"或空间并排**(TODO2 B);v1 先 navigate |
| **streaming** | skeleton-then-fill + debounce 防闪(50–100ms)+ 结构块原子渲染 + 平滑 cadence > 吞吐 | — | = 你北极星(已是);确认结构块"先骨架后填字"(0019) |
| **规模** | 虚拟化 + **history 与 context 解耦** + context-window 指示器 + 压缩边界可视 | — | 0029 虚拟化(已做);加 context-window 指示 + compaction 分隔线(Plan 22) |

([progressive disclosure — NN/G](https://www.nngroup.com/articles/progressive-disclosure/)、[extended thinking — Claude](https://docs.claude.com/en/docs/build-with-claude/extended-thinking)、[reasoning — OpenAI](https://openai.com/index/learning-to-reason-with-llms/)、[Copilot per-hunk review — Warp/Copilot docs]、[Cline auto-approve](https://docs.cline.bot/features/auto-approve)、[Claude Code TodoWrite](https://platform.claude.com/docs/en/agent-sdk/todo-tracking)、[streaming UI patterns](https://thepromptbench.com/ai-product-ux/streaming-ui-patterns-that-dont-break/)、[JIT pacing CHI 2026](https://dl.acm.org/doi/10.1145/3772363.3798936))

---

## 6. opencode 在行业坐标里的位置

opencode 是**中段、扎实**的实现:数据模型(parts + ToolRegistry + ToolState)对齐主干;渲染(PacedMarkdown + remend + morphdom)= 你的北极星;但**审批走 dock 阻塞**(vs Zed 行内)、**sub-agent 走 navigate**(vs nested)、**diff 不支持 per-hunk accept**(纯展示)。它的 SKIP_PARTS(patch/step 不渲染)是务实选择。**你已经深读了它(见 [opencode 调研](./opencode-desktop-part-rendering-and-interaction.md)),本文是把它和 Zed/Vercel/OpenHands 并置看分歧。**

---

## 7. 对本项目 plan 的修正/补充建议

1. **确认 Zed 为技术对标**:SDF primitive / glyph atlas / List 虚拟化你都在做;**可借 SumTree 高度索引**替代 SpatialGrid 每帧重建(接 Plan 19 / TODO2 C)。
2. **Plan 23 补 fallback tool renderer**(GenericTool)+ 确认两级注册表 = 业界主干。
3. **Plan 23 R3 加 per-hunk diff**(若做可写 agent;Copilot 已落地、用户预期)。
4. **0031 借 source≠role 解耦 + 两类错误区分**(OpenHands);`Blocked` 借 Agent Inbox 四动作 schema。
5. **reasoning auto-collapse + raw CoT 显隐配置**(Plan 23 R1):对齐 Anthropic 折中路线。
6. **审批路线二选一**:行内下拉(Zed,轻)vs 阻塞 dock(opencode,你 0031 已选 dock)——确认 dock,但 read-only 类可不阻塞(graduated autonomy)。
7. **加 context-window 指示 + 压缩边界可视**(规模透明,用户预期)。

---

## 8. 参考(分组)

**GPU/native(技术最近邻)**:[Zed: Rust+GPU 120fps](https://zed.dev/blog/videogame) · [Zed Rendering Pipeline — DeepWiki](https://deepwiki.com/zed-industries/zed/2.2-rendering-pipeline-and-layout) · [Zed Agent Panel — DeepWiki](https://deepwiki.com/zed-industries/zed/8.1-agent-panel-and-ui) · [gpui README](https://github.com/zed-industries/zed/blob/main/crates/gpui/README.md) · [Zed Edit Prediction](https://zed.dev/docs/ai/edit-prediction)
**TUI**:[Claude Code Fullscreen](https://code.claude.com/docs/en/fullscreen) · [Crush Message Rendering — DeepWiki](https://deepwiki.com/charmbracelet/crush/5.3-message-rendering) · [glamour](https://github.com/charmbracelet/glamour) · [opentui](https://github.com/anomalyco/opentui) · [Aider edit-formats](https://aider.chat/docs/more/edit-formats.html)
**数据模型/SDK**:[Vercel UIMessage](https://ai-sdk.dev/docs/reference/ai-sdk-core/ui-message) · [Vercel tool usage](https://ai-sdk.dev/docs/ai-sdk-ui/chatbot-tool-usage) · [assistant-ui Tools](https://www.assistant-ui.com/docs/guides/tools) · [assistant-ui Reasoning](https://www.assistant-ui.com/docs/ui/reasoning) · [CopilotKit tool-rendering](https://docs.copilotkit.ai/generative-ui/tool-rendering) · [LangGraph Interrupts](https://docs.langchain.com/oss/python/langgraph/interrupts) · [agent-inbox schema](https://github.com/langchain-ai/agent-inbox)
**事件平台**:[OpenHands Events](https://docs.openhands.dev/sdk/arch/events) · [OpenHands 论文](https://arxiv.org/pdf/2407.16741) · [LangGraph Streaming](https://docs.langchain.com/oss/python/langgraph/streaming) · [goose 2.0 ACP](https://goose-docs.ai/blog/2026/04/08/goose-acp-and-new-tui/) · [SWE-agent ACI 论文](https://arxiv.org/pdf/2405.15793)
**IDE 扩展**:[Cline auto-approve](https://docs.cline.bot/features/auto-approve) · [Cline checkpoints](https://docs.cline.bot/core-workflows/checkpoints) · [Cline plan/act](https://docs.cline.bot/core-workflows/plan-and-act) · [Roo modes](https://docs.roocode.com/basic-usage/using-modes) · [Continue agent](https://docs.continue.dev/ide-extensions/agent/how-it-works)
**设计模式**:[progressive disclosure — NN/G](https://www.nngroup.com/articles/progressive-disclosure/) · [extended thinking — Claude](https://docs.claude.com/en/docs/build-with-claude/extended-thinking) · [reasoning — OpenAI](https://openai.com/index/learning-to-reason-with-llms/) · [streaming UI patterns](https://thepromptbench.com/ai-product-ux/streaming-ui-patterns-that-dont-break/) · [JIT token pacing — CHI 2026](https://dl.acm.org/doi/10.1145/3772363.3798936) · [Claude Code todo](https://platform.claude.com/docs/en/agent-sdk/todo-tracking)

> 可信度:Zed/Vercel/assistant-ui/CopilotKit/LangGraph/OpenHands/Cline/Claude Code 为官方文档或源码(高);DeepWiki 为源码解读(二手但带行号);设计量化数据(如 Copilot +28%)与部分 CoT 隐藏细节为二手,已在正文标注。
