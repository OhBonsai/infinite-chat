# Plan 38 · 进度账本(效果预设库)

> 逐 milestone 记录:做了什么 / 七槽取参点清单(file:行号)/ 快照结果 / 遗留。
> GOAL:[plan38-effect-preset-library-goal](./plan38-effect-preset-library-goal.md)
> 前置:plan36 完成;plan37 可选(反馈类槽位)。

## E0 · ADR + schema(七槽盘点)

- **ADR**:[0041-effect-preset-library](../decision/0041-effect-preset-library.md) ——
  schema、三档定义、profile 正交、接线原则(E1 硬门)、扩展条款、**七槽现状盘点表**
  (Enter app.rs:3809 / Exit :1468 / Emphasis rect-fx4 + :4169 / Idle :3445 /
  Thinking :3517 / Hover :3640 / Celebrate 无)。
- **schema**(`primitives/src/effects.rs`,纯数据 serde):`EffectPreset` 七槽;
  `Default = subtle`(现观感显式化:idle 0.15Hz/±8%、hover 下划线 0.9、exit 0、spring off);
  `off()` 全恒等;`expressive()` 保守组合(spring / dissolve 400 / flash 0.6 / trail 0.7);
  `builtin(name)` 未知名 None(AR12 不半应用)。
- **测试**:三档 insta 快照 + JSON 往返(缺字段默认/坏数据整拒)。

## E1 · subtle 显式化(行为等价重构,golden 硬门)

(未开始)

## E2 · off / expressive + API + 面板

(未开始)

## E3 · 收口(三档断言 / celebrate 时间盒 / thinking §4 对账)

(未开始)

## DoD 对账

(未开始)
