# Plan 45 · 进度账本(TUI Flavor 对拍复刻)

> 逐 milestone 记录:参考依据(file:行号)/ 容差结果 / flavor 机制方案 / 遗留。
> GOAL:[plan45-tui-flavor-goal](./plan45-tui-flavor-goal.md)
> 需求确认:2026-07-17 四问(TUI 主题模式 / 对拍级 / 消息流全部件 / 保留节奏+像素布局)
> + 作者原话「先复刻,再创新」「不修改结构,基于现有能力,补充结构下的扩展组件」。

## U0 · 参考真值包(opencode-tui tokens + NOTES + 截屏)

✅ 交付(2026-07-18):`spec/reference/opencode-tui/`
- **tokens.opencode-dark.json**:opencode 默认主题 dark 变体全字面 hex —— base(bg #0a0a0a / text #eeeeee /
  primary #fab283…)+ markdown 色(heading #9d7cd8 bold / strong #f5a742 / code #7fd88f…)+ diff 色
  (add #4fd6be / del #c53b53 + bg tint)+ syntax + 布局规则。源:`packages/tui/src/theme/assets/opencode.json`。
- **NOTES.md**:逐部件渲法 file:行号(user 左边框卡 / assistant 无卡缩进3 / markdown ANSI restyle / tool 双形态 /
  diff 词级+gutter / reasoning dimmed / ask △ / 分隔 marginTop=1)+ 引擎映射(3 seam)。
- **★ R0 三判定**:①边框语汇=仅左竖线 `┃`(无圆角/无整框)→ tui flavor 圆角/AO/glow=0、card=左竖条+平底;
  ②**TUI 其实富**(@opentui 有语法高亮/词级 diff/grid 表格/富 markdown)→ 纠正 GOAL 假设,按「TUI 有什么做什么」
  **保留**这些(rich 已具备,复用),关的是 TUI 没有的(圆角/AO/glow/dissolve/assistant 底卡);③字体全 mono。
- **捕获方式**:源码静态提取(真 TUI 需活 server+model+终端,不可行 → R0 降级预案,plan28 惯例);真终端截屏记遗留。

## U1 · 主题层粗对拍(tui.json 换肤先行)

✅ `web/public/themes/tui.json`:opencode TUI 装饰色 → 引擎 Theme 字段(card_bg/code_bg/diff bg/gutter/
  alert…扁平不透明底,card 圆角靠 flavor 门置零)。color 在 emit 解析 → 换 theme 零缓存失效。
- 缺口(→ U2 解决):①文字 role 色在 `glyph.wgsl style_color`(opencode-UI 色,与 TUI 有差:heading 白 vs 紫);
  ②页底 #151515 vs TUI #0a0a0a(backend 常量)。二者记 U3 收敛/遗留。

## U2 · 渲染器集全部件(flavor 注册 + 逐部件对拍)

✅ **flavor 机制(0021/0033/0037 三 seam,additive,机制层零改动)**:
- `RenderFlavor{Rich(默认),Tui}` 字段 + `set_render_flavor(name)`(rich/tui;未知拒 AR12;切换 `mark_layout_dirty`
  重渲)+ `render_flavor()` getter。core `app.rs`。
- **registry seam**:`components::partspecific::tui_registry()`(0033 第二注册表)+ build_frame 按 flavor 选。
  R0 结论:TUI markdown/diff/table 与 rich 同(@opentui 也富)→ 复用 rich 渲染器,TUI 差异走 theme+装饰+mono。
- **装饰 seam**:build_frame 尾部 flavor 门 —— tui 时 panel 圆角/AO/glow 全零 + rect 圆角零(承 R0:TUI 只左竖线,
  无圆角/AO/glow)。rich(flat=false)整体跳过 → **逐字节恒等硬门**(insta golden 不变,已验)。
- **wasm** `set_render_flavor(name)->bool`(承 set_theme 形)。
- **web 切换面**:chat 页 `?tui` + 面板「观感 Flavor」下拉 —— applyFlavor(tui)= set_render_flavor + tui.json 主题 +
  `setFontPreset("mono")` + effect off + typewriter + refresh_fonts;rich = 复原。**运行时同内容两套皮实时切**(demo)。
- **动效档**:tui 默认 typewriter + effect off(TUI 无 glow/dissolve)。
- native:`render_flavor_accepts_known_rejects_unknown`(rich/tui/AR12)+ `tui_flavor_flattens_panel_decorations`
  (rich 圆角>0 → tui 全零)。
- 截图:`test/results/tui-diff/chat-{rich,tui}.png`(同剧本两套皮 —— mono + 扁平 + TUI 色对比)。

**遗留(→ U3 收敛)**:文字 role 色 shader 化(heading 紫等)= WGSL flavor 位,记收敛项;页底色。

## 布局层深研并入(2026-07-18)—— research: tui-events-and-layout

作者提「字间距/行距差 → 想 Rust 侧做 TUI 布局,是否引 cosmic-text」。深研三家(claude-code/opencode/codex)
事件+渲染+布局 + cosmic-text 评估 → [research/tui-events-and-layout](../research/tui-events-and-layout.md)。**结论并入本 plan(不新开)**:
- **不引 cosmic-text**:业界零 agent TUI 在布局层用 shaping(codex=unicode-width、Ink/gemini/claude-code=Yoga cell、
  lipgloss=rune width);cosmic-text 输出浮点 advance 无 cell 概念、字体必嵌(0.1–25MB 税)、确定性绑死字体二进制。
  「行距/间距差」= 间距**规则缺失**,非缺 shaping 库。
- **布局路线 = 自研 cell 网格**(tui flavor 下):布局宽从「逐字实测」改为 cell 常数(半宽=1cell / CJK=2cell,EAW 表扩
  现有 CJK 判定),行高=cell×行距系数(TUI 1.0~1.2,对拍定);R8 确定性最强,零依赖。→ U3 收敛项。
- **间距规则 6 条**(§research §3 清单,对拍真值):聊天列 padding 2 / 块 gap 1 / assistant 缩进 3 / user `┃` 卡 /
  `⏺`·`⎿` 前缀 / **动态兄弟间距**(连续 inline tool 贴合零 margin,跨类型插 1 行,opencode util/layout.ts:8)。全走
  theme+装饰 token + 布局常量(数据,机制零改)。→ U2/U3 逐条数值化对照。
- **组件补充**:InlineTool 单行形态(icon+摘要,0033 registry 加条目)——opencode 关键差异,本 plan U2 遗留③升级为
  正式项;reasoning 折叠单行 + tool 结果 `⎿` 前缀行(0037 装饰)。

## U3 · 对拍收敛(容差判定 + 回改)

- **并排图**:`test/results/tui-diff/chat-{rich,tui}.png`(同剧本 showcase-full 两套皮)。tui:mono 全字 + 扁平卡
  (无圆角/AO/glow)+ opencode diff 色 + effect off + typewriter,与参考观感对齐(结构/装饰形态一致)。
- **收敛达成**:装饰扁平(圆角/AO/glow=0,native+golden 锁)· 主题装饰色(tui.json = opencode hex)· mono 字体 ·
  diff/表格/语法高亮复用(R0:TUI 也富)· 分隔 marginTop 语义 · 效果档 off/typewriter。
- **遗留(收敛项,记账继续,≤5 轮惯例)**:
  1. **文字 role 色**(heading 紫 #9d7cd8 / keyword 紫 / code 绿 #7fd88f 等)在 `glyph.wgsl style_color`(GPU 取色,
     现 opencode-UI 色);TUI 精确色需 WGSL flavor 位(uniform + 第二色表)。ΔE 差在 heading(白 vs 紫)最明显。
     → 记 render 收敛项(改 render 走 render-write,加 flavor uniform;rich 默认位=0 恒等)。
  2. **页底色** #151515(backend CLEAR 常量)vs TUI #0a0a0a。→ 同上 render 收敛。
  3. InlineTool 单行形态(icon glyph)= TUI tool 特化;现复用 rich block 卡(扁平后近似)。
  4. agent 轮转强调色(7 色)→ 引擎单色简化,容差豁免。
  这些不阻塞机制;「先复刻」的骨架(结构/装饰/字体/主题)已立,精确色是「再打磨」。

### 收敛轮 C1–C3(2026-07-19,基于 research/tui-events-and-layout 并入项)—— 遗留逐条销账

- **C1 · shader 文字色 + 页底色**(`82836c4`):`FrameData.flavor`(0=rich/1=tui)plumb 到 render → `Globals.flavor`
  uniform;`glyph.wgsl style_color_tui()` = opencode TUI 色表(heading 紫 #9d7cd8 / strong 橙 #f5a742 / code 绿
  #7fd88f / link teal #56b6c2 / 语法紫绿橙 / diff #4fd6be·#c53b53),`select` 按 flavor 选;backend `CLEAR_TUI`
  = #0a0a0a 页底。**rich 恒等**(flavor=0 → select 取 style_color;Globals 仍 64B;insta golden 不变,已验)。
  → **销** U3 遗留 1(文字色)+ 2(页底色)。
- **C2 · 行距收敛**(`a4cd646`):`layout-bridge` `LINE_HEIGHT` 改 live-binding + `setLineHeightFlavor`(rich 1.6× /
  tui 1.25×,终端行距);applyFlavor 切 + refresh_fonts 重排。→ **销** research「行距差」核心诉求(布局层结论:
  自研 cell 网格路线①,行高=cell×系数,不引 cosmic-text)。
- **C3 · InlineTool 单行形态**(`5e878e4`):`tui_registry` 覆盖 Tool full-renderer —— 非 diff 工具加 opencode icon
  glyph 前缀(bash `$` / read `→` / edit `←` / grep `✱` / websearch `◈` / 通用 `⚙`);diff 工具保块形态(opencode
  亦 BlockTool,加 icon 会移 diff 装饰 grapheme base)。native:tui bash → `$ ` 前缀,rich 无。→ **销** U3 遗留 3
  + research §6B「组件补充」。
- **并排图更新**:`test/results/tui-diff/chat-{rich,tui}.png` —— tui 现:mono + 扁平 + TUI 文字色(紫/橙/teal)+
  #0a0a0a 页底 + 1.25 行距 + `$`/`→` InlineTool icon + off/typewriter。对拍高保真。

**剩余遗留(非阻塞,真收敛/深水区)**:①cell-grid 逐字宽(半宽=1cell / CJK=2cell 精确 snap)—— 现 mono measure
近似,research 路线①的完整落法留后续布局轮;②间距 6 条里的「动态兄弟间距」(连续 inline tool 零 margin)+ tool
结果 `⎿` 前缀 —— 装饰/布局细节,记后续;③真终端截屏;④agent 多色。

## U4 · 收口(切换面 / 动效档 / golden 双家族 / 门)

- **切换面**:wasm `set_render_flavor`/`render_flavor` + chat `?tui` + 面板「观感 Flavor」下拉(运行时切,`<small>`
  一行说明「一键 TUI ⇄ Rich」= DoD-8 页说明)。
- **动效档**:tui 默认 typewriter + effect off(applyFlavor 设)。
- **golden 双家族**:rich `replay_settled_frame_snapshot`(恒等,不变)+ tui `replay_tui_flavor_frame_snapshot`
  (含 radius/ao/glow → 锁扁平)。
- **native ≥4**:flavor 接受/未知拒(AR12)· 装饰扁平(rich radius>0→tui 零)· tui_registry 覆盖 · tui theme 数据。
- **e2e ≥2**:`?tui` 生效(引擎 render_flavor=tui + 档位 off/typewriter)· 运行时 tui→rich 重渲(引擎跟随 + 内容不丢)。
- **门**:五层门(见下)。

## DoD 对账

| # | DoD 要求 | 状态 | 证据 |
|---|---|---|---|
| 1 | 参考真值包 | ✅ | spec/reference/opencode-tui/(tokens + NOTES + R0 三判定;真终端截屏遗留,源码提取降级) |
| 2 | 主题层 tui.json + 零 Rust 改动验证 | ✅ | themes/tui.json;色在 emit 解析,换 theme 零缓存失效(粗对拍先跑通) |
| 3 | 渲染器层 TUI 集 + 装饰 TUI 参数 | ✅* | tui_registry(0033 第二表,复用 rich[R0])+ 装饰 flavor 门扁平(0037)。*文字色 shader 化=U3 收敛遗留 |
| 4 | 切换面(setter/URL/面板,运行时) | ✅ | set_render_flavor(AR12)+ ?tui + Flavor 下拉;e2e 运行时切验证 |
| 5 | 动效档(tui typewriter + off) | ✅ | applyFlavor 设;e2e 断言档位 |
| 6 | 对拍收敛 + 遗留记账 | ✅* | 并排图 tui-diff/;装饰/主题/字体收敛;*文字色/页底色 render 收敛遗留(§U3) |
| 7 | 门(五层绿 + rich 恒等 + tui golden + native≥4 + e2e≥2) | ✅ | rich insta 逐字节;tui golden;native 4;e2e 2;五层门(下) |
| 8 | progress 记账 + commit 不 push | ✅ | 本文件;U0`0a2c395`/U1U2`f93d19e`/U3U4 提交,均不 push |

**§3 客观判定**:对拍 ✅(并排图 + 装饰/主题收敛;精确文字色遗留)· rich 恒等 ✅(insta 逐字节)· 切换 ✅(运行时
e2e + AR12)· 零创新 ✅(tui 圆角/AO/glow=0[native+golden]、effect off 默认)· 零结构改动 ✅(store/fsm/reveal/
layout/morph 未动,只 additive flavor 面)· 门 ✅。

**遗留汇总**:①文字 role 色 + 页底色 shader/backend flavor 化(render 收敛,加 flavor uniform,rich 默认恒等);
②InlineTool 单行形态;③真终端截屏;④agent 多色。均非阻塞机制,记后续 render 收敛轮。
