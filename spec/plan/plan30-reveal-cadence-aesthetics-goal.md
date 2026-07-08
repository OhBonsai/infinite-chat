# Plan 30 · GOAL:兑现北极星「阅读体验优先」——rap-flow 节奏函数 + 全类型揭示策略(agent 直跑,无人介入)

- 日期:2026-07-08
- 状态:**GOAL(可执行任务书)** —— 给 Claude Code 的完整执行合同,读完即可自主跑到 DoD。
- 优先级:**P1**(北极星正身)。作者在 [design/thinking.md §3](../design/thinking.md)明写**极端想要**:「阅读体验 > 实时性,揭示节奏由我们掌控,不被 token 到达率牵着走」。plan8 只落了 v1 简约子集,节奏函数本体 + 全类型骨架先行还没上。
- 前置:
  - [design/thinking.md §1/§3/§5](../design/thinking.md)(元素揭示策略 / 节奏自主 / rap-flow 美感——本 GOAL 的取向真值)
  - [decision/0019-reveal-gating-and-choreography](../decision/0019-reveal-gating-and-choreography.md)(门控 × 编排,§158-165 待办即本 GOAL)· [0017-markdown-streaming-landing](../decision/0017-markdown-streaming-landing.md)(§128-132 揭示调度器待办)· [0020-content-node-identity-model](../decision/0020-content-node-identity-model.md)(节点树 = Selector 寻址底座,已落地)
  - [plan8-reveal-cadence](./plan8-reveal-cadence.md) + [plan8_progress](./plan8_progress.md)(v1 简约子集已落地:骨架先行 + 表格不闪 raw;本 GOAL 升级其调度器)
  - [research/reveal-rhythm](../research/reveal-rhythm.md)(rap flow / groove / 语音韵律 → 节奏函数分层)· [research/chat-page-visual-motion-survey §3](../research/chat-page-visual-motion-survey.md)(**运动 token 数值手册,直接抄**:词间隔 150–250ms / stagger 20–35ms / 缓动曲线 → CurveId)
  - 记忆:`md-reveal-cadence-north-star`(结构块不闪 raw、先骨架再填字、渲染节奏自主)

---

## 0. 铁律(第一条,承 plan8 教训)

北极星是**主观体验**,但落地必须**可判定**:节奏函数的每个分层都要有**确定性 native 测试**(同 seed → 逐 ms 相等)+ **单调性断言**(边界停顿 ∝ 节点层级)。禁止"自评好看"——达标靠数值断言 + 重放对拍,不靠 rubric。

## 1. GOAL 与完成定义(DoD)

**GOAL**:(A) 把 `RevealScheduler` 从匀速升级为**分层节奏函数**——基速 + 边界停顿(∝ 节点层级/标点,最大杠杆)+ 末延长 + 重音拍(强调单元前微停)+ seeded 1/f 微定时;出**三预设**(打字机 / 朗读 / rap flow)。(B) 把"结构块不闪 raw + 骨架先行"从**只表格**泛化到**列表 / 围栏 / 公式 / 图片 / 链接**全类型。(C) 落 [chat-page-visual-motion §3] 的**运动 token 数值手册**进 `motion.rs` 常量表 + CurveId,供 A/B 与 plan28 共用默认值。

**DoD(全满足才算完)**:

1. **三预设可切**:`set_reveal_preset(typewriter|reader|flow)` + 六旋钮 setter（tempo/rest/finalLengthening/accent/microtiming/globalCps),wasm API + 调试面板接上。
2. **确定性**:同 seed → 逐 ms 揭示序列相等(native 测,R8/R9);**typewriter 预设 == 旧匀速行为**(回归等价,证升级无观感回退)。
3. **单调**:边界停顿随节点层级/标点单调增(native 断言);append-only ⇒ 门只增、调度器只前进无回滚(0017 §6)。
4. **全类型骨架先行**:列表(逐项)/围栏(底先字后)/公式(骨架→typeset)/图片(占位→纹理)/链接(`](url)` 前不闪、闭合后提升)——各类结构块**流式中绝不闪 raw**;`is_pending_structure` 从表格泛化到全类型。
5. **motion token 落表**:词间隔 / stagger / 单字符时长 / 缓动曲线 = [chat-page-visual-motion §3] 数值,进 `motion.rs`;新增 native 快照测证 token 表数值稳定。
6. `node test/run.mjs` 四层全绿;golden 帧(S1/S2 等揭示场景)按新节奏重录且双跑一致;**动画组 ≤4 / idle 全退场**断言保持(不回退 plan25/26 资产)。
7. `spec/plan/plan30_progress.md` 逐 milestone 记录(做了什么 / 参考 file:行号 / 数值断言结果 / 遗留)。
8. **全程不 git commit / 不 push**;工作区即交付。

**启动**:`claude --permission-mode acceptEdits "读 spec/plan/plan30-reveal-cadence-aesthetics-goal.md,按其执行到 DoD 全满足"`(中断重跑同句,读 plan30_progress 续跑)。

## 2. Milestones(每步端到端可跑)

- **M1 · motion token 数值手册**([chat-page-visual-motion §3] 抄数值):`motion.rs` 常量表(dur 分档 / stagger / 词间隔 / 单字符 ≤200ms)+ CurveId 扩 enter/exit/expressive 三曲线(bezier 拟合成解析式);style-config/style-panel 暴露。**这是 A/B 的地基,也给 plan28 静态观感层复用**。native 快照测 token 数值稳定。
- **M2 · 节奏函数分层**([reveal-rhythm §2/§3]):`RevealScheduler` 匀速 → Δ_base(词/字单位)→ 边界停顿表(0020 节点层级 + 标点,**大杠杆先行**)→ 末延长 → 重音拍(强调单元前微停 + 略慢)→ seeded 1/f 微定时(小幅点缀,别过度)。揭示时钟以注入 `dt_ms` 推进(可重放)、限速/排队释放,**不跟 smoother 的 token 率**(§113 解耦)。
- **M3 · 三预设 + API**:`typewriter`(= 旧匀速回归等价)/ `reader`(朗读:边界停顿 + 末延长)/ `flow`(rap:加重音拍 + 微定时);`set_reveal_preset` + 六旋钮 setter;调试面板下拉。native 测三条(确定性 / typewriter 等价 / 边界停顿单调)红→绿。
- **M4 · 全类型 raw 抑制**(0019 §158/§163):`RevealUnit{Glyph,Line,Row,Block}` + `content_gate(active_block)` 泛化 `is_pending_table` → 段落逐字、表格到行/整表、列表到项、围栏到闭合、公式/图片到闭合、链接到 `](url)`。纯文本/行内不抑制(逐字可调速)。
- **M5 · 全类型骨架先行**(thinking §1B/§4 · 0019 §162):结构块 `layout_gate` 满足 → 先入场**容器面板**(0018 panel alpha fade)→ 字 stage 延后 `offset_ms`(表格=框/网格先→cell 字;代码块=底先→字;引用=左条先;公式=骨架→typeset;图片=占位框→纹理)。骨架/字端点交 0016 补间,调度器只定"何时"。

## 3. 客观判定

| 维度 | 判据 |
|---|---|
| 确定性 | 同 seed 逐 ms 揭示序列 byte 级相等(native) |
| 回归等价 | typewriter 预设与旧匀速 golden 帧一致 |
| 边界停顿单调 | 层级越深 rest 越大(native 断言表) |
| raw 不闪 | 全类型流式重放:任一帧不出现字面 `\|a\|b\|`/`![..](..`/`[t](u`/半截围栏(读图 + 断言 hold) |
| 骨架先行 | 结构块面板 alpha 入场早于其首字 spawn(时序断言) |
| 资产不回退 | 动画组 ≤4 + idle 活跃 ShaderBox 归零(e2e 断言,TESTREPORT) |

## 4. 铁律(承 AGENTS.md / dev-practices §4)

- **AR7** 吐字单位 = grapheme cluster · **AR9** 流式前沿 hold 区,不提交有歧义字节(= raw 抑制的根据)· **AR1/AR3** 渲染只读、settled 恒等 · **AR10** 每帧一次跨界。
- **CR1** core 零平台依赖 · **R8/R9** 揭示时钟走注入 `dt_ms`、微定时走 seeded 随机(**这是确定性可重放的命根**,绝不用 `now()`/裸 rand)。
- **T6** native 优先 · **T8** 视觉走 off-profile golden(确定性几何)· **T9** 先复现再改。
- **0→1 一次做对**:节奏函数一次落全五层(base/rest/末延长/重音/微定时),预设是同一函数的参数组,不为旧匀速留双轨(旧即时揭示路径删净,唯一揭示源 = 调度器,0019 §54)。
- **不 commit / 不 push**;改 M5/M6(core)前过 AR 清单 + `Skill(rust-write)`。

## 5. 文件归属(worktree 并行边界)

- **主战场**:`crates/core/src/reveal.rs`、节奏调度器、`motion.rs`(新建或扩)、`content.rs` 的 **gate/pending 判定部分**(不动 markdown 解析主体)、effects 里的 stage→端点通道。
- **禁区(留给 plan29/31)**:`spatial.rs`/tier 回收(plan29)、`math.rs`/embed/overlay(plan31)。公式/图片的骨架先行**只做"占位框 + 揭示时序"**,真身 typeset/纹理由 plan31 提供——本 GOAL 对接口留桩,不实现 math/embed 本体。
- content.rs 与 plan31 有潜在重叠:本 GOAL 只碰 gate/reveal-unit,plan31 只碰 Embed/math 节点产出;各自在 progress 记一行触碰范围。

## 6. 风险与预案

| 风险 | 预案 |
|---|---|
| 微定时过度 → 反而机械/晃 | [reveal-rhythm §4] 铁律:微定时只小幅 1/f 点缀,大杠杆放结构性停顿;旋钮默认值保守 |
| 列表/围栏抑制过激 → 正文卡顿 | 纯文本/行内不抑制;结构块保守:确认到结构标记才 hold(0017 §10 收窄 §3) |
| 公式/图片骨架依赖 plan31 未就绪 | 骨架先行用占位框 + 估高(thinking §1D),真身接口留桩;plan31 合入后接真身 |
| golden 帧因节奏变全红 | 预期内:按新节奏重录 S1/S2,双跑一致即可(非回归失败) |

> 一句话:**把"匀速打字机"换成"有句读呼吸 + 重音落点 + 一点人味微抖"的节奏函数,并让每类结构块先骨架再填字**——这是作者极端想要的北极星,plan8 起了头,plan30 落全。
