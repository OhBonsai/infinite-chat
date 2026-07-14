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

- **独立 project**:`web/feature/`(自带 playwright.config,testDir 独立 → 不入五层门);
  单 worker 保捕获确定;`FEATURE_ONLY=<group|id>` 增量重捕。
- **DSL 解释器**(feature-report.spec.ts):scene→URL 五映射;steps 全实现(settle/waitMs/
  pause/panUp/pushText/pushTool/removePart/tapFold/tapLink/js);断言两式 —— textVisible
  (引擎可见文本,空串=有字即可)+ ink(截图对左上角底色的非底色像素占比 >0.1%)。
- **双通道**:png = 按项 dpr 开 context 定帧截图(clip 可选);webm = 720p recordVideo
  独立 context,close 落盘改名 <id>.webm(+ 大小非空断言);均出 <id>.json 元数据。
- **real-server 卷**:kill-reconnect.json 拷入 web/public/replays/(rel-reconnect 场景就绪)。
- **三样板绿**:md-inline(png dpr2)/ fx-dissolve(webm;踩坑:断言跑在步骤末,末帧全溶
  → 调 manifest 停在溶解中段)/ theme-panel(clip 面板)。
- **产物**:test/results/feature-assets/(gitignored)。

## R2 · 全量捕获(40+ 项资产 + fail-loud 验证)

(未开始)

## R3 · 报告生成器(单文件内嵌 + 样式 + 体积预算)

(未开始)

## R4 · 收口(一键命令 / 文档 / 作者验收)

(未开始)

## DoD 对账

(未开始)
