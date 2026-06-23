# infinite-chat · 产品介绍片 — 导演表(讨论稿 v0.1)

> infinite-chat 介绍页 / launch film 的 Director's Notes
> 单页 · 引擎自驱播放 · 底部视频播放器进度条(自动播放 / 暂停 / 调速)
> 体裁:**多幕剧**,每幕一个能力(按二级分类),以一个**真实 chat 页面**收尾
> 不是功能罗列,是**世界观叙事**:把"对话=新的操作系统入口"讲出来
> 参考语境:huashu-design `launch-film-30s-sample.md`(借其格律,不借其调性)
>
> ⚠️ 讨论稿:幕序、侧重、slogan、收尾都待你拍板;路线图未实现项已标 `[ROADMAP]`。

## 0. 决议 v0.2(已拍板)

1. **规模前置作第一记重拳** + 2. **规模作高潮** → 规模**双现**:开场后先给一记"惊鸿一瞥"(快速拉远预告),中段构建,结尾**全爆发**作情绪峰值(经典 bookend)。
3. **slogan = 「一条永不结束的对话」**(+ `infinite-chat.`)。
4. **路线图项全提上**作"即将到来"预告(glow-orb / 3D·raymarch / 多实例 / a11y 镜像)。
5. **节奏优先 ≈ 40s**,每幕更狠(4–5s/幕)。
6. **无 BGM / SFX**(纯视觉 + 引擎自驱)。

**canonical 幕序(v0.2,9 拍 ~40s)**:
`0 开场caret` → `1 规模·惊鸿`(首拳)→ `2 流式` → `3 SDF+数学` → `4 富内容` → `5 特效+路线图预告` → `6 容错/弱网` → `7 规模·全爆发`(**高潮**)→ `8 收尾 chat + slogan`。
逐幕分镜(Part IV)内容沿用,按此新序重排;精确 timing 见 [plan17 实现](../plan/plan17-intro-film.md)。

---

## 目录

- [Part I · 导演陈述](#part-i--导演陈述)
- [Part II · 视觉系统](#part-ii--视觉系统)
- [Part III · 故事弧 / 幕结构](#part-iii--故事弧--幕结构)
- [Part IV · 逐幕分镜](#part-iv--逐幕分镜)
- [Part V · 播放器 / 进度条](#part-v--播放器--进度条)
- [Part VI · 制作清单(引擎钩子 · 已上线 vs 路线图)](#part-vi--制作清单)

---

# Part I · 导演陈述

## 1.1 这不是一支"功能介绍片"

绝大多数渲染库的介绍页都长一个样:一段 hero 标题 → 几个 feature 卡片滑过 → "npm install" → 结束。它在**列功能**,没有一秒在**讲为什么**。

infinite-chat 的介绍片要讲一个判断。判断只有一句:

> **「对话,是 LLM 时代新的操作系统入口。它配得上一个真正的引擎。」**

这不是 slogan,是世界观。现在的 chat 界面基本都是垃圾——逼你一个任务开一条新会话、长会话越用越卡、富内容渲染得又丑又糙、流式吐字像打嗝。**它们配不上背后那个正在改变世界的模型。** 这支片要让观众在 60 秒内信一件事:**一条对话可以永不结束、无比流畅、效果上限拉满**——因为它底下是一个游戏引擎。

如果观众看完只记住一件事,我要的是:「原来 chat 可以做成这样」。功能能记多少都是 bonus。

## 1.2 视觉语言的语境对话

- **Apple Silicon launch films**:typography 会跳舞。我们的 hero 不是 logo,是**一条正在生长的对话流**——它在 60 秒里缩放、冻结、断流重连、收束成一屏真实 chat。
- **黑色而非象牙白**:huashu 那支片是出版社的白;我们是**深空里的一块活画布**。infinite-chat 天生在暗底上发光(SDF 文字自带锐利),这是它的母语,不强行套白底。
- **反 slop**:不用紫渐变、不用 emoji 作图标、不堆 glow/particle、不画 SVG 人物。克制是品味。accent 青只在该亮时亮。

## 1.3 hero 是什么

**hero = 一条无限会话(infinite session)本身**。它有四个反复出现的"识别记号":

1. 一个**闪烁的 caret**(写作的最小单位,贯穿全片);
2. 文字**逐字涌现**的节奏感(流式);
3. 一抹**青色 accent**(`#3DF5D0`,只作点睛);
4. 一条**永远向下、可无限缩放**的 thread(无边画布)。

这四样在 12+ 个镜头里有 12 种状态,但内核不变——就像 Apple 让 "M1" 三个字符成为舞剧主角。

## 1.4 观众画像(按重要性)

- **A · 做 AI 产品 / agent 的工程师与设计师(主受众)**:他们在为"对话界面"头疼(长会话卡、富内容糙、流式难看)。承诺:60 秒内你会想"这个我能直接 import 进 React/Vue"。
- **B · 关注前端/图形/渲染的技术人**:他们想看"游戏引擎手法做 UI"到底是什么。承诺:看到 SDF 无限缩放 + 块冻结 + 弱网重连这种**别处看不到的硬货**。
- **C · 路过的设计/产品人**:留下"这家有审美、跟别的 AI 工具不一样"的印象。整套反 slop 自检为他们做。

## 1.5 节奏哲学

非匀速。**慢拍 — 加速 — 顶峰 — 收束**:

- **0–6s 慢拍**:空画布 + caret,给"空"一个时间,告诉观众"慢下来"。
- **6–46s 能力幕**:按二级分类一幕一能力,一气呵成,每幕不松手。中段(规模/特效)是加速带。
- **46–52s 顶峰**:断网→重连→刷新不丢历史(容错),情绪最紧,然后**化解**。
- **52–60s 收束**:镜头拉远成一屏真实 chat,所有能力同时在场,slogan 落地,印章。

**高潮在容错那一幕(≈48s),不在最后**。最后是 resolution。

## 1.6 一句话定位

> **infinite-chat —— 一条永不结束的对话,配得上 LLM 时代的主入口。**

---

# Part II · 视觉系统

## 2.1 色板(暗底,克制)

| 名称 | HEX | 作用 | 上限占比 |
|---|---|---|---|
| Ink | `#0D0F17` | 主底(深空蓝黑) | 60–70% |
| Indigo | `#1A1040` | 次级底 / 面板 | <15% |
| Slate | `#222838` | 分隔线 / 卡片边 | <5% |
| Smoke | `#7C8499` | 次级文本 / metadata | <8% |
| Paper | `#E8EAF2` | 主文本(近白,不刺眼) | 20–25% |
| Cyan | `#3DF5D0` | 主 accent(caret / 高光 / 当前幕) | 5–8% |
| Cyan-Hot | `#7BFFE6` | 高光瞬时(NEW / 命中一瞬) | <1% |

**铁律**:全片不出现以上之外的色;青系合计 <8%;任何文本只用 Paper / Smoke / Cyan 之一。**禁紫渐变**。

## 2.2 字体系统

- 正文 / 标题:引擎的 **SDF 文字**(系统 sans 栈烘 TinySDF;英文可走 MSDF)。卖点本身就是"字是 SDF 图元",所以**就用引擎自己渲染**,不另套 web font 做画面文字。
- 界面 chrome(进度条 / 幕标 / 计数):**JetBrains Mono**(等宽,letter-spacing 0.18em),与画布内容区分。
- 数学:引擎 **KaTeX MSDF**(LaTeX 一等公民,Plan 12)。
- caret:3px 宽 block,Cyan,贯穿全片的签名。

## 2.3 网格与画布

- 画布 1920×1080,安全边 80px。
- 画面分两层:**世界层**(引擎画布,内容在世界坐标,可缩放平移)+ **chrome 层**(DOM 叠加,固定屏幕坐标:幕标、进度条、计数)。对应引擎 0022 DOM 叠加层。
- hero thread 默认锚在左 1/3 栏(user/assistant 左右分栏走 Plan 13)。

## 2.4 动效与"引擎自驱"原则

**关键**:这支片**不是后期 AE 合成的视频**,是**引擎实时驱动**的——导演表里每个镜头都对应一组 `ChatCanvas` API 调用(见 Part VI)。这样:① 介绍片本身就是"产品在跑",最高可信度;② 进度条 seek = 引擎 `seek_reveal` / 重放;③ 调速 = `set_reveal_cps/slow`。

Easing 仅 4 条:`expoOut`(默认入场)、`overshoot`(accent 弹出)、`linear`(底纹/相机匀速)、`expoIn`(退场)。幕间过渡 = **相机运镜 + cross-dissolve**,不硬切。

## 2.5 Chrome 元素(贯穿)

- **A · 左上 幕计数**:`ACT 02 · 流式` + 小圆点进度(当前幕 Cyan 实心)。Mono 12px。
- **B · 右上 版本签**:`● INFINITE-CHAT · webgpu`,Cyan dot 极弱呼吸。
- **C · 底部 播放器进度条**(见 Part V):这是本片**唯一**允许长驻的底部 chrome(因为产品需求明确要它)。
- **D · 右下 watermark**:`OhBonsai / infinite-chat`,rgba 0.3,静态。

## 2.6 音频(可选)

无旁白。BGM 走 minimal-cinematic(Max Richter register):room tone + piano 单音累积(0–8s)→ 弦乐铺底(能力幕)→ pulse 加入(规模/特效)→ swell 到容错顶峰(48s)→ decay 收在 chat 一屏。SFX:keypress、whoosh、单 chime 切幕、断网"glitch"一瞬、重连"resolve"和弦。**频段隔离**防打架。

## 2.7 反 AI slop 自检(per-shot)

```
□ 无紫渐变 / 无霓虹赛博
□ 无 emoji 作图标
□ 无 SVG 人物 / 抽象人形
□ 无色板外颜色;青 accent <8%
□ chrome 用 Mono,内容用引擎 SDF;不混
□ 不堆 effects(同一特效出现 ≥3 次即装饰)
□ 假文用真能读的内容(产品 hook 句),不用 Lorem ipsum
□ 每镜有一处"暂停值得截图"的 120% 细节
□ 幕间是运镜+dissolve,不硬切
□ 本镜对应的引擎 API 真能驱动(已上线 ✓ / 路线图标注)
```

---

# Part III · 故事弧 / 幕结构

按**二级分类**,一幕一能力。总时长目标 ~60s(进度条可调速,内容时长自适应)。

| 幕 | 二级分类 | 时间 | 一句卖点 | 状态 |
|---|---|---|---|---|
| **ACT 0** | 开场 | 0–6s | 一个 caret,一条对话的诞生 | ✓ |
| **ACT I** | 流式 Streaming | 6–16s | 字随节奏长出来,不是打嗝 | ✓ |
| **ACT II** | SDF 文字 | 16–24s | 无限缩放,永远锐利;数学也是一等公民 | ✓ |
| **ACT III** | 无限画布 / 规模 | 24–34s | 100+ 轮、上万行,只画"看得见的一屏" | ✓ |
| **ACT IV** | 富内容 Rich | 34–42s | 代码 / 表格 / 公式 / 图,渲染得好看 | ✓ |
| **ACT V** | 特效 Effects | 42–48s | 程序化 SDF 动效 + 搞怪猫分割线 + agent 发光身份 | ✓ / `[ROADMAP]` |
| **ACT VI** | 容错 / 弱网 | 48–53s | 断网丢包不慌,刷新不丢历史(**高潮**) | ✓ |
| **ACT VII** | 收尾 · Chat | 53–60s | 镜头拉远,一屏真实对话,所有能力同时在场 | ✓ |

**情绪曲线**:0–6 平 → 6–34 稳步上 → 34–42 富内容小高原 → 42–48 特效兴奋 → 48 容错**顶峰** → 53–60 收束 resolution。

**幕序待议**:可把"无限画布/规模(ACT III)"前置作第一记重拳(它是项目最强卖点),或保留在中段做加速带。讨论点①。

---

# Part IV · 逐幕分镜

镜头格式:`[TIME] | FUNCTION` · **[VISUAL]** · **[ENGINE]**(驱动 API)· **[ANIM]** · **[AUDIO]** · **[CHROME]** · **[WHY]**。

---

## ACT 0 · "空画布 · caret"

**[0.0–3.0s] | FUNCTION** 开场,给"空"一个时间。
**[VISUAL]** 全屏 Ink `#0D0F17`,极淡星点/网格底纹(opacity ≤0.04)。画面中央偏左一个 Cyan caret 开始闪烁(macOS 终端节奏,0.7s 周期)。
**[ENGINE]** 空 `ChatCanvas`;caret = 一个 ShaderBox 小图元(plan16)或内容光标。
**[ANIM]** 底纹 0→0.04(500ms linear);caret 闪两次。
**[AUDIO]** room tone 进;piano 第一音。
**[CHROME]** 全隐藏。
**[WHY]** caret 是写作的最小单位,是 hero 的种子。给白(黑)一个时间 = 告诉观众慢下来。

**[3.0–6.0s] | FUNCTION** 第一条消息诞生。
**[VISUAL]** caret 后逐字打出一句 user 提问(真能读:"帮我把这份会议纪要整理成周报")。打完,assistant 侧开始"思考中"——一个 **agent 发光环**(glow-orb)亮起脉冲。
**[ENGINE]** 走真实 transport / replay 喂第一条;assistant 头像位 glow-orb(plan16 §2.6 `[ROADMAP]`,未实现则用 caret 脉冲替代)。
**[ANIM]** 逐字 stagger 50ms;glow-orb fade-in + 加速脉冲(streaming busy)。
**[CHROME]** B 版本签淡入。
**[WHY]** 建立"这是一条真实对话"的前提,后面所有能力都长在这条对话上。

---

## ACT I · 流式 Streaming(6–16s)

**[6–11s] | 平滑吐字 + reveal 节奏** assistant 回复开始流式涌现。镜头**同一句话,用三种节奏重放**:慢打字(顿挫)→ 匀速平滑 → 快涌。展示"字随节奏长出来,不是打嗝"。
**[ENGINE]** `set_reveal_cps(9→…)` / `set_reveal_slow(0.4)` / `restart_reveal()`(已上线 ✓,Plan 8/0019)。平滑器=蓄水池匀速(0002 §6)。
**[VISUAL]** 文字逐字 fade(GPU `spawn_time`,零 CPU);Cyan caret 跟在前沿。
**[WHY]** 流式正确性是一等公民——这是 Rust 核心最不可替代的价值。

**[11–16s] | 形变 morph + remend** 流到一半,故意露出"半截语法"(如 `**粗`),引擎 **remend** 主动补全防闪烁;一个词从一种字形 **morph** 成另一种(past→current 双关键帧)。
**[ENGINE]** 0016 形变 + 0017 提交前沿/和解 + remend(0002/README §6)(已上线 ✓)。
**[WHY]** 别人流式渲染会闪、会跳;我们**不闪不跳**——这是"暂停值得截图"的硬细节。

---

## ACT II · SDF 文字(16–24s)

**[16–21s] | 无限缩放锐利** 镜头对准 assistant 回复里的一个词,**相机推近 8×、16×**——文字边缘**始终锐利**(SDF 本性),不糊。再拉回。旁边浮一行 Mono 小字:`SDF primitive · 任意缩放清晰`。
**[ENGINE]** `zoom_at(factor, sx, sy)` 对称推拉(已上线 ✓);文字=SDF 图元(0011),英文 MSDF(0015)。
**[WHY]** 这是 DOM/位图做不到的。一镜讲清"为什么是 GPU 引擎,不是又一个 react 组件"。

**[21–24s] | 数学一等公民** 对话里出现一条公式 $\sum_{i=1}^{n} i = \frac{n(n+1)}{2}$,随推近**同样锐利**(KaTeX MSDF)。
**[ENGINE]** Plan 12 数学 SDF(已上线 ✓)。
**[WHY]** 富内容不是贴图——公式也是可无限缩放的一等 SDF 公民。

---

## ACT III · 无限画布 / 规模(24–34s)

**[24–30s] | 100+ 轮不卡** 镜头**急速向上拉远**:一条对话从一屏,变成 10 轮、50 轮、**100+ 轮、上万行**的长河,飞速滚过,**fps 计数器(右上 debug)纹丝不动**。
**[ENGINE]** 视口裁剪 + 块冻结(settled)+ GPU 实例化(Plan 2/3,已上线 ✓);`?debug` 帧统计真显 fps。
**[VISUAL]** 远观时文字成"纹理密度",有山河感;Cyan 标出"当前可见的一屏"。
**[WHY]** **项目最强卖点**:fps/内存只与"可见一屏"成正比,与历史总量无关。DOM 会节点爆炸,我们不会。

**[30–34s] | 无边画布 · 块冻结** 在长河里自由平移/缩放,**已读区块冻结不重排**(settled),只有尾部在生长。用一道光扫过,冻结的块是"实心",活跃尾块在"呼吸"。
**[ENGINE]** `pan_by` + 块冻结(0001/0002,已上线 ✓)。
**[WHY]** 无限会话(infinite session)= 本项目的存在理由。这一幕是它的视觉证明。

---

## ACT IV · 富内容 Rich(34–42s)

**[34–37s] | 代码块视口** 一个代码块出现:**行窗(maxHeight)+ 边缘淡入淡出 + 双向滚动 + 行号 + 复制图标**(copy→✓ morph)。镜头滚动代码、点 copy 变对勾。
**[ENGINE]** Plan 15 / 0027(已上线 ✓);copy→✓ = shaderbox `mix` morph(plan16)。
**[37–39.5s] | 表格对齐** 一张表两趟布局列对齐,逐行流入(行框揭示风格)。
**[ENGINE]** 0014 两趟布局 + Plan 8B 表格揭示(已上线 ✓)。
**[39.5–42s] | 图片 / 嵌入** 一张图片嵌入(textured quad,浏览器解码),随相机平移**跟手**;旁边一个 mermaid/SVG 占位 `[ROADMAP/部分]`。
**[ENGINE]** Plan 14 图片(✓);mermaid 浏览器光栅(0007,部分)。
**[WHY]** 富内容渲染得**好看**,不是"又丑又糙"。每一种都是真实可读的迷你内容,不是 bar lines。

---

## ACT V · 特效 Effects(42–48s)

**[42–44.5s] | shaderbox 程序化 SDF** 工具调用流里弹出一排**程序化动态图标**(tool icons:read/grep/shell…)+ 装饰动效,全是 SDF 片元、缩放锐利。
**[ENGINE]** Plan 16 ShaderBox + icons(部分上线;tool-icons 已设计 ✓,引擎接线进行中)。
**[44.5–46.5s] | 搞怪猫分割线** 一条 `---` 渲染成**会动的猫**(rule_cat,smin 融合),"喵"一下——全片唯一的俏皮瞬间。
**[ENGINE]** `rule_cat.wgsl`(已上线 ✓)。
**[46.5–48s] | agent 发光身份 + 3D 预告** assistant 的 glow-orb 身份环呼吸;镜头一翻,露出**raymarch SDF / 3D 相机**的一角作"即将到来"预告。
**[ENGINE]** glow-orb(plan16 §2.6 `[ROADMAP]`)、3D 相机 + raymarch(0024 `[ROADMAP]`)。
**[WHY]** "效果上限拉满"。特效是数据不是分支(0002 §5.1)——可插件、可热加载。3D 一角给未来留钩子。

---

## ACT VI · 容错 / 弱网(48–53s)·**高潮**

**[48–50.5s] | 断网不慌** 流式吐到一半,**网断了**(画面顶一道极弱 glitch,"OFFLINE"红点)。但已上屏的内容**纹丝不动、不闪不丢**;尾部进入 Loading 态,不是崩。
**[ENGINE]** 容错状态机 + 平滑器停吐不回滚(0003,已上线 ✓)。
**[50.5–53s] | 重连 · 刷新不丢历史** EventSource **自动重连**,从快照 `resync` 续上,缺的 delta 补齐,**确定性重放**对账;模拟一次"刷新页面"——历史**完整回来**。一记 resolve 和弦。
**[ENGINE]** catch-up/live 双模 + `resync_from_snapshot` + 幂等快照(0003,已上线 ✓)。
**[WHY]** 这是高潮:**弱网不丢不错、刷新不丢历史**。别人做不到的可靠性,在最紧的时刻化解——情绪峰值。

---

## ACT VII · 收尾 · Chat(53–60s)

**[53–57s] | 一屏真实对话** 镜头从特写**缓缓拉远**,前面所有片段收束成**一屏完整、真实、好看的 chat**:user 左、assistant 右(Plan 13 分栏),流式尾巴还在轻轻跳,代码块/公式/图安静各就各位,角落 glow-orb 呼吸。
**[ENGINE]** 真实 chat 布局(Plan 13,已上线 ✓);可接真 server(`?server=`)或 replay。
**[57–60s] | slogan + 印章** 画面静下来,中央偏下浮出 slogan(引擎 SDF 大字,Cyan 句点):

> **一条永不结束的对话。**
> **infinite-chat.**

下方一行 Mono 极小:`Rust · WebAssembly · WebGPU · React/Vue import`。最后一秒只剩 caret 在 slogan 末尾闪一下,piano 残响。
**[WHY]** 不回到"功能",回到**那条对话本身**。所有能力的意义,就是让这一屏对话**永不结束、始终丝滑**。

---

# Part V · 播放器 / 进度条

底部常驻一条**视频播放器式控制条**(DOM chrome,固定屏幕坐标):

```
 ▶/⏸   ●━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━○   0:23 / 1:00    速度 0.5× 1× 2×
        │   │    │     │     │    │   │  │
        0   I    II    III   IV   V   VI VII   ← 幕 chapter 刻度(hover 显幕名)
```

**交互**:
- **自动播放**:进页即从 ACT 0 开始(可配 `?autoplay=0` 关)。
- **暂停 / 播放**:`set_paused(true/false)`(已上线 ✓);暂停时画面停在当前帧,符合"每镜暂停值得截图"。
- **调速**:0.5× / 1× / 2× → 映射到 `set_reveal_cps`/`set_reveal_slow` + 导演时钟缩放(流式揭示随之快慢)。
- **拖动进度条 seek**:跳到某幕 → `seek_reveal(target_ms)` + 相机/内容状态跳转(已上线 ✓ seek_reveal;跨幕状态跳转需导演层记录每幕入点)。
- **幕 chapter 刻度**:7 段,点击直达该幕;当前幕 Cyan。
- **不循环**(按你之前定的:reel 不循环);播完停在收尾 chat 一屏,可手动 replay 或拖回。

**实现层**:一个"导演时钟"(JS)按导演表的时间轴推进,在每个镜头入点调对应 `ChatCanvas` API;进度条 = 导演时钟的 UI;seek = 导演时钟跳点 + 重建该点引擎状态。

---

# Part VI · 制作清单(引擎钩子 · 已上线 vs 路线图)

| 幕 | 能力 | 引擎 API / 出处 | 状态 |
|---|---|---|---|
| 0 | caret / 首条消息 | replay/transport;ShaderBox caret | ✓ |
| 0 | agent glow-orb 身份 | plan16 §2.6 | `[ROADMAP]` |
| I | reveal 节奏/编排 | `set_reveal_cps/slow` `restart_reveal`(0019/Plan8) | ✓ |
| I | 平滑吐字 | 蓄水池平滑器(0002 §6) | ✓ |
| I | 形变 morph + remend | 0016/0017 + remend | ✓ |
| II | 无限缩放锐利 | `zoom_at`;SDF 图元(0011)/MSDF(0015) | ✓ |
| II | 数学一等公民 | KaTeX MSDF(Plan 12/0013) | ✓ |
| III | 100+ 轮不卡 | 视口裁剪+块冻结+实例化(Plan2/3) | ✓ |
| III | 无边画布平移 | `pan_by`(0001) | ✓ |
| III | fps 可观测 | `?debug` 帧统计(0012) | ✓ |
| IV | 代码块视口 | Plan15/0027;copy→✓ morph | ✓ |
| IV | 表格两趟布局 | 0014;表格揭示 Plan8B | ✓ |
| IV | 图片嵌入 | Plan14(textured quad) | ✓ |
| IV | mermaid/SVG 嵌入 | 0007(浏览器光栅) | 部分 |
| V | shaderbox 程序化动效 | Plan16/0028 | 进行中 |
| V | tool icons / deck | plan16-tool-icons / §2.5 | 设计✓·接线中 |
| V | 猫分割线 | `rule_cat.wgsl` | ✓ |
| V | 3D 相机 / raymarch | 0024 | `[ROADMAP]` |
| VI | 弱网容错 / 重连 | 0003 容错状态机 | ✓ |
| VI | 刷新不丢历史 | `resync_from_snapshot`/快照(0003) | ✓ |
| VII | 真实 chat 分栏 | Plan13(Taffy 0023) | ✓ |
| VII | DOM 叠加 chrome | 0022 | ✓ |
| — | 多实例同步 | 0008 | `[ROADMAP]` |
| — | 无障碍 DOM 镜像 | README 原则 9 | `[ROADMAP]` |

---

# 讨论点(等你拍板)

1. **幕序**:是否把 ACT III(无限画布/规模)前置为第一记重拳?它是最强卖点。
2. **高潮位**:容错(ACT VI)作高潮 vs 规模(ACT III)作高潮——选哪个做情绪峰值?
3. **slogan 定稿**:「一条永不结束的对话」/「对话是新的操作系统入口」/「配得上那个正在改变世界的模型」/ 英文版?
4. **路线图项**:glow-orb、3D/raymarch、多实例、a11y 镜像——**作"即将到来"预告秀出来**,还是这版完全不提、只展已上线?
5. **时长**:60s 是否合适?可压到 40s(每幕更狠)或放到 90s(每能力更从容)。
6. **音频**:要不要 BGM/SFX(影响"片"的质感,但也增加实现成本)?
