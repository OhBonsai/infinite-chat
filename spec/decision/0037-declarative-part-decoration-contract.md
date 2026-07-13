# 0037 · part 装饰声明式契约(DecorationOp)—— diff 卡首租

- 日期:2026-07-13(Plan 32 D0;GOAL 原文写 0035,该号已被 plan31 math ADR 占用 → 顺延本号)
- 状态:**采纳**(0033 渲染契约的 additive 扩展;0032「装饰=数据非 widget 对象」的深化;
  plan33 crate 拆分的前提)
- 参考:[0033](0033-part-render-contract.md) · [0018 panel 图元](0018-sdf-panel-decoration-primitive.md) ·
  [0021 主题运行时](0021-runtime-theme-tokens.md) · [research/code-diff-rendering-zed-makepad](../research/code-diff-rendering-zed-makepad.md)

## 1. 问题

part 的**装饰**(diff 行底/gutter 条/词级高亮…)一直长在 `app.rs build_frame` 里,靠扫描
StyleRole 逐行反推几何 —— 渲染器(partspecific)只能经"角色"间接表达装饰意图,加一种装饰 =
改 app.rs 一段扫描。组件不自治,crate 拆分(plan33)无从谈起。

## 2. 决策

RenderFn 契约 additive 扩展:specific 渲染器除 `Vec<StyledSpan>` 外可产
**声明式装饰指令**:

```rust
pub struct DecorationOp {
    pub kind: DecorKind,      // Band(整宽行底)/ Gutter(左缘条)/ CharBg(字符级底)
    pub span: (u32, u32),     // 锚定:渲染后 display glyph 区间 [start,end)(与 clusters 对齐)
    pub slot: DecorSlot,      // 语义色槽 —— 色值 emit 时经 theme 解析(0021:颜色不进缓存)
    pub group: u32,           // 分组 id(gloop 邻接融合 / hunk 归属)
}
pub struct RenderOutput { pub spans: Vec<StyledSpan>, pub decorations: Vec<DecorationOp> }
```

- **锚定 = glyph 区间,不是几何**:renderer 纯函数(CR1/R8),不知盒宽/折行;app 摆放时把区间
  经 `cache.placed` 解析成世界矩形(逐行分段)。区间随 0020 身份稳定,虚拟化重建(plan29)零特判。
- **色 = 语义槽**:`DecorSlot` → theme 字段映射在 emit 端解析,换主题下一帧生效(0021),
  装饰随块缓存不失效。
- **注册**:`RenderRegistry::register_full(kind, FullRenderFn)`;旧 `RenderFn` 自动包装
  (decorations 空)→ 既有渲染器零改动。
- **首租 = diff**:`render_diff` 产 spans + ops;`app.rs` 的 `flush_diff_band` 逐行反推路径
  **删净**(0→1 不留双轨)。其余 part(tool 卡底/引用条/代码面板)本期不迁(Surgical
  Changes),契约留位,plan33 逐租迁入。

## 3. app 摆放规则

`block_decorations` 收 `cache.decorations`:
- `Band`:区间内按行分段(y 相同的 placed 连续段),每行一条整宽 rect(x = 盒左缘,w = 盒宽);
  同 `group` 相邻行交 gloop(0018 扩参,Plan 32 D3)。
- `Gutter`:区间行范围的左缘细条(宽 = `floor(0.275 × 行高)`,Zed 语义值)。
- `CharBg`:区间内逐段 glyph AABB 圆角小底(词级高亮)。

## 4. 不变量

装饰指令随 `BlockCache` 冻结(纯数据);settled 块装饰恒等(AR3);未知 `DecorKind`/`DecorSlot`
兜底不画不 panic(AR12);同输入同指令(R8,进渲染器契约一致性闸)。
