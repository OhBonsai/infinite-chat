# Plan 3 实施总结(plan3-canvas K/L 落地记录)

- 日期:2026-06-14
- 范围:[plan3-canvas.md](./plan3-canvas.md) 的 **K、L 两相位**(引擎底座);M–S 见 TODO
- 状态:**K/L 实现 + 一轮真机调试**;SDF 文字 + 2D 相机 + 空间索引底座就位,观感已可用;本次迭代(字体/quad/性能/可观测)部分为 Rust 改动**待重建验证**(见 §8)
- 提交:`af11613`(K)→ `e49472b`(L)→ `ed89181`(plan3 记录)→ `37aa83c`(改名 infinite-chat)→ `182582e`(SDF 调参);**本次迭代改动尚在工作区未提交**
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

- ~~**SDF 字宽近似**(方形 tile 压窄字)~~ → **本次已修(§8.3)**:改方形 cell quad + advance 推进笔位。
- **MSDF 拐角**:单通道 SDF(从位图)极端 zoom 下拐角圆;待真机评估,触发条件见 0011 §6,本期不做。
- **L5 统一矩形/图片图元**:本期只跑通文字 quad;矩形(面板/代码块底/表格网格)、图片 quad 占位
  留 M/O。
- **可视滚动条**(Plan 2 起的推迟):细化推迟;~~块内 glyph 级裁剪~~ → **本次已加(§8.4)**。
- ~~**@font-face 自带字体打包**(K5)~~ → **本次已接实时路径(§8.1)**:用 LXGW 文楷等宽;离线打包/子集化仍为部署步骤。
- **TinySDF 下沉 wasm**:先 JS 生成(复用浏览器整形),是否下沉后议(0011)。

---

## 7. 待真机验证(本环境无 GPU/浏览器)

2×–8× 缩放锐利度、拐角质量(是否要 MSDF)、2D 裁剪 fps/内存曲线、相机平移缩放手感(≥60fps、
像素对齐)。运行同 Plan 2:`node scripts/serve.mjs` → `cd web && npm run dev` → 页面里滚轮平移、
ctrl+滚轮缩放。

---

## 8. 真机调试迭代(2026-06-14 续,本次)

接初版 K/L 后做了一轮真机调试,渲染观感已可用(markdown 结构/配色/CJK 正常)。改动如下(JS = Vite 热重载即生效;Rust = **需 `wasm-pack build` 重建验证**)。

### 8.1 字体:先接自带 → 最终改回系统字体栈(JS)
- 先试自带 LXGW 文楷等宽(FontFace await + 族切换);**后因"小包体优先"反转为浏览器系统字体栈**:`pretext-bridge` `SANS`/`MONO` = `ui-sans-serif, system-ui, PingFang/雅黑/Noto…`,`main.ts` 去掉 FontFace。Canvas2D `ctx.font` = CSS font-family **逐字形 fallback**,零打包;跨端字形随系统(接受)。
- 固定字形改由**离线 MSDF**("文字当图片",见 §8.8、0011 §3.5)。
- ⚠️ `web/public/fonts/LXGWWenKaiMono-Light.ttf`(28MB)已不被代码引用,但仍在 `public/` → **提交前删除或 gitignore**(§8.9)。

### 8.2 SDF 调参(已提交 182582e)
- `RADIUS` 16→8(≈fontPx/6,细笔画峰值 ~0.69 饱满);shader edge 0.5→**0.46**(浅字深底加粗找回字重);AA 带 **0.6×fwidth**(更锐)。

### 8.3 quad 模型修正(JS,解 §6 字宽近似)
- `pretext-bridge.layout`:advance 矩形 quad → **方形 cell quad**(uv 取整 tile)+ **advance 仅推进笔位/折行** → 窄拉丁字**不再横向压扁**。`TILE_PX`/`SDF_BUFFER` 收为 pretext-bridge 单一来源、glyph-raster 复用。

### 8.4 性能:glyph 级 y 裁剪(Rust,解 §6 推迟项)
- **诊断**:`views` 每 part 一块 → 一条长消息是**一个巨块**,块级裁剪永不命中 → `build_frame` 每帧发整篇 glyph + 逐字 `String::clone` → 主线程分配风暴 → 卡顿、滚动像冻。
- **修复**:`build_frame` 加块内 **glyph 级 y 裁剪**(只发与视口相交的字),每帧发射量降到约一屏。

### 8.5 运行时可观测(Rust+JS,新增,补"无性能日志"缺口)
- core `FrameStats`(发射/总 glyph、可见/总块);atlas 占用/容量/**累计淘汰**;RAF 帧耗时/丢帧。
- `?debug` 每秒一行 `tracing target=perf`:`fps / frame_ms(avg,max) / dropped / glyphs=emit/total / blocks / atlas=used/cap / evict`。**emit≈total = 每帧发整篇;evict 持续涨 = atlas thrash**。详见 [TODO 可观测性](../../TODO.md)。

### 8.6 关键诊断结论(教训)
- **单消息 = 一个巨块** → 块级裁剪不够,必须 glyph 级;后续可按 y 二分 / 拆 md 块为多 view。
- **小字偏软是单通道 SDF 的物理上限**(无 hinting);再要锐 → TILE_PX↑(+Rust 同步+重建+内存↑)或 MSDF(0011 §6)。
- **Rust 改动必须 `wasm-pack build`**,只 `npm run dev` 看不到 → 易误判"没生效 / 滚不动"。
- **缺运行时可观测**导致这次全靠截图反推 → 已补帧统计(§8.5)。

### 8.7 卡口状态(诚实)
- 初版 K/L 提交时 **59 测全绿**(§4)。**本次新增的 Rust 改动(glyph 裁剪 + FrameStats + atlas 计数 + RAF 日志 + web-sys Location feature)尚未在本环境重建/重测**(沙箱无 cargo);字体/quad 为 JS。落地前需 `cargo fmt/clippy/test` + `wasm-pack build` 复跑。

### 8.8 本次的决策与文档(只动文档,代码未跟进的部分已标注)
- **正文字体 = 浏览器系统字体栈**(零打包,小包体优先);固定字形只走离线 MSDF。
- **文字图元:双模 → 三源**([0011 §3.5](../decision/0011-gpu-text-as-sdf-primitive.md)):位图(运行时 Canvas2D,**默认主力**)/ 单通道 SDF(运行时 TinySDF,**opt-in 特效**)/ MSDF(**离线预烘,"文字当图片"**)。实例 `kind` 分支、atlas 按通道分页。**注意:代码现仍是初版"全 SDF",三源/双模为文档定调,落地见 [TODO K′]**。
- **[0012](../decision/0012-debugger-gui-html-vs-egui.md) 调试器决策**:DOM 面板 + 引擎自绘调试几何,**否决 egui**(小包体/解耦);含调试组件矩阵 + 3 通道数据流(stats 快照 / 控制 setter / 点选 hit-test)+ 优先级。
- **TODO 新增**:K′ 三源文字图元、可观测性(帧统计 ✅ / HUD / span)、调试器(P1–P3)、字体子集化。
- 受影响 ADR:演进 0009(BR5→系统字体栈 / 三源)、升级 0007(SDF=特效承重)、新增 0012。

### 8.9 提交准备(commit 指引)
- **改动文件**:
  - JS:`web/src/{pretext-bridge,glyph-raster,main}.ts`(系统字体栈 / quad 方形 cell / 去 FontFace)
  - Rust:`crates/core/src/{app,lib}.rs`(glyph 级裁剪 + `FrameStats`)、`crates/render/src/{atlas,backend}.rs`(atlas 占用/淘汰统计)、`crates/wasm/src/lib.rs`(RAF perf 日志 + `atlas_stats`)、`crates/wasm/Cargo.toml`(web-sys `Location`)
  - 文档:`README`、`TODO`、`spec/decision/{0003,0009,0011}` 改 + 新增 `0012`、`spec/plan/{plan3-canvas,plan3_progress}`
- **未跟踪、勿提交大文件**:`lxgw-wenkai-v1.522/`、`lxgw-wenkai-v1.522.zip`、`web/public/`(28MB ttf)→ 删除或加 `.gitignore`。
- **提交前必跑**(本环境无 cargo,均未跑):`cargo fmt --check` / `cargo clippy -D warnings`(native + wasm32)/ `cargo test`(确认 FrameStats/裁剪不破 59 测)/ `wasm-pack build`(验 web-sys `Location` 编过)/ `cd web && tsc`。
- 建议拆 3 个 commit:① JS(字体/quad)② Rust(glyph 裁剪 + 可观测)③ docs(0012 + 三源 + plan3)。

---

## 9. 下一步(plan3 M–S / TODO)

- **M**:块装饰(面板/代码块底/表格网格,用 L5 的矩形 quad)、词级折行。
- **N**:富 shader 特效(发光/描边/溶解,直接建在 SDF 片元上,0007)。
- **O**:embed(图片→mermaid→卡片,0007 三层,用图片 quad)。
- **P/Q/R/S**:标签层(0006)、input/选区/hit-test(含可点超链接)、a11y DOM 镜像 + WebGL2/Canvas2D 降级、打包。
