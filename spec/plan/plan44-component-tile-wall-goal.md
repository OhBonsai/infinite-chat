# Plan 44 · GOAL:组件 Tile 墙 —— 引擎 tile 布局模式 + /components/ 总览页 + 首页两幕换墙(agent 直跑,无人介入)

- 日期:2026-07-15
- 状态:**GOAL(可执行任务书)** —— 给 Claude Code 的完整执行合同,读完即可自主跑到 DoD。
- 执行方式:**顺序执行,master 工作区直跑**。当前 master = plan28–42 全量 + plan43 C0(导演文档)。与 plan43 的关系见 §5(本 GOAL 的 T3 修订导演文档 S3/S4 两镜,建议 43 的 C1 起点合并到本 GOAL 之后执行或由本 GOAL T3 直接吸收——记账定夺)。
- **作者需求确认(2026-07-15 四问 + 追问)**:①实现层 = **引擎新增 tile 布局**(真渲染,引擎能力真升级);②落点 = **两处**(独立 /components/ 页全量墙 + 首页 S3/S4 用墙的局部框取);③动态性 = **静态成品态 + hover 才播**(首页幕例外:幕内错峰演示 2-3 tile);④尺寸 = **固定设计表**(4 列基网格,写死 manifest);⑤追问「tile 可以基于 SDF 做吗」= **是,且为铁律**——tile 容器 = 0018 panel(圆角/描边/AO/hover glow 现成参数),标题 = 引擎 glyph,整墙零 DOM 内容层。
- 需求本体:**Agent 会话卡全组件** + **Markdown 全类型组件**各一面 tile 墙;总宽 = chat 列宽;每 tile = 标题 + 真实渲染实例;**异构尺寸但边缘严格对齐**(Windows Metro / Grafana 语法,非 masonry 乱流)。

---

## 0. 铁律(四条)

1. **Tile 即 SDF**:容器/标题条/描边/hover 态全走 panel+glyph 图元,零 DOM 内容层、零截图(降级模式除外);缩放锐利是验收项。
2. **边缘对齐是硬断言**:4 列基网格 + 统一 gap(token);每个 tile 的四边必须落在网格线上(native 断言逐 tile 校验),异构只在 span(1×1/2×1/2×2/4×2…),不在偏移。
3. **默认安静**:组件页全 settled → idle 零重绘(既有断言);动 = hover 才播(同时 ≤1 tile 在动)。首页幕内错峰演示是唯一例外且幕末收敛。
4. **恒等硬门**:tile 布局是**新增模式**,非 tile 场景(chat/首页现有幕/一切既有页)代码路径零变化——默认态 golden 全集逐字节(承 plan36/37/42 惯例)。

## 1. GOAL 与完成定义(DoD)

**GOAL**:(A) 引擎新增 **tile 布局模式**(新 ADR,取下一空闲编号,预计 0042):列宽内 4 列基网格,块按设计表 span 摆位,panel 作 tile chrome;(B) **/components/ 第五页**:两面墙(Agent 卡 ≥9 tile + Markdown ≥16 tile),静态 + hover 重播;(C) **首页 S3/S4 两幕**改为墙的局部框取(导演文档修订);(D) 组件全清单 + 尺寸设计表 = T0 交付,先交作者过目。

**DoD(全满足才算完)**:

1. **ADR + tile 布局器**:`TileSpec{cols=4, gap, rows:[{id, span:(w,h), title, content_ref}]}`(serde,manifest 驱动——**布局=数据**);core 布局器把每 tile 的块 origin/width 按网格分配(行高 = 行内最高 tile 撑起,同行顶对齐;span 高不足者底部留白不拉伸内容);tile chrome = panel(底/描边/标题条)+ 标题 glyph。**非 tile 模式零触碰**(恒等硬门)。
2. **组件全清单(T0,作者过目)**:
   - **Agent 卡墙(≥9)**:user 气泡 / assistant 正文 / reasoning 卡 / tool-bash(completed)/ tool-read(running+shimmer)/ tool-edit+diff 卡(折叠态,**可点展开**)/ ask 交互卡 / compaction 分隔 / 错误兜底卡。
   - **Markdown 墙(≥16)**:标题层级 / 粗斜删+行内码 / 链接(可点)/ 无序+嵌套 / 有序 / 任务列表 / 引用 / Alert(≥2 型)/ 代码块(高亮+行号)/ 代码折叠视口 / 表格(对齐+CJK)/ hr+标题线 / 脚注 / 行内公式 / 块级公式 / 图片 / SVG 矢量 / emoji+CJK 混排。
   - 每 tile:`标题(Mono eyebrow 风)+ 实例内容(写死进 manifest,R8)`。
3. **尺寸设计表(T0 同交)**:4 列网格初稿——大格 4×2(diff 卡/表格/代码块)、中格 2×2(tool-bash/ask/块级公式)、横条 4×1(compaction/hr 族)、标准 2×1(多数)、小格 1×1(链接/行内码/emoji);T0 按内容容量四问(huashu)逐 tile 核定后写死。
4. **/components/ 页**:第五页入 nav(全站导航更新含 gallery 注入版);两面墙纵向排布(墙间大区隔 + 区标题);**hover 才播**——tile 命中(命中路由加 tile 级)→ 该 tile 单独重播流式揭示(单 part 替换术,markdown 页已验证;同时 ≤1);移开收敛回成品态;diff tile 折叠可点(既有 fold 路)。无 GPU 降级 = pages-assets 网格幻灯(同设计表布局)。
5. **首页 S3/S4 换墙(T3)**:导演文档修订两镜(S3 = Markdown 墙局部 4-6 tile 框取 + 幕内错峰演示 2-3 个;S4 = Agent 卡墙局部,diff tile 自动展开一次);字幕深链改「全部组件 → /components/」;plan43 密度断言按修订版更新(墙局部 = 多块但仍单焦点区,阈值随导演文档修订定)。
6. **验收与门**:对齐断言(逐 tile 四边在网格线,native)+ 缩放锐利(zoom ×2 定帧,SDF 判据)+ hover 重播 e2e(播/收敛/同时 ≤1)+ idle 断言(整页静止零活跃)+ 恒等硬门(既有 golden 全集逐字节)+ 五层门全绿;组件页 golden(两墙定帧,dpr=1/2)。
7. `spec/plan/plan44_progress.md` 记账;**按 milestone commit,不 push**;T0 单独 commit 交作者过目后再动工。

**启动**:`claude --permission-mode acceptEdits "读 spec/plan/plan44-component-tile-wall-goal.md,按其执行到 DoD 全满足"`(中断重跑同句,读 progress 续跑)。

## 2. Milestones

- **T0 · ADR + 清单 + 设计表(过目点)**:ADR(tile 模式:数据契约/布局规则/与单列模式的边界/恒等条款)+ 组件全清单(DoD-2 逐项内容文案写死)+ 尺寸设计表(每 tile span + 容量四问核定)→ **单独 commit 交作者过目**。
- **T1 · 引擎 tile 布局器**:core `TileSpec` 解析 + 块 origin 分配器 + panel chrome + 标题 glyph;恒等硬门先验(非 tile 路径 golden 逐字节);native:对齐断言/行高撑起/坏 manifest 整拒(AR12)。**单列假设排雷**:tile 场景关 follow(静态页无锚底)、揭示/命中/tier 按块工作不需改(块本就是独立单元,只是 origin 不再恒为列左缘)——排雷结果记账。
- **T2 · /components/ 页**:第五页骨架 + 两面墙 manifest 全量装配 + hover 才播(tile 命中 + 单 tile 替换重播)+ diff tile 可点 + 降级幻灯 + smoke/golden。
- **T3 · 首页两幕换墙**:导演文档 S3/S4 修订(镜头表/ENGINE CUES/密度阈值)+ home-scenes 接线(墙局部框取 + 幕内错峰演示)+ plan43 对接记账。
- **T4 · 收口**:全量断言(对齐/锐利/idle/恒等)+ 五层门 + 文档(DEPLOY/README/nav)+ DoD 对账。

## 3. 客观判定

| 维度 | 判据 |
|---|---|
| 对齐 | 逐 tile 四边 ∈ 网格线集合(native 精确断言);gap 全等 |
| SDF 纯度 | 组件页内容层零 DOM 元素(e2e DOM 断言:canvas + chrome 薄壳 only);zoom ×2 定帧锐利 |
| 恒等 | 非 tile 场景 golden 全集逐字节(硬门);chat/首页既有幕零变化 |
| 安静 | 组件页载入 settle 后 FrameStats 活跃组=0;hover 播时 ≤1 tile 活跃,移开 N 帧收敛 |
| 全量 | Agent ≥9 / Markdown ≥16 tile,清单逐项有 golden 内容 |
| 交互 | hover 重播 / diff 折叠点开 / 链接 tile 可点(白名单内)各一条 e2e |

## 4. 铁律(承 AGENTS.md / dev-practices §4)

- **§0 四条**(SDF 纯度 / 对齐硬断言 / 默认安静 / 恒等硬门)。
- **布局=数据**(0002 §5.1 家族):TileSpec manifest 驱动,换墙/改 span 零代码。
- **AR1/AR3** 渲染只读、hit-test 只读 settled · **AR12** 坏 manifest 整拒 · **R8** 实例内容写死、hover 重播走注入时钟。
- **Surgical**:HeightIndex/boxlayout 单列主路径不动(tile 是旁路模式);组件页块数少(~30)不需虚拟化,`?novirt` 语义沿用。
- **按 milestone commit,不 push**;T0 过目后才动 T1;改 core 过 AR 清单,改 render 先 `Skill(render-write)`,改 web 先 `Skill(bridge-write)`。

## 5. 变更边界

- **主战场**:core(tile 布局器新模块 + tile 场景装配)、primitives(TileSpec)、web(`components/index.html` + components.ts + nav 五页化)、导演文档 S3/S4 节、home-scenes。
- **非目标域**:chat/markdown/gallery/dev 四页、单列布局主路径、效果系统语义。
- **与 plan43 的衔接**:本 GOAL T3 = plan43 S3/S4 幕的最终形态;plan43 其余幕(S1/S2/S5/S6)不受影响;两 plan 的执行顺序 = **44 全程在 43 C1 之前**(墙是幕的素材,先有墙再定语法),plan43 C1 的"语法两幕"改为 S1+S3(墙版)——在 plan43 progress 记一行。

## 6. 风险与预案

| 风险 | 预案 |
|---|---|
| 单列假设埋雷比预想深(锚底/revealed_height/选区按列宽算) | T1 排雷清单先行(grep 假设点记账);tile 场景关 follow + 静态全揭示,绕开流式锚底;选区/查找在组件页可禁用记遗留 |
| 行内多 tile 的 broad-phase(HeightIndex 一维) | 同 y 带返回整行 tile(可见集略超),30 块量级无感;不改索引结构 |
| CJK/公式内容溢出 tile 容量 | T0 容量四问逐 tile 核定 + 内容裁剪写死;溢出 = 换 span 或删内容,不缩字号 |
| hover 重播与 settled 恒等冲突 | 重播 = 单 part 替换(新 part id),播完 settle 回成品态;AR3 断言把关 |
| 移动端 4 列过窄 | 设计表出 2 列变体(同 manifest 双档);或移动端直接降级幻灯,T2 定夺记账 |
| 与 plan43 执行顺序纠缠 | §5 已定:44 先行,43 C1 语法幕改为墙版;两边 progress 互记 |

> 一句话:**给引擎装上第二种布局语法(tile 网格),用它把全部组件排成一面 Metro 式鎏金墙**——每格都是真渲染、边缘像 Grafana 一样齐、默认安静指哪动哪;首页借它的局部,组件页给它全量。
