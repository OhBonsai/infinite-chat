# Plan 32 · GOAL:code diff 体验升级 —— Zed 三层视觉 + makepad 风格动效 + 装饰声明式契约(agent 直跑,无人介入)

- 日期:2026-07-13
- 状态:**GOAL(可执行任务书)** —— 给 Claude Code 的完整执行合同,读完即可自主跑到 DoD。
- 执行方式:**顺序执行,master 工作区直跑**(不开 worktree)。当前 master = plan28(opencode 观感还原)+ plan29(四级 tier 虚拟化)+ plan30(节奏/骨架先行)全量成果。
- 定位:**components 化的第一个租户**——本 GOAL 同时立「part 装饰声明式契约」(新 ADR 0035),diff 卡是第一个完全自治的组件;plan33 的 crate 拆分以此契约为前提。
- 前置(全部已核实,直接站上去):
  - [research/code-diff-rendering-zed-makepad](../research/code-diff-rendering-zed-makepad.md)(**本 GOAL 的设计真值**:Zed 数据/显示层 + makepad 手法 + D1–D4 落地方案,含可抄数值)
  - 现状:`partspecific.rs:422-495`(diff_parse_lines 首字符分类 + render_diff)· `app.rs` flush_diff_band(整宽红绿带,装饰长在 build_frame 里)· `theme.rs` `diff_add_bg/del_bg`(opencode 参考色已入)
  - [0018 panel 图元](../decision/0018-sdf-panel-decoration-primitive.md)(装饰 = 参数)· [0033 part 渲染契约](../decision/0033-part-render-contract.md)(本 GOAL 扩它)· [0016 morph](../decision/0016-streaming-morph-render-model.md) · [0025 SDF 节点动画](../decision/0025-sdf-node-animation-system.md) · [0032 SDF 世界非组件框架](../decision/0032-interaction-architecture-sdf-world-not-component-framework.md)
  - makepad 参照(本机 `~/w/agentscode/makepad`):`draw_selection.rs`(gloop 三行滑动窗口)· `session.rs:157-195`(折叠 scale ×0.9 指数逼近)· `layout.rs:292-299`(fold/scale 网格)

---

## 0. 美学基准(两源分工,承 plan28 第一铁律)

- **静态数值**(色/间距/圆角/字号):仍以 **opencode 参考真值优先**(plan28 R0 流程);opencode 没有的维度(词级高亮色、gutter 条宽)抄 **Zed 语义**(research §2.2 数值表)。无参考依据的美学决策不做。
- **形态与动效**(参考静态截图覆盖不到的):沿用 **makepad 风格**——gloop 连续圆角高亮、折叠 scale 动画、SDF 缩放不糊。这是"参考没有、我们更讲究"的增强层(同 plan28 R4 对动效的定位)。
- diff 语义色维持 绿=增 / 红=删(行业惯例 + opencode 同款);「蓝/黄/红分级」记忆适用于状态标注,不适用 diff。

## 1. GOAL 与完成定义(DoD)

**GOAL**:(A) diff 数据 hunk 化 + 词级区间(D1);(B) Zed 三层视觉:半透 hunk 行底 + gutter 色条 + 词级高亮 + 双列行号(D2);(C) makepad 手法:gloop 连续圆角行底 + 上下文折叠/展开 scale 动画(D3/D4);(D) **ADR 0035「part 装饰声明式契约」**:renderer 除 StyledSpan 外产声明式装饰指令,diff 装饰发射权从 app.rs 下放——diff 卡成为第一个自治组件。

**DoD(全满足才算完)**:

1. **hunk 化**:`diff_parse_hunks` 纯函数——`@@ -a,b +c,d @@` 解析行号、连续 -/+ 段配对成 `DiffHunk{Added|Deleted|Modified}`;Modified 且行数相等且 ≤5 行(Zed MAX_WORD_DIFF_LINE_COUNT 同值)产词级区间;native 快照测 + proptest(畸形 diff 不 panic,AR12 精神)。
2. **三层视觉落地**:行底半透 hunk 色(叠卡底)、gutter 每 hunk 一条(宽 `floor(0.275×行高)`,Zed 值)、词级字符底、双列行号(old/new,Dim 色);数值来源逐项записано(opencode 或 Zed,research §2.2)。`+a -d` 徽章 + 5 格条保留。
3. **gloop 连续圆角**:相邻同色 diff 行底经 SDF smooth-union 融成连续圆角块(panel shader 加 prev/next 行宽参数,makepad `draw_selection.rs` 手法 + IQ smin);单行退化普通圆角;WebGL2 路不崩。
4. **上下文折叠**:连续 Context 行 > 3 时折叠为「⋯ n unchanged lines」一行,点击展开;展开/收起 = 被折行 scale 指数逼近(每帧 ×0.9 → snap,makepad 范式,走 0025/0016 通道注入 dt 保确定性 R8);SDF 字形随 scale 缩不糊;折叠行 gutter 画 Zed 式强圆角小 marker(圆角 = 行高)。命中走 0032 ②(tool 卡折叠同路)。
5. **ADR 0035 就位 + diff 首租**:新 ADR(append-only)定义 `DecorationOp`(panel 参数列表 + 锚定 span 区间)契约;`render_diff` 产 spans + decorations;app.rs 的 `flush_diff_band` 逐行反推路径**删净**(0→1 不留双轨),build_frame 只按指令摆放。其余 part(tool 卡等)本期**不迁**(Surgical Changes),契约留好位。
6. `node test/run.mjs` 四层全绿;**≥3 native 新测**(hunk 解析快照 / 词级区间 / 折叠 scale 确定性)红→绿;golden 新增 diff 场景帧(折叠态 + 展开态),dpr=1/2 双份;`FrameStats` 动画组 ≤4 / idle 退场断言保持。
7. `spec/plan/plan32_progress.md` 逐 milestone 记录(做了什么 / 参考依据 file:行号 / 数值来源 / 遗留)。
8. **按 milestone 正常 commit,不 push**;每个 commit 前全门绿。

**启动**:`claude --permission-mode acceptEdits "读 spec/plan/plan32-diff-experience-goal.md,按其执行到 DoD 全满足"`(中断重跑同句,读 plan32_progress 续跑)。

## 2. Milestones(每步端到端可跑)

- **D0 · ADR 0035 装饰声明式契约**:先立契约再动手——`DecorationOp`(种类:band/gutter/underline/char_bg…,全部化为 0018 panel 参数 + span 区间锚定)、renderer 签名扩展(0033 之上 additive)、app 摆放规则(区间→世界矩形,复用现 role 反推的几何逻辑)。ADR 里写明:这是 0032 的深化(装饰=数据,不是 widget 对象),plan33 拆分的前提。
- **D1 · hunk 化数据**(core 纯函数):`diff_parse_hunks` + 词级区间(词对齐 LCS,分词=空白+标点,语言感知记遗留);`render_diff` 改产 spans + `DecorationOp`;`diff_parse_lines` 旧路径由新实现覆盖(快照测迁移)。
- **D2 · 三层视觉**:行底(半透 hunk 色,数值:opencode `--surface-diff-*` 为主、透明度档参考 Zed 0.12/0.16)→ gutter 条(0.275×lh,`version_control_*` 语义色进 theme,hex 若 opencode 无则按 Zed 默认色系配)→ 双列行号(Dim,diff 行换语义色)→ 词级字符底。每层一个 DecorationOp 种类,发射全在 `render_diff`。验收:与 opencode diff 卡并排 diff(plan28 §5 容差同款)+ 超出参考的维度记「Zed 语义」来源。
- **D3 · gloop 连续圆角**(render/WGSL):panel shader 增 prev/next 邻行参数 + smin;band 指令按 hunk 分组喂邻接信息;`Skill(render-write)` 先行;WebGL2 兜底(无邻接参数时退化独立圆角,观感可接受)。
- **D4 · 上下文折叠 + scale 动画**:折叠判定(Context 连续 >3)进 D1 数据层(确定性);「⋯ n unchanged lines」行 + 点击命中(0032);展开/收起 scale 通道(0025 slot,注入 dt,idle 收敛恒等 AR3);golden 折叠/展开双态帧。

## 3. 客观判定

| 维度 | 判据 |
|---|---|
| hunk 解析 | 快照测:混合 diff(增/删/改/hunk 头/无尾换行)→ hunk 列表稳定;proptest 不 panic |
| 词级区间 | 等行数 ≤5 行才产;区间在行内且不重叠(不变式断言) |
| 视觉数值 | 每项装饰的色/宽/透明度在 progress 注明来源(opencode 参考或 Zed 值);并排图 `restore-diff/plan32-*.png` dpr=1/2 |
| gloop | 相邻行融合处无接缝(放大截图人检);单行退化正确(golden) |
| 折叠确定性 | 同录像双跑折叠/展开帧序列一致(注入 dt);scale 收敛后恒等(AR3) |
| 契约生效 | app.rs 无 diff 专属装饰分支(grep 断言 flush_diff_band 删净);装饰全部来自 DecorationOp |
| 零回退 | 四层门全绿;非 diff 场景 golden 逐字节不变(契约只迁 diff) |

## 4. 铁律(承 AGENTS.md / dev-practices §4)

- **0→1 一次做对**:装饰契约按目标形态一次定(区间锚定 + panel 参数),diff 旧逐行反推路径删净,不留双轨。
- **AR1/AR3** 渲染只读、动画收敛恒等 · **AR12** 畸形 payload 兜底不 panic · **AR10** 每帧一次跨界。
- **CR1** core 零平台依赖(hunk 解析/折叠判定纯函数)· **R8/R9** 折叠动画走注入 dt,无 `now()`。
- **T6** native 优先 · **T8** off-profile golden · **T9** 先复现再改。
- 参考代码 MIT(Zed AGPL **只抄数值/结构语义,绝不搬源码**;makepad MIT/Apache 手法可抄,gloop shader 自写 WGSL)。
- **按 milestone commit,不 push**;改 render 先 `Skill(render-write)`,改 core 过 AR 清单。

## 5. 变更边界(顺序执行,手术纪律)

- **主战场**:`partspecific.rs`(R3 段)、新 `diffmodel.rs`(或并入 partspecific)、`app.rs` 装饰摆放段(只删 diff 分支 + 加 DecorationOp 摆放)、`render` panel shader(gloop)、`theme.rs`(增词级/gutter 语义色)、ADR 0035。
- **非目标域**:`reveal.rs`/`rhythm.rs`(plan30 成果勿动;diff 行的揭示节奏走既有通道)、`spatial.rs`/tier(plan29)、`math.rs`/embed(plan31)、tool/reasoning 卡的装饰迁移(plan33 或后续)。
- **与 plan29 衔接**:diff 卡随块 tier 回收/重建(折叠态是数据不是 presentation?——**折叠态入 view 级持久字段**,Cold 重建后保持,同 code_scroll 语义);scale 动画只在 Hot。
- **与 plan30 衔接**:diff 块揭示 = 骨架先行家族(卡框先现、行按节奏后到)已由 plan30 M5 覆盖,本 GOAL 不改揭示时序。

## 6. 风险与预案

| 风险 | 预案 |
|---|---|
| gloop 参数在 storage buffer 布局不够 | 0018 §2 增量通道已支持扩参;实在不够 v1 退化「每 hunk 一个圆角块」(仍优于逐行) |
| 词对齐 LCS 在长行超时 | 行长 > 512B 跳过词级(Zed 同思路上限);记 FrameStats |
| 折叠 scale 与 boxlayout 高度联动复杂 | scale 只作用块内行(行高 × scale 求和进块高),块外走既有 reflow 补间(0016);先 native 测高度函数 |
| opencode 参考无 diff 完整样式(plan28 已记 playground stub) | 已降级源码静态提取(plan28 遗留-2 同款);缺口维度用 Zed 语义并记录 |
| Zed 是 AGPL | 只抄行为语义与数值(宽度/透明度/限流值),shader 与实现全自写;research 文档已注明 |

> 一句话:**数据 hunk 化、视觉抄 Zed 三层、动效走 makepad 的 gloop + scale 折叠、装饰立声明式契约**——diff 卡从"最差的组件"变成"第一个自治组件",给 plan33 拆分打样。
