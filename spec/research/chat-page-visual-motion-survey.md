# 研究:chat 页面视觉与动效 —— 业界 2025-26 现状 + 运动美学数值手册

- 日期:2026-07-06
- 类型:research(业界对照 + 可直接抄的数值/公式)
- 触发:启动「reveal 节奏美感升级 + chat 页面重设计」。本篇补两块既有研究没覆盖的:**① 业界 chat 页面的整体视觉设计**(layout/组件形态/shader 效果语汇);**② 运动设计的节奏 token 数值体系**(时长/缓动/stagger/微定时的具体数字)。
- 与既有 research 的分工:滚动/布局稳定/结构确定性 → [streaming-chat-ux-standards](./streaming-chat-ux-standards.md);感知第一性原理 → [perception-readability-stability](./perception-readability-stability.md);groove/韵律理论 → [reveal-rhythm](./reveal-rhythm.md);agent 数据模型/Zed 对照 → [agent-ui-industry-survey](./agent-ui-industry-survey.md)。**本篇只补"长什么样、动多快"**。
- 产出去向:[design/chat-page-design.md](../design/chat-page-design.md)(设计方案正主)。

---

## 1. 业界 chat 页面 layout 共识(2025-26)

- **三栏收敛**:左会话 rail + 中央消息流 + 可选右 artifact/引用面板;rail <~1200px 折叠为图标。Claude/ChatGPT/Perplexity 均如此。
- **消息列 max-width ≈ 720–768px**:Claude ≈768、ChatGPT ≈768、Perplexity ≈720;超宽则长回答可读性崩。
- **无气泡范式**:严肃 AI 产品(Claude/ChatGPT/Cursor)已放弃 SMS 式圆彩气泡 → **全宽平铺消息 + 微弱背景/缩进区分角色**;user 消息右对齐或仅底色区分。与本项目 plan13「一回合一盒、绝不为气泡而气泡」同向。
- **meta 延迟出现**:时间戳/copy/regenerate 等 per-message 操作只在 Complete 态显示;流式期间唯一常驻控件 = Stop,结束后原位换 Regenerate。
- **流式指示器**:各家光标形态不同(Cursor 细竖条 / Claude 实心小方块 / ChatGPT 脉冲圆点),是最廉价的"alive"信号;首 token 前(200ms–2s)显示脉冲点或单条 shimmer 线,**禁止假进度条**。
- **reasoning 折叠光谱**:ChatGPT 闪动标签 + 留一行可读推理、完成自动折叠;Claude 最小透明度 + 动画图标 + 计时器、展开区独立滚动;Grok 快速滚动片段。共识:**进度指示的价值是"电梯镜子效应"缩短感知等待**,不是全量透明。
- **代码块两遍 pass**:闭合 fence 前按纯文本/无高亮渲染,闭合后再上高亮——Cursor/Claude 都这么做,观感顺滑(与 0017 活跃块整块重解析同构)。
- **tool 卡四档折叠**:full / collapsed / auto-collapsed(最新展开其余折叠)/ hidden(AI SDK 词汇),error 默认展开。

来源:[Setproduct AI chat UI](https://www.setproduct.com/blog/ai-chat-interface-ui-design) · [Digestible UX reasoning 折叠](https://www.digestibleux.com/p/how-ai-models-show-their-reasoning) · [AI SDK Terminal UI](https://ai-sdk.dev/docs/agents/terminal-ui) · [Warp 2.0](https://www.warp.dev/blog/reimagining-coding-agentic-development-environment)

## 2. 文字揭示的业界形态

- 主流 = **逐词/逐块直出 + 光标**,非人为打字机;词级淡入(Gemini 式)有开源复刻 FlowToken:按 word/char 切分 + fadeIn/blurIn/dropIn,典型 0.5–0.6s ease-in-out;**完成的消息把 animation 置空降内存**(= 我们 settled 冻结)。
- Gemini 生成时背景做**流体渐变位移**(深蓝→青绿→琥珀),文字与按钮随后 fade in;Apple Intelligence 是屏幕边缘虹彩 glow 随语音起伏。
- 这与 [streaming-chat-ux-standards §7] 的"block 内容 + 词级 karaoke 装饰"结论一致:**词级 fade 是装饰层,不是阅读模式**。

来源:[FlowToken](https://github.com/Ephibbs/flowtoken) · [Gemini 渐变](https://60fps.design/shots/google-gemini-response-gradient-animation) · [AppleIntelligenceGlowEffect](https://github.com/jacobamobin/AppleIntelligenceGlowEffect)

## 3. 运动节奏 token 数值手册(抄这里)

### 3.1 时长分档(各体系对照)

| 档 | Material 3 | IBM Carbon | Fluent 2 | 用途 |
|---|---|---|---|---|
| 微反馈 | short1–4 = 50–200ms | fast-01 = 70ms / fast-02 = 110ms | ultraFast 50 / faster 100 | 按钮态、hover、字形淡入 |
| 组件过渡 | medium1–4 = 250–400ms | moderate-01 = 150 / moderate-02 = 240ms | fast 150 / normal 200 | 卡片展开、块进场 |
| 大面积 | long1–4 = 450–600ms | slow-01 = 400ms | slow 300 / slower 400 | 面板、骨架、表格生长 |
| 页面级 | extra-long1–4 = 700–1000ms | slow-02 = 700ms | ultraSlow 500 | 背景压暗、场景转换 |

- NN/g:绝大多数 UI 动画应落在 **100–500ms**;hover 反馈 ≤150ms、按压 ≤50ms。
- **进出不对称**:退场比进场短 ~20%(开 250 / 关 200;展开 300 / 收起 250);**进场用 decelerate,退场用 accelerate**。
- 时长应随移动距离/元素尺寸非线性放大(Carbon Motion Generator 思路)。

### 3.2 缓动曲线(可直接进 CurveId 表)

| 名 | bezier | 用途 |
|---|---|---|
| M3 emphasized-decelerate | `(0.05, 0.7, 0.1, 1)` | 进场(峰速进入、静止收尾) |
| M3 emphasized-accelerate | `(0.3, 0, 0.8, 0.15)` | 退场 |
| Carbon productive-entrance | `(0, 0, 0.38, 0.9)` | 高频小元素进场(字形/行) |
| Carbon expressive-standard | `(0.4, 0.14, 0.3, 1)` | 大件表达性过渡(卡片/骨架) |
| Ant 主曲线 | `(0.645, 0.045, 0.355, 1)` | 企业级通用 |

- Carbon 明确**禁用 bounce/stretch 装饰曲线**(productive 场景);spring 用于手势驱动/可打断,**纯状态宣告(文字揭示)用 duration+easing 更稳**。与 0025「弹性用解析阻尼当一条 curve、不引状态」一致。

### 3.3 stagger / 编排

- Material choreography:密集列表 stagger **≤20ms/项**;通用惯例 **25–50ms**;强调顺序语义 50–100ms;总时长要封顶(元素多则衰减间隔)。
- M2 speed:2–5 个对象的序列 300–400ms 总长;6+ 对象/页面级 500–700ms。
- **同屏活跃动画组 ≤4**(MOT 多目标追踪上限;详见 perception 篇)——正在动的字应聚成 ≤4 个感知组(词/行级),不是 40 个 glyph 各自为政。

### 3.4 微定时 / 呼吸(接 reveal-rhythm §2 的数值化)

- **1/f 微偏差公式**:`delay_i = i·Δ + ε_i`,ε 取 1/f(粉)噪声、**σ ≈ 0.2Δ、钳制 ±50ms**(groove 研究:±50ms 内的微时序是"人味"来源;系统性硬凹偏移反降 groove)。seeded(R8)。
- **呼吸/待机脉动**:平静呼吸 ≈ 0.1Hz;UI 呼吸取 **0.1–0.25Hz 正弦、幅度 ≤5%**(alpha ±3–5%、scale ±1–2%)。持续脉动**避开 3–30Hz 光敏带**;WCAG 2.3.1:任何 1s 内闪烁 ≤3 次。
- **阅读速率匹配**:注视 200–250ms + 眼跳 20–40ms;推荐**词级揭示间隔 150–250ms(≈4–7 词/s ≈ 240–420wpm)**,恰好一个注视周期;chunk 内字符 stagger 20–35ms;单字符效果时长 ≤200ms(一次注视内完成,免拖尾过眼跳)。

来源:[M3 motion tokens](https://m3.material.io/styles/motion/easing-and-duration/tokens-specs) · [Carbon motion](https://carbondesignsystem.com/elements/motion/overview/) · [Fluent 2 motion](https://fluent2.microsoft.design/motion) · [NN/g duration](https://www.nngroup.com/articles/animation-duration/) · [不对称时长](https://developers.google.com/web/fundamentals/design-and-ux/animations/asymmetric-animation-timing) · [motion.dev stagger](https://motion.dev/docs/stagger) · [1/f 节拍偏差](https://pmc.ncbi.nlm.nih.gov/articles/PMC6934603/) · [MOT 上限](https://jov.arvojournals.org/article.aspx?articleid=2121950) · [RSVP 速率](https://elvers.us/perception/rsvp/) · [WCAG 动画](https://www.w3.org/WAI/WCAG21/Understanding/animation-from-interactions.html)

## 4. shader 驱动 UI 的效果语汇(可迁到 SDF 组件)

| 语汇 | 先例 | 配方 | 本项目落点 |
|---|---|---|---|
| **SDF 圆角/阴影全 UI** | Zed GPUI(120fps 出货) | 圆角矩形 SDF + erf 高斯阴影近似 | 已同构(0018 panel);佐证路线 |
| **FBM 流体渐变** | Stripe / Gemini 生成背景 | 多倍频 simplex FBM + sin/cos 调制 UV + fragment 内 blend mode | thinking/streaming 活性材质(ShaderBox) |
| **边缘虹彩 glow** | Apple Intelligence / Siri | 屏幕/卡片边缘沿 SDF 距离的动态渐变发光,随活动起伏 | running tool 卡描边、活跃 turn 指示 |
| **lensing / specular** | Apple Liquid Glass(WWDC25) | 实时折射底层内容 + 高光响应运动 + 交互瞬间 flex | 高端档:hover/press 的面板高光(参数化,不引纹理回读) |
| **mask reveal / wipe** | Codrops GSAP×shader 惯用 | uniform progress 驱动 SDF mask 揭示 | 骨架先行的"框长出" + 块级 wipe 入场 |
| **词级 fade/blur-in** | Gemini / FlowToken | per-word 0.5–0.6s 淡入(可 blur) | glyph enter profile 的词级分组版 |
| **metaball/smin 融合** | 通用 SDF 语汇 | `op_smin` 连接相邻形体 | 气泡分组融合、装饰接合(Plan 10 相位 4) |

来源:[Zed videogame](https://zed.dev/blog/videogame) · [alexharri WebGL gradient 解构](https://alexharri.com/blog/webgl-gradients) · [Liquid Glass](https://developer.apple.com/videos/play/wwdc2025/219/) · [Codrops shader reveal](https://tympanus.net/codrops/2025/10/08/how-to-animate-webgl-shaders-with-gsap-ripples-reveals-and-dynamic-blur-effects/)

## 5. 对本项目的定调结论(6 条)

1. **layout 抄收敛范式**:内容列 ~768 世界单位 max-width、无气泡、角色靠对齐+弱底区分——plan13 方向正确,差的是 spacing/密度/meta 的细节收口。
2. **节奏 token 化**:把 §3 数值收进一张 `MotionTokens` 表(duration 档 × ease 档 × stagger 档),所有组件动画只引用 token,不写裸数——与 0021「样式=数据」同构,节奏后续可整表调。
3. **词级是装饰、块级是内容**(既有结论再确认),且词间隔 150–250ms、字符 stagger 20–35ms、单字符 ≤200ms 有生理依据可直接当默认值。
4. **"有规律的重复"落地 = 结构性停顿(大杠杆)+ 1/f 微偏差(σ=0.2Δ、±50ms、seeded)+ 呼吸 0.1–0.25Hz ≤5%(待机层)**;同屏动画组 ≤4。
5. **shader 语汇优先级**:① 骨架 mask reveal(闭合律 + 已有 panel 参数即可)② running 态边缘 glow/shimmer(辨识度高、面积小)③ thinking 区 FBM 活性材质(ShaderBox 现成)④ lensing 高光(高端档,后置)。都符合 0028 五护栏(小面积/可冻结/慢时钟)。
6. **稳定性没有新东西**——业界三件套(滚动三态/防闪/防 shift)既有 research 已覆盖且本引擎机制侧已实现;chat 页面重设计**不必再动稳定性地基,只动观感层**。
