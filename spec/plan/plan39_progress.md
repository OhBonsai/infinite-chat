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

## R4 · 收口(一键命令 + 文档 + 验收)

- **一键入口**:`test/feature-report.mjs`(= 捕获 spec + 生成器);`--only <group|id>` 增量、
  `--report-only` 仅重出报告(实测 exit 0)。
- **文档**:test/AGENTS.md §2.5(常驻资产用法,每大 plan 收口重跑);README「功能一览」一节
  指向报告。
- **DoD 对账**:见下。

## DoD 对账

1. **manifest 全量** ✅ 9 大项 / 44 小项(每项人话「你会看到…」+ 出处 plan/ADR);schema
   unit 测(结构/DSL/断言/retina/webm 段数,五层门内)。
2. **捕获确定性** ✅ 定帧惯例(注入时钟/settle);≥5 项(实 6)双跑逐字节;webm 场景脚本
   一致(编码非确定性豁免);每大项 ≥1 张 dpr=2。
3. **webm ~10 段** ✅ 10 段(3–8s,独立 context recordVideo,零剪辑)。
4. **单文件报告** ✅ 13.4MB(≤80MB)自包含;总览头(commit/门状态)+ 粘性目录 + 卡片 +
   懒加载;暗色引擎色系;断网可开。降质阶梯就位(未触发)。
5. **常驻化** ✅ 一键命令 + 不进五层门 + schema 在 unit 层 + 产物 gitignore、
   manifest/生成器/spec 入 git。
6. **覆盖断言** ✅ 每项 e2e 断言 + 生成器资产缺失非 0 退出(实测 exit 1)。
7. **不回退** ✅ 五层门全绿(461/461,含新 4 个 schema 单测);记账完整。
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

## R4 · 收口(一键命令 + 文档 + 验收)

- **一键入口**:`test/feature-report.mjs`(= 捕获 spec + 生成器);`--only <group|id>` 增量、
  `--report-only` 仅重出报告(实测 exit 0)。
- **文档**:test/AGENTS.md §2.5(常驻资产用法,每大 plan 收口重跑);README「功能一览」一节
  指向报告。
- **DoD 对账**:见下。

## DoD 对账

1. **manifest 全量** ✅ 9 大项 / 44 小项(每项人话「你会看到…」+ 出处 plan/ADR);schema
   unit 测(结构/DSL/断言/retina/webm 段数,五层门内)。
2. **捕获确定性** ✅ 定帧惯例(注入时钟/settle);≥5 项(实 6)双跑逐字节;webm 场景脚本
   一致(编码非确定性豁免);每大项 ≥1 张 dpr=2。
3. **webm ~10 段** ✅ 10 段(3–8s,独立 context recordVideo,零剪辑)。
4. **单文件报告** ✅ 13.4MB(≤80MB)自包含;总览头(commit/门状态)+ 粘性目录 + 卡片 +
   懒加载;暗色引擎色系;断网可开。降质阶梯就位(未触发)。
5. **常驻化** ✅ 一键命令 + 不进五层门 + schema 在 unit 层 + 产物 gitignore、
   manifest/生成器/spec 入 git。
6. **覆盖断言** ✅ 每项 e2e 断言 + 生成器资产缺失非 0 退出(实测 exit 1)。
7. **不回退** ✅ 五层门全绿(461/461,含新 4 个 schema 单测);记账完整。

## 截图质量改进(2026-07-14 作者反馈:糊 / 宽视口 / 高度)

反馈三点 → 三处改进(harness,不改引擎):
- **高清**:默认画幅 dpr=1 → **dpr=2**(MSDF 字缩放后清晰);`manifest.defaults.dpr`。
- **手机竖屏**:视口 1280×900 → **600×1200**(对话内容列 ~545px 居中占满,两侧不再大片空白;
  像手机屏);`manifest.defaults.viewport{W,H}`,面板/画廊类项 `capture.viewport` 覆盖回宽画幅。
- **合身高度**:harness 加 `autoCropBottom` —— 无 clip 项截完窄视口后按内容底边裁掉空白
  (「截全部或部分高度」;如 diff 卡 2400px → 414px,reasoning → 142px)。
- **8 项 chat-full → empty 合成**:card-{turn-group,tool-states,diff,reasoning,compaction,ask}
  + md-{image,svg} 原走 film 回放页(左对话 + 右播放器,窄视口下对话被挤裁),改 empty 场景
  合成事件(pushText/pushTool/permission.asked)重现,走手机竖屏;顺带捕获从 ~14s 提速到 ~4s。
- **踩坑**:dpr=2 下引擎世界坐标 = 设备像素,`?gallery` 格栅 CSS 位置是 dpr=1 坐标的一半
  (画廊格 clip 需按 CSS 重算);tool-states 用非 context 工具(bash/edit)才不被折叠成一行、
  四态可辨。报告体积 12.7 → 约 13MB(仍 ≤80MB)。

- **commit**:R0 eefd2a1 / R1 8452d23 / R2 1c2ca8b / R3 3d79eef / R4 本次;不 push。
- **遗留**:①chat-full=film 页,截图右缘一线播放器 chrome(可加无 chrome 场景根除);
  ②「报告 diff」(两次体检对比)记 backlog 不做;③Safari 需 poster 兜底(报告注明推荐 Chrome)。
 / R1 8452d23 / R2 1c2ca8b / R3 3d79eef / R4 本次;不 push。
- **遗留**:①chat-full=film 页,截图右缘一线播放器 chrome(可加无 chrome 场景根除);
  ②「报告 diff」(两次体检对比)记 backlog 不做;③Safari 需 poster 兜底(报告注明推荐 Chrome)。
