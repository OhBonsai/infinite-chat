# 研究:agent TUI 的事件模型 · 消息渲染 · 行距/间距布局 —— claude-code / opencode / 业界 + cosmic-text 评估

- 日期:2026-07-18
- 类型:research(本地源码一手 + 业界 Web + 依赖评估)
- 触发:作者判「字的间距/行距表现非常差」,想在 **Rust 侧**实现 TUI 布局(行距/间距),问是否引 cosmic-text;要求先识别 claude-code + opencode TUI 的事件/结构/渲染,再定方案。
- 一手源码:`~/w/agentscode/claude-code`(转译 TS 源 + sourcemap)· `~/w/agentscode/opencode/packages/tui`(TS)· `~/w/agentscode/codex/codex-rs`(Rust 一手)。
- 与 plan45 关系:本篇是 plan45 U0 参考真值的**深化**(事件+布局层),U2/U3 的布局收敛直接用本篇结论。

---

## 0. 一句话结论(先给方案)

**Rust 侧做等宽 cell 布局器,不引 cosmic-text。** 业界铁证:**没有一个主流 agent TUI 在布局层用 shaping**——codex 用 `unicode-width`、Ink/gemini/claude-code 用 Yoga cell 计数、lipgloss 用 rune width。行高=1 cell、宽度=EAW 查表,是终端生态的既定契约。cosmic-text 是像素级 shaping 引擎(输出浮点 advance),用它做 TUI 网格既大材小用、又要再补一层 cell-snap 胶水、还把确定性绑死在字体二进制上。本引擎的「行距/间距差」是**间距规则缺失**(没有 TUI 那套 margin/缩进/gap 规则),不是缺 shaping 库——补规则即可,归 plan45 布局层。

---

## 1. 三家事件模型对照(大模型事件 → 结构)

### claude-code(Anthropic SDK 原生流)
- 源:`@anthropic-ai/sdk` BetaMessageStream。事件全集(`services/api/claude.ts:1979` switch):`message_start` / `content_block_start` / `content_block_delta` / `content_block_stop` / `message_delta` / `message_stop`。
- block 子类型(`content_block.type`):`text` / `thinking` / `redacted_thinking` / `tool_use` / `server_tool_use` / `advisor_tool_result` / `connector_text`。
- delta 子类型(`delta.type`):`text_delta`(拼 text)/ `input_json_delta`(拼 tool 输入 JSON 字符串)/ `thinking_delta` / `signature_delta` / `citations_delta`(忽略)。
- **增量法**:字符串 `+=` 拼接;`content_block_stop` 时 normalize 成 `AssistantMessage` yield;`message_delta` **回填**最终 usage/stop_reason 到已发消息(直接 mutation)。
- tool 配对:`buildMessageLookups`(`utils/messages.ts:1151`)建 `toolResultByToolUseID` / `resolvedToolUseIDs` / `erroredToolUseIDs` 三表,渲染时判 spinner/绿/红。

### opencode(两套并存,新旧过渡)
- **旧/主用**(TUI 实际渲染):扁平 `message[sessionID]` + `part[messageID]` 分离;事件 `message.updated` / `message.part.updated` / `message.part.delta` / `permission.*` / `question.*` / `session.status`。`message.part.delta` = `{sessionID,messageID,partID,field,delta}` 通用增量,`part[field] += delta`(`context/sync.tsx:388`)。二分查找维护有序数组 + upsert;**消息硬上限 100 条**,超出 shift 最旧(sync.tsx:317)。
- **新/细粒度**(`session.next.*`,`context/data.tsx`):`text.started/delta/ended` · `reasoning.*` · `tool.input.started/delta/ended` · `tool.called/progress/success/failed` · `step.*` · `compaction.*`——每种 part 独立 begin/delta/end,tool state `pending→running→completed/error`。这是"细粒度流式"的标准写法(与 codex 同构)。
- 事件类型权威定义:`packages/sdk/js/src/v2/gen/types.gen.ts`(`Event` union ~4351,`SessionStatus` 705)。

### codex(Rust,最完整的事件设计参考)
- `codex-rs/protocol/src/protocol.rs` `EventMsg` ~90 变体:**delta 与整块并存**——`AgentMessageContentDelta`/`ReasoningContentDelta`/`ExecCommandOutputDelta` 流式,`AgentMessage`/`AgentReasoning` 完成时整块确认(`HasLegacyEvent` 做 v1/v2 兼容)。工具成对 `ExecCommandBegin/End`、`McpToolCallBegin/End`、`PatchApplyBegin/Updated/End`;审批 `ExecApprovalRequest`/`ApplyPatchApprovalRequest`/`RequestUserInput`;条目 `ItemStarted/Completed`;上下文 `ContextCompacted`/`TurnDiff`。
- **UI 是纯事件消费者**,渲染层与 agent loop 完全解耦(TUI 与 IDE app-server 共用同一 EventMsg 流)。

**三家共识**:流式 = delta 增量拼接;终态 = 整块确认;tool = begin/end 成对 + 三态 + 结果配对表。**本引擎已全对齐**(store 三表 + delta 对账 + fsm 三态)——事件层无需改,plan45 只碰观感。

## 2. 消息渲染映射(内容类型 → 组件/形态)

| 内容 | claude-code | opencode |
|---|---|---|
| assistant text | `AssistantTextMessage`,`⏺` 前缀 + 2 列缩进,右列 Markdown | `TextPart` → `<markdown streaming tableOptions={style:grid}>`,`paddingLeft=3` |
| thinking/reasoning | `AssistantThinkingMessage` | `ReasoningPart`:折叠单行标题 `+/- Thought:… · duration`,展开 `<code filetype=markdown>` dim |
| tool_use | `AssistantToolUseMessage`:状态点(`ToolUseLoader` 闪烁 spinner,完成绿/错误红/进行暗)+ 工具名 + `(参数摘要)` | `ToolPart`→`toolDisplay` 12 特化 + Generic;**两形态**:InlineTool(单行 icon+摘要)/ BlockTool(`┃` 左边框块) |
| tool_result | `UserToolResultMessage`,前缀 `⎿`(2空格+⎿+2空格 dim) | 各 tool 块内;输出折叠 `collapseToolOutput(maxLines,maxChars)` |
| diff | `StructuredDiff`→**Rust/NAPI ColorDiff** 出 ANSI,gutter(marker+行号)+词级高亮 | 原生 `<diff view=split/unified showLineNumbers>`,>120 列 split |
| markdown | `Markdown.tsx`:`marked.lexer`→`formatToken`→ANSI→`<Ansi>`;表格走 flexbox `<MarkdownTable>`;highlight.js;块间 `gap=1` | 原生 `<markdown>` renderable;语法高亮 = tree-sitter WASM + nvim-treesitter 查询 |
| 折叠 | `collapsed_read_search` 组 + `OffscreenFreeze` | Read 多次折叠;点击展开 |

**tool 两形态是关键差异点**:opencode InlineTool(单行 `$`/`→`/`←`/`✱` icon + 摘要,连续零间距贴合)vs BlockTool(左竖线块 + 输出)。本引擎现只有块卡 → plan45 可补 **InlineTool 单行形态**(这就是"结构下的扩展组件")。

## 3. 布局系统:行距/间距的精确来源(本篇核心)

**三家都用 flexbox/cell 模型,单位 = 终端行×列(整数)。没有 baseline/行高概念——行高恒为 1 cell,"行距" = 空行。**

### claude-code(Ink + 真 Yoga）
- 消息间空行:`marginTop={addMargin?1:0}`(`AssistantTextMessage.tsx:258` / `AssistantToolUseMessage.tsx:285`);`addMargin` 由是否续接上条同源决定。
- 缩进:`⏺`/状态点 `minWidth={2}`(1列圆点+1列空=2列缩进);tool 结果 `⎿` = `"  "+⎿+"  "`(`MessageResponse.tsx:22`,2+1+2)。
- markdown 块间 `gap={1}`;wrap 用 ink `wrap-text` + `get-east-asian-width`(CJK 双宽)。

### opencode(@opentui + Yoga）
- 聊天列 `paddingLeft=2 paddingRight=2 paddingBottom=1 gap=1`(`session/index.tsx:1167`)——**左右各 2 列、子块间 1 行**。
- user 卡:`border=["left"]`(`┃` SplitBorder)`marginTop={index===0?0:1}` 内层 `paddingTop=1 paddingBottom=1 paddingLeft=2`。
- assistant text/reasoning:`paddingLeft=3 marginTop=1`(**统一缩进 3 列**)。
- **动态兄弟间距** `setPreLayoutSiblingMargin`(`util/layout.ts:8`,Yoga 布局前跑):按前一兄弟节点类型决定 `marginTop`——连续 inline 工具行**零间距贴合**,遇 text/block/user 边界才插 1 行。这是最精妙的间距规则,本引擎值得抄。
- BlockTool:`border=["left"] paddingTop=1 paddingBottom=1 paddingLeft=2 marginTop=1 gap=1`。
- 内容宽度 `dimensions.width - (sidebar?42:0) - 4`;`wide = width>120`。
- **无 CJK 双宽处理**(应用层):`Locale.truncate` 按码元 `str.length`,宽度测量全交原生 cell buffer。

### codex(ratatui,纯 cell)
- 一切以 cell 计;缩进用固定前缀列 `LIVE_PREFIX_COLS=2`(`ui_consts.rs`);垂直间距 = 空 `Line`,无 margin 抽象。
- `HistoryCell::desired_height(width)` 用 `Paragraph::wrap{trim:false}.line_count(width)` 量真实行数。
- wrap:`live_wrap.rs` `RowBuilder` 增量、基于 `unicode-width` 计 cell;`wrapping.rs` 保 URL 完整。

**间距规则清单(可直接抄进 plan45 tui flavor)**:
1. 聊天列左右 padding 2 列、块间 gap 1 行、底 padding 1 行。
2. user 消息 = 左竖线 `┃` 卡 + 内 padding(上下 1 / 左 2),首条无上边距其余 marginTop 1。
3. assistant 内容统一缩进(opencode 3 列 / claude-code 2 列——tui flavor 取参考主 = opencode 3 列)。
4. assistant text 前缀 `⏺`(2 列)、tool 结果前缀 `⎿`。
5. **动态兄弟间距**:连续同类(inline tool)贴合零 margin,跨类型边界插 1 行。
6. 块间距全整数 cell,禁小数/亚像素。

## 4. 流式渲染机制(delta → 重渲)

- **共同模式(2025-26 收敛)**:稳定区进 scrollback + 尾部小活动区重渲。
  - claude-code:`<Static>` 静态区 + 流式预览块单独渲;`StreamingMarkdown` 在最后顶层块边界切 `stablePrefix`(memo 不再 parse)+ `unstableSuffix`(只 re-lex 增长块);Ink 16ms render 节流批 delta;逐行显示(只显已完成整行)。
  - codex:`streaming/controller.rs` stable/tail 两区,`emitted_stable_len ≤ enqueued ≤ rendered`;**表格 holdback**(pipe-table 从表头整段扣在 tail 到流末,因列宽会变);`AdaptiveChunkingPolicy` Smooth/CatchUp 双档带滞回(打字机+追赶);resize 整体重渲不做字节级 remap。
  - opencode:`message.part.delta` 只对目标 part 字段 `+=`,SolidJS 细粒度响应,`<markdown streaming>` 原生增量重排。
  - aider:`rich.Live` 尾部窗口 + 稳定行 print 进 stdout(同构更粗)。
- **本引擎对照**:骨架先行 + reveal 调度 + 块冻结 = 同一思想的更精细版(逐字 vs 逐行);codex 的"表格 holdback"= 本引擎"表格网格先行"的对偶;**节奏系统(reader/typewriter)对应 codex AdaptiveChunkingPolicy**。事件+流式层本引擎全覆盖,无需借。

## 5. cosmic-text 评估 + Rust 侧 TUI 布局三路线

**cosmic-text 是什么**:pop-os 纯 Rust 多行文本(0.19 shaping 已从 rustybuzz 换 **HarfRust**;raster=swash optional;字体发现 fontdb;layout 自研含 bidi)。`Metrics::new(font_size, line_height)` **line_height 显式可控**;`Attrs.letter_spacing_opt`(EM,0.19+)。wasm:no_std shaping/layout ✓、font loading/rendering ✗;wasm32 无系统字体,只能 `load_font_data` 喂字节。

**错配**:输出是像素级 `LayoutGlyph`(浮点 advance),**无 cell 概念**;fallback 命中非等宽字体 → 网格歪,要自己 snap;确定性绑死字体二进制(跨版本 harfrust/字体换版会挪 glyph)。

**三路线对比**:

| 维度 | ① 自研 cell 布局器(Rust) | ② cosmic-text | ③ JS measureText + mono 规则 |
|---|---|---|---|
| 单位 | 整数 cell,行高=cell 恒定 | 浮点像素,需 snap | 像素,cell_width 启动校准一次 |
| 宽度来源 | `unicode-width`(EAW 表) | 字体 advance | 浏览器度量 |
| line_height/letter_spacing | 定义即所得(cell 常数) | Metrics 显式 + Attrs letter_spacing | CSS/canvas 属性 |
| CJK 双宽 | EAW 表 2 cell,确定 | 字体说了算,fallback 可破 | 字体+浏览器 |
| wasm 体积 | ~几十 KB(unicode 表) | 代码 ~0.5–1MB + **字体必嵌**(Latin 0.1–0.3MB / CJK 8–25MB,本地 LXGWWenKaiMono 实测 25.6MB) | 0(字体走浏览器) |
| **确定性(R8)** | **最强**:整数表驱动、零字体依赖、可 golden | 条件确定(锁版本+锁字体+禁系统扫描);跨版本不稳 | 最弱;mono 规则收敛到单常数 |
| 工作量 | 小(advance 恒定+EAW 表+换行器;codex `live_wrap.rs`/`wrapping.rs` 可抄) | 中(集成+snap 胶水+字体管线) | 最小(改量宽规则) |
| 与引擎接口 | 输出 cell 网格,glyph 源自由 | layout/渲染耦合同字体 | 布局在 JS,Rust 拿不到,跨界往返 |
| 隐性风险 | 布局宽≠渲染宽(渲染端须 advance=cell,glyph 居中/裁进 cell) | fallback 破网格;"No wrapping/Ellipsize" 未完成 | 布局结果不进 Rust,快照测难做 |

**竞品**:swash 直用=自己重写 cosmic-text 中层(不值);glyphon=cosmic-text+wgpu(你已有渲染层,仅参考);**业界佐证:全体 agent TUI 布局层用表驱动 cell,shaping 只在终端模拟器渲染端**。

## 6. 给本引擎的落地结论(接 plan45)

**A. 布局层(解决"行距/间距差")= 自研 cell 布局器路线①,但轻量落法**:
- 本引擎已有 mono 路(glyph advance)+ JS measureText 校准 → **不必推翻**:tui flavor 下把布局宽度规则从"逐字实测"改为 **cell 网格**(`advance = 半宽 cell 常数;CJK/宽字符 = 2 cell`,EAW 表由现有 CJK 判定扩),行高 = `cell × 固定行距系数`(TUI 无行距→系数 1.0~1.2,对拍定)。R8 确定性最强,不引任何依赖。
- 间距规则(§3 清单 6 条)进 tui flavor 的**装饰/布局参数**:聊天列 padding 2、块 gap 1、assistant 缩进 3、user `┃` 卡、`⏺`/`⎿` 前缀、**动态兄弟间距**(连续 inline tool 贴合)。这些是数据(theme/装饰 token + 布局常量),归 plan45 U2/U3,机制零改。
- **不引 cosmic-text**:体积税(字体必嵌 0.1–25MB)+ 确定性倒退 + cell-snap 胶水,三重不划算;它擅长的 bidi/复杂文种 TUI 用不上。

**B. 组件层补充(结构下的扩展组件,非改结构)**:
- **InlineTool 单行形态**(opencode 关键差异):tool 卡除现有块形态,补一个单行 `icon + 摘要` 渲染器(0033 registry 加条目);连续 inline 工具零间距贴合靠动态兄弟间距。
- reasoning 折叠单行标题态、tool 结果 `⎿` 前缀行——都是 0037 装饰 + 渲染器条目。

**C. 与 plan45 的并入**:本篇的间距清单 + cell 布局规则 + InlineTool 形态 = plan45 U2/U3 的**布局收敛真值**;plan45 §1 DoD-3(渲染器集)增补一条 InlineTool,U3 对拍把 §3 六条间距规则逐条数值化对照。**不新开 plan**,并入 plan45 执行(progress 记一行"布局层结论来自本 research")。

---

参考(一手源码):claude-code `services/api/claude.ts` / `components/Message.tsx` / `Markdown.tsx` / `ink/styles.ts` · opencode `packages/tui/src/{context/sync.tsx,routes/session/index.tsx,util/layout.ts,ui/border.ts}` + `packages/sdk/js/src/v2/gen/types.gen.ts` · codex `codex-rs/{protocol/src/protocol.rs,tui/src/{streaming/controller.rs,markdown_stream.rs,live_wrap.rs,ui_consts.rs}}`。
业界:[ratatui layout](https://ratatui.rs/concepts/layout/) · [ink](https://github.com/vadimdemedes/ink) · [lipgloss](https://github.com/charmbracelet/lipgloss) · [glamour v2](https://github.com/charmbracelet/glamour) · [bubbletea MVU](https://github.com/charmbracelet/bubbletea) · [gemini-cli 架构](https://deepwiki.com/google-gemini/gemini-cli/1.1-architecture-overview)。
cosmic-text:[repo](https://github.com/pop-os/cosmic-text) · [Attrs 0.19](https://docs.rs/cosmic-text/latest/cosmic_text/struct.Attrs.html) · [egui#3378 wasm/no_std 确认](https://github.com/emilk/egui/issues/3378) · [UAX#11 EAW](http://www.unicode.org/reports/tr11/) · [wcwidth 实践](https://github.com/jquast/wcwidth)。
