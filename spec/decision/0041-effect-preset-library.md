# 决策记录 0041:效果预设库 —— 场景槽 × 预设的数据驱动效果系统

- 日期: 2026-07-14(Plan 38 E0)
- 状态: 已采纳
- 前置: [0002 §5.1](0002-event-driven-pipeline.md)(效果=数据的终局)· [0018](0018-sdf-panel-decoration-primitive.md) ·
  [0025](0025-sdf-node-animation-system.md) · [0037](0037-declarative-part-decoration-contract.md) ·
  [0040](0040-feedback-and-post-pass.md)(36 造词汇、37 开通道 —— 本篇编字典)
- 范围: 把 plan36/37 的散装效果参数组织成「场景 × 预设」数据表;换一套效果 = 换一份数据,零代码。

## 1. 背景 / 问题

效果参数散落各处(enter profile 硬编码于 emit、dissolve/spring/trail/post 各有 setter、
idle 呼吸与 shimmer 内联)——"当前默认观感"没有单一真值源,"换一套气质"要改代码。

## 2. 决策

```rust
// primitives::effects(纯数据,serde;JSON 可加载,坏数据整份拒绝回退当前档 AR12)
pub struct EffectPreset {
    pub name: String,
    pub enter: EnterSpec,      // 曲线(default/spring)+ 时长因子
    pub exit: ExitSpec,        // dissolve 时长(0=即时)
    pub emphasis: EmphasisSpec,// hit-flash 强度/时长(触发器由调用方)
    pub idle: IdleSpec,        // 呼吸脉冲幅度/频率(0=无)
    pub thinking: ThinkingSpec,// shimmer 开关 + 拖尾 decay(0=无)
    pub hover: HoverSpec,      // 链接下划线强度 / 卡片提亮
    pub celebrate: CelebrateSpec, // v1 显式 None(粒子层远期)
}
```

- **内置三档**:`off`(全恒等)/ `subtle`(默认 —— **现观感的显式化**,把散落默认值收进
  一份数据,从此默认观感有单一真值源)/ `expressive`(36/37 词汇的保守组合:enter=spring、
  exit=dissolve 400ms、emphasis=flash、idle=呼吸、thinking=shimmer+微拖尾)。
- **正交性**:EffectProfile(full/reduced/off)与 preset 正交 —— profile off 压制一切
  (置零),reduced 减档;preset 决定"哪种",profile 决定"多少"。
- **接线原则(E1 硬门)**:七槽全部查表取参,替换硬编码默认(行为等价重构 —— subtle 档
  golden 逐字节证明);未接效果的槽显式 None,禁"部分查表部分硬编码"双轨(0→1)。
- **扩展条款**:新增效果 = schema 加字段(向后 serde default);新增预设 = 加一份数据;
  改数值必过 insta 快照(同 motion token 惯例)。

## 3. 七槽现状盘点(E0 audit,2026-07-14)

| 槽 | 现取参点 | 收编为 |
|---|---|---|
| Enter | app.rs:3809 `enter_profile_id` + `spring_enter` 开关(N4) | EnterSpec.curve |
| Exit | app.rs:1468 `exit_dissolve_ms`(N3) | ExitSpec.dissolve_ms |
| Emphasis | rect fx mode 4(N3,无默认触发器)+ motion.arrive_boost(app.rs:4169) | EmphasisSpec |
| Idle | app.rs:3445/3452 WIDGET_PULSE 呼吸(plan25 M2a 内联参数) | IdleSpec |
| Thinking | app.rs:3517 shimmer_title + feedback_decay(F1) | ThinkingSpec |
| Hover | app.rs:3640 link_underline + M2f pointer 提亮 | HoverSpec |
| Celebrate | 无 | None(记遗留) |

## 4. 备选与否决理由

- **preset 放 web 层(JSON→逐 setter)**:切换非确定性输入、不进重放;否决(R8:preset
  切换须是录像可含的确定输入)。
- **每槽独立 setter 维持现状**:无单一真值源,"换气质"永远是代码活;否决。

## 5. 影响与实现入口

- 正:效果系统三层闭环(36 词汇/37 通道/38 字典);默认观感获得单一真值源。
- 负:七槽取参点一次性改道(E1 行为等价硬门兜底)。
- 入口:`crates/primitives/src/effects.rs` · core 七槽查表 · wasm `set_effect_preset` ·
  记账 [plan38_progress](../plan/plan38_progress.md)。
