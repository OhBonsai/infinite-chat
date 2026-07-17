# Plan 43 · 进度账本(首页画面化重构)

> 逐 milestone 记录:四问表 / 密度数值 / 直跳对拍 / 作者过目结论 / 遗留。
> GOAL:[plan43-homepage-slide-composition-goal](./plan43-homepage-slide-composition-goal.md)
> 方法论:huashu-design(位置四问 / earn its place / 一处 120% / 先 2 幕定语法)。

## C0 · 幕剧本重写(6 幕 × 位置四问,交作者过目)

- ✅ **交付物 = [design/homepage-director-notes.md](../design/homepage-director-notes.md)**(2026-07-15,
  按 huashu-design 导演笔记五部格式):Director's Statement(故事一行「每个字都是一等图元」+
  语境对话 + 观众画像 + 反臃肿自检)/ Visual System(8 色板铁律 + 字体 + 构图网格 + 动画字典 +
  Chrome 四件 + 音频预留 + per-shot 自检)/ Story Arc(三幕 42s + 情绪曲线,PEAK≈30s)/
  六镜 Shot-by-Shot(每镜四问 + TIMECODE + VISUAL + **ENGINE CUES 可执行序列** + SUBTITLE +
  CHROME + ANTI-SLOP + WHY)/ Production Manifest(plan43 对接表:文案真值/密度阈值/资产复用/降级)。
- 幕单:S1 TITLE(0-6s)· S2 THE CLAIM(6-12s)· S3 THE TABLE(12-19s)· S4 THE CARD(19-26s)·
  S5 THE BLOOM ★(26-36s)· S6 THE DOOR(36s+ 停留)。chat 钩子仅 S4 尾字幕深链。
- **待作者过目**:过目通过即放行 C1;C1 执行时补「slide_set part id 集 + 直跳对拍方案」细化记账。

## C1 · 语法两幕(S1+S3 精良,dpr=2 截图交作者定语法)

**替换式通道落地(架构,与语法方向无关,已跑通)**

- `web/src/pages/home-slides.ts` — `SlideDeck`:单 assistant 消息(`home`/`home-slide`),每幕 `build(id)` =
  **推本幕新 part(fresh id `s${seq}`)→ 删旧 part**(顺序承 `markdown.ts` 已验证替换术:始终 ≥1 part,
  免消息塌陷)。part id 集 = 每幕 `SLIDES[id]` 长度(title/door 1×text · claim/bloom 3×text ·
  table 1×text · card 1×tool)。幂等:任意直跳 → 先推本幕再删他幕 → 画布只余本幕内容(终态一致,C3 对拍)。
- `web/src/pages/home-scenes.ts` — 8→**6 幕**(title/claim/table/card/bloom/door);`enter()` 只设
  效果/节奏预设,内容替换交 `SlideDeck`;字幕瘦身(内容幕只留 eyebrow+深链,不与引擎抢标题)。
- `web/src/pages/home.ts` — `onScene` 加 `deckRef?.build(scene.id)`;`whenReady` 废母文档、建 `SlideDeck`;
  引擎就绪后 `prevIdx=-1` 让首幕重建。

**新增引擎能力(additive,缺口:短内容垂直居中)**

- 病根:引擎 `build_frame` 锚底 clamp `pan[1]=clamp(0, revealed-visible)` → 不足一屏内容恒**顶贴**,
  无法居中(app.rs:3373)。母文档时代靠长文档 + 相机框取绕开;替换式每幕短内容 → 直撞此约束。
- 修:`App.focus_valign`(默认 0=顶锚,**行为恒等旧版 → goldens 零改动**);>0 且内容不足一屏时
  `pan[1] = -(visible_h - content_height) * frac`(负 pan 内容整体下移居中)。presentation-only(只挪相机、
  不改布局/reveal)→ 不破 R8。`content_height` 用**全排版底**(含未揭示)→ 揭示中不漂。
  wasm 暴露 `set_focus_valign(frac)`;`SlideDeck` 构造时设 `0.42`(略高于正中)。
- 入口:`crates/core/src/app.rs`(field 1419 区 / setter near pan_by / build_frame ~3373)、
  `crates/wasm/src/lib.rs`(near pan_by)。

**踩坑(记 C2/C3 复用)**

- ★ **暂停引擎冻结事件泵**:`set_paused(true)`(播放器 Space / 页隐藏)后 `push_event` 不 drain → 内容替换
  静默失效。截图 harness 勿按 Space,靠**点进度点**停自动播(交互即停 autoplay,不冻引擎)。
- ★ 替换顺序 = **先推新 part 再删旧**(markdown.ts 铁律);反序(先删到空)会塌陷消息。
- `visible_messages` **不按 session 过滤**(全 view 线性文档);home 单 session 无碍,但 probe 用别的 sessionID
  会「看不见」(被 home 内容顶掉/挤到屏外)。

**S1+S3 截图(dpr=2)**:`test/results/plan43/s1-title.png`、`s3-table.png`(替换+垂直居中已生效)。

**语法定稿 = 引擎主角 + 居中放大(DoD-S3「单块居中放大」+ thesis「引擎排 keynote」;作者「keep going till finished」授权自主定)**

再加两项引擎能力(additive,默认关 → 恒等旧版,goldens 不变):
- `set_focus_zoom(z)`(围视口中心绝对缩放):幕主角放大。per-幕 ZOOM(home-slides):title 1.0(DOM 承标题)/
  claim 1.45 / table 1.18 / card 1.12 / bloom 1.0(编队自框)/ door 1.0。
- **水平居中**(focus_valign>0 时按**实际墨迹左右界**定 pan.x):修「放大后左漂/短行左贴」。宽表居中不溢。
  默认 focus_valign=0 → pan.x 不动 → 聊天流原样,goldens 不变。
- 揭示提速:reader 5.5cps 一屏内揭不完 → 正文幕(claim/table/card)改 typewriter + 38/44/60 cps,~2s 落屏。
- S6 door 画布静场(`SLIDES.door=[]`)免引擎行与 DOM 入口四卡重叠;`tap_fold_first`(wasm)接 S4 折叠 cue
  (短 diff 无折叠行 → 优雅 no-op;fold 动画待长 diff 触发,记 deferred)。

六幕 dpr=2 截图入 `test/results/plan43/`(s1-title/s2-claim/s3-table/s4-card/s5-bloom/s6-door)——
逐幕居中、大字、留白、单焦点、120% 仅 bloom。

## C2 · 全幕铺开(密度断言)

- S2/S4/S5/S6 均落地(见上;S5 复用 plan42 BloomController)。claim/bloom 三行合并为**单 part**(可见 part=1)。
- **密度断言(DoD-3)**:`pages-smoke.spec` 新增「landing 密度」测——逐幕 click dot → 读 `visible_turns`:
  可见 part≤2、distinct turn==1(door=0 画布静场)、正文幕文本≤阈值(S1≤20/S2≤60;表/卡/绽放单块免字数)。
- 转场:沿用 `flashTransition`(金光横扫 + 画布 blur 遮重建帧)。

## C2 · 全幕铺开(密度断言)

(未开始)

## C3 · 收口(smoke / 幂等对拍 / 降级同步)

- **pages-smoke 更新**:幕点 8→6;新增「landing 密度」逐幕断言(见 C2)。
- **直跳幂等对拍(DoD-2)**:`pages-smoke` landing 测「直跳幂等」——直跳第 3 幕 `frameTop` == 「先跳第 6 幕
  再跳第 3 幕」`frameTop`(替换式先推本幕再删他幕 → 终态只余本幕,与到达路径无关)。密度测进一步佐证
  (任意直跳 → part≤2、turn==1 = 只本幕内容)。
- **降级幻灯同步**:`home-fallback` SCENE_ASSET 重映射到 6 新幕 id(title/claim/table/card/bloom;door 交
  DOM 入口浮层)。`?slides` 烟测绿(6 点、点第 3 幕激活 1 张)。
- **LIVE·WGPU 水印**:`index.html` 加右下角 Mono 水印 + teal 呼吸点(导演文档 Chrome 件;声明真渲染非录屏)。
- 三页(chat/markdown/gallery)零改动;pages-smoke 三页测原样绿。

## DoD 对账

| # | DoD 要求 | 状态 | 证据/说明 |
|---|---|---|---|
| 1 | 幕单收敛 6 幕、每幕一焦点 | ✅ | title/claim/table/card/bloom/door;chat 钩子并入 S4「完整对话→chat/」+ S6 入口卡 |
| 2 | 替换式 + 居中框取 + 标题层级大字 | ✅* | SlideDeck 替换 + focus_valign 垂直居中 + 实际墨迹水平居中 + per-幕 zoom。*注:引擎不缩放 H1/H2 字号 → 「大字」由 keynote zoom 达成(非标题层级),记偏差 |
| 3 | 逐幕密度断言(part≤2/turn==1/文本≤阈值) | ✅ | pages-smoke「landing 密度」测 |
| 4 | C1 语法帧 dpr=2 截图 + commit | ✅ | test/results/plan43/ 六幕图;作者「keep going」授权自主定语法 |
| 5 | 不回退(播放器/bloom/五层门/smoke/三页) | ✅ | 五层门绿;bloom 幕保留;smoke 更新;三页零改 |
| 6 | 字幕层瘦身(eyebrow+标题≤12 字) | ✅ | SUBTITLES 瘦身;正文幕只 eyebrow+深链 |
| 7 | progress 记账 + milestone commit | ✅ | 本文件;C1 foundation `532e81e` + C1-polish/C2/C3 提交 |

**§3 客观判定**:密度 ✅(逐幕断言)· 幂等 ✅(frameTop 对拍 + 密度佐证)· 语法一致 ✅(六幕共用 focus_valign
0.42 + per-幕 zoom map + 统一字幕构图)· 一处 120% ✅(仅 bloom 有编队;余幕动画组 ≤2)· 不回退 ✅(门绿)。

**遗留(deferred)**:① S4 diff 卡折叠动画 —— `tap_fold_first` 已接 cue,但当前短 diff 不产折叠汇总行 →
优雅 no-op(卡默认展开可读);要动画需换长 diff。② 引擎标题字号层级(H1/H2 同正文)—— 本 plan 用 zoom 绕过,
根治需引擎排版加标题 scale(非本 plan 范围)。

## Plan 44 T3 衔接(2026-07-17)

S3/S4 两镜由 plan44 T3 **改为 tile 墙局部框取**(承 plan44 §5「44 先行,43 C1 语法幕改墙版」的最终形态):
- S3 = Markdown 墙局部 3 tile(HEADINGS/RICH/TABLE);S4 = Agent 卡墙局部 3 tile(USER/ASSISTANT/diff,自动展开)。
- 字幕深链改「全部组件 → /components/」;`home-slides.ts` TILE_SLIDES(tile 模式)+ `home-scenes.ts` SUBTITLES。
- **密度断言放宽**:S3/S4 = 墙局部(≤9 可见 part / ≤3 tile),仍单焦点区(非 chat 墙);S1/S2/S5/S6 不变。
- 引擎 tile 模式(0042)脱锚底 + focus_valign 居中框取沿用;plan43 其余幕零改动、五层门仍绿。
