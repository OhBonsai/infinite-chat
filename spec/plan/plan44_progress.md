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

✅ 落地(2026-07-17):
- **`primitives/tile.rs`**:`TileSpec`/`TilePlacement`(serde + `from_json` + `is_valid`);坏 span/空 → 非法。
- **`core/tilelayout.rs`**:skyline first-fit 摆位器 → 每 tile `TileRect`(整数格)+ 网格步长 + 墙总高。
- **`core/app.rs` 集成**:`tile_spec: Option<TileSpec>` 字段(默认 None)+ `set_tile_manifest(json)`(AR12 整拒)+
  `active_tile_spec`/`tile_view_widths`;`build_frame` tile 分支(`boxpos` 覆盖每 view origin/width + 采 chrome +
  墙总高 → content_height/max_pan_y);`ensure_layouts` per-view 折行宽(tile 内宽);tile chrome = 每 tile 一个
  `FramePanel`(圆角 + 边 + 弱 AO + header 色带,标题作内容首行落带内);id 高位 `0xF11E` 免撞。
- **`wasm/lib.rs`**:`set_tile_spec(json) -> bool` 薄转发。
- **测试**:tilelayout 单测 4(**逐 tile 四边落网格线** + skyline + 填缝 + 列宽)+ app 引擎测 2(端到端:3 消息→3
  panel 落格 + follow=Released;坏 manifest 退单列无 chrome)。

**单列假设排雷(风险 §6)结果**:
- follow/锚底 → tile 模式 `set_tile_spec` 强制 `follow=Released`(静态墙不锚底);`max_pan_y`/`revealed_height`/
  `content_height` 在 tile 模式取网格外接盒 `total_h`(墙非 y-单调,反扫揭示前沿失效)。✅
- `HeightIndex`(一维 y-单调)→ tile 墙非单调,broad-phase 略过选(~30 块无感,**不改索引结构**);记遗留。
- `origin.x=列左` → `boxpos` 覆盖每 tile origin;`block_decorations` 吃传入 `box_w`(per-tile),满宽装饰按 tile 宽走。✅
- per-view 折行宽 → `ensure_layouts` tile 分支吃 tile 内宽;单列 `None` → `wrap_width` 恒等。✅

**恒等硬门**:`replay_settled_frame_snapshot`(insta golden)+ 确定性测**逐字节不变**;clippy/fmt 绿。tile 是纯旁路(None 时零触碰)。

## T2 · /components/ 页(两面墙 + hover 才播 + 降级)

(未开始)

## T3 · 首页 S3/S4 换墙(导演文档修订)

(未开始)

## T4 · 收口(对齐/锐利/idle/恒等断言 + 文档)

(未开始)

## DoD 对账

(未开始)
