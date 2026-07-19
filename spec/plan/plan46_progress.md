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

✅ **重开后收(2026-07-19,L5 纠偏完成)**。原判定 `cell_w = measureText('M')` **确不成立**(实证:系统等宽栈非双宽,CJK≈1.66×拉丁 → CJK 侧漏格、字间距过宽)。**根因**:cell 网格要求「双宽字体」(拉丁半格、CJK 恰 2 格),系统等宽栈不满足,且 chat 页**从未加载** LXGW MSDF(boot.ts 只挂数学 MSDF)→ 退系统字体。**修**:tui 切 LXGW WenKai Mono MSDF(实测拉丁 M=a=i=14、CJK=28,比值**恰 2.0**)+ `cell_w = msdfAdvancePx`(渲染同源)→ box=advance,glyph 填满格,无过宽(截图实证 rich/tui 并存正确)。见 L5。
- [x] 布局盒宽 = `n×cell_w`(layout_cells 写死;CJK n=2)—— 网格几何本身正确,保留。
- [x] native:`cjk_two_cells_and_mixed_advance` 锁盒宽 = n×cell;`replay_tui_cell_frame_snapshot` golden 锁逐字 col。
- ❌ **误判**:「校准 cell_w=measureText('M') → 布局宽=渲染宽」——measureText 量 CSS MONO 字体(且校准可能早于 webfont 加载 → 比例回退字体,'M' 超宽),≠ SDF atlas 画字的 advance。
- ❌ **误删遗留**:「亚 cell 居中 mono 路无需」——该假设仅当 cell_w 精确 == 每字渲染 advance 才成立,现不成立 → 居中/裁切是**安全网非可选**,移交 L5c。

## L4 · 收口(校准面 / 剩余间距 / golden / 门)

✅ (2026-07-19)
- [x] 校准面:`layout-bridge.calibrateCellMetrics()`(cell_w=mono 'M' advance / cell_h=LINE_HEIGHT[tui 1.25×])→ chat applyFlavor(tui) 注入 `set_cell_metrics`(开机一次,非每帧);rich 清 `set_cell_metrics(0,0)` 退原路。
- [x] golden 家族:rich `replay_settled_frame_snapshot`(恒等)+ tui `replay_tui_cell_frame_snapshot`(逐字 col 整数,CJK 世@col3 size18)。
- [x] native ≥5(实际 11:cellwidth 5 + celllayout 5 + metrics AR12 1)。
- [x] e2e:`cell:tui 逐字进 Rust`(强制重排 JS layout 回调 delta==0)+ `cell:rich 对照`(delta>0);measureText 仅开机校准。
- 遗留:**剩余间距 2 条**(动态兄弟间距 + tool 结果 `⎿` 前缀)—— 显示细节,非布局关键,承 plan45「后续」记后续;真终端截屏。

## L5 · 对拍纠偏(根因修复)

✅ **完成(2026-07-19)**。根因 = cell 网格要求双宽字体,系统等宽栈非双宽(CJK 1.66×拉丁)且 chat 页未加载 LXGW MSDF。照参考终端(等宽由**字体**保证)修:

- [x] **L5a · 校准换源**:`calibrateCellMetrics` 的 `cell_w` 改取 `msdfAdvancePx('M', FONT_SIZE)`(LXGW baked advance = 渲染同源);measureText 仅 MSDF 未就位时兜底。`applyFlavor(tui)` 先 `await loadMsdf` 再校准 → cell_w 直接 = 14(不经错值)。
- [x] **L5b · 全 mono(双宽)atlas**:tui `applyFlavor` 切 `setLayoutGlyphMode("msdf")` + `set_glyph_mode(3=ForceMsdf)` → 量宽源与渲染源都 = LXGW(双宽,拉丁 14 / CJK 28)。比「强制 CSS mono raster」更对(CSS 等宽栈本就非双宽,是病根);rich 切回 `bitmap`(系统比例),两侧 glyph 源同步,rich 观感不变(截图实证)。
- [x] **L5c · 字形填格**:cell_w == 每字渲染 advance(L5a 同源)→ box = advance,glyph 天然填满格,**无需**亚 cell 居中(L3「误删」在同源前提下确实无需;缺字 TinySDF 兜底罕见)。
- [x] **切 flavor 竞态守卫(await LXGW 副作用)**:`applyFlavor(tui)` await MSDF 加载期间用户切走 → 旧 tui 收尾回灌覆盖 rich(内容清空)。修:代际计数 `flavorGen`,await 后代号变则停手;switch e2e 由「单读 visible_turns」改「轮询」(高负载帧循环慢,内容本在 store,防假阴)。
- [x] **额外根因(实机新发现,非原 L5 列):** ① **表格塌成纯文本**——cell bypass 丢 `table_panels`;修:`tui_cell_active && tables.is_empty()` 才走 cell,表格块留 JS 表引擎(placeTable,tui mono 下列自然对齐)。② **diff/代码整行折断**——`layout_cells` 未守 no_wrap;修:code/CodeLineNum/Diff(Added/Removed/Ctx)角色整行不折(viewport 横裁),补 celllayout no_wrap。
- [x] **验收**:实机 rich‖tui 并排(showcase-full,同内容)——tui 拉丁/代码/CJK 无过宽字间距、表格列对齐、diff 单行裁切;rich 比例字体不受影响。native `code_text_does_not_wrap`(含 diff 角色);e2e `cell:tui 校准渲染同源 + 双宽`(cell_w==msdf 拉丁、CJK==2×cell_w 硬断言)+ `文本逐字进 Rust`(重排 JS layout ≪ rich)。**未做**:与 opencode 官方截图逐像素对拍(无官方图源;以「双宽不变量 + 目视终端观感」替代)。

## DoD 对账

| # | DoD 要求 | 状态 | 证据 |
|---|---|---|---|
| 1 | EAW 宽度表(core 纯函数) | ✅ | cellwidth::char_cells + 5 native |
| 2 | 校准通道(注入,非每帧) | ✅ | set_cell_metrics(AR12)+ calibrateCellMetrics;e2e measure 稳态 0 |
| 3 | core cell 布局器 | ✅ | celllayout::layout_cells + 5 native + golden |
| 4 | flavor 分支接管(rich/tile/未校准 恒等) | ✅ | tui_cell_active 三重门;rich insta golden 逐字节 |
| 5 | 渲染侧 cell 对齐(布局宽≠渲染宽) | ✅ | L5a 校准换源:cell_w = LXGW msdf advance = 渲染 advance → box=advance,glyph 填满格;e2e 双宽硬断言 |
| 6 | 剩余间距规则 2 条 | ✅ | ✅ tool 结果 `⎿` 前缀(tui_tool_render_full;native + 浏览器 `→ Explored / ⎿ 1 read`);✅ 动态兄弟间距(layout_chat `tui`+`inline_tool` 参数:连续 inline tool 贴合 0、跨类型留 space_part;**rich 恒等**由 `tui=false` 原路 + 空 slice 保证,`replay_settled_frame_snapshot` 逐字节不变;native `tui_consecutive_inline_tools_stack_tight` + `partspecific::tui_tool_is_inline` 分类。间距值=复用 plan45 对拍的 space_part,非盲值) |
| 7 | 门(五层 + rich 恒等 + tui cell golden + native≥5 + e2e≥2) | ✅ | native 12(+code no_wrap)/ flavor e2e 4 / tui cell golden / rich 恒等;目视 rich‖tui 并排过 |
| 8 | progress 记账 + commit 不 push | ✅ | 本文件;L0-L2`b25d6f8` / L3-L4`8f4cfd0`,不 push |
| 9 | 目视对拍收敛(L5,无过宽字间距) | ✅ | LXGW 双宽(拉丁 14/CJK 28=2×)→ 字形填满格,实机无过宽;e2e 双宽硬断言锁死。表格塌 + diff 折 两额外根因一并修 |

**DoD-6 收口(2026-07-19)**:两条间距规则均落地。① `⎿` 前缀(`8e5b1a3`);② 动态兄弟间距(boxlayout 加 `tui`/`inline_tool` 门控,rich 走原均匀 gap 路径逐字节恒等,tui 连续 inline tool 贴合)。rich 恒等铁律:`tui=false`+空 slice → 与改动前同一代码路径 → `replay_settled_frame_snapshot` 不变(已验)。

**§3 客观判定**(证据在括号):逐字进 Rust ✅(e2e 重排 JS layout ≪ rich)· rich 恒等 ✅(insta 逐字节 + 实机比例字体不变)· cell 网格 ✅(golden col 整数 + 实机表格列齐)· CJK 2-cell ✅(native + LXGW CJK=2×cell_w e2e)· 校准一次 ✅(applyFlavor await loadMsdf 后注入)· 布局宽=渲染宽 ✅(L5a 同源 cell_w=msdf advance)· **校准同源(L5a)✅ · 全双宽 atlas(L5b,LXGW)✅ · 无过宽字间距(L5)✅**(实机 + e2e 双宽硬断言)· 门 ✅(五层,rich 恒等 + 双宽 e2e)。

## 门 · 干净全绿(2026-07-19,gate11)

`node test/run.mjs` **513 passed / 0 failed**(gates 4 / native 382 / unit 54 / e2e 72 / perf 1)—— 含 L5 全量 + DoD-6 两规则 + todowrite 清单。空载即过(此前多轮 perf 假红见下,均属机器负载)。

## 门 · 环境说明(2026-07-19)

五层门 **gates/native/unit 恒绿**(native 381/381:cellwidth 5 + celllayout 6 + AR12 1 + boxlayout 7[含 tui 贴合]+ components tui ⎿ + 全 rich 恒等 golden 逐字节)。**perf + 部分 timing e2e 在长跑轮偶发红** —— 根因 = 本机 Claude.app 桌面进程(实测 ~27% CPU)在 perf 测量窗抢占,frameMsAvg 被拖到 15–46ms(fps 17–47),`/dev.html?bench`(perf 靶,本 plan 代码零触碰)饿帧超阈;同轮 timing e2e(copy/cell/formation/resilience/visual golden)因帧不稳 5s 超时。**每一个偶发红均已单跑隔离验证通过**(空载 fps 60);且 `gate7` 得过**一次干净 510/0**(L5 全量)。**代码正确性已穷尽验证**(native + 隔离 e2e + rich golden 逐字节 + V7 布局 golden 过);perf 红属机器负载,非回归。

**收敛账**(≤5 轮):L0–L4 结构 → L5 目视纠偏(cell_w 换 msdf 同源 + LXGW 双宽 + 表格留 JS + diff no_wrap + 切 flavor 竞态守卫)。**五层门 510/0 全绿**(gates4/native379/unit54/e2e72/perf1;perf 需机器空载,曾因 Claude.app CPU 抢占多次假红,空载即过)。commit:结构 `b25d6f8`/`8f4cfd0`/`2990d97`,L5 纠偏 `6b617c0`(不 push)。遗留:opencode 官方逐像素对拍(无图源,以双宽不变量替代);**动态兄弟间距**(DoD-6 半:threading 进 shared boxlayout 危及 rich 恒等 + 精确间距值需 opencode 对拍源,无源不盲改 → 记残留)。`⎿` 前缀本轮已补(`6b617c0` 之后)。
