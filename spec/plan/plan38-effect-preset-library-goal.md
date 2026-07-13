# Plan 38 · GOAL:效果预设库 —— 场景 × 预设的数据驱动效果系统收口(agent 直跑,无人介入)

- 日期:2026-07-14
- 状态:**GOAL(可执行任务书)** —— 给 Claude Code 的完整执行合同,读完即可自主跑到 DoD。
- 执行方式:**顺序执行,master 工作区直跑**。前置 **plan36 完成**(noise/glow/shadow/dissolve/flash/spring 词汇在库);plan37 完成则反馈类槽位可用,未完成则该类槽位留空注册(不阻塞本 GOAL)。
- 定位:**TODO2-A 效果系统第三期(编排层/收口)**。catalog §8 三缺口之三:**效果预设库本身**——把 36/37 的散装能力组织成「场景 × 预设」的数据表,让"换一套效果"= 换一份数据,零代码。这是 0002 §5.1「效果是数据」的最终形态,也是 thinking §4「后面很多效果都从这条路走」的兑现。
- 前置:
  - [research/shader-effects-catalog](../research/shader-effects-catalog.md) §8(接入通道对照)· [research/sdf-animation-system](../research/sdf-animation-system.md) §4/§5(shape×material×motion×easing 正交分解 = preset 的维度论)· [research/animation-system-survey](../research/animation-system-survey.md) §8(分期建议)
  - [0018](../decision/0018-sdf-panel-decoration-primitive.md)/[0025](../decision/0025-sdf-node-animation-system.md)/[0037](../decision/0037-declarative-part-decoration-contract.md)(参数/slot/装饰三通道 = preset 的执行面)· [0002 §5.1](../decision/0002-event-driven-pipeline.md)(profile 三档)
  - 现有场景钩子(全部已在,preset 只是接线):spawn/enter(reveal+rhythm)、exit(0016+plan36 dissolve)、强调(plan36 flash)、待机(plan25 三件套:光标呼吸/orb/边缘条)、thinking(plan28 shimmer)、settle(fsm 位点)、hover/press(plan25 M2)

---

## 0. 铁律(第一条):预设是数据,场景是钩子,两边都不新增机制

本 GOAL **零新 shader、零新图元、零新动画机制**——只做三件事:①定义 preset 数据 schema;②把既有场景钩子接到 preset 查表;③内置三档预设 + 面板/API。若发现某效果"接不上"(缺参数/缺钩子),记遗留回流 36/37,**不在本 GOAL 里加机制**(Surgical Changes)。默认预设 = 现观感逐字节(golden 硬门,三连承 36/37)。

## 1. GOAL 与完成定义(DoD)

**GOAL**:(A) 新 ADR(取当时下一空闲编号,预计 0041):`EffectPreset` schema——场景槽 × 效果参数集,数据驱动零分支;(B) 内置三档:`off` / `subtle`(默认,== 现观感)/ `expressive`;(C) 场景全接线:enter/exit/emphasis/idle/thinking/hover/celebrate;(D) wasm API + 面板矩阵 + 每预设 golden。

**DoD(全满足才算完)**:

1. **ADR 就位**:schema 定稿——`Preset { scene_slots: { Enter, Exit, Emphasis, Idle, Thinking, Hover, Celebrate } → EffectSpec }`;`EffectSpec` = 既有参数面的引用(0018 panel/rect 参数 + 0025 slot 曲线/时长 + 0037 DecorSlot + rhythm 预设名 + 36/37 效果参数),**纯数据可序列化**(serde;JSON 可加载);profile 三档与 preset 正交(off 压制一切);扩展条款:新增效果 = schema 加字段,新增预设 = 加一份数据。
2. **三档内置**:`off`(全恒等);`subtle` = **现观感的显式化**(把散在各处的当前默认值收进一份 preset 数据——从此"默认观感"有单一真值源);`expressive` = 36/37 能力的品味组合(enter=spring+微 glow、exit=dissolve、emphasis=flash、idle=呼吸 glow、thinking=shimmer+微拖尾(37 在则)、celebrate=留空或轻量 spark(时间盒,缺席不判负))。
3. **场景接线全覆盖**:七槽全部从 preset 查表取参(替换散落的硬编码默认值,**行为等价重构**——subtle 档 golden 逐字节证明);未接效果的槽位显式 `None`。
4. **API + 面板**:wasm `set_effect_preset(name)` + `set_effect_override(scene, json)`(单槽热调);style-panel「Effects」矩阵(场景 × 当前值,预设下拉);preset 数据快照 native 测试锁定(同 motion token 惯例)。
5. **硬门三连**:subtle(默认)== 现 golden 全集逐字节;off == 全恒等;expressive 新 golden 家族(dpr=1/2,双跑一致);动画组 ≤4 / idle 全退场 / perf 门在三档下全绿(expressive 的 idle 也必须归零——待机效果走慢时钟 + 收敛,plan25 断言扩到预设级)。
6. native ≥4(schema 序列化往返/预设快照/单槽 override/off 恒等)+ e2e ≥2(预设切换生效/idle 退场三档);`node test/run.mjs` 五层全绿。
7. `spec/plan/plan38_progress.md` 逐 milestone 记账;**按 milestone commit,不 push**。

**启动**:`claude --permission-mode acceptEdits "读 spec/plan/plan38-effect-preset-library-goal.md,按其执行到 DoD 全满足"`(中断重跑同句,读 progress 续跑)。

## 2. Milestones

- **E0 · ADR + schema**:盘点七槽现有取参点(file:行号进 progress)→ schema 定稿 → ADR;`EffectSpec` 落 `primitives`(纯数据,components/core 共用)。
- **E1 · subtle 显式化(行为等价重构)**:现默认值收进 subtle preset 数据 → 七槽改查表 → **golden 逐字节硬门**(这一步是本 GOAL 的 plan33 式"纯搬家")。
- **E2 · off / expressive + API**:off 全恒等;expressive 组合(36/37 词汇,数值保守起步);wasm setter + override + 面板矩阵;预设快照测试。
- **E3 · 收口**:expressive golden 家族 + 三档断言(idle/动画组/perf)+ celebrate 时间盒(≤1 轮,缺席记遗留)+ DoD 对账;`design/thinking.md §4` 待办四条对账打勾(0018 效果层入口自此闭环)。

## 3. 客观判定

| 维度 | 判据 |
|---|---|
| 等价重构 | subtle 档 golden 全集逐字节(E1 硬门) |
| off 恒等 | off 档 == 全效果参数置零帧(对拍) |
| 数据驱动 | 切预设零重编译;新预设 = 纯 JSON(e2e 加载自定义 preset 生效) |
| 单一真值源 | grep 断言:七槽无 preset 之外的效果默认值硬编码残留 |
| 三档纪律 | 每档:动画组 ≤4、idle 活跃归零、perf 门绿 |
| 快照稳定 | preset 数据 insta 快照(改数值必显式过快照,同 motion token) |

## 4. 铁律

- **0002 §5.1 终局**:效果=数据 → 预设=数据集;查表无分支,off=置零。
- **AR2/AR3** 只写 presentation、收敛恒等 · **R8** preset 切换是确定性输入(进重放录像)。
- **0→1**:七槽一次接全,不留"部分查表部分硬编码"双轨(E1 硬门保证)。
- **美学纪律**(承 plan28/32):subtle=参考观感不动;expressive 数值从 catalog 引用处抄(motion token/生理数值),无依据的组合保守起步,**留旋钮不留主观断言**。
- 改 core/primitives 过 AR 清单;**按 milestone commit,不 push**。

## 5. 变更边界

- **主战场**:`primitives`(EffectSpec/preset 数据)、core 七槽取参点(app/reveal/effects 通道)、wasm setter、web 面板。
- **非目标域**:shader 本体(36/37 定稿)、partrender 语义、新效果开发(缺什么记遗留回流)。
- **与后续衔接**:preset 是 plan36/37 的收口,也是未来一切新效果的入册处——此后"加效果"的定义 = 36 式加词汇 → 38 式入预设,两步各自有既定通道。

## 6. 风险与预案

| 风险 | 预案 |
|---|---|
| subtle 显式化挖出隐藏耦合(默认值散得比预想广) | E0 先盘点(file:行号清单);耦合过深的槽位记遗留分期,不硬拆——但已接槽位必须单真值源 |
| expressive 观感翻车(组合灾难) | 数值全部保守起步 + 面板旋钮实时调;不达标就降回 subtle 数值,记录尝试(plan25 教训:不自评好看,靠可调 + 作者实看) |
| celebrate 无轻量粒子通道 | 时间盒 ≤1 轮:glyph instance 复用的 spark 试一把,不成留空槽记遗留(粒子层是 TODO2 远期) |
| preset JSON 被恶意/坏数据喂 | serde 失败整份拒绝回退当前档(AR12),不半应用 |

> 一句话:**36 造词汇、37 开通道、38 编字典**——七个场景槽 × 三档预设,把"效果系统"从散装参数收成一份可换、可调、可快照的数据;默认观感一字不变,而"换一套气质"从此只是换一份 JSON。
