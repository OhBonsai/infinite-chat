# 决策记录 0012:可观测 / 调试器 GUI —— HTML DOM 浮层 vs Rust egui

- 日期:2026-06-14
- 状态:已采纳
- 前置:0001(架构 / content→layout→render 契约)、0003 §5(降级 / 可观测)、0011(SDF 图元 / 后端分级)、[TODO 可观测性](../../TODO.md)
- 来源:本轮"调试器/HUD 用 HTML 还是 egui"的讨论

## 1. 背景

运行时可观测已落一层:`FrameStats`(发射/总 glyph、可见/总块)+ atlas 占用/容量/淘汰 + RAF 帧耗时/丢帧,`?debug` 时节流打 `tracing target=perf` 到 console(见 TODO 可观测性、0011 §8.5 落地)。

下一步想要一个**调试器 / dev 面板**:
- **读数**:上述 stats 的实时面板(fps、帧耗时、emit/total、atlas、evict…);
- **控件**:字体 option、SDF 参数(RADIUS/edge/band)热调、debug 可视化开关;
- **调试几何**:裁剪框、atlas 页查看、glyph 包围盒、相机网格(Turitzin 式 debug viz)。

问题:这套 GUI **用 HTML(DOM 浮层)还是 Rust(egui)** 写。

## 2. 选项与对比

| 维度 | A. HTML/DOM 浮层 | B. Rust egui(egui + egui-wgpu + epaint) |
|---|---|---|
| **包体**(小包体优先,0011/字体决策) | wasm **零增**;dev-only `<div>`,prod 不挂 | **数百 KB+**;即使 dev-only 也须 feature-gate 才不进 prod |
| **读引擎状态** | 需把 stats 跨 wasm 边界暴露给 JS(小成本) | **Rust 原生直读**,无边界——egui 唯一真优势 |
| **控件** | HTML input 原生;字体本就是 JS/CSS 串 | egui 自带 slider/plot,但走其 IM 范式 |
| **集成复杂度** | 浮层贴上即可,与渲染管线**完全解耦** | 多挂一个 egui render pass + **输入路由**(与画布 pan/zoom 抢指针/键盘)+ z-order |
| **prod 剥离** | `?debug` 不挂,天然零成本 | 必须 `#[cfg(feature=…)]` 整套排除 |
| **出 bug 影响** | 调试器挂了不影响渲染 | 共用 wgpu pass + 输入,耦合 |
| **数据来源现状** | stats 已算好,加 getter 即可 | 同能读,但要进 egui |

(C. 维持现状:只 console 打印——不满足"面板 + 控件 + 调试几何"。)

## 3. 决策

**采纳 A:HTML/DOM 面板 + 引擎自绘调试几何;否决 egui。** 即:

1. **DOM 控制面板**(stats 文字 + 控件):`?debug` 时挂 `position:fixed` 角落 `<div>`,纯 HTML/CSS。
2. **调试几何**(裁剪框/atlas 页/包围盒/相机网格)数据在 Rust/渲染侧 → **由引擎自绘**、flag 开关;**不交给 egui**。
3. 即 **面板(数字+控件)走 DOM,调试几何走引擎自绘**,两边都不引 egui。

理由(均扣项目原则):

- **小包体 / 可嵌入组件**:DOM 浮层对 wasm 零增、dev-only 天然剥离;egui 数百 KB 且需 feature-gate,与"小包体优先"(字体决策、0011)正面冲突。
- **需求是简单控件 + 读数**(字体 option、几个开关/滑杆),不是深挖引擎内部的字段级 inspector → DOM 足够;egui 的"原生读状态"优势抵不过其成本。
- **解耦**:DOM 调试器不碰渲染管线/输入;egui 共用 wgpu pass 与指针/键盘,易互扰。
- **已身处 DOM 页**:字体 option 尤其是 JS/CSS 串的事(`pretext-bridge` 的 `SANS`/`MONO`),本就该在 JS 侧。

egui 的唯一好处(无边界读 Rust 状态)用一个 `ChatCanvas.stats()` getter 即可替代。

## 4. 落地清单:调试组件矩阵 + 数据流 + 优先级

目标导向:最终是**无比流畅 + 效果上限**的 chat 页 →(流畅)性能/裁剪/atlas、(上限)渲染正确性 + 实时调参、(chat)流式/FSM/解析。判定原则:**要对齐世界/相机坐标的 → 画布(引擎自绘);数字/图表/控件 → DOM。**

### 4.1 组件矩阵

| 子系统 | 关键数据 | 画布(引擎自绘) | DOM(面板/控件) | 数据方向 |
|---|---|---|---|---|
| 性能/帧 | fps、frame_ms(build/draw)、丢帧、RAF 抖动、wasm 内存、GPU 时间戳 | (可选)帧耗时条 | 数字 + sparkline;暂停/单步 | 引擎→DOM |
| 裁剪/空间 | 总块·glyph / 可见(emit 比)、grid 占用、visible_rect | **块 AABB 框、grid 格、可见矩形** | emit/total、overlay 开关 | 计数引擎→DOM;开关 DOM→引擎 |
| atlas/缓存 | used/cap、淘汰、页数、tile kind、thrash | **atlas 纹理查看、本帧新/淘汰高亮** | used/cap/evict、thrash 警告 | 引擎→DOM;viewer 开关 DOM→引擎 |
| 布局/文字 | glyph quad、baseline、行盒、advance vs cell、折行点、role/kind | **glyph 包围盒、baseline、行框、按 role/kind 上色** | 开关;点选读 cluster/role/pos/size | overlay 引擎;点选 hit-test 引擎→DOM |
| 流式/smoother | 缓冲水位、有效 cps、spawn_time 分布、本帧新字、Turn/Part/Tag FSM、对账事件 | **按年龄上色、生长尾高亮** | 水位计、cps 读数、FSM 徽章、事件日志;cps 滑杆/暂停/重放 | 状态引擎→DOM;控制 DOM→引擎 |
| 特效调参 | SDF(RADIUS/edge/band)、fade_ms、glow/描边/溶解、compute 开关、profile | (可选)单字放大预览 | **全部滑杆/下拉热调** | DOM→引擎(setter→uniform) |
| 相机/画布 | pan/zoom、visible_rect、LOD 档 | **可见矩形、LOD 阈值线** | pan/zoom 读数、reset/缩放到适配 | 双向 |

### 4.2 数据流(3 条 dev-only 通道)

1. **状态快照拉取(引擎→DOM,读)**:`ChatCanvas.stats() -> JsValue` 返回一帧快照(FrameStats + atlas + frame_ms + camera + smoother 水位 + FSM 状态);面板**每 ~200ms 拉一次**(非每帧,省跨界)。现有 `FrameStats` 扩字段即可。
2. **控制下发(DOM→引擎,写)**:setter —— debug flags(哪些 overlay 开)、特效参数(写 shader uniform/profile)、cps、相机操作;批量下发(沿用 AR10 精神,调试期可放宽)。
3. **点选查询(DOM→引擎→DOM)**:指针点画布 → 引擎 **hit-test**(用 [0011 §3.3④](./0011-gpu-text-as-sdf-primitive.md) 的 CPU 基础盒模型,不回读 GPU)→ 回传实体信息给 inspector。
- 事件类(FSM 跳变/对账)走**环形缓冲**:引擎写、面板拉取时一并取走,避免逐条跨界。
- 画布 overlay **不走数据回传**(数据本在引擎内),DOM 只发开/关标志 → 跨界量仅"快照 + 控制 + 点选"。

### 4.3 优先级(扣目标)

1. **流畅组**(第一优先):性能数字+sparkline(DOM)+ 裁剪框/grid(画布)+ atlas used/cap/evict(DOM)。直接暴露卡顿根因(如单巨块)。
2. **质量组**:glyph 包围盒/baseline/atlas viewer(画布)+ 点选 inspect(DOM)。布局/字形正确 = 上限地基。
3. **效果上限组**:特效参数热调(DOM)+ 按年龄上色(画布)+ smoother 水位/FSM(DOM)。边调边看逼近上限。

### 4.4 门控
- 面板 + 自绘几何全挂 `?debug` 后;prod 不挂 → **零成本,守小包体**。字体 option 走 JS(改 `ctx.font` 来源,不过 wasm)。

## 5. 后果

- prod **零调试开销**;调试器与渲染**解耦**(调试器 bug 不伤渲染)。
- 深挖 Rust 内部需经边界 getter(可接受,数量小);若将来要"字段级原生 inspector"再议(见下)。

## 6. 重新评估触发条件

- 需要**深挖引擎内部、字段超多的 Rust 原生 inspector**,**且**包体不在乎(独立 dev 构建 / 非嵌入组件)→ 再评估 egui(届时 feature-gate 出 prod)。
- 若项目从"可嵌入 wasm 组件"转为**独立桌面/原生 app** → egui 的包体顾虑消失,可重开。

## 7. 来源 / 链接

- 现有可观测:`crates/core` `FrameStats`、`crates/render/atlas` stats、`crates/wasm/lib.rs` RAF `?debug` perf 日志;[TODO 可观测性](../../TODO.md)、[0011 §8.5]、[plan3_progress §8.5](../plan/plan3_progress.md)
- 相关:0001(契约)、0003 §5(降级/可观测)、0011(SDF/后端分级)、字体决策(正文系统字体栈 / 小包体优先)
