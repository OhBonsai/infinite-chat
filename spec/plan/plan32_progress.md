# Plan 32 · 进度账本(diff 体验 + 装饰契约)

> 逐 milestone 记录:做了什么 / 参考依据(file:行号)/ 数值来源(opencode 或 Zed)/ 遗留。
> GOAL:[plan32-diff-experience-goal](./plan32-diff-experience-goal.md)

## D0 · ADR 0037 装饰声明式契约(GOAL 原文 0035,该号已被 plan31 占用 → 顺延)

- **做了什么**:新增 [0037](../decision/0037-declarative-part-decoration-contract.md);
  `partrender.rs` 落契约:`DecorationOp{kind,span,slot,group}` + `RenderOutput{spans,decorations}`,
  `RenderRegistry::register_full`(旧 RenderFn 自动包装,既有渲染器零改动)。
- **锚定**:display glyph 区间 `[start,end)`,不是几何 —— renderer 纯函数(CR1/R8),
  app `place_decorations` 经 `cache.placed` 解析成世界矩形。
- **颜色**:`DecorSlot` 语义槽,emit 端经 theme 解析(0021,装饰随块缓存不失效)。
- **删旧**:`app.rs flush_diff_band` + diff_rows 逐行反推路径删净(0→1 不留双轨);
  file-card bbox 的角色扫描保留(那是 plan28 容器,不是 diff 装饰)。

## D1 · hunk 化数据 + 词级区间

- **做了什么**:`partspecific.rs` 新增 `DiffHunk{old_start,new_start,lines,word_ranges}` +
  `diff_parse_hunks`(解析 `@@ -a,b +c,d @@`,-/+ 成对分组)+ `word_change_range`
  (空白切词公共前后缀,>512B 行跳过)。
- **数值来源(Zed 语义值,非源码)**:`MAX_WORD_DIFF_LINE_COUNT = 5`(等行数 Modified hunk
  ≤5 行才做词级 diff);`FOLD_CONTEXT_OVER = 3`(连续 Context >3 折叠)。
- **native 测试**:`hunk_parse_snapshot_mixed`(insta)、`word_ranges_only_for_equal_small_hunks_and_in_line`、
  proptest `hunk_parse_never_panics`(AR12)。

## D2 · 三层视觉(行底 / gutter / 词级 / 行号)

- **做了什么**:`render_diff_full` 一次产 spans + 三层装饰;`tool_render_full` 注册为 Tool
  的 full 渲染器(trigger 行后偏移合并);`app.rs place_decorations` 按 kind 摆放。
- **三层**:Band = 整宽行底(逐行分段);Gutter = 左缘条,宽 `floor(0.275×行高)`(Zed 语义值,
  下限 2px);CharBg = 词级字符底(圆角 3 / pad 1)。
- **行号**:双列 `{old:>4} {new:>4}`(context)/ 单列(±),折叠行推进行号。
- **颜色来源(opencode live-dark tokens)**:gutter add/del/hidden = `#c4ffc0`/`#fec3b8`/`#23345a`
  (--surface-diff-*-stronger);词级 add/del = `#012902`/`#430501`(--surface-diff-*-strong);
  入 `theme.rs` 5 个新字段。
- **折叠(数据层,交互在 D4)**:连续 Context >3 → `⋯ n unchanged lines` 汇总行,默认折叠。
- **native 测试**:`render_diff_full_emits_three_layer_decorations`(2 band/1 gutter/2 word +
  确定性 R8)、`long_context_folds_into_summary_row`(行号跨折叠推进)。
- **验收帧**:`test/results/restore-diff/plan32-d2-dpr{1,2}.png`(showcase-full Edit 卡)。

## D3 · gloop 连续圆角

- **做了什么**:`FrameRect`/`RectInstance` 增 `gloop: [f32;4]`(`[prev_dx, prev_w, next_dx,
  next_w]`,`w<=0` 无邻接);`rect.wgsl` fragment 对上/下邻行盒(同高、y 平移 ± 一行)与本盒
  `op_smin`(sdf.wgsl 既有多项式 smin,0025)平滑并 → 相邻同色行底融成无接缝连续圆角块。
  全零 = 独立矩形,非 diff 像素不变。
- **手法来源**:makepad `draw_selection.rs` 邻行参数手法(MIT,**只借手法,WGSL 自写**);
  smin 为 IQ 多项式(sdf.wgsl 已有 `op_smin`,复用不新写,0026 共识①)。
- **落点偏差(记账)**:GOAL 原文写「panel shader 扩参」,实际 band 走 **rect 管线**
  (FrameRect→RectInstance→rect.wgsl),故邻接参数加在 rect 图元;panel 是容器图元,不动。
  ADR 0037 §3 已同步改写。
- **融合核**:`k = max(2×radius, 1)` —— k=r 时接缝角点(场值 ≈0.414r,smin 仅压 k/4)留缺口,
  2r 才闭合;`DIFF_BAND_RADIUS = 4px`(app.rs 常量,含推导注释)。
- **满行盒**:glyph 行盒间有 leading → 相邻带(缝 < 半行高)y1 拉到下条 y0 闭合
  (跨色也闭合,同 opencode 满行连续;gloop 仍只融同色);绘制序 = 带垫底,gutter/词底压上。
- **native 测试**:`band_gloop_adjacency_chains_same_color_only`(同色贴邻才融/换色断链/
  有缝断链/单行退化,纯函数 R8);render 侧 naga 校验拼接后 rect.wgsl 过。
- **WebGL2 兜底**:同一 WGSL 经 naga 转 GLSL(wgpu GLES 后端),无特性依赖,不需双路。
- **验收帧**:`test/results/restore-diff/plan32-d3-gloop-dpr{1,2}.png` + 4× 放大人检
  (绿块行缝无接缝、红绿交界硬边、单行独立圆角)。

## D4 · 上下文折叠 + scale 动画

- **契约扩展**:`DecorKind` += `FoldSummary`(汇总行:Zed 式强圆角 gutter marker,宽
  `floor(0.35×行高)`、圆角 = 行高 → shader 半尺寸夹紧成胶囊;research §2.2 deleted-fold
  marker 语义值)+ `FoldRegion`(展开区:不画,glyph span 供 scale 包络);
  `RenderCtx` += `unfolded: u64` 位图(bit i = 折叠区 i 展开;≥64 恒折叠,保 Copy)。
- **交互(0032 ②)**:`place_decorations` 对 FoldSummary 登记世界命中盒 →
  `Engine::tap` ask 未中时查 `fold_targets` → 命中即 `diff_unfold.insert(key)` +
  `fold_anim.insert(key, 0.9)` + 失效该块 cache(重排版,unfolded 进 RenderCtx)。
  wasm `fold_hit_targets()`(屏幕 px JSON,同 ask_hit_targets)供 e2e/宿主。
- **持久态**:`diff_unfold: HashSet<u64>`(key = view<<32|区序号)—— **Cold 回收不清**
  (GOAL §和 plan29 衔接:重建后保持;不同于 code_scroll 滚动位置随块回收)。
- **scale 动画(makepad session.rs **范式**,实现自写)**:`fold_scale_step` 纯函数 ——
  `factor = 0.9^(dt/16.667)`(60fps 一帧恰 ×0.9,帧率无关 R8),残差 <0.01 snap 到
  **恰好 1.0**(收敛恒等 AR3);advance() 注入 dt 推进,收敛即从 map 移除(空 map 零成本)。
  应用:build_frame 发射处,FoldRegion span 内 glyph 绕区左上锚等比缩放(SDF 字任意
  scale 锐利);包络只动 presentation,不写 model(AR2/RD11)。
- **native 测试(3 个)**:`fold_scale_deterministic_and_converges_to_identity`(双跑一致 +
  ×0.9 语义 + snap 恒等)、`unfolded_region_emits_rows_and_fold_region_op`(展开行 + 指令 +
  确定性)、`tap_on_fold_summary_expands_context`(engine 级:tap 真命中 → 展开 → 收敛)。
- **golden(双态)**:`web/tests/visual.spec.ts` V8 —— `diff-fold-folded.png` +
  `diff-fold-expanded.png`(tap 真路径展开,放行时钟收敛后冻结 → 帧确定)。
- **验收帧**:`test/results/restore-diff/plan32-d4-{folded,expanded}-dpr{1,2}.png`。
- **遗留**:①收起交互未做(展开单向;Zed 是 gutter 控件收起,0032 命中盒可加)——
  scale 通道方向无关,补交互即可;②展开动画期间 Band/Gutter 装饰静置(只 glyph 缩放),
  收敛后对齐;③词级 diff 分词=空白(语言感知记 D1 遗留)。

## DoD 对账

1. **hunk 化** ✅ `diff_parse_hunks` + 快照/词级/proptest(D1)。
2. **三层视觉** ✅ 行底/gutter(0.275×lh)/词级/双列行号;数值来源逐项在 D2 节。
3. **gloop** ✅ rect 图元邻接参数 + `op_smin`(k=2r);单行退化;WebGL2 同 WGSL 经 naga。
4. **上下文折叠** ✅ >3 折叠、tap 展开(0032)、scale ×0.9 指数逼近(注入 dt,snap 恒等)、
   强圆角 marker(0.35×lh / 圆角=行高)。收起交互遗留(见 D4 节)。
5. **ADR + 首租** ✅ 0037(号顺延自 0035);`flush_diff_band` 删净
   (grep 仅存注释里的「已除役」记载);装饰全部经 DecorationOp。
6. **测试** ✅ native 新增 7 个(D1 3 + D2 2 + D3 1 + D4 3,超 ≥3);golden V8 双态;
   全门 `node test/run.mjs` 绿(每 milestone commit 前跑)。
7. **progress 记账** ✅ 本文件逐 milestone(做了什么/依据/数值来源/遗留)。
8. **commit 政策** ✅ 按 milestone commit(D0-D2 `a86f327`、D3 `a779635`、D4 本次),不 push。
