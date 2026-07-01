# 研究:可读性与稳定性的人眼生理学 / 心理学地基(Gestalt 及其它)

- 日期:2026-06-30
- 类型:research(感知科学 → 设计第一性原理 → ADR 映射)
- 触发:`streaming-chat-ux-standards.md` §7 定了"逐行 > 逐字"、§6 定了"morph 不 snap / 骨架先行"。这些是**taste + 工程**得出的;本篇回填它们背后的**人眼生理学 + 认知心理学**证据,让每条设计律有第一性原理背书,而非仅凭品味。
- 关联:0016(morph/NodeId)、0019(gate×choreography)、0005(turn 分组)、0029(冻结/工作集)、reveal-rhythm、md-reveal-cadence 北极星、streaming-chat-ux-standards §6/§7。
- 分级:🔵 直接指导本引擎参数/默认 · 🟡 参考

---

## 0. 一句话

"可读性"与"稳定性"两组关切各有成熟感知科学地基,且**恰好对上本引擎既有机制**:
- **可读性** = 阅读眼动(saccade/fixation/regression)+ 中央凹/旁中央凹 + 串行注意(E-Z Reader)→ 解释"为什么逐字/移动目标伤阅读、骨架先行助预览、冻结块给回视留锚"。
- **稳定性** = **物体文件(object files)+ 似动(apparent motion)+ Gestalt 组织律 + 变化盲/注意捕获 + 平滑追随/眩晕** → 解释"为什么 morph 不 snap、为什么布局跳变/raw 闪那么刺眼、reflow 速度要有上限"。

一句话:**你的 0016(morph+稳定 id)本质是在为大脑的"物体文件"保真;你的 0019(骨架先行)本质是在借 Gestalt 的闭合律;你要消灭的 raw 闪/跳变本质是"非自主注意捕获"。**

---

## 1. 可读性侧:阅读的眼动生理学

**眼动三件套 + 硬数字**(Rayner 等经典):阅读不是匀速扫描,而是 **fixation(注视,约 200–250ms)→ saccade(跳视,平均 6–8 字符)→ regression(回视,占阅读时间约 10–15%)**。regression = 主动往回看没看清/没理解的地方——**它要求"往回看的目标还在、且没动"**。

**中央凹 + 旁中央凹预览**:只有中央凹约 2° 是高清;**perceptual span 约 14–15 字符、不对称偏右**,且**文本越难 span 越窄**。旁中央凹(fovea 两侧约 5°)能**预览**下一个词,降低下一次注视成本。

**串行注意**(E-Z Reader / SWIFT):注意大体串行地逐词识别,**当前词的难度决定旁中央凹预览多少**。

**→ 对本引擎(§7 的生理根据):**
- **逐字/移动目标伤阅读**:逐字 = 退化 RSVP,压制 saccade/regression;更糟的是文字随生长 reflow,**"想回视的目标还在动"** → 违反 regression 的前提 → 认知负荷升。🔵
- **冻结块 = 给回视留稳定锚**:0029 的 settled/冻结前缀,正好让眼睛能安全回视已读内容(目标不再变)。这是块冻结的**可读性理由**(此前多讲性能)。🔵
- **骨架先行 = 喂旁中央凹预览**:0019 先出表格/列表骨架,等于给旁中央凹一个"接下来是什么结构"的预览锚,降低进入成本。🔵
- **短语/块揭示匹配 span**:一次揭示一个可被单次注视群消化的短语(≈一个 perceptual span 量级),而非逐 glyph。🔵

---

## 2. 稳定性侧一:物体文件 + 似动(morph 不 snap 的第一性原理)

**物体文件(Object Files;Kahneman, Treisman & Gibbs 1992)**:大脑为场景中每个实体维护一份 episodic 表征(object file),**靠时空连续性(spatiotemporal continuity)在时空中追踪"哪个去了哪"("which went where"),可在数秒尺度上保持身份**——**且身份主要由时空连续性决定,而非特征**。

**似动 / phi / beta(Wertheimer;Korte 定律)**:两个分离的闪现,若时空参数合适,被知觉为**同一物体在移动**,而非两次闪烁。

**→ 对本引擎(0016 的第一性原理):**
- **snap = 破坏时空连续性 → 大脑判定"旧物体消失、新物体出现" → 突兀**;**morph(translate/scale 补间)= 维持连续性 → 同一 object file 被更新为"移动了" → 顺滑**。这正是 0016"位移用 translate、尺寸用 scale、绝不 snap"的**知觉学证据**。🔵
- **NodeId `(block_seq, glyph_idx)` = 显式维护 object file 身份**:0016 靠 append-only 保证稳定 id,恰好对应"身份由时空连续性/位置序决定"。**你在软件里手动实现了大脑的对象对应机制**。🔵
- **补间时长要落在似动的"同一物体"窗口**:太快=闪(两次 onset),太慢=拖影;spawn/morph 时长应调到被读作"一个物体移动"的区间(与 §5 追随速度上限联合定界)。🟡

---

## 3. 稳定性侧二:Gestalt 组织律(骨架/分组/对齐的依据)

**Gestalt(Wertheimer / Koffka / Köhler):知觉是"统一整体"而非孤立部件。** 与本引擎相关的律:

| Gestalt 律 | 含义 | 本引擎用处 |
|---|---|---|
| **闭合(Closure)/ Prägnanz** | 缺了一部分,大脑也补全为完整图形 | **骨架先行(0019)**:先画表格网格/行框,大脑已把它读作"一个完整的表",再填字不突兀 🔵 |
| **共同命运(Common Fate)** | 一起运动的元素被归为一组 | 表格整行/整列 reflow 时**一起动**→被读作"这一行是一个单元";morph 应按结构单元整体位移 🔵 |
| **邻近(Proximity)/ 共同区域(Common Region)** | 靠近/同一容器内的元素成组 | **turn 分组(0005)**、消息气泡/缩进块 = 用间距与容器把一回合聚成一组 🔵 |
| **连续(Continuity)** | 排在一条线/曲线上的元素更相关 | 列表 bullet 对齐、表格列对齐(0014)= 沿垂直线成组 → 对齐差会破坏成组感 🔵 |
| **相似(Similarity)** | 同形/同色成组 | 角色配色(0021 数据驱动样式):同角色同色 = 一眼归组 🟡 |
| **图形-背景(Figure-Ground)** | 前景与背景分离 | 代码块/引用块的底 + 选区高亮画在文字下(0030)= figure/ground 分层 🟡 |

**→ 核心一条**:**骨架先行之所以有效,是因为闭合律**——用户看到网格骨架就已"脑补"出完整表格,填字是"往已存在的整体里补细节",而**逐行铺 raw `| a | b |` 是"从碎片拼整体",违反 Prägnanz** → 这是北极星"结构块不闪 raw"的知觉学根。🔵

---

## 4. 干扰的知觉根源:变化盲 + 注意捕获(为什么跳变/闪那么刺眼)

**注意捕获(Yantis & Jonides 1984/1988)**:**突现(abrupt onset)会非自主地捕获注意**,即使它与当前任务无关也照样打断;机制是**亮度/运动 transient**("It's all about the transient")。

**变化盲(Change Blindness)**:当变化伴随一个**全局 transient**(遮蔽了局部 transient)时,人**看不到**大变化;反之,一个孤立的局部 transient 会**强行拉走注意**。

**→ 对本引擎(§1/§2 稳定地基的知觉根):**
- **布局跳变、raw→结构 snap、auto-scroll 抖 = 制造了局部亮度/运动 transient → 非自主把眼睛从当前注视点拽走** → 读者"被决定"看哪里(正是 shadcn 第 1 条"绝不违背读者意图移动"的生理解释)。🔵
- **平滑 morph 之所以不打断**:连续渐变**不产生尖锐 transient** → 不触发捕获;这从反面证明"补间而非跳变"不仅美观,而且**不抢注意**。🔵
- **逐字打字机的隐忧**:每个字是一次微 onset;密集微 transient 在阅读区持续制造捕获 → 分散对语义的注意(RSVP 抬高认知负荷的机制之一)。故逐字要**降为装饰层 + 可关**(§7)。🔵

---

## 5. 运动的生理约束:平滑追随速度上限 + 眩晕

**平滑追随(Smooth Pursuit)**:眼睛能平滑跟踪移动目标,但**追随增益(gain≈0.9–1.0)只在目标速度 < ~20°/s 时保持;超过 ~40°/s 平滑追随崩溃,改用"catch-up saccade"(跳着追)**。

**眩晕 / vection**:**vection(自我运动错觉)主要由周边视野驱动**;大范围周边运动 → 不适/晕动。

**→ 对本引擎:**
- **给 0016 的 morph/reflow 一个速度上限**:若某次 reflow 让内容位移速度超过可平滑追随的区间(换算到视角),眼睛只能靠 catch-up saccade = 又变成"跳" → 前功尽弃。**结论:补间时长要保证峰值位移速度落在可追随区(约 <20–40°/s 等效),必要时宁可拉长补间。**🔵(给 0016 §8 的时长 policy 一个硬边界依据)
- **大位移别甩到周边视野**:长距离 reflow/滚动尽量发生在中央、或整体平移而非局部乱跳,避免触发 vection 不适。🟡
- **reduced-motion 的生理正当性**:对运动敏感人群,周边运动可致真实不适 → `prefers-reduced-motion` 命中就整块秒显(§1③/§7)。🔵

---

## 6. 汇总:感知研究 → 本引擎机制 → 设计律

| 感知研究 | 关键结论 | 本引擎机制/ADR | 设计律 |
|---|---|---|---|
| 阅读眼动(Rayner) | fixation 200–250ms、regression 占 10–15%、需稳定回视目标 | 块冻结(0029)、§7 | 冻结块给回视留锚;按短语揭示 |
| perceptual span / 旁中央凹 | span≈14–15 字符、能预览下一词 | 骨架先行(0019) | 骨架=旁中央凹预览锚 |
| 物体文件(Kahneman/Treisman 1992) | 身份靠时空连续性、维持数秒 | NodeId(0016 §4.1) | 维持稳定 id = 保真 object file |
| 似动 / Korte | 平滑分离闪现=同一物体移动 | morph(0016) | translate/scale 补间,绝不 snap |
| Gestalt 闭合 / Prägnanz | 大脑补全缺失为完整 | 骨架先行(0019) | 先骨架后填字(北极星) |
| Gestalt 共同命运/邻近 | 同动/邻近成组 | turn 分组(0005)、行整体 reflow | 结构单元整体位移;间距分组 |
| 注意捕获(Yantis) | 亮度/运动 transient 非自主夺注意 | 消灭 raw 闪/跳变、平滑滚动 | 稳定=不制造尖锐 transient |
| 变化盲 | 全局 transient 掩盖局部变化 | —— | 变更宁可渐变,别突现 |
| 平滑追随上限 ~20–40°/s | 超速→catch-up saccade(跳) | morph 时长 policy(0016 §8) | reflow 峰值速度设上限 |
| vection(周边驱动) | 周边大运动致眩晕 | reduced-motion、居中位移 | 大位移不甩周边;可关动效 |

---

## 7. 可落地(对本引擎的具体输入)

- 🔵 **0016 morph 时长加"可追随"硬约束**:补间参数不仅调"好看",还要让峰值位移速度落在平滑追随区;写进 0016 §8 policy 的边界条件。
- 🔵 **0019 默认 cadence 定为"短语/块 + 装饰逐字高亮"**,并把"骨架先行"标注为**闭合律**的应用(不是随意选择)。
- 🔵 **把"消灭 raw 闪/跳变"重述为"消灭非任务性 transient"**:验收时可量化——流式过程中不应出现读者注视区外的**突现/亮度跳变**(除有意的到达高亮)。
- 🟡 **调试面板加"感知参数"**:补间速度(对照追随上限)、揭示粒度(字/短语/块)、transient 强度——便于按感知边界调观感(并入 reveal 速度配置)。
- 🟡 **a11y 呼应**:运动敏感/reduced-motion 的生理正当性写进 0030,和读屏播报节奏并列为"感知无障碍"。

---

## 来源

- Gestalt 原理:<http://www.scholarpedia.org/article/Gestalt_principles> · IxDF <https://ixdf.org/literature/topics/gestalt-principles>
- 物体文件 / 物体持续:<https://perception.yale.edu/papers/10-Scholl-Flombaum-EncycOfPerc.pdf> · 时空连续性(PNAS)<https://www.pnas.org/doi/10.1073/pnas.0802525105> · 特征与空间时间共同引导持续(PMC)<https://pmc.ncbi.nlm.nih.gov/articles/PMC6999814/>
- 阅读眼动 / E-Z Reader / perceptual span:<https://www.sciencedirect.com/topics/psychology/e-z-reader-model> · 旁中央凹预览(PMC)<https://pmc.ncbi.nlm.nih.gov/articles/PMC6388670/> · Rayner BBS <https://web-archive.southampton.ac.uk/cogprints.org/1169/3/raynerbbs.pdf>
- 注意捕获 / 变化盲 / transient:非自主捕获(Springer)<https://link.springer.com/article/10.3758/BF03212254> · 新物体需感觉 transient 才捕获 <https://link.springer.com/article/10.3758/APP.72.5.1298> · 眼动捕获对比(JOV)<https://jov.arvojournals.org/article.aspx?articleid=2193180> · 注意捕获与非注意盲(TICS)<https://www.cell.com/ajhg/abstract/S1364-6613(00)01455-8>
- 平滑追随 / vection:Smooth Pursuit(ScienceDirect 概览)<https://www.sciencedirect.com/topics/medicine-and-dentistry/smooth-pursuit-eye-movement> · 眼动·vection·晕动(PubMed)<https://pubmed.ncbi.nlm.nih.gov/12793532/>
