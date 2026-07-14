# Plan 40 · GOAL:GitHub Pages 重构 —— 引擎即宣传片的 cinematic 官网(agent 直跑,无人介入)

- 日期:2026-07-14
- 状态:**GOAL(可执行任务书)** —— 给 Claude Code 的完整执行合同,读完即可自主跑到 DoD。
- 执行方式:**顺序执行,master 工作区直跑**。当前 master = plan28–39 全量成果(效果系统三部曲 + 功能体检 44 项资产 harness 在手)。
- 作者需求(2026-07-14):①整体介绍 + 入口,该有的功能都放上去;②完整对话页;③markdown demo 页;④整体效果完备、**风格化游戏风格 = 英雄联盟最新 trailer 的感觉**(= 2026 赛季开场动画《Salvation》/ For Demacia:庄重史诗、白金+皇家蓝+鎏金、克制大气、绯红点缀、缓慢推镜与光尘)。
- 核心设计立场:**引擎即宣传片**——官网的 cinematic 感不靠贴图和视频,靠引擎自己跑(intro film + expressive 预设 + noise 雾 + glow/dissolve/拖尾)。官网本身就是最大的 demo。
- 前置资产:[DEPLOY-pages](./DEPLOY-pages.md)(gh-pages 链/PAGES_BASE/VITE_DEMO 全在,沿用)· plan17 intro film · plan25 showcase 剧本 · plan36–38 效果系统(fbm/glow/dissolve/spring/拖尾/预设)· plan39 feature harness(44 项资产可再生)· gallery.html(69 格)· real-server 录像 4 卷。

---

## 0. 铁律(两条)

1. **风格有真值,版权零素材**(承 plan28 第一铁律):P0 先建 LoL 风格参考包(色板/字形语汇/动效语汇的**数值化**),美学决策对照参考;但**零 Riot 资产**——不用其字体(Beaufort 用免费近似:Cinzel/Marcellus 类衬线 display + LXGW 中文)、不用其图像/音频/纹章图形,只抄气质与数值(色相/节奏/构图),grep 断言站内无 riot/lol 素材文件。
2. **引擎渲染优先,静态资产兜底**:一切能用引擎实时跑的都实时跑;无 WebGPU/WebGL2 环境显示定帧 key art + webm 兜底(plan39 资产),并给"请用 Chrome 打开"提示。

## 1. GOAL 与完成定义(DoD)

**GOAL**:重构 GitHub Pages 为四页站:**landing**(cinematic hero + 功能全览 + 入口)、**chat**(完整对话重放)、**markdown**(全类型 demo + playground)、**gallery**(既有,纳入统一导航);整体 LoL《Salvation》气质;沿用现有部署链。

**DoD(全满足才算完)**:

1. **P0 风格参考包**:`spec/reference/lol-style/`——从《Salvation》trailer 截帧 + LoL 官网 UI 提取:色板(深底 #0a0e14 系 / 德玛西亚白金 / 皇家蓝 / 鎏金 #c8aa6e 系 / 绯红点缀)、字形语汇(衬线 display 大标题 + 小字距全大写 eyebrow;免费字体选型记依据)、动效语汇(缓慢放大推镜 3–5s / 光尘粒子 / 金线描边入场 / 雾层视差 / 字幕式淡入,各配时长数值)、构图(全屏 hero / 三分暗部留白)。版权注记同 §0。
2. **landing(index 或 home.html,定名记账)**:①hero = 引擎 canvas 全屏跑 cinematic 开场(intro film 变体:标题 dissolve-in + fbm 雾背景 + 金色 glow 描边 + 慢推镜),8–12s 一循环,`prefers-reduced-motion` 降静帧;②功能全览区 = 九大项 × 精选 ~15 小项卡片(标题/一句话/截图或 webm,资产从 plan39 harness 精选导出为 `web/public/pages-assets/` **入库**——CI 无 GPU 不能现场捕获,取舍记账),LoL 风鎏金描边卡 + 悬停 glow;③入口区 = chat/markdown/gallery/GitHub 四个大入口卡;④页脚:技术栈一句话(Rust+wasm+WebGPU/SDF)+ 版本/commit。
3. **chat 页(完整对话)**:引擎重放完整会话(showcase-full 剧本 + real-server 卷二选一或双 tab),带**重放控制条**(播放/暂停/进度/倍速/节奏预设 typewriter·reader·flow 切换/效果预设 subtle·expressive 切换)——把 demo 变成"可玩的宣传片";输入框禁用态改为控制条(VITE_DEMO 既有机制扩展)。
4. **markdown 页**:①全类型轮播(标题/列表/任务/引用/Alert/代码折叠/表格网格/公式/链接/图片/SVG 矢量,逐段流式揭示,循环播放);②**playground**:textarea 粘贴任意 markdown → 本地合成流逐字喂引擎(纯前端零服务),实时看流式揭示;预设旋钮同 chat 页。
5. **统一导航与风格**:四页共享 nav(金线下划线当前态)+ 风格 token(CSS 变量单一来源 `pages-theme.css`,数值出自 P0 参考包);移动端可浏览(hero 降级/卡片单列);dpr=2 清晰。
6. **部署与门**:vite 多入口(现有 input 扩展);PAGES_BASE 路径全通;`pages.yml` 增量调整(不推倒);**pages smoke e2e**(本地 preview 起子路径版:四页加载/导航/引擎 canvas 有画/资产 200/grep 零 riot 素材)进五层门 e2e 层或独立 project(记账定夺);五层门全绿不回退;DEPLOY-pages.md 更新。
7. `spec/plan/plan40_progress.md` 记账(参考包依据/资产清单/体积/遗留);**按 milestone commit,不 push**(部署验证=本地 preview;真站首绿待作者 push 后观察,规则同 plan35-T4)。

**启动**:`claude --permission-mode acceptEdits "读 spec/plan/plan40-github-pages-cinematic-goal.md,按其执行到 DoD 全满足"`(中断重跑同句,读 progress 续跑)。

## 2. Milestones

- **P0 · 风格参考包**:web 检索/截帧《Salvation》与 LoL 官网 → `spec/reference/lol-style/`(tokens.md + 参考帧 + 免费字体选型);产 `web/src/pages-theme.css` token 表(色/字/动效时长)。无参考依据的样式决策不做。
- **P1 · 站点骨架 + 导航**:四页 IA(landing 新建;chat/markdown 从现 index 演化或新入口,现 harness 保留为 `?dev` 不上导航);vite 多入口 + PAGES_BASE 全通;统一 nav/页脚;移动端布局骨架。
- **P2 · hero cinematic(引擎即宣传片)**:intro film 变体场景(标题 dissolve/spring 入场 + fbm 雾 + 金 glow + 慢推镜循环);reduced-motion/无 GPU 降级(定帧 + webm);hero 下滚动揭示(IntersectionObserver 淡入,LoL 字幕式)。
- **P3 · 功能全览区**:从 plan39 harness 精选 ~15 项导出 `pages-assets/`(脚本 `--only` 增量;体积预算 ≤15MB 入库,webm 限 5–6 段);卡片化(鎏金描边/hover glow/懒加载);九大项锚点分区。
- **P4 · chat 页**:重放控制条(进度/倍速/预设双下拉)+ showcase-full 与 real-server 卷;完整对话从头到尾可看可玩。
- **P5 · markdown 页**:全类型轮播 + playground 合成流;预设旋钮;空态引导文案。
- **P6 · 收口**:pages smoke e2e + 本地 preview 走查(四页/移动端/Safari 降级提示)+ 体积记账 + DEPLOY-pages.md/README 更新 + 验收帧(四页 dpr=1/2)+ DoD 对账。

## 3. 客观判定

| 维度 | 判据 |
|---|---|
| 风格有据 | pages-theme.css 每个 token 在 lol-style/tokens.md 有出处行;免费字体选型记依据 |
| 版权红线 | grep/资产清单断言:站内零 Riot 字体/图像/音频;字体许可均免费可分发 |
| 四页可用 | preview 子路径版 smoke:加载/导航/canvas 有画(ink 断言)/控制条交互生效 |
| 降级完备 | 无 WebGPU(强制 WebGL2/都无)与 reduced-motion 两路降级各有 e2e 或人检记录 |
| 体积 | pages-assets ≤15MB;首屏(不含 wasm)增量 ≤2MB;wasm 体积不因本 plan 增长 |
| 不回退 | 五层门全绿;现有 golden 全集逐字节(pages 改动不触引擎) |

## 4. 铁律

- **引擎零改动优先**:本 plan 主战场在 web/;若 hero/控制条需要引擎新 API(如 film 场景参数),只 additive 且过 AR 清单,记账。
- **0→1**:导航/主题 token 一次定,不留新旧两套页面并存(旧 index 演示模式被新 IA 收编,`?dev` 保留开发 harness)。
- **确定性**:hero 循环用注入时钟场景(可 golden 定帧);smoke 截图走既有定帧惯例。
- **按 milestone commit,不 push**;资产入库前过体积预算;改 web 先 `Skill(bridge-write)`。

## 5. 变更边界

- **主战场**:`web/`(新页面/nav/pages-theme.css/pages-assets/vite 入口)、`.github/workflows/pages.yml`(增量)、`spec/reference/lol-style/`、DEPLOY-pages.md。
- **非目标域**:引擎 core/render 语义(hero 场景走既有 film/preset/效果 API);plan39 报告体系(资产导出复用其 harness,不改其 DoD);真实 server 模式(`?server` 保留不动)。
- **与 plan39 衔接**:pages-assets = feature harness 的精选导出(同一 DSL 同一确定性),清单在 manifest 加 `pages: true` 标记,单命令导出。

## 6. 风险与预案

| 风险 | 预案 |
|---|---|
| LoL 风格学过头 → 像皮肤站不像技术官网 | 参考包只取"气质层"(色/节奏/字距);信息密度保持工程站(功能卡片仍是内容主体);hero 短循环不喧宾 |
| CI 无 GPU 不能生成 pages-assets | 资产本地生成入库(≤15MB 预算);CI 只构建不捕获;README 记再生成命令 |
| hero 循环耗电/风扇 | 循环走引擎 idle 纪律(动画组 ≤4);tab 不可见暂停(visibilitychange);reduced-motion 静帧 |
| Safari WebGPU 缺失 | 既有 WebGL2 兜底自动走;hero 若 WebGL2 观感不达 → 定帧 + webm 降级路 |
| 单页体积膨胀 | webm 限段限码率;图片 dpr=1 为主;懒加载;预算表进 progress |

> 一句话:**用引擎自己当开场动画,用体检资产当功能画册,用重放当可玩 demo**——四页一个气质(Salvation 式庄重史诗),部署链不换,版权零风险。

Sources(风格参考): [Inven Global — 2026 For Demacia Cinematic "Salvation"](https://www.invenglobal.com/articles/20055/riot-games-reveals-2026-league-of-legends-for-demacia-season-cinematic-salvation) · [GameSpace — Season 2026 Cinematic](https://gamespace.com/all-articles/news/league-of-legends-check-out-2026-season-1-cinematic-salvation/) · [ArtStation — Salvation 美术](https://www.artstation.com/artwork/gRerlL)
