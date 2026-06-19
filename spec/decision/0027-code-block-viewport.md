# 决策记录 0027:代码块视口 —— maxHeight 行窗 + 边缘淡入淡出 + 双向滚动 + 行号 gutter + 复制图标

- 日期:2026-06-19
- 状态:已采纳(原型验证前)
- 前置:[plan13 Taffy 盒子布局](../plan/plan13-chat-box-layout.md)(代码块 = Taffy 叶子)、[plan14 图片嵌入](../plan/plan14-image-embed.md)(copy.svg→纹理)、[0018 SDF 面板装饰](0018-sdf-panel-decoration-primitive.md)(代码底)、[0011 逐字 SDF/per-glyph alpha](0011-gpu-text-as-sdf-primitive.md)、[0019 揭示](0019-reveal-gating-and-choreography.md)、[0016 形变](0016-streaming-morph-render-model.md)、[research 代码高亮](../research/code-block-syntax-highlighting.md)(逐字色,正交)
- 范围:代码块从"整段平铺 + 一条底 rect"升级为**裁剪滚动视口**——固定行窗 + 边缘 fade + 双向滚 + 行号 + 复制图标。

## 1. 背景 / 问题

- 代码块可能很长;聊天流里一段 200 行代码会把对话顶飞,**毁无限会话体验**(0000)。
- 现状:`StyleRole::CodeBlock` 整段平铺 + `block_decorations` 一条背景 rect(`app.rs`),**无高度上限、无滚动、无行号、无复制**。
- 目标(作者定):① **maxHeight = 6 行**的行窗,超出部分**上淡出 / 下淡入**(行数阅读);② 块内**纵 + 横滚动**;③ **行号**;④ **复制图标**(`web/public/copy.svg`)直接画进去;⑤ **复制交互、选中暂不做**(图标先在)。

## 2. 决策

**代码块 = 一个"裁剪滚动视口"(clip scroll viewport)**:外层盒高**钳到 maxHeight(默认 6 行,[0021] 可配)**,内部是 N 行代码的可滚动内容,渲染按视口**软裁剪 + 边缘淡入淡出**,左侧**行号 gutter**,右上角**复制图标**(钉住不滚)。六条:

1. **行窗盒(Taffy 叶子,固定高)**:代码块在 Taffy(plan13)= 高 `min(N,6)·lineH + chrome` 的**叶子**;内部 N 行代码独立内部布局,**不参与外层块流**。⇒ **过 6 行后盒高稳定**,不再顶走下文(无限滚锚定友好,接 plan13 §4.5 接缝3);流式只在视口内发生。
2. **软裁剪 = 边缘淡入淡出(per-glyph alpha,非硬 scissor)**:视口外的字 build 期**不发**(cull);上/下边缘 ~0.75 行 fade band 内,字 alpha 按到边距离渐变 → **上淡出、下淡入**。**复用现有逐字 alpha,无新管线**。顶端时不淡顶(首行清晰)、底端时不淡底——**仅"还有更多"那侧淡**。
3. **流式 = 自动跟随底部(tail)**:代码流入时新行在底"淡入"、旧行上滚出顶"淡出" → `scrollY` 自动跟最新 6 行(= 锚底的块内版);用户上滚则脱离跟随看历史。**"上淡出下淡入"本质 = 流式 tail + 边缘 fade。**
4. **双向滚动(块内状态 + 命中路由)**:每代码块 `scrollY(行) + scrollX(px)`,按 [0020] 节点 key 存;指针落视口内 → wheel/drag 滚**该块**(不滚画布)。需**最小 CPU 命中盒**(0011 §3.3④)。纵 clamp `[0, N-6]`,横 clamp `[0, 最宽行−视口宽]`。
5. **行号 gutter**:左列,宽 = 位数×字宽 + pad;每行号一字(**新角色 `CodeLineNum`** / Dim 色)。gutter **随纵滚**(号跟行)、**横滚不动**(号固定左侧);号同受纵向 fade;gutter 与代码间一条分隔线。
6. **复制图标(copy.svg)**:`web/public/copy.svg` 走 plan14 SVG→纹理,画成 `FrameImage` **钉右上角**(不随滚)、在裁剪层之上。**复制交互 / 选中本期不做**——图标先在(后续接 TODO Q 输入层 + 剪贴板)。

## 3. 备选与否决理由

- **硬 scissor 裁剪(无 fade)**:wgpu `set_scissor_rect` 每块一区,但① 无淡入淡出(作者要);② 多块多 scissor 分批麻烦。→ 取**软裁剪 per-glyph alpha**(fade 免费 + 不分批);**横向无 fade 处可用 scissor**(= 表格 C 横滚同款,本 ADR 顺带立此机制)。
- **不限高整段平铺(现状)**:长代码顶飞对话,违无限会话。
- **DOM overlay 代码块(0022)**:放弃 SDF 缩放锐利 + 与文字同台 morph,否决(同 0007/0013 取向)。
- **行号作 gutter 列进 Taffy**:号需"纵滚动横不动",非普通盒流;**内部自管更简单**,不污染 Taffy。
- **复制图标用 SDF widget(0026)重画**:已有现成 `copy.svg`,plan14 纹理路直接吃,**不重画**(轻)。

## 4. 影响

**正**:长代码不顶飞对话(盒高 ≤6 行稳定);流式 tail + fade 好看且助读;**立起"块内裁剪 + 滚动"机制**(表格 C 横滚、未来任意可滚区复用);复制图标即视,为后续复制交互铺路;行窗使代码块成本上限可控(渲染只 6 行 × per-frame)。

**负**:引入**块内滚动状态 + 命中路由**(最小 hit-test,= TODO Q 前哨);per-glyph fade 增 build 期少量计算;横向若用 scissor 则分批;`CodeLineNum` 等新角色 + gutter 布局。

**接点**:plan13(Taffy 叶子 + §4.5 锚底)/ plan14(copy.svg 纹理 + Embed)/ 0018(底面板)/ 高亮研究(逐字色,与本 ADR 正交:高亮定色、视口定裁剪滚动)/ 0019(揭示与 tail 协同)/ 0016(盒高 1→6 行进窗补间)。

## 5. 实现入口(file:符号)

- `crates/core/src/app.rs`:`block_decorations`(代码底迁视口面板)、`build_frame`(视口外 cull + 边缘 fade alpha + tail `scrollY`)、新**块内滚动状态**(`HashMap<node_key,(scrollX,scrollY)>`)。
- `crates/core/src/content.rs`:代码行**注入行号** + `StyleRole::CodeLineNum`(数值尾追加,守 0001/0026 稳定)。
- `crates/core/src/frame.rs`:复用 `FrameImage`(copy 图标)/ `FramePanel`(底 + gutter 分隔);如走横向 scissor,加视口裁剪元信息。
- `web/src/input.ts`:指针命中代码块 → 路由 wheel/drag 到块 `scrollX/Y`(不滚画布)。
- `web/src/image-loader.ts`(plan14):预载 `web/public/copy.svg` → 纹理。
- 落地清单 → [plan15](../plan/plan15-code-block-viewport.md)。
