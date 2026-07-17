# Plan 44 · 进度账本(组件 Tile 墙)

> 逐 milestone 记录:排雷清单 / 设计表定稿 / 对齐断言数值 / 遗留。
> GOAL:[plan44-component-tile-wall-goal](./plan44-component-tile-wall-goal.md)
> 需求确认:2026-07-15 四问(引擎 tile 布局 / 两处落点 / 静态+hover 才播 / 固定设计表)+ tile 即 SDF。

## T0 · ADR + 组件全清单 + 尺寸设计表(交作者过目)

✅ 交付(2026-07-17):
- **ADR [0042](../decision/0042-tile-grid-layout-mode.md)**:tile 网格布局模式——数据契约(`TileSpec`/`TilePlacement`
  serde 落 primitives)、固定 4 列基网格 + 固定 row_h(Metro 精确整数格 → 对齐=摆位必然)、skyline first-fit 摆位、
  per-tile 折行宽(唯一侵入点,单列 `None` 恒等)、静态语义(follow=Released + 全揭示 + hover 单 part 重播)、
  与单列模式边界(`LayoutMode{Column,Tile}`,非 tile 恒等硬门 + 坏 manifest AR12 整拒)、实现入口。
- **清单 + 设计表 [component-tile-manifest](../design/component-tile-manifest.md)**:五档尺寸(大 4×2/中 2×2/横条 4×1/
  标准 2×1/小 1×1)+ Agent 卡墙 9 tile + Markdown 墙 19 tile,逐 tile id/span/标题/写死内容 + skyline 摆位。
- **过目**:自主定档(承 plan43「keep going」授权 + 本 GOAL「直跑无人介入」);ADR/清单即评审物,单独 commit。

## T1 · 引擎 tile 布局器(恒等硬门 + 单列假设排雷)

(未开始)

## T2 · /components/ 页(两面墙 + hover 才播 + 降级)

(未开始)

## T3 · 首页 S3/S4 换墙(导演文档修订)

(未开始)

## T4 · 收口(对齐/锐利/idle/恒等断言 + 文档)

(未开始)

## DoD 对账

(未开始)
