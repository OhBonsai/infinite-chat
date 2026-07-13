# Plan 30 · 进度账本(揭示节奏美学)

> 逐 milestone 记录:做了什么 / 参考依据(file:行号)/ 数值断言结果 / 遗留。
> GOAL:[plan30-reveal-cadence-aesthetics-goal](./plan30-reveal-cadence-aesthetics-goal.md)

## M1–M3 · 审计结论:已由 plan25 patch 落地(2026-07-13 核对)

plan30 GOAL 起草于 2026-07-08;2026-07-07 合入的 plan25 patch(`882dfc6`)已实现 M1–M3 主体。
逐项核对(file:符号):

| Milestone | 落地(plan25) | 测试 |
|---|---|---|
| M1 motion token | `motion.rs`(dur 分档/stagger/space + CURVE_ENTER/EXIT/EXPRESSIVE 拟合式,glyph.wgsl apply_curve 同值)+ style-config MOTION_TOKENS + 面板 "Motion" | `motion::token_table_snapshot`(数值锁) |
| M2 节奏五层 | `rhythm.rs` 548 行:tempo(词/CJK 拍)→ 边界停顿(0020 层级+标点)→ 末延长 → 重音 → seeded 1/f 微定时(splitmix64,seed=块 key)+ accelerando;调度器 ms 亏欠预算;额外项 ∝ tempo(→0 即时,回归零) | `deterministic_same_seed_same_costs`(R8)/`boundary_rest_monotone_with_level`(单调)/`latin_word_is_one_beat` 等 7 测 |
| M3 三预设+API | `RhythmPreset{Typewriter,Reader,Flow}`;wasm `set_reveal_preset`/`set_rhythm`(六旋钮:tempo/rest/final_lengthening/accent/microtiming/accelerando;globalCps= 既有 `set_reveal_cps`)+ 面板 "Rhythm" 下拉 | `typewriter_zero_layers_uniform`(旧匀速等价)/`rest_scales_with_tempo_instant_at_zero` |

- DoD-1/2/3/5 由上表覆盖;golden 帧已于 plan25/28 按 reader 预设重录并全门复验多轮。
- 差异记录:GOAL 旋钮名 `globalCps` 实现为独立 `set_reveal_cps`(语义等同,plan18 起已有)。

## 边界③守卫 · rehydrate 零调度器介入 —— ✅(2026-07-13)

- native `rehydrate_bypasses_reveal_scheduler_instant_full`:Cold 块 promote 后 3 帧内全量可见
  (reader 慢节奏下调度器路径一帧 ≈0 字 → 若介入必挂);plan29 的 byte-identical 测同守。

## M4 · 全类型 raw 抑制 —— ✅(2026-07-13,plan8/22 基础上补缺)

- 已有(核对):表格(`is_pending_table`→Row 门)、围栏(remend 补全+Line/Block 门)、列表(Line 门)、
  显示公式(奇数 `$$` hold)、链接/图片 `](url` 半截(`strip_forming_link` 裁尾)。
- **本轮补**:成形中图片 `![alt…`/`![alt]`(`](` 未到)裁尾 —— 规则:`![` 后无 `)` 且未跨行
  (跨行=非图片语法,放行字面;0017 §10 保守原则,裸 `[` 不裁)。
- **DoD-4 判据编码**:`streaming_prefix_sweep_never_leaks_raw_structure` —— 混合结构文档
  **每个 char 前缀**渲染均不含 `](`/`|--`/`$$`/`![`/围栏原文(全类型一次扫)。

## M5 · 全类型骨架先行 —— ✅(2026-07-13)

- 表格:plan25 已有(网格先行,panel reveal 纵向揭示,V5 golden)。
- **本轮**:code 面板与引用/Alert 左条的装饰门从「已揭字累积」改为「**结构确认即整框/整条**」
  (块入 cache = 围栏闭合/remend 补全)→ 容器先现、字按节奏后到(0019 §162;
  旧「避免空框先现」顾虑被骨架先行设计取代,表格同款在先)。
- 时序断言:`skeleton_panel_enters_before_first_code_glyph`(慢揭示逐帧:code_bg 首现帧 <
  首个代码字帧)。
- 公式/图片真身(typeset/纹理)= plan31 范畴:图片已有 alt 占位→纹理链(plan14),公式 pending
  hold 已有;骨架接口就绪,不实现本体(GOAL §5 边界)。
- 遗留:字 stage 的显式 `offset_ms` 延迟通道(现骨架先行由「容器整框 + 字逐拍」自然形成;
  若要更强的骨架停留感,调度器加 stage 延迟——记待需求)。
- 验收帧:`restore-diff/plan30-skeleton-dpr{1,2}.png`。
