# 研究:DOM 叠加层(世界定位 + 随相机缩放/平移 + 主题联动 + 事件钩子)

- 日期:2026-06-16
- 状态:研究 / 需求陈清 → **已升 [ADR 0022](../decision/0022-dom-overlay-layer.md)**(决策/契约定调;本文为其调研底稿)
- 触发:canvas 不可能复刻整个 HTML 生态。需要一条**逃生口**:在画布上为真实 HTML DOM 留位(绝对定位的 box),能随相机缩放/平移,能 light/dark 主题联动,带位置 + 样式,并开放一些简单事件钩子。作者指出"这个定位问题在 3D 场景里很常见"。

---

## 1. 需求陈清(先确认我的理解)

你要的是一层**与画布相机同步的 DOM 叠加层**,用来嵌入 canvas 自身画不了(或不值得画)的"完整 HTML 生态"内容——比如可交互控件、`<input>`/`<video>`/`<iframe>`、复杂富媒体、第三方 widget、需要无障碍/可选中/可复制的内容等。具体诉求拆成五点:

1. **留位**:在排版里给一个 DOM box 预留一块**世界矩形**(占位),正文围绕/接续它流动。
2. **跟随相机**:box 的屏幕位置随相机 pan/zoom 实时更新(可选"随缩放放大"或"固定像素大小"两种)。
3. **主题联动**:engine 切 light/dark 时,DOM box 跟着换样式(单一主题源)。
4. **位置 + 样式**:box 同时拿到几何(位置/大小)与样式(主题令牌)。
5. **事件钩子**:开放挂载/尺寸/点击/可见性/主题变更等简单 hook,让宿主接 DOM 行为;box 也能把量到的尺寸**回报**给 engine 触发重排。

定性:这是 **WebGL/Canvas 场景上叠真实 DOM** 的经典模式(three.js CSS3DRenderer、`@react-three/drei <Html>`、deck.gl/Mapbox HTML marker、Google Maps WebGL OverlayView 都是它),只是我们是 **2D 正交相机**,比 3D 简单很多。

## 2. 业界实践(取观念)

**`@react-three/drei <Html>`(本仓 `drei/src/web/Html.tsx`,最贴近的范本)**
每帧把对象世界坐标 `project(camera)` → NDC → 屏幕 px,然后给一个 `position:absolute` 的 DOM 元素设 `transform: translate3d(x,y,0) scale(s)`(`transform-origin:0 0`)。要点全在这:
- **两种模式**:① 默认"billboard":只 translate,`distanceFactor` 决定是否随距离/缩放缩(给则缩、不给则固定像素);② `transform`:用 `matrix3d` 跟 3D 朝向(我们 2D 用不上)。正交相机的缩放因子 = `camera.zoom`(正合我们)。
- **pointer-events**:外层容器 `none`(事件穿透回画布),box 本体 `auto`(可交互)。**关键模式**。
- **可见性**:背向相机/视口外 → `display:none`(剔除);深度 → `zIndex`。
- **更新门控**:zoom/位置变化超 `eps` 才改 transform(省 reflow)。
- **portal/target**:DOM 挂到画布父节点;尺寸由 `useThree().size` 给。
- 遮挡(raycast/blending)是 3D 专属,我们不需要。

**three.js CSS3DRenderer**:把 DOM 当 3D 对象放进场景。**核心限制**:CSS3D 元素**不参与 WebGL 深度缓冲**——DOM 永远盖在 canvas 之上,无法与 canvas 内容真正前后遮挡(除非特殊 blending 组合)。**性能**:每帧更新每个元素,**200+ 元素时 DOM 更新成本显著** → 剔除屏外、IntersectionObserver、限制可见数。

**deck.gl / Mapbox**:`WebMercatorViewport` 每帧把世界坐标投影到屏幕像素,与地图相机(zoom/rotate)严丝合缝同步;HTML marker 用虚拟 DOM 框架管理。和 canvas 的关系分 **interleaved / overlaid / reverse-controlled** 三档——对应"DOM 与画布如何叠"。大规模 marker 要靠虚拟 DOM + 剔除。

**共同结论**:DOM 叠加 = 每帧"世界→屏幕投影 + CSS transform";天生**永远在 canvas 之上**(无真遮挡);规模大要剔除/虚拟化;换来的是**完整 HTML 生态 + 可交互 + 无障碍 + CSS 主题**。

## 3. 映射到我们(2D 正交,更简单)

我们已有相机 `Camera2D`(`world_to_screen`)、节点身份(0020)、形变补间(0016 `Scene`/`PanelScene`)。DOM 叠加层只是再加一个"非 canvas 通道":

- **叠加容器**:画布上方一个 `position:absolute; inset:0; pointer-events:none; overflow:hidden` 的 `<div>`(与 canvas 同尺寸/同父)。主题 CSS 变量挂在它上。
- **一个 DOM box**:宿主提供的元素放进容器,`position:absolute; left:0; top:0; transform-origin:0 0; pointer-events:auto`。每帧设:
  ```
  transform = translate3d(sx, sy, 0) scale(k)
  ```
  `(sx,sy) = camera.world_to_screen(box.worldPos)`;`k = camera.zoom`(随缩放)或 `k=1`(固定像素 billboard)。**用 transform(GPU 合成)不用 top/left**。
- **留位(reserve slot)**:content 层产一个**占位节点/块**,带世界矩形(宽高来自宿主声明的固有尺寸/宽高比,或 box 量到后回报);layout 把它当一个不可断块,正文围绕/接续。box 跟踪这个世界矩形。
- **与 morph 同步**:占位节点的世界矩形可走 0016(像 `PanelScene` 那样补间)→ streaming reflow 时 box **平滑移动**,不跳变。投影用**插值后**的世界矩形。
- **主题联动**:engine 主题(light/dark,见 0021 `RenderStyle`)→ 在叠加容器上切 class / 设 CSS 变量(`--fg`/`--bg`/…)→ box 用 CSS 变量自动换肤。单一主题源,canvas(palette)与 DOM(CSS vars)同切。
- **事件钩子(开放给宿主)**:
  - 生命周期:`onMount(el, ctx)` / `onUnmount`。
  - 几何:`onTransform({sx,sy,scale,visible})`(一般内部用,可选暴露)。
  - 尺寸回报:box 量到 DOM 尺寸 → `reportSize(worldW, worldH)` → engine 重排(类似 iframe 自适应高度)。
  - 交互:box `pointer-events:auto` → **原生 DOM 事件直接可用**(click/input/…);另给 `onVisibilityChange(visible)` / `onThemeChange(mode)`。
- **剔除**:世界矩形不在视口 → `display:none`(不投影、不 transform),同 CSS3DRenderer 性能建议。
- **z 序**:DOM 恒在 canvas 之上(限制,见 §5)。聊天嵌入物本就在最上层,够用。

## 4. 与"栅格化进 canvas"的取舍(两条嵌入路)

富媒体嵌入有两条路,**正交并存,按内容选**:

| | A. 栅格化进 canvas(纹理 quad) | B. DOM 叠加层(本研究) |
|---|---|---|
| 代表 | 0007/0013(mermaid SVG→raster→texture)、emoji RGBA | 本文 |
| 优点 | 统一进画布:与 SDF/morph/相机/裁剪原生一致;单次渲染;海量可扩 | 完整 HTML 生态、可交互、无障碍、可选中、CSS 主题、原生控件 |
| 缺点 | 失交互/无障碍/选中;动态内容要重栅 | **永在 canvas 之上**(无真遮挡);每帧投影;规模大需剔除/虚拟化;另一套渲染路 |
| 适用 | 静态/展示型富媒体(图、图表、公式渲染图) | 交互/输入/视频/iframe/第三方 widget/需 a11y |

**定位**:canvas 管 99%(海量文本/表格/装饰),**DOM 叠加是少数富/交互嵌入的逃生口**——正如 drei `<Html>` 在 3D 场景里的角色。

## 5. 限制 / 非目标

- **无真遮挡**:DOM 恒盖 canvas 之上,不能与画布内容按深度交错(CSS3D 同限制)。聊天场景可接受。
- **规模**:每个 box 每帧投影 + DOM transform;几百个起 DOM 成本显著 → 必须剔除屏外 + 限制可见数(同业界)。本逃生口面向**少量**嵌入,不是"把整篇 markdown 用 DOM 画"。
- **不取代 canvas 渲染**:文本/表格/装饰仍走 canvas;DOM 仅承载 canvas 画不了的。
- **坐标/DPR**:投影需对齐设备像素 vs CSS px(transform 用 CSS px,relative 画布的 CSS 尺寸)。

## 6. 落地建议(分阶段,未排期)

1. **最小**:叠加容器 + 单个 billboard box(translate+scale 跟随相机,pointer-events 模式,剔除)。世界位置先用一个手填锚点。
2. **留位 + 回报**:content 产占位节点(世界矩形)+ box 尺寸回报 → 重排;接 0016 让 reflow 平滑。
3. **主题 + 钩子**:CSS 变量主题联动(并 0021);开放 onMount/reportSize/onVisibility/onTheme。
4. **声明式嵌入**:content 的"embed 节点"(kind=html)→ 宿主按 kind 注册 DOM 渲染器(与 0007 embed FSM、TODO O 合流)。

## 7. 与现有的关系

- **0007 富媒体嵌入 / 0013 math / TODO O**:那条偏"栅格化进 canvas";本研究是其**交互/HTML 生态**的另一半(路 B)。二者按 §4 分流。
- **0020 节点身份**:DOM box 用节点 id 定位 + 配对(留位节点 = 一种 `NodeKind::Embed`)。
- **0016/PanelScene**:box 世界矩形走补间 → reflow 不跳变(同表格框)。
- **0021 主题/可配置**:主题源统一,canvas 走 palette、DOM 走 CSS 变量,一处切换两处生效。

---

参考:本仓 `drei/src/web/Html.tsx`(billboard/transform 双模、`distanceFactor` 缩放、pointer-events 外 none 内 auto、视口剔除、zIndex 深度、eps 门控)· [three.js CSS3DRenderer](https://threejs.org/docs/pages/CSS3DRenderer.html)(DOM 入 3D 场景,不参与深度缓冲)· [HTML overlays/labels in three.js](https://www.ramijames.com/learn-threejs/interaction/html-overlays-and-labels)(CSS2D 每帧更新、200+ 元素剔除)· [deck.gl WebMercatorViewport](https://deck.gl/docs/api-reference/core/web-mercator-viewport)(世界→屏幕投影同步相机)· [deck.gl × Mapbox 集成](https://deck.gl/docs/developer-guide/base-maps/using-with-mapbox)(interleaved/overlaid/reverse-controlled)· [Mapbox GL DOM marker 性能](https://codepen.io/hyperknot/pen/PBGzME) · [Google Maps WebGL Overlay View](https://developers.google.com/maps/documentation/javascript/webgl/webgl-overlay-view)。
