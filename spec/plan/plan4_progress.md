# Plan 4 实施总结(plan4-polish 4A/4B/4C 落地记录)

- 日期:2026-06-14
- 范围:[plan4-polish.md](./plan4-polish.md) 的 **4A 排版收口 / 4B markdown 观感 / 4C 基础调试器**
- 状态:**4A / 4B / 4C 全部落地**。4B 唯「表格两趟对齐」经 [决策 0014](../decision/0014-table-two-pass-layout.md) 拆为独立相位(非遗留欠账,理由见 §6)
- 提交:`6f059fd`(4A)→ `b96fb3a`(4C)→ `efa73e9`(4B 矩形图元/装饰/调试几何)→ **本次**(引用条/Alert/hr/设计令牌 + 表格 ADR)
- 设计源:[决策 0011](../decision/0011-gpu-text-as-sdf-primitive.md)(L5 统一实例管线)、[0012](../decision/0012-debugger-gui-html-vs-egui.md)(DOM 面板不引 egui)、[0001 §2.2 修订](../decision/0001-canvas-architecture.md)(不引 pretext)、[0014](../decision/0014-table-two-pass-layout.md)(表格拆相位)

---

## 1. 一句话

把 Plan 3 的「能渲染」收口成「观感对、可调试」:**4A** 词边界折行 + CJK 禁则 + H1–H6 分级 +
per-role 度量(并清掉 pretext 遗留);**4B** 在统一实例管线加**矩形/圆角 quad 图元**,从字形角色
派生代码块底 / 行内码 chip / 引用左条 / H1·H2 细线;**4C** 打通 `stats()` 数据通道 + `?debug` DOM
面板 + 引擎自绘调试几何。`core` 契约(0001 §2.2)不动,矩形与文字同相机/裁剪/实例化、不破块冻结。

---

## 2. 交付物(相对 Plan 3 的增量)

```
crates/core/src/
├── content.rs   # StyleRole 拆 H1–H6 + 新增 Rule/AlertLabel;blockquote→Quote;mark_alerts 哨兵;hr→Rule 锚点
├── frame.rs     # 新增 FrameRect(pos/size/color/radius/stroke);FrameData.rects(先于 glyphs 绘)
├── theme.rs(新)# 4B3 设计令牌单一出处:装饰色 + alert_bar/alert_bg(按类型)
└── app.rs       # block_decorations():角色 run → 装饰 rect(码底/chip/引用·Alert 条+淡底/标题线/hr);debug_geometry(4B/4C3)
crates/render/src/
├── scene.rs           # 新增 RectInstance(#[repr(C)] Pod,5 顶点属性)
├── backend.rs         # 第二条 rect 管线(仅绑 globals,无 atlas);draw(glyphs, rects, …):rect 先画作背景
└── shaders/rect.wgsl  # 圆角矩形 SDF(sd_round_box)+ fwidth AA;stroke>0 = 仅边框(调试框)
crates/wasm/src/
└── lib.rs       # GpuSink 从 frame.rects 建 RectInstance 传 draw;ChatCanvas.set_debug_geometry()
web/src/
├── layout-bridge.ts # (4A)Intl.Segmenter 词折行 + 轻禁则 + per-role advance + roleScale(H1..H6)
├── debug-panel.ts   # (4C2)?debug DOM 浮层:fps sparkline / 暂停单步 / atlas thrash;本次加 geometry 切换钮
└── wasm-pkg.d.ts    # ChatCanvas.set_debug_geometry 声明同步
```

---

## 3. 各相位落地 + 验证

| 相位 | 内容 | 关键实现 | 验证 |
|---|---|---|---|
| **4A** 排版收口 | 词折行 / CJK 禁则 / H1–H6 / per-role 度量 / 清 pretext | `Intl.Segmenter('word')` 逐词段折行(拉丁不断词、CJK 可断);HEAD_BAN 悬挂禁则;`StyleRole` 带级别 + `roleScale`;`pretext-bridge`→`layout-bridge` 改名删依赖 | content 标题分级测 + 真机:`Aus\|lia` 断词消失、H1–H6 有层级 |
| **4B** markdown 观感 | 矩形 quad 图元 + 装饰 + 令牌 | `FrameRect`/`RectInstance`/`rect.wgsl`(圆角 SDF);`block_decorations()` 从角色 run 派生:代码块底 / 行内码 chip / 引用左条 / **Alert 类型色条+淡底** / H1·H2 底线 / **hr 细线**;blockquote→Quote、`mark_alerts` 哨兵穿透 jcode、hr→Rule 锚点;**`theme.rs` 固化设计令牌**;rect 管线只绑 globals,与文字同相机/裁剪 | core 装饰/内容 8 测(code bg/chip/alert 条+底/hr/blockquote/alert label/rule) + theme 2 测 + naga rect shader |
| **4B2** 表格对齐 | 拆出独立相位 | 比例字体二维受限布局 + 契约扩张过大 → [决策 0014](../decision/0014-table-two-pass-layout.md):A 等宽网格(下一相位)→ B 比例实测(升级) | — |
| **4C** 基础调试器 | 数据通道 + DOM 面板 + 自绘几何 | `ChatCanvas.stats()` 出 FrameStats+atlas+frame_ms+zoom;`?debug` 浮层(sparkline/暂停单步/thrash);`set_debug_geometry` → 块 AABB 描边 + 视口框 | wasm stats/pause/step + 面板 geometry 钮;`?debug` 帧统计无回归 |

---

## 4. 关键设计点

- **角色派生装饰(无新契约)**:装饰不进 content/layout 契约,而在 `build_frame` 末由
  `BlockCache` 的 `roles`/`placed` 反推几何(代码块取 CodeBlock run 的 y 包络、行内码按同行聚成
  chip、引用取 Quote run、H1/H2 在块底画线)。**好处**:零协议改动、随块冻结缓存自然复用、
  每帧只对可见块算(已过空间索引裁剪)。
- **第二条管线、同相机**:rect 复用 `Globals`(同 viewport/cam_pan/cam_zoom),独立 bind layout
  (无 atlas 纹理/采样器)。`draw(glyphs, rects, …)` 一个 render pass 内**先画 rect 作背景再画字**。
- **圆角 + 描边一套 shader**:`sd_round_box` 有符号距离场,`stroke==0` 实心填充、`stroke>0` 取
  内外两次 smoothstep 之差 = 仅边框 → 调试框/视口框复用同图元。`fwidth` AA 任意缩放锐利。
- **调试几何走 flag、直连引擎**:`set_debug_geometry` 直接 mutate engine(面板由人手触发,
  start 之后 state 必已就位),无需多穿一个共享 flag 过 `init_and_run`。
- **GitHub Alert 哨兵穿透 jcode**:vendored jcode 底层 pulldown 无 GFM alert 扩展,会把
  `[!NOTE]` 当空链接**吞掉**(实测首行只剩 `│ ` 栏线)。故在 `parse_markdown` 前用
  `mark_alerts` 把 `> [!TYPE]` 的标记换成私用区哨兵 `\u{E000}TYPE`,使其以纯文本穿过 jcode,
  再在 `emit_blockquote` 还原为 `AlertLabel` 角色;同时剔除 jcode 自带的 `│ ` 字面栏线(改画
  矩形左条)。类型色在 `block_decorations` 由 `AlertLabel` 字形拼回的标签查 `theme::alert_*`。
- **hr = 零墨锚点**:`ThematicBreak` 不再吐 `─` 字符,改发一个 `StyleRole::Rule` 的空格(无墨),
  render 侧据其 y 画整宽细线 rect——与文字管线解耦,缩放下恒锐。
- **设计令牌单一出处(4B3)**:装饰色集中进 `crate::theme`(对标 github-markdown-css 深色);
  文字色仍在 `glyph.wgsl::style_color`、字号比例在 `layout-bridge.ts::roleScale`,三处构成跨语言
  令牌表,`theme.rs` 文件头注明分工。

---

## 5. 卡口

- `cargo fmt --check` ✓ / `cargo clippy --workspace --all-targets -D warnings` ✓(native + `--target wasm32`)
- `cargo test --workspace` ✓:**core 64 native + 3 replay**、**render 6**(含 rect.wgsl naga 校验)
- `cargo fmt --check` ✓ / `cargo clippy …`(native + wasm32)✓ / `wasm-pack build` ✓ / `tsc` ✓

---

## 6. 唯一拆出项:表格两趟对齐(4B2 → 独立相位)

`hr` / GitHub Alert / 设计令牌固化本轮**已交付**;仅 **4B2 表格列对齐**拆出,**理由经
[决策 0014](../decision/0014-table-two-pass-layout.md) 定调**:它不是"加个装饰",而是给扁平
`(text, role)` 文本管线补一条**比例字体下的二维受限布局**子路径——列宽要 JS 实测、单元格要
受限宽再折行、还要扩 content→layout 契约带表结构,量级与风险比 4B 其余项高一个级别,夹在 4B
里仓促做会破坏 AR10 契约稳定。

落地分两步(0014 §4):**A 等宽网格**(纯 Rust、可测、列必对齐、复用已就位的矩形图元画
网格/斑马/表头,**不碰契约**)作下一相位 v1;**B 比例体实测两趟**列为升级项,需单独评审。
现状表格维持 `" │ "` 平铺直到 A 落地。
