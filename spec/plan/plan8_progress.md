# Plan 8 进度(reveal 节奏自主:0019 落地)

- 状态(2026-06-17):**已落地(v1 简约子集)**。五相位 8A–8E 全过卡口(native+wasm clippy 0、102 core 测试、wasm-pack、tsc 0)。
- 一句话:把"何时够格揭示"(gate)与"按何序/节奏揭示"(style/stage)从渲染机制剥成数据驱动 policy,**揭示调度器成为唯一 spawn_time 来源**,节奏与 token 解耦、结构块骨架先行、不闪 raw。

## 各相位落地

| 相位 | 落地 | 关键文件 |
|---|---|---|
| **8A** 就绪门 | `RevealUnit{Glyph,Line,Row,Block}`(单调)+ `content_gate(src)`(泛化 `is_pending_table` 为"成形到第几级")+ `layout_gate`(列宽定=`Block`)+ `is_pending_structure`(表格 + 半截 `$$` 公式 raw 抑制) | `core/src/reveal.rs`、`core/src/content.rs` |
| **8B** 揭示风格 | `RevealStyle{gate, stages}` 纯数据;`Selector`(Frame/Grid/Header/Cell/RowGlyphs/Glyphs/ByKind)、`Dep`(Content/Layout 双门 + Stage 边 + Now);3 表格风格(Raw/RowFrame/Full)+ text/skeleton 默认;`resolve()` 在 0020 节点树上落地为逐 glyph `GlyphPlan{tier, delay_ms}` | `core/src/reveal.rs` |
| **8C** 调度器 | `RevealScheduler`:解耦揭示时钟(`reveal_cps` 限速 + `slow` 放慢,注入 `dt_ms` 可重放);**收编即时揭示**——删 `spawn=revealed[j].1/now`,改由 `Engine::schedule()` 按 (tier,序) 释放 display 字形、定 `spawn_time = 释放时刻 + delay`;smoother 退为"到达"、调度器管"呈现" | `core/src/reveal.rs`、`core/src/app.rs` |
| **8D** 骨架先行 | 结构块字带骨架/表头 delay → 晚于即时入场的容器底/框/网格(表头先于 body、底先于码);全结构块 raw 抑制(8A);native CollectSink 回归锁定 | `core/src/app.rs`(测试) |
| **8E** 调试 UI | wasm 绑定 `set_reveal_cps`/`set_reveal_slow`/`set_table_reveal_style`;调试面板"表风格"+"揭示速度"下拉,**实时**生效(无 reload),配置持久化 | `crates/wasm/src/lib.rs`、`web/src/debug-panel.ts` |

## 验收(DoD)对照

1. **结构块绝不闪 raw**:✓ `is_pending_structure`(表格 + 半截 `$$`);列表/围栏由 marker/`remend` 闭合本就不闪(保守不误伤)。
2. **骨架先行**:✓ 容器(底/框/网格)即时入场,cell/码字带 `delay_ms`(tier 累加)→ spawn 更晚;表头早于 body。
3. **节奏与 token 解耦**:✓ `reveal_cps` 限速 + `slow` 放慢(全局揭示时钟);默认 `INFINITY`=跟内容到达(不回归)。
4. **无跳变 + 可重放**:✓ 几何仍走 0016/`PanelScene`;调度器时间走注入 `dt_ms`,同 dt 序列 → 同 spawn(确定性测试)。
5. **纯文本逐字不回归**:✓ 默认风格 = 逐字、`reveal_cps=INFINITY` → 等价现状,速度可调。

## v1 简约取舍(留后续,符合 plan8 "终态结构完整、实现取简约子集")

- **面板入场 alpha 淡入**(`FramePanel.spawn_time` → panel.wgsl / `PanelScene`):容器当前**实心即时**入场(骨架先行的时序已满足),其自身的淡入是视觉打磨,留后续(同 0019 "0016 exit 淡出留尾")。
- **完整每类块 style 数据库 / 可配**:v1 内置 text/skeleton + 表格 3 风格;后续抽数据表。
- **`RevealUnit::Subtree`(嵌套根整体)/ 乐观预测 / 每类块独立配速 / 用户偏好持久化**:未做(0019 §8 留 policy)。
- **`?verify` 黄金样张对拍**(同 case × 速度/风格):未做,留 [TODO V];当前靠 `?replay` + native 渲染数据回归。

## 卡口命令

```
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo clippy --workspace --target wasm32-unknown-unknown -- -D warnings
cargo test --workspace
(cd crates/wasm && wasm-pack build --target web --out-dir ../../web/pkg)
(cd web && npx tsc)
```

## 浏览器人工验(8E)

`?debug` 面板 → "表风格" 选 整表/行框/原始、"揭示速度" 选 慢/极慢,跑 `?replay=n-all` / `c06-*` 表格 case:慢放下肉眼验**框先于字、表头先于 body、无 raw 闪**;`?debug` 节点框看 Selector 选中。
