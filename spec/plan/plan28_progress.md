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

## R2 · 消息骨架

(未开始)

## R3 · 逐组件

(未开始)

## R4 · 动效 / R5 · 收口

(未开始)
