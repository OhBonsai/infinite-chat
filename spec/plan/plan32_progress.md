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

(未开始)

## D4 · 上下文折叠 + scale 动画

(未开始)

## DoD 对账

(未开始)
