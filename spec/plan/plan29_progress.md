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

## V1 · SumTree 高度索引

(未开始)

## V2 · tier 回收器

(未开始)

## V3 · 重建路径

(未开始)

## V4 · 长会话回归 + after 对比

(未开始)
