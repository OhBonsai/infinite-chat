# Plan 7(内容节点树地基:0020 落地)

- 状态(2026-06-16):**已落地**(地基 + 测试,除人工验证外全绿)。`crates/core/src/nodes.rs`:`NodeKind`/`Node`/`NodeTree` + append-only 扁平表构建(嵌套区间 + parent 撑开 + `(block_seq,node_seq)` key)+ 查询(`nodes_of_kind`/`node_at`/`ancestors`/`children`/`range_of`)+ 不变式校验器;`content::parse_markdown_nodes` 拍平时顺带建树;`app` 存 `BlockCache.nodes` + `Engine::block_nodes` 暴露;`NodeKind::Embed` 占位 + 下游接口面文档(7D)。**测试**:9 个结构样例(n01–n09)过不变式 + kind/嵌套/表格/`node_at`/祖先/append 稳定/glyph_key 打包(node_tests)+ engine 集成测;101 workspace 测试全绿。
- **评审修订(2026-06-16)**:① 折掉死的并行 API `parse_markdown_tables`(无非测试调用方)→ 统一走 `parse_markdown_nodes`(单源准则);② **`?debug` 节点框叠加已接**(`app::node_debug_rects`,随 `debug_geometry` 开关、复用 4C3 叠加 → 经 `FrameData.rects` 到 render/web,无需新 wasm/web 代码)→ 节点树已被消费;③ 嵌套列表忠实建成 `List→ListItem→List`(嵌套 List 父 = 外层最近 ListItem)。
- **7B 取舍**:节点树为**权威身份模型**,`TableRegion` 暂留作 layout 喂料视图(测试证 `TableCell` 节点区间与之等价),**物理合并(placeTable 直读节点)留后续**(低优,评审 #3)。`?debug` 节点框的浏览器肉眼复核 = 人工验证(目标排除项)。
- 日期:2026-06-16
- 范围:把"内容是什么结构、每个结构块/run/cell 的稳定身份"显式化成一张表。**只建地基与查询能力,不动消费侧**(reveal/Taffy/embed 是后续 plan)。
- 前置:[0020](../decision/0020-content-node-identity-model.md)(本 plan 落地它)、0010(jcode `Document` 块树,拍平来源)、[0016](../decision/0016-streaming-morph-render-model.md)(`NodeId` 现状)、0014 B(`TableRegion`)、0017(append-only / 提交前沿)、[0021](0021-js-rust-boundary-and-configurable-render.md)(契约/边界)。
- 相位:**7A 数据结构 + 构建 → 7B 统一现有身份 → 7C 过界 + 缓存 + 查询 → 7D 下游接口预留 → 7E 测试 + 调试可视**;一相位 ≈ 一/数 PR,末过卡口。
- **不含**(均为本地基的**下游消费者**,各自后续 plan):reveal 调度/骨架先行(0019)、Taffy 盒子布局(0023)、DOM embed(0022)、节点级 morph 升级(0016 路 A)。本 plan 只**铺表 + 暴露查询**,不写任何一个消费者。

## 0. 定位 / 为什么先打这层地基

现在身份是**三处零散**:0016 的 glyph `NodeId(block_seq,glyph_idx)`、0014 的 `TableRegion`(run 区间)、未来 0019 的 Selector / 0022 的 embed / 0023 的 Taffy 树各要一套。0020 的洞察:**append-only ⇒ 子树在扁平 glyph 数组里连续 ⇒ 树退化成"区间 + parent 下标"的扁平表**,免指针、免 diff、免重排,还能喂 GPU。**先把这张表铺好**,后面 reveal/Taffy/embed 都在它上面长,不必各造轮子。

**简约边界**:结构按**终态**铺(节点树是唯一身份源,`TableRegion`/glyph/`FramePanel` 身份**折进它**),实现取最小子集——扁平 glyph 流保留是**目标设计**(GPU/AR10,非兼容妥协);路径哈希 v1 先用 `(block_seq,node_seq)`;不引 Taffy、不写消费者。**0→1 不留并行旧产物、不为兼容打补丁**(设计准则)。

**In**:`NodeKind` + `Node`(kind/parent/range/key)+ `NodeTree`;从 jcode `Document` 拍平时顺带建;收编现有 glyph/Table 身份;过界 sidecar + `BlockCache` 缓存;**查询 API**(按 kind / 区间包含 / 祖先);调试可视(节点框)。
**Out**:任何消费者(reveal/Taffy/embed/节点级 morph);通用渲染改动;tile 分桶(0020 §6 正交,后续)。

**铁律**:content→layout→render 契约不破(节点树旁挂,不进扁平 glyph 流,同 TableRegion 精神)/ AR10 每帧一次跨界 / core 确定性可测(CR1)/ append-only 身份稳定(0017 §6)/ **opinionated 简约单实现**。

---

## Phase 7A — 数据结构 + 构建(0020 §3/§4)

> 域:`core`(content)。**简约**:扁平 `Vec<Node>`,文档序,随 `emit_*` 顺带建(已在遍历块树,additive)。

**数据**
```rust
pub enum NodeKind { Doc, Paragraph, Heading, List, ListItem, Quote, CodeBlock,
                    Table, TableRow, TableCell, Run /* 同样式连续段 */, Glyph }
pub struct Node {
    pub kind: NodeKind,
    pub parent: u32,        // 父下标(根 = 自身/哨兵)
    pub range: (u32, u32),  // [start,end) into 块内扁平 glyph 数组(嵌套区间,0020 §3A)
    pub key: u64,           // 跨帧稳定身份;v1 = (block_seq<<32)|node_seq(append-only 稳定)
}
pub struct NodeTree { nodes: Vec<Node> } // 文档序,按 range.start 有序
```

**任务**
- [ ] `content.rs::emit_*` 拍平 `Document` 时**记区间**建 `NodeTree`:进块 push 容器节点(占位 range 待回填)、出块回填 `end` + `parent`;run/glyph 叶按发射序记 range。
- [ ] `key` v1 = `(block_seq, node_seq)` 打包(0017 append-only ⇒ 稳定);**路径哈希留后续**(0020 §3C,v1 不做)。
- [ ] `parse_markdown_tables` 旁加产 `NodeTree`(或新 `parse_markdown_nodes`),additive,不改现有返回。
- 卡口:`cargo test`——嵌套区间正确(子树连续、祖先包含)、parent 链对、表/cell/list-item 命中;现有 content 测试不回归。

## Phase 7B — 收编为唯一身份源(0020 §4;直接折进,不并行)

> 域:`core`。**节点树 = 唯一身份源**,把三处零散身份**折进它、替换掉**(0→1 不留并行旧产物)。

**任务**
- [ ] **glyph 叶 = 退化节点**:`Glyph` 叶 `range` 长度 1;`key` 即 0016 `NodeId` 的同源打包(高 32 block_seq、低 32 idx)→ glyph 身份直接是节点身份,不另存。
- [ ] **表格折进节点**:`Table/TableRow/TableCell` 节点直接承载 cell 区间;**`TableRegion` 收敛为"从节点树派生"或直接由节点表取代**(placeTable 改读 `TableCell` 节点的区间,而非单独的 `TableRegion`)——目标是单结构,不双轨。
- [ ] **面板 id = 节点 key**:`FramePanel.id` 直接取对应 `Table` 节点 key(删掉 6D 里 `(block_seq,表序号)` 的独立打包)。
- 卡口:身份一致性单测(glyph key == NodeId;cell 节点 range 正确驱动 placeTable);重放 c06* 渲染结果正确(正确性,非"兼容旧产物")。

## Phase 7C — 过界 + 缓存 + 查询 API

> 域:`core`(seam/app)。节点树旁传(同 `TableRegion`)、缓存、暴露查询——查询是地基的**对外能力**(下游 Selector/embed/morph 都用它)。

**任务**
- [ ] sidecar 旁传:`NodeTree` 经 layout 边界旁传(**不进扁平 glyph 流**,AR10 不破);`app` 存 `BlockCache.nodes`。
- [ ] **查询 API**(纯 CPU,有序表上二分/线性):
  - `nodes_of_kind(kind) -> &[Node]`;`node_at(glyph_idx) -> 最内层节点`;`ancestors(node) / children(node)`;`range_of(key)`。
  - 命中测试基础:`glyph 属于哪个 cell/list-item`(区间包含)。
- 卡口:查询单测(kind 过滤、区间包含、祖先链);O(活动块) 重建,与会话长度无关。

## Phase 7D — 下游接口预留(只定形,不实现消费者)

> 域:`core`(类型/trait 草案 + 文档)。**不写消费者**,只把地基对下游的接口面定清,确保 0019/0022/0023 接得上。

**任务**
- [ ] 文档化三个消费点(0020 §下游):① 0019 `Selector` = `nodes_of_kind` + 区间;② 0022 `EmbedNode` = `NodeKind::Embed` 占位(本 plan 加该 kind 枚举 + 留 range,不做 DOM);③ 0023 Taffy 树 = 节点 + parent;④ 0016 节点级 morph = 按节点 range 整体补间(0020 §6 纪律:身份 key 化、与渲染 buffer 顺序解耦)。
- [ ] 留 `NodeKind::Embed`(占位,content 暂不产,接口先在)。
- 卡口:无新行为;仅类型/文档;`cargo doc` 通。

## Phase 7E — 测试 + 调试可视

> 域:`core` + `web`(调试)。

**任务**
- [ ] 节点树覆盖测试:段落/标题/列表(嵌套/有序/任务)/引用/代码块/表格 → 节点结构 + 区间 + 身份。
- [ ] `?debug` 节点框叠加(复用 4C3 几何叠加):按 kind 描边节点 range 的 AABB,肉眼验证树对不对。
- 卡口:卡口全绿;调试叠加可见各结构节点边界。

---

## 评审(取舍 / 风险 / 备选)

**为什么"地基先行、消费者后置"**:0019/0022/0023 都要"按节点寻址"。把身份地基**一次铺对**(append-only 区间编码),三个消费者各省一套轮子;且地基改动可控,可在不写消费者的前提下独立落地、独立验证(调试叠加)。

**简约取舍(v1 主动砍掉,留后续)**:
- **路径哈希 `key`** → v1 用 `(block_seq,node_seq)`(append-only 已够稳);需跨会话/diff 稳定时再上哈希(0020 §3C)。
- **SoA / 上 GPU** → v1 `Vec<Node>` CPU 表(0020 §5 节点效果再说)。
- **tile 分桶正交校验** → 不在本 plan(0020 §6,上 tile 时再守纪律)。
- **任何消费者**(reveal/Taffy/embed/morph 升级)→ 全部后续 plan;本 plan 只铺表 + 查询 + 接口面。

**风险**:
- **契约扩张**:content→layout 多一个 `NodeTree`(旁挂)。注:扁平 glyph 流保留是**目标设计**(GPU/AR10),非兼容妥协;`TableRegion` 折进节点树(单结构)。
- **身份折并正确性**:glyph/Table 折进节点后,placeTable/morph 仍需产正确几何。缓解:7B **单结构驱动**(节点树 → placeTable / FramePanel),重放 c06* 验渲染正确。
- **构建正确性**:嵌套块(引用里套列表套代码)的 parent/区间易错。缓解:7E 覆盖测试 + `?debug` 节点框肉眼验。
- **过度设计**:地基没消费者时容易镀金。缓解:本 plan 只产"3 个消费者确实要的"字段(kind/parent/range/key)+ 查询,不多(简约)。

**备选(不选的)**:① 不铺地基、各消费者各造身份 —— 三套轮子 + 不一致(现状的放大)。② 直接上 0023 Taffy(树 + 布局一起)—— 把地基和布局耦在一起、改动巨大,与"打好地基、简约"冲突。**故选"先铺纯身份地基,布局/揭示/embed 后接"。**

**验收(DoD)**:
1. 一棵 `NodeTree` 正确描述任意 markdown(段/标题/列表嵌套/引用/代码/表格)的结构 + 稳定身份。
2. glyph(0016)/ 表格(0014/6D)身份**折进节点树成单一源**(无并行旧身份结构),渲染正确。
3. 查询 API 可"按 kind 取节点 / 命中 glyph→最内节点 / 取祖先链"。
4. `?debug` 节点框叠加可视;覆盖测试全绿;卡口全绿。
5. 0019/0022/0023 的接口面已定、能接(文档 + `Embed` kind 占位)。

## 测试 / 验收(整套:怎么测)

地基**无新用户可见行为**(不像 reveal 有可眼看体验)→ **以 native 单测为主**(CR1,`cargo test -p infinite-chat-core` 不需 GPU/wasm),外加渲染数据回归 + `?debug` 节点框肉眼兜底。四层 A/B/C + 卡口:

### A. 单元测试(`cargo test -p infinite-chat-core`)—— 主力

1. **树结构**:每种构造 → 断言 kinds + parent + 嵌套(段落/标题/列表[有序/无序/嵌套/任务]/引用[嵌套]/代码块/表格 Table·Row·Cell/混合)。
2. **嵌套区间不变式**(写可复用 `assert_tree_invariants(&NodeTree)`,套全部用例):子 `range ⊆` 父;兄弟按 start 有序不重叠;叶子连续覆盖、无空洞;根覆盖 `[0,glyph_count)`;parent 链合法无环。← 最值钱,挡"树建错"。
3. **身份折并等价**:glyph 叶 `key`==0016 `NodeId`;`TableCell` 节点 range==原 cell run 区间 → 喂 placeTable 产同样列;`FramePanel.id`==`Table` 节点 key。
4. **查询 API**:`nodes_of_kind` / `node_at(glyph_idx)`(命中最内 cell/list-item)/ `ancestors` / 区间包含。
5. **append-only 稳定**(0017 §6):同前缀追加内容 → 已有节点 `key` + `range.start` 不变(`parse("a\n\nb")` 的 a key == `parse("a\n\nb\n\nc")` 的 a key);只尾部变。

### B. 渲染数据回归(native,**不需 GPU**)

`CollectSink`(support.rs)驱动 `Engine`,断言折并身份后 `build_frame` 产的 `FrameGlyph`/`FramePanel`(几何+角色)与基线一致 → 证明"折进节点树"没改渲染结果。(像素级:重放 `c06*` 截图对拍,需本机 GPU。)

### C. `?debug` 节点框可视(浏览器,肉眼)—— 调试用例

`?debug` 开节点框叠加(7E,复用 4C3 几何叠加):按 kind 描边每个节点 `range` 的 AABB。用法 `?replay=<case>&debug`(复用 5D 重放),逐 case 扫一眼"树是否套对每个结构块"。**新增节点树调试用例**(`web/public/cases/`),覆盖各结构:

| case | 内容 | 验什么 |
|---|---|---|
| `n01-headings` | `#`/`##`/`###` 混排 + 段落 | Heading 各级 + Paragraph 边界 |
| `n02-inline` | 段落含 **粗**/*斜*/`码`/[链接]/~~删~~ | Run 节点切分(同样式连续段)|
| `n03-list-flat` | 无序 + 有序 + 任务 `- [ ]`/`- [x]` | List/ListItem 节点 + 序号/勾选 |
| `n04-list-nested` | 2–3 层嵌套列表 | parent 链 + 区间包含(深嵌套)|
| `n05-quote-nested` | 引用 + 嵌套引用 + **引用里套列表套代码** | 跨类嵌套 parent/区间最易错处 |
| `n06-codeblock` | 围栏代码块(带语言标注)| CodeBlock 节点边界 + 不误切 |
| `n07-table` | 复用 `c06-all`(对齐/CJK/内联/残缺/宽)| Table/Row/Cell 节点 + 与 placeTable 对齐 |
| `n08-mixed` | 标题+段落+列表+引用+代码+表格 一条长消息 | 整树覆盖 + 兄弟有序 + 根覆盖 |
| `n09-edge` | 空列表项、超深嵌套、引用紧贴代码、空表/单列 | 边界不崩、区间不空洞 |

> 每个 case 同时是 A 单测的输入(同一 markdown 串复用):单测断言结构,`?debug` 肉眼复核 AABB 套得对不对。

### 卡口

`cargo fmt --check` / `clippy -D warnings` / `cargo test`(含 A)全绿;`wasm-pack build` + `tsc`(本 plan 主要动 core)。验收对齐上面「评审 · DoD」5 条。

## 依赖 / 与 ADR 对应

| 相位 | 兑现(0020) | 依赖(已落地) |
|---|---|---|
| 7A | §3 嵌套区间 + §4 数据结构 | content / jcode Document |
| 7B | §4 统一 NodeId/TableRegion/FramePanel | 0016/0014/6D |
| 7C | §4 查询 + §1 契约接合(sidecar) | seam/app/TableRegion 过界 |
| 7D | §下游(0019/0022/0023/0016 接口面)| — |
| 7E | §9 落地清单(测试) | 4C3 调试叠加 / 5D 重放 |

> 后续(本 plan 之外,均消费本地基):**reveal 节奏自主(0019)**、**Taffy 盒子布局(0023)**、**DOM embed(0022)**、**节点级 morph + exit 淡出(0016 留尾)**、路径哈希 `key` / tile 分桶正交(0020 §3C/§6)。
