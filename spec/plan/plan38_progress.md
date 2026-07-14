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

- **Engine 收编**:`effect_preset: EffectPreset`(默认 subtle)取代散装
  `spring_enter`/`exit_dissolve_ms` 字段(编译期保证无旁路 —— 即 DoD「grep 断言无
  preset 外硬编码」的强化版);既有 setter(set_spring_enter/set_exit_dissolve_ms)改写
  preset 槽保 wasm 兼容;新 `set_effect_preset(name)`(未知名不动当前档 AR12)+
  `effect_preset_name()`。
- **七槽接线**:Enter=emit spring 判定;Exit=orphan dur/进度/getter 三处;
  Idle=两站点呼吸(hz 查表、amp=站点基值×主控缩放,subtle=1 等价);
  Thinking=shimmer 门 + streaming 期 trail(max(手动, preset),subtle=0 恒等);
  Hover=下划线 alpha(theme RGB + preset alpha,subtle 0.9=现值);
  Emphasis/Celebrate=数据在册、触发器显式缺位(0041 允许 None,遗留)。
- **IdleSpec 语义**:amp 定为主控缩放(1=站点基值原样)—— 两站点基值不同(0.05/0.08),
  绝对值语义无法单值等价;快照随之更新(显式过审)。
- **native**:off 档恒等面(shimmer 位不出/呼吸幅度全 0/无拖尾/退场即时/未知名拒绝);
  全量 254 零回退。golden 逐字节 = 本门 e2e 佐证(subtle 默认)。
- **踩坑**:批量替换脚本中途断言失败会丢弃已成功的前段替换(文件末尾统一落盘)——
  重放三处丢失的槽位接线后 off 测才绿。

## E2 · off / expressive + API + 面板

- **API**:core `set_effect_preset_json`(整份 JSON,坏数据整拒 AR12);wasm 三导出
  `set_effect_preset(name)→bool` / `effect_preset_name()` / `set_effect_preset_json`。
- **面板**:Effects 节顶部预设下拉(subtle 默认/off/expressive)。
- **e2e**:单测覆盖 —— 默认 subtle、切 expressive 生效、未知名不动当前档、切 off、
  自定义 JSON 生效(数据驱动零编译判据)、坏 JSON 不半应用。

## E3 · 收口(三档断言 / celebrate 时间盒 / thinking §4 对账)

(未开始)

## DoD 对账

(未开始)
