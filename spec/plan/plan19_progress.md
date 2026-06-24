# Plan 19 进度(会话虚拟化:fps/内存 ∝ 可见一屏)

- 状态(2026-06-24):**P1 + P2 落地 + 数据验证达标**(fps ∝ 可见、retained ∝ 可见)。P3 不在本期。
- 验收复用 Plan 18 `?bench`(headless Chromium WebGPU,Playwright)+ 本 plan §2 per-phase 计时。

## ⚠️ 数据驱动修正:plan19 §0 的 fps 瓶颈假设是**错的**

§0 推断「每帧唯一 O(总驻留 glyph) 成本 = `build_frame::sizes` 的宽度 fold」。**per-phase 实测推翻**:

10k 行稳态(P1 前),分段中位(ms):

| 段 | ms | 说明 |
|---|---|---|
| `build_frame` 全部(layout+grid+emit) | **~2.3** | sizes fold 实测仅 ~1.6ms,**非瓶颈** |
| **`advance`** | **~67** | ← 真瓶颈 |
| └ `adv_ingest`(= ingest_events + **enqueue_new_text**) | **47.8** | ← 真凶 |
| └ roles / reveal / ensure / schedule | ~0 | settled 后 O(1) 跳过,符合 0025 §4 |

**真凶**:`enqueue_new_text` 每帧对**每个 part** `graphemes(text).map(to_owned).collect()` —— 即把**整段历史文本重切 grapheme + 堆分配整个 `Vec<String>`,每帧一次**。516k 字符 → ~47ms/帧。与 `sizes`/Taffy 无关。

> fold 466k 个 f32 取 max 实测 ~1.6ms(CPU 对连续内存极快);**堆分配 516k 个 `String`/帧**才是数量级杀手。教训:猜瓶颈不如插桩量。

## P1 — 真修:`enqueue_new_text` O(1) 短路(救 fps)

`PartView` 加 `pushed_bytes`;`enqueue_new_text` 先比文本**字节长度**(O(1)),未变即跳过重切。已 settled 的海量历史 part 由此恒 O(1)/帧。**行为等价**(177 测试不变)。

附带:`BlockCache.content_width` 缓存(`ensure_layouts` 算一次,`sizes` 读它免每帧 fold)—— 实测仅省 ~0.8ms(非瓶颈,但 P2 释放几何后 Warm 块靠它占位,**必留**,修 0029 §3/§7 只缓存 height 的遗漏)。`?sizefold` 为其 A/B 开关。

### P1 验收(headless Chromium,Playwright,10k 行 / 200 turn 稳态中位)

| 指标 | before | **after P1** | 门槛 |
|---|---|---|---|
| 核心帧时 frameMs | 73–98 ms | **6.3 ms** | — |
| `advance` | ~67 ms | **0.3 ms** | — |
| `adv_ingest` | 47.8 ms | **0.2 ms** | per-phase < 5% ✅ |
| fps(headless,vsync/compositor 限) | 10 | **51**(p95-low 41) | 真机 ≥58(headless 仅趋势门) |
| retained_glyphs | 466,650 | 466,650 | P1 不动内存 ✅ |

> headless fps 受 compositor 限(frameMs 6.3ms = 理论 ~158fps);真机 vsync 应达 ~60。CSV:`web/bench-results/plan19-p1on.csv`(after)/ `plan19-p1off.csv`(content_width A/B 对照)。
> 复跑:`cd web && BENCH_OUT=plan19-p1on.csv npm run bench`(P1-off 对照加 `BENCH_QS="sizefold&spread=60"`)。

### P1 度量补强(plan19 §2)

- **per-phase 计时**:`Engine::phase_ms()`(`web-time` Instant,native+wasm)→ advance 五子段 + build_frame 三段;`stats()` 暴露,`?debug` 面板 `phase ms`/`advance ms` 行,`?bench` CSV 逐列。**归因从断言变实测**(本页修正即其产物)。
- **中位 + P95**:`?bench` 稳态尾 9 拍取 median/P95(headless fps 抖,不取单次)。
- 真机复测 / 场景 D(史+慢流尾):待 P2 后一并补。

## P2 — Hot/Warm 工作集(救内存)

`Tier{Hot,Warm}` + `PartView.agg{content_width,height}`(释放后仍留 → 占位稳定)。`reclaim`(每帧 O(views),
本帧 `drawable` 位置 + 视口,滞回带 promote 1.5 屏 / release 3.0 屏):屏外 `settled` 块 → Warm(`cache=None`
丢 placed/clusters/roles/strike/nodes/math/embeds);进 promote 带 → Hot(下帧 `ensure_layouts` 重建,源
`revealed` → R8 逐字节等价)。`sizes`/`drawable` 用 `agg` 让 Warm 块照常参与 boxlayout(上方块不塌,0029 §3)。
`?novirt` 全 Hot 对照。`FrameStats.tier_counts[2]`/`rebuilds_this_frame` 度量。

### P2 验收(headless Chromium,10k 行 / 200 turn 稳态,锚底)

| 指标 | P2-off(`?novirt`) | **P2-on** | 门槛 |
|---|---|---|---|
| **retained_glyphs** | 466,650 | **9,432** | ≤ 可见基线 ×1.5 ✅(= 可见窗) |
| tier(hot/warm) | 200 / 0 | **4 / 196** | 屏外释放 ✅ |
| **rebuilds/帧**(稳态) | 0 | **0** | =0(scenario C 无 thrash)✅ |
| wasm 线性内存 | 60.1 MiB | **30.8 MiB** | 计数回落 + 斜率趋平 ✅ |
| fps / frameMs | 47 / 6.2ms | 51 / 5.7ms | 不回退 ✅ |

> **retained_glyphs 466k → 9.4k(49×)**:驻留几何从「∝ 历史」变「∝ 可见窗」(4 个 Hot 块)。这就是
> README「内存只与可见一屏成正比」的实测兑现。CSV:`plan19-p2on.csv` / `plan19-p2off.csv`。
> 复跑:`cd web && BENCH_OUT=plan19-p2on.csv npm run bench`(P2-off 加 `BENCH_QS="novirt&spread=60"`)。

### P2 不变量(native 测,CR1/R8)

- `p2_release_then_rebuild_is_byte_identical`:释放 → `ensure_layouts` 重建 `placed` **逐字节等价**(R8;源 = revealed,确定性)。
- `p2_release_keeps_layout_stable`:释放屏外块 → 可见块世界 y 与全 Hot 一致(**零跳变**根,0029 §3 靠 `agg` 占位)。
- `p2_retained_falls_back_to_visible_window`:多 turn → retained 远小于满载 + 出现 Warm + rebuilds=0。

### 仍属范围 / 薄后续

- **跳滚(teleport)1–2 帧重建延迟**:`scroll_by(-1000)` 跳到已释放块 → promote 后下帧重建(虚拟列表通例)。
  渐进滚动(≤1.5 屏/帧)由 promote 带覆盖,无空白。`?verify` 截图 scenario B 来回 + scenario D(史+慢流尾)
  浏览器自动化为薄后续(逻辑已由上述 native 测覆盖)。
- **真机绝对 fps**:headless 受 compositor 限(median 51);真机 vsync 复测留记。
- P3(Cold/FrozenFar:丢 revealed/Store,靠 0003 快照重取)**不在本期**(10k 不需要)。
