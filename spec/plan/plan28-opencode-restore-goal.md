# Plan 28 · GOAL:以 opencode desktop 为基准,SDF 世界像素级还原 chat 页面(agent 直跑,无人介入)

- 日期:2026-07-07
- 状态:**GOAL(可执行任务书)** —— 替换 Plan 25 的主观 rubric 闭环;本文是给 Claude Code 的完整执行合同。
- 基准决策(作者拍板):**还原基准 = opencode desktop 的 chat 页面**(不考虑 Claude cowork)。还原后 = "GPU/SDF 版 opencode desktop"。
- 前置:[plan25-chat-page-design-goal](./plan25-chat-page-design-goal.md) + [plan25_progress](./plan25_progress.md)(机制/harness 全部复用;M0–M4 的 token/节奏/动效基建是本 GOAL 的地基)· [design/chat-page-design.md](../design/chat-page-design.md)(§3 节奏系统保留;§2/§4 的观感规格**被本 GOAL 的参考数值取代**)· [research/opencode-desktop-part-rendering-and-interaction](../research/opencode-desktop-part-rendering-and-interaction.md)(**参考实现源码地图,含 file:行号**)· [knowledge/opencode.md](../knowledge/opencode.md)
- **参考实现位置(已核实)**:opencode monorepo 本机 `~/w/agentscode/opencode`(UI = SolidJS,`packages/app` + `packages/ui`);若路径不存在 → `git clone https://github.com/anomalyco/opencode`(浅克隆)。**样式真值以 web UI(packages/app)为准**——desktop 即其壳,样式同源。

---

## 0. 为什么换路子(Plan 25 的教训,一条铁律)

Plan 25 的视觉闭环靠 agent 自评 rubric,taste 项自己给自己打分永远"达标",作者实看 `/chat` 判定不合格(卡片零内边距、块间无呼吸、色彩无层级)。**教训固化为本 GOAL 第一铁律:**

> **禁止任何"自评好看"。所有观感判定必须对照参考真值:样式数值对照(源码/computedStyle 抄来的精确数字)或参考截图并排 diff。没有参考依据的美学决策一律不做,存疑抄参考。**

## 1. GOAL 与完成定义(DoD)

**GOAL**:把 `/chat` 页面(showcase 剧本 + 实时对话同一渲染路径)还原成 opencode desktop chat 页面的观感——布局、间距、配色、字号层级、每类 part 组件的视觉结构与状态、以及其动效语汇;引擎能力(无边画布/虚拟化/morph/节奏调度)全部保留,只换观感层。

**DoD(全满足才算完)**:

1. **参考真值包**产出并入库:`spec/reference/opencode-ui/`(见 §3)——token sheet(JSON+MD)+ 全场景参考截图 + 组件盒模型注记。
2. 逐组件还原通过 §5 客观容差(布局盒 ≤4px 或 ≤3%、色 ΔE ≤5、字号/行高/圆角一致),并排对比图存档 `test/results/restore-diff/`。
3. `node test/run.mjs` 四层全绿;黄金帧(V1/V3–V7)按新观感重录且双跑一致。
4. Plan 25 的机制资产无回退:节奏三预设、动画组 ≤4、idle 全退场、确定性测试全保留全绿。
5. `spec/plan/plan28_progress.md` 逐 milestone 记录(做了什么/参考依据 file:行号/容差结果/遗留)。
6. **全程不 git commit / 不 push**;工作区即交付。

**启动**:`claude --permission-mode acceptEdits "读 spec/plan/plan28-opencode-restore-goal.md,按其执行到 DoD 全满足"`(中断重跑同句,读 progress 续跑)。

## 2. 总管线(两侧三步)

```
参考侧:起 opencode web UI + 固定场景会话 → Playwright 截图 + getComputedStyle dump
        → spec/reference/opencode-ui/(token sheet + 参考 PNG + 注记)
还原侧:token sheet 灌入 theme.rs/motion.rs/布局常量 → 逐组件改观感层
闭环:  同场景「参考 PNG ‖ SDF PNG」并排拼图 → §5 容差判定 → 不过则回改(抄数值优先)→ 过则下一组件
```

## 3. R0 · 参考真值捕获(第一个 milestone,产物进 git 工作区)

**跑起参考 UI**(按优先级尝试,任一成功即可):
1. `packages/app` dev server + 本地 `opencode serve`,**注入夹具会话**:优先向 opencode 的会话存储(其磁盘 session 存储格式,从源码 `packages/core` 确认路径)写入构造好的 session/message/part JSON,UI 打开即渲染历史——确定、无 LLM 依赖;
2. 不行则:真 server + API 建会话,再把 part 数据经其 API/存储回填;
3. 再不行(UI 起不动):**降级为"源码静态提取"**——样式全部从 `packages/ui` 源码(CSS/变量/组件)直读,截图参考改用 opencode 官网/README 现成截图,盒模型靠源码推导。progress 里记录降级理由。

**夹具场景 = 我方 design-review S1–S10 的同款对话**(user 提问 / 长 markdown+表格+代码 / thinking / read→edit(filediff)→bash 三态 tool / context 折叠组 / permission 卡 / compaction):两侧渲染同一份内容,diff 才有意义。视口统一(如 1280×900,dpr=1),**禁用参考侧动画后截 settled 帧**(`prefers-reduced-motion` 或 CSS 覆盖)+ 各组件状态帧。

**dump 内容(写进 `spec/reference/opencode-ui/`)**:
- `tokens.json`:对每个目标组件节点 dump `getComputedStyle`(padding/margin/gap/font-size/line-height/font-weight/color/background/border/border-radius/max-width)+ 全局 CSS 变量表(它的主题 palette);
- `NOTES.md`:每组件盒模型草图(嵌套结构 + 关键尺寸)+ 状态矩阵(tool pending/running/done/error 等各态的视觉差)+ 动效清单(transition 时长/easing、`PacedMarkdown` 24ms 批次、`TextShimmer`,标 file:行号);
- `shots/<scene>/<state>.png`:参考截图全集。
- **许可注记**:opencode MIT;style 数值与结构可抄,不拷贝其代码/资产文件本体(图标另说:形状用 SDF 自绘)。

## 4. 还原 milestones(R1–R5,每个跑 §2 闭环)

- **R1 全局 token 置换**:tokens.json 的 palette/字号层级/圆角/间距整表灌入 `theme.rs` + `motion.rs` + boxlayout 常量(列宽/边距按参考;HiDPI ×dpr 逻辑保留);删掉与参考冲突的自造观感(如头像 orb 列**若参考无此结构则整列移除**,等待/流式指示改按参考的 shimmer 形态)。验收:S9 整页并排图,列/底色/字号一眼同族。
- **R2 消息骨架**:user/assistant 消息的容器结构、对齐、meta 行(参考的时间/操作摆法)、turn 间距;滚动锚定语义不动。
- **R3 逐组件还原**(顺序按屏占比):text/markdown(含表格/代码块/引用)→ reasoning(Thinking 状态行 = turn 级合成 + shimmer,照调研 §0.1)→ tool 卡(通用 + bash/edit 特化,四态)→ context 折叠组("Gathered context" 卡)→ diff(filediff 呈现)→ permission/question 卡 → compaction 分隔。每组件:先抄 NOTES 盒模型/tokens 数值 → SDF 图元表达(panel/widget/glyph 现有能力;需加参数才加)→ 并排 diff → 容差过 → 下一个。**每组件迭代 ≤5 轮**,不过记遗留继续。
- **R4 动效对齐**:参考侧动效语汇为准绳(它有什么我们做什么:shimmer/渐显/展开过渡),映射到 0025/0028 机制;**我们的节奏系统(reader 预设/词拍/边界停顿)保留为增强层**——静帧必须与参考一致,动起来允许比参考更讲究。动画组 ≤4 / idle 退场断言保持。
- **R5 收口**:黄金帧全部重录(双跑一致)+ 四层门 + design-review 全场景重跑 + progress/DoD 对账;`design/chat-page-design.md` 头部加一行「§2/§4 观感规格由 plan26 参考真值取代,§3 节奏系统仍有效」。

## 5. 客观判定(替代 rubric 观感项;全部可脚本/可读图核对)

| 维度 | 容差 | 判法 |
|---|---|---|
| 布局盒(每组件 x/y/w/h、padding、gap) | ≤4px 或 ≤3%(取大) | 两侧同场景截图 + 参考 tokens 数值对照 |
| 颜色(bg/fg/border 每状态) | ΔE*ab ≤5(理想=直抄 hex) | tokens.json vs theme.rs 数值比对 |
| 字号/行高/字重层级 | 完全一致(px 级) | tokens vs 我方 role 表 |
| 圆角/描边宽 | ≤1px | 同上 |
| 结构完整(该有的元素都有、没有的别加) | 逐项清单 | NOTES.md 组件清单打勾 |
| 像素 diff | **不苛求**(字体光栅路径不同) | 并排拼图人可复核,agent 只判上述数值项 |

每组件产 `test/results/restore-diff/<component>.png`(左参考右 SDF 拼图)+ 一行结论进 progress。

## 6. 铁律(承 Plan 25,增补)

- §0 第一铁律:无参考依据的美学决策不做。
- **不 commit / 不 push**;版本轨迹靠 progress。
- CR1 / R8-R9 / 文字不进 fragment / 不引框架(0032)/ 几何不 snap / 测试结论只读 TESTREPORT / flaky 找根因。
- **引擎能力零回退**:虚拟化、morph、节奏调度、选区/复制/查找、a11y、`FrameStats` 断言——还原只动观感层;若参考结构与引擎机制冲突(如参考用 DOM 滚动区),以引擎机制为准、观感向参考靠。
- 参考代码 MIT:抄数值/结构,不整段搬其源码与资产。

## 7. 风险与预案

| 风险 | 预案 |
|---|---|
| `~/w/agentscode/opencode` 不存在/版本旧 | 浅克隆 GitHub 最新;调研文档行号漂移时按符号名重定位 |
| 参考 UI 起不动(bun/依赖/server 夹具难) | §3 降级路线 3(源码静态提取);记录降级 |
| 夹具会话格式与版本不符 | 回源 `packages/core` 会话存储/OpenAPI 重取(knowledge §7 流程),回填 knowledge |
| 字体不同导致行高/折行差异 | 对齐盒模型与字号数值,不对齐字形;折行差异不判负 |
| 参考是 DOM 滚动列表、我们是无边画布 | 视口内观感对齐;画布平移/缩放是超集能力,不视为偏差 |
| headless WebGPU 起不来 | 唯一允许求助人的情形(同 Plan 25) |

---

> 一句话:**参考真值包(R0)把"品味"变成"数字",还原循环把"设计"变成"对拍"。** agent 从 R0 开始顺序执行,每组件并排 diff 收敛,跑到 DoD 为止;引擎能力只增不减,节奏系统作为超越参考的增强层保留。
