# 「Every Glyph, First-Class.」

> Director's Notes for the **infinite-chat homepage film**(plan43 C0 剧本 = 本文档)
> ~42 s loop + 尾幕停留 · 全屏单 canvas(引擎真渲)+ DOM 字幕/chrome 薄壳 · 60 fps(rAF)
> Director: huashu-design 格式 × infinite-chat film 体系(plan17/41)
> 渲染者:引擎本人(每一帧都是 wgpu 实时渲染,零视频素材)
> Color base: abyss #0A0E14 · 鎏金 #C8AA6E · 骨白 #F0E6D2 · 皇家蓝 #0AC8B9(点缀)· 绯红 #C24A44(仅 diff)
> Type: Cinzel(DOM display)· LXGW WenKai(引擎正文/中文)· JetBrains Mono(引擎代码)
> 音频:**默认静音**(网页 autoplay 约束);SFX/BGM 为预留设计,见 II.6

---

## 目录

- Part I · Director's Statement
- Part II · Visual System
- Part III · Story Arc(完整 timeline)
- Part IV · Shot-by-Shot Storyboard(六镜)
- Part V · Production Manifest(与 plan43 的对接)

---

# Part I · Director's Statement

## 1.1 这不是一支「功能介绍片」,也不是一份聊天记录

上一版首页犯的错(作者亲判"臃肿"):把镜头架在一整场聊天对话上拉来拉去。每一幕都在"展示对话",没有一幕在"讲"。观众看到的是 chat 墙的六个局部。

**这支片不要做这个。** 我们要讲一个故事,故事只有一行:

> **「每个字都是一等图元。」(Every glyph is a first-class primitive.)**

这不是 slogan,是这个引擎的世界观:文字不是 DOM 节点,是 GPU 上的 SDF 实例——所以它能被逐字揭示、逐字调度、逐字飞行,最后**整段话飞起来聚成一朵花,再一个字不少地落回原位**。观众看完只记住这一件事就够了,功能清单是 bonus。

## 1.2 视觉语言的语境对话

- **LoL《Salvation》(2026 For Demacia)**:庄重史诗、克制大气、缓慢推镜与光尘。我们继承它的**节制**——金是点缀不是底色,黑暗给纵深不给压抑。(零 Riot 资产,只对话气质;plan40 P0 铁律沿用。)
- **Apple "Designed by Apple in California"**:慢拍。每幕字幕比阅读速度慢半拍,强迫停留。我们没有旁白,typing 节奏(引擎的 reader 预设)就是我们的旁白。
- **Apple Silicon launch films**:typography 能跳舞。他们的舞者是 "M1" 两个字符;我们的舞者是**全部字符**——这是只有本引擎能拍的镜头(S5)。
- **Kenya Hara《白》→ 我们的「黑」**:abyss 底不是"还没画",是内容本身。每幕 ≥55% 负空间。

## 1.3 观众画像

- **A · 开发者/引擎同行(60%)**:30 秒内必须确信「这是 wgpu 实时渲染,不是视频」→ 所有幕右下角常驻 `LIVE · WGPU` chrome;S3/S4 必须有"视频做不到"的证据(流式骨架先行、点击折叠)。
- **B · 潜在使用者(30%)**:必须知道「它渲染 markdown/对话,可嵌入」→ S3/S4 各给一个真组件,S6 给入口。
- **C · 路人(10%)**:留下「有品位、不像 AI slop」的印象 → II.7 自检表为他们存在。

## 1.4 节奏哲学

慢拍 — 展开 — 蓄力 — **顶峰(S5 bloom)** — 缓收。关键决策:**高潮在第 ~30 秒(花成形+瓣落),不在结尾**;S6 是 resolution 不是 climax。每幕内部也有微曲线:enter 转场(0.45s)→ 内容揭示(reader 节奏)→ 静置呼吸(≥1.5s,给眼睛落点)。

## 1.5 这支片不做的事(反 slop + 反臃肿自检)

| 不做 | 原因 |
|---|---|
| 不把完整对话搬进首页 | 臃肿的定义;完整对话住 /chat/,首页只给 6 秒钩子(并入 S4 字幕深链) |
| 每幕不超过一个焦点 | earn its place;第二个焦点 = 删 |
| 不用紫渐变/emoji 图标/GitHub-dark 蓝底霓虹 | AI slop 三件套 |
| 字幕不超过「eyebrow + ≤12 字标题」 | 内容职责归引擎画面,DOM 只做点题 |
| 效果不叠加演示(dissolve+trail+glow 同屏轰) | 一个效果出现三次就是 slop;每幕最多一种新效果 |
| 幕内零页码/时间码 | chrome(进度点)统一承载 |
| 不用视频/截图冒充实时渲染 | 「LIVE · WGPU」是本片的人格;降级模式才允许媒体,且明示 |

## 1.6 一句话定位

> **"Every Glyph, First-Class."** —— A 42-second live-rendered film where the text itself is the actor.

---

# Part II · Visual System

## 2.1 色板(承 lol-style tokens,出处 spec/reference/lol-style/tokens.md)

```
名称          HEX       作用                        占画面上限
──────────────────────────────────────────────────────────
Abyss        #0A0E14   主底(引擎 clear 色 + DOM 底)  60-75%
Abyss-2      #101822   vignette 深层/卡底            <15%
Bone         #F0E6D2   引擎正文主字色                 15-25%
Gold         #C8AA6E   鎏金:字幕标题/金线/焦点强调    5-8%
Gold-hot     #E8CD8A   高光变体(bloom 花瓣/hover)    <2%
Teal         #0AC8B9   皇家蓝绿:交互态/链接           <3%
Crimson      #C24A44   绯红:仅 diff 删除行(S4)       <3%
Smoke        #8A97A6   次级文本/metadata             <5%
```
铁律:任何一幕不出现表外色;金系合计 <10%;字幕只用 Gold/Bone/Smoke 三色。

## 2.2 字体系统

```
层级            字体                大小       用途(载体)
─────────────────────────────────────────────────────────
Display XL     Cinzel 700          64-88px    S1 主标题(DOM 字幕层)
Eyebrow        JetBrains Mono 700  12px/.22em 每幕 eyebrow(DOM)
Scene title    Cinzel 600          28-36px    S2-S6 幕题(DOM)
引擎 H1/H2     LXGW(引擎 role)     引擎字级    S2 三行大字(引擎渲)
引擎正文       LXGW / JB Mono      引擎字级    S3/S4 组件内容(引擎渲)
Footer         JetBrains Mono 500  11px/.18em chrome/watermark
```
铁律:DOM 只允许 Cinzel + Mono;引擎内容走引擎 role 表,不为片子加特例字级。

## 2.3 构图网格(视口相对,非固定 1920)

- safe zone:上下 10vh、左右 8vw;字幕层锚在下 1/3 线(y=66vh);
- **焦点区**:中央 72vw × 56vh——每幕的引擎焦点内容经 `scroll_to` + 内容宽度设计落入此区,单栏居中;
- chrome 区:右上 nav(品牌+GitHub)、底部中 进度点、右下 `LIVE · WGPU` watermark;
- 负空间 ≥55%/幕(密度断言的另一面,plan43 DoD-3)。

## 2.4 动画系统(全部走既有 token,零新曲线)

```
Easing(motion.rs 既有):enter/exit/expressive + Spring(id5,仅 S5)
Duration 字典:
  幕间转场 dissolve 遮罩       450ms(--t-diss)
  字幕 eyebrow/标题入场        320ms,标题 delay 120ms
  引擎内容揭示                 reader 预设(rhythm token,不改值)
  S4 折叠展开 scale            0.9^(dt/16.7) 指数逼近(plan32 D4)
  S5 聚花/散场                 各 2.6s ease;成花停留 4.4s
  幕内静置呼吸                 ≥1.5s(idle 纪律内)
Stagger 法则:字幕两行 120ms;引擎逐字 stagger 由 rhythm 管,导演不越权。
幕间过渡:统一 dissolve 遮罩(0→0.6→0)盖住 enter 重建帧;无硬切。
```

## 2.5 Chrome 元素(贯穿)

- **A · 底部中 · 进度点 ×6**:当前点鎏金实心,其余 Smoke 空心;可点直跳;自动播时当前点做 4s 极弱呼吸。
- **B · 右上 · nav**:`INFINITE CHAT`(Cinzel 14px)+ GitHub;静态。
- **C · 右下 · watermark**:`LIVE · WGPU`(Mono 10px,rgba bone .32)+ 2px teal 呼吸点(8s 周期)——"这是实时渲染"的人格徽章。
- **D · vignette**:顶 10% 渐晕 + 底 40% 压暗(字幕可读性,plan41 H2 已调)。

## 2.6 音频系统(预留设计;站点默认静音,进度点旁预留 🔇 位,v2 实装)

- BGM 曲线:0-12s 单音钢琴稀疏 → 12-26s 弦乐低垫渐入 → 26-30s swell → 30-36s 顶峰持续 → 36s+ decay 留残响(Max Richter 语域)。
- SFX 字典(实装时):S1 字符 spawn 微 tick(-26dB)/ S3 表格网格浮现 soft bloom(-20)/ S4 折叠 paper flip(-18)/ S5 聚花 ascending sweep(-12)+ 瓣落细碎 rustle(-24)+ 点击绽放 impact(-10)/ S6 sign-off 弱 chime(-18)。频段隔离同 huashu 规范(BGM 低频,SFX 中高频)。

## 2.7 反 slop 自检表(per-shot,执行时逐镜过)

```
□ 表外色 0     □ 焦点 ==1     □ 字幕 ≤12 字     □ 负空间 ≥55%
□ 新效果 ≤1 种 □ 无视频/截图  □ 转场 dissolve   □ 有一处 120% 细节(pause 值得截图)
□ 位置四问已答(见 Part IV 每镜头部)
```

---

# Part III · Story Arc(完整 timeline)

## 3.1 三幕结构

```
ACT I · INVITATION(00.0 – 12.0s)  观众进入;主张被说出
  S1  00.0 – 06.0   TITLE      开场:黑场里长出标题
  S2  06.0 – 12.0   THE CLAIM  三行大字:它是什么

ACT II · PROOF(12.0 – 26.0s)  两个"视频做不到"的证据
  S3  12.0 – 19.0   THE TABLE  流式 markdown:骨架先行,网格先于文字
  S4  19.0 – 26.0   THE CARD   diff 卡:折叠展开,词级高亮

ACT III · MIRACLE & DOOR(26.0 – 42.0s + hold)
  S5  26.0 – 36.0   THE BLOOM  ★ 高潮:整段文字聚成花,瓣落,散场归位
  S6  36.0 – 42.0+  THE DOOR   尾幕:静场 + 四入口(停留,循环回 S1)
```

## 3.2 情绪曲线

```
强度
 │                                        ╔══════╗
 │                                     ╔══╝      ╚═╗
 │                          ╔══════════╝           ╚═╗
 │                ╔═════════╝                        ╚═══╗
 │         ╔══════╝                                      ╚═══════
 │   ╔═════╝
 └───┴────┴────┴────┴────┴────┴────┴────┴────┴────┴────┴────┴──→
     0    4    8   12   16   20   24   28   32   36   40  42s
     │         │         │         │    │         │
   标题现   三行大字   网格先行  折叠展开  聚花!   散场归位→门
                                        PEAK≈30s(成花+首批瓣落)
```

**Emotional beats**:02.5s 标题成形(第一次 awe)· 08s 「每个字都是一等图元」落定(主张)· 14s 表格网格先于文字浮现(第一次 "咦?")· 21s 点击展开折叠(第一次"它是活的")· **28.6s 全屏文字起飞聚花(climax 起点)· 30s 成花+瓣落(PEAK)**· 34s 一字不少落回原位(魔术揭底)· 37s 门打开(resolution)。

---

# Part IV · Shot-by-Shot Storyboard

每镜格式:`[四问]`(叙事角色/观众距离/温度/容量)→ `[TIMECODE|FUNCTION]` `[VISUAL]` `[ENGINE CUES]`(引擎调用序列,可执行)`[SUBTITLE]`(DOM)`[CHROME]` `[ANTI-SLOP]` `[WHY]`。

---

## SHOT S1 · "TITLE"

**[四问]** hero / 1m 笔记本 / 安静庄重 / 标题+一行,thumbnail 无压力
**[TIMECODE]** 00.0–06.0s `|` **FUNCTION** 开场。给黑场一个时间,再让标题长出来。

**[VISUAL]** 全屏 Abyss。0-1.2s 近乎全空(仅 vignette + watermark 呼吸点)。1.2s 起引擎在焦点区渲一行氛围字(Bone,expressive 预设:dissolve-in + 微 glow),内容 = 一行短诗式定位(≤14 字,写死进剧本);DOM 字幕层随后现主标题。**负空间 ~75%,全片最空的一幕——Kenya Hara 的黑。**

**[ENGINE CUES]**
```
enter: slide_set(S1)            // 替换式:画布仅 S1 内容(plan43 架构)
       set_effect_preset("expressive"); scroll_center(S1)
00.0   dissolve 遮罩淡出(450ms)
01.2   restart_reveal()          // 氛围行按 reader 逐字长出
```
**[SUBTITLE]** 02.0s eyebrow `RUST · WASM · WEBGPU · SDF` → 02.3s `INFINITE CHAT`(Cinzel 700, 88px, Gold)→ 03.2s 副行「文字即图元的对话渲染引擎」(Bone, 18px)。
**[CHROME]** 进度点 dot1 亮;nav/watermark 常驻;无其他。
**[ANTI-SLOP]** 无 logo 前置;无粒子迎宾;120% 细节 = 主标题字距(Cinzel -0.02em)与引擎氛围行的基线对齐(DOM 与 canvas 同格)。
**[WHY]** 同 Apple 2013:给空白时间 = 告诉观众"慢下来"。引擎先说一句话再出标题——渲染者先于品牌出场,这是"引擎即宣传片"的开宗明义。

---

## SHOT S2 · "THE CLAIM"

**[四问]** 主张幕 / 投屏级大字 / 笃定 / 三行 ≤60 字
**[TIMECODE]** 06.0–12.0s `|` **FUNCTION** 用引擎自己的字说出世界观。

**[VISUAL]** 焦点区仅三行引擎 H2 级大字(Bone,居中,行间 1.6):
「每个字都是一等图元。」/「GPU 上的 SDF 实例——可揭示、可调度、可飞行。」/「fps 与内存,只随可见的一屏。」
逐行按 reader 节奏揭示(句读呼吸可感)。**这一幕本身就是流式节奏的 demo——不另设 demo。**

**[ENGINE CUES]** `slide_set(S2); set_reveal_preset("reader"); restart_reveal()`;cue@1.5s/3.2s 第二三行(part 分行)。
**[SUBTITLE]** eyebrow「是什么 · WHAT」;无标题(引擎大字即标题——字幕让位)。
**[CHROME]** dot2;其余同。
**[ANTI-SLOP]** 无要点 icon;无三栏卡;120% 细节 = 第三行末句号后 0.4s 的末延长停顿(rhythm 的 finalLengthening,懂的人会心)。
**[WHY]** 主张必须由引擎亲口说——用它的字、它的节奏。观众此刻已在看 demo 而不自知。

---

## SHOT S3 · "THE TABLE"(证据一:结构先于文字)

**[四问]** 单点演示 / 1m / 专注 / 单块表格+一段引言,居中
**[TIMECODE]** 12.0–19.0s `|` **FUNCTION** 展示"视频做不到"之一:流式中骨架先行——网格框先浮现,文字后填入。

**[VISUAL]** 焦点区一块 markdown:一行引言 + 一张 4×3 表格(含对齐/CJK)+ 一行行内公式。关键时刻:**14s 表格网格线先整体浮现(panel reveal),15s 起文字逐格填入**;18s 公式 typeset 落定。单块居中放大(内容宽 ~48vw),周围全空。
**[ENGINE CUES]** `slide_set(S3); set_reveal_preset("reader"); restart_reveal()`(骨架先行是引擎默认行为,导演只管取景)。
**[SUBTITLE]** eyebrow「流式 MARKDOWN」;标题「先骨架,再文字」(≤8 字);17.5s 右下小字幕淡入「全类型 → /markdown/」(teal,深链)。
**[CHROME]** dot3。
**[ANTI-SLOP]** 只此一块,无轮播;120% 细节 = 表格 CJK 列像素对齐(pause 截图可验)。
**[WHY]** ACT II 的第一证据选表格,因为"网格先于文字浮现"是普通 DOM 渲染与视频都给不出的瞬间——它一秒说清 0018+0019 两个 ADR。

---

## SHOT S4 · "THE CARD"(证据二:它是活的)

**[四问]** 单点演示 / 1m / 精密 / 单张 diff 卡
**[TIMECODE]** 19.0–26.0s `|` **FUNCTION** 展示交互性:diff 卡折叠展开 + 词级高亮。

**[VISUAL]** 焦点区一张 edit 工具 diff 卡(gloop 连续圆角行底,绿/绯红,词级高亮,双列行号,「⋯ n unchanged lines」折叠行)。21s **自动触发展开**:被折上下文以 scale 指数动画长出(plan32 D4);23.5s 复叠回。这是全片唯一 Crimson 出场。
**[ENGINE CUES]** `slide_set(S4)`;cue@2.0s `tap_fold(0)` 展开;cue@4.5s `tap_fold(0)` 收起。
**[SUBTITLE]** eyebrow「AGENT 会话卡」;标题「工具与 diff,原生图元」;24.5s 右下「完整对话 → /chat/」(teal,**chat 钩子在此,全片仅此一处**)。
**[CHROME]** dot4。
**[ANTI-SLOP]** 单卡,无 tool 三态并列(砍);Crimson 只在卡内 <3%;120% 细节 = 折叠行 gutter 的胶囊 marker(0.35×行高,Zed 语义)。
**[WHY]** 证据二回答"能交互吗":折叠动画 = 引擎活着的心跳。chat 深链放在这——想看更多的人此刻动机最强。

---

## SHOT S5 · "THE BLOOM" ★ 高潮

**[四问]** 奇观 / 全场 / 敬畏→释然 / 花 + 字幕一行
**[TIMECODE]** 26.0–36.0s `|` **FUNCTION** 全片唯一 120% 时刻:主张的物证。

**[VISUAL]** 26.0s S2 的三行大字重现(密集内容集,花更满)。**28.6s 全部字符起飞**——沿贝塞尔弧、角度桶匹配、外瓣后到,2.6s 聚成一朵 k=5 鎏金玫瑰(Gold-hot);30.0s 成花,**首批花瓣(vesica 叶形)剥离飘落**,每 650ms 一撒;若指针在场,风场把字拨开、松手回弹;点击 = 绽放脉冲 + 40 瓣。33.4s 起 2.6s **散场:每个字沿原路落回原位,一字不少**。
**[ENGINE CUES]** BloomController(plan42 G4,现成):`formation_begin(rose k5) → progress 0→1(2.6s) → 停留 4.4s + petals(18/650ms) → reverse(2.6s) → formation_end`;`set_wind` 由指针驱动。
**[SUBTITLE]** 28.6s「每个字都是一等图元」(Gold,回扣 S2——主张与物证同框);34.5s 淡出。
**[CHROME]** dot5;自动播暂停提示(用户交互时)。
**[ANTI-SLOP]** 全片效果预算的 90% 花在此;花瓣是分析 SDF 不是贴图;120% 细节 = 散场后与起飞前**逐字节同帧**(工程即美学)。
**[WHY]** climax 设在 30s 不在结尾(1.4 节奏哲学);"落回原位"比"聚成花"更重要——魔术的揭底是诚实:你的内容从未被破坏,这就是 presentation/model 分离的世界观表达。

---

## SHOT S6 · "THE DOOR"

**[四问]** 尾幕 / 1m / 平静 / 四卡+页脚
**[TIMECODE]** 36.0–42.0s 后停留(用户不接管则 6s 后循环回 S1)`|` **FUNCTION** resolution + 出口。

**[VISUAL]** 引擎静场(S1 氛围行残留,零活跃动画,idle 归零);DOM 入口浮层浮现:四张鎏金描边卡(chat / markdown / gallery / GitHub)+ 一行页脚(技术栈 + commit)。
**[ENGINE CUES]** `slide_set(S6)`;效果收敛(idle 断言在此可测)。
**[SUBTITLE]** 无 eyebrow;卡片自带文案(各 ≤6 字 + 一行说明)。
**[CHROME]** dot6;进度点旁「↻」提示循环。
**[ANTI-SLOP]** 四卡无 icon 堆砌(文字卡);页脚一行,无 sitemap;120% 细节 = 卡 hover 的 teal 金双色描边过渡(全片唯一 hover 态)。
**[WHY]** 门要安静——climax 已给完,这里只需让人**顺手推门**。停留而非硬循环:尊重读完的人。

---

# Part V · Production Manifest(对接 plan43)

| 项 | 对接 |
|---|---|
| 本文档地位 | = plan43 C0 交付物(幕剧本 + 四问表);C1 起按本文执行 |
| 内容架构 | 替换式 `slide_set()`(plan43 DoD-2);S5 复用 S2 内容集 |
| 文案真值 | 各幕文案以本文 [VISUAL]/[SUBTITLE] 为准,写死进 home-scenes(R8) |
| 时长 token | 各幕时长/转场 450ms/呼吸 ≥1.5s 进 pages-theme token,可调不硬编码 |
| 密度断言 | S1≤20 字(引擎)/S2≤60/S3 单块/S4 单卡/S6 四卡——进 pages-smoke(plan43 DoD-3) |
| 既有资产复用 | S3=showcase 表格段裁剪;S4=feature-manifest diff part;S5=plan42 BloomController 原样;S6=现 outro 层瘦身 |
| 降级 | 无 GPU:六幕同构幻灯(pages-assets);reduced-motion:静帧手动;S5 降级 = fx-dissolve 短片(已有) |
| 音频 | II.6 为预留设计,不在 plan43 范围;实装另立 plan |
| 两个过目点 | ①本文档(C0)②S1+S3 dpr2 截图(C1 定语法)——作者按 II.7 自检表验收 |

> 终帧愿景:一个路人在 42 秒里看到——黑场长出金字、引擎亲口说出主张、表格先骨架后文字、diff 卡自己开合、然后**整段话飞成一朵花又落回原文**、最后一扇安静的门。他记住的只有一句:每个字都是一等图元。
