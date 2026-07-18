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

## U3 · 对拍收敛(容差判定 + 回改)

(未开始)

## U4 · 收口(切换面 / 动效档 / golden 双家族 / 门)

(未开始)

## DoD 对账

(未开始)
