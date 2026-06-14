# Plan 4 实施总结(plan4-polish 4A/4B/4C 落地记录)

- 日期:2026-06-14
- 范围:[plan4-polish.md](./plan4-polish.md) 的 **4A 排版收口 / 4B markdown 观感 / 4C 基础调试器**
- 状态:**4A / 4C 已落地并提交;4B 核心落地**(矩形 quad 图元 + 装饰 + 调试几何);表格两趟对齐等观感尾账留下一轮(见 §6)
- 提交:`6f059fd`(4A)→ `b96fb3a`(4C)→ **4B 本次提交**(矩形图元 + 装饰 + 调试几何)
- 设计源:[决策 0011](../decision/0011-gpu-text-as-sdf-primitive.md)(L5 统一实例管线)、[0012](../decision/0012-debugger-gui-html-vs-egui.md)(DOM 面板不引 egui)、[0001 §2.2 修订](../decision/0001-canvas-architecture.md)(不引 pretext)

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
├── content.rs   # StyleRole 拆 H1–H6(Heading2..6);heading(level)/is_heading()(4A3)
├── frame.rs     # 新增 FrameRect(pos/size/color/radius/stroke);FrameData.rects(先于 glyphs 绘)
└── app.rs       # block_decorations():角色 run → 装饰 rect;debug_geometry flag + 块 AABB/视口框(4B/4C3)
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
| **4B** markdown 观感 | 矩形 quad 图元 + 装饰 | `FrameRect`/`RectInstance`/`rect.wgsl`(圆角 SDF);`block_decorations()` 从角色 run 派生:代码块底(圆角)/ 行内码 chip(逐行聚)/ 引用左条 / H1·H2 底线;rect 管线只绑 globals,与文字同相机/裁剪 | core 装饰 3 测(code bg / inline chip / debug geo) + naga rect shader 校验 |
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

---

## 5. 卡口

- `cargo fmt --check` ✓ / `cargo clippy --workspace --all-targets -D warnings` ✓(native + `--target wasm32`)
- `cargo test --workspace` ✓:**core 57 native + 3 replay**、**render 6**(含 rect.wgsl naga 校验)
- `wasm-pack build --target web` ✓ / `cd web && npx tsc` ✓

---

## 6. 尾账(留下一轮 / 仍属 4B 但未做)

- **4B2 表格两趟列宽对齐**:max-content 列宽 + padding + 边框,需动 `layout-bridge` 做两趟度量
  (现表格按行 `" │ "` 平铺)。属较大改动,单独一轮更稳。
- **4B1 其余装饰**:表格网格 / 斑马 / 表头、`hr`、GitHub Alert 左色条 —— 需 content 侧把
  这些块/令牌识别出来(Alert 需解析 `> [!NOTE]`),再走同一 `block_decorations` 套路补 rect。
- **4B3 设计令牌固化**:把 github-theme 字号/间距/色实测值收进集中表喂 `StyleRole` 映射(现散在
  `layout-bridge` 的 `roleScale` 与 shader `style_color`)。
- 这些都不阻塞「矩形 quad 与文字 quad 同相机/裁剪/实例化(不破块冻结)」这条 4B 核心 DoD。
