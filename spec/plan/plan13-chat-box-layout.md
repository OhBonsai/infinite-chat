# Plan 13 — chat 级盒子布局(Taffy)+ 用户/模型左右分栏 + web 调试输入框

- 日期:2026-06-19
- 前置:[0023 Taffy 盒子布局](../decision/0023-taffy-box-layout.md)(主)、[0020 内容节点身份](../decision/0020-content-node-identity-model.md)(树,Plan 7 已落 `nodes.rs`)、[0005 Turn 聚合](../decision/0005-turn-aggregation-and-settlement.md)(**角色/回合,硬约束源**)、[0000 §无限会话/反例](../decision/0000-overview.md)、[0016 形变](../decision/0016-streaming-morph-render-model.md)、[0021 JS-Rust 边界/样式](../decision/0021-js-rust-boundary-and-configurable-render.md)、[0001 §2.2 measureText 护城河](../decision/0001-canvas-architecture.md)
- 一句话:把 chat 内容从「build_frame 手搓块竖直堆叠」升级为 **Taffy 盒子树(over 0020 节点树)**,并在最外层按 **0005 角色**做**微信式左右分栏**——**user = 一个盒(右)**、**assistant 一个回合 = 一个容器盒(左)**(结构稳定;盒内画什么随 part 类型/文本语法分阶段长出,v1 先落 markdown/纯文本,见 §2.2);另在 `web/` 加一个**纯前端调试输入框**,直接 `POST /session/{id}/message` 和 opencode serve 实时对话(回包走现有 Rust SSE 渲染)。

> 范围决策(2026-06-19,与作者确认):盒子布局**直接上 Taffy**(非最小手搓);输入发送**web 直接 POST**(不经 wasm)。

---

## 0. 现状 & 缺口

- **布局**:`crates/core/src/app.rs::build_frame`(~1060)对每个 part(view/`block_seq`)`top += c.height + BLOCK_GAP` 竖直堆叠——**一列、手搓、无嵌套弹性、无左右、无角色感**。0023 已定 Taffy 方向但未落地(§10 清单)。
- **节点树**:0020/Plan 7 已落 `crates/core/src/nodes.rs`(`NodeTree`,扁平 `Vec<Node>` 嵌套区间 + parent),`app.rs::block_nodes()` 暴露,但**尚无消费者**——Taffy 是它的第一个真正消费者。
- **角色**:`store`/`fsm` 已有 role(user/assistant,见 app.rs snapshot 测试);**build_frame 当前不区分角色**,user 与 assistant 一样左对齐铺一列。
- **输入**:`web/src/transport.ts` 其实是**揭示时间轴播放器**(非 SSE);`web/src/input.ts` 是画布 pan/zoom。**真正的实时对话只能靠 `scripts/chat.mjs`(CLI)发消息**;浏览器里没有输入口。
- **缺口**:chat 级布局(角色气泡 + container)目前只散见于 [0000 §反例]「一个回答别切五个气泡」与 [Plan 10 §4]「气泡分组融合」,**无正主**。本 plan 落地它。

## 1. 终态数据流

```
opencode 事件 → FSM 收敛(0003/0005)→ 0020 节点树/turn 投影(append-only)
  → 角色分栏外壳(本 plan §2)+ Taffy 盒子布局(§3,over 节点树,measureText 作叶子 measure)
  → build_frame 读盒位发 FrameGlyph/FrameRect/FramePanel(收编手搓堆叠)
  → 0016 形变补间(reflow 不跳变)→ render
```

Taffy 给**目标态盒位**,0016 给**过渡**,二者正交(0023 §6)。

## 2. 角色分栏模型(0005 为准)

```
ChatContainer                 ← 最外 Taffy flex column(整条会话一个大盒;无限会话 = 往下加 Turn)
  └ Turn(0005 纯投影,不存储)
      ├ UserRow      align-self:flex-end(右)   → 一个 User 盒
      └ AssistantRow align-self:flex-start(左) → 一个 Assistant 容器盒(回合内 parts 作内部块)
```

### 2.1 结构不变量(硬,不可违反)
- **每个角色一个盒**:assistant **一个回合 = 一个容器盒**(回合内多 message / 多 part 作**盒内部块**堆叠,**绝不为气泡而气泡**——守 0005 拍平语义 + 0000 §反例);user 一条 = 一个盒。
- 角色来自 store(0005),**不新增存储**;turn 边界 = 下一条 user 锚点(0005 §2),重投影即恢复。
- 左右 = Taffy `align-self` + 盒 `max_width`(留边),**不是新图元、不是新管线**。

### 2.2 盒内内容模型(演进,**v1 = 现状切片,后续按类型/语法长出**)
盒子结构稳定,但**盒内"画什么"是分阶段扩展的,不固定为 markdown / 纯文本**:
- **assistant 容器盒内 = 异构 part 块**:reasoning / tool / text / …,**每种 part 类型有各自的渲染 + 动画处理**(tool 调用有它的展开/状态机动画,reasoning/text 有各自的揭示)——接 Part 状态机(0002)+ 0005 拍平 + 0019 揭示编排。**本 plan v1 先只落 markdown(text part)**;tool / 其余 part 类型**后续按类型各自接处理(单独排期)**。"一回合一盒"约束的是**不另起气泡**,**不是**让 part 失去各自的处理。
- **user 盒内 = 文本,但文本带特殊语法**:`@ref` / 图片 / 文件 / link / 技能… 是 text 的特殊语法,**后续按语法渲染不同内容**(接 P 标签层 0006 / O 嵌入 0007)。**本 plan v1 先纯文本**。

> 一句话:**「一角色一盒 + 左右分栏」= 稳定结构;「盒内渲染什么」随 part 类型 / 文本语法分阶段扩展,markdown 与纯文本只是 v1 切片。**

### 2.3 动画与 Taffy 的接缝(§2.2 各类 part 动画怎么和布局共处)

后续 tool / reasoning / text 各有各的渲染 + 动画,**与 Taffy 不在同一根轴上,本身不冲突**——架构把**几何 / 过渡 / 表现 / 状态**拆四层,动画落在后三层,Taffy 只守几何:

| 轴 | 谁负责 | 改布局? |
|---|---|---|
| **目标几何**(盒 rect) | **Taffy**(静态,每次算一遍,不认识时间) | 是(唯一定位者) |
| **盒位过渡**(上方增高顶下面、列变宽) | **0016**:Taffy 给目标 rect,0016 从旧→新补间 | 否(追 Taffy 的目标态) |
| **盒内表现**(逐字淡入、tool pop、scale-in、glow、reasoning 渐显) | **0025 anim + 0019 揭示节奏**:盒几何内做 alpha/scale/offset | 否(几何内变换) |
| **状态**(tool pending/running/done) | **Part FSM**(0002) | 否(驱动上面三层) |

**唯一真接缝 = "改尺寸的动画"**(tool 展开/折叠、reasoning 折叠、流式块增高)。纪律:

```
part 的 FSM/动画 → measure 报「目标态尺寸」 → Taffy 局部重排 → 下游盒位变 → 0016 平滑
```

- **measure 只报离散"目标态尺寸",时间维度交 0016/0025**;**不让 measure 逐帧吐插值高度**(否则动画时间逻辑漏进布局层,职责糊 → 那才会冲突)。例:tool 展开 = measure 报展开后最终高,0016 补间高度 0→full。
- **enter 动画要占位**:Taffy 按最终尺寸排,内容在盒内淡入(0019 骨架先行),避免真身到达跳变。
- **exit/折叠**:0016 exit 淡出(路 A)+ measure 趋 0,Taffy 收编后移除。

**已有样板**:**表格就是这套模式的验证**——`placeTable` 作叶子 measure、`PanelScene` 走 0016、reveal gating 控节奏(0014 + Plan 8/9)。tool / reasoning **照表格这条已验证的路复制**:各作一个 Taffy 叶子(measure 报目标尺寸)+ 各自的 PanelScene/glyph anim + 各自揭示编排,**不是新冲突,是样板复用**。

## 3. Taffy 落地(over 0020 节点树)

照搬 0023 §10,逐项:
- 引 `taffy` crate(`crates/core`,wasm 目标);节点 `kind` → Taffy `Style`(block/flex,margin/padding/gap/align),**样式数据驱动**(0021,不写死)。
- `NodeTree`(nodes.rs)↔ Taffy 树:`parent` 给层级,容器节点(Doc/Message/List/Quote/TableCell)→ Taffy 容器,叶子(文本 run / 表格 / embed)→ 带 measure 的叶子。append-only ⇒ 只重排活动尾部子树,前缀冻结复用(0005 块冻结)。
- **叶子 measure**(守 measureText 护城河,0001/0021):
  - 文本块 → measure 回调打 **JS `measureText`** + 缓存(`Map<text+width, size>`);
  - 表格 → `placeTable`(0014)**降级为叶子 measure**(只算表内几何,文档流位置交 Taffy);
  - 图片/embed → `reportSize`(0022)/解码尺寸占位。
- Taffy 布局输出盒位 → **替** build_frame 手搓 `top += height + gap` → 喂 0016/`PanelScene`/embed 补间。

## 4. web 调试输入框(纯前端便利件)

- 新文件 `web/src/chat-input.ts`:画布下方一个 `<textarea>` + 发送按钮(Enter 发送 / Shift+Enter 换行)。
- 发送 = `fetch(`${serverUrl}/session/${sessionId}/message`, { method:POST, body:{ parts:[{type:text,text}], model } })`——**复用 main.ts 已有的 `serverUrl`/`sessionId` config**;`model` 显式带(沿用 `scripts/chat.mjs` 默认,避免空回,见 knowledge/opencode.md §4)。
- **回包不特殊处理**:assistant 的 SSE delta/updated 由**现有 Rust transport(M1)**接收并渲染——输入框只负责「把话发出去」。
- **纯 web、零 wasm/core 改动**(作者明确:输入只是方便输入的效果);路径前缀可配(knowledge §2,默认无前缀)。

## 5. 相位拆分(每相位独立可验)

| 相位 | 交付 | 验证 |
|---|---|---|
| **① Taffy 地基** | 引 taffy;`NodeTree`→Taffy 树 + kind→Style 映射(数据驱动) | cargo:构树正确 + 纯 block 堆叠结果 ≈ 现 `top+=height+gap`(等价回归) |
| **② 叶子 measure** | 文本 measure 回调(+缓存)/ placeTable 降级叶子 / embed 占位 | cargo + tsc:折行行数/尺寸与现 measureText 一致;缓存命中 |
| **③ 收编堆叠** | Taffy 盒位替 build_frame 手搓堆叠;接 0016/PanelScene | cargo:重放 case 块位置不回退;**GPU(人工)**:streaming 增高/列变宽平滑不跳 |
| **④ 角色分栏** | ChatContainer flex + per-Turn UserRow(右)/AssistantRow(左);**assistant 一回合一盒**硬约束 | cargo:多 part assistant 仍**一个盒**;角色→align;max-width 留边。**GPU(人工)**:左右上屏对 |
| **⑤ web 输入框** | `chat-input.ts` 直 POST + 复用 config + 带 model | tsc 绿;**人工**:起 opencode serve → 输入 → user 右气泡 + assistant 左流式 |
| **⑥ 基准+卡口** | 长消息 reflow 布局耗时 + measureText 回调缓存命中率 | 全卡口绿;基准入册 |

> **沙箱约束(同 Plan 12)**:core(Taffy 树/measure/盒位)与 tsc 可在沙箱跑测;**左右上屏、streaming 平滑、输入实时对话须人工 GPU/浏览器 + 本地 opencode serve 验收**。

## 6. 测试用例提纲

- [ ] 正常:单 user + 单 assistant → 左右分栏;assistant 多 message/part → **一个盒**(0005 硬约束)。
- [ ] 正常:嵌套列表 / 多级引用 → Taffy 缩进/gap 正确。
- [ ] 边界:空 assistant 回合;超长 user 文本(max-width 折行、不铺满);表格在流中列变宽 → reflow 不跳变(0016)。
- [ ] 边界:append-only 增量——仅活动尾部子树重排,前缀盒位冻结复用。
- [ ] 错误:measure 回调缓存 miss / measureText 异常;Taffy 输出 NaN 防护;POST 失败(网络 / 无 model 空回)有提示不崩。

## 7. Scope · 不做什么

- ❌ 不做全 CSS / RTL / 复杂脚本(0023 §7);文字测量**不 Rust 化**(0021 否)。
- ❌ **不把 assistant 一回合拆成多气泡**(0005 拍平不动)——但 part **按类型各自渲染/动画**仍是方向(§2.2),只是本 plan v1 只落 markdown(text part),tool/其余 part 类型后续单独排期。
- ❌ v1 user 盒只渲纯文本;`@ref`/图片/文件/link/技能等富语法后续(§2.2),本 plan 不含。
- ❌ 输入框不做富文本/附件/历史回溯/快捷键完整——只够联调实时对话;**不进 Taffy、不动 wasm**(页面外壳走 web CSS)。
- ❌ 头像 / 连续同发送者气泡融合(Plan 10 §4)、hover/选中——本 plan 不含,留后续。

## 8. Risk / Open Questions

- **measure 回调跨界频次**(0023 §9):活动块文本叶子多 → 布局期 measureText 回调多;靠缓存 + 只重排活动块守。相位⑥基准量化。
- **wasm 包体**:+Taffy(纯 Rust,中等),可接受。
- **契约扩张**:content→layout 从「扁平 run + sidecar」升到「节点树 + Taffy 样式」(0023 §9),改动面大,与 0020 同期。
- **Open**:① tool / reasoning 等 part 类型在 assistant 盒内的**各自渲染 + 动画**(§2.2)如何分相位——结构上确定**落同一盒、不另起气泡**,但每类的呈现单独排期。② user 文本特殊语法(`@ref`/图片/文件/link/技能)的渲染接 0006/0007,单独排期。③ 角色分栏模型是否回填一条 **ADR 0027**(目前决策散在 0005/0000/本 plan;落地后建议上提)。

## 9. Done

Taffy 收编 build_frame 块堆叠、重放无回退;**user 右 / assistant 左,assistant 一回合一个容器盒(v1 内部落 markdown,part 类型/富语法后续按 §2.2 扩展)**;web 输入框直 POST 可与本地 opencode serve 实时对话(回包走现有 SSE);卡口(cargo fmt/clippy/test native+wasm、wasm-pack build、tsc)全绿;reflow 布局基准入册。

## 10. 关联

- decision:[0023](../decision/0023-taffy-box-layout.md)(主)/ [0020](../decision/0020-content-node-identity-model.md) / [0005](../decision/0005-turn-aggregation-and-settlement.md)(硬约束)/ [0016](../decision/0016-streaming-morph-render-model.md)(盒位过渡)/ [0019](../decision/0019-reveal-gating-and-choreography.md)·[0025](../decision/0025-sdf-node-animation-system.md)(揭示/盒内动画,§2.3)/ [0002](../decision/0002-event-driven-pipeline.md)(Part FSM)/ [0014](../decision/0014-table-two-pass-layout.md)(measure+补间样板,§2.3)/ [0021](../decision/0021-js-rust-boundary-and-configurable-render.md) / [0001 §2.2](../decision/0001-canvas-architecture.md);可上提 **ADR 0027**(角色分栏模型 + §2.3 动画接缝纪律)。
- Code 入口:`crates/core/src/app.rs::build_frame`(~1060,被收编)、`crates/core/src/nodes.rs`(`NodeTree`)、`web/src/main.ts`(config:serverUrl/sessionId)、`web/src/chat-input.ts`(新)、`scripts/chat.mjs`(POST 范本)、knowledge/opencode.md §4(发消息契约)。
