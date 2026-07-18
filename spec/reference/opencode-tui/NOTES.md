# opencode TUI 参考真值注记(Plan 45 U0)

- 参考仓库:`~/w/agentscode/opencode/packages/tui/src/`;渲染引擎 = **@opentui/core + @opentui/solid**
  (SolidJS → 终端 cell buffer)。`<markdown>`/`<code>`/`<diff>`/`<text>`/`<box>` 是 opentui-core renderable,
  不在本仓 —— 样式真值从 opencode 侧 theme + 各 route 组件的**属性**提取(source extraction)。
- **捕获方式 = 源码静态提取**(DoD-1 / 风险 §6 的 R0 降级预案):真 TUI 需活 opencode server + model + 终端
  录制,本环境不可行 → 按 plan28 R0 惯例走源码提取。色值是**字面 hex**(`opencode.json` dark 变体),
  确定性强,不依赖终端主题差异。真终端截屏遗留(需人工在装了 opencode 的终端跑;记 U0 遗留)。
- **许可**:opencode MIT。本包只收数值/结构对拍用,不搬源码本体。
- tokens:`tokens.opencode-dark.json`(色板 + markdown/diff/syntax 色 + 布局规则)。

## ★ R0 关键判定(三个,钉死美学方向)

1. **边框语汇 = 仅左侧重竖线 `┃`**(`ui/border.ts:15-21` SplitBorder;`EmptyBorder` 把其余 box-drawing
   位空格化)。**无 `┌─┐└┘` 角、无圆角、无整框**。→ tui flavor:card 圆角=0、AO=0、glow=0,card 退化为
   **左边竖条 + 平底填充**(user/tool/error 卡);assistant 正文**无卡无底**。表格/diff 内部才有 box-drawing 网格。
2. **TUI 其实是「富」的**(纠正 GOAL §0 的假设):@opentui 的 `<markdown>` **有** tree-sitter 语法高亮、
   markdown 强调/标题/列表/引用**按 ANSI 属性 restyle**(不留字面 `#`/`>`/`-`)、**表格渲成 grid**
   (`style:"grid"`,`routes/session/index.tsx:1692`);`<diff>` **有**词级高亮 + 行号 gutter + 分屏
   (`:2351-2369`)。→ 按 §0-1「TUI 有什么做什么」:**保留**语法高亮/词级 diff/grid 表格/富 markdown
   (引擎 rich 已具备,复用即可);**关掉**的是 TUI 没有的:圆角、AO、glow、dissolve、assistant 底卡。
3. **字体 = 终端等宽**(1 cell = 1 char),源码无 font-size/line-height 配置(grep 空)。→ tui flavor 全 mono
   role(引擎 `layout-bridge.ts` 已有 `"mono"` preset,`setFontPreset("mono")` + `refresh_fonts()`)。

## 逐部件渲法(file:行号)

### user 消息(`routes/session/index.tsx:1361-1465` UserMessage)
- 左边框卡(`border:["left"]`,char `┃`,`borderColor=agent色`);填充 `backgroundPanel #141414`;
  padding 上下1/左2;消息间 `marginTop=1`(首条0)。
- 文本**原样渲染,不解析 markdown**(`:1415` `<text>{text()}</text>`,fg=text)。无 "You"/前缀。
- 附件 = 彩色 inline badge `[ img ] filename`。

### assistant 消息(`:1467-1570` AssistantMessage)
- **无边框、无卡、无底**;各 part 直接渲染,缩进 `paddingLeft=3`。
- 尾行元数据:`▣ Build · <model> · <duration>`(agent 色方块 + muted 文字);中断 → `▣` muted `· interrupted`。
- assistant 错误 = 左边框卡,`borderColor=error`,bg backgroundPanel,muted 文字。

### markdown text(`:1681-1700` TextPart)
- `<markdown>` renderable,`paddingLeft=3`,`marginTop=1`;`syntaxStyle` 全 tree-sitter;`streaming=true`。
- 强调/标题/列表/引用/链接 = **ANSI 属性 + 色**(见 tokens.markdown):粗→bold+橙、斜→italic+黄、
  标题→bold+紫(H1 下划线)、引用→italic+黄、链接→下划线、删除线→muted、行内码→绿+背景=background。
- **不保留字面 `#`/`>`/`-`**(renderable 消费之)。**表格 = grid**(`style:"grid"`)。

### 代码块 / 代码
- markdown 内 fenced → 语法高亮(tree-sitter)。独立文件内容(write)用 `<code>`+`<line_number>`(行号,minWidth=3)。
  无圆角边框、无独立底盒(底 = theme background)。

### tool 卡(`:1704-1781` ToolPart,两形态)
- **InlineTool** 单行(`:1908-1992`):`paddingLeft=3` + 2 列 icon glyph + 文本。pending=`~ text`;
  完成=`<icon> content`(icon:bash`$` glob/grep`✱` read`→` write/edit`←` webfetch`%` websearch`◈` 通用`⚙`);
  完成褪为 muted;失败→error;拒绝→**删除线**。
- **BlockTool**(`:1994-2040`):左边框卡(`┃`,borderColor=background),bg backgroundPanel;标题行 muted;
  长输出折叠到 N 行(bash 10/通用 3)"Click to expand"。

### diff / 文件编辑(`:2330-2381` Edit / `:2383-2457` ApplyPatch)
- `<diff>` renderable:宽>120 分屏否则 unified;`showLineNumbers`(行号 gutter,per add/remove 底 tint);
  **词级高亮在**(`addedSignColor=#b8db87`/`removedSignColor=#e26a75`,wrapMode 默认 word);
  +/- 行整行底 `diffAddedBg/diffRemovedBg`,fg `diffAdded/diffRemoved`。标题 `← Edit path`。

### reasoning(`:1580-1679` ReasoningPart)
- `paddingLeft=3`,`marginTop=1`;流式=spinner+`Thinking: title`,完成=`Thought: title · dur`
  (warning 色 × 0.6 opacity);hide 模式 = 单行 `+ `/`- ` 折叠。body = markdown,**dimmed 语法**(fg textMuted)。

### 权限 ask(`routes/session/permission.tsx`)
- 头:`△ Permission required`(warning 三角);muted icon+title;body 按类型(edit→`<diff>`,bash→`$ cmd`,
  list→`- pattern` 明文前缀);选项 `Allow once / Allow always / Reject`;拒绝卡 = 左边框 error 色。
- todo(`component/todo-item.tsx`):`[✓]`完成 / `[•]`进行(warning) / `[ ]`待办(muted)。

### 分隔
- **无消息间横线**;分隔 = `marginTop=1`(一空行)+ 卡的左边框形态。compaction = 居中顶边 rule ` Compaction `。

## 布局 / 间距
- 外框 `paddingLeft/Right=2 paddingBottom=1`;assistant 内容缩进 **3 cell**;卡内 `paddingLeft=2`。
- 统一元素间距 = `marginTop=1`(一空行);卡内 `gap=1`。无显式 max-width;分屏 diff 阈值宽 120。

## 引擎映射(flavor 机制,U2 用;架构见 0021/0033/0037)
- **theme** → `web/public/themes/tui.json`(色板;color 在 emit 解析,换 theme 零缓存失效)。
- **font** → web 侧 `setFontPreset("mono")` + `refresh_fonts()`(0021 边界:字体在 JS)。
- **渲染器** → `components::partspecific::tui_registry()`(0033;register 一行/renderer;`render_flavor` 字段选)。
  R0 结论:markdown/diff/table 渲法与 rich 同(TUI 也富)→ 复用;差异在 theme + 装饰扁平 + mono。
- **装饰参数**(圆角/AO/glow)硬编码在 `app.rs build_frame`(505 radius / 510-518 ao/glow 等)→ **flavor 门置零**
  (tui:radius=0、ao=0、glow=0、card→左竖条 char/1px)。
- **切换** → wasm `set_render_flavor("tui"|"rich")`(承 `set_reveal_preset` 形;未知拒 AR12);切换 `mark_layout_dirty()`
  重渲(registry 换 → 缓存的 spans 变,须失效)。
- **动效档** → tui:rhythm typewriter 默认 + effect preset off。

## 遗留
- 真终端截屏未采(需活 opencode + 终端;R0 降级源码提取,plan28 惯例)。
- agent 轮转强调色(7 色)= 引擎单色 user/assistant 简化;记容差豁免。
- InlineTool 单行形态(icon glyph + 文本)= 富形态的 TUI 特化,U2/U3 视收敛程度做或记遗留。
