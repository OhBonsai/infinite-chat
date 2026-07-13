# Plan 33 · GOAL:crate 分层拆分 —— primitives / components 抽离,纯搬家零行为变化(agent 直跑,无人介入)

- 日期:2026-07-13
- 状态:**GOAL(可执行任务书)** —— 给 Claude Code 的完整执行合同,读完即可自主跑到 DoD。
- 执行方式:**顺序执行,master 工作区直跑;必须单独跑**——本 GOAL 是全仓机械搬家(mod 路径大面积变动),**不与任何功能 plan 交替**,前置 plan32 完成(装饰契约 ADR 0035 已生效,diff 卡已自治)。
- 定位:把「SDF 图元词汇」与「part 组件」从 core 抽成独立 crate——**分层不是组件框架**(0032 铁律:一切仍是数据 + 纯函数,无 widget 对象/无保留树);产出新 ADR 0036 记分层决策。
- 动机(作者确认):① 遏制 app.rs(6.4k 行)/core(16.5k 行)巨石化;② 组件(diff 卡/code view/tool 卡)将来可被第二宿主消费(类比 makepad `CodeView` 只读组件的形态);③ 编译并行 + 测试边界。
- 前置:
  - plan32 的 **ADR 0035 装饰声明式契约**(renderer 产 spans + DecorationOp——没有它组件是"哑"的,拆了白拆)
  - 已核实切缝:`partrender.rs`/`partspecific.rs` 对内仅依赖 `content::{StyleRole, StyledSpan, parse_markdown, plain}`(2026-07-13 grep 验证)
  - [dev-practices §4.2 CR 铁律](../dev-practices.md) · [0026 shader 模块化](../decision/0026-modular-shader-organization.md)(render 侧已按图元分文件,天然对口)· [0032](../decision/0032-interaction-architecture-sdf-world-not-component-framework.md)

---

## 0. 分层目标形态(ADR 0036 的骨架)

```
crates/
  primitives/   ← 新:图元词汇 + token(零依赖,CR1 同款)
      frame.rs(FrameGlyph/FramePanel/FrameRect)· shaderbox.rs · theme.rs · motion.rs
      + 类型下沉:StyleRole / StyledSpan / DecorationOp(0035)(纯数据,从 content/partrender 下沉)
  components/   ← 新:part 组件 = 纯函数「语义 → spans + decorations」
      partrender.rs(registry/契约)· partspecific.rs · diffmodel(plan32 产物)
      · codeblock.rs · highlight.rs(依赖按 C1 审计结果定)
  core/         ← 留:引擎机制
      store/fsm/smoother/protocol/content(解析主体)/reveal/rhythm/app/spatial/boxlayout/camera/embed/math/nodes/…
  render/       ← 不动(依赖 primitives 的图元类型)
  wasm/         ← 不动(薄壳,CR5)
依赖方向:primitives ← {components, core, render};components ← core(core 注入 registry);禁反向。
```

**命名注意**:crate 名避免"framework"气味;`primitives`/`components` 或 `sdf-vocab`/`parts`,ADR 定夺(倾向前者,直白)。

## 1. GOAL 与完成定义(DoD)

**GOAL**:按 §0 形态把 core 拆成 primitives + components 两个新 crate,**纯移动零重写零行为变化**;组件层获得独立可消费性(一个不依赖 core/transport/store 的 native 示例直接把 RenderPart 渲染成 spans+decorations)。

**DoD(全满足才算完)**:

1. **ADR 0036 就位**:分层边界、依赖方向、类型下沉清单、命名、"何时该再拆/何时不该"(防过度拆分条款);append-only 新编号。
2. **两 crate 成立**:workspace 编译;`primitives` 零外部依赖(std only,或最多 serde derive——审计后 ADR 定);`components` 只依赖 `primitives`(若 content 解析函数拆不动,components 对 parse 的依赖改为**函数参数注入**,保持 crate 级零依赖 core)。
3. **纯搬家证明(硬门)**:全部 golden 帧**逐字节不变**;native 快照零变化;`node test/run.mjs` 四层全绿。任何行为 diff = 不合格。
4. **独立消费示例**:`crates/components/examples/render_part_dump.rs`(native)——喂一个 tool/diff RenderPart JSON,打印 spans + decorations;`cargo run --example` 可跑,不链 core/wasm/render。这是"第二宿主可用"的最小证明。
5. **卫生门**:每个新 crate 有 README + `//!` mod doc(R5);`cargo deny check` 过;公开 API 面最小化(R4:能 `pub(crate)` 不 `pub`);编译时间 before/after 记 progress(全量 + 增量改 components 单文件两档)。
6. **app.rs 减负数字**:搬出的行数记账(预期 partspecific/codeblock/highlight/frame/theme/motion/shaderbox ≈ 3k+ 行离开 core);app.rs 本身只改 use 路径。
7. `spec/plan/plan33_progress.md` 逐 milestone 记录。
8. **按 milestone commit,不 push**;每个 commit 前全门绿(搬家中间态也必须全绿——按依赖顺序搬,不留断头)。

**启动**:`claude --permission-mode acceptEdits "读 spec/plan/plan33-components-crates-goal.md,按其执行到 DoD 全满足"`(中断重跑同句,读 plan33_progress 续跑)。

## 2. Milestones(每步全门绿,可中断)

- **C0 · 依赖审计 + ADR 0036**:精确 grep 每个候选文件的 `use crate::` 面(重点:codeblock/highlight/theme/motion 是否反向咬 app/store;DecorationOp 在 0035 里的归属);产类型下沉清单;写 ADR(含 §0 图 + 禁反向依赖 + 防过度拆分条款);**发现拆不动的文件就留 core 并记原因,不硬拆**。
- **C1 · primitives crate**:建 crate → 平移 frame/shaderbox/theme/motion + 下沉 StyleRole/StyledSpan/DecorationOp → core/render 改 use 重导出(`pub use primitives::*` 过渡镜像,保外部 API 不变)→ 全门绿 + golden 逐字节。
- **C2 · components crate**:平移 partrender/partspecific/diffmodel(+ 审计通过的 codeblock/highlight)→ core 注入 registry 处改依赖 → 独立消费示例(DoD-4)→ 全门绿 + golden 逐字节。
- **C3 · 收口**:删过渡重导出里已无引用的项(grep 证明)→ README/mod doc/deny → 编译时间对比入 progress → architecture.md 补一节「crate 分层」(M 域映射不变,标注各域所在 crate)→ AGENTS.md §4 Rust workspace 行更新。

## 3. 客观判定

| 维度 | 判据 |
|---|---|
| 零行为变化 | golden 全集逐字节;native 快照零 diff;四层门绿(每个 milestone) |
| 依赖方向 | `cargo tree -i` 断言:primitives 不依赖 workspace 任何 crate;components 不依赖 core/render/wasm |
| 独立消费 | example 在 `cargo build -p components --examples` 下编译运行,link 面 grep 无 core |
| 纯移动 | 每个搬家 commit 的 diff 中,搬移文件内容变化仅限 `use` 路径与 `pub` 可见性(review 抽查) |
| 编译收益 | 增量改 components 单文件的重编译时间 < 改 core 同文件 before 值(数值入 progress) |

## 4. 铁律(承 AGENTS.md / dev-practices §4)

- **纯搬家纪律(本 GOAL 第一铁律)**:禁止顺手重构/改名/优化(Surgical Changes 极端化)——发现想改的记 progress 遗留,不动手。行为 diff = 立即回退该步。
- **CR1→CR1'**:原"core 零平台依赖"扩展为"primitives/components/core 均零平台依赖"(ADR 0036 正式化)。
- **CR5** wasm 薄壳不变 · **0032** 分层≠框架:components 仍是纯函数注册表,无对象树、无生命周期、无事件系统。
- **R4/R5** 新 crate API 面最小 + README/mod doc · **T6** 搬家后测试跟着文件走(组件测试进 components)。
- **按 milestone commit,不 push**;中间态全门绿(搬家顺序按依赖拓扑:先 primitives 后 components)。

## 5. 变更边界

- **触碰**:Cargo.toml(workspace)、被搬文件的 use/pub、core/render 的 use 路径、architecture.md/AGENTS.md 两处文档行。
- **不触碰**:任何函数体逻辑、shader、测试断言值、web/ TS(wasm API 不变故 TS 零感知)。
- **执行窗口**:单独跑,期间不排其它 plan(rebase 面 = 全仓 use 路径,交叉必炸)。

## 6. 风险与预案

| 风险 | 预案 |
|---|---|
| content.rs 解析主体与 StyledSpan 类型耦合过深拆不净 | 只下沉类型(StyleRole/StyledSpan 是纯数据),parse 函数留 core;components 需要 parse 时经函数指针/参数注入(审计 C0 定) |
| 循环依赖浮现(如 theme 咬 app 常量) | C0 审计先行;咬点就地断开(常量归 primitives)或该文件留 core 记遗留 |
| 过渡重导出永久化 | C3 强制删空 + grep 断言;ADR 写明镜像只活到 C3 |
| golden 因浮点/编译单元变化漂移 | 理论上纯搬家不该漂;若漂 = 有隐藏行为依赖(如 const 求值顺序),视为 bug 排根因(T10),不重录 golden |

> 一句话:**plan32 让组件会"说"装饰,plan33 让它搬进自己的房子**——primitives 是词汇表,components 是组件库,core 只剩引擎;golden 逐字节不变是唯一硬通货。
