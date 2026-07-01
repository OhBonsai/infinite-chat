# 研究:流式对话 UX 的"美学/正确性标准" —— shadcn 15 条的同类工作与本引擎对应

- 日期:2026-06-30
- 类型:research(标准梳理 + 与 ADR 对应)
- 触发:`chat-ui-libs-ai-elements-shadcn.md` §2.5 摘录了 shadcn `MessageScroller` 提出的"优秀流式滚动 15 条"。本篇回答:**这套"chat 美学标准"是不是孤例?业界/学界有无同类成文工作?** 结论:不是孤例——它是过去一年"streaming UX"写作潮里**最完整的一次成文**;同类材料按"与它的重合度"分三档,多数覆盖"滚动三态 + 布局稳定 + a11y",但 shadcn 在**锚定/重开位置/permalink 跳转**上仍领先、少有别处成文。
- 关联既有 research:本篇补的是"**滚动意图 + 布局稳定 + a11y**"这条线;流式平滑/cadence 一面见 [industry-llm-chat-rendering](./industry-llm-chat-rendering.md)、[reveal-rhythm](./reveal-rhythm.md);agent 设计语言/学界 JIT pacing 见 [agent-ui-industry-survey](./agent-ui-industry-survey.md)。
- **感知地基(本篇 §6/§7 的第一性原理来源)**:本篇给出"怎么做"的标准与地基;**为什么这样做**(人眼生理学 + 认知心理学:阅读眼动 / 物体文件 / Gestalt / 注意捕获 / 平滑追随)见 [perception-readability-stability](./perception-readability-stability.md)。两篇配套阅读。
- 分级:🔵 直接可抄进本引擎 · 🟡 值得排期 · 🔴 边界/仅参考

---

## 0. 一句话结论

"chat 的美学标准"确有一批同类工作,但**没有单一权威**——它是由**组件库(shadcn / AI Elements / ElevenLabs UI / prompt-kit)、工程库(use-stick-to-bottom)、craft 长文(Smashing)、可用性启发式(NN/g)、可量化 Web 标准(CLS)、学界(CHI JIT pacing)** 共同拼出的。它们高度收敛于三块地基:**① 滚动三态机 · ② 布局稳定 · ③ 流式 a11y(live region + reduced-motion)**。shadcn 的独到增量是把 ①/② 推到**"回合锚定/重开位置/可寻址跳转"**这一层。

---

## 1. 三块收敛地基(几乎人人都讲)

### ① 滚动三态机(anchored / drifted-up / jumped-back)

- **贴底(anchored)**:仅当读者已在底边时,新 token 到达才跟随。
- **上滚脱离(drifted-up)**:任何"读者意图"信号出现即释放跟随——滚轮/触摸/键盘滚动键/拖滚动条,**以及**选中文本、打开链接、搜索。
- **点回底(jumped-back)**:显式"jump to latest" pill 才重新进入跟随。
- 反复被描述为"every well-tuned chat UI 都这么做"。**60–100px 的底部阈值**是公认防抖细节(布局微变不误判为用户上滚);**新 stream 开始要重置 `userScrolled`**,否则上一条的一次上滚会静默关掉下一条的自动跟随。
- → 本引擎对应:**这正是我报告 R10 建议的"滚动跟随 FSM"**,应在相机层实现 `Following/Released/Anchoring` + 释放信号集合。🔵

### ② 布局稳定(streaming 内容长大不得推走读者正在看的)

- 三大症状:**scroll 抢位、layout shift、render frequency 过高**(流比 60fps 快,狂刷 DOM/主线程)。
- 通解:**只写变化的节点、每帧 flush 一次(rAF 批处理)**、内容上方变化时保持读者位置。
- 有**可量化的官方标准**:Web Vitals 的 **CLS(Cumulative Layout Shift)** 就是"布局稳定性"的正式度量;浏览器还有 **CSS `overflow-anchor`(Scroll Anchoring)** 原语(Safari 不支持,故 use-stick-to-bottom 不依赖它,自己算)。
- → 本引擎对应:0016(几何变化用 translate/scale 补间不 snap)、0017(提交前沿/活跃块)、0029(工作集)已从**渲染机制**侧覆盖;本引擎因 GPU retained scene + 块冻结,天然规避 DOM 的"每 token 重排"。**"每帧一次跨界"= rAF 批处理的引擎版**(0016 AR10)。**交叉验证通过**,无需改。🔵(作为验证)

### ③ 流式 a11y(动态内容的无障碍地基)

- **live region**:容器 `role="log"` + `aria-live="polite"` + `aria-atomic="false"`(只播报新增、别整条重读)。这是"读屏能听到流式内容却不被逐 token 噪音淹没"的标准写法。
- **prefers-reduced-motion**:命中就**跳过打字机、整段一次性渲染**;闪烁光标也要停(闪烁属 WCAG flashing content)。
- **键盘/焦点**:stop 按钮必须可 Tab 可达;隐藏用 `display:none`(移出 tab 序)而非 `opacity:0`/`visibility:hidden`(仍可聚焦=陷阱);`:focus-visible` 做焦点环;装饰光标 `aria-hidden="true"`。
- → 本引擎对应:**直接补强 0030 的 DOM 镜像**——0030 主攻选区/复制,对"流式播报节奏"着墨少。把 `role=log`/`aria-atomic=false`/`aria-busy`(流式时)/`prefers-reduced-motion→关闭 spawn 动画整帧显示 加入 0030 规范,消除"canvas 文字读屏逐字念"这一最常见翻车点。🔵(= 报告 R13,证据加强)

---

## 2. 同类工作三档(按与 shadcn 15 条的重合度)

### 第一档 · 几乎同一份标准,切入角度不同

| 来源 | 形态 | 独到点(相对 shadcn) |
|---|---|---|
| **Smashing《Designing Stable Interfaces for Streaming Content》**(Joas Pambou, 2026-04) | craft 长文 + 可跑 demo | **以 a11y 为骨架**组织全篇;把"scroll/layout-shift/render-freq"三症状讲透;**中断处理**成体系(stop 要清 buffer/去光标/标记未完/给 retry/新消息中途发要先停旧流) |
| **`use-stick-to-bottom`**(StackBlitz Labs;AI Elements `Conversation` 的真库) | 工程库 + rationale | **velocity-based spring 而非 easing**(变长内容更稳);不靠 debounce 区分"用户滚 vs 程序滚";ResizeObserver 抓增/缩;**不依赖 `overflow-anchor`**(自算,绕开 Safari) |

### 第二档 · 口径化的"三态模型"与反面证据

| 来源 | 价值 |
|---|---|
| [Intuitive Scrolling for Chatbot Streaming](https://tuffstuff9.hashnode.dev/intuitive-scrolling-for-chatbot-message-streaming) / [Streaming UI Patterns That Don't Break](https://thepromptbench.com/ai-product-ux/streaming-ui-patterns-that-dont-break/) | 把三态机讲成"人人该做"的默认;IntersectionObserver 底部 anchor 元素做法 |
| bug 线程:[py-shiny #1988](https://github.com/posit-dev/py-shiny/issues/1988) · [hermes-webui #677](https://github.com/nesquena/hermes-webui/issues/677) · [GitNexus #765](https://github.com/abhigyanpatwari/GitNexus/issues/765) | **反面证据**:大量产品栽在同一个"流式抢滚动位置"上 → 证明这些规则是真需求,不是洁癖 |
| 其他组件库:[ElevenLabs UI Conversation](https://ui.elevenlabs.io/docs/components/conversation) · [prompt-kit Chat Container](https://www.prompt-kit.com/docs/chat-container) | 各自把"贴底/滚动按钮/isAtBottom context"固化成组件 API,与 AI Elements `Conversation` 同构 |

### 第三档 · 更上位的"对话美学/可信度"标准(非滚动层)

| 来源 | 主张 |
|---|---|
| [NN/g 启发式 × 对话式 AI](https://medium.com/design-bootcamp/nielsens-heuristics-revisited-for-conversational-ai-90e1c613ce05) | 把 Nielsen 十大可用性启发式重述到 chat:透明披露用 AI、不确定时给 **2–4 个 scoped 选项**而非瞎猜、可见的不确定性比隐藏的模糊更不伤信任 |
| **Vitaly Friedman《Design Patterns for AI Interfaces》**(Smart Interface Design Patterns) | 目前最体系化的付费 pattern 库:progressive disclosure of reasoning、citation、confidence 指示、feedback 捕获 |
| [Thinking Machines Lab《Interaction Models》](https://thinkingmachines.ai/blog/interaction-models/) | 上位框架:**实时在场的 interaction model + 后台异步 reasoning model**;后台跑长推理,前台保持在场并把结果织回对话 → 对本引擎 reasoning/长任务流式有结构指导 |
| Web Vitals **CLS** | "布局稳定性"的**可量化官方标准**;shadcn 第 7/12 条本质=把 CLS 降到对话场景 |
| 学界:**CHI 2026 JIT pacing**(见 agent-ui-industry-survey §0.4) | boundary-aligned pacing 作为一等调度目标 = 本引擎 reveal cadence(0019)的学术 peer |

---

## 3. shadcn 真正领先、少有别处成文的部分(→ 优先抄)

多数同类只到"滚动三态 + 布局稳定 + a11y"。**以下几条 shadcn 讲得最细,别处罕见成文,建议 R10 直接抄:**

1. **新回合锚到视口近顶部 + 保留上一条 peek**(`scrollPreviousItemPeek`,默认 ~64px):新回合"长进屏幕"而非把旧内容推走,且与上文视觉相连。🔵
2. **重开已存会话停在 `last-anchor`(最后一个有意义回合,通常是用户最后一句),不是绝对底部**;无锚回退 end。🔵
3. **可寻址跳转 `scrollToMessage`:可在 row 挂载前排队**(覆盖 permalink 先到),挂载后缺失返回 `false` **不进猜测式重试循环**。🟡
4. **中断/重试/重生成/分支/错误都不得偷移位置**(把"稳定"从流式扩展到全部状态变更)。🔵
5. **可见性追踪 pay-for-what-you-use**:无人订阅=零成本(本引擎 `?debug` 可见块上报同款惰性)。🟡

> 这几条本质是把 §1① 的"滚动三态"从"贴底/脱离"升级为**"以回合(turn)为单位的锚定与寻址"**——正好契合本引擎 0005(Turn 聚合)已有的 turn 概念:**anchor = turn 边界**,与 shadcn"scrollAnchor 不绑 role、绑 turn 起点"完全一致。

---

## 4. 与本引擎 ADR 的对应总表

| 标准条目 | 同类来源 | 本引擎对应 | 动作 |
|---|---|---|---|
| 滚动三态机 + 释放信号(含选区/链接/键盘) | use-stick-to-bottom / Smashing / 三态诸文 | 无专门 ADR(散在相机/输入) | 🔵 新增"滚动跟随 FSM"(报告 R10),并入 0031 |
| 布局稳定 / 不 snap / rAF 批处理 | Smashing / CLS / overflow-anchor | 0016 / 0017 / 0029(机制侧已覆盖) | 🔵 作为验证,保持 |
| live region + aria-atomic=false + reduced-motion | Smashing / NN/g a11y | 0030(偏选区/复制) | 🔵 补播报规范(报告 R13) |
| 回合锚定 near-top + peek | shadcn(独到) | 0005 Turn 聚合(有 turn 概念) | 🔵 anchor=turn 边界,报告 R10 |
| 重开停 last-anchor | shadcn(独到) | 0003 重新水化 / 0029 | 🔵 打开位置策略 |
| permalink/jump 可排队 | shadcn(独到) | 无 | 🟡 排期 |
| 中断/重试不偷位 | shadcn / Smashing 中断处理 | 0005 中断插分隔 / 0031 resilience | 🔵 纳入 FSM 用例 |
| reasoning 渐进披露 + 完成折叠 | AI Elements / Vitaly / 第三档 | 0006 thinking / 0019 编排 | 🟡 报告 R3 |
| 可量化验收(CLS 思想) | Web Vitals | testing-and-benchmark | 🟡 加"锚定/保位"确定性重放用例 |

---

## 5. 边界(不照搬)

- 🔴 **CLS/overflow-anchor 是 DOM 世界度量/原语**,本引擎不用其实现,只借"布局稳定是可量化目标"这一观念,验收走 native 确定性重放。
- 🔴 **打字机→reduced-motion 整段秒显**在 DOM 是"关动画";本引擎等价为"gate 全开、spawn 动画时长=0 瞬显"(0019 一行配置),但**仍走同一渲染路径**,不做两套。
- 🔴 第三档(NN/g、Vitaly、Interaction Models)偏**对话/产品设计**,非渲染引擎职责;作为外围控件(输入框/审批/披露)设计参考,不进 wasm 引擎契约。

## 6. 补充地基 ④:结构确定性 —— markdown 流式 ≠ text 流式

§1–§3 的三块地基**默默假设内容模型是"纯文本:只追加、过去不变"**。但 LLM 吐的是 **markdown**,它打破纯文本流唯一的不变量("尾部追加,已到达字符的几何/语义永不变"),两种方式:

- **回溯重解释**:后来的 token 改写前面已到达内容的**语义 + 几何**(`**foo` 闭合才变粗且变宽;一行文字下一行 `|---|` 回头变表头;`[label]` 到 `(url)` 才成链接)。
- **结构歧义**:一个前缀可能是 N 种东西(`| a | b` 是表格还是带竖线文本?半开 ` ``` ` 之后整条尾巴都不是 markdown,直到闭合围栏)。

**确定性的形式判据 = chunk-boundary independence**:渲染结果必须是「累计文本」的纯函数,**与网络在哪切 chunk、以何时序到达无关**。危险做法是把增量 chunk 喂给有状态流式解析器(输出会因切点不同而变)。本引擎 0017 已绕开:**每 tick 拿活跃块累计原文整块重解析(pulldown 原样)**,输入不含 chunk 边界信息 → 天然确定。此点应从实现细节**提升为显式不变量**。

**两层机制**(本引擎各已有):① "已冻结语义会不会被后来 token 改" → **提交前沿**(最后未闭合块之前全冻结);② "活跃块半截语法怎么不闪 raw" → **gate + 骨架(0019)/ remend 愈合**。外部印证:Streamdown 的 `remend` = 进解析器前一条按优先级排的 fixup 链补全未闭合 `**`/`[t(`/`$`,自陈 "deterministic AST transform";opencode = `remend healing(linkMode:text-only)+ marked 重解析`。本引擎多一层 0016 逐字补间。

**逐构造门控表(揭示成结构的最粗安全粒度):**

| 构造 | 歧义窗口 | 门控粒度 | 确定性 | 容错 / 回退 |
|---|---|---|---|---|
| 超链接 `[t](url)` | 局部有界 | `)` 闭合;label 文字稳定先揭示,装饰+href 延后 | 活跃块整块重解析 | 半截 → 仅显 label;流末未闭 → CommonMark 字面(确定) |
| 引用链接 `[t][id]` | **跨块无界** | 先当文本(opencode text-only 同解) | 定义 append-only 单调 | 定义永不到 → 永久字面;到了 → 一次性非动画 restyle(0017 §7) |
| list | 项内有界;松/紧+嵌套待后续空行 | **逐项**(item 行完成);marker 行首即知 → 骨架 bullet 先出 | 项文本行完即 settle;松/紧留活跃区 | 松→紧=间距 morph(非 snap);中途断 remend 收尾;序号由位置定 |
| 表格 | **多行回溯**:到分隔行才判定 | **整表门**:分隔行确认 → 骨架网格→表头→cell;`is_pending_table` 已 hold | 列宽=0014 两趟(对当前行确定);后到行撑宽→morph | **投机代价**:hold 首行=赌是表;非表 → **一行内确定性回退为文本** |
| 代码围栏 | 半开 ` ``` ` 后整条尾巴 | 围栏行确认即进代码块 | 活跃块重解析 | 流末未闭 → remend 补闭合围栏 |

**规律**:纯文本按字揭示;markdown 必须按「每构造最粗安全粒度」揭示 —— 行内按"闭合"、list 按"项"、table 按"分隔行确认后逐格"、code 按"围栏内行"。这即 0019 的 gate 轴。

**④ 结构确定性 / 语义提交单调(新增地基)—— 不变量四则:**

1. **冻结即定**:已提交块的语义永不翻转,只有活跃区可变(提交前沿)。
2. **完备才揭示**:结构块到完备阈值(gate)才现,用户永不见 raw→结构翻转。
3. **投机必有界回退**:任何乐观投机(hold pending-table)都配确定性、有界的回退(非表则一行内退成文本)。
4. **延后解析单调只增**:reference link / 松紧 / 列宽等只前进(high-water mark),不撤回已显示结构。

同时给原 ②③ 打 markdown 补丁:

- **②布局稳定** 泛化:markdown 版不是"过去绝不动",而是"**过去只在活跃区内、以 morph(非 snap)动**"(0016)。列宽增长、`**`闭合变宽是合法的"稳定"——只要不跳。
- **③a11y** 按**语义粒度而非语法粒度播报**:别让读屏念完 `| a | b |` 再念"table 2 columns";活跃块 gate 期间 `aria-busy`,settle 后按块播报一次。缝合 ③↔④。

**本引擎缺的可落地项**(方向 0017+0019 已有):
- 🔵 补全 0017 §10 / 0019 §9 的**逐构造门控表**(list/code/link/math 的 raw 抑制 + 骨架;table 已 ✅)。
- 🔵 把"**投机 + 有界确定性回退**"从 table 特例(`is_pending_table`)**提成通则不变量**写进 ADR。
- 🔵 加**切点 fuzz 测试**:同段 markdown 在每个字节边界切一刀喂入,断言最终渲染 + committed 块序列完全一致(proptest 化,补在 c01–c10 之外)= 坐实 chunk-boundary independence。

---

## 7. 补充地基 ⑤:揭示粒度与可读性 —— 逐行 vs 逐字

**问题**:字幕/流式内容是**逐行(块/短语)出**还是**逐字出**?哪个更可读?这是"揭示粒度"对可读性的直接影响,决定本引擎 cadence 默认(0019 / reveal-rhythm / 北极星)。

**证据(字幕学 + 阅读科学)结论一致:逐行/按短语 = 更可读;逐字 = 更"有生命感"但更累。**

- **逐行/按短语(block)**:让眼睛做它本来会做的事——**saccade(跳视)+ fixation(注视)+ regression(回视)**,一眼一个块读完("read in a single pass"),读完可移开再回来。理想字幕块 3–7 词、按句法边界断行,认知负荷最低。
- **逐字(typewriter)= 退化版 RSVP**:RSVP(定点逐词呈现)**压制了 saccade/fixation/regression**。而**约 10–15% 的阅读时间本是 regression**(回看没看清/没理解的地方)——逐字揭示让"想回看的目标还没稳定/还在动",**强制串行摄入 + 抑制眼动 → 反而抬高认知负荷**;高速逐词呈现已被证据显示**损害理解**(母语/非母语同此)。
- **更糟的耦合(本引擎要警惕)**:AI chat 的逐字**不只是 RSVP,还叠加"目标在动"**——文字随生长不断 reflow。逐字揭示 = 强制串行摄入 **+** 移动的阅读目标,两个坏处叠加。逐行则给一个**稳定的块**先读、再 settle。

**为何逐字仍流行**:它传达**"活着/正在生成"的生命感与节奏**,在短视频(TikTok 词级字幕)是**参与感/风格**装置——但那是**到达动画**,不是**阅读模式**。

**设计综合(本引擎可两全)**:字幕学给的最优解是 **block 呈现(可读)+ 词级 highlight(karaoke,有生命感)** 分离。映射到本引擎:**揭示一个 settled 的行/块(稳定、可读),同时用 GPU `spawn_time`/高亮传达进度与生命感** → 拿到生命感**而不牺牲稳定阅读目标**。这正是把"实时性"降级为**装饰层**、"可读性"升为**内容层**,与北极星(阅读体验 > 实时性)、0019(gate=内容层 / choreography=装饰层)完全一致。

**⑤ 揭示粒度地基 —— 三则:**

1. **内容按短语/块揭示,不按字**:结构块尤甚(表格/列表/代码);纯文本可退到"短语/行"而非逐 glyph。给回视留稳定目标。
2. **逐字仅作装饰层**:`spawn_time` 淡入/高亮传生命感,叠加在**稳定块**上,不作为唯一揭示手段;速度可调、可关。
3. **reduced-motion → 整块秒显**:命中偏好直接跳过逐字动画、整块呈现(与 §1③ 一致)。

→ 落地:reveal-rhythm 的 groove/phrase-final lengthening 应作用在**短语/块边界**(与此地基同源);0019 调度器默认 cadence 从"逐帧逐字"改为"**按短语/块 + 装饰性逐字高亮**"。🔵(呼应 0017 §10 未竟项)

---

## 来源

- Smashing《Designing Stable Interfaces for Streaming Content》:<https://www.smashingmagazine.com/2026/04/designing-stable-interfaces-streaming-content/>
- use-stick-to-bottom(GitHub / npm):<https://github.com/stackblitz-labs/use-stick-to-bottom> · <https://www.npmjs.com/package/use-stick-to-bottom>
- 三态模型长文:<https://tuffstuff9.hashnode.dev/intuitive-scrolling-for-chatbot-message-streaming> · <https://thepromptbench.com/ai-product-ux/streaming-ui-patterns-that-dont-break/>
- 反面证据 issue:<https://github.com/posit-dev/py-shiny/issues/1988> · <https://github.com/nesquena/hermes-webui/issues/677> · <https://github.com/abhigyanpatwari/GitNexus/issues/765>
- 其他组件库:<https://ui.elevenlabs.io/docs/components/conversation> · <https://www.prompt-kit.com/docs/chat-container>
- NN/g 启发式 × 对话式 AI:<https://medium.com/design-bootcamp/nielsens-heuristics-revisited-for-conversational-ai-90e1c613ce05>
- Thinking Machines Lab《Interaction Models》:<https://thinkingmachines.ai/blog/interaction-models/>
- shadcn MessageScroller(15 条原文):<https://ui.shadcn.com/docs/components/message-scroller>
- CSS Scroll Anchoring / CLS:<https://developer.mozilla.org/en-US/docs/Web/CSS/overflow-anchor> · <https://web.dev/articles/cls>
- markdown 流式(§6):Streamdown Unterminated Block Parsing <https://streamdown.ai/docs/termination> · Streamdown remend <https://mintlify.wiki/vercel/streamdown/api/remend> · Preventing Flash of Incomplete Markdown(HN)<https://news.ycombinator.com/item?id=44182941> · 本仓库 `spec/decision/0017`·`0019`·`0014`
- 揭示粒度/可读性(§7):字幕分段与可读性 <https://www.ncbi.nlm.nih.gov/pmc/articles/PMC7901653/> · 字幕格式的认知负荷 eye-tracking <https://www.tandfonline.com/doi/full/10.1080/1475939X.2024.2433259> · RSVP 与理解降级 <https://www.researchgate.net/publication/328925418_Rapid_serial_visual_presentation_Degradation_of_inferential_reading_comprehension_as_a_function_of_speed> · 字幕分段标准 <https://subtitling.net/standards/subtitle-segmentation>
