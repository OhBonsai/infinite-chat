# 决策记录 0033:part 渲染契约 —— 分派注册表 + 通用兜底

- 日期:2026-06-24
- 状态:**已采纳(契约已实现:`crates/core/src/partrender.rs`)**
- 前置:[0001](0001-canvas-architecture.md)(content→layout→render 契约)、[0002](0002-event-driven-pipeline.md)/[0006](0006-inline-tags-and-extensibility.md)(注册表项不是代码分支)、[0032](0032-interaction-architecture-sdf-world-not-component-framework.md)(数据驱动分派)、[Plan 22](../plan/plan22-opencode-events-and-fsm.md)(生产者 + 兜底)、[Plan 23](../plan/plan23-part-render-implementation.md)(消费者 + specific 渲染器)。
- 定位:**把 Plan 22 与 Plan 23 的唯一接触面抽成独立、可冻结的契约**,让二者**并行开发**:Plan 22 产出数据 + 兜底渲染器;Plan 23 注册 specific 渲染器覆盖。本篇 = 该契约的**单一真相源**(改契约双方都须知)。

---

## 1. 背景:并行开发需要一条冻结的缝

Plan 22(全事件/状态/兜底渲染,"丑骨架")与 Plan 23(逐类漂亮渲染器)若散在一份 plan 里,改一处两边受影响。抽成独立契约后:**Plan 22 不依赖 Plan 23**(兜底自洽)、**Plan 23 只对着契约写纯渲染函数**(不碰事件/状态/store)、**唯一接触面 = registry 注册一行**。

这正是 opencode `PART_MAPPING` + `GenericTool` 兜底、业界 `tool 名→渲染器 + fallback`(Vercel/assistant-ui/CopilotKit,见 [业界调研](../research/agent-ui-industry-survey.md)§2)的收敛模式。

## 2. 契约(三件,已冻结)

```
① 输入投影 RenderPart        store 从 part 填的渲染面向投影(与 protocol Part 解耦)
② 分派 RenderRegistry        PartKind → RenderFn;缺省=兜底,specific 覆盖
③ 输出 Vec<StyledSpan>       渲染器吐既有 content→layout 契约的 StyledSpan(0001)
```

**公开 API(`infinite_chat_core` 导出):**

```rust
pub enum PartKind { Text, Reasoning, Tool, File, Compaction, Unknown }

pub struct RenderPart {            // store 填(Plan 22);渲染器只读
    pub kind_tag: String,          // 身份标签:"tool:bash · running" / "reasoning" / "compaction"
    pub text: String,              // text/reasoning 正文(markdown);非文本可空
    pub payload_json: Option<String>, // 结构化 part 的 JSON dump(tool input/output…)
}

pub struct RenderCtx { pub width: f32, pub folded: bool }  // 渲染器只读上下文

pub type RenderFn = fn(PartKind, &RenderPart, &RenderCtx) -> Vec<StyledSpan>;  // 纯函数(CR1/R8)

pub fn fallback_render(PartKind, &RenderPart, &RenderCtx) -> Vec<StyledSpan>;  // Plan 22 兜底

pub struct RenderRegistry { /* PartKind→RenderFn + fallback */ }
impl RenderRegistry {
    pub fn new() -> Self;                                  // 缺省全兜底
    pub fn register(&mut self, PartKind, RenderFn);        // Plan 23 注册 specific 覆盖
    pub fn render(&self, PartKind, &RenderPart, &RenderCtx) -> Vec<StyledSpan>;  // 命中 specific 否则兜底
    pub fn has_specific(&self, PartKind) -> bool;          // 可观测/测试
}
```

## 3. 不变量(为什么这样能并行且永远完整)

1. **Plan 22 单独 = 完整(丑)UI**:`RenderRegistry::new()` 全兜底 → 任何 part(含 `Unknown`)都渲染成「标签 + 原始内容」,不 panic(AR12)。
2. **Plan 23 逐类覆盖,未覆盖走兜底**:`register(kind, pretty)` 只换该 kind;其余继续兜底 → **UI 始终完整,只是越来越漂亮**。
3. **加渲染器 = 注册一行**(0006/0032):不动事件/状态/`build_frame` 骨架,冲突面极小。
4. **纯函数(CR1/R8)**:`RenderFn` 无平台依赖、同输入同输出 → 兜底与 specific **共用同一套渲染快照测试**;入录像重放 oracle。
5. **输出对齐既有契约(0001)**:`Vec<StyledSpan>` 直接喂 layout;Plan 23 的面板/diff 等富图元在 specific 渲染器内额外产出(本契约只锁 StyledSpan 这条主线,富块作扩展)。

## 4. 谁产谁消

| | Plan 22(生产 + 兜底) | Plan 23(消费 + specific) |
|---|---|---|
| `RenderPart` | **填**(store→投影) | 只读 |
| `RenderRegistry` | **建** + 注册兜底 | `register` specific |
| `RenderFn` | `fallback_render` | `render_tool_bash` / `render_reasoning` / … |
| `build_frame` | 经 registry 出块(走 tier) | 不碰 |
| 事件/状态/store | **全拥有** | 不碰 |

## 5. 决策与不决定的

**采纳**:`partrender` 契约(`PartKind` / `RenderPart` / `RenderCtx` / `RenderFn` / `fallback_render` / `RenderRegistry`)为 Plan 22↔23 唯一接触面,**已实现于 `crates/core/src/partrender.rs`(含单测)**。Plan 22 填投影 + 装兜底 + 经 registry 出块;Plan 23 只 `register` specific 渲染器。

**理由**:把唯一接触面冻结成独立 ADR + 代码,使两份 plan 真并行、互不阻塞、UI 始终完整;数据驱动(0006/0032)使加渲染器零骨架改动;纯函数保 CR1/R8 可测。

**不决定的(留实现)**:`RenderCtx` 字段扩充(theme/status 细节随 Plan 23 需要加,**向后只增不改**);富图元(panel/diff)如何作为 StyledSpan 之外的第二输出(Plan 23 §1 定,本契约先锁 StyledSpan 主线);`PartKind` 是否细分 tool 子类(目前 tool 名走 `kind_tag` 字符串 + Plan 23 二级分派,不进 `PartKind`);`RenderPart` 是否带 part_id 给折叠态(Plan 23 命中层需要时加)。

---

## 6. 实现 + 契约一致性闸(保护并行)

见 `crates/core/src/partrender.rs`:契约类型 + `fallback_render` + `RenderRegistry` + 单测 + **契约一致性 harness**。`crates/core/src/lib.rs` 导出。

**`assert_renderfn_conforms(f: RenderFn)`(契约一致性闸)**——这是让 Plan 22↔23 **安全并行**的关键验证:**任何注册进 registry 的渲染器(兜底 + Plan 23 的每个 specific)都必须过它**。断言三条不变量:① 任意对抗性 part 不 panic;② 确定性(同输入同输出,R8);③ 非空输入→非空输出(内容不丢)。配一组对抗样本(空/只标签/只 json/畸形 unicode/超长 × 全 `PartKind`)+ 兜底的 proptest。

> **用法(Plan 23 每个 specific 渲染器的测试)**:`crate::partrender::assert_renderfn_conforms(my_specific_render);` —— 注册一行渲染器,就过同一道契约闸。某 specific 若 panic/非确定/丢内容,**立刻红,不会悄悄破坏 Plan 22 端与重放端**。

参考:[Plan 22 §3](../plan/plan22-opencode-events-and-fsm.md)/[Plan 23](../plan/plan23-part-render-implementation.md)· [0001](0001-canvas-architecture.md)(StyledSpan 契约)· [0006](0006-inline-tags-and-extensibility.md)/[0032](0032-interaction-architecture-sdf-world-not-component-framework.md)(数据驱动注册表)· [业界调研 §2](../research/agent-ui-industry-survey.md)(收敛的 parts+registry+lifecycle)。
