# Plan 39 · 进度账本(功能体检报告)

> 逐 milestone 记录:盘点清单 / 捕获覆盖 / 报告体积数值 / 遗留。
> GOAL:[plan39-feature-inspection-report-goal](./plan39-feature-inspection-report-goal.md)

## R0 · 功能盘点(feature-manifest.json,交作者 review)

- **清单**:`test/feature-manifest.json` —— 9 大项 / **44 小项** / 10 段 webm;每项
  `{id, group, kind, title, desc(「你会看到…」+ 出处 plan/ADR), capture{scene, steps,
  assert, clip?, dpr?, durationMs?}}`。场景全复用既有资产(showcase/chat-full/empty/
  gallery/debug),steps 为 R1 捕获 DSL(settle/waitMs/pause/pushText/pushTool/
  removePart/js/panUp/tapFold/tapLink)。
- **分布**:流式节奏 5 / Markdown 8 / 结构揭示 3 / 会话卡片 7 / 画布导航 4 / 文本交互 4 /
  效果系统 7 / 主题配置 3 / 可靠性 2(+ 每组 ≥1 张 dpr=2,DoD-2)。
- **schema 测试**(unit 层入五层门):结构/唯一性/组引用、DSL 语法、断言必带、webm 时长
  3–8s、「你会看到+出处」文案纪律、retina 覆盖、webm 段数 8–12。
- **遗留标记**:rel-reconnect 的 real-server 卷需拷入 web/public/replays(manifest 已
  `sceneTodo` 注记,R1 落地)。
- **交 review**:清单先行 commit,作者可增删小项/改解释,R1 起按清单实现捕获。

## R1 · 捕获 harness(png 定帧 + webm 分段 + 断言)

(未开始)

## R2 · 全量捕获(40+ 项资产 + fail-loud 验证)

(未开始)

## R3 · 报告生成器(单文件内嵌 + 样式 + 体积预算)

(未开始)

## R4 · 收口(一键命令 / 文档 / 作者验收)

(未开始)

## DoD 对账

(未开始)
