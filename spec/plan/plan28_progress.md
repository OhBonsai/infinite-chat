# Plan 28 · 进度账本(opencode 还原)

> 逐 milestone 记录:做了什么 / 参考依据(file:行号)/ 容差结果 / 遗留。
> 参考仓库:`~/w/agentscode/opencode` @ `4ddfa7c6`(fix(stats): reorder leaderboard cards)。

## R0 · 参考真值捕获

- [进行中] 2026-07-07 起。
- **主题基准决策**:捕获 dark scheme 为主(我方引擎现为暗色;参考 `packages/ui/src/styles/theme.css:349` `@media (prefers-color-scheme: dark)` 块),light 变量一并入 tokens.json 备查。
- 全局 token 源:`packages/ui/src/styles/theme.css`(:root 309 个变量;typography/spacing/radius/shadow + 语义色板 light/dark 双份)。
- 捕获路径:侦察中(app+server 夹具 vs storybook vs 静态提取)。

## R1 · 全局 token 置换

(未开始)

## R2 · 消息骨架

(未开始)

## R3 · 逐组件

(未开始)

## R4 · 动效 / R5 · 收口

(未开始)
