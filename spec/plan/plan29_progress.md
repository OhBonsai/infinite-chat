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

## V2 · tier 回收器

(未开始)

## V3 · 重建路径

(未开始)

## V4 · 长会话回归 + after 对比

(未开始)
