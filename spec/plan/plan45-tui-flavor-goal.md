# Plan 45 · GOAL:TUI Flavor 对拍复刻 —— opencode TUI 观感包,先复刻再创新(agent 直跑,无人介入)

- 日期:2026-07-17
- 状态:**GOAL(可执行任务书)** —— 给 Claude Code 的完整执行合同,读完即可自主跑到 DoD。
- 执行方式:**顺序执行,master 工作区直跑**。当前 master = plan28–44 全量成果(五 crate / flavor 所需的 0021 ThemeTokens + 0033 渲染注册表 + 0037 装饰契约全在)。
- **作者战略(2026-07-17,需求澄清四问 + 两句原话)**:HTML 级样式细节太多 → 小目标先立:**复刻 opencode TUI 的 chat 对话**;「先复刻,再创新」;「不修改结构,基于现有能力,最多补充结构下的扩展组件」。四问确认:①落点 = 引擎 TUI 主题模式(flavor 可切);②保真度 = 对拍级;③范围 = 消息流全部件;④动效 = 保留节奏 + 像素布局(严格字符网格留 v2)。
- **架构不冲突声明(执行前提)**:TUI = 观感层第三消费者——ThemeTokens TUI 数据(0021)+ 0033 registry 新渲染器集 + 0037 装饰参数换发;**store/fsm/reveal/layout/morph 零改动**;additive 面仅 `set_render_flavor` setter + components TUI 渲染器 + web `?tui` 入口。
- 参考真值:**opencode TUI = TypeScript**(本机 `~/w/agentscode/opencode/packages/tui`:`src/theme/` 主题、`src/routes/session/dialog-message.tsx` 消息渲染、`src/component/`)——样式可直接源码提取;plan28 R0 参考包流程复用。

---

## 0. 铁律(三条)

1. **复刻期零创新**(作者原话「先复刻,再创新」):TUI 没有的(圆角/AO/glow/富动效/词级 diff 高亮…)在 tui flavor 下一律关;TUI 有什么做什么,**它的降级也照抄**(如它不渲表格就照它的显示法);存疑抄参考(plan28 第一铁律)。创新的去处 = rich flavor(现观感),切换即达。
2. **对拍不自评**:美学判定 = TUI 参考真值(源码数值 + 真 TUI 截屏)容差对拍(plan28 §5 同款:布局盒 ≤4px 或 ≤3%、色 ΔE≤5 理想直抄、字号行高一致);没有参考依据的决策不做。
3. **恒等硬门**:flavor 默认 rich,rich 路径零变化——既有 golden 全集逐字节(承 36/37/42/44 惯例);tui 是纯旁路。

## 1. GOAL 与完成定义(DoD)

**GOAL**:(A) TUI 参考真值包(源码提取 + 真 TUI 截屏);(B) `flavor=tui` 观感包:ThemeTokens TUI 版 + 全 mono role + 0033 TUI 渲染器集(消息流全部件)+ 0037 装饰 TUI 参数;(C) `set_render_flavor` 切换 + chat 页 `?tui`/面板下拉;(D) 逐组件对拍收敛。

**DoD(全满足才算完)**:

1. **参考真值包** `spec/reference/opencode-tui/`:tokens(theme 色板→sRGB 钉死 / 字号行高 / 缩进与间距规则 / 边框语汇:box-drawing 字符还是线,照实提取)+ 逐部件 NOTES(text/markdown 渲法、tool 卡行式结构、diff +/- 形态、thinking、权限 ask、回合分隔的**状态矩阵**,标 file:行号)+ 真 TUI 截屏全场景(起 server 跑真 TUI;起不动 → 降级源码提取 + 记账,同 plan28 R0 预案)。
2. **主题层**:`themes/tui.json`(ThemeTokens 运行时加载,0021 既有通道)——色板 + 全 mono 字体 role 映射(mono 路现成);**零 Rust 改动**验证(纯数据换肤先跑通一版粗对拍)。
3. **渲染器层**:components crate 注册 **TUI 渲染器集**(0033 registry;`register_flavor("tui", …)` 或等价机制,additive):text/markdown TUI 渲法(照参考:强调/缩进/代码块/列表/引用…含它的表格处理方式)、tool 行式卡、diff 纯 +/- 行(无词级/无 gutter/无 gloop——除非参考有)、thinking、权限 ask、回合分隔。装饰经 0037 换 TUI 参数(直角/1px 或字符边框,R0 定)。
4. **切换面**:core/wasm `set_render_flavor("tui"|"rich")`(未知名拒,AR12;切换使块缓存失效重渲);chat 页 `?tui` 参数 + 面板「Flavor」下拉;**切换是运行时的**(同内容两套皮实时切,这本身就是最好的 demo)。
5. **动效档**:tui flavor 下 rhythm 预设默认 **typewriter**(最贴终端)、效果预设默认 **off**(TUI 无 glow/dissolve);节奏保留可调(引擎的魂不丢),但对拍判据只看静帧。
6. **对拍收敛**:逐部件「参考 PNG ‖ 引擎 PNG」并排图存 `test/results/tui-diff/`,§0-2 容差判定;每部件迭代 ≤5 轮,不过记遗留继续(plan28 惯例)。
7. **门**:五层门全绿;rich 恒等硬门(golden 逐字节);tui flavor 新 golden 家族(全场景静帧,dpr=1/2);native ≥4(flavor 切换/未知名拒/TUI 渲染器快照/theme 数据快照);e2e ≥2(?tui 生效 + 运行时切换重渲)。
8. `spec/plan/plan45_progress.md` 记账(参考依据 file:行号 / 容差结果 / 遗留);**按 milestone commit,不 push**。

**启动**:`claude --permission-mode acceptEdits "读 spec/plan/plan45-tui-flavor-goal.md,按其执行到 DoD 全满足"`(中断重跑同句,读 progress 续跑)。

## 2. Milestones

- **U0 · 参考真值包**:跑真 TUI 截屏(server 起法承 plan35 T3 经验;S1-S10 同款场景喂给它)+ 源码提取(theme/dialog-message/组件逐个,file:行号)→ tokens + NOTES + shots;**边框语汇/表格渲法/ANSI→sRGB 三个关键判定在此钉死**。单独 commit(对拍真值 = 一切美学决策依据)。
- **U1 · 主题层粗对拍**:tui.json(色板+mono role)运行时加载 → chat 页 `?tui` 只换 theme 先看整体色/字对不对(渲染器还是 rich 的)→ 粗对拍记差距清单(哪些必须靠渲染器层解决)。
- **U2 · 渲染器集全部件**:flavor 注册机制(0033 上 additive,机制方案记 progress——若 registry 需小扩展,过 AR 清单)→ 逐部件 TUI 渲染器(text/md → tool → diff → thinking → ask → 分隔)+ 0037 装饰 TUI 参数;每部件完成即并排对拍一轮。
- **U3 · 对拍收敛**:全场景并排图 → 容差判定 → 回改(抄数值优先)→ 收敛;遗留清单记账。
- **U4 · 收口**:切换面(setter/URL/面板)+ 动效档默认(typewriter+off)+ golden 双 flavor 家族 + 五层门 + DoD 对账;README/chat 页说明一行(「TUI ⇄ Rich 一键切」——这本身是卖点)。

## 3. 客观判定

| 维度 | 判据 |
|---|---|
| 对拍 | 逐部件布局盒 ≤4px 或 ≤3%;色 ΔE≤5(理想直抄 hex);字号/行高一致;并排图入库 |
| rich 恒等 | flavor=rich:golden 全集逐字节(硬门) |
| 切换 | 运行时切 flavor → 同内容重渲两套皮(e2e);未知名整拒 |
| 零创新 | tui flavor 下 grep/断言:圆角=0、AO/glow 参数=0、效果预设=off 默认;TUI 没有的部件形态不出现 |
| 零结构改动 | 机制层(store/fsm/reveal/layout/morph)diff 为空(git diff 断言;flavor 面除外) |
| 门 | 五层门全绿;tui golden 家族 dpr=1/2 双跑一致 |

## 4. 铁律(承 AGENTS.md / dev-practices §4)

- **§0 三条**(零创新 / 对拍不自评 / rich 恒等)。
- **观感=数据**:theme 是 JSON、渲染器是注册表条目、装饰是参数——TUI 包整体可拆卸。
- **AR12** 未知 flavor/坏 theme 整拒 · **R8** 内容与对拍场景写死 · **CR1** TUI 渲染器纯函数(components,native 可测)。
- 参考代码许可:opencode MIT——数值/结构可抄,不搬源码本体(plan28 同款注记)。
- **按 milestone commit,不 push**;改 components 走既有纯函数纪律;改 wasm additive。

## 5. 变更边界

- **主战场**:`spec/reference/opencode-tui/`(新)、`web/public/themes/tui.json`、`components`(TUI 渲染器集 + registry flavor 面)、`primitives/theme` 若需字段(additive)、wasm `set_render_flavor`、web chat 页 `?tui`/面板。
- **非目标域**:引擎机制层全部(§3 零结构改动断言)、首页/组件页/gallery(TUI flavor 对它们可用但不主动接,记后续)、严格字符网格布局(v2)、TUI 输入框/状态栏(范围外,消息流 only)。
- **与后续的关系**:「TUI→HTML 渐进丰富」= 在 flavor 机制上迭代 rich(或新增 flavor),本 GOAL 把通道修好。

## 6. 风险与预案

| 风险 | 预案 |
|---|---|
| 真 TUI 截屏起不动(server/终端录制) | 降级源码静态提取 + 终端手动截屏若干关键帧;记账(plan28 R0 预案同款) |
| TUI markdown/表格渲法比预期简陋 | **照抄简陋**(§0-1);差距记录进 NOTES,不脑补 |
| flavor 注册机制需动 registry 结构 | 优先 additive(双 registry 或条目带 flavor 键);确需改 0033 接口 → 小 ADR + 旧行为等价测试 |
| ANSI 色在暗色终端外观依赖终端主题 | 以 opencode TUI 自带 theme 的确定色值为准(源码提取),不模拟终端差异 |
| mono 下 CJK 宽度与终端双宽字符规则差异 | R0 记录 TUI 的 CJK 处理;引擎 mono 路按其度量对拍,差异记容差豁免 |
| 切换重渲成本 | 块缓存失效一次性重排(同改 theme/字体既有路径);长会话下测一次数值记账 |

> 一句话:**给引擎穿上第一件"别人的衣服"——TUI flavor 用一份 JSON + 一套注册渲染器 + 一组装饰参数对拍复刻 opencode TUI,机制层零改动**;复刻立住,"再创新"就是 flavor 通道上的自由发挥,而一键 TUI⇄Rich 切换本身就是引擎最硬的 demo。
