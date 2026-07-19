# Plan 46 · 进度账本(TUI Cell 网格布局器)

> 逐 milestone 记录:参考依据(file:行号)/ native 数值 / 校准方案 / 恒等验证 / 遗留。
> GOAL:[plan46-tui-cell-layout-goal](./plan46-tui-cell-layout-goal.md)
> 上游:承 plan45 U3 剩余遗留①(cell-grid 逐字宽)+ ②(动态兄弟间距 + `⎿` 前缀);调研 research/tui-events-and-layout §3/§4/§6。
> 起点事实(2026-07-19 摸查):字宽/advance/CJK 判定现全在 JS(`layout-bridge.ts`:`layout` :466 / `advanceFor` :232 / `measureText` :242 / `isCJK` :246);core 侧零字宽、无 EAW 表;`ensure_layouts`(app.rs:3138)仅选 registry 时看 flavor,逐字 layout 调用(app.rs:3279)对 flavor 一无所知——**这是分支接管点**。flavor 已就位:`RenderFlavor`(app.rs:1550)/ `is_tui`(:2104)/ `set_render_flavor`(:2084)/ `Globals.flavor`(plan45 C1)。MSDF baked advance 已存在但只在 JS:`msdf.ts` `advances` :29 / `msdfAdvancePx` :42。

## L0 · 校准通道 + EAW 宽度表

✅ (2026-07-19)
- [x] `crates/core/src/cellwidth.rs`:`char_cells(ch)->u8`(0/1/2),扩自 `layout-bridge.ts:246` isCJK 区间。
- [x] `CellMetrics{cell_w,cell_h}` + `App.cell_metrics` + `set_cell_metrics`(AR12 坏值≤0/NaN 拒退原路 + mark_layout_dirty)。
- [x] wasm `set_cell_metrics(w,h)` 转发。
- [x] native:`char_cells` 5 测(拉丁1/CJK2/emoji2/组合0/混排)+ `cell_metrics_injects_and_rejects_bad`(AR12)。

## L1 · cell 布局器 v0(单列拉丁)

✅ (2026-07-19)
- [x] `crates/core/src/celllayout.rs`:`layout_cells(spans, wrap_cols, metrics) → LayoutResult`(同 seam 结构)。
- [x] `ensure_layouts` 分支(app.rs 逐字 layout 调用点):`tui_cell_active()`(tui+Column+已校准)→ celllayout;否则 `self.layout.layout`。
- [x] **rich 恒等硬门**:`replay_settled_frame_snapshot` insta golden 逐字节不变(验)。
- [x] native:`latin_places_on_integer_cell_grid`(x=col*cell_w)+ `word_preferred_wrap_at_space` + `newline_resets_col_advances_line`。

## L2 · CJK 2-cell + 折行禁则

✅ (2026-07-19)
- [x] `char_cells` 接入(CJK/宽 `col+=2`);`HEAD_BAN` 码点数组行首禁则(抄 layout-bridge.ts:260)+ CJK 逐字可断点。
- [x] native:`cjk_two_cells_and_mixed_advance`(中=2cell,混排 x 累加)+ `head_ban_hangs_to_prev_line`(句号悬挂不落行首)。
- ★ **坑**:celllayout 里 CJK 注释触发 clippy/rustc span ICE(`rustc_span` multi-byte 断言,pedantic 下)→ 该文件注释改全 ASCII 规避;HEAD_BAN 用 `\u{}` 码点数组(非字面 CJK)。

## L3 · 渲染侧 cell 对齐(布局宽≠渲染宽)

✅ (2026-07-19)
- [x] 布局盒宽 = `n×cell_w`(layout_cells 写死;CJK n=2)。校准 `cell_w = measureText('M')` = mono 渲染 advance → **布局宽=渲染宽**(mono),glyph 落格无错位(截图验:CJK/拉丁均齐)。
- [x] native:`cjk_two_cells_and_mixed_advance` 锁盒宽 = n×cell;`replay_tui_cell_frame_snapshot` golden 锁逐字 col。
- 遗留:非 mono 回退字形的亚 cell 居中(x 内缩 `(box-adv)/2`)—— mono 路 adv=box 无需,回退字记后续(§6 风险)。

## L4 · 收口(校准面 / 剩余间距 / golden / 门)

✅ (2026-07-19)
- [x] 校准面:`layout-bridge.calibrateCellMetrics()`(cell_w=mono 'M' advance / cell_h=LINE_HEIGHT[tui 1.25×])→ chat applyFlavor(tui) 注入 `set_cell_metrics`(开机一次,非每帧);rich 清 `set_cell_metrics(0,0)` 退原路。
- [x] golden 家族:rich `replay_settled_frame_snapshot`(恒等)+ tui `replay_tui_cell_frame_snapshot`(逐字 col 整数,CJK 世@col3 size18)。
- [x] native ≥5(实际 11:cellwidth 5 + celllayout 5 + metrics AR12 1)。
- [x] e2e:`cell:tui 逐字进 Rust`(强制重排 JS layout 回调 delta==0)+ `cell:rich 对照`(delta>0);measureText 仅开机校准。
- 遗留:**剩余间距 2 条**(动态兄弟间距 + tool 结果 `⎿` 前缀)—— 显示细节,非布局关键,承 plan45「后续」记后续;真终端截屏。

## DoD 对账

| # | DoD 要求 | 状态 | 证据 |
|---|---|---|---|
| 1 | EAW 宽度表(core 纯函数) | ✅ | cellwidth::char_cells + 5 native |
| 2 | 校准通道(注入,非每帧) | ✅ | set_cell_metrics(AR12)+ calibrateCellMetrics;e2e measure 稳态 0 |
| 3 | core cell 布局器 | ✅ | celllayout::layout_cells + 5 native + golden |
| 4 | flavor 分支接管(rich/tile/未校准 恒等) | ✅ | tui_cell_active 三重门;rich insta golden 逐字节 |
| 5 | 渲染侧 cell 对齐(布局宽≠渲染宽) | ✅* | box=n×cell + 校准 cell_w=mono advance;*非 mono 回退字亚 cell 居中记遗留 |
| 6 | 剩余间距规则 2 条 | ⬜* | *动态兄弟间距 + ⎿ 前缀 = 显示细节,记后续(承 plan45「后续」) |
| 7 | 门(五层 + rich 恒等 + tui cell golden + native≥5 + e2e≥2) | ✅ | native 11 / e2e 2 / tui cell golden / rich 恒等;五层门下 |
| 8 | progress 记账 + commit 不 push | ✅ | 本文件;L0-L2`b25d6f8` / L3-L4`8f4cfd0`,不 push |

**§3 客观判定**:逐字进 Rust ⬜ · rich 恒等 ⬜ · cell 网格 ⬜ · CJK 2-cell ⬜ · 校准一次 ⬜ · 布局宽≠渲染宽 ⬜ · 门 ⬜。
