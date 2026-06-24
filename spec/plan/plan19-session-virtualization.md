# Plan 19:会话虚拟化落地 —— 打到「fps/内存只与可见一屏成正比」

- 日期:2026-06-24
- 状态:**P1 + P2 落地 + 数据达标**(2026-06-24,见 [plan19_progress](./plan19_progress.md))。⚠️ §0 的 fps 瓶颈假设被 per-phase 实测推翻——真凶是 `enqueue_new_text` 每帧重切 grapheme(非 `sizes` fold),详见进度页。P3 不在本期。
- 前置:[0029](../decision/0029-session-virtualization-and-glyph-working-set.md)(工作集模型)、[Plan 18](./plan18-scale-memory-verification.md) / [plan18_progress](./plan18_progress.md)(before 基线 + O(历史) 实测证据)、[0025 §4](../decision/0025-sdf-node-animation-system.md)(settled 冻结=降级前置)、[0016](../decision/0016-streaming-morph-render-model.md)(释放块不在 Scene)、[0020](../decision/0020-content-node-identity-model.md)(重建后身份不变)、[0003](../decision/0003-fault-tolerance.md)(快照=重新水化,仅 Phase 3)
- 目标:把 README「fps/内存只与可见的一屏成正比,与历史总量无关」**从口号做成实测达标**:10k 行 / 200 turn 下 **fps 回 ~60、`retained_glyphs` 跟可见窗(滚动来回回落)**,且**滚动零跳变、确定性可重建**。

---

## 0. 关键发现(决定本 plan 的分相):fps 与内存可分离

Plan 18 实测 + 代码核实(`app.rs::build_frame`)定位:**每帧唯一的 O(总驻留 glyph) 成本是 `sizes` 的宽度 fold**——

```rust
// build_frame 第 1 步:为 boxlayout 算每个 view 的内容宽。每帧、对每个 view 的每个 placed 求 max。
let sizes = self.views.iter().map(|v| match &v.cache {
    Some(c) if !c.placed.is_empty() => {
        let w = c.placed.iter().filter(|p| p.size[0] > 0.0)
                 .map(|p| p.pos[0] + p.size[0]).fold(0.0, f32::max);  // ← O(总 glyph)/帧
        (w, c.height)
    } _ => (0.0, 0.0),
}).collect();
```

其余每帧 pass 都**不是** O(总 glyph):`revealed_height` 循环找到首个有已揭字的块即 `break`(O(1 块));`grid` 重建 / `drawable` / `refresh_roles` / `ensure_layouts` dirty-check / `schedule` 都是 **O(views)** 且每 view O(1)(settled 跳过,0025 §4)。

> **推论(本 plan 的杠杆)**:10k 行 fps 60→10 的主因是 `sizes` 每帧 fold 466k 个 placed。**把块内容宽像 `height` 一样缓存一次**,`sizes` 即降为 O(views)——**fps 几乎可独立于内存释放、零数据模型改动地救回来**。内存(retained 466k)是另一回事,靠真正的几何释放解决。**故分三相:先廉价救 fps,再结构性救内存。**

这同时**修正 0029 草案 §3/§7**:释放几何要保「布局稳定」,不能只缓存 `height`,**必须连块内容宽 `content_width` 一起缓存**(否则 `sizes` 无源)。

---

## 1. 三相策略(渐进可验,每相端到端跑 `?bench`)

| 相 | 做什么 | 攻克 | 数据模型改动 | 风险 |
|---|---|---|---|---|
| **P1 聚合缓存** | `BlockCache` 加 `content_width`(随 `height` 在 ensure_layouts 算一次);`sizes`/`drawable` 读缓存不再 fold `placed` | **fps**(O(总glyph)→O(views)) | 加 1 字段,**不释放、不改形状** | **极低**(纯缓存,等价改写) |
| **P2 Hot/Warm 工作集** | 屏外 settled view **释放重几何**(`placed`/`clusters`/`roles`/`strike`/`nodes`/`math`/`embeds`),留 `(content_width,height,revealed_bottom)`+`revealed`;重入走 `ensure_layouts` 重建;滞回 | **内存**(retained∝可见) | `PartView` 加 `tier`+释放/重建;`BlockCache` 拆「聚合(留)/几何(可弃)」 | 中(释放/重建往返等价是核心不变量) |
| **P3 Cold/FrozenFar(后置,非本期目标)** | 再丢 `revealed`/`Store text`,重入靠 0003 快照重取 | 极致规模(>内存预算) | `Store` 冷段可丢 | 高,**10k 行不需要,defer** |

**本期目标 = P1 + P2**。P1 单独就让 fps 达标(可先发、先验);P2 让内存达标。P3 仅当 P2 后内存仍超预算才启,留 [TODO2 §C](../../TODO2.md)。

### 1.1 P1 — 聚合缓存(救 fps)

- `BlockCache` 加 `content_width: f32`:在 `ensure_layouts` 算 `placed` 后**一次性** fold 出(与 `height` 同处),随块冻结缓存。
- `build_frame::sizes` 改读 `(c.content_width, c.height)`,删掉每帧 fold。
- (可选)`revealed_height`:为每 view 缓存「已揭字最低底」增量维护,或维持现状(已 O(1块),非瓶颈,可不动)。
- **验收(P1)**:`?bench` 跑到 10k → **fps ≥ 58**;per-phase 计时(见 §2)显示 `build_frame` 的 sizes 段占比从主导降到可忽略。**内存此相不变**(retained 仍 466k,符合预期——P1 只救 fps)。

### 1.2 P2 — Hot/Warm 两档工作集(救内存)

- `BlockCache` 显式拆两组:**聚合(释放后仍留)** = `content_width`/`height`/`revealed_bottom`/`revealed_len`;**重几何(可弃)** = `placed`/`clusters`/`roles`/`strike`/`nodes`/`math`/`embeds`。
- `PartView` 加 `tier: Tier { Hot, Warm }`(本期两档);回收器(节流,~200ms / 滚动停顿)按 `grid` 距离 + `settled` 调档:**屏外 + settled + 超滞回带 → Warm(释放重几何)**;**进可见滞回带 → Hot(`ensure_layouts` 重建)**。
- `build_frame::sizes`/`drawable` 用**聚合**参与 boxlayout:Warm 块**照常占位**(布局稳定不跳变,0029 §3 不变量),但 `drawable` 标记其无几何 → 不进 emit。**关键:释放 placed 的 view 仍要进 `sizes`/`boxlayout`,否则上方块塌缩**(修 0029 草案漏的 drawable `continue` 行为)。
- 重入重建走现成的 `ensure_layouts`(`cache.placed` 本就是 `None→重排`,R8 确定性);重建源 = `revealed` + `max_width`。
- **验收(P2)**:`?bench` 场景 B(滚到顶→回底)`retained_glyphs` **回落到 ≈ 可见基线(≤ ×1.5)**;`?verify` 截图滚动来回**同块 y 不变(零跳变)**;场景 C 静止 `rebuilds_this_frame` 稳态 = 0;来回滚动 thrash 峰值 < 可见块 ×2(滞回有效)。

---

## 2. 度量与归因补强(扣 Plan 18 的数据可信度短板)

承「数据狗吗」复盘的三条短板,本 plan 一并补,作为 P1/P2 的验收前提:

- **per-phase 计时**:`?bench` 加 `performance.measure`(`advance` vs `build_frame`;`build_frame` 内 `sizes`/`grid`/emit;`advance` 内 `refresh_roles`/`ensure_layouts`/`schedule`)→ 证明 P1 前 sizes 主导、P1 后归零。**归因从断言变实测**。
- **多次取中位 + P95 帧时**:fps/帧时 N≥5 次跑,报 median + P95,不取单次(headless fps 抖)。
- **真机复测**:10k 关键点(60→? )在非 headless 真机各跑一次,确认绝对值(headless 仅作 CI 趋势门)。
- **新增场景 D 流式尾块压历史**:预载 N 历史(瞬显 settled)+ 尾部**慢速真流式**(`cps` 正常),测**尾帧 fps**——这才是北极星真实场景(活动尾 + 海量静止史)。P1/P2 后尾帧应 ≥ 58。

---

## 3. 改动清单(精确到 file:符号;CR1/R8 守则见 §5)

**core(`crates/core`)**

- `app.rs::BlockCache`:加 `content_width: f32`;(P2)按「聚合/几何」分组并允许几何为空。
- `app.rs::ensure_layouts`:算 `placed` 后求 `content_width`(fold 一次)存缓存。
- `app.rs::build_frame`:`sizes` 读缓存;(P2)`drawable` 让 Warm 块带聚合占位、不 emit。
- `app.rs::PartView`:(P2)加 `tier`、`release_geometry()`、`is_warm()`;`FrameStats` 加 `tier_counts[2]`/`rebuilds_this_frame`。
- `app.rs::Engine`:(P2)加 `reclaim()`(节流回收器,扫活跃 view 按 grid 距离+settled 调 tier);`frame`/`advance` 末尾调。
- `nodes.rs`/`reveal.rs`:确认 Warm 释放 `nodes` 后,reveal/morph 对该(已 settled)块零引用(已 settled → gate 全开,重建即 instant)。

**wasm(`crates/wasm`)**

- `lib.rs::stats()`:暴露 `tierCounts`/`rebuildsThisFrame`(并入 `StatsSnapshot`)。

**web(`web/src`)**

- `main.ts` `?bench`:加 per-phase `performance.measure` 采样 + 中位/P95 聚合 + 场景 D;`debug-panel.ts` 显示 tier 分布 + 本帧重建数。

**测试(native 优先,T 铁律)**

- `release→ensure_layouts 重建` 往返**逐字节等价**(proptest:任意 revealed 序列,Hot vs Warm-then-rebuild 的 `placed` 相等)→ R8。
- `Warm 块仍参与 boxlayout`:释放几何后上方块 y 不变(回归 0029 §3 不变量)。
- `scale_stats_grow_with_history`(已存):P1 不变;P2 加「滚动来回 retained 回落」断言。

---

## 4. 验收(扣 Plan 18 §5 after + 本 plan 新增)

| 项 | 门槛 | 相 |
|---|---|---|
| fps @10k(中位,真机) | ≥ 58 | P1 |
| per-phase:sizes 段占比 | P1 后 < 5% | P1 |
| 尾帧 fps(场景 D:10k 史 + 慢流尾) | ≥ 58 | P1/P2 |
| `retained_glyphs` 滚动回底(场景 B) | ≤ 可见基线 ×1.5 | P2 |
| 滚动来回跳变(`?verify` 截图) | 同块 y 不变(零跳变) | P2 |
| 静止 `rebuilds_this_frame`(场景 C) | 稳态 = 0 | P2 |
| 来回滚动 thrash 峰值 | < 可见块数 ×2 | P2 |
| 卡口 | clippy 0 / native test / wasm32 build / tsc | 全相 |

before 基线(plan18_progress):A 斜率 46.76 glyph/行、fps 60→10、retained 满载 466k。after 对比即填 0029 §5。

---

## 5. 铁律与风险

- **CR1**:`tier`/回收/释放逻辑全在 `core`,无平台依赖 → native 可测往返等价。
- **R8 确定性**:释放→重建复用 `ensure_layouts`(`cache: None→重排`),源 = `revealed`+`max_width`,纯函数 → 逐字节等价(proptest 守)。
- **布局稳定(0029 §3)**:`content_width`+`height` 永不释放;Warm 块照常占位 → 释放/重建不动任何其它块 y。**这是「零跳变」的根**,也是 0029 草案需补的修正(原只提 height)。
- **只降级 settled**(0025 §4):尾块在长 → 恒 Hot;锚底跟随时稳定后再回收。
- **0016 morph 正交**:释放块本不在活动 `Scene`(只装活动尾)→ 释放/重建与补间无关,无回滚动画;重入按 instant 瞬显。
- **滞回防 thrash**:进 Hot 阈值 < 退 Warm 阈值;回收节流(非每帧)。具体距离/带宽 P2 实测调(`?bench` 场景 B 的 thrash 峰值为准)。
- **可关**:`?novirt` 全程 Hot = P1 前行为,作对照/兜底。

---

## 6. 落地顺序(PR 切分,每个端到端可跑 + 过卡口)

1. **PR-A(P1)**:`content_width` 缓存 + `sizes` 读缓存 + per-phase 计时。跑 `?bench` → fps 达标即交付(内存不变,符合预期)。**最高性价比,先发。**
2. **PR-B(P2-机制)**:`BlockCache` 聚合/几何拆分 + `PartView.tier` + `release_geometry` + 重建路径 + native 往返等价测试。`?novirt` 默认开(不改行为)。
3. **PR-C(P2-策略)**:回收器(grid 距离+settled+滞回)+ `tier_counts`/`rebuilds` 度量 + debug 面板;开虚拟化。跑 `?bench` B/C/D → 内存/跳变/thrash 达标。
4. **PR-D(回归门)**:after 基线入库 + `scale` 断言升级 + 真机/多次中位记录;填 0029 §5、翻 0029 状态为「已采纳」。
5. P3(Cold/FrozenFar)**不在本期**,留 0029 §6「不决定的」+ TODO2 §C。

---

参考:[0029](../decision/0029-session-virtualization-and-glyph-working-set.md)(模型,本 plan 修正其 §3/§7 的 width 缓存遗漏)· [Plan 18](./plan18-scale-memory-verification.md)/[plan18_progress](./plan18_progress.md)(证据+harness)· [0025 §4](../decision/0025-sdf-node-animation-system.md)(settled)· [0016](../decision/0016-streaming-morph-render-model.md)· [0020](../decision/0020-content-node-identity-model.md)· [0003](../decision/0003-fault-tolerance.md)(P3 重新水化)· 虚拟列表窗口外卸载/回挂惯例。
