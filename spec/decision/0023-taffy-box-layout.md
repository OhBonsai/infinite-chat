# 决策记录 0023:Taffy 盒子布局 —— over 0020 节点树,measureText/placeTable 作叶子 measure

- 日期:2026-06-16
- 状态:已采纳(方向 + measure 边界定调;落地分相位,见 §10)
- 前置:[0001 §2.2](0001-canvas-architecture.md)(排版在 JS / measureText 护城河 / 可替换接口)、[0020](0020-content-node-identity-model.md)(内容节点树)、[0021](0021-js-rust-boundary-and-configurable-render.md)(JS-Rust 边界 / 主题源)、[0022](0022-dom-overlay-layer.md)(DOM 叠加 embed)、[0016](0016-streaming-morph-render-model.md)(形变补间)、0005/0017(状态机/提交前沿)、0014(表格两趟)
- 触发:chat 组件**内容本身**是一棵盒子树(message → 块:段落/标题/代码块/表格/图片/列表项/embed),需要真正的 **box layout**(嵌套、弹性、间距、对齐、块项换行),不能再靠 build_frame 里手搓的"块竖直堆叠"。Zed 的 GPUI 用 **Taffy**(纯 Rust,实现 CSS Block + Flexbox + Grid;前身 Stretch,Yoga 的 Rust 超集)做这件事——本项目同采。

---

## 1. 决策:采用 Taffy 作盒子布局引擎,over 0020 节点树

**每个内容节点(0020)= 一个 Taffy 节点**(带 block/flex/grid 样式 + 可选 measure 函数)。Taffy 算出每个盒子的位置/大小,再喂渲染。它**收编** build_frame 里手搓的块竖直堆叠 + placeTable 的外层排布,统一成一棵可嵌套的盒子树。

链路:
```
LLM 消息流 → 状态机(0005 回合 / 0017 提交前沿)→ 节点树/set(0020,append-only)
   → Taffy 盒子布局(块位置/大小、嵌套、间距、对齐、换行)
   → render(canvas 字/SDF 装饰 + 0022 DOM 叠加)
```

## 2. 节点树 = Taffy 树

0020 的 `Node`(嵌套区间 + parent 下标)直接当 Taffy 树用:`parent` 给层级,`kind` 决定 Taffy 样式 + 是否叶子。

- **容器节点**(Doc/Message/List/Quote/TableCell…)→ Taffy block/flex 容器(竖直堆叠、缩进、gap)。
- **叶子节点**(文本 run 块、表格、图片、embed)→ Taffy 叶子,带 **measure 函数**(§3)。
- **样式来源**:节点 kind → Taffy `Style`(由 0021 的样式数据驱动:margin/padding/gap/direction/align),不写死。

## 3. 叶子 measure:各接各的测量源

Taffy 布局时对叶子调 `measure(known, available) -> Size`。各类叶子接不同测量:

| 叶子 | measure | 来源 |
|---|---|---|
| 文本块 | 在 `available.width` 下折行 → (宽, 行数×行高) | **`measureText`**(JS,系统字体,0001 护城河) |
| 表格 | 像素两趟算列宽 + 格内折行 → 整表尺寸 | **`placeTable`**(0014 B;退化成"表格叶子的 measure") |
| 图片 / embed | 固有尺寸 / 宿主回报 | 0022 `reportSize` / 图片解码尺寸 |
| 代码块 / 公式 | 同文本(等宽)/ 渲染图尺寸 | measureText / 0013 |

**关键:placeTable 不再是"外层排布",降级为表格这个叶子的 measure**;Taffy 负责把表格盒子摆进文档流。

## 4. 关键岔路:文字测量边界(本 ADR 的核心取舍)

**文字测量护城河在 JS(`measureText` + 系统字体),Taffy 是 Rust。** Taffy 布局期要对文本叶子反复 measure(min-content / max-content / 最终宽)。三条路:

1. **Taffy 编进 wasm + measure 回调打到 JS `measureText`**(布局期每文本叶子跨界几次;JS 端 `Map<text+width, size>` 缓存,measureText 本就微秒级)。**守住护城河**;代价 = 布局期多次**小**跨界(非 AR10 的大数据跨界,缓存 + 仅活动块脏 → 可控)。← **采纳**。
2. 文字测量搬进 Rust(cosmic-text + 打包字体)→ Taffy 全 Rust 自洽。**[0001 §2.2]/[0021] 否**(打包中文字体、与浏览器渲染分叉),仅 native 渲染层才考虑。
3. 混合:行内折行仍我们算,Taffy 只用**预量固有尺寸**排块 —— 块高随宽变(折行),固有尺寸近似 → 错排,不准。**弃**。

**AR10 关系**:AR10 约束的是"每帧一次**大数据**跨界(glyph 数组)",不变;Taffy 的 measure 回调是布局期的**小尺寸查询**,带缓存 + 只在活动块脏时跑(append-only,0017),量级可控。把它视为"layout 这一步内部的多次轻量回调",与"每帧一次提交渲染"正交。

## 5. append-only ⇒ 增量布局

Taffy 自带脏标记 / 布局缓存。内容 append-only(0017 提交前沿)→ 只有**活动块尾部子树**脏 → 局部重排,前缀冻结复用(同 0005 块冻结 / 0020 区间)。measure 缓存进一步省 measureText 调用。

## 6. 与现有的收编 / 接合

- **收编 build_frame 块堆叠**:现"`top += height + gap`"竖直堆叠 → Taffy block 容器;块间距/缩进/嵌套(引用、列表)交 Taffy。
- **收编 placeTable 外层**:表格在文档流的位置由 Taffy 定;placeTable 只算表内几何(作叶子 measure)。
- **0016 形变**:Taffy 输出的盒子位置 = morph 的输入。streaming reflow(上方增高/列变宽)→ 盒子位置变 → 字走 `Scene`、框走 `PanelScene`、embed 走其补间(0022 §4)→ **不跳变**。Taffy 给"目标态",0016 给"过渡"。
- **0022 embed**:embed = Taffy 叶子(reportSize → measure),其世界矩形由 Taffy 定,DOM box 投影到该矩形。
- **0021 主题/边界**:Taffy `Style`(margin/gap/direction…)由样式数据驱动、可配置;measure 走 JS(护城河)。

## 7. 边界 / 非目标

- **不做全 CSS**:只用 Taffy 的 block/flex/grid 子集排聊天内容,不追求浏览器全套布局。
- **RTL / 复杂脚本**:非目标(同 0021;真要则连 pretext 一起评估)。
- **文字测量不 Rust 化**(§4.2 否);native 渲染层是另一条线。
- **页面外壳**:外壳若走 web 宿主用 CSS flex 即可;Taffy 主要管 **canvas 内容盒子树**(外壳是否也 Taffy 由宿主形态定,见 §10)。

## 8. 决策小结

采用 **Taffy** 作盒子布局引擎,**over 0020 节点树**(节点=Taffy 节点 + 样式 + measure);**叶子 measure 接 measureText/placeTable/reportSize**;**文字测量走"Taffy(wasm)+ measure 回调打 JS measureText + 缓存"**(守护城河,§4.1);Taffy 输出喂 0016 形变(reflow 不跳变);append-only ⇒ 增量重排。收编 build_frame 块堆叠 + placeTable 外层。

**理由**:chat 内容本质是盒子树,Taffy 是出货验证(Zed/Bevy/Servo/Dioxus)的纯 Rust box-layout,实现 block/flex/grid,正好排块/嵌套/图片/embed;与 0020 节点树合一;唯一硬点是文字测量边界——用"measure 回调 + 缓存"把 measureText 护城河保住,代价可控。

## 9. 风险

- **measure 回调跨界频次**:活动块文本叶子多时,布局期 measureText 回调多 → 靠缓存(text+width)+ 只重排活动块 + measureText 本身快。需基准:长消息 reflow 的布局耗时。
- **wasm 包体**:Taffy 进 wasm 增体积(纯 Rust,中等);可接受。
- **契约扩张**:content→layout 从"扁平 run + sidecar"升到"节点树 + Taffy 样式";是 0020 的自然延伸,但改动面大,需与 0020 同期落地。

## 10. 落地清单(分相位;未排期,chat 页面期做)

- [ ] 引 `taffy` crate(core,wasm 目标);节点 kind → Taffy `Style` 映射(样式数据驱动,0021)。
- [ ] 0020 `Node` 树 ↔ Taffy 树构建(append-only 增量;前缀冻结)。
- [ ] 叶子 measure:文本(measure 回调 → JS measureText + 缓存)、表格(placeTable)、图片/embed(reportSize)。
- [ ] Taffy 布局输出 → 盒子位置 → 喂 build_frame(替手搓堆叠)→ 0016/PanelScene/embed 补间。
- [ ] 基准:长消息 reflow 布局耗时 + measureText 回调次数(缓存命中率)。
- [ ] (定形态)页面外壳:web CSS flex vs Rust/Taffy 统一,由宿主集成形态定。

---

参考先例:[Taffy](https://github.com/DioxusLabs/taffy)(纯 Rust,CSS Block+Flexbox+Grid;前身 Stretch)· [zed-industries/taffy](https://github.com/zed-industries/taffy) + [GPUI `TaffyLayoutEngine`](https://deepwiki.com/zed-industries/zed/2-gpui-framework)(Zed 出货用法:UI 树先 taffy 量+定位再交渲染;每 window 一个 engine)· 也用于 Bevy / Servo / Blitz / Dioxus。接合:[0020](0020-content-node-identity-model.md)(树)/ [0001 §2.2](0001-canvas-architecture.md)·[0021](0021-js-rust-boundary-and-configurable-render.md)(measure 护城河/样式)/ [0016](0016-streaming-morph-render-model.md)(形变)/ [0022](0022-dom-overlay-layer.md)(embed 叶子)。
