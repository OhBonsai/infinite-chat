# Plan 18:规模 / 内存验证方案(10k 行 fps + 内存曲线)

- 日期:2026-06-23
- 状态:**度量段(§2/§6 步骤 1–2)已落地 + before 基线已采**(2026-06-24,见 [plan18_progress.md](./plan18_progress.md));0029 实现 + after 对比不在本期
- 前置:[0029](../decision/0029-session-virtualization-and-glyph-working-set.md)(本方案为其 before/after 度量 + 回归门)、[0012](../decision/0012-debugger-gui-html-vs-egui.md)(stats 通道 / 自绘几何复用)、[TODO §Plan 2 欠账](../../TODO.md)(10k 行 fps/内存 benchmark = "一直挂着的待验项")、[TODO §V](../../TODO.md)(黄金样张回归思路)
- 目标:**先量化"内存随历史线性增长"的现状(before),再为 0029 提供 after 对比 + 回归阈值**。一句话:把 README 那句"内存只与可见一屏成正比"从**口号变成可测的数字**。

---

## 1. 为什么先做这个(顺序铁律)

0029 改的是 `Store` / `PartView` / `BlockCache` 的核心形状。**没有 before 曲线就动核心结构 = 盲改**,违背"0→1 一次做对"。本 Plan 的"度量"段(§2)**不依赖 0029**、可独立落地,产出三条基线曲线;0029 实现后跑同样脚本得 after,对比即验收。

## 2. 度量项(需新增暴露的 + 已有的)

### 2.1 新增(core → wasm getter)

`FrameStats` / `stats()` 现有 fps/帧耗时/glyph·block visible·total/atlas·used·cap·evict/src 计数,但 **无内存与保留量**。补:

| 字段 | 来源 | 含义 |
|---|---|---|
| `store_chars` | `Σ Store.parts[*].text.len()` | 真相源文本总量(历史规模代理) |
| `retained_views` | `engine.views.len()` | 驻留 PartView 数 |
| `retained_glyphs` | `Σ view.cache.placed.len()`(有 cache 的) | **驻留逐字几何总量(0029 主攻对象)** |
| `retained_nodes` | `Σ view.cache.nodes.len()` | 驻留节点树规模 |
| (0029 后)`tier_counts[4]` | 各 Tier 的 view 数 | Hot/Warm/Cold/FrozenFar 分布 |
| (0029 后)`rebuilds_this_frame` | 回收器计数 | 重建/重新水化频次(thrash 监控) |

### 2.2 wasm 线性内存(主指标,JS 侧读)

wasm-bindgen 导出 `memory`。JS:

```js
import init, * as wasm from "../pkg/infinite_chat_wasm.js";
const bytes = wasm.memory.buffer.byteLength;   // wasm 堆总字节(单调增,反映分配峰值)
```

辅:Chrome `performance.memory.usedJSHeapSize`(JS 侧 overlay/纹理引用)。**主看 wasm 线性内存**——core 数据结构都在这。

> 注:wasm `memory.buffer` 只增不减(`memory.grow` 不回缩)。故 0029 的"内存回落"指**`store_chars`/`retained_glyphs` 计数回落 + wasm 内存增长曲线斜率趋平**,而非 `byteLength` 物理回缩。回缩需 Worker 重建实例(TODO2 §C),不在本期判据。

## 3. 测试驱动(复用现有重放 + 合成生成器)

### 3.1 长会话生成器(新增 `scripts/gen-longsession.mjs`)

产一个 N 行的合成重放 case(JSON,与现有 `web/src/replay.ts` 吃的格式一致):混合 60% 段落文本、15% 列表、15% 代码块、10% 表格(覆盖各 BlockCache 子结构)。参数 `--lines 10000 --turns 200`。落 `web/public/replays/longsession-10k.json`。

### 3.2 入口:`?bench=longsession&lines=N&speed=K`

`main.ts` 加 bench 分支(`?bench` 时):加载生成 case、按 `speed` 喂、每 1k 行采样一次 `{lines, wasmBytes, fps, frameMsAvg, retained_glyphs, retained_views, store_chars}` → console.table + 累积成 CSV(`copy()` 到剪贴板或下载)。**纯 dev 分支,prod 零成本**(同 `?debug`)。

## 4. 三个场景(各对应一个北极星断言)

| 场景 | 操作 | 当前预期(before) | 0029 目标(after) | 断言 |
|---|---|---|---|---|
| **A 增长曲线** | 持续 append 到 10k 行,每 1k 采样 | `retained_glyphs` / wasm 内存 ∝ 行数(线性斜率) | append 期斜率仍随**可见窗**有界;远块降级 → `retained_glyphs` 增长趋平 | "内存 ∝ 历史"被量化 |
| **B 滚动来回** | 滚到顶 → 回底 | `retained_glyphs` 不回落(屏外不释放) | 回底后 `retained_glyphs` 回落到 ≈ 可见基线(±1.5×);滚动**无跳变** | "可见一屏"成立 + 释放安全 |
| **C 静止结算** | 10k 行全 settled 后静止 | fps 应已满帧(验证 `settled` O(1) 跳过) | 不回退,fps ≥ 58 | 冻结路径有效 |

> 场景 B 的"无跳变"用 [0012](../decision/0012-debugger-gui-html-vs-egui.md) 的 `?verify` 自绘几何(块 AABB)+ 截图比对(接 [TODO §V] 黄金样张)——滚动来回前后,同一块的 y 不变(`height` 不释放的不变量,0029 §3)。

## 5. 判据 / 阈值(先测后定,本表占位)

- **before**:跑 §4 三场景,记录基线(尤其 A 的斜率 = bytes/1k行、B 回底后 `retained_glyphs`)→ 入库 `spec/diagnose/` 或本 plan 进度页。
- **after(0029)**:
  - A:`retained_glyphs` 增长斜率较 before 降 ≥ 80%(远块释放生效)。
  - B:回底后 `retained_glyphs` ≤ 可见基线 × 1.5;滚动来回**零跳变**(截图 diff)。
  - C:fps ≥ 58、`rebuilds_this_frame` 稳态 = 0(静止不 thrash)。
  - thrash 护栏:场景 B 来回滚动期 `rebuilds_this_frame` 峰值 < 可见块数 ×2(滞回带有效)。
- **回归门**:把 A 斜率 + B 回落值存基线快照,纳入 `?bench` 可复跑;后续改 `Store`/`PartView`/裁剪相关代码须重跑不回退(接 benchmark 常态化,TODO2 §C)。

## 6. 落地步骤(度量段可独立先发)

1. **(度量,不依赖 0029)** `FrameStats` + `stats()` 加 §2.1 字段;`main.ts` 加 `?bench` 采样 + CSV 导出;`gen-longsession.mjs` + 入库 10k case。
2. **(度量)** 跑 §4 三场景,记录 **before 基线**(本身就是有价值的交付:首次量化规模行为)。
3. **(0029 实现后)** 跑 after,填 §5 判据,设回归门。
4. **(常态化)** `?bench` 纳入 CI 冒烟或本地周期跑(TODO2 §C benchmark 体系)。

## 7. 非目标

- 不测 BiDi / 多字体兼容(与 [TODO §V] 同源:opinionated 单实现)。
- 不测真机多设备矩阵(先单机 Chrome WebGPU 基线;WebGL2 专测属 [TODO §R])。
- 不在本期追求 wasm 物理内存回缩(需 Worker 重建,TODO2 §C)——本期只验**计数回落 + 增长斜率趋平**。

---

参考:[0029](../decision/0029-session-virtualization-and-glyph-working-set.md)(被验证对象)· [0012](../decision/0012-debugger-gui-html-vs-egui.md)(stats 通道 / 自绘几何 / `?verify`)· [TODO §Plan2 欠账 + §V](../../TODO.md)· wasm-bindgen `memory` 导出 · 虚拟列表 benchmark 惯例(滚动来回测窗口外卸载)。
