# Plan 43 · GOAL:首页画面化重构 —— 每幕是设计的画面,不是聊天记录的取景框(agent 直跑,无人介入)

- 日期:2026-07-15
- 状态:**GOAL(可执行任务书)** —— 给 Claude Code 的完整执行合同,读完即可自主跑到 DoD。
- 执行方式:**顺序执行,master 工作区直跑**。前置 plan41/42 完成(播放器 8 幕 + 字阵 bloom 在)。
- 作者反馈(2026-07-15):当前首页**太臃肿**——"一堆 chat 放在后面"。要**简单、突出重点**。参考 `~/w/agentscode/huashu-design` 技能的画面设计法(作者认可其画面)。
- 病根诊断:plan41 的「母文档」架构 = 一整场对话 + 幕相机框取 → **每幕都是 chat 墙的局部**,没有一幕是被"设计"出来的画面;字号是正文级、密度是对话级、焦点是散的。
- 方法论来源(huashu-design SKILL.md,已通读):位置四问 / earn its place / 一处 120% 其余 80% / 先 2 幕定语法再铺量 / 单页零自带页码 / 反 slop(不堆 stats/icon/渐变)。
- **导演文档(C0 交付物,已完成 2026-07-15)**:[design/homepage-director-notes.md](../design/homepage-director-notes.md)——按 huashu 导演笔记五部格式写的完整 timeline(Director's Statement / Visual System / Story Arc 三幕 42s / 六镜 Shot-by-Shot 含 ENGINE CUES / Production Manifest)。**本 GOAL 的幕单/文案/时长/密度阈值以该文档为真值**;下文 §1 的幕单为其摘要,冲突以导演文档为准。

---

## 0. 设计语法(从 huashu 移植,本 GOAL 的第一铁律)

1. **每幕先答位置四问**(写进幕剧本,每幕四行):①叙事角色(hero/是什么/单点演示/高潮/尾幕——各幕不同);②观众距离 = **1m 笔记本偏投屏**(→ 大字、低密度:每幕大标题 ≤12 字、辅文 ≤2 行);③视觉温度(安静庄重,金/深蓝,承 lol-style token);④容量 thumbnail(内容塞不下 = 砍内容,不是缩字号)。
2. **Earn its place**:每幕**恰一个焦点**(一段大字 / 一个组件卡 / 一场动效),其余全删。**完整对话不进首页**——它住在 /chat/,首页只给 6 秒钩子 + 深链。"把 chat 堆在后面"即臃肿的定义。
3. **一处 120%**:全片只允许一个"值得截图"的高潮(plan42 字阵 bloom 天然是它);其余幕克制到 80%。
4. **先 2 幕定语法**:先做 S1 + 一个内容幕到精良,**截图交作者过目定语法**,再铺其余(huashu deck 铁律,防方向错返工 N 次)。
5. 反 slop:不加装饰 icon/无意义数字/渐变堆叠;幕内零页码(chrome 承载)。

## 1. GOAL 与完成定义(DoD)

**GOAL**:保留播放器模式与单 canvas 引擎真渲染,把「母文档 + 相机框取」重构为「**每幕一份画面内容**」——幕切换 = 引擎内容替换(单 part 替换术,markdown 页已验证),每幕是按 §0 语法构图的 slide;总幕数收敛(8→6),chat 全量演示撤出首页。

**DoD(全满足才算完)**:

1. **幕单收敛为 6 幕**(剧本可在 C0 调,但总数 ≤6、每幕一个焦点):
   - S1 hero:大标题 + 一行定位(引擎渲大字号氛围行,字幕层承标题)——安静;
   - S2 是什么:**三行大字**(引擎渲 H2 级,居中留白),不是自述段落;
   - S3 单点演示 ①:一块流式 markdown(表格或公式,**单块居中放大**,骨架先行看得清);
   - S4 单点演示 ②:一张 tool/diff 卡(单卡居中,折叠展开一次);
   - S5 高潮:字阵 bloom(plan42,唯一 120% 时刻);
   - S6 尾幕:入口四卡 + 一行页脚(现有)。
   chat 钩子并入 S4 尾 cue 或 S6 卡片文案("完整对话 → /chat/"),**不再有"完整对话幕"**。
2. **内容架构改替换式**:废母文档;每幕 enter() = "确保画布只有本幕内容"(固定 part id 集,upsert 本幕 + remove 他幕——幂等:任意直跳终态一致,e2e 对拍);内容居中框取(scroll_to + 视口居中),字号走 markdown 标题层级(H1/H2 大字)。
3. **每幕密度断言**:e2e 逐幕检查——可见文本 ≤N 字(阈值进剧本,如 S2≤60 字)、可见 turn ==1(除 S6);"chat 墙"不再出现(可见 part 数 ≤2)。
4. **语法先行**:C1 先交 S1+S3 两幕截图(dpr=2)入 `test/results/plan43/` 并 commit,**GOAL 在此设作者过目点**;其余幕按定稿语法铺。
5. **不回退**:播放器行为(自动播/接管/直跳/循环/降级)原样;plan42 bloom 幕保留;五层门全绿;pages-smoke 更新(幕数/密度断言);chat/markdown/gallery 三页不动。
6. 字幕层同步瘦身:每幕 eyebrow+标题 ≤12 字,删多行要点字幕(内容职责还给引擎画面)。
7. `spec/plan/plan43_progress.md` 记账(四问表/密度数值/对拍);**按 milestone commit,不 push**。

**启动**:`claude --permission-mode acceptEdits "读 spec/plan/plan43-homepage-slide-composition-goal.md,按其执行到 DoD 全满足"`(中断重跑同句,读 plan43_progress 续跑)。

## 2. Milestones

- **C0 · 幕剧本(✅ 已由导演文档交付)**:[homepage-director-notes.md](../design/homepage-director-notes.md) 即 C0 交付物——六镜四问表/文案真值/ENGINE CUES/密度阈值俱全;**作者过目该文档后 C1 放行**。执行 agent 补一项:替换式 `slide_set()` 的 part id 集与直跳对拍方案细化进 progress(导演文档 Part V 已定原则)。
- **C1 · 语法两幕**:替换式通道落地(home-scenes buildSlide/enter)+ S1+S3 做到精良(居中/大字/留白/骨架先行);dpr=2 截图 commit,**作者定语法点**。
- **C2 · 全幕铺开**:S2/S4/S5/S6 按语法完成;密度断言 e2e;转场沿用 dissolve。
- **C3 · 收口**:pages-smoke 更新 + 直跳幂等对拍 + 降级幻灯同步瘦身(fallback 卡片同 6 幕)+ DoD 对账。

## 3. 客观判定

| 维度 | 判据 |
|---|---|
| 密度 | 逐幕可见文本 ≤阈值、可见 turn/part 数断言(e2e);无 chat 墙 |
| 幂等 | 任意幕直跳 vs 顺播:可见文本集一致(对拍) |
| 语法一致 | 六幕共用同一构图 token(居中列宽/标题字级/留白比,进 pages-theme) |
| 一处 120% | 仅 S5 有 bloom 级动效;其余幕动画组 ≤2 |
| 不回退 | 播放器 e2e 原样绿;三页 smoke 原样;五层门绿 |

## 4. 铁律

- **§0 五条设计语法**(本 GOAL 存在的理由)。
- 引擎零改动优先(替换术用既有 push_event/part.removed;缺口才 additive)。
- 内容文案写死进剧本(确定性,R8);幕内零墙钟。
- **作者过目两点**(C0 剧本 / C1 语法帧)——设计方向的人工把关,不自评好看(plan25 教训)。
- **按 milestone commit,不 push**;改 web 先 `Skill(bridge-write)`。

## 5. 变更边界

- **主战场**:`web/src/pages/home-scenes.ts`(重写)/ home.ts / index.html 字幕层 / home-fallback(同步 6 幕)/ pages-smoke。
- **非目标域**:引擎、chat/markdown/gallery/dev 四页、film director、bloom 实现(只调用)。
- plan41 的母文档/scroll_to 框取术**退役**(替换术取代);踩坑记录(c.call Proxy/follow 覆盖)仍有效沿用。

## 6. 风险与预案

| 风险 | 预案 |
|---|---|
| 替换式破坏直跳幂等 | enter 收敛到固定终态(本幕 part 集);对拍断言把关;转场 dissolve 已盖重建帧 |
| 大字号排版超出引擎习惯(H1 级正文少见) | 先 C1 两幕验证;不达标退"正常字号 + 相机 zoom 放大"方案(世界空间 zoom plan42 已通) |
| 删内容后首页"太空" | 空白靠构图(居中/留白比 token),不是靠填回内容(huashu:空白是设计问题) |
| 作者过目往返拉长周期 | 两个过目点各一轮;文案预写进剧本降低往返 |

> 一句话:**从"给聊天记录拍宣传片"改成"给引擎排六页 keynote"**——每幕一个焦点、大字、留白,chat 回它自己的页面;唯一的烟花留给字阵开花。
