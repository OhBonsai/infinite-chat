# Plan 3 实施总结(plan3-canvas K/L 落地记录)

- 日期:2026-06-14
- 范围:[plan3-canvas.md](./plan3-canvas.md) 的 **K、L 两相位**(引擎底座);M–S 见 TODO
- 状态:**K/L 全部实现、过卡口、各自提交**;SDF 文字 + 2D 相机 + 空间索引底座就位,像素/手感待真机
- 提交:`af11613`(K)→ `e49472b`(L)
- 设计源:[决策 0011](../decision/0011-gpu-text-as-sdf-primitive.md)(SDF 图元 / 数据结构层)
- 配套:[plan2_progress](./plan2_progress.md)、[../architecture.md](../architecture.md)

---

## 1. 一句话

把 Plan 2 的"一维滚动位图文字列"升级成 **SDF 文字图元 + 2D 相机 + CPU 空间索引的无边画布
底座**:文字换距离场(无级缩放清晰、可叠 shader 特效),渲染搬到相机 + 空间索引。`core` 大脑
逻辑不变,接口契约(0001 §2.2)不动。新增 **59 个 native 测试**(Plan 2 是 46)。

---

## 2. 交付物(相对 Plan 2 的增量)

```
crates/render/src/
├── atlas.rs          # 位图单页 → SdfAtlas:R8 多页纹理数组 + TileAllocator(LRU,纯 CPU 可测)
├── scene.rs          # GpuInstance + layer(atlas 页);实例由 sink 按可见集构建
├── backend.rs        # atlas alloc/upload + draw(instances, camera);D2Array 绑定
├── effects.rs        # Globals + cam_pan/cam_zoom
└── shaders/glyph.wgsl# 距离场采样(smoothstep+fwidth)+ 世界→相机→clip 变换
crates/core/src/
├── camera.rs   (新)  # Camera2D:pan/zoom/world↔screen/visible_rect(纯数学,native 测)
├── spatial.rs  (新)  # SpatialGrid:均匀网格 broad-phase(native 测)
├── frame.rs          # FrameData + cam_pan/cam_zoom;FrameGlyph.pos = 世界坐标
└── app.rs            # scroll → 相机;build_frame:空间索引查可见 + 出世界坐标 glyph
crates/wasm/src/
├── glyph_bridge.rs   # rasterize_sdf → SDF tile
└── lib.rs            # draw(camera);ctrl-滚轮缩放 / 滚轮平移
web/src/glyph-raster.ts # TinySDF(Mapbox MIT):fillText→alpha→EDT→R8 SDF
```

---

## 3. 各相位落地 + 验证

| 相位 | 内容 | 关键实现 | 验证 |
|---|---|---|---|
| **K** 位图→SDF | 无级缩放清晰文字图元 | `SdfAtlas`(R8 多页数组)+ `TileAllocator`(两级稀疏:key→定长 tile 槽→页池,O(1) 分配/LRU/钉住,0011 §3.3③);距离场 shader;web TinySDF | atlas alloc/evict/pin/开页 4 测 + naga SDF shader |
| **L** 相机+空间索引 | 无边画布:平移缩放 + 2D 裁剪 | `Camera2D`(pan/zoom-at-point);`SpatialGrid`(broad)+ AABB(narrow);`build_frame` 出世界坐标;相机 uniform + shader 变换;ctrl-滚轮缩放 | camera 4 测 + spatial 4 测 + app 相机平移裁剪 + 块冻结回归 |

---

## 4. 卡口结果(全绿)

```
cargo fmt --all --check                                    ✓
cargo clippy --workspace --all-targets -- -D warnings      ✓ (native)
cargo clippy -p ...-wasm --target wasm32 -- -D warnings     ✓
cargo test --workspace                                      ✓ 59 native 测
cargo build -p ...-wasm --target wasm32-unknown-unknown     ✓
web tsc                                                      ✓
```

---

## 5. 设计落地对照(0011)

| 0011 条目 | 落点 |
|---|---|
| §3.3③ 两级稀疏:key→定长 tile→页池 | `TileAllocator` + `SdfAtlas`(R8 多页) |
| §3.3② CPU 树 / GPU 扁平,两消费者 | `SpatialGrid`(CPU 裁剪)+ `GpuInstance`(GPU 实例);不互掺 |
| §3 SDF 图元、shader 读距离场 | `glyph.wgsl` smoothstep+fwidth;保留 `spawn_time` 淡入 |
| §3.2 base 位置权威 | layout 位置为世界真值;相机/动效是其上变换,不回写 |
| §1 无边画布(相机平移缩放) | `Camera2D` + ctrl-滚轮缩放 |

---

## 6. 有据偏差 / 推迟项(均在代码/commit 标注)

- **SDF 字宽近似**:固定方形 tile,窄字按比例 quad 采样整 tile → 轻微水平压缩;精确 per-glyph
  tile bbox 留后续。
- **MSDF 拐角**:单通道 SDF(从位图)极端 zoom 下拐角圆;待真机评估,触发条件见 0011 §6,本期不做。
- **L5 统一矩形/图片图元**:本期只跑通文字 quad;矩形(面板/代码块底/表格网格)、图片 quad 占位
  留 M/O。
- **可视滚动条**(Plan 2 起的推迟)+ **块内 glyph 级裁剪**:块级裁剪已平坦,细化推迟。
- **@font-face 自带字体打包**(K5):光栅仍用 Canvas 系统字体出 SDF;打包字体是部署步骤,待接。
- **TinySDF 下沉 wasm**:先 JS 生成(复用浏览器整形),是否下沉后议(0011)。

---

## 7. 待真机验证(本环境无 GPU/浏览器)

2×–8× 缩放锐利度、拐角质量(是否要 MSDF)、2D 裁剪 fps/内存曲线、相机平移缩放手感(≥60fps、
像素对齐)。运行同 Plan 2:`node scripts/serve.mjs` → `cd web && npm run dev` → 页面里滚轮平移、
ctrl+滚轮缩放。

---

## 8. 下一步(plan3 M–S / TODO)

- **M**:块装饰(面板/代码块底/表格网格,用 L5 的矩形 quad)、词级折行。
- **N**:富 shader 特效(发光/描边/溶解,直接建在 SDF 片元上,0007)。
- **O**:embed(图片→mermaid→卡片,0007 三层,用图片 quad)。
- **P/Q/R/S**:标签层(0006)、input/选区/hit-test(含可点超链接)、a11y DOM 镜像 + WebGL2/Canvas2D 降级、打包。
