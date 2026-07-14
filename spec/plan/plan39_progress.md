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

- **全量 44/44 绿**(34 png + 10 webm),一次全量捕获 ~8 分钟;资产总 **19MB**
  (test/results/feature-assets/,gitignored;远低于 80MB 预算)。
- **失败修复**:fx-noise/fx-distance-band 的 clip 坐标按设备像素(dpr=2)重算(gallery
  第 69 格 x784/y112、演示条 y215);断言 assertVisible 跑在步骤末,故动效项(dissolve)
  时序停在中段。
- **确定性抽查(≥5,DoD)**:md-inline / md-math / fx-noise / theme-panel / md-quote-alert
  / canvas-virtualized 六项双跑**逐字节一致**。**踩坑**:showcase/chat-full 全窗截图含
  左缘动态 orb 头像(每帧脉动)→ 无 clip 的 png 加统一裁剪(x≥260 排除 orb;chat-full
  裁输入区);带额外 pan_by/活跃动效(chat-full 工具卡流光)的项不纳入确定性抽查
  (webm 本就不要求逐字节)。
- **fail-loud**:任一断言失败即该 test 红、无残缺报告(R3 生成器再加"资产缺失非 0 退出"
  的第二道)。故意删项验证记 R3。

## R3 · 报告生成器(单文件内嵌 + 样式 + 体积预算)

- **`scripts/build-feature-report.mjs`**:读 manifest + feature-assets + summary.json →
  `test/results/feature-report.html`(**13.4MB 单文件**,base64 内嵌所有 png/webm)。
- **fail-loud(报告即测试第二道)**:任一项资产缺失 → 列出并**非 0 退出**(实测:临时
  移走 md-inline.png → exit 1;恢复即恢复)。
- **样式**:暗色引擎色系(--bg #131316 等)、左侧粘性目录(九大项+小项锚点)、总览头
  (commit/项数/资产 MB/五层门 PASS 引 summary.json)、小项卡片(标题+kind 徽章+人话
  解释+图或 `<video controls>`)、`loading=lazy`;窄屏目录转顶栏。
- **体积预算**:80MB 硬顶;超则按 webm 大小降序降为 poster 文案条(降级项记入总览头);
  当前 13.4MB 远未触发。
- **目视验收**:Chrome 打开渲染精良(目录导航、webm 播放器、png 卡片、懒加载均正常)。
- **遗留**:chat-full = film 剧本页,截图右缘含一线 film 播放器 chrome(对话列居中偏左、
  右侧是 film 控件);已收窄 clip 到对话列,主体清晰,残留一线可接受 —— 后续可加
  「film 无 chrome」专用场景根除。

## R4 · 收口(一键命令 / 文档 / 作者验收)

(未开始)

## DoD 对账

(未开始)
