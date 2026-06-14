# Plan 4(排版收口 + 观感追平 + 基础调试器)

- **状态(2026-06-14)**:4A / 4C **已落地**;4B **核心落地**(矩形 quad 图元 + 代码块底/行内码 chip/引用左条/H1·H2 细线 + 调试几何),表格两趟对齐 / GitHub Alert 左条 / hr / 设计令牌固化**留尾**(见 [plan4_progress](./plan4_progress.md) §尾账)。
- 日期:2026-06-14
- 范围:从原 [TODO](../../TODO.md) 拆出的**近期可做项**(非愿景层);愿景层(效果上限/画布产品化/极致规模/交互深度)见 [TODO2](../../TODO2.md)。
- 前置:Plan 3 K/L 已落地([plan3_progress](./plan3_progress.md));决策 [0001 §2.2 修订](../decision/0001-canvas-architecture.md)(不引 pretext)、[0011](../decision/0011-gpu-text-as-sdf-primitive.md)、[0012](../decision/0012-debugger-gui-html-vs-egui.md)。
- 相位:**4A 排版收口 → 4B markdown 观感 → 4C 基础调试器**;一相位 ≈ 一/数 PR,末尾过卡口。

## 0. 定位

Plan 3(K/L)把底座搭好(SDF/位图图元 + 相机 + 空间索引)。**Plan 4 = 把"能渲染"收口成"观感对、可调试"**:修折行/层级/装饰、追平 GitHub markdown 结构观感、打通调试器看得见性能。**不碰**愿景层(效果系统/画布产品化/极致规模 → TODO2)。

**In**:词折行 + CJK 禁则、H1–H6 分级、块装饰 quad + 表格对齐、github 令牌、pretext 清理改名、per-role 度量、调试器 P1 + 数据通道。
**Out(→ TODO2)**:富特效系统/动效框架、画布产品化、极致规模、交互深度;**Out(→ 留 TODO.md)**:K′ 双模三源、O 嵌入、P 标签层、Q input/选区、R a11y/降级、S 打包。

**铁律**:content→layout→render 契约不动(0001 §2.2)/ AR10 每帧一次跨界 / CR1 core 零平台依赖 / 小包体。

---

## Phase 4A — 排版收口(JS 为主 + 少量 Rust)

> 域:`layout`(JS bridge)· `content`(Heading 分级)。修可见的折行/层级问题 + 清理 pretext 遗留。

**任务**
- 4A1 **词边界折行(手搓,不引 pretext)**:`Intl.Segmenter(granularity:'word')` 找词段 → 逐 word 段累加折行(拉丁不断词、CJK 每字可断),替换逐 grapheme 贪婪折行。
- 4A2 **CJK 禁则(选做,悬挂法)**:`HEAD_BAN=。，、;:?!)」』】》"'%…` / `TAIL_BAN=(「『【《"'`;折行点一次检查:行首禁则字挂本行末(断点右移、略超宽),行尾左括号挤下一行。~20 行。
- 4A3 **拆 H1–H6 分级**:`content.rs` 现合并为一类 `Heading` → 逐级字号(StyleRole 带级别)。
- 4A4 **per-role 精确度量**(Plan2.5 欠账):粗/斜/code 排版宽度(现按 body 度量近似)→ 按角色度量。
- 4A5 **pretext 清理**:删 `web/package.json` 未用的 `@chenglou/pretext`;`pretext-bridge.ts` → 改名 `layout-bridge.ts`、去注释 pretext 命名(0001 §2.2 修订)。

**DoD**
- 英文不再中途断词(`Aus|lia` 消失);CJK 基本不顶行首禁则字;
- H1–H6 字号有层级;粗/斜/code 宽度不再明显偏差;
- `@chenglou/pretext` 已删、文件改名、`npm i` 与构建正常;
- 范围 = LTR(中英),**不做 BiDi**(0001 §2.2)。

---

## Phase 4B — markdown 观感追平(quad 图元 + 装饰)

> 域:`render`(矩形 quad 图元)· `scene`(装饰实例)· `content`/layout(表格、令牌)。追平 GitHub 结构/间距/配色。

**任务**
- 4B1 **矩形/圆角 quad 图元**:在统一实例管线加 rect 图元(0011 L5 占位 → 实做):代码块底、行内码 chip、引用左条、表格网格+斑马+表头、hr、H1/H2 细线、GitHub Alert 左色条。
- 4B2 **表格两趟列宽对齐**:max-content 列宽 + padding `6px/13px` + 边框。
- 4B3 **固化 github-theme 设计令牌**:字号/间距/色(会话实测值)→ 喂 `StyleRole` 映射。

**DoD**
- 与 GitHub 渲染并排:结构/间距/配色基本一致(字形不强求,自带字体决策 0009/0011);
- 表格列对齐、有边框/斑马/表头;代码块/引用/Alert 有底色/左条;
- 矩形 quad 与文字 quad 同相机/裁剪/实例化(不破块冻结)。
- 参考:[0004](../decision/0004-markdown-and-embeds.md)、`github-markdown-css` 令牌。

---

## Phase 4C — 基础调试器(看得见性能,0012)

> 域:`api`(stats getter / setter)· `render`/`scene`(自绘几何)· web(DOM 面板)。先打通数据通道 + P1 流畅组。

**任务**
- 4C1 **数据通道**:`ChatCanvas.stats() -> JsValue`(FrameStats + atlas used/cap/evict + frame_ms + camera);控制 setter(debug flags / 相机);(点选 hit-test 留 P2)。
- 4C2 **DOM 面板(P1 流畅组)**:`?debug` 角落浮层——性能数字 + sparkline(fps/frame_ms)、暂停/单步、atlas used/cap/evict + thrash 警告。
- 4C3 **引擎自绘几何(P1)**:块 AABB 裁剪框、SpatialGrid 格、visible_rect(flag 开关)。

**DoD**
- `?debug` 出面板:实时 fps/帧耗时/发射 glyph vs 总/atlas;prod 不挂、零成本;
- 裁剪框/grid overlay 可切,定位卡顿(如再现"单巨块")一眼可见;
- 不引 egui(0012)。

---

## 节奏与门控

- **4A 先行**(改动小、修可见问题、清理遗留);**4C 尽早**(后续都靠它"边调边看");**4B** 居中(需 4C 验证不破性能更稳)。
- 建议顺序:**4A → 4C(数据通道+P1)→ 4B**;4B 用 4C 的面板验收(出 glyph/帧耗时不退)。
- 每相位末:`cargo fmt/clippy/test` + `wasm-pack build` + `tsc` 过卡口;关键看 `?debug` 帧统计无回归。
- 完成后:愿景层从 [TODO2](../../TODO2.md) 取;产品相位(O/P/Q/R/S)从 [TODO](../../TODO.md) 取。
