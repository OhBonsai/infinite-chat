# Plan 18 进度 / before 基线(规模 · 内存度量)

- 状态(2026-06-24):**度量段(§2 / §6 步骤 1–2)落地 + before 基线已采**。0029 实现与 after 对比**不在本期**(goal:不做 0029)。
- 沙箱约束:native(`cargo test`)可测 `retained_*` / `store_chars`(纯 CPU 数据结构,无 GPU);
  **fps / wasm 线性内存须浏览器 `?bench`**(沙箱无 WebGPU)。

## 已落地

| 项 | 落地(file:符号) |
|---|---|
| **度量字段(§2.1)** | `FrameStats.{store_chars, retained_views, retained_glyphs, retained_nodes}`(app.rs);build_frame 末尾 O(views) 累加(`Vec::len()` O(1),不扫内容);`Store::char_count()`(store.rs,Σ `text.len()` 字节) |
| **wasm 暴露** | `StatsSnapshot` + `stats()` 加 `storeChars/retainedViews/retainedGlyphs/retainedNodes`;`?debug` 面板 `retained`(g/v/n)+ `store` 行(debug-panel.ts) |
| **wasm 线性内存(§2.2)** | JS 侧 `wasmModule.memory.buffer.byteLength`(main.ts `?bench` 采样器读) |
| **长会话生成器(§3.1)** | `scripts/gen-longsession.mjs`(60% 段落 / 15% 列表 / 15% 代码 / 10% 表格;`--lines/--turns`)→ `web/public/replays/longsession-10k.json`(records 格式,200 turn / 独立 part) |
| **浏览器入口(§3.2)** | main.ts `?bench=longsession&lines=<tag>&spread=<ms>`:载长会话 → 每 1s 采 `{turns, storeChars, retainedGlyphs, fps, frameMsAvg, wasmMiB}` → console.table + CSV(`window.__benchCSV` + 复制剪贴板) |
| **native 基线采集器** | app.rs `bench_scale_before`(`#[ignore]`):200×50=10k 行,跑 §4 三场景,打印 CSV + before 断言;`scale_stats_grow_with_history`(常驻回归:retained_* 随历史增长) |

## before 基线实验数据(native,10k 行 / 200 turn / viewport 800px)

复跑:`cargo test -p infinite-chat-core --release bench_scale_before -- --ignored --nocapture`

```
scenario,      lines, turns, store_chars, retained_views, retained_glyphs, retained_nodes, frame_glyphs
A_growth,       1000,    20,       50790,             20,           45810,           6540,         2261
A_growth,       2000,    40,      102030,             40,           92070,          13080,         2261
A_growth,       3000,    60,      153270,             60,          138330,          19620,         2261
A_growth,       4000,    80,      204510,             80,          184590,          26160,         2261
A_growth,       5000,   100,      255750,            100,          230850,          32700,         2261
A_growth,       6000,   120,      307890,            120,          278010,          39240,         2304
A_growth,       7000,   140,      360030,            140,          325170,          45780,         2304
A_growth,       8000,   160,      412170,            160,          372330,          52320,         2304
A_growth,       9000,   180,      464310,            180,          419490,          58860,         2304
A_growth,      10000,   200,      516450,            200,          466650,          65400,         2304
B_top,         10000,   200,      516450,            200,          466650,          65400,         2298
B_back_bottom, 10000,   200,      516450,            200,          466650,          65400,         2312
C_settled,     10000,   200,      516450,            200,          466650,          65400,         2312
```

### 解读(三北极星,§4)

- **A 增长曲线** — `retained_glyphs` 与行数**严格线性**:斜率 = **46.76 glyph/行**(`store_chars` ≈ 51.6 字节/行,`retained_nodes` ≈ 6.54/行,`retained_views` = turn 数)。✅ 量化了 README 那句"内存 ∝ 历史"。
- **关键反差** — `frame_glyphs`(本帧实发/可见)全程**恒定 ≈ 2.3k**,而 `retained_glyphs` 涨到 **466,650**:10k 行时**驻留 / 可见 ≈ 200×**。这正是 0029 要攻的缺口(驻留应跟**可见窗**,而非历史)。
- **B 滚动来回** — 滚到顶、回底,`retained_glyphs` **恒 = 466,650**(== 满载)。✅ 证实 before:屏外**不释放**。
- **C 静止结算** — 30 帧静止,`retained_glyphs` **恒定**,无 thrash。✅ 冻结路径不回退。

> **回归门(before 快照)**:A 斜率 ≈ 46.76/行;B 回底 == 满载(467k);C 恒定。`bench_scale_before` 已把这三条编码为断言 → 改 `Store`/`PartView`/裁剪相关代码须复跑不回退。

### 浏览器侧基线(fps + wasm 线性内存)— Playwright 无人值守采集

native 测不到 fps / wasm 物理内存。用 Playwright(headless 完整 Chromium / Metal·Dawn WebGPU)驱动
`?bench` 自动采集。复跑:

```
npm --prefix web run gen:longsession        # 生成 case(已入库 web/public/replays/longsession-10k.json)
npm --prefix web run bench                   # Playwright 跑;CSV 落 web/bench-results/plan18-before-browser.csv
# 或手动:cd web && npm run dev,open 'http://localhost:5173/?bench&debug'(控制台导出 window.__benchCSV)
```

`?bench` 下 `set_stream_rate(1e9)` + `set_reveal_cps(∞)` → 内容随 Player(`spread` 控)释放即时载满,
采到「随历史增长」的真实曲线。**采得(2026-06-24,headless Chromium 149 / WebGPU,viewport ≈ 默认窗口)**:

| turns | lines | retained_glyphs | frame_glyphs(画) | **fps** | frame_ms | **wasm MiB** |
|---:|---:|---:|---:|---:|---:|---:|
| 21 | ~1k | 48,123 | 2,182 | 61 | 10.3 | 22.3 |
| 50 | ~2.5k | 115,200 | 2,182 | 50 | 19.3 | 22.3 |
| 100 | ~5k | 230,850 | 2,182 | 24 | 40.2 | 32.9 |
| 150 | ~7.5k | 348,750 | 2,224 | 15 | 67.9 | 47.2 |
| **200** | **~10k** | **466,650** | **2,224** | **10** | **99.4** | **60.1** |

(完整逐拍曲线见 `web/bench-results/plan18-before-browser.csv`;retained_glyphs / store_chars 与 native 完全一致 → 两路互校。)

### 解读 — 这就是 0029 要攻的证据

- **fps ∝ 1/历史**:fps 从 **60 → 10**(6×)随 retained 涨,而 `frame_glyphs`(实际画)**全程 ≈ 2.2k 恒定**(视口 cull 生效)。→ 每帧成本是 **O(历史) 而非 O(可见)**:`build_frame` 的 sizes/ensure_layouts 等 pass 逐帧遍历**全部** view(宽度 fold over `placed` = O(总驻留几何))→ 10k 行时单帧 ~99ms。**这正是 0029(虚拟化 / 工作集)要消除的。**
- **wasm 线性内存单调增** 22.3 → 60.1 MiB(`memory.grow` 不回缩):~38 MiB 装 466k 驻留 glyph + 516k 源字符 + 65k 节点。
- before 北极星(§4)三场景:A 线性增长(已量化)、B 屏外不释放、C 静止恒定 —— native 已断言;浏览器 fps/内存曲线补齐**性能维度**。

## after(0029)— 不在本期

§5 after 判据(A 斜率降 ≥80%、B 回底 ≤ 可见基线 ×1.5、C fps≥58 且 thrash=0)与 `tier_counts`/`rebuilds_this_frame` 度量待 0029 实现后填。本期只交付 **before 量化 + 回归门 + 可复跑工具**。
