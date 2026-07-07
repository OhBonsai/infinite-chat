# Plan 25 · GOAL:自主闭环落地整套 chat page design(agent 直跑,无人介入)

- 日期:2026-07-06
- 状态:**GOAL(可执行任务书)** —— 本文件是给 Claude Code 的完整执行合同:读完即可自主跑完,不需要作者介入。
- 设计正主:**[design/chat-page-design.md](../design/chat-page-design.md)**(§3 节奏系统 / §4 组件规格 / §6 相位)——本 GOAL = 把它的 P0→P3 全部落地。
- 数值/依据:[research/chat-page-visual-motion-survey](../research/chat-page-visual-motion-survey.md);机制 ADR 见设计文档头部链接。
- 关键能力(已核实存在,闭环靠它们):`window.__chat.push_event(raw)` 合成任意事件 · `set_paused(true)` + `tick(dt)` / `seek_reveal(ms)` 确定性定帧 · Playwright headless WebGPU 截图(`web/tests/visual.spec.ts` 先例)· `node test/run.mjs` 四层门 + `test/results/TESTREPORT.md`。

---

## 0. GOAL 与完成定义(DoD)

**GOAL**:实现 chat-page-design.md 的 P0(MotionTokens)→ P1(节奏函数)→ P2(组件动效)→ P3(高端档,best-effort),每步用 **Playwright 截图 → agent 读图自评 → 迭代** 的视觉闭环收敛观感,全程卡口全绿。

**DoD(全部满足才算完)**:

1. `node test/run.mjs` 四层全绿(gates/native/unit/e2e),含新增测试。
2. M1–M3(= P0–P2)的验收清单(§4)逐项勾掉;M4(P3)完成或在 progress 文档记录明确的 defer 理由。
3. `web/tests/design-review.spec.ts`(本 GOAL 新建的截图 harness)产出全场景截图,agent 按 §5 rubric 自评通过(≥ 达标线),评审记录写进 `spec/plan/plan25_progress.md`。
4. 收敛后的关键帧固化为 `visual.spec.ts` 黄金图(`--update-snapshots` 一次性入册),此后进默认门做像素回归。
5. 节奏确定性 native 测试:同 seed 同输入 → 揭示时刻序列逐 ms 相等(R8)。
6. `spec/plan/plan25_progress.md` 记录每个 milestone 的做了什么/验证方式/遗留。

**启动方式(作者唯一要做的事)**:在仓库根跑
`claude --permission-mode acceptEdits "读 spec/plan/plan25-chat-page-design-goal.md,按其闭环协议自主执行到 DoD 全满足"`
(或加 `--dangerously-skip-permissions` 全自动;中断后重跑同一句,agent 读 progress 续跑。)

## 1. 铁律(违反任一条 = 本 GOAL 失败,优先级高于进度)

- 按仓库流程走:先读 `AGENTS.md` + `DEVMEM.md §2`;写 Rust/shader/TS 分别遵守 `/rust-write` `/render-write` `/bridge-write` `/test-write` 技能铁律。
- **CR1**:core 不引 wasm/web/wgpu;节奏函数、边界表、词分组全在 core、纯函数、native 可测。
- **R8/R9**:一切随机 seeded(块 key 为种)、一切时间走注入时钟;禁 `now()`/裸 rand。
- **0016 品味约束**:几何绝不 snap;色变 snap 合法;入场动画不动盒位(不破滚动锚定)。
- **0018/0032 边界**:文字不进 fragment 特效;不引任何组件框架;新效果一律 = panel/widget/ShaderBox 参数。
- 测试结论只读 `test/results/TESTREPORT.md`,不自评"应该没问题";flaky 找根因不重跑硬过。
- **不 git commit / 不 push**:全程只改工作区,一口气跑到 DoD;版本轨迹靠 `spec/plan/plan25_progress.md` 逐 milestone 记录(做了什么/验证方式/遗留),提交与否由作者跑完后自行决定。
- **不越范围**:不动解析/FSM/store/稳定性地基;不做暗主题/BiDi/产品化。

## 2. 闭环协议(每个 milestone 循环跑)

```
读设计文档对应节 → 实现 → node test/run.mjs(读 TESTREPORT)
  → 跑截图 harness(§3)→ Read 截图 PNG → 按 §5 rubric 打分写评审
  → 不达标:定位差距(数值?编排?shader?)→ 调 token/参数/实现 → 回到实现
  → 达标:固化黄金帧 → 更新 progress(不 commit)→ 下一 milestone
```

- **迭代上限**:同一 milestone 视觉自评 ≤ **5 轮**;5 轮未达标 → 把当前最好版本 + 差距分析记入 progress 的「遗留」,继续前进(不无限打磨)。
- **每轮必须改数据先于改代码**:观感差距优先调 MotionTokens / RevealStyle / panel 参数(设计原则 5:一切皆数据);连续两轮调数据无效才允许动实现。
- **截图必须确定性**:`set_paused(true)` 后用 `seek_reveal(ms)`/`tick(dt)` 走到目标帧再截;动态 ShaderBox 区(orb/光标/edge_glow)单独截「相位固定帧」(pause 后 tick 到固定相位);比对区照 `visual.spec.ts` 先例 clip 裁剪。

## 3. 截图 harness(M0 第一件事建好,后续所有 milestone 复用)

新建 `web/tests/design-review.spec.ts`(**execution-only,不做断言**,不进默认门;跑法 `npm --prefix web run e2e -- design-review.spec.ts`):

- 复用 `helpers.ts` 的 `assertWebGpu`/`bootVisible`;场景数据用 `push_event` 合成(参考 `parts-render.spec.ts` 的事件构造)。
- 每场景输出 PNG 到 `test/results/design-review/<scene>/<frame>.png`(gitignored),文件名含场景+seek 毫秒。
- **场景矩阵 × 定帧点**(设计文档 §4 组件规格的可视化全集):

| 场景 | 构造 | 截帧点(seek/tick) |
|---|---|---|
| S1 正文节奏(reader 预设) | 长 markdown text part 流式 | 揭示 20% / 50% / 80% / settled 四帧 + 连续 8 帧(200ms 间隔)看节奏 |
| S2 预设对比 | 同文本 × typewriter / reader / flow | 各取 50% 一帧 + 8 连帧 |
| S3 tool 卡 | bash tool 三态 + error(push_event 变 status) | pending / running(edge_glow 相位 0 与 π)/ done / error 各一帧 + 折叠/展开中点帧 |
| S4 reasoning 卡 | reasoning part 流式→完成 | 骨架入场中点 / running(FBM 若已做)/ auto-collapse 中点 / 收起后 |
| S5 代码块 | 长围栏代码流式 | 底板 wipe 中点 / 逐行揭示中 / 闭合上色后 |
| S6 表格 | 风格 3 全表门 | 网格 wipe 中点 / 表头 stage / cell 填充中 / settled |
| S7 diff | edit tool + filediff | 行带 wipe 中点 / settled + 5 格条 |
| S8 待机生命感 | 首 token 前 + 流式中 | orb/光标/指示条各相位 0、π/2 两帧 |
| S9 整页 layout | 3 轮完整对话(user+assistant 多 part) | 整页一帧(间距/分组/meta)+ 滚动后一帧 |
| S10 交互反馈 | dispatch pointer hover/press 到卡片/链接 | hover 前后两帧 / press 帧 |

- M0 时先只建 S1/S2/S9(当时只有 token 改动可看),此后每做完一个组件往矩阵补场景——**harness 随实现长大,矩阵是终态**。

## 4. Milestones(= 设计文档 §6 相位,含验收清单)

### M0 · 基建 + P0 token 收口
- [ ] `design-review.spec.ts` harness + S1/S2/S9 场景可出图。
- [ ] MotionTokens:core 侧 `motion.rs`(dur/ease/stagger 常量表)+ `CurveId` 扩 3 条曲线(enter/exit/expressive,bezier 拟合成解析式)+ style-config/style-panel 暴露;胶水见 0021 两条生效路径。
- [ ] 间距 token(space.turn/part/blockGap/inset)替换 BLOCK_GAP 等裸数;user 盒弱底面板;meta 行 Complete 态延迟显。
- [ ] 现有 enter profile / panel 默认值改引 token。
- [ ] 验收:gates+native+e2e 绿;S9 截图自评「间距节奏」达标;新增 1 个 native 测试(token 表数值稳定性快照)。

### M1 · P1 节奏函数(北极星本体)
- [ ] `RevealScheduler` 匀速 → 分层节奏函数(设计 §3.2):Δ_base(词/字单位)→ 边界停顿表(0020 节点层级 + 标点)→ 末延长 → 重音拍 → seeded 1/f 微定时;`typewriter/reader/flow` 三预设。
- [ ] wasm API:`set_reveal_preset(name)` + 六旋钮 setter;调试面板接上。
- [ ] native 测试:① 确定性(同 seed → 逐 ms 相等)② typewriter 预设 = 旧匀速行为(回归等价)③ 边界停顿单调 ∝ 层级。红→绿。
- [ ] 验收:S1/S2 连帧自评「节奏可感知:句读呼吸、词成拍、无机械感」;`?debug` 无 fps 回退(TESTREPORT + bench.spec)。

### M2 · P2 组件动效(逐组件小循环,顺序如下)
- [ ] panel/widget 加 `reveal_progress`(mask wipe)参数 → 骨架 wipe 入场铺开(reasoning/tool/代码块/表格/引用条)。
- [ ] 词级 spawn 分组 + 到达高亮(弱,默认开、旋钮可关)。
- [ ] 待机生命感三件套:行内光标(呼吸 0.15Hz)/ 首 token 前 orb(复用 glow_orb)/ 回合左缘指示条;全部慢时钟 + idle 全退场(FrameStats 断言活跃 ShaderBox 归零)。
- [ ] tool 卡 running edge_glow(panel 参数);徽章 widget fn;diff 5 格条 widget fn。
- [ ] hover/press 反馈(panel aoStrength/tint + scale 0.99);光标 pointer 命中走 0032 ②。
- [ ] 每完成一项 → 对应场景截图自评 → 达标才下一项;S3–S8/S10 逐步补全。
- [ ] 验收:场景矩阵全出图 + rubric 全达标;e2e 新增「idle 回冻结」「动画组 ≤4」两条自动断言。

### M3 · 黄金帧固化 + 汇总验收
- [ ] 达标帧挑代表作(每组件 1–2 帧)迁入 `visual.spec.ts` `toHaveScreenshot`(裁动态区),`--update-snapshots` 入册。
- [ ] `node test/run.mjs` 完整四层全绿;`spec/plan/plan25_progress.md` 汇总;TODO.md 的「节奏美感升级」条目标注已落地 → 指向本 plan。

### M4 · P3 高端档(best-effort,允许 defer)
- [ ] Plan 10 相位 4 最小件:徽章 SDF `mix` morph(pending→running→✓)。
- [ ] 墨团选区(smin 并集)· thinking_flow FBM · lensing 高光 —— 逐项试,风险大或 2 轮不达标即 defer 并记录。

## 5. 视觉自评 rubric(agent 读 PNG 逐项打 ✓/✗;每场景全 ✓ = 达标)

**客观项(硬性,✗ 即改)**:
1. 无 raw markdown 残留(`**`/裸 `|---`/半截围栏)——截图 + readRuns 双查。
2. 无元素越界/重叠/截断(卡片文字溢出、徽章错位、meta 顶到正文)。
3. 间距肉眼呈规律级差:turn 间距 > part 间距 > 块内间距,整页扫一眼能分清回合分组。
4. 骨架先行可见:结构块场景的"中点帧"必须是**先框后字**(有骨架无正文),绝不出现半截 raw 文本铺开。
5. wipe/入场中点帧几何连续(无断裂、无闪白、无错位双影)。
6. 同帧活跃动效组 ≤4(数截图里"正在动"的独立区域 + FrameStats 交叉)。
7. settled 帧与 idle 帧完全静态(两次 tick 后截图逐像素相等)。

**观感项(taste,连帧序列上判)**:
8. S1/S2 连帧:reader 预设能看出句读停顿(标点/段落处的帧间揭示量明显小于句中);flow 预设密而不乱;typewriter 明显机械(作为反差基线)。
9. 词成拍:连帧里新字以词簇出现,不是均匀单字撒点。
10. 生命感克制:orb/光标/glow 在相位两帧间有可见变化但幅度小(不刺眼、面积小);整页不"躁"。
11. 卡片有层次:底/边/徽章/标题/正文对比分明,running 态一眼可辨,error 醒目但不闪。
12. 整页气质:接近"Claude/Zed 的安静克制",不是"营销页特效堆砌"——存疑时选更收敛的参数。

**打分记录**:每轮写 `test/results/design-review/REVIEW-<milestone>-r<n>.md`(场景 × 12 项 + 差距归因 + 本轮改动),最终汇入 progress。

## 6. 风险与预案(agent 自查)

| 风险 | 预案 |
|---|---|
| headless WebGPU 起不来 | `assertWebGpu` 先例;若环境缺 GPU → e2e 层黄跳过是**不可接受**的(本 GOAL 靠图收敛)→ 停下报告环境问题,这是唯一允许求助人的情形 |
| 截图非确定(动态区) | 一律 pause+seek;动态区裁剪或相位定帧;连续两次同 seek 截图 diff 非零 → 先修确定性再继续 |
| 节奏函数拖慢长文本 | 边界停顿只作用活跃块;催化项:总时长封顶(块级 stagger 压缩);bench.spec 盯 fps |
| wgsl 改 vertex/参数布局 | wgpu 严格校验——gates 的 wasm-build + wgsl 拼接测试先行;实跑靠 e2e 场景帧 |
| 微定时破重放 | seed=块 key;M1 确定性测试是闸,先红后绿 |
| 5 轮不收敛 | 记录遗留继续走(§2);宁可整体完成度 90% 也不卡死单点 |

---

> 一句话:**这份 GOAL = 设计文档(拍什么样)× 截图 harness(拍下来)× rubric(像不像)× 迭代上限(不失控)× 仓库卡口(不回退)。** agent 从 M0 开始顺序执行,每步闭环,跑到 DoD 为止。
