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

**⏳ 待作者定语法(C1 过目点,plan §4「不自评好看」)**:引擎内容目前**自然字号 + 左对齐**(chat 列左贴),
DOM chrome 承大字居中。DoD-S3「单块居中放大」要引擎**水平居中 + 放大(zoom)** —— 需再改引擎(text-align/
水平 pan + keynote zoom)。三方向待选:①引擎主角+放大(忠于 thesis,多改引擎)②DOM 标题+引擎实况(省改动、
偏离 thesis)③引擎主角+自然字号(水平居中、不 zoom)。**S2/S4/S5/S6 铺开挂起,待方向定**(防方向错返工)。

## C2 · 全幕铺开(密度断言)

(未开始)

## C3 · 收口(smoke / 幂等对拍 / 降级同步)

(未开始)

## DoD 对账

(未开始)
