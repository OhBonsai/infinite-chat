# Plan 39 · GOAL:功能体检报告 —— 全量可感知功能的 e2e 捕获 + 单文件 HTML 报告(agent 直跑,无人介入)

- 日期:2026-07-14
- 状态:**GOAL(可执行任务书)** —— 给 Claude Code 的完整执行合同,读完即可自主跑到 DoD。
- 执行方式:**顺序执行,master 工作区直跑**。当前 master = plan28–38 全量成果(五 crate、五层门 449+ 测、效果系统三部曲收口)。
- 定位:**这一大轮(plan28–38)的完整功能体检与介绍**——把"人可感知的功能点"(非性能)全量梳理成清单,逐项经 e2e 捕获截图/webm,产出一份**精良的单文件 HTML 报告**(作者双击即看:大项→小项→解释→图/视频)。**常驻资产**:一条命令可重跑,以后每个大 plan 收口后刷新。
- 作者需求确认(2026-07-14 四问):报告=**单文件自包含**(截图与 webm 均 base64 内嵌);动效=**截图 + 关键动效 webm(~10 段)**;范围=**全量细项**;定位=**常驻可重跑**。
- 前置资产(全部已在,本 GOAL 只组织不新造):showcase 剧本(S1–S10 + showcase-full)、`?gallery`(69 格)、replay 录像(含 real-server 4 卷)、shoot-sdf.mjs(SHOT_DPR 定帧)、style-panel 全旋钮(rhythm/motion/effects/preset)、五层门全绿。

---

## 0. 铁律(第一条):报告即测试,缺项即红

**manifest 是单一真值源**:每个功能小项在 `test/feature-manifest.json` 里登记(id/大项/小项/解释/捕获方式);捕获 spec 逐项执行时**必须带可见性断言**(核心元素/文本/像素存在)——某项捕获失败 = 该功能红 = 报告不生成残缺版而是 fail loud。这满足"这一大轮修改能通过 e2e 测试"的本意:**体检本身就是一层 e2e**。

## 1. GOAL 与完成定义(DoD)

**GOAL**:(A) 全量梳理 plan1–38 的人可感知功能点 → `feature-manifest.json`(预计 8–10 大项 / 40+ 小项,含人话解释);(B) 捕获 harness:确定性定帧截图 + Playwright 分段 webm;(C) 单文件 `feature-report.html` 生成器(暗色精良样式,目录导航,图/视频内嵌);(D) 一键重跑命令 + 文档,常驻化。

**DoD(全满足才算完)**:

1. **manifest 全量清单**:大项覆盖(以盘点为准,预计):①流式与节奏(逐字揭示/三预设/句读呼吸/追赶/shimmer thinking/待机指示)②Markdown 全类型(标题/粗斜删/行内码/嵌套列表/任务框/引用/Alert/代码块+折叠视口+copy/表格网格对齐/hr/脚注/链接/图片/公式行内块级/SVG 矢量)③流式结构揭示(全类型 raw 不闪/骨架先行/表格网格先行/reflow 平滑/magic-move 列宽身份)④会话卡片(turn 聚合/tool 卡四态/diff 卡三层视觉+词级+行号+gloop+折叠展开/reasoning/ask 交互卡/compaction)⑤画布与导航(pan/zoom 缩放锐利/滚动跟随三态+回底按钮/代码块内滚动/长会话回看)⑥文本交互(gloop 选区/复制 text+html/链接点击回调/白名单降级/hover 手型)⑦效果系统(spring 入场/dissolve 退场/glow/shadow/条纹/noise 四象限/拖尾/post 三件/三档预设切换)⑧主题与配置(ThemeTokens/style 面板/dpr=2 清晰度)⑨可靠性可感知面(kill-reconnect 录像重放:断线内容不丢+续答/a11y 播报属性)。每小项:`{id, group, title, desc(做了什么+出处 plan/ADR), capture:{kind:png|webm, scene, steps}}`。**manifest 入 git,先于捕获实现交作者 review(R0 即交付"梳理")**。
2. **捕获确定性**:截图走定帧惯例(注入时钟/paused/settle 等待,复用 shoot-sdf 路);同 manifest 双跑截图逐字节一致(webm 例外:编码非确定,要求场景脚本一致 + 时长固定 ±10%,记录于 manifest)。截图主 dpr=1(控体积),**每大项至少 1 张 dpr=2** 证 retina 清晰。
3. **webm 关键动效 ~10 段**(每段 3–8s,独立 Playwright context 录制,天然分段零剪辑,不引 ffmpeg):流式揭示(reader 预设)/骨架先行(表格)/diff 折叠展开 scale 动画/dissolve 退场/spring 入场/拖尾(feedback)/预设切换(subtle→expressive)/滚动跟随(脱离+回底)/magic-move 表格列宽/ask 卡交互。清单可在 R0 调整,总数 8–12。
4. **单文件报告**:`test/results/feature-report.html`——顶部总览(大项计数/生成时间/commit/门状态引 SUMMARY-JSON)+ 左侧粘性目录 + 大项分节 + 小项卡片(标题/解释/出处/图或视频);暗色样式(引擎 theme 色系,精良:圆角卡片/等宽代码字/懒加载 `loading=lazy`);**体积预算 ≤80MB**——超预算自动降质阶梯(png→webp q85→q70;webm 码率降档),仍超则该项降为 poster 帧 + 记录;无网络依赖,双击即开。
5. **常驻化**:一键 `node test/feature-report.mjs`(= 捕获 spec + 生成器);不进五层门(按需跑,README/test/AGENTS.md 注明"每个大 plan 收口后重跑");manifest schema 测试进 unit 层(五层门内,保 manifest 不腐);捕获产物 gitignore,报告本体 gitignore(重跑即得),manifest + 生成器 + spec 入 git。
6. **覆盖断言**:生成器校验 manifest 每项资产齐全 + 每项 e2e 断言过;缺失 = 退出码非 0(fail loud)。
7. 五层门全绿不回退;`spec/plan/plan39_progress.md` 记账(盘点清单/体积数值/遗留);**按 milestone commit,不 push**。

**启动**:`claude --permission-mode acceptEdits "读 spec/plan/plan39-feature-inspection-report-goal.md,按其执行到 DoD 全满足"`(中断重跑同句,读 progress 续跑)。

## 2. Milestones

- **R0 · 功能盘点(梳理交付)**:扫全部 plan progress + showcase 场景 + gallery + 面板能力 → `feature-manifest.json` 全量清单(解释文案人话优先:先"你会看到什么",再"出处");schema unit 测试;**清单本身 commit,供作者 review 增删**。
- **R1 · 捕获 harness**:`web/tests/feature-report.spec.ts`(独立 project,不入门)——png 定帧通道(settle/paused/注入时钟)+ webm 分段录制通道(per-段新 context,recordVideo)+ 逐项可见性断言;先跑通 3 项样板(1 静 1 动 1 面板)。
- **R2 · 全量捕获**:40+ 项资产全齐;双跑截图一致抽查;缺项断言生效(故意删一项验 fail loud)。
- **R3 · 报告生成器**:`scripts/build-feature-report.mjs`(读 manifest+资产 → 单文件内嵌);样式精良(暗色/目录/卡片/懒加载);体积预算与降质阶梯;总览头(commit/门状态)。
- **R4 · 收口**:一键命令封装 + 文档 + 作者验收(打开报告过目)+ DoD 对账;遗留记账(如未来"报告 diff"——两次体检对比,记 backlog 不做)。

## 3. 客观判定

| 维度 | 判据 |
|---|---|
| 清单完整 | manifest 覆盖九大项;每项有解释与出处;schema 测试绿 |
| 报告即测试 | 每项捕获带断言;故意缺项 → 生成器非 0 退出(演示记录进 progress) |
| 确定性 | 截图双跑逐字节(抽查 ≥5 项);webm 场景脚本一致 |
| 单文件可用 | 断网打开正常;体积 ≤80MB(实际数值记 progress);目录锚点/视频播放正常 |
| retina | 每大项 ≥1 张 dpr=2 样张 |
| 不回退 | 五层门全绿;新增 unit(manifest schema)绿 |

## 4. 铁律

- **报告即测试**(§0):捕获失败即红,不出残缺报告。
- **零新依赖**:webm 用 Playwright 自带 recordVideo;图片压缩用现有 pngjs/sharp 已有则用、没有则 png 原样 + 尺寸控制(dpr=1 + 视口裁剪),**不为压缩引重依赖**(超预算优先裁剪视口而非引库)。
- **确定性惯例**:定帧走注入时钟(R8);动效段固定脚本固定时长。
- **Surgical**:不改引擎代码;若捕获暴露真 bug → 按 T9 单独修、commit 分开、记 progress。
- **按 milestone commit,不 push**;manifest/生成器/spec 入 git,产物 gitignore。

## 5. 变更边界

- **主战场**:`test/feature-manifest.json`、`web/tests/feature-report.spec.ts`、`scripts/build-feature-report.mjs`、`test/feature-report.mjs`(一键入口)、README/test/AGENTS.md 两行。
- **非目标域**:引擎全部代码;五层门结构(仅 unit 层加 schema 测);golden 体系(报告截图与 golden 互不混用——golden 管回归,报告管展示)。
- **与既有资产关系**:场景全部复用 showcase/gallery/replay/面板;发现"想展示但没有现成场景"的功能点 → manifest 里 `scene:"TODO"` 记遗留,不为报告新造引擎路径(下轮 plan 补场景)。

## 6. 风险与预案

| 风险 | 预案 |
|---|---|
| 单文件 + webm 体积爆炸 | 预算 80MB 硬顶;降质阶梯;webm 限 720p/低码率/≤12 段;最后手段:该段降 poster 帧 + progress 记录 |
| Playwright recordVideo 含窗口 chrome/尺寸不齐 | 固定 viewport 录制;录制段先隐藏调试面板;接受轻微编码噪声(webm 不进确定性判据) |
| 40+ 项捕获跑一次太久 | 支持 `--only <group|id>` 增量重捕;R2 记录全量耗时;报告生成与捕获分离(资产在即可重出报告) |
| 动效场景难定脚(如 thinking shimmer 需流式中态) | 复用 replay 定点暂停 + 放行时钟;实在不可定的段用 webm(本来就是动效通道) |
| base64 视频部分浏览器兼容 | webm/VP8 是 Chrome/Edge/Firefox 通路;Safari 兜底 poster 帧 +提示(报告注明推荐 Chrome 打开) |

> 一句话:**把 plan28–38 这一大轮的所有"人能看见的东西"编成一份带断言的清单,让 e2e 逐项拍照录像,再装订成一本双击即看的单文件体检报告**——以后每轮收口重跑一次,功能不腐、介绍常新。
