# Plan 12 — 数学(LaTeX)一等 SDF 公民:终态设计

- 日期:2026-06-18(终态设计版,不分期;按最终目标直接落)
- 前置:[0013 §8](../decision/0013-math-latex-rendering.md)(翻转 B)、调研 [research/math-latex-sdf-rendering](../research/math-latex-sdf-rendering.md);契约 [0001](../decision/0001-content-layout-render-contract.md)、[0011](../decision/0011-gpu-text-as-sdf-primitive.md)、[0014 表格 side-channel 先例](../decision/0014-table-two-pass-layout.md)、[0015 atlas/字形源](../decision/0015-glyph-source-fallback.md)、[0016 morph](../decision/0016-streaming-morph-render-model.md)、[0019 reveal](../decision/0019-reveal-gating-and-choreography.md)、[0025/Plan10 动画](../decision/0025-sdf-node-animation-system.md)
- RaTeX API 已读本地源核实(见 §2);本仓不收 RaTeX 源(`.gitignore` 排除 `/RaTeX/`),走 Cargo git 依赖钉 rev `1e5ae7d`(`ratex-parser`/`ratex-layout`/`ratex-types` 已入 workspace + core)。
- **进度(2026-06-19)**:spike 通过;`core/math.rs`(`layout_math`+`math_to_frame`)、`StyleRole` 数学角色、`build_frame` 显示数学 data 层集成(`BlockCache.math` 缓存 + RaTeX SDF 字形替 raw TeX)、web 字体脚手架(`fontForRole`/`math-fonts.ts`/copy 脚本)**均落地 + native/wasm/tsc 全绿**;视觉栅化与行内/Path/MSDF/动画待人工 GPU 实跑。详见 [plan12_progress.md](plan12_progress.md)。

---

## 0.5 执行前置 / 交接须知

给接手者:本仓**不入库**字体源与烘焙产物(见 `.gitignore`),RaTeX 走 git 依赖,故执行前需:
1. **RaTeX**:无需手动 clone——`cargo build` 按 workspace 的 git 依赖(rev `1e5ae7d`)自动拉取(`ratex-parser/layout/types`)。要读源可自行 clone 到 `/RaTeX`(已 gitignore)。
2. **字体源**:把 LXGW(OFL)放 `lxgw-wenkai-v1.522/`(仓库根);KaTeX 字体 TTF、英文拉丁字体另备(数学相位用)。均不入库。
3. **烘焙 MSDF(构建前必跑)**:`npm i -g msdf-bmfont-xml` 后跑 `scripts/bake-msdf.mjs`(见 §6.6)生成 `web/public/fonts/*-msdf.*` / `web/src/assets/*-msdf.*`。**这些是 build 产物、不在仓库**——不跑则运行时无字体 atlas。建议固化进 `web/package.json` 的 `bake:*` script,并在 `build` 前置(或 CI step)。
4. **构建顺序**:bake 字体 → `wasm-pack build`(crates/wasm)→ `vite build`。Rust 改动本地 `cargo test`;无 cargo 环境(如 CI 镜像)需装 Rust + wasm-pack。
5. **已知修复**:`scripts/bake-msdf.mjs` 的 rename 守卫 bug 已修(重烘会正确覆盖旧 json,不再 json/png 错配)。

## 1. 终态目标(一句话)

LaTeX 行内 `$…$` 与显示 `$$…$$` **每个符号都是 SDF atlas 实例**(和正文同级:缩放锐利、可逐字上色、走 0025 逐字动画/跨式 morph),由 **RaTeX 在 core 排版**(给绝对坐标),**复用现有 atlas/glyph/rect 管线**,确定性 + 按 TeX hash 缓存,极端/出错 LaTeX **直接退回纯文本**。**无 opaque 纹理、无 MathJax/SVG 依赖。**

终态判据:`$$\sqrt{\frac{a}{b}}=\sum_{i=0}^{n} x_i$$` 正确排版,放大边缘锐利,慢放可见逐符号"写出",两个相邻公式切换有符号级 morph;`质能 $E=mc^2$ 守恒` 行内与文字基线对齐、随段折行。

---

## 2. RaTeX 接口(已核实,设计依据)

```rust
let nodes = ratex_parser::parse(latex)?;                          // &str → Vec<ParseNode>
let lbox  = ratex_layout::layout(&nodes, &LayoutOptions{ style, color, .. });
let dl: ratex_types::DisplayList = ratex_layout::to_display_list(&lbox);
```
`DisplayList { items: Vec<DisplayItem>, width, height, depth }`(f64,**em**;`total_height = height+depth`)。
`DisplayItem`(serde `tag="type"`):
- `GlyphPath { x, y, scale, font: String, char_code: u32, color }`
- `Line { x, y, width, thickness, color, dashed }`
- `Rect { x, y, width, height, color }`
- `Path { x, y, commands: Vec<PathCommand>, fill, color }`

坐标系:**x 右增、y 下增(屏幕系)、原点=盒左上、baseline 在 y=height**。`char_code` 已是该字体 cmap 码点(`math_alpha` 已解析)→ `char::from_u32` + 对应 KaTeX 字体即正确字形。`font` 形如 `"Main-Regular"`/`"Math-Italic"`/`"Size1-Regular"`…(`FontId::as_str`)。`LayoutOptions.style` = `MathStyle::Display`(`$$`)或 `Text`(`$`)。

单位换算:**px = em × MATH_PX**(`MATH_PX` = 数学基准像素,取与正文 `FONT_SIZE` 协调,如 = FONT_SIZE)。

---

## 3. 数据流(终态,跨 content→layout→render)

数学排版在 **core(RaTeX)**;行折断/行内定位仍在 **JS**。二者经**侧信道**相遇——**完全照搬表格的 `tables`/`table_panels` 模式**(0014):core 先算好数学盒尺寸,交 JS 在行内预留;JS 回报盒的落点;core 再把数学字形摆到该落点。

```
parse: pulldown ENABLE_MATH(已开)→ jcode InlineMath/MathDisplay 携原文 TeX
  │  (content.rs:现当 Code 显示 → 改)
  ▼
content.rs:
  · 显示 $$ → 一个 math 块(MathBlock,携 tex)
  · 行内 $  → 在 span 流里发**一个零墨锚点 grapheme**(role=Math,占位)+ 记录 MathRun{ anchor_idx, tex }
  ▼
BlockCache 构建(core,带缓存):
  · 对每个 math run/块:math::layout_tex(tex, display) → MathLayout(见 §4),按 tex hash 缓存
  · 收集 math 盒尺寸 → Vec<MathBox{ anchor_idx, width, height, depth }>(px)
  · 调 LayoutEngine::layout(spans, tables, **maths**, max_width):JS 把每个 anchor_idx 处当**原子盒**
    (advance=width,行高吃 height/depth,盒内不断行),回报该 grapheme 的 placed pos/size
  · cache 存:MathLayout 列表 + 各自 placed origin
  ▼
build_frame(core):每个 math run,origin = placed 锚点;遍历 MathLayout.items:
  · Glyph → FrameGlyph(pos = origin + (x,y)×scale 已折 px;role = 该 KaTeX 字体的 math 角色;cluster = char)
  · Line/Rect → FrameRect
  · Path → FramePathTile(见 §6,栅成 SDF tile 当 atlas 字形)
  ▼
render(已有管线):FrameGlyph→glyph pipeline(SDF atlas);FrameRect→rect pipeline;
  math 字形用 KaTeX 字体栅化(JS rasterize_fn 按 math 角色选字体);0025 anim 逐字。
```

**关键**:数学字形 `pos` 由 core 写(RaTeX 绝对坐标),JS 只负责(a)行内盒预留/回报落点、(b)栅化字形外观。**无新 GPU 管线**(Path 也走 atlas)。

---

## 4. core 数学模块(`crates/core/src/math.rs`)

```rust
pub struct MathLayout {
    pub width: f32, pub height: f32, pub depth: f32,   // px(已 × MATH_PX)
    pub items: Vec<MathItem>,                          // 块内相对 px(origin 由上层加)
}
pub enum MathItem {
    Glyph { dx: f32, dy: f32, size: f32, font: MathFont, ch: char, color: [f32;4] },
    Rule  { dx: f32, dy: f32, w: f32, thickness: f32, color: [f32;4] },   // Line
    Fill  { dx: f32, dy: f32, w: f32, h: f32, color: [f32;4] },           // Rect
    Path  { dx: f32, dy: f32, path_key: u64, color: [f32;4] },            // 矢量轮廓 → SDF tile
}
pub enum MathFont { MainRegular, MainBold, MainItalic, MainBoldItalic, MathItalic,
                    MathBoldItalic, Ams, Caligraphic, FrakturRegular, FrakturBold,
                    SansRegular, SansBold, SansItalic, Script, Typewriter,
                    Size1, Size2, Size3, Size4 }  // == RaTeX FontId(去 CJK/Emoji)

pub fn layout_tex(tex: &str, display: bool) -> MathLayout;  // parse→layout→to_display_list→映射+×MATH_PX
```
映射:`DisplayItem::GlyphPath{font,char_code,scale,x,y}` → `MathItem::Glyph{ font: MathFont::parse(&font), ch: char::from_u32(char_code), size: scale×MATH_PX, dx: x×MATH_PX, dy: y×MATH_PX }`;`Line`→`Rule`;`Rect`→`Fill`;`Path`→`Path{ path_key = hash(commands) }`(commands 另存 `HashMap<u64, Vec<PathCommand>>` 供栅化端取)。坐标系一致(y 下、baseline=height),直接用。

缓存:`HashMap<u64 /*tex hash + display + MATH_PX*/, MathLayout>`(确定性,见调研);路径命令表同 key 级缓存。

`MATH_PX` 常量(core),与 web `FONT_SIZE` 对齐。

---

## 5. 行内/显示接入(侧信道,照搬表格)

**seam.rs**:`LayoutEngine::layout` 增第 3 个侧信道入参(对称 `tables`):
```rust
fn layout(&mut self, spans: &[StyledSpan], tables: &[TableRegion],
          maths: &[MathBox], max_width: f32) -> LayoutResult;
pub struct MathBox { pub anchor: usize, pub width: f32, pub height: f32, pub depth: f32 } // grapheme idx + px
```
- **JS 实现**(layout-bridge):遇到 `anchor` 处的 grapheme,不按字体测量,而按 `MathBox` 当**原子内联盒**:占 advance=width、对基线下沉 depth、行高吃 height+depth、盒内不可断、可整体换行到下一行。回报该 grapheme 的 `PlacedGlyph{ pos, size=(width, height+depth) }`。
- **显示 `$$`**:整块即一个 math run,`anchor`=块首占位;盒 = MathLayout 尺寸;JS 左对齐(或居中)返回 origin。块 `block_height = height+depth`。
- **MonospaceLayout(native stub)**:同样接 `maths`,按盒尺寸推进(测试用)。
- app build_frame:用回报的 `placed[anchor].pos` 作 origin,加 MathLayout.items 的相对坐标 → world。

> 为何不把数学塞进 JS layout:RaTeX 是 Rust(core),且数学排版远超 JS 文本测量能力(见"复杂度启示");core 算盒、JS 只摆盒,职责干净,且复用表格已验证的侧信道形状。

---

## 6. 栅化 + 字体 + Path(终态:预烘多字体 MSDF,英文 + KaTeX 嵌入 lib)

复用现成 MSDF 离线烘焙通路(0015):BMFont(`*.json` 度量 + `*.png` atlas 页)→ `msdf.ts::loadMsdf` → `ChatCanvas::load_msdf` → `MsdfAtlas`(D2Array 页)。`resolve()` Auto:码点命中烘集 → MSDF,否则退 TinySDF。**改造要点如下。**

**6.1 MSDF 子系统:单字体 → 多字体(必做架构改)**
现 `msdf::MsdfFont` / `load_msdf` 单字体(lookup 仅按 codepoint)。KaTeX 19 face 同码点不同形 → 必须 **`(font_id, codepoint)` 双键**。改:`load_msdf` 接 `{ font: String, ... }` 注册一只字体到共享 `MsdfAtlas`(多只字体共用页,id 命名空间化);`resolve()` 按 `(font, cp)` 查。英文/数学/CJK 都走这套。

**6.2 三套烘集(你离线烘,产 BMFont)**
- **English/ASCII**(新,小):一只讲究的拉丁文本字体烘 ASCII+常用标点 → **嵌入 lib**(替代现在用 LXGW 等宽拉丁渲英文)。
- **KaTeX 数学**(新,小):19 face(Main/Math/AMS/Size1–4/Caligraphic/Fraktur/SansSerif/Script/Typewriter)各烘其用到的码点 → **嵌入 lib**(KaTeX 字形集小)。
- **CJK LXGW**(已存在,大 ~10MB):保持**外部 fetch**(`/fonts/lxgw-msdf.*`),不嵌入。

**6.3 嵌入 lib(英文 + KaTeX 小烘集,免网络)**
小烘集体积小(各约一页 512–1024² + 度量),**打进包**而非 fetch:
- 首选(简单,复用现解码路):**bundle 内联** —— 烘集放 `web/src/assets/`,`msdf.ts` 改 `fetch` 为 Vite `import`(json 直接 import;png 用 `?url`/`?inline` 资产),仍走 `createImageBitmap` 解码 → `load_msdf`。
- 备选(真·自包含 wasm):wasm crate `include_bytes!` json+png,Rust 侧解码(加 `png` 依赖或存预解码 RGBA)→ 内部 `msdf_init`/`upload`,零 JS。
推荐先 bundle 内联(0 新依赖、复用解码);要单文件 wasm 再上 `include_bytes!`。

**6.4 角色 → 字体**
content.rs 追加连续 math `StyleRole`(每 `MathFont` 一个,**末尾不移位**);core 把 RaTeX `font: String`("Main-Regular"…)映成对应 math 角色,并据此选 MSDF 字体键。英文用 ASCII 烘集的字体键。字号在 `FrameGlyph.size`(SDF 缩放无关,一 tile 通用)。

**6.5 Path(根号/大定界符)= 运行时,仍进 atlas**
`MathItem::Path` 尺寸随内容变(不可预烘)→ JS `rasterize_fn` 新源 `kind=PathSdf`:`Path2D` 描命令 → 填充 → 转 SDF → 上传 tile(key=path_key),当"特殊字形"实例渲染(复用 FrameGlyph)。**多数定界符其实走 `Size1–4` 字形(GlyphPath,已在 KaTeX 烘集),仅极大者才是 Path** → 运行时栅化量很小。

> 净效果:英文 + 数学 = **预烘 MSDF、嵌入 lib、(font,cp) 双键、尖角锐利、零运行时栅化**;CJK 仍外部大烘集;仅动态 Path + 生僻字退运行时。一切仍走同一 `MsdfAtlas`/glyph pipeline。

**6.6 烘焙脚本 + 换字体流程(已有 `scripts/bake-msdf.mjs`)**
工具:`msdf-bmfont-xml`(npm,预打包 msdfgen 二进制,免编译;`npm i -g msdf-bmfont-xml`)。脚本产 BMFont(`<name>.json` 度量 + `<name>.*.png` 页 + `<name>.charset.txt`)→ 正是 `msdf.ts`/`load_msdf` 吃的格式。参数:`--font <ttf> --out <dir> --size 40 --range 4 --type msdf --texture 2048,2048 --name <name> --extra <charset.txt>`。字符集 = ASCII + CJK 标点 + GB2312 一级(~3760)+ `--extra`。
- **CJK 正文字体**(本仓主字体):源在 `lxgw-wenkai-v1.522/`(OFL);**选用 `LXGWWenKaiMono-Regular`**(等宽=零布局风险;比 Light 更耐 MSDF 距离场侵蚀、屏幕更锐)。重烘:`node scripts/bake-msdf.mjs --font lxgw-wenkai-v1.522/LXGWWenKaiMono-Regular.ttf --name lxgw-msdf`。
- **English/ASCII 小烘集**(数学相位用):`--font <拉丁字体> --name latin-msdf --extra <仅 ASCII charset>`(裁掉 CJK 段以缩小)。
- **KaTeX 19 face**:逐 face `--font KaTeX_Main-Regular.ttf --name katex-main-regular --extra <该 face 码点>` …;`name` 须与 core 的 `MathFont`/RaTeX `FontId::as_str` 对齐(见 §6.4)。
- **换字体(后续需求)**:只换 `--font` + 重跑脚本;名(`--name`)不变 → 运行时零改动,资源替换即生效。多字体共存靠 §6.1 的 `(font, cp)` 双键 + `load_msdf` 注册多份。建议把常用烘命令写进 `package.json` scripts(如 `bake:cjk`/`bake:latin`/`bake:katex`)固化。

---

## 7. 动画(0025,逐字最终效果)

数学字形 = `FrameGlyph` → `GpuInstance`,带 `anim` profile,直接享 Plan 10/0025:
- **逐符号"写出"公式**:按 DisplayList 顺序/结构给 spawn_time 错峰(reveal 节奏接 0019;math run 作一个 reveal 单元,内部按 item 序释放)。
- **跨式 morph**(Manim `TransformMatchingTex` 的 SDF 版):同 `(font,ch)` 符号在前后两公式间 `mix` 几何(接 0016 morph + Plan 10 §4/§5);身份键 = `(math_run_id, item_seq)` 或 `(font,ch,出现序)`。
- 缩放/着色:逐字 color 来自 RaTeX(`\color`)或主题;放大走 SDF 锐利。

---

## 8. 兜底(退回纯文本)

`ratex_parser::parse` 报错或 RaTeX 不支持 → **直接退回纯文本**:把该 run 的原文 TeX 当普通文字渲染(走现有 `StyleRole`,如 `Code`/`Normal`),不引 MathJax/SVG。content.rs 捕获 parse 错 → 该 run 不发 math 占位、改发普通文字 span。最简、零额外依赖;主路恒 B。

---

## 9. 落地改动清单(按文件,供开发)

1. **Cargo**(已done):workspace + core 加 `ratex-parser/layout/types`(git rev `1e5ae7d`)。
2. **`crates/core/src/math.rs`(新)**:`MathLayout`/`MathItem`/`MathFont`/`layout_tex` + tex-hash 缓存 + path 命令表。`MATH_PX` 常量。
3. **`content.rs`**:`MathDisplay` 块不再当 Code → MathBlock(携 tex);行内 Math run → 零墨锚点 grapheme(新 math 占位角色)+ 记录 `MathRun{anchor,tex,display}`。追加 19 个 `MathFont` 对应的 `StyleRole`(末尾,数值稳定)。
4. **`seam.rs`**:`LayoutEngine::layout` 加 `maths: &[MathBox]`;`MathBox` 结构;`LayoutResult` 不变(math 落点走 placed[anchor])。
5. **`support.rs`(MonospaceLayout)** + **`app.rs`(test 桩)**:接 `maths` 参,按盒推进。
6. **`app.rs`**:BlockCache 构建时对 math run 调 `layout_tex`(缓存)→ 组 `MathBox` → 传 layout;cache 存 MathLayout + origin。`build_frame`:math run → FrameGlyph/FrameRect(+Path tile)。`enter_profile_id` 给 math 角色配进场;reveal 单元含 math。
7. **`frame.rs`**:必要时给 FrameGlyph 标 math(其实 role 已够);Path tile 复用 FrameGlyph(kind=PathSdf)。
8. **`crates/wasm/src/msdf.rs` + `lib.rs`**:MSDF **单字体 → 多字体**(`load_msdf` 接 `font` 名注册到共享 `MsdfAtlas`,lookup 按 `(font, cp)`);`resolve` 按 font 选烘集 + `kind=PathSdf` 源;layout 调用传 `maths`。
9. **离线烘焙(你做)**:产 BMFont 烘集 —— ① English/ASCII(讲究拉丁字体)② KaTeX 19 face;CJK LXGW 已有。工具 `msdf-atlas-gen`。
10. **嵌入 lib**:英文 + KaTeX 小烘集放 `web/src/assets/`,`msdf.ts` 改 `fetch`→Vite `import`(json+png 资产),启动即载;CJK 仍 `/fonts/` fetch。(或 wasm `include_bytes!` 走自包含,见 §6.3。)
11. **`web/src/layout-bridge.ts`**:`layout` 接 `maths` 侧信道(原子盒预留/回报);`fontForRole`/`roleScale` 加 math 角色(仅 TinySDF 兜底时用,主路走预烘 MSDF)。
12. **`web/src/glyph-raster.ts`**:`rasterize` 加 `PathSdf`(Path2D→SDF);英文/数学未命中 MSDF 时的兜底字体。
13. **测试/case**:native `layout_tex` 单测(分式/上下标/根号/∑ 的盒尺寸 + item 数);重放 case `g-math`(`$$`/`$` 混排,慢放看逐字)。

---

## 10. 风险 / 边界
- **沙箱无 cargo**:Claude 不能编 Rust;每步本地 `cargo test` + `wasm-pack build` + GPU 实跑(数学接入尤其 GPU 验)。
- **行内基线盒时序**:core 先算盒(RaTeX)→ JS 预留 → core 摆字,是最复杂处;`$$` 显示数学最简,先打通它验证全链,再开行内(实现顺序自定,设计上同一机制)。
- **Path 栅化**:Path2D→SDF 的质量/缓存;多数定界符走 Size 字形可绕开。
- **体积**:KaTeX 字体 ~300KB–1MiB,懒加载;RaTeX wasm 增量本地量。
- **守界**:不自研 TeX 排版 / 不自研 OT MATH 表(用 RaTeX 的 CM 路线);字体无关/MATH 表驱动 = 后续(评估 Typst-math)。

> 终态 = RaTeX 在 core 产 `DisplayList` → `MathLayout`(px,相对)→ 经 MathBox 侧信道与 JS 行排版相遇 → `build_frame` 落 FrameGlyph/FrameRect/Path-tile → 复用 SDF atlas/glyph/rect 管线 → 0025 逐字动画。数学与正文同级,一切皆 SDF,难的排版交给 RaTeX 的 ~2.8 万行,我们只做最强的 SDF 与动画那层。
