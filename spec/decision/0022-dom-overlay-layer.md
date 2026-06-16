# 决策记录 0022:DOM 叠加层 —— canvas 上的世界定位 HTML 逃生口(相机同步 + 主题联动 + 事件钩子)

- 日期:2026-06-16
- 状态:已采纳(模型 + 契约定调;落地分相位,见 §9)
- 前置:0001(画布架构 / content→layout→render 契约)、0007(富媒体嵌入)、0013(math)、[0016](0016-streaming-morph-render-model.md)(形变补间 / `PanelScene`)、[0020](0020-content-node-identity-model.md)(节点身份)、[0021](0021-js-rust-boundary-and-configurable-render.md)(主题源 / JS-Rust 边界)、`drei/src/web/Html.tsx`(范本)、[research/dom-overlay-layer](../research/dom-overlay-layer.md)(调研)
- 触发:canvas 不可能复刻整个 HTML 生态。需要一条**逃生口**——在画布上为真实 HTML DOM 留位(绝对定位 box),随相机缩放/平移,light/dark 主题联动,带位置 + 样式,开放简单事件钩子。该"世界定位 DOM"是 3D 场景的常见模式(drei `<Html>` / CSS3DRenderer / deck.gl)。调研见 [research/dom-overlay-layer](../research/dom-overlay-layer.md)。

---

## 1. 决策:新增「DOM 叠加层」作为富/交互嵌入的逃生口

canvas 渲染管 99%(海量文本/表格/装饰);**少量"画不了或不值得画"的内容(交互控件、`<input>`/`<video>`/`<iframe>`、第三方 widget、需无障碍/可选中)走一层与相机同步的真实 DOM 叠加**。这是 0011「装饰=SDF」与本逃生口的分工,也是 drei `<Html>` 在 3D 场景里的角色。

**与"栅格化进 canvas"两条路并存,按内容分流**(调研 §4):

| | A. 栅格化进 canvas(纹理 quad) | B. DOM 叠加(本 ADR) |
|---|---|---|
| 代表 | 0007/0013(SVG→raster→texture)、emoji RGBA | 本文 |
| 优 | 统一进画布:SDF/morph/相机/裁剪原生一致、单次渲染、海量可扩 | 完整 HTML 生态、可交互、无障碍、可选中、CSS 主题、原生控件 |
| 劣 | 失交互/选中;动态要重栅 | **永在 canvas 之上**(无真遮挡);每帧投影;规模大需剔除 |
| 用 | 静态/展示型富媒体 | 交互/输入/视频/iframe/需 a11y |

## 2. 数据模型:Embed 节点(content)+ 占位世界矩形

content 层产一个**占位节点** `NodeKind::Embed`(0020):它在文档流里占一个**世界矩形**(不可断块),正文围绕/接续它流动。节点带:

```rust
struct EmbedNode {
    id: u64,            // 节点身份(0020;= DOM box 配对键)
    kind: EmbedKind,    // html / iframe / video / custom(宿主按 kind 注册渲染器)
    world: Rect,        // 预留世界矩形(宽高:宿主声明的固有尺寸/宽高比,或 box 量后回报)
    // 内容载荷(url / html / 自定义参数)按 kind 解释,core 不感知语义
}
```

- **留位**:layout 把 `world` 当一个块的盒子,文本不进它内部(canvas 不画其内容)。
- **身份**:`id`(0020)既是文档节点身份,也是 DOM box 的配对键 + 0016 补间键。
- **core 不碰 DOM**:core 只产"这里有个 id+世界矩形+kind+载荷"的占位;真实 DOM 全在 web 层(CR1)。

## 3. 叠加层(web)+ 相机同步

- **容器**:画布上方一个 `position:absolute; inset:0; pointer-events:none; overflow:hidden` 的 `<div>`(与 canvas 同尺寸/父)。主题 CSS 变量挂其上。
- **每个 box**:宿主元素放进容器,`position:absolute; left:0; top:0; transform-origin:0 0; pointer-events:auto`。每帧设
  ```
  transform = translate3d(sx, sy, 0) scale(k)
  ```
  `(sx,sy) = camera.world_to_screen(embed.world.topleft)`;`k = camera.zoom`(**scale 模式**:随缩放放大,drei `distanceFactor` 同精神)或 `k = 1`(**fixed 模式**:固定像素 billboard)。**用 transform(GPU 合成),不用 top/left**。
- **pointer-events**:容器 `none`(空白处事件穿透回画布),box 本体 `auto`(可交互)——drei 关键模式。
- **剔除**:`world` 不在视口 → `display:none`(不投影);限制可见数(CSS3DRenderer 性能建议)。
- **更新门控**:zoom/位置变化超 `eps` 才改 transform。
- **z 序**:DOM 恒在 canvas 之上(§7 限制)。

## 4. 与 0016 形变同步(reflow 不跳变)

embed 的 `world` 矩形随 streaming reflow 变(上方内容增高、列变宽推移)→ **走 0016 补间**(像 `PanelScene`:`id → past/current` 几何插值)。投影用**插值后**的世界矩形 → box **平滑移动/缩放**,与字/框同 `dur`、不跳变。

## 5. 主题联动(单一主题源,接 0021)

engine 主题(light/dark,0021 `RenderStyle`)是**唯一源**:切换时 ① canvas 走 palette(0021);② 叠加容器切 class / 设 CSS 变量(`--fg`/`--bg`/`--accent`…)。box 用 CSS 变量自动换肤 → **一处切换,canvas 与 DOM 同步换主题**。宿主也可订阅 `onThemeChange(mode)` 做非 CSS 的联动。

## 6. 事件钩子(开放给宿主)

- **生命周期**:`onMount(el, ctx)` / `onUnmount(id)`。
- **尺寸回报(关键双向)**:box 量到 DOM 尺寸 → `reportSize(id, worldW, worldH)` → engine 更新 `EmbedNode.world` → 重排(类似 iframe 自适应高度)。
- **交互**:box `pointer-events:auto` → **原生 DOM 事件直接可用**(click/input/…);另暴露 `onVisibilityChange(id, visible)`、`onThemeChange(mode)`、可选 `onTransform({sx,sy,scale})`。
- **engine→web 每帧**:每个可见 embed 给 `{id, sx, sy, scale, visible}`(投影结果),web 应用 transform。复用现有"每帧一次跨界"(AR10):embed 变换随帧数据带出。

## 7. 限制 / 非目标

- **无真遮挡**:DOM 恒盖 canvas 之上,不能与画布内容按深度交错(CSS3D 同限制)。聊天嵌入在最上层,可接受。
- **规模**:每 box 每帧投影 + DOM transform;几百起成本显著 → 剔除屏外 + 限可见数。**面向少量嵌入**,绝不"用 DOM 画整篇 markdown"。
- **不取代 canvas**:文本/表格/装饰仍走 canvas;DOM 只承载 canvas 画不了的。
- **DPR**:transform 用 CSS px(相对画布 CSS 尺寸),与设备像素投影对齐。
- 复杂脚本/RTL 仍非目标(与 0021 一致)。

## 8. 决策小结

采纳 **DOM 叠加层**作富/交互嵌入逃生口:content 产 `NodeKind::Embed` 占位(id + 世界矩形 + kind + 载荷,core 不碰 DOM);web 层叠加容器把宿主 DOM box 按 `world_to_screen` + `scale(zoom|1)` 每帧定位(pointer-events 外 none 内 auto、视口剔除、eps 门控);**接 0016**(世界矩形补间 → reflow 不跳变)、**0020**(id)、**0021**(单一主题源:canvas palette + DOM CSS 变量);开放挂载/尺寸回报/可见性/主题/原生交互钩子。**与"栅格化进 canvas"(0007/0013/TODO O)按内容分流**(§1 表)。

**理由**:canvas 不可能复刻 HTML 生态;DOM 叠加是业界成熟逃生口(drei `<Html>`/CSS3DRenderer/deck.gl),在 2D 正交相机下尤其简单(投影 = `world_to_screen`,缩放 = `zoom`)。把它建成**节点 id 驱动 + 接 0016/0020/0021** 的一等通道,而非临时 hack,使主题/形变/身份与画布统一,代价仅"少量 DOM 每帧投影"。

## 9. 落地清单(分相位;未排期)

- [ ] **最小**:叠加容器 + 单 billboard box(`world_to_screen` + translate/scale 跟随相机、pointer-events 模式、视口剔除);世界位置先用手填锚点。
- [ ] **Embed 节点**:content 产 `NodeKind::Embed`(0020)+ 世界矩形留位;layout 围绕它流;`block_decorations`/build_frame 带出每帧 embed 变换。
- [ ] **尺寸回报 + 重排**:`reportSize` → 更新 world → relayout(复用 `refresh_fonts`/脏判)。
- [ ] **接 0016**:embed 世界矩形走补间(`PanelScene` 同款)→ reflow 平滑。
- [ ] **主题联动**:叠加容器 CSS 变量随 0021 主题源切换;`onThemeChange` 钩子。
- [ ] **钩子 + 声明式**:`onMount/onUnmount/onVisibilityChange/onTransform`;按 `EmbedKind` 注册宿主渲染器(与 0007 embed FSM / TODO O 合流)。
- [ ] scale vs fixed 两模式 + zIndex 序;`?debug` 看 embed 边界。

---

参考先例:本仓 `drei/src/web/Html.tsx`(billboard/transform 双模、`distanceFactor` 缩放、pointer-events 外 none 内 auto、视口剔除、zIndex、eps 门控)· [three.js CSS3DRenderer](https://threejs.org/docs/pages/CSS3DRenderer.html)(DOM 入场景,不参与深度缓冲)· [deck.gl WebMercatorViewport](https://deck.gl/docs/api-reference/core/web-mercator-viewport) / [deck.gl × Mapbox](https://deck.gl/docs/developer-guide/base-maps/using-with-mapbox)(世界→屏幕投影同步相机;interleaved/overlaid)· [three.js HTML overlays/labels](https://www.ramijames.com/learn-threejs/interaction/html-overlays-and-labels)(200+ 元素剔除)· [Google Maps WebGL OverlayView](https://developers.google.com/maps/documentation/javascript/webgl/webgl-overlay-view)。调研全文:[research/dom-overlay-layer](../research/dom-overlay-layer.md);相关 [0007](0007-rich-media-embeds.md) / [0016](0016-streaming-morph-render-model.md) / [0020](0020-content-node-identity-model.md) / [0021](0021-js-rust-boundary-and-configurable-render.md)。
