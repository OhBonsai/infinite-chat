# Plan 31 · 进度账本(富内容:math / magic-move / overlay / svg)

> 逐 R 记录:做了什么 / 参考依据(file:行号)/ 容差结果 / 遗留。
> GOAL:[plan31-rich-content-goal](./plan31-rich-content-goal.md)

## ADR · 新决策

- ✅ **0035**(math = RaTeX display list 一等公民,supersedes 0013 渲染路线)—— 2026-07-13。
- ⏳ 0036(SVG 矢量原生 vs 栅格)随 R4 落。

## R1 · math RaTeX 一等公民 —— ✅(2026-07-13;主体系 plan12 落地,本轮补验证/收口)

**审计结论(file:符号)**:管线全在 —— `math.rs:127` layout_math(DisplayItem→MathGlyph/MathRule,
`Path` 根号勾 v1 缓)、`app.rs:2512-2536/3135-3174` 帧集成(display ×1.3 居中、基线对齐、
MATH_IDX_BASE 防 morph 撞键)、katex-msdf 预烘 + TinySDF 兜底、8 个既有 native 测、
plan30 raw 抑制含 `$$` hold。本轮补三缺:

| 缺口 | 落地 |
|---|---|
| 定位快照 | `positioning_snapshot_baseline_sup_fraction`(insta:x^2 / e^{iπ}+1=0 / \frac{a}{b} 逐字 em 坐标锁定) |
| KaTeX 参考容差 | `web/scripts/check-math-katex.mjs` → `spec/reference/math-katex-CHECK.md`:**4/4 PASS,Δ=0.0000**(sup 抬升 0.3630 == σ_sup2;frac 杠上下 0.4070/0.9560 == num1/denom1±axis±rule/2;sub 0.1500 == σ_sub1)。教训:首版用 DOM span 盒量测全错(span 盒含行高/strut ≠ 基线)→ 改对照 KaTeX dist 度量常数(精确参考);DOM 渲染仅存并排参考图 `restore-diff/math-katex-ref.png` |
| M5 骨架门接真身 | **结论:天然满足** —— RaTeX 排版**同步**(闭合即 typeset,零等待期),半截 `$$…` 由 raw 抑制 hold(plan30 M4);0007 Placeholder→Ready swap 针对异步资源(图片),math 无异步段 → swap 点存在但恒零时长,不加占位框(加了也没帧可见) |

- 遗留:RaTeX `Path`(根号勾/大定界符)未映射(math.rs:196,上游 DisplayItem 已有;
  遇真实需求接 FrameRect 折线近似或等上游 SDF path)。

## R2 · magic-move

(未开始)

## R3 · DOM overlay

(未开始)

## R4 · streaming-svg

(未开始)
