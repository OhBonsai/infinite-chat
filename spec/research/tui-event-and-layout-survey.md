# 研究:Agent TUI 的事件模型与布局系统 —— claude-code / opencode / codex / 业界 + cosmic-text 评估

- 日期:2026-07-18
- 类型:research(三路并行源码级调研汇总;本地一手:`~/w/agentscode/{claude-code, opencode, codex}`)
- 触发:引擎 TUI flavor 下字间距/行距表现差(plan45 U3 遗留的深层原因);作者要求:①识别两参考项目的大模型事件/结构/渲染;②TUI 界面与事件设计业界全景;③评估 Rust 侧实现 TUI 布局(含 cosmic-text)。
- 本地源码可信度:claude-code = 转译 TS 源码树(带 sourcemap 原文,`claude-code-rebuild`);opencode = packages/tui TS 源;codex = codex-rs Rust 源。全部 file:行号可复核。

---

## 0. TL;DR(六条)

1. **事件模型三流派**:claude-code = SDK 原生流(`content_block_delta` 按 index 定位 + 应用层扩展);opencode = **双模型过渡**(旧 `message.part.delta{field,delta}` 通用字符串增量【我们已对接的就是这套】+ 新 `session.next.*` 细粒度 40+ 事件);codex = **SQ/EQ 协议 crate**(~90 个 `EventMsg` 变体,delta 流式 + 整块终态确认并存)。共同点:**delta 累积 + 终态整块对账**——与本引擎 AR4 同构,无需改协议层。
2. **布局全体系 = 整数 cell,零 shaping**:Ink/Yoga(claude-code、gemini-cli)、@opentui/Yoga(opencode)、ratatui Constraint(codex)、lipgloss(crush)——margin/padding/gap 全是"行×列"整数;**没有任何主流 agent TUI 在布局层做字体 shaping**,宽度全部查表(unicode-width/EAW)。
3. **行距的真相**:终端世界**行高恒等于 1 cell,"行距"只能用空行表达**;间距规则是精确的小整数(见 §4 规范表)。我们 TUI flavor 排版差的病根:仍走比例字体 measureText + 1.4× 行高常数 + 无 cell 语义——不是调参问题,是**布局模型不对**。
4. **流式渲染共同收敛**:「稳定区进 scrollback + 尾部小活动区重渲」(codex escape 注入 / Ink `<Static>` / aider rich.Live);codex 的 `AdaptiveChunkingPolicy`(Smooth 每拍一行 / CatchUp 倾泻,滞回防抖)是打字机节奏的完整参考——**我们的 rhythm 五层比它强,保留即可**。
5. **cosmic-text 结论:不引**(本轮)。它解决的是"比例文本 shaping+bidi"——TUI 等宽网格用它是大材小用且还缺 cell 化胶水;wasm 字体税 8–25MB(CJK);跨版本输出不字节稳定。**推荐:自研等宽 cell 布局器**(整数域/EAW 查表/~几十 KB/确定性最强,layout seam 本就可替换)。cosmic-text 记为"未来整个 layout 搬进 Rust"的远期选项。
6. 三参考的**间距数值已抄齐**(§4),可直接做 TUI flavor 的布局 token 表——落地建议见 §6(→ plan46)。

## 1. 事件模型对比

### 1.1 claude-code(SDK 流 + 应用扩展)

链路:`@anthropic-ai/sdk` BetaMessageStream → `services/api/claude.ts:1979` switch 消费 → yield 完整 `AssistantMessage`(content_block_stop 时)+ 透传 `{type:'stream_event', event}` → UI 中枢 `utils/messages.ts:2930 handleMessageFromStream`。

| 事件 | 结构要点 |
|---|---|
| message_start | 初始 message + usage;记 ttftMs(claude.ts:1980) |
| content_block_start | `{index, content_block}`;子类型 tool_use/server_tool_use/text/thinking/redacted_thinking(:1996) |
| content_block_delta | `{index, delta}`;delta 子类型 text_delta/input_json_delta(拼 JSON 字符串)/thinking_delta/signature_delta(:2083) |
| content_block_stop | normalize 单 block → yield AssistantMessage(:2171) |
| message_delta | 终态 usage/stop_reason **回填 mutation**(:2213) |

应用层扩展消息:`tombstone/progress/attachment/system/tool_use_summary/stream_request_start`(QueryEngine.ts:758)。tool 配对靠 `buildMessageLookups`(messages.ts:1151):`toolResultByToolUseID`/`resolvedToolUseIDs`/`erroredToolUseIDs` 三索引驱动 spinner/颜色。

### 1.2 opencode(双模型)

- **旧模型(TUI 实际渲染用,= 本引擎已对接)**:`message.updated` / `message.part.updated` / **`message.part.delta{sessionID,messageID,partID,field,delta}`**(通用字符串字段增量,sdk types.gen.ts:4839)。store = `context/sync.tsx` 单 switch(:162):二分 upsert + `part[field] += delta`(:385);**消息硬上限 100 条**(:317)。
- **新模型 `session.next.*`(40+ 细粒度)**:`text.started/delta/ended`、`reasoning.*`、`tool.input.started/delta/ended`、`tool.called/progress/success/failed`、`step.*`、`compaction.*`(types.gen.ts:4416-4838)——**tool 生命周期显式化**(pending→running→completed 状态迁移,data.tsx:307-352)。这是 opencode 的演进方向,值得跟踪(我们 fsm 的 tool 状态与此同构)。
- 周边:permission/question(v1+v2 双版)、session.status{idle/busy/retry}、session.error(8 类 union)、todo/lsp/file.watcher/pty…(§素材全集见 types.gen.ts 4351-5236)。

### 1.3 codex(协议 crate,Rust)

`codex-rs/protocol/src/protocol.rs` `enum EventMsg` ~90 变体:生命周期(TurnStarted/Complete/Aborted/TokenCount)+ **delta 与整块并存**(`AgentMessageContentDelta` 流式 → `AgentMessage` 终态;Reasoning/Plan 同构)+ 工具 Begin/End 成对(Exec/McpToolCall/WebSearch/PatchApply…)+ 审批(ExecApprovalRequest/RequestUserInput)+ `ItemStarted/Completed`。TUI 是纯事件消费者(SQ/EQ 解耦)。

### 1.4 对照结论(对本引擎)

三家收敛于同一形状:**流式 delta(定位:index / partID+field / item)→ 累积 → 终态整块确认**。本引擎 protocol/store(AR4 delta+updated 对账)已同构于 opencode 旧模型;若未来对接 claude-code/codex,只是 protocol 层加解码变体,store/fsm 无需改(0031 已预留)。**opencode `session.next.*` 是最值得预研的演进**(tool 生命周期显式事件 ≈ 我们 fsm 内部迁移的外化)。

## 2. 渲染映射(消息 → 组件)

| 内容 | claude-code(Ink/React) | opencode(@opentui/Solid) |
|---|---|---|
| 分派点 | `components/Message.tsx` switch | `session/index.tsx:1572 PART_MAPPING` + `<Dynamic>` |
| text | AssistantTextMessage:`⏺`(minWidth=2)+ Markdown | TextPart:`<markdown streaming tableOptions=grid conceal>` paddingLeft=3 |
| thinking | AssistantThinkingMessage | ReasoningPart:折叠标题 `+/- Thought: …·duration` / `<code filetype=md streaming>` 降饱和 |
| tool_use | AssistantToolUseMessage:`ToolUseLoader` 闪烁 ⏺(未决闪/成绿/错红)+ 名(bold,truncate-end)+ (参数) + 进度行;每 Tool 自带 render* 方法族(Tool.ts) | ToolPart 双形态:**InlineTool**(单行:icon 2 列 + 摘要;icon 表:bash`$` read`→` write/edit`←` grep`✱` task`│→✓` 等)/ **BlockTool**(┃ 左边框块:Shell 输出/diff/todo);completed 且 showDetails=false 整块隐藏 |
| tool_result | UserToolResultMessage;前缀 `⎿`(2 空格+⎿+2 空格,MessageResponse.tsx:22);Read/Search 多次**折叠成组**(CollapsedReadSearchContent + OffscreenFreeze) | 输出折叠 collapseToolOutput(Shell 10 行/Generic 3 行)+ "Click to expand" |
| diff | Rust/NAPI ColorDiff → ANSI 行 → RawAnsi 双列(gutter=marker+右对齐行号+2 空格);词级 diffAddedWord/RemovedWord | 原生 `<diff view=split|unified(宽>120 切