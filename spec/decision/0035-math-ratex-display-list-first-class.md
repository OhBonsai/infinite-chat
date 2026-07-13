# 0035 · math = RaTeX display list 一等公民(supersedes 0013 的渲染路线)

- 日期:2026-07-13(Plan 31 R1;实现主体系 Plan 12 落地,本篇把路线定位正式化)
- 状态:**采纳**(append-only;0013 原文不改,其「渲染路线」节由本篇取代,动机/字体结论仍有效)
- 参考:[research/math-latex-sdf-rendering §6/§116](../research/math-latex-sdf-rendering.md) ·
  [0013](0013-math-latex-rendering.md) · [plan12](../plan/plan12-math-rendering.md) ·
  [0025 SDF 节点动画](0025-sdf-node-animation-system.md)

## 1. 决策

公式渲染 = **RaTeX(Rust)排版 → 扁平 display list → 每字/每横线 1:1 映射 FrameGlyph/FrameRect
→ KaTeX 字体经 MSDF atlas 成 SDF 实例**。不用 KaTeX/MathJax 取坐标(研究 §43),不做
「整张公式纹理」过渡形态(0→1 一次做对)。

- 依赖钉版:`ratex-{parser,layout,types}` @ git rev `1e5ae7d`(workspace Cargo.toml)。
- `DisplayItem` 映射(接入前读源核实,math.rs:150-196):`GlyphPath`→MathGlyph(ch/role/em
  尺寸/基线 dx,dy)、`Line`/`Rect`→MathRule;`Path`(根号勾/大定界符)v1 缓;15 个
  `StyleRole::Math*` 角色 ↔ KaTeX 字族双向映射。
- 字体链:预烘 `katex-msdf`(2048² 单页,role×0x110000+codepoint 键)+ 运行时 TinySDF 兜底
  (字体迟载后 `refresh_fonts` 一次重栅)。

## 2. 为什么(护城河)

每个公式字形都是与正文同管线的 SDF 实例 → **任意缩放锐利 + 可逐实例动画(0025)+ 参与
morph/magic-move(Plan 31 R2)**。opaque 纹理三者皆不可(0013 研究 §86)。

## 3. 与揭示的关系(Plan 30 衔接)

- 流式半截 `$$…` 由 raw 抑制 hold(plan30 M4,content.rs `is_pending_structure`);
- 闭合后走骨架门:**占位骨架(估宽高的 shimmer 框)先现 → RaTeX typeset 真身 swap**
  (plan30 M5 语义;Plan 31 R1 接真身,门逻辑零改动)。

## 4. 容差与判定(plan28 §5 同款)

与 KaTeX 参考渲染并排:布局盒 ≤4px 或 ≤3%(取大);native 定位快照(基线/上下标/分数盒)
锁定回归。失败者不入(invalid TeX → `ok=false` → 原文兜底,AR12)。
