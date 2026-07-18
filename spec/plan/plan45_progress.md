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

(未开始)

## U2 · 渲染器集全部件(flavor 注册 + 逐部件对拍)

(未开始)

## U3 · 对拍收敛(容差判定 + 回改)

(未开始)

## U4 · 收口(切换面 / 动效档 / golden 双家族 / 门)

(未开始)

## DoD 对账

(未开始)
