# Plan 33 · 进度账本(crate 分层拆分)

> 逐 milestone 记录:做了什么 / 依赖审计结果 / golden 逐字节证明 / 编译时间数值 / 遗留。
> GOAL:[plan33-components-crates-goal](./plan33-components-crates-goal.md)
> 前置:plan32 完成(ADR 0035 装饰契约生效)。

## C0 · 依赖审计 + ADR 0039(GOAL 原文 0036,号被 plan31 占用 → 顺延)

- **ADR**:[0039-crate-layering-primitives-components](../decision/0039-crate-layering-primitives-components.md)
  —— §0 图、依赖方向、类型下沉清单、parse 注入方案、防过度拆分条款(三判据)。
- **审计结果(grep 2026-07-14)**:
  | 文件 | crate 依赖 | 归属 |
  |---|---|---|
  | frame.rs (241) | 无 | primitives |
  | shaderbox.rs (246) | 无 | primitives |
  | theme.rs (208) | 无 | primitives |
  | motion.rs (143) | theme(同迁) | primitives |
  | highlight.rs (476) | 无 | components |
  | codeblock.rs (151) | 仅 doc 注释提 app(改注释) | components |
  | partrender.rs (467) | content::{parse_markdown,plain,StyleRole,StyledSpan} | components(类型下沉 + parse 注入) |
  | partspecific.rs (1242) | partrender ×12、support::graphemes ×5、content ×2 | components(同上) |
  | support.rs (230) | content/frame(测试 harness) | **留 core**,只下沉 graphemes |
- **parse 注入点**:partrender 3 处 + partspecific 4 处 → `RenderCtx.parse: fn(&str)->Vec<StyledSpan>`
  (无 Default,构造点显式注入;components 测试 dev-dep core 注真 parse,cargo 允许 dev 环)。
- **下沉清单**:StyleRole/StyledSpan(content→primitives)、DecorationOp/DecorKind/DecorSlot/
  RenderOutput(partrender→primitives)、graphemes(support→primitives)。
- **搬移量预估**:components ≈2.3k 行 + primitives ≈1k 行 ≈ 3.3k 行离 core(DoD-6 达标)。

## C1 · primitives crate

- **搬移(git mv 保历史)**:frame.rs / shaderbox.rs / theme.rs / motion.rs →
  `crates/primitives/src/`(四文件 crate 依赖仅存在于 doc 注释,代码级零依赖,C0 审计证)。
- **类型下沉**:`style.rs` ← content 的 StyleRole/StyledSpan(248 行);`decoration.rs` ←
  partrender 的 DecorationOp/DecorKind/DecorSlot/RenderOutput(59 行);`text.rs` ←
  support 的 graphemes。
- **强制迁位一处**:`StyleRole::from_code_class` 咬 highlight(components 域)→ 改为
  content.rs 自由函数 `role_from_code_class`(唯一调用点同文件;纯位置迁移)。
- **镜像**:core 的 frame/theme/motion/shaderbox.rs 变 `pub use primitives::*` 单行镜像;
  content/partrender/support 局部 `pub(crate) use` 镜像 —— crate 内外路径零变化;
  **对外 API 面零扩**(DecorationOp 族原经私有 mod 不可达,撤了误加的门面导出)。
- **可见性调整(搬家必然)**:primitives 内 8 处 `pub(crate)` → `pub`(跨 crate 可达;
  API 面收束留 C3 复核)。
- **依赖**:primitives 仅 serde/serde_json/unicode-segmentation(零 workspace 依赖,
  `cargo tree` 断言见门日志);core += primitives。
- **验证**:workspace clippy -D warnings 0 错;全 workspace 测试零失败(测试随文件迁入
  primitives 照跑);全门 + golden 逐字节见本节门结果。

## C2 · components crate + 独立消费示例

(未开始)

## C3 · 收口(重导出清理 / 文档 / 编译对比)

(未开始)

## DoD 对账

(未开始)
