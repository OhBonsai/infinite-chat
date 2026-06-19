# Plan 13 进度(chat 盒子布局 Taffy + 用户/模型左右分栏 + web 输入框)

- 状态(2026-06-19):**①–⑥ 的 core / tsc 可验部分全部落地 + 测试通过;measure 跨四层接通**。
  Tier C 嵌套子树→Taffy(list/quote 缩进驱动块布局)与「左右上屏 / streaming 平滑 / 实时对话」
  **须人工 GPU + 浏览器 + 本地 opencode serve 验收**(plan §7 已预告)。
- 沙箱约束(同 Plan 12):Claude 有 cargo(native + wasm32 编译/测试已过)与 tsc,但**无 GPU/浏览器**
  → 盒位补间的视觉平滑、左右气泡上屏、实时对话回包渲染须人工实跑。本轮把所有**纯 core / tsc 可验**
  的部分做完并测试,GPU/集成相位给出精确接入点。

## 已落地(验证)

| 相位 | 落地(file:符号) | 验证 |
|---|---|---|
| **① 角色通路** | `store::Role` + `Store.message_role`/`part_role`(store.rs);`PartView.role`(app.rs)+ `refresh_roles`;`group_turns`→`boxlayout::TurnGroup`(app.rs) | cargo:`role_from_snapshot_user_vs_assistant`、`group_turns_user_opens_assistant_groups_into_one_box` |
| **② taffy + Tier A** | `taffy 0.7`(std/taffy_tree/flexbox/block_layout);`boxlayout.rs::layout_chat`(ChatRoot▸Turn▸UserBox(右)/AsstBox(左)▸per-view 叶子);build_frame 收编 `top+=height`,glyph/math/装饰/裁剪/grid/锚底全按盒 origin 平移 | cargo:user 右对齐、assistant 左 0、气泡夹 BUBBLE_MAX、回合竖直堆叠、一回合一 AsstBox(4 测);**GPU 人工**:左右上屏 |
| **③ Tier B 等价 + 锚底** | `disable_rounding()` 保子像素;锚底读盒 origin + 已揭字底(= 末盒 computed bottom) | cargo:`legacy_stacking_equivalence_pm0`(块 y == 旧 `top+=height+BLOCK_GAP`,±0.01)、`anchor_bottom_sticks_to_computed_box_bottom` |
| **④ measure 四层** | `MeasuredSize` + `LayoutEngine::measure`(seam.rs,默认 layout 派生);`MonospaceLayout::measure`(support.rs);wasm `LayoutBridge.measure_fn` + impl(NaN/Inf 防护 + layout 回退);web `measure()`+缓存+`measureCacheStats()`(layout-bridge.ts);config 接线(main.ts / wasm-pkg.d.ts);`?debug` 命中率行(debug-panel.ts) | cargo:`measure_matches_layout_derived_size`、`measure_wraps_at_avail_width`;tsc 绿;wasm32 build 绿 |
| **⑤ 0016 接合** | 盒 origin 已并入 glyph/panel 世界位(②);渲染侧 Scene/PanelScene 按 (block_seq,glyph_idx)/panel id 身份补间该端点 | cargo:`reflow_shifts_box_origin_into_glyph_endpoints`(宽 +200 → 右对齐 user 盒右沿 +200);**GPU 人工**:reflow/列变宽平滑不跳 |
| **⑥ web 输入框** | `web/src/chat-input.ts`(textarea + 发送 + error;Enter 发/Shift+Enter 换行;`POST /session/{id}/message` `{parts,model:{providerID,modelID}}`;`ensureSession`/`parseModel`);main.ts 仅 live server 挂载 | tsc 绿;**人工**:起 opencode serve → 输入 → user 右 + assistant 左流式 |
| **⑦ 基准+卡口** | measure 缓存命中率入 `?debug` perf 行(debug-panel.ts `measureRow`) | core 136 测绿、core+wasm clippy 绿、tsc 绿、wasm32 build 绿 |

## 卡口状态(本轮)

- `cargo test -p infinite-chat-core` → **136 绿**。
- `cargo clippy -p infinite-chat-core --all-targets -D warnings` → **绿**。
- `cargo clippy -p infinite-chat-wasm --all-targets --all-features -D warnings` → **绿**。
- `cargo build -p infinite-chat-core --target wasm32-unknown-unknown` → **绿(CR1:core 零 wasm-bindgen)**。
- `cargo build -p infinite-chat-wasm --target wasm32-unknown-unknown` → **绿**。
- `cd web && tsc --noEmit` → **绿**。
- ⚠ **既有(非本 plan)**:`cargo clippy --workspace --all-targets --all-features` 与 `cargo fmt --all --check`
  在 `crates/render/src/backend.rs` 的 `#[cfg(test)]` WGSL 校验助手处报 `clippy::panic`(workspace `panic="warn"`
  被 `-D warnings` 升级)+ 格式 diff。**HEAD 即如此,先于本 plan**,在 render crate、与 Plan 13 无关——未改动。
- `wasm-pack test --headless --chrome` → 需浏览器,沙箱未跑(人工卡口)。

## 待人工 GPU / 浏览器 / opencode serve 实跑(代码已就位)

- **左右上屏**:user 气泡靠右、assistant 容器靠左,`max_width` 文档宽内排(§4.5 相机正交)。
- **streaming 平滑**:盒位 reflow(上方增高顶下、列变宽)经 0016 Scene/PanelScene 补间不跳变。
- **实时对话**:`?server=…&session=…` 起本地 opencode serve → 输入框发 → assistant SSE 回包走现有 Rust transport 渲染。
- **measure 缓存命中率基准**:`?debug` perf 行实测(沙箱无渲染循环驱动 measure)。

## 仍属 Plan 13 范围、单独排期(plan §2.2 / §7 Open)

- **Tier C 嵌套子树→Taffy**:`nodes.rs` NodeTree 子树映射 Taffy 容器(List/Quote `padding_left=INDENT×depth`、
  Table 叶子 measure=placeTable),让块内缩进/折行由 Taffy 驱动并消费 ④ 的 measure。当前块内堆叠仍由
  LayoutEngine 内部完成(measure 复用其折行 → 与现状几何一致,故 ④ 上线零回归);Tier C 是把该职责上提到
  Taffy 的**两趟 relayout**改造,**须配 GPU 验缩进/折行回归**,接入点 = build_frame `ensure_layouts` 按列宽重排。
- **tool / reasoning part 各自渲染 + 动画**(§2.2):结构已定「一回合一盒、不另气泡」,各 part 类型呈现单独排期。
- **user 富语法**(@ref / 图片 / 文件 / link,0006/0007):v1 纯文本,按语法渲染单独排期。
