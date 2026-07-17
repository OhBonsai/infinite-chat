# 决策记录 0042:引擎 Tile 网格布局模式(Metro 式组件墙)

- 日期: 2026-07-17
- 状态: 已采纳
- 前置: 0018(SDF panel 图元)· 0023(boxlayout 单列 Taffy 盒树)· 0037(声明式 part 装饰)· 0039(crate 分层)· 0038(scroll follow FSM)
- 范围: 给引擎新增**第二种块布局语法**——4 列基网格的 tile 墙(异构 span、边缘严格对齐、SDF chrome),与单列 chat 布局并存、互不触碰。

## 1. 背景 / 问题

引擎至今只有**一种**块布局:`boxlayout::layout_chat`(0023)——角色左右分栏 + 竖直单列堆叠(chat 语义)。Plan 44
需要把「Agent 会话卡全组件」「Markdown 全类型」各排成一面 **Metro/Grafana 式 tile 墙**(异构尺寸、边缘像栅格
一样齐、默认安静指哪动哪),用于 `/components/` 总览页与首页 S3/S4 幕的局部框取。

单列布局做不到:它把每 view 摆在**居中内容列的左缘**(`origin.x` 恒 = 列左),竖直堆叠。tile 墙要的是**二维网格
摆位**(每块落在某 (col,row) 网格格),块宽 = span 决定而非恒等列宽。硬塞进单列会破坏 chat 语义且无法对齐。

约束:① tile 必须是**真渲染 SDF**(容器 = 0018 panel,标题 = glyph,零 DOM 内容层,缩放锐利);② 边缘对齐是
**硬断言**(逐 tile 四边落网格线);③ 默认安静(静态成品态,hover 才播);④ **恒等硬门**——非 tile 场景
(chat / 首页现有幕 / 一切既有页)代码路径与 golden **逐字节不变**(承 plan36/37/42 惯例)。

## 2. 决策

**新增 `LayoutMode` 旁路,不改单列主路径。**

### 2.1 数据契约(布局=数据,manifest 驱动)

`TileSpec` 落 `primitives`(serde,与 `FramePanel`/`MotionTokens` 同层),换墙/改 span **零代码**:

```rust
pub struct TileSpec {
    pub cols: u32,            // 基网格列数(设计表 = 4)
    pub gap: f32,            // 统一 gap(格间 + 行间,单一 token)
    pub row_h: f32,          // 基行高(1 格的像素高;设备 px)
    pub pad: f32,            // tile 内衬(panel 边 → 内容)
    pub title_h: f32,        // 标题条高(panel header)
    pub tiles: Vec<TilePlacement>,
}
pub struct TilePlacement {
    pub id: String,          // = store 里该 tile 内容的 messageID(块归属靠它反查)
    pub span: [u32; 2],      // (span_w 列, span_h 行);异构只在此,不在偏移
    pub title: String,       // 标题条 glyph 文案(Mono eyebrow 风)
}
```

**固定基网格 + 固定行高**(Windows Metro 精确语义,非 masonry 乱流):
- 列宽 `col_w = (W - (cols-1)*gap) / cols`(`W` = 引擎 `max_width`)。
- tile 像素矩形:`x = col*(col_w+gap)`,`y = row*(row_h+gap)`,
  `w = span_w*col_w + (span_w-1)*gap`,`h = span_h*row_h + (span_h-1)*gap`。
- ⇒ **每 tile 四边恒落网格线**(`x ∈ {k*(col_w+gap)}`、`y ∈ {k*(row_h+gap)}`),对齐是**摆位的必然**,非事后校正。

### 2.2 摆位算法(确定性 skyline first-fit)

按 manifest 顺序处理 tile;维护每列的 `skyline[col]`(已填到的行号)。对 span=(w,h) 的 tile:
找**最左**的连续 `w` 列、其 `skyline` 取这 w 列的 **max**(顶对齐到该行)→ 放此 (col,row);
把这 w 列 `skyline` 更新为 `row + h`。全程整数格坐标 → 无偏移、可 native 精确断言、可重放。

「行内最高 tile 撑起、同行顶对齐」由**固定 `row_h` + span_h 撑多行**实现:标准内容占 1 行高,超高内容
(diff/表格/代码块)给 span_h=2。**内容不足格高 → 底部留白(不拉伸内容)**;超格 = 换 span 或裁内容(T0 容量四问),**不缩字号**。

### 2.3 内容摆位 + chrome

- 每 tile `id` = store 里该内容块的 `messageID`。tile 布局器把该块的 `origin` 设为 **tile 内容区左上**
  (`tile.x+pad`, `tile.y+title_h+pad`),`box_w` = tile 内宽(`w-2*pad`)。其余 `build_frame` 逐字不变
  (glyph world = origin + 块内相对 placed;装饰吃传入的 `box_w`)。
- **per-tile 折行宽**:内容按 tile 内宽折行(非全局列宽)。span 固定 → tile 内宽**先验已知** → `ensure_layouts`
  在 tile 模式吃**每 view 折行宽覆盖**(唯一侵入单列外的管线点;单列模式该覆盖为 `None` → 行为恒等)。
- **chrome** = 每 tile 一个 `FramePanel`(承 user 气泡 emitter:fill + 1px border + radius,可选 AO;
  `header_ratio`=title_h/h 出标题条),标题 glyph 渲在标题条内。零 DOM、零截图。

### 2.4 静态语义(默认安静 + hover 才播)

tile 模式 = 静态页:强制 `follow = Released`(相机冻结,不锚底)、内容**全揭示**(`revealed_height = content_height`,
无流式前沿)。`max_pan_y` 从**网格整体外接盒**算(超一屏可滚,否则 0)。broad-phase:同 y 带返回整行 tile
(可见集略超,~30 块量级无感;不改 HeightIndex 结构,风险 §6 已录)。**hover 才播** = 单 tile 单 part 替换术
(新 part id 重播流式揭示,markdown 页已验证;同时 ≤1)→ 播完 settle 回成品态(AR3 断言把关)。

### 2.5 与单列模式的边界(恒等硬门)

`LayoutMode { Column(默认), Tile(TileSpec) }`。`Column` 分支 = 现 `layout_chat` **零改动**;`build_frame` 里
仅在 `Tile` 分支替换 `boxpos` 来源 + 应用 per-view 折行宽/关 follow/全揭示。**非 tile 场景:代码路径与全部
既有 insta golden 逐字节不变**(新增 tile golden 是独立 snapshot)。坏 manifest(负 span/空/cols=0)→ 整拒
退 `Column`(AR12),不 panic。

## 3. 备选与否决理由

- **masonry(瀑布流)**:列内自由堆高、边缘不齐 → 违「对齐硬断言」。否决。
- **变行高(行高随内容)**:与竖直 span 冲突(哪些块算「同行」歧义),对齐断言退化为浮点容差。否决,取**固定 row_h + span_h 撑多行**(Metro 精确整数格,断言=精确等)。
- **DOM 网格(CSS grid 套 canvas 片)**:违「tile 即 SDF、零 DOM 内容层」。否决。
- **复用 `layout_chat` 塞 flex-wrap**:Taffy flex-wrap 不给「span + 严格网格」,且会污染单列主路径恒等。否决,tile 走独立布局器模块。

## 4. 影响

**正**:引擎多一种可复用布局语法(组件页 + 首页幕 + 未来 dashboard 皆可用);布局=数据(manifest 换墙零代码);
tile 全 SDF(缩放锐利、指哪动哪);对齐可 native 精确断言(整数格)。

**负**:`ensure_layouts` 加一处 per-view 折行宽分支(tile 专用,单列 `None` 恒等);HeightIndex 对 tile 非 y-单调
→ broad-phase 略过选(30 块无感,大墙需 2D 索引,记遗留);tile 场景禁流式锚底/选区语义弱化(静态页可接受,记遗留)。

## 5. 实现入口

- `crates/primitives/src/tile.rs`(新):`TileSpec` / `TilePlacement`(serde)。
- `crates/core/src/tilelayout.rs`(新):skyline 摆位器 → 每 tile `(id, origin, w, h)` + 网格线集合(断言用)。
- `crates/core/src/app.rs`:`LayoutMode` 字段 + `set_tile_spec()`;`build_frame` Tile 分支(boxpos 来源 + per-view 折行宽 + follow=Released + 全揭示);`ensure_layouts` per-view 折行宽覆盖。
- `crates/core/src/app.rs`:tile chrome panel emitter(承 user 气泡 `app.rs:3550` 路)。
- `crates/wasm/src/lib.rs`:`set_tile_spec(json)` 薄导出。
- 断言:`crates/core/tests/tile_layout.rs`(对齐/行高撑起/坏 manifest 整拒 + insta tile golden);既有 golden 全集恒等(硬门)。
