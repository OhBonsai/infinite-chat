# Plan 29 · 进度账本(会话虚拟化 + SumTree)

> 逐 milestone 记录:做了什么 / 参考依据(file:行号)/ before-after 数值 / 遗留。
> GOAL:[plan29-session-virtualization-goal](./plan29-session-virtualization-goal.md)

## V0 · 度量对齐 / before 基线 —— ✅(2026-07-13)

- 度量字段核对:`FrameStats.{store_chars,retained_views,retained_glyphs,retained_nodes}` 全在
  (app.rs:1001-1007,plan18 落的);`?debug` 面板 / wasm `stats()` 暴露在。**零补字段。**
- Tier 现状:`enum Tier { Hot, Warm }`(app.rs:719)+ 滞回升降(app.rs:3359-3361,plan19 P2)——
  plan29 = 扩 Cold/FrozenFar + SumTree。
- **native before 复采**(`bench_scale_before --ignored`,当前 master = plan28 全量成果):
  - A 斜率 = **46.48 glyphs/行**(plan18 时 46.76,漂移 -0.6% = plan28 渲染改动,同量级线性);
  - B 回底 retained_glyphs = **463850 == 满载**(屏外不释放——native 单帧场景 Warm 未触发);
  - C settled 恒定 ✓。
- **浏览器 before 复采**(bench.spec,`web/bench-results/plan29-before-browser.csv` 入库;
  plan18 历史基线文件不动):200 turn 满载后 —— retainedGlyphs **9376**(Hot 4 / Warm 196:
  P2 Warm 几何释放在浏览器滚动路径已生效)、fps_median **50**、frameMs_median **11.87**、
  **ph_grid_median 1.8ms**(SpatialGrid 每帧重建 = plan19 真凶,V1 靶子)、wasmMiB **30**。
- 结论:尺子全在、before 双侧复现 → 可动结构(0029 §82 铁律满足)。

## V1 · 高度区间索引(前缀和路线)—— ✅(2026-07-13)

- 设计:聊天列**单列纵排**(boxlayout 文档序,y 单调)→ 可见性 = 一维区间查询。
  `SpatialGrid`(HashMap 均匀格,每帧 clear+insert O(N))**整个删除**,换
  `HeightIndex`(spatial.rs 重写):按 y0 有序区间数组 + **max-y1 前缀**(变高块防漏)
  + 二分 → O(log N + K) 查询;常序输入零排序、容量帧间复用。GOAL 写 SumTree/前缀和二选一,
  单列场景前缀区间数组即最简正确形(增量树留给未来多列需求)。
- 不变式校验器(GOAL §6):`assert_matches_linear`(二分 == 线性扫)进测试;6 个单测 +
  变高防漏例(整屏代码块横跨多视口)。
- `height` view 级(0029 §84):Warm 已由 `view.agg.height` 承担(agg 挂 PartView 即 view 级);
  Cold/FrozenFar 沿用同载体 → V2 落地,本步不加冗余字段。
- **基准(GOAL §3 判据)**:`bench_height_index_query_sublinear`(release)——
  10k 块 0.0028s / **100k 块 0.0025s**(每万次查询)→ 块 ×10 耗时**恒定**,远优于线性。
- **浏览器 after-V1**(`plan29-after-v1-browser.csv`,同场景 vs V0 before):

| 指标 | before | after-V1 |
|---|---|---|
| ph_grid_median | 1.8ms | **0ms** |
| frameMs_median | 11.87 | **6.0** |
| fps_median / p95_low | 50 / 2 | **60 / 60** |
| wasmMiB | 30 | 29.7(V2 才动内存) |

## V2 · tier 回收器(四级)—— ✅(2026-07-13)

- `Tier` 扩全四级(0029 §2 驻留矩阵;带宽实测定:promote 1.5 / Warm 3 / Cold 8 / Frozen 24 屏,逐级滞回):
  - **Warm**(现状):释 `cache`(placed+nodes;与 0029 矩阵差异 —— 矩阵 Warm 留 nodes,
    plan19 实现即已随 cache 一起释,沿用并记录);
  - **Cold**:再丢 `revealed`+`spawn`(grapheme 级 String 驻留 = 长会话内存大头)+ 随块清
    `code_scroll`/`image_registry` 条目(0029 §88;presentation 态,重建同键重登);
  - **FrozenFar**:再释 Store text(`release_text`/`text_released`;Store 语义放宽为
    「可由 server 快照重取的缓存」0029 §5)——**仅 `resync_available`**(真 server)时启用;
    重放/bench 无源自动封顶 Cold → 重放确定性不破(R8)。
- 修复途中踩坑:Cold 块曾被 `ensure_layouts`(空 revealed 排空 cache)与 drawable
  (`(None,Warm)` 匹配漏)双路塌高 → 两处扩匹配 + 非 Hot 一律不重排;`p2_release_keeps_layout_stable`
  与 `scroll_to_brings_block_into_view` 回归即刻抓住(T9 价值)。
- `FrameStats.tier_counts` 扩 [4];debug 面板显示 V4 接。

## V3 · 重建路径(与 V2 同 commit 落地)

- **Cold→Hot**:`rehydrate(i)` = display_source 重切 grapheme + `spawn` 全 `Some(-1e9)`
  (catch-up 恒等收敛 → 瞬显零动画,AR6)+ cache 交下帧 `ensure_layouts`(纯函数)→
  可见字形与释放前**逐字节一致**(native `tier_cold_releases_then_rehydrates_byte_identical`)。
  不走 smoother(零到达节流参与,直接终态)。
- **FrozenFar→Hot**:display_source 空 + `text_released` → `request_resync()` 边沿标志;
  wasm rAF 轮询 `wants_resync()` → 强制 Phase J 周期对账立即拉快照 → `resync_from_snapshot`
  全量覆盖自愈 `text_released`(store apply 写入清标志);等待期凭 `agg` 占位(高度不塌)。
- 守卫:`frozen_far_only_with_resync_source`(无源封顶 Cold;有源释文 + 滚回请求重取 + 边沿读清)。
- **浏览器 after-V2**(`plan29-after-v2-browser.csv`,200 turn 满载停在底部):

| 指标 | before | after-V1 | after-V2 |
|---|---|---|---|
| wasmMiB | 30 | 29.7 | **21.3**(-29%) |
| tier(Hot/Warm/Cold+) | 4/196/- | 4/196/- | **4/3/193** |
| frameMs_median | 11.87 | 6.0 | **4.36** |
| fps_median / p95_low | 50/2 | 60/60 | 60/60 |

## V3 · 重建路径

(未开始)

## V4 · 长会话回归 + after 对比 —— ✅(2026-07-13)

- **滚动来回内存回落(DoD-1)**:bench.spec 增 roundtrip 段(底→顶→底,采 wasm 线性内存 + 四档):
  `bottom=21.3 → top=21.3 → back=21.3 MiB`(**零攀升**,断言 ≤ 稳态×1.10 编码在测里);
  回程 tier = 3h/0w/197c/0f。before(plan18)同场景只涨不降(30MiB 满载 + 屏外零释放)。
- **native after 采集器** `bench_scale_after`(release 手跑,断言编码):
  满载驻留 463850(before/novirt)→ after 来回后 **7032**(-98.5%,Hot 窗口量级);
  `full 9376 / top 9106 / back 7032`,tiers 收敛 [3,0,197,0]。
- `?novirt` 对照在(main.ts:156,plan19 就位);debug 面板 tier 行扩四档
  (`Nh/Nw/Nc/Nf · rebuild N`),wasm `stats()` 增 tierCold/tierFrozen。
- 验收帧:`test/results/restore-diff/plan29-bench-dpr{1,2}.png`(长会话页顶部,虚拟化开)。
- fps(DoD-2):after 稳态 60/60(vs plan19 达标 50);frameMs 4.36(before 11.87)。

## DoD 对账(1–8)

| DoD | 状态 |
|---|---|
| 1 内存回落 | ✅ 浏览器 roundtrip 21.3 MiB 零攀升(断言 ≤×1.10);native 驻留 -98.5%;before/after 数值上表 |
| 2 fps 不回退 | ✅ 60/60(before 50/2);TESTREPORT 全绿 |
| 3 无跳变 | ✅ `tier_cold_releases_then_rehydrates_byte_identical`(可见字形逐字节一致) |
| 4 确定性 | ✅ 同上(重建确定)+ replay oracle 全绿 + FrozenFar 无源封顶(重放不受回收影响) |
| 5 SumTree 替换 | ✅ SpatialGrid 删净 → HeightIndex(前缀区间+二分);基准 10k/100k 查询耗时恒定 |
| 6 四层门 + ≥2 native | ✅ 381/381;新增 native:spatial×6 + tier_cold + frozen_far + bench_after 断言 |
| 7 progress 逐 milestone | ✅ 本文件 |
| 8 按 milestone commit 不 push | ✅ V0/V1/V2+V3/V4 四个 commit |

> 收口:「无限会话」初心两条 —— fps 稳(plan18/19)+ **内存跟可见屏走(本 GOAL)**,均已数值兑现。
