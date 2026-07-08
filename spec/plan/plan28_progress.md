# Plan 28 · 进度账本(opencode 还原)

> 逐 milestone 记录:做了什么 / 参考依据(file:行号)/ 容差结果 / 遗留。
> 参考仓库:`~/w/agentscode/opencode` @ `4ddfa7c6`(fix(stats): reorder leaderboard cards)。

## R0 · 参考真值捕获

- [进行中] 2026-07-07 起。
- **主题基准决策**:捕获 dark scheme 为主(我方引擎现为暗色;参考 `packages/ui/src/styles/theme.css:349` `@media (prefers-color-scheme: dark)` 块),light 变量一并入 tokens.json 备查。
- 全局 token 源:`packages/ui/src/styles/theme.css`(:root 309 个变量;typography/spacing/radius/shadow + 语义色板 light/dark 双份)。
- 捕获路径:侦察中(app+server 夹具 vs storybook vs 静态提取)。

## R1 · 全局 token 置换 —— 基本完成(2026-07-08),黄金帧重录中

参考依据全部 = `tokens.live-dark.json` + NOTES.md;逐项:

| 落点 | 改动 | 参考依据 |
|---|---|---|
| `glyph.wgsl style_color` | 文字全表:正文 #A0A0A0 / strong·标题·代码 #EDEDED / weak #707070 / weaker #505050;Link #c0d4fb;syntax 8 色 → opencode --syntax-*;Diff 文字 #c4ffc0/#ec2f14;AskButton 深字 #191919(亮 primary 底) | NOTES §2/§4/§8;层级靠亮度不靠色相 |
| `backend.rs CLEAR` | 页底 #0D0F17(蓝黑)→ **#151515** | --background-stronger(时间线容器) |
| `theme.rs` 默认表 | 全表置换:code_bg #202020 / 边框族 #282828 / user_bg #1C1C1C / diff 底 #001401·#240200 / selection #022fa6 / ask 按钮 = 亮 primary #ede8e4;护栏测试改锚 opencode 真值 | --surface-inset/-base/--border-weak/--surface-diff-*/--button-primary |
| `motion.rs` | space_turn 40→**24**,space_part 16→**12**(style-config.ts 镜像同步) | session-turn.css gap |
| `boxlayout.rs` | CONTENT_MAX 760→**800**;BUBBLE_RATIO 0.85→**0.82** | message-timeline.tsx:1096;user-message-body max-width 82% |
| 头像 orb 列 | **整体移除**(发射块+常量+测试改「不得出现」);流式指示留给 thinking 行/行内光标 | 参考无头像结构(NOTES §0/§1) |

- 快照:insta replay 快照按新观感重录;native 258 全绿,clippy 0。
- 验收帧:`test/results/restore-diff/r1-sdf.png`(settled 定帧)vs 参考 shots —— 底色/列宽/
  字号层级/代码面板/表格网格一眼同族 ✓。
- 遗留(转 R2/R3):① 波浪分隔线(我方 turn/hr 装饰)参考没有 → R2 撤;② h1/h2 下划线参考
  没有 → R3-markdown;③ Alert 蓝底保留(参考无此组件,GitHub 特性继续)。

## R2 · 消息骨架 —— ✅ 完成(2026-07-08)

| 项 | 改动 | 参考依据 |
|---|---|---|
| `---` 分隔线 | 喵喵分隔线(WIDGET_RULE_CAT)退役 → WIDGET_RULE 素细线(bright 0.95 ≈ 全宽实线,≤1px 容差);测试改锚 | 参考 hr = 素线(shots/s9) |
| user 气泡 | FramePanel 补 1px `card_border`(#282828)描边 + 圆角 10→**6**;padding 8×12(inset 推导,恰合参考) | message-part.css:128-137 |
| 标题下划线 | `head_rule` → alpha 0 关闭(装饰仍在管线,数据驱动关) | 参考标题无下划线(shots/s2) |
| turn/part 间距 | R1 已到位(24/12),本段验证 ✓ | session-turn.css |

- 验收帧:`test/results/restore-diff/r2-sdf.png`(showcase)+ `r2-mini.png`(/chat 600px 列:
  气泡/工具卡/落定 ask 卡全款)。
- 附:`seek_reveal+set_paused` 定帧会产生代码块/表格**重影假象**(morph 中间态),截图工具改为
  1e9 cps 自然收敛 —— 非渲染回归,已记录。
- assistant 尾部 meta 行(Agent·Model·时长)归入 R3-text(内容型特性,非骨架)。

## R3 · 逐组件

### R3.1 markdown/text —— ✅(2026-07-08)

真值源:`markdown.css`(逐行)+ `computed/s2-markdown.json`(实测:text-part 容器 #A0A0A0,
**markdown 内层 #EDEDED 14px/160%** —— R1 的"正文=text-base"判断被 computed 修正)。

| 项 | 改动 | 依据(markdown.css) |
|---|---|---|
| 正文/表体色 | Normal/Italic/TableCell/TableEm → **#EDEDED**;Bold/TableStrong → #FFFFFF 近似(单字重字体无法 medium,ΔE≈4 容差内) | 容器 color = --text-strong;strong 靠字重 |
| 标题 | **全级同字号 14px**(roleScale 全 1.0);色 #EDEDED | h1-h6 font-size:14px + medium |
| 行内码 | 色 **#00ceb9**(--syntax-string)+ 无底 chip(code_chip alpha 0) | :not(pre)>code(背景被注释) |
| mono 字号 | code/syntax/行号 roleScale 0.93(≈13px) | .shiki font-size 13px |
| hr | **纯留白无线**(hr_rule alpha 0;R2 素线再修正) | "Invisible spacing only" |
| 引用条 | --border-weak-base #282828 | blockquote border-left 2px |
| 表格 | 表头无底、无 AO、无圆角(TableStyle + style-config 镜像) | th/td 仅底线结构 |

- 帧:`test/results/restore-diff/r3-markdown-sdf.png`。

### R3.2 reasoning + R3.3 tool 卡 —— ✅(2026-07-08)

| 项 | 改动 | 依据 |
|---|---|---|
| reasoning | **裸弱化正文**:撤 "💭 Thinking" 合成标题(那是 turn 级状态行,R4 shimmer 归位);折叠 → 单行 "Thinking" 弱字 | message-part.css:280 / NOTES §3 |
| 整卡面板 | tool/reasoning 的 SDF 卡底 + hover 抬起 + press 缩放 + running 边缘流光 + 四态徽章 **全部退役**(机制保留管线,数据不发;card_status 留 R4 shimmer) | 参考 trigger 无底、无徽章(shots/s4) |
| tool trigger | `▸ name [badge]` → **显示名(strong)+ 副题(weak)**:bash→Shell、websearch→Web Search…;副题 = payload.title 或 compact input | basic-tool.css / NOTES §4 |
| bash 正文 | `$ cmd` + output 走**围栏代码**(复用代码块管线 → #202020 inset 面板 + mono 13px,与参考 bash-output 同构) | message-part.css:358 |
| 测试 | 12 native + 5 e2e 断言改锚新观感;3 个退役机制测试 → 「无卡无徽章」缺席守卫;渲染快照重录 | — |

- 帧:`test/results/restore-diff/r3-tool-mini.png`。
- 遗留:① bash 面板带行号(参考无)→ 待代码块行号开关;② 11243s 全门 = 机器睡眠冻结计时器
  (patch 已录同款),6 失败中 4 为睡眠伪象,复跑全绿;真失败 2 处为漏改断言,已修。
- 遗留:① 表格仍画竖线/外框(我方 table shader 为整格网;参考仅横线)→ 待 shader 加线模式;
  ② markdown 垂直节奏(标题 mb24/p mb12/li mb8)未逐项对齐(排版器行距模型不同)→ 记录待测量。

## R4 · 动效 / R5 · 收口

(未开始)
