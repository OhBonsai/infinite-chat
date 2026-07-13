# Plan 29 · GOAL:兑现「无限会话」——tier 分级虚拟化 + SumTree 高度索引(agent 直跑,无人介入)

- 日期:2026-07-08(2026-07-09 修订:改顺序执行 + commit 政策跟 plan28 作者改令)
- 状态:**GOAL(可执行任务书)** —— 给 Claude Code 的完整执行合同,读完即可自主跑到 DoD。
- 执行方式:**顺序执行,master 工作区直跑**(不开 worktree;排在 plan28 遗留清单之后或与之交替,同一时刻只有一个 agent 动代码)。
- 优先级:**P0**(初心正身)。初心第 0 节「infinite session」目前只证了 fps、**没证内存回落**——本 GOAL 补上这块。
- 前置(全部已落地,直接站上去):
  - [decision/0029-session-virtualization-and-glyph-working-set](../decision/0029-session-virtualization-and-glyph-working-set.md)(tier 模型 = 本 GOAL 的设计真值)
  - [plan18-scale-memory-verification](./plan18-scale-memory-verification.md) + [plan18_progress](./plan18_progress.md)(**度量段已落地 + before 基线已采**——本 GOAL 的对照基线)
  - [plan19-session-virtualization](./plan19-session-virtualization.md) + [plan19_progress](./plan19_progress.md)(P1/P2 落地;**⚠️ 真凶已被实测定位**:`enqueue_new_text` 每帧重切 grapheme、`SpatialGrid` 每帧 O(总块) 重建;P3 未做 = 本 GOAL)
  - [research/agent-ui-industry-survey §1](../research/agent-ui-industry-survey.md)(Zed **SumTree/前缀和** 做变高项 O(log N) 高度索引 → 替 SpatialGrid 每帧重建)
  - [architecture.md](../architecture.md) M8 scene / M11 input / M13 app

---

## 0. 为什么是 P0(一句话)

初心动机是「一条 100+ 轮、上万行、还在流式增长的会话始终丝滑」。plan18/19 已证 **fps 稳**,但**内存只涨不降**这条初心承诺仍未兑现——settled 块的几何/字形/节点没有按距离回收。本 GOAL 把 0029 的 tier 回收器落地,并顺手用 SumTree 消灭 plan19 点名的 `SpatialGrid` 每帧重建瓶颈。

## 1. GOAL 与完成定义(DoD)

**GOAL**:实现 0029 的**分级工作集回收**——按距可见区的距离把 `PartView` 降级(Hot→Warm→Cold→FrozenFar),逐级丢几何/节点/字形;滚回时确定性重建。并把块高度索引从 `SpatialGrid` 每帧重建换成 **SumTree/前缀和**(O(log N) 定位 + 增量更新)。观感/正确性零回退。

**DoD(全满足才算完)**:

1. **内存回落**:plan18 长会话场景(≥100 轮,上万行),来回滚动后内存回落到 **visible 基线 ±10%**(before 曲线在 plan18_progress;after 曲线本 GOAL 采,并排入 `plan29_progress`)。
2. **fps 不回退**:滚动/静止 fps ≥ plan19 达标值;静止结算满帧;`?debug` 无掉帧(读 `report/TESTREPORT.md`,不自评)。
3. **无跳变**:Cold→Hot / FrozenFar→Hot 重建后,块 `abs_origin` 与重建前一致(±0,catch-up 瞬显、零动画,守 AR6)。
4. **确定性**:同一录像重放两次,回收/重建序列一致,最终 `Store` + 可见几何一致(R8/R9;`tests/replay.rs` 同款)。
5. **SumTree 替换**:`SpatialGrid` 每帧全量重建路径删除;高度索引改 SumTree/前缀和,增量更新脏区;新增 native 基准证明"长会话下高度查询不随总块数线性增长"。
6. `node test/run.mjs` 四层全绿;**至少 +2 native 测试**(tier 回收确定性 + SumTree 高度索引不变式),红→绿。
7. `spec/plan/plan29_progress.md` 逐 milestone 记录(做了什么 / 参考 file:行号 / before-after 数值 / 遗留)。
8. **按 milestone 正常 commit(承 plan28 R5 作者改令),不 push**;每个 commit 前全门绿。

**启动**:`claude --permission-mode acceptEdits "读 spec/plan/plan29-session-virtualization-goal.md,按其执行到 DoD 全满足"`(中断重跑同句,读 plan29_progress 续跑)。

## 2. Milestones(每步端到端可跑,不留半成品)

- **V0 · 度量对齐**:确认 plan18 度量字段在(`FrameStats` 的 `store_chars/retained_views/retained_glyphs/retained_nodes`;缺则按 0029 §82 补)。**先采 before 曲线复现**(plan18 场景),数值抄进 progress。没有 before 不动结构(0029 §82 铁律)。
- **V1 · SumTree 高度索引**(agent-ui §1):`height` 提升为 view 级字段(脱离 `BlockCache`,0029 §84);块堆叠/AABB 查询改走 SumTree/前缀和(O(log N) + 增量);删 `SpatialGrid` 每帧 O(总块) 重建。验收:高度查询基准不随总块数线性增长;可见区间二分正确。
- **V2 · tier 回收器**(0029 §83/§85/§86):`PartView` 加 `tier: Tier` + `release_to(tier)`/`rehydrate()`;`spawn` 结算后压 `settled`;`SpatialGrid`→SumTree 输出**分级距离**(可见/Warm/Cold/FrozenFar)而非布尔;**滞回回收器**(节流扫活跃 view,按距离调 tier:Warm 释 `release_geometry` / Cold 丢 `nodes` / FrozenFar 丢 `revealed`+Store text)。
- **V3 · 重建路径**(0029 §87/§88):Cold→Hot 走 `ensure_layouts`(已确定性);FrozenFar→Hot 走 `resync_from_snapshot` catch-up 瞬显(AR6 零动画);`image_registry`/`code_scroll` 随块 tier 回收(降 Cold 清条目、重建重登)。
- **V4 · 长会话回归 + after 对比**(0029 §89/§90):plan18 长会话来回滚 case——内存回落 visible 基线、滚动无跳变、静止满帧;`?novirt` 关虚拟化对照;debug 面板显各 tier 块数 + 本帧重建次数。after 曲线 vs before 并排入 progress。**验收截图 dpr=1 与 dpr=2 各一份**(plan28 R4+ 教训:dpr=1 截图漏掉 retina 回归;shoot-sdf.mjs 已有 SHOT_DPR)。

## 3. 客观判定(可脚本/可读 TESTREPORT 核对)

| 维度 | 判据 |
|---|---|
| 内存回落 | after 峰后回落 ≤ visible 基线 ×1.10;progress 附 before/after 数值 |
| 高度查询复杂度 | native 基准:总块 ×10 时查询耗时不 ×10(证 O(log N)/前缀和) |
| 重建一致 | 重建前后 `abs_origin` 逐块 diff == 0 |
| 确定性 | 同录像双跑:tier 迁移序列 + 最终可见几何 byte 级一致 |
| 观感零回退 | plan19 达标 fps 不降;`?debug` 无掉帧(TESTREPORT) |

## 4. 铁律(承 AGENTS.md / dev-practices §4,必守)

- **引擎能力零回退**:虚拟化只增不减既有能力(选区/复制/查找/a11y/morph/节奏)。
- **AR1** 渲染只读状态 · **AR3** settled 几何只读(hit-test/选区/滚动)· **AR6** catch-up 零动画零平滑(重建即瞬显)· **AR10** 每帧一次跨界 · **AR11** Turn 投影不存储。
- **CR1** `core` 零 wasm/web-sys/wgpu 依赖 · **R8/R9** core 禁 `now()`/裸 rand(回收器节流时钟走 `Clock` seam、随机走注入种子,保重放)。
- **T6** 纯逻辑 native 测 · **T9** 先写复现再 fix · **T10** flaky 找根因不重跑。
- **0→1 一次做对**:tier 模型按 0029 目标一次落全四级,不为兼容旧路径留双轨(旧 `SpatialGrid` 路径删净)。
- **按 milestone commit,不 push**(承 plan28 作者改令);commit 前全门绿,进度轨迹 progress + commit 双记。
- 改 M8/M13(core)前过 AR 清单;改 M8-M10(render)先 `Skill(render-write)`。

## 5. 变更边界(顺序执行,手术纪律)

- **主战场**:`crates/core/src/spatial.rs`(→ SumTree)、`app.rs`(grid 用法 / view tier / 回收器)、`scene`/`PartView` 相关、`FrameStats`。
- **可能触碰**:`render` 侧仅"释放/重建 GPU 资源视图"接口,不改渲染语义。
- **非目标域**(Surgical Changes,AI Rules 3):`reveal.rs` 节奏逻辑、`content.rs` 解析、`math.rs`、embed/overlay 观感——不顺手改;若必须动,只加 tier 钩子、不改其语义,并在 progress 记一行。
- **与 plan28 遗留的衔接**:plan28 遗留 §1(Explored 分组行)要动 `build_frame` 块合成,与本 GOAL 的 view/tier 相邻——顺序执行天然无冲突,但**后跑的一方 rebase 时注意 `app.rs`**;若本 GOAL 先跑,给 view 加 tier 字段时不动 `group_message_parts` 桶机制。

## 6. 风险与预案

| 风险 | 预案 |
|---|---|
| SumTree 增量更新写错致高度漂移 | 加不变式校验器(Σ 叶高 == 根;二分命中 == 线性扫);proptest 乱序插入/更新 |
| FrozenFar 重建瞬显仍闪一帧 | 走 `resync_from_snapshot` catch-up 路(AR6),重建帧不进 effects |
| headless WebGPU 起不来 | 唯一允许求助人的情形(同 plan25/28);native 逻辑测不受影响先推进 |
| before 基线无法复现(环境漂移) | 先在本 worktree 重采一份 before,注明与 plan18 的差异 |

> 一句话:**plan18 把尺子造好了,plan19 找到了真凶,plan29 收网**——tier 回收让内存跟着可见屏走,SumTree 让高度查询跟着 log 走,「无限会话」这句初心才算兑现。
