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

## R2 · magic-move —— ✅(2026-07-13,v1 按 GOAL §6 预案收窄)

**审计结论**:0016 保留态 Scene(morph.rs)已有 from→to 补间(CPU-mix 路 B)、reflow 平滑、
append-only 前缀稳定、表格列宽 PanelScene 补间 —— DoD 的行为面大半在。本轮补:

| 项 | 落地 |
|---|---|
| **内容稳定身份** | `assign_stable_ids`(纯函数,前后缀 diff):正文 `glyph_ids`(view 级,cache 重建时按 cluster 序列 diff;append-only 时 == 旧位置键,零回归)+ **数学区 `math_ids`**(region 按序配对,ch 序列 diff;`$x$→$x^2$` 既有字保身份)。发射处 `glyph_idx` 语义升级为稳定身份(唯一消费者 = morph NodeId,选区/查找不经此) |
| **峰值速度上限** | morph.rs `MAX_MOVE_SPEED_PX_PER_MS=1.2`(perception §5):update 时按位移拉长 `move_dur`,cubic-out 峰速 ≈3×均速 ≤ 上限;测试数值采样验证 |
| **测试** | `stable_ids_prefix_and_suffix_survive_midblock_insert`(增/插/删/还原)/`stable_ids_deterministic_under_seeded_insert_storm`(seeded LCG 200 次乱插,双跑逐字节一致,R8)/`tool_rewrite_midinsert_keeps_tail_identity`(集成:tool 全量重写中段插入,尾部 morph 键稳定)/`math_glyph_sequence_keeps_identity_on_superscript_growth`(机制级)/`move_velocity_capped_by_stretching_duration` |
| **路 B 保留(差异记账)** | DoD 原文要 shader `target_pos+move_start` lerp;现状 0016 已采 CPU-mix 路 B,行为等效(双位置插值、可打断不回跳),GPU 化属性能重构非行为缺口 —— 判据验证的是行为(双跑一致/身份复用/速度),不为形态重写引擎(0→1 不留双轨的反面:不为形态废弃已达标机制)。记为可选性能优化 |

- **v1 收窄(预案授权)**:text part **中段重写**的完整 magic-move 记遗留 —— 其对账路径现为
  append-only 语义(gs[pushed..] 尾推),中段改写会推错内容(**既有缺口**,先于本轮;修复涉及
  重写检测 + spawn 保持免闪,另立任务);公式真身变形依赖该路径,机制层已备(math_ids),
  接通即得。表格列宽/正文 reflow/tool 重写三场景已达标。

## R3 · DOM overlay —— ✅(2026-07-13,0022 分阶段 v1)

**审计结论**:动图 billboard 链(camera.world_to_screen → frame_embeds → <img>)、canvas-rect
对齐、Embed FSM(Placeholder→Ready)、key 复用回收均在(plan14/0022 最小切片)。本轮升级:

| 项 | 落地 |
|---|---|
| **transform 定位** | left/top(触发重排)→ `translate3d + scale`(GPU 合成,0022 §47);基准尺寸取 mount 帧,后续尺寸变化走 scale |
| **世界/相机分离** | `frame_embeds` 输出改 `{cam:{px,py,zoom}, embeds:[{…,wx,wy,ww,wh}]}` —— **世界矩形**(布局产物)变化走 0016 同款补间(120ms cubic-out,可打断不回跳);**相机**每帧直乘零过渡(否则快滚迟滞 = 脱节)。差异记账:FrameEmbed 未经 core Scene,web 侧同式 lerp,视觉等效 |
| **EmbedKind 钩子注册表** | `registerEmbedRenderer(kind, {mount,onTransform?,onVisibilityChange?,unmount?})`(0022 §89);内置 "image";未知 kind 兜底 image(AR12) |
| **像素断言 e2e** | `embed-overlay.spec.ts`:动画 SVG 数据 URI(SMIL 嗅探→动图)→ pan 137px / zoom ×1.35 后 overlay 盒与引擎屏幕矩形差 **≤1px**(GOAL §3 判据) |

- 遗留(0022 后续阶段,等首个非 image 消费者):`NodeKind::Embed` 产出(html 围栏 →
  EmbedKind::Html)、通用 `reportSize(key,w,h)`(image natural_size / ask 高度两先例已证路;
  通用化随 html kind 落,避免无消费者死 API)、主题联动(image 透明无样式,html 才有意义)。

## R4 · streaming-svg

(未开始)
