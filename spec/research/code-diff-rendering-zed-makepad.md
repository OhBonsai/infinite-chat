# 研究:code diff 渲染 —— Zed 全链路 + makepad 编辑器手法 → 本引擎 diff 卡片升级

- 日期:2026-07-13
- 类型:research(源码级调研 + 落地建议)
- 触发:本引擎 diff 卡片观感差(朴素逐行分类 + 整宽红绿带,无行号/词级/hunk 结构)。
- 源码:Zed = 本机 `~/w/agentscode/zed`(@ main,行号以本机为准);makepad = 本机 `~/w/agentscode/makepad`。
- 本引擎现状:`crates/core/src/partspecific.rs:422-495`(diff_parse_lines 首字符分类 + render_diff)+ `app.rs:258-310`(整宽 diff 行底色带 flush_diff_band)+ `theme.rs` `diff_add_bg/diff_del_bg` 两色。

---

## 0. TL;DR(对本引擎最重要的 6 条)

1. **Zed 的 diff 是"数据层三件套"**:imara-diff(Histogram)全量后台重算 + SumTree\<Hunk\> 按 Anchor 存(编辑零修补)+ 新旧 hunk 树对比出最小失效范围。我们是聊天卡片(内容 settled 后不变),**不需要 anchor/增量**,但 hunk 化数据模型该抄。
2. **删除行是"织入行流"不是另画一块**:Zed 现行方案把 base text 切片作为 `DiffTransform::DeletedHunk` 注入 MultiBuffer 坐标系,删除行走与普通行**完全相同**的 layout/paint 流水线。对我们:diff 行就是普通 run,身份(`diff_status`)随行携带,装饰按身份画——现架构已同构,补的是**身份的粒度**(行级→hunk 级 + 词级)。
3. **观感 = 三层叠加**:行底(hunk 色 12-16% 透明度)+ gutter 色条(每 hunk 一个 quad,`宽 = floor(0.275 × line_height)`,添加/修改/删除三色 0.7 透明度)+ **词级高亮**(`version_control_word_added/deleted`,叠在行底上)。行号也随 diff 状态换色。
4. **词级 diff 有严格限流**:只对"行数相等的 Modified hunk 且 ≤5 行"算(`MAX_WORD_DIFF_LINE_COUNT=5`),按语言分词;渲染侧先视口过滤再单调 cursor 批量转换坐标。防 O(n²) 词对齐失控。
5. **makepad 给了两件 SDF 神器**:(a) 选区/高亮的 **gloop(SDF smooth-union)三行滑动窗口**——逐行 quad 但相邻行平滑融合成连续圆角块,正是 GitHub 式 diff 高亮的高级版;(b) **折叠动画 = 每行 scale 每帧 ×0.9 指数逼近 + SDF 字形随 scale 缩不糊**——上下文行折叠/展开的现成范式。
6. **落地不换架构**:全部映射到现有 0018 panel(行底/gutter 条/词级高亮 = 面板参数)+ 0016 morph(展开/折叠补间)+ 0020 节点(hunk 作节点)+ 0033 渲染契约(partspecific 只产语义)。

---

## 1. Zed 数据层(`crates/buffer_diff/src/buffer_diff.rs`)

### 1.1 Hunk 模型

```rust
pub struct DiffHunk {
    pub range: Range<Point>,                    // 查询时现算,不入库
    pub buffer_range: Range<Anchor>,            // buffer 侧:Anchor 随编辑漂移
    pub diff_base_byte_range: Range<usize>,     // base 侧:裸 byte(base 不可变)
    pub secondary_status: DiffHunkSecondaryStatus,
    pub buffer_word_diffs: Vec<Range<Anchor>>,  // 词级:buffer 侧
    pub base_word_diffs: Vec<Range<usize>>,     // 词级:base 侧(相对 hunk 起点)
}
```

- kind 不存储,由几何推导:buffer 区间空 = Deleted;base 区间空 = Added;否则 Modified。
- 存储 `SumTree<InternalDiffHunk>`,Summary 含 buffer_range/base_range/added_rows/removed_rows;同一棵树可按 buffer Anchor **或** base byte offset 两种坐标 seek(两个 SeekTarget 实现),区间查询 O(log n) 剪枝。
- base text 是完整只读 `language::Buffer`(**删除文本可语法高亮**);删除文本按 `diff_base_byte_range` 从 base rope 切片取。

### 1.2 计算

- **imara-diff `Algorithm::Histogram`**,`lines_with_terminator` 行粒度;不用 git2/similar。自定义 `HunkSink` 每个 `process_change(before, after)` 直接产 hunk;**无 context 行概念**(上下文/展开由 UI 层管)。
- 后台 executor 全量重算;增量性靠三招:version 向量没变直接复用旧树(SumTree clone = Arc 廉价)、新旧 hunk 树双 cursor 归并算最小 `changed_range` 只失效那段、返回 Task drop 即取消。
- **词级 diff**(`HunkSink::process_change` 内):仅当 `base_line_count == buffer_line_count && ≤ MAX_WORD_DIFF_LINE_COUNT(5)` 且 per-language `word_diff_enabled` 才调 `language::word_diff_ranges`(接受 language_scope 按语言分词)。

## 2. Zed 显示层(`crates/editor/src/{element,git}.rs`、`multi_buffer.rs`)

### 2.1 删除行 = 织入行流(现行方案,替代旧 block 注入)

```rust
enum DiffTransform {                      // multi_buffer.rs ≈L716
    BufferContent { summary, inserted_hunk_info },
    DeletedHunk { summary, buffer_id, hunk_info, base_text_byte_range, has_trailing_newline },
}
```

展开 hunk 时把 base text 切片注入 MultiBuffer 坐标空间,删除行成为普通显示行;每行 `RowInfo.diff_status: Option<DiffHunkStatus>` 携带身份,**绘制层无特殊分支**——这是可扩展性的主要来源。折叠(默认)时 deleted hunk 不占行,只在 gutter 画一个强圆角小色块(`Corners::all(1.0 × line_height)`)。

### 2.2 三层视觉(可直接抄的数值)

| 层 | 做法 | 数值/键 |
|---|---|---|
| 行底 | 连续高亮行段合并成一个 quad;filled(未暂存)/hollow(已暂存,带边框)两种 | 不透明度 `light 0.16 / dark 0.12`;`editor_diff_hunk_{added,deleted}_background` + `_hollow_{background,border}` |
| gutter 色条 | **每 hunk 一个 quad**(非每行);可点击(hitbox 切换展开) | 宽 `floor(0.275 × line_height)`(element.rs:5291);`version_control_{added,modified,deleted}`.opacity(0.7);deleted 折叠 marker 宽 `floor(0.35 × line_height)`、圆角 `1.0 × line_height` |
| 词级高亮 | 叠在行底上的字符区间背景 | `version_control_word_{added,deleted}`;Modified hunk 的新行用 word_added、删除行用 word_deleted |

另:**行号随 diff 状态换色**(`LineNumberStyle::DiffAdded/DiffDeleted` → version_control 色);modified hunk 展开后增删段共用 modified 色(分色实验被 revert,PR #25020)。

### 2.3 交互与性能

- hunk 控件(Stage/Restore/上下跳)只为 **hovered** hunk 构建元素,浮在 hunk 首行右上,其余 hunk 零元素成本。
- 词级高亮三层节流:视口过滤 → 单调前进 `DisplayPointConverter` 批量坐标转换(均摊近常数)→ 计算侧 ≤5 行上限。
- 一切从 `display_diff_hunks_for_rows(可见行)` 出发,视口外 hunk 不参与 layout/paint。

## 3. makepad 编辑器(`~/w/agentscode/makepad/code_editor/`,实现约 7.4k 行)

与本引擎同族(GPU + SDF + 等宽网格坐标),四个手法直接可迁:

1. **gloop 选区/高亮**(`draw_selection.rs`):逐行画 quad,shader 里 `sdf.box(本行) + sdf.box(prev 行) + sdf.gloop()`(SDF smooth-union),Rust 侧维护 prev/next 三行滑动窗口传参,quad 左右外扩 8px 留融合空间 → 多行高亮融成**连续圆角块**。diff 的红/绿行底带用它,缩放/动画下边角始终平滑。
2. **折叠动画**(`session.rs:157-195` + `layout.rs:292-299`):每行 `scale∈[0.1,1]`,行高与 fold 列之后的宽度同乘 scale;每帧 `scale *= 0.9` 指数逼近、到阈值 snap;有行在动就请求下一帧(自驱动)。SDF 字形随 scale 缩小不糊。**这就是"折叠上下文行"该有的动画**。
3. **行级虚拟化**:预计算 `y: Vec<f64>` 前缀数组 + 二分求可见 `[line_start,line_end)`,所有绘制层只遍历窗口;脏标记 = `y.truncate(dirty_line+1)` 惰性续算。(与我们 plan29-V1 HeightIndex 同构,可互证。)
4. **波浪下划线 shader**(`code_editor.rs:57-66`):`pos.y + 0.03*sin(pos.x*w)` + stroke,一行代码级的"该行有问题"标注。

无 diff/git 集成;文本模型是 `Vec<String>` 行数组(非 rope),不要照搬去做大文件编辑器,但对 diff 卡片(百行级)无碍。

## 4. 差距对照(本引擎现状 vs Zed)

| 维度 | 本引擎(plan23 R3 + plan28) | Zed |
|---|---|---|
| 数据 | 首字符逐行分类(Added/Removed/Hunk/Context),无 hunk 结构 | hunk 化(Added/Modified/Deleted 由几何推导)+ 词级区间 |
| 增删配对 | 无(- 行与 + 行只是先后排列) | Modified hunk 内 old/new 配对,词级高亮指认改了哪 |
| 行底 | 整宽(卡内衬 12px)纯色带,`diff_add_bg/diff_del_bg` 全不透明深色 | hunk 色 12-16% 透明叠底 + hollow 变体;连续行段合并 quad |
| gutter | 无 | 每 hunk 色条(0.275×lh)+ 折叠 deleted marker + 可点击 |
| 行号 | 无 | 双列行号(old/new),色随 diff 状态 |
| 词级 | 无 | word_added/word_deleted 字符级背景(≤5 行限流) |
| 语法高亮 | diff 行无 | base text 是 language::Buffer,删除行也有高亮 |
| 上下文折叠 | 无(全量平铺) | 默认只显变更 + 展开;我们对应"长 context 折叠" |
| 摘要 | `+a -d` 文本徽章 | ±行数 + 5 格条(GitHub 风,本引擎 plan25 M2d 已有 5 格条数据) |

## 5. 落地方案(不换架构,分四步,每步独立可验)

> 映射:数据 = partspecific(0033 契约,只产语义);装饰 = 0018 panel 参数;动画 = 0016 morph;hunk 身份 = 0020 节点(新增 `NodeKind::DiffHunk` 或复用区间);颜色 = theme.rs + opencode 参考真值(plan28 R0 流程:diff 色先抄 opencode `--surface-diff-*`,词级色 opencode 若无则抄 Zed 的 word 色语义自配)。

- **D1 · hunk 化数据模型**(core,纯函数,native 可测):`diff_parse_lines` 升级为 `diff_parse_hunks`——把连续 `-`/`+` 段配对成 `DiffHunk{ kind: Added|Deleted|Modified, old_lines, new_lines, old_start, new_start }`(行号从 `@@ -a,b +c,d @@` 解析);Modified 且行数相等且 ≤5 行时算**词级区间**(LCS/最长公共子序列词对齐即可,输入是聊天 diff,规模小,不需要 imara-diff;分词先按空白+标点,语言感知留后)。输出 StyledSpan 时行携带 hunk id + 词级区间 sidecar(0020 区间同款)。
- **D2 · 三层视觉**(app.rs 装饰 + theme):行底带改 **hunk 色半透明**(叠在卡底上,数值先抄 opencode 参考,不确定处抄 Zed 的 0.12/0.16 档);左缘加 **gutter 色条**(每 hunk 一个 panel,宽 0.275×行高 floor);**双列行号**(old/new,Dim 色,diff 状态行换 version_control 语义色);词级区间画字符级背景(panel 子矩形或 glyph 底,叠行底上)。`+a -d` 徽章保留 + 5 格条(plan25 M2d 数据已有)。
- **D3 · gloop 连续圆角**(render,WGSL):行底带 shader 加 prev/next 行宽参数 + smooth-union(makepad gloop 手法,IQ smin 即 0025 已引的库),相邻同色行融成连续圆角块;单行时退化为普通圆角矩形。这步纯观感增强,可与 D2 并做或延后。
- **D4 · 上下文折叠 + 展开动画**:context 行 > N(如 3)时默认折叠为一行"⋯ n unchanged lines"(点击展开);展开/收起用 makepad 范式——被折行 scale 指数逼近(0016 端点 + 每帧推进,或直接复用 0025 动画通道),SDF 字形缩小不糊;折叠行 gutter 画 Zed 式强圆角小 marker。交互命中走 0032(已有 tool 卡折叠同款路径)。

**明确不抄的**:Anchor/SumTree/增量重算(聊天 diff settled 后不变,无编辑漂移问题);staging/hollow 态(无 git 语义);MultiBuffer(我们本来就是统一行流)。

## 6. 开放问题

- 词级分词是否接 0004 高亮的语言感知(Zed 用 language_scope)?v1 空白+标点足够,记遗留。
- diff 行内要不要语法高亮(Zed 靠 base text 是 Buffer;我们可对 old/new 文本各跑一次现有高亮再叠 diff 色)?建议 D2 后评估观感再定。
- 双列行号占宽 vs 聊天列宽紧张:可仅 hover/展开时显示,或只显 new 行号。

---

参考:Zed `crates/buffer_diff/src/buffer_diff.rs`(DiffHunk/SumTree/imara-diff/word diff 限流)· `crates/editor/src/element.rs:5291-5340`(gutter_strip_width/diff_hunk_bounds)· `crates/editor/src/git.rs`(DisplayDiffHunk/控件)· `crates/multi_buffer/src/multi_buffer.rs`(DiffTransform::DeletedHunk)· `crates/theme/src/styles/colors.rs`(diff 色键)· makepad `code_editor/src/draw_selection.rs`(gloop)· `session.rs:157-195`(折叠动画)· `layout.rs:292-299`(scale 网格)。本引擎:partspecific.rs R3 · app.rs flush_diff_band · plan23/25/28 · 0016/0018/0020/0033。
