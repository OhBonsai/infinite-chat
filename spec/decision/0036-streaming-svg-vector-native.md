# 0036 · streaming-svg = 矢量原生(路 B),翻转 0007 的「流式 SVG 走纹理」条

- 日期:2026-07-13(Plan 31 R4)
- 状态:**采纳**(append-only;0007 原文不改,其「SVG 一律整图纹理」条由本篇取代;
  位图/动图路线不变)
- 参考:[research/image-rendering-warp-jcode §5](../research/image-rendering-warp-jcode.md) ·
  [0035 math display list](0035-math-ratex-display-list-first-class.md)(同范式)·
  [0016 morph](0016-streaming-morph-render-model.md) · [0025 SDF 动画](0025-sdf-node-animation-system.md)

## 1. 决策

流式 SVG(assistant 生成的行内 `svg` 围栏 / .svg 内容)**矢量原生解析 → 复用既有 SDF/quad
图元管线**,不整图栅格化。理由与 0035 同构:栅格纹理做不到 ①逐元素流式揭示(plan30 骨架门)
②缩放任意锐利 ③参与 0016 补间/0025 逐实例动画(研究 §5:「可变纹理三者皆不可」)。

## 2. v1 白名单(超集降级,AR12 不 panic)

| SVG 图元 | 引擎表达 |
|---|---|
| `<rect>`(含 rx) | `FrameRect`(圆角) |
| `<circle>`/`<ellipse>`(圆) | `FrameRect` radius = 半径(rect.wgsl 圆角全开即圆) |
| `<line>`/`<polyline>`(水平/垂直段) | 细 `FrameRect`(无旋转通道 → 斜线不在 v1) |
| `<text>` | 正文 glyph 链(SDF 字) |
| 其余(path/斜线/渐变/滤镜/嵌套 transform) | **整图纹理降级**(plan14 位图链),`FrameStats` 计降级数 |

- 坐标:viewBox → 世界 px 归一;颜色:fill/stroke 十六进制/named 白名单解析,未知色兜中性。
- 流式:逐元素揭示接 plan30 骨架门(容器框先现,元素按序 spawn);半截 `<rect` 由围栏
  raw 抑制天然 hold(svg 在 ``` 围栏内)。
- 确定性:解析纯函数(CR1/R8),native 快照锁定。

## 3. 与位图的分流

PNG/JPEG/GIF 照旧 plan14 纹理/overlay 路;`image/svg+xml` 且**静态**且白名单内 → 矢量原生;
动画 SVG(SMIL/CSS)仍走 DOM overlay(0022,浏览器原生播)。判定顺序:动画嗅探 → 白名单
试解析 → 降级纹理。
