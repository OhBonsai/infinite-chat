# opencode 桌面端 part 渲染与 agent 交互调研(+ 转化到本引擎)

- 日期:2026-06-24
- 状态:研究 / 设计依据(供 Plan 22 修正 + 渲染落地方案)
- 范围:opencode monorepo(`/Users/wp/w/agentscode/opencode`)桌面/Web UI(SolidJS,`packages/app` + `packages/ui`)对 reasoning/tool/file/patch/step/compaction 等 part 的**数据模型 / 渲染 / agent 交互**设计,及如何转化到本项目 GPU/SDF 渲染引擎。
- 方法:3 路并行源码精读(渲染 / 状态 / 交互),引用均带 `file:行号`。

---

## 0. TL;DR(对本项目最重要的 5 条)

1. **验证你的北极星(两条)**:opencode 桌面端**独立实现了和你几乎一样的机制**——① **节奏化逐字揭示** `PacedMarkdown`(24ms 批次 + 按词/标点边界 snap,`message-part.tsx:193-209`),= 你 0017/0019 的 reveal cadence;② **remend 修补未闭合 markdown** + `morphdom`/`isEqualNode` 跳过未变节点(结构块不重渲染/不闪,`markdown.tsx`/`markdown-stream.ts`),= 你的"块冻结 + remend"。**第一方实现印证你的核心设计。**
2. **修正 Plan 22(两条)**:reducer 的 **`SKIP_PARTS = {patch, step-start, step-finish}`**——这三类 part **不存、不渲染**(`event-reducer.ts:228-254`);**diff 不来自独立 patch part**,而是挂在 **tool part 的 `state.metadata.filediff`**(edit/write/apply_patch)。→ Plan 22 §3"step-finish→footer、patch→独立 diff 块"要改。
3. **两级渲染注册表**:`PART_MAPPING[part.type]` → tool 再走 `ToolRegistry[part.tool]`(缺省 `GenericTool`)。**数据驱动、按工具名分派**——和你"效果是数据、插件=注册表项"(0006/0002)同构,值得照搬成你的渲染分派结构。
4. **context 工具分组**:连续的 `read/glob/grep/list` 被折叠成一个"Gathering/Gathered context"卡(`groupParts`,`message-part.tsx:559`)——**降噪关键 IA**,直接采纳。
5. **交互全靠"坞"(Dock)+ 阻塞 turn**:permission/question 是叠在输入框上方的 DOM Dock,**会阻塞 turn**(服务端 `Deferred.await`);sub-agent(task 工具)**不嵌套、是跳转**。→ 你的交互层 = 0022 DOM overlay + 0031 `Blocked` FSM 态。

---

## 1. 数据模型(状态侧)

### 1.1 Part 联合(`packages/sdk/js/src/v2/gen/types.gen.ts:376-637`)

```
Part = TextPart | ReasoningPart | FilePart | ToolPart | SubtaskPart
     | StepStartPart | StepFinishPart | SnapshotPart | PatchPart
     | AgentPart | RetryPart | CompactionPart
```

共有 `id/sessionID/messageID/type`。关键变体字段:
- **TextPart** `text, synthetic?, ignored?, time{start,end?}`(`synthetic` 把"评论"编码进 part)。
- **ReasoningPart** `text, time{start,end?}`。
- **FilePart** `mime, filename?, url, source?`(image/file)。
- **ToolPart** `callID, tool, state: ToolState, metadata?`(**核心,见 1.2**)。
- **StepFinishPart** `reason, cost, tokens{input,output,reasoning,cache{read,write}}` —— **reducer SKIP,不进 store**;token/cost 聚合在 `AssistantMessage.cost/tokens`。
- **PatchPart** `hash, files[]` —— **reducer SKIP**。
- **CompactionPart** `auto, overflow?, tail_start_id?`。
- **RetryPart** `attempt, error, time{created}`;**AgentPart**/**SubtaskPart**(task)。

### 1.2 ToolState 状态机(`types.gen.ts:475-530`)

```
pending → running → completed | error
```

| status | 携带 |
|---|---|
| `pending` | `input{}`, `raw`(未解析流式 JSON) |
| `running` | `input{}`, `title?`, `metadata?`, `time{start}` |
| `completed` | `input{}`, `output:string`, `title`, `metadata`, `time{start,end}`, `attachments?:FilePart[]` |
| `error` | `input{}`, `error:string`, `metadata?`, `time{start,end}` |

**字段随状态收窄/扩展**:`title/time` 在 running 才有,`output/attachments` 仅 completed,`error` 串仅 error。**done 判定靠 `time.end` 是否存在**,不存布尔。

### 1.3 流式累积 + 单一 reducer(`event-reducer.ts`)

- **唯一 reducer**(一个大 switch),对 SolidJS store 细粒度变更;数组用 `Binary.search` 有序 O(log n) upsert;写回用 `reconcile()` 最小 diff。
- **`message.part.delta`**:`{messageID,partID,field,delta}` → 同时 ① `part_text_accum_delta[partID] += delta`(累积缓冲)② part 对象 `[field] += delta`(通常 `field="text"`)。**文本/推理按字段字符串拼接,不替换整 part**。
- **`message.part.updated`**:整 part `reconcile` 替换 + **清空** accum 缓冲。
- **`session.status`** 联合 `{idle}|{busy}|{retry,attempt,message}`;**无独立 `session.idle`/`session.error` 事件**;错误承载在 `AssistantMessage.error`(由渲染层解读)。
- 乐观 temp 消息 + 分页拉取 `mergeOptimisticPage` 去重(`directory-sync.ts`);重连 → 目录重入队 → `session.messages` 拉取。

---

## 2. 各 part 渲染设计(`packages/ui/src/components/`)

> 调度:`PART_MAPPING[part.type]`(`message-part.tsx:1261`)→ `<Dynamic>`;tool 再走 `ToolRegistry[part.tool]`(`:1309`),缺省 `GenericTool`。

| part | 组件/设计 | 状态/折叠 |
|---|---|---|
| **text** | `PacedMarkdown`(流式)/`Markdown`(完成);marked+DOMPurify+morphdom;尾部 meta(`Agent·Model·时长·Interrupted`)+ 复制 | streaming = assistant 且 `time.completed` 未定;`interrupted = error.name==="MessageAbortedError"` |
| **reasoning** | 极简:同 PacedMarkdown,**无折叠/copy/label**,视觉暗化交 CSS;需 `showReasoningSummaries && text.trim()` | "Thinking" 状态行是 **turn 级合成**(非 part):无可见 part 时 `TextShimmer "Thinking"` + 从 reasoning 抽 heading |
| **tool(通用)** | `BasicTool`:`Collapsible`,trigger=图标+title(`TextShimmer active={pending}`)+subtitle+args,body=output | `pending=status∈{pending,running}` → title 微光、args/output 隐藏、**禁止展开**;`defer` 懒挂载(从底部 rAF 逐帧) |
| tool: **bash** | console 图标;`<pre>$ cmd\n\n输出`(stripAnsi);右上复制;`[data-scrollable]` | 默认折叠(设置 `shellToolPartsExpanded`) |
| tool: **edit** | code 图标;title=Edit+文件名;右侧 `DiffChanges(+x/-y)`;body=`fileComponent mode="diff"` + `DiagnosticsDisplay` | diff 源 = `state.metadata.filediff`(patch/before/after) |
| tool: **write** | 同 edit;`fileComponent mode="text"`(整文件) | |
| tool: **apply_patch** | `patchFiles(metadata.files)` 多文件 `Accordion`,每文件 FileIcon+变更标签;非删除默认展开 | |
| tool: **read/glob/grep/list** | **被 `groupParts` 折叠成 context 组**(见 2.1) | 默认收起 |
| tool: **task(subagent)** | task 图标;title=agent 名(tone 调色);running 显 `Spinner`;`(background)` 标记;**点击跳子 session** | `hideDetails` |
| tool: **webfetch/websearch** | 链接 + 跳转图标;websearch 抽 url 列表 | `hideDetails` |
| tool: **error** | `ToolErrorCard`(红 Card + Collapsible) | |
| **file** | 仅 user message;`attached`(data:)→`<img>`/FileIcon;`inline`→`HighlightedText` 区间高亮 | |
| **patch** | **无独立渲染**(SKIP);diff 全走 tool 的 fileComponent | |
| **step-start/finish** | **无渲染**(SKIP);无 token/cost 数字展示 | |
| **compaction** | `MessageDivider label="compaction"`(线—label—线) | |
| **todowrite** | Checkbox 只读清单 + "完成/总数";**但在时间线被 `HIDDEN_TOOLS` 隐藏**,只在 Dock 复用 | |

### 2.1 回合分组 + context 折叠(降噪 IA)

- 一个 turn = user message + 其下 `parentID` 指向它的所有 assistant message;`AssistantParts` 跨消息扁平合并 part(`message-part.tsx:628`)。
- **`CONTEXT_GROUP_TOOLS={read,glob,grep,list}` 连续段** → 折叠成一个 `type:"context"` 组(`groupParts`,`:559`)→ `ContextToolGroup`("Gathering/Gathered context" 卡 + 读/搜/列计数);其余 part 各自独立行。
- **turn 末 diff 摘要**:非 working 且有改动 → `session-turn-diffs`("N changed files" + `DiffChanges` + 每文件 Accordion,超 10 折叠)。
- **timeline 虚拟化**:`message-timeline.tsx` 用 virtua `Virtualizer` 把 turn 展平成 `TimelineRow` 序列(UserMessage / AssistantPart* / TurnDivider(compaction|interrupted) / Thinking / DiffSummary / Error),带 per-session cache + bottom-anchor 锁定。

---

## 3. agent 交互形态(`packages/app/src/pages/session/composer/`)

> 核心 = **prompt dock**:叠在输入框上方的一组"坞",由 `SessionComposerRegion` 编排;`blocked() = 有 permission 或 question 请求` → 隐藏普通输入框、只显示对应坞。

| 形态 | 触发(event) | UI 形态 | 用户选项 | 回复 | 阻塞 turn? |
|---|---|---|---|---|---|
| **Permission(工具授权)** | `permission.asked` | `DockPrompt kind="permission"`:warning + 工具描述 + `patterns` | **Deny / Allow always / Allow once**(`onDecide reject/always/once`) | `permission.respond(POST .../permission/:id/reply)` | **是**(服务端 `Deferred.await`) |
| **Question(反问)** | `question.asked` | `DockPrompt kind="question"`:多问卷分页,radio/checkbox/`multiple` + "Type your own" textarea;键盘可达 | 选项 / 自定义 / 拒答(Escape) | `question.reply(POST .../question/:id/reply)` / `.../reject` | **是** |
| **Abort/Stop** | — | 发送按钮 working 时变 stop | 点击/回车停止 | `session.abort(POST /session/:id/abort)`;本地排队则 cancel `AbortController` | 解除(pending Deferred 被 `fail`) |
| **Sub-agent(task)** | `session.created{parentID}` | tool 卡(非嵌套);running Spinner;`(background)` | **点击跳子 session**(`navigateToSession`) | — | 子会话内输入禁用,显"Back to parent" |
| **Todo dock** | `todo.updated` | 坞内只读清单(Checkbox readOnly,in_progress 脉冲);可折叠 | 仅展示 | — | 否(旁路) |
| **Followup/Revert dock** | — | running 时排队消息(Send now/Edit)/ 回滚列表(Restore) | 操作按钮 | — | 否 |

**关键**:permission/question **阻塞 turn**——服务端 `ask` 建 `Deferred` 并 await,工具挂起,直到 reply/reject;**autoAccept**(目录级规则命中)不进坞、即时 `respondOnce`、turn 不停;**子会话授权汇总到父级坞**(`session-request-tree`)。

---

## 4. 转化到本引擎(逐类 → 你的图元 + GPU/DOM 决策)

> 原则:**静态展示 → GPU(SDF 块,走 0029 tier 虚拟化);需交互(展开/点击/勾选)→ DOM overlay(0022)**。两者用同一相机同步(embed-overlay 机制)。所有非文本 part 都是**块**,纳入 PartView/BlockCache/tier。

| part | opencode 做法 | 本引擎落点 | GPU/DOM |
|---|---|---|---|
| **text** | PacedMarkdown + meta | 已有(markdown→StyledSpan);meta footer = 小号弱化文本 | GPU |
| **reasoning** | 极简暗化区 | **思考区**(0006 标签区,`Reasoning`/Dim 角色);"Thinking" 合成行 = turn 级状态指示 | GPU |
| **tool(卡)** | `BasicTool` 折叠卡 | **SDF 面板(0018)** 作卡底 + 状态徽章 + title(reveal 微光对应你的 spawn 动画)+ args/output;**折叠交互 → DOM overlay 命中或 GPU hit-test(0011 §3.3④)** | 卡底 GPU;**展开/折叠按钮 DOM overlay** |
| tool: **bash** | `<pre>` 输出 | 代码视口(0027)+ 复制按钮(已规划 Plan 21) | GPU 文本 + DOM 复制 |
| tool: **edit/write/apply_patch** | fileComponent diff + DiffChanges | **代码视口(0027)+ 增删行底色(rect)** + **DiffChanges 条形 = SDF 小图元(0018/widget)**;多文件 = 多块 | GPU |
| tool: **read/glob/grep/list** | context 折叠组 | **采纳 context 分组**:连续段折叠成一个"Gathered context"面板(0018)+ 计数 | GPU 卡 + DOM 展开 |
| tool: **task(subagent)** | 卡 + 跳转 | 卡(0018)+ **点击跳子会话**(无限画布可"缩放进子会话"或导航) | GPU 卡 + DOM/hit-test 点击 |
| **file**(附件) | img/FileIcon | 图片走 embed(0007 纹理 quad);文件 chip = SDF 面板 + 图标 | GPU(图片纹理) |
| **patch** | SKIP | **不渲染**(diff 挂 tool);移除 Plan 22 的独立 patch 块 | — |
| **step-start/finish** | SKIP | **不渲染**;若要 usage,聚合到 turn footer(opencode 也不显示 token/cost) | — |
| **compaction** | MessageDivider | **分隔线**(0026 widget rule + 标签"上下文已压缩") | GPU |
| **todowrite** | 隐藏(只在 Dock) | 时间线隐藏;Todo 清单走 **DOM Dock**(0022) | DOM |

**关键张力:tool 卡的折叠交互**。opencode 用 Solid `Collapsible`(纯 DOM)。你是 GPU——两条路:(a)**卡底/内容 GPU 画,折叠箭头 + 命中走 DOM overlay 透明热区**(复用 Plan 21 文本层的命中桥);(b)**全 GPU**:卡是块,折叠 = 高度动画(0016 morph)+ hit-test(0011 §3.3④)路由点击。建议 **(a) 起步**(交互交 DOM,视觉交 GPU),与 ADR 0030 的"DOM 管交互/GPU 管视觉"一致。

**虚拟化必须覆盖工具卡**:长会话里几十个 tool 卡/diff 都是块 → 走 0029 Hot/Warm tier,屏外释放重几何。**别让工具卡撑爆内存**(Plan 22 已注)。

---

## 5. 交互形态 → 你的架构

- **permission/question Dock = DOM overlay(0022)**:叠在输入框上方,**不在画布**;阻塞 turn → 对应 **0031 `Blocked{permission|question}` FSM 态**(本就规划了)。Deny/Allow once/Allow always 三选 + 自定义答案,REST reply 在 TS 侧(0031 边界:I/O 在 TS)。
- **abort**:发送按钮 working→stop;`session.abort` POST 在 TS;FSM → `Stopped`(0031 F11,冻结 + reconcile 跳过)。
- **sub-agent(task)= 跳转非嵌套**:你的无限画布有更好的选择——**可"缩放/平移进子会话"或并排展开**(对应 TODO2 B 画布产品化的分支/树视图);v1 先做"点击导航"(同 opencode),后期升级成空间嵌套。
- **autoAccept**:目录级自动批准规则 → TS 侧拦截即时 respond,不进 Dock、turn 不停。
- **todo/followup/revert Dock**:非阻塞旁路,DOM overlay。

---

## 6. 对 Plan 22 的修正建议

1. **step-start/step-finish/patch 列入 SKIP**:不存不渲染(对齐 opencode);Plan 22 §3 删"step-finish→footer、patch→独立 diff 块"。usage 若要展示,聚合到 turn footer(opencode 也不显示 token/cost,建议同样不显示)。
2. **diff 挂 tool part**:edit/write/apply_patch 的 diff 来自 `tool.state.metadata.filediff`,**不是** patch part。渲染 diff = tool 卡内嵌代码视口。
3. **采纳 context 工具分组**:read/glob/grep/list 连续段折叠成"Gathered context"面板 —— 降噪,Plan 22 §3.1 三桶分组里加这一层。
4. **采纳两级注册表**:`part.type → tool.tool` 二级分派(数据驱动),对应你 0002/0006"注册表项"。
5. **docks 阻塞 turn**:permission/question → 0031 `Blocked` 态 + DOM overlay,Plan 22 §4 已含,补"阻塞=服务端 Deferred await,UI 隐藏输入只显坞"。
6. **paced reveal/remend 已验证**:你的 reveal cadence + remend 不是孤例,opencode 第一方同款 → 0017/0019 信心 +1。
7. **tool 卡折叠**:按 ADR 0030"DOM 管交互/GPU 管视觉"——卡视觉 GPU、折叠热区 DOM overlay。

---

## 7. 参考(opencode 源码)

渲染:`packages/ui/src/components/message-part.tsx`(PART_MAPPING + ToolRegistry + 各 tool render)· `basic-tool.tsx`(通用卡 + defer)· `markdown.tsx`/`markdown-stream.ts`(marked+DOMPurify+morphdom + remend)· `session-turn.tsx` · `diff-changes.tsx`/`tool-error-card.tsx`/`tool-status-title.tsx`。
状态:`packages/app/src/context/global-sync/event-reducer.ts`(唯一 reducer,SKIP_PARTS)· `server-sync.tsx`(SSE 分流 + 重连)· `directory-sync.ts`(乐观/分页/去重/LRU)· `global-sync/types.ts`(State + part_text_accum_delta)· `packages/sdk/js/src/v2/gen/types.gen.ts`(Part 联合 / ToolState / SessionStatus)。
交互:`packages/app/src/pages/session/composer/{session-composer-region,session-composer-state,session-permission-dock,session-question-dock,session-todo-dock,session-followup-dock,session-request-tree}` · `context/permission.tsx` · `components/prompt-input.tsx` + `prompt-input/submit.ts` · `packages/server|opencode/src/{permission,question}`。
本项目接点:[0031](../decision/0031-event-fsm-resilience-and-js-rust-boundary.md) · [Plan 22](../plan/plan22-opencode-events-and-fsm.md) · [0006](../decision/0006-inline-tags-and-extensibility.md) · [0007](../decision/0007-rich-media-embeds.md) · [0018](../decision/0018-sdf-panel-decoration-primitive.md) · [0022](../decision/0022-dom-overlay-layer.md) · [0027](../decision/0027-code-block-viewport.md) · [0029](../decision/0029-session-virtualization-and-glyph-working-set.md) · [0030](../decision/0030-text-copy-selection-and-a11y-mirror.md)。

> 可信度:opencode 源码为第一方核实(高,带 file:行号);转化方案为本项目设计建议。
