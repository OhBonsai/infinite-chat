# 设计方案:chat 页面重设计 —— SDF/shader 组件 × 节奏系统 × 交互 × layout

> **Plan 28 注记(2026-07-08)**:§2/§4 的观感规格已由 [plan28 参考真值包](../reference/opencode-ui/NOTES.md)(opencode desktop 对拍)**取代**;§3 节奏系统仍有效(作为超越参考的增强层保留)。

- 日期:2026-07-06
- 状态:**设计方案(待评审 → 拆 plan 落地)**
- 触发(作者):现 chat 页面观感"拉跨"——匀速逐字揭示、装饰平淡、无节奏无生命感。启动「reveal 节奏美感升级」,要求给出结合 SDF/shader 优势的**完整页面设计**:组件的 shader、animation、交互、layout、元素;**节奏参数全部可后调**。
- 调研地基:[chat-page-visual-motion-survey](../research/chat-page-visual-motion-survey.md)(业界形态 + 数值手册,本篇数值来源)· [reveal-rhythm](../research/reveal-rhythm.md)(节奏理论)· [perception-readability-stability](../research/perception-readability-stability.md)(感知第一性原理)· [streaming-chat-ux-standards](../research/streaming-chat-ux-standards.md)(稳定性/粒度地基)· [agent-ui-industry-survey](../research/agent-ui-industry-survey.md)(Zed/opencode 对照)
- 机制地基(全部已有,本篇只做 policy/数据):[0016 morph](../decision/0016-streaming-morph-render-model.md) · [0018 SDF 面板](../decision/0018-sdf-panel-decoration-primitive.md) · [0019 gate×choreography](../decision/0019-reveal-gating-and-choreography.md) · [0020 节点树](../decision/0020-content-node-identity-model.md) · [0025 SDF 动画](../decision/0025-sdf-node-animation-system.md) · [0026 widget](../decision/0026-modular-shader-organization.md) · [0027 代码视口](../decision/0027-code-block-viewport.md) · [0028 ShaderBox](../decision/0028-shaderbox-primitive.md) · [0030/0032 交互三层](../decision/0032-interaction-architecture-sdf-world-not-component-framework.md) · [0033 part 渲染契约](../decision/0033-part-render-contract.md) · [Plan 8 调度器](../plan/plan8-reveal-cadence.md) / [Plan 10 anim 相位](../plan/plan10-sdf-animation-system.md) / [Plan 13 盒布局](../plan/plan13-chat-box-layout.md) / [Plan 23 part 渲染](../plan/plan23-part-render-implementation.md)

---

## 0. 一句话 + 现状诊断

**一句话**:机制(0016/0018/0019/0025/0028)已全部到位,页面"拉跨"不是缺机制,是缺**一层成体系的 policy**——把零散默认值升级为「**MotionTokens 节奏体系 + 每组件的 shader/动画/交互规格**」,全部数据驱动、整表可调。

现状差距(对照调研 §5 定调):

| 维度 | 现状 | 差距 |
|---|---|---|
| 节奏 | `RevealScheduler` 匀速 `reveal_cps` + `slow` | 无边界停顿/重音/微定时 = "机械打字机"(reveal-rhythm 点名的反例) |
| 揭示粒度 | 逐 glyph 配额 | 应为「块内容 + 词级装饰」双层(streaming-ux §7) |
| 组件观感 | tool/reasoning 卡已有底+徽章(Plan 23),但静态平面 | 无 running 生命感、无骨架先行入场、无 hover/press 反馈 |
| 待机生命感 | 无 | 首 token 前/running 态没有"活着"的信号层 |
| 节奏一致性 | 各组件动画数值零散 baked | 无统一 token,改观感要挖代码 |

**非目标**:不动稳定性地基(滚动三态/防闪/防 shift 已被机制覆盖,survey §5.6);不动解析/FSM/store;不引组件框架(0032);不做 BiDi/主题系统扩张;不在本篇定死任何节奏数值——**数值全为默认值提案,进 token 表可调**。

---

## 1. 五条设计原则(从调研提炼,验收时逐条对照)

1. **稳定压倒装饰**:任何动效不得制造读者注视区外的尖锐 transient(perception §4);位移过渡峰值速度落在平滑追随区(perception §5);`prefers-reduced-motion` → gate 全开 + 动画时长 0。
2. **块级是内容、词级是装饰**:阅读目标 = settled 的稳定块;逐字/逐词效果只作 karaoke 式装饰层,可关(streaming-ux §7)。
3. **有规律的重复 = 结构 > 微抖**:节奏美感的大杠杆是**结构性停顿**(边界 ∝ 节点层级)+ 末延长;1/f 微定时只做 σ=0.2Δ、±50ms 封顶的点缀(reveal-rhythm §2 / survey §3.4)。
4. **克制的生命感**:同屏活跃动画组 ≤4(MOT 上限);待机呼吸 0.1–0.25Hz、幅度 ≤5%;持续动效面积小、挂慢时钟(0028 护栏 4);闪烁 1s ≤3 次(WCAG)。
5. **一切皆数据**:动画=Anim 数据(0025)、揭示=RevealStyle 数据(0019)、样式=Palette/RenderStyle 数据(0021)、节奏=本篇 MotionTokens——改观感永远是改表,不是改管线。

---

## 2. 页面 layout(元素总览)

### 2.1 世界层(SDF 画布,Taffy 盒,plan13 骨架不变)

```
ChatContainer(flex column,内容列 max-width ≈ 768 世界单位,居中;两侧留白 ≥48)
  └ Turn(0005 投影)                    ← turn 间距 = space.turn(大,分组靠间距不靠线)
      ├ UserRow(align-self: end)       ← user 盒:弱底面板,右对齐,max-width ~85% 列宽
      └ AssistantRow(align-self: start)← assistant 回合容器盒(无底色,全宽)
          ├ part 块 × N(text/reasoning/tool/diff/file/compaction,part 间距 = space.part)
          └ meta 行(小号 Dim:模型·耗时·copy;Complete 态才现)
```

- **无气泡范式**(survey §1):assistant 不给底色,靠左对齐 + 间距成组(Gestalt 邻近律);user 给一块**弱底圆角面板**(0018,退化参数:无网格、小圆角、低对比 fill)标记"对方"。
- **间距 token**(进 style-config,示意默认):`space.turn = 40` · `space.part = 16` · `space.blockGap = 12`(现 BLOCK_GAP 收编)· `space.inset = 16`(卡内边距)。间距节奏本身就是"有规律的重复"的第一层(视觉 rhythm:regular)。
- **锚定**:滚动跟随三态 + 新回合锚近顶 + peek(streaming-ux §1/§3,已列 R10)——本篇不重复,只要求所有新组件的入场动画**不得改变锚定语义**(动画只动 alpha/scale/局部 offset,不动盒位)。

### 2.2 屏幕层(DOM,0022/0032 ③)

| 元素 | 位置 | 说明 |
|---|---|---|
| 输入区 | 底部固定 | 现 `chat-input.ts`;流式中变 Stop(survey §1 meta 惯例) |
| permission/question/todo dock | 底部/右下固定 | 现 `dock.ts`(0031 Blocked) |
| jump-to-latest pill | 内容列底缘悬浮 | 脱离跟随时出现;点击回底(三态机的显式回门) |
| find bar / style 面板 / debug | 既有 | 不动 |

---

## 3. 节奏系统(本篇核心;Plan 8 调度器升级的 policy 全集)

### 3.1 MotionTokens(一张表,所有组件只引用 token)

**duration 档**(survey §3.1,Carbon/M3 折中):

| token | 默认 | 用途 |
|---|---|---|
| `dur.glyph` | 200ms | 单字形/词内元素进场(≤ 一次注视) |
| `dur.element` | 240ms | 行内元素、徽章、光标态变 |
| `dur.block` | 320ms | 块/卡片进场、折叠展开 |
| `dur.panel` | 420ms | 骨架/网格/大面板生长 |
| `dur.exitFactor` | 0.8 | 退场 = 进场 × 0.8(不对称惯例) |

**ease 档**(进 0025 `CurveId` 表,新增三条 bezier 等价曲线):

| token | 曲线 | 用途 |
|---|---|---|
| `ease.enter` | ≈ cubic-bezier(0, 0, 0.38, 0.9)(Carbon entrance) | 一切进场(decelerate) |
| `ease.exit` | ≈ (0.2, 0, 1, 0.9)(Carbon exit) | 一切退场(accelerate) |
| `ease.expressive` | ≈ (0.4, 0.14, 0.3, 1) | 骨架/面板生长、大过渡 |
| (禁)bounce/stretch | — | 不入表(Carbon 惯例;弹性要用走解析阻尼 curve,0025) |

**stagger 档**:`stagger.glyph = 25ms`(词内字符,20–35 区间)· `stagger.item = 50ms`(列表项/表格行/cell 组)· `stagger.stage = 60ms`(0019 Stage 间,如网格→表头→cell)。总时长封顶:N 项 stagger 总长 ≤ `dur.panel`,超则压缩间隔。

### 3.2 节奏函数(替换匀速 cps;reveal-rhythm §2 七层的参数化)

揭示单元的释放时刻由分层函数给出(实现在 `RevealScheduler`,输入全部来自 0020 节点树 + 标点,CR1 纯函数):

```
release(unit_i) = t_{i-1}
  + Δ_base(unit)                          // ① tempo:词(拉丁)/字(CJK)= 180ms 默认(150–250 可调)
  × lengthen(unit)                        // ④ 末延长:边界前最后 1–2 单元 ×1.4
  + rest(boundary_level)                  // ③ 边界停顿(大杠杆):见下表
  + accent_pause(unit)                    // ⑤ 重音拍:强调单元前 +100ms,自身 ×1.2
  + ε_i                                   // ⑥ 微定时:seeded 1/f,σ=0.2Δ,clamp ±50ms
```

**边界停顿表**(∝ 节点层级,封顶 800ms;语音韵律典型 0.2–0.3s):

| 边界 | 默认 rest | 来源信号 |
|---|---|---|
| 逗号/顿号/分号 | 100ms | 标点 |
| 句号/问叹号 | 250ms | 标点 |
| 行内结构闭合(链接/行内码/粗体) | 80ms | 节点边界 |
| 列表项间 | 150ms | 节点(Item) |
| 段落间 | 400ms | 节点(Para) |
| 块间(表格/代码/标题) | 600ms | 节点(Block) |

**预设三档**(一键切,兼当回归对拍基线):

| 预设 | 配置 | 定位 |
|---|---|---|
| `typewriter` | rest=0、lengthen=1、accent=0、ε=0 | 现状等价(退化验证 + 重放兼容) |
| `reader`(**默认**) | 全默认值 | "朗读"感:句读呼吸、助读 |
| `flow` | Δ_base=120ms、accent 强、ε 满幅、accelerando 开 | "rap flow":快而有 groove |

**旋钮**:`tempo / rest / finalLengthening / accent / microtiming / accelerando` 六个标量进 style-config + 调试面板(实时);微定时 PRNG 以块 key 为种(R8)。**块级 gate 不受节奏影响**——节奏只调度"已过门的单元何时上屏"(0019 ③ 层职责,机制不动)。

### 3.3 装饰层(karaoke,叠在稳定块上)

- **词级进场分组**:glyph 按词分组共享 spawn 相位(词内 `stagger.glyph`),整词一个"拍"——落实"拍点在词不在字";CJK 单字即词。实现 = core 给 anim/spawn_time 时按 Run 内词边界分组,shader 不改。
- **到达高亮(可选,默认弱)**:最新揭示的词短暂提亮(tint +8%,`dur.element` 内衰减)= karaoke 读头;这是**有意的**注意引导(唯一主角原则,survey §1 staging)。
- `reduced-motion`:装饰层整体关闭,gate 语义不变。

---

## 4. 组件规格(元素 × shader × animation × 交互)

> 通用规则:**视觉=SDF(①)、命中=节点树(②)、真表单=DOM(③)**(0032);文字永远走 glyph 管线,shader 只画容器(0018);所有动画引用 §3 token;enter/exit 走 0025 profile,几何变化走 0016 morph。

### 4.1 user 消息盒

- **元素/layout**:右对齐弱底面板 + 文本;附件(file part)作 chip 行。
- **shader**:`panel.wgsl` 退化参数(fill=角色弱底、radius=10、无网格、无 AO 或极弱)。
- **animation**:进场 = 面板 `Alpha+Scale(0.97→1)` `dur.block`/`ease.enter`,文字整块同帧显(用户自己的话不必揭示);乐观发送态 alpha 0.85,确认后 1.0(0016 补间)。
- **交互**:hover 显 meta(copy/edit);命中层路由,视觉反馈 = panel `hoverCell`/tint 参数。

### 4.2 assistant 回合容器 + meta 行

- **元素/layout**:无底容器;左缘可选 2px 活跃指示条(仅 streaming 中,见 4.4);meta 行(模型·耗时·copy)Complete 态渐显。
- **animation**:回合进场 = 容器内首块的入场即是(容器自身不动画,守锚定);meta 行 `Alpha` `dur.element`,延迟 `stagger.stage`。
- **交互**:copy 按钮(Plan 21 已有);容器无命中动作。

### 4.3 text 正文(markdown)

- **元素**:标题/段落/列表/引用/Alert/行内码/链接/脚注……(既有 StyleRole 全集)。
- **shader**:字 = `glyph.wgsl`(enter profile 词级分组,4.3 无新 shader);装饰(引用条/Alert 底/行内码 chip)= panel 退化参数,收编零散 FrameRect(0018 §6 既定方向)。
- **animation**:
  - 揭示 = §3.2 节奏函数(内容层)+ §3.3 词级装饰层;
  - glyph enter profile 默认:`Alpha + 上浮 6px + Scale 0.96→1`,`dur.glyph`/`ease.enter`(现 profile 表改数值即得);标题用 `dur.block` + 前置 accent 拍;
  - 结构块骨架先行(0019 风格 2/3):列表 = marker 先现(`stagger.item` 逐项)、引用 = 左条先 wipe 出(mask,见 §5)再填字。
  - reflow = 0016 morph(不动)。
- **交互**:链接命中 → hover 下划线亮(glyph tint 参数)+ pointer;点击走②路由 → 宿主回调/DOM 打开;选区高亮已有(V2)。

### 4.4 流式指示器(光标 + 首 token 前占位)——**生命感的主载体**

- **元素**:① 行内光标:文字流末尾一个小方块(Claude 形态);② 首 token 前 / reasoning 等待:一枚小 glow orb(ShaderBox 已有 `glow_orb`)+ 可选单条 shimmer 线;③ streaming 中回合左缘 2px 指示条。
- **shader**:光标 = widget 小 fn(`sd_round_box` + 呼吸 alpha);orb = `shaderbox/glow_orb.wgsl`(参数:色相随 part 类型微移);指示条 = panel 左条参数 + 沿 y 的缓慢流光(fract(y·k − t·v) 渐变,ShaderBox 或 panel 加一个 flow 参数)。
- **animation**:光标呼吸 = 0.15Hz 正弦 alpha ±5%(**不是 blink**——1s ≤3 次且更柔);orb 呼吸 scale ±2%;全部挂慢时钟 30fps(0028 护栏 4)。收到首 token → 光标就位、orb `Alpha+Scale` 退场 `dur.element × exitFactor`。
- **交互**:无;`?debug` 显示其活跃计数(FrameStats 已有 ShaderBox 计数)。
- **性能**:三者面积合计 < 数千 px,0028 成本模型下可忽略;idle 时全部退场 → 页面回到全冻结。

### 4.5 reasoning 卡(thinking 区)

- **元素/layout**:`💭 Thinking` 标题 + 弱化正文 + 计时器(survey:Claude 形态,共识=折叠光谱中段);完成后 auto-collapse 到单行摘要。
- **shader**:panel 弱底 + 左条;**running 态可选 FBM 活性材质**(ShaderBox 新 shader `thinking_flow`:2–3 倍频 value-noise 流动,极低对比度、仅卡片底,面积受卡高裁剪;高端档,P3)。
- **animation**:进场 = 骨架先行(空卡 wipe 入 `dur.block`)→ 标题 → 正文按 §3.2(Δ_base ×1.3 更从容,"思考"语义);完成折叠 = 高度 0016 morph `dur.block × exitFactor` + 正文 alpha 退。
- **交互**:标题行命中 → 折叠/展开(②控件态 by node id);展开区超高时内部滚动(0027 视口机制复用)。

### 4.6 tool 卡(通用 + bash/edit/task 特化,Plan 23 R2 升级)

- **元素/layout**:卡底 + 状态徽章(pending/running/done/error)+ `▸ name` 标题 + args/output 区;read/grep 连续段折叠成 context 组(已有 R4)。
- **shader**:
  - 卡底 = panel(圆角 8、AO 弱、fill=CARD_BG);
  - **running 态 = 边缘 glow 描边**(survey §4 语汇 ②):panel 加参数 `edge_glow(强度, 相位)`——沿 `sd_round_box` 距离 0 处一条 1–2px 渐变光带,相位随时间沿周长流动(Apple Intelligence 边缘光的卡片版);面积极小(仅描边),挂慢时钟;
  - 徽章 = widget 小 fn(圆点/勾/叉 SDF),error 红、done 绿、running 主题色。
- **animation**:进场 = 骨架先行(空卡 `dur.block`,`ease.expressive`)→ 标题 `stagger.stage` → body;状态迁移 = 徽章 SDF `mix` morph(pending→running→✓,Plan 10 相位 4 的图标 morph 首用)`dur.element`;error = 徽章 shake **不做**(违反原则 1),改为红 tint 渐入 + 卡边框 1 帧内变红(色变 snap 合法,0016 §4.2);折叠/展开 = 高度 morph `dur.block`。
- **交互**:标题行 = 折叠把手(②);bash 卡 copy 热区(DOM,Plan 21 复用);task 卡点击 → 进子会话(0032 §4);链接类(webfetch)→ 4.3 链接同规。

### 4.7 diff 块

- **元素**:文件头 + `+x −y` 摘要 + 增删行底色带(已有 R3)。
- **shader**:行底色带 = panel 行 band(已有);**新增 DiffChanges 5 格条** = widget 小 fn(5 个 `sd_round_box`,按比例填绿/红/灰)——Plan 23 遗留项归入本设计。
- **animation**:行带逐行 `stagger.item` wipe 入(mask progress,自上而下);摘要数字与格条同帧。
- **交互**:文件头折叠(②);per-hunk 操作留待可写 agent(0032 已缓议)。

### 4.8 代码块(0027 视口)

- **元素**:底板 + 语言标签 + copy 按钮 + 视口内代码;超高滚动。
- **shader**:底板 panel;语言标签 chip;**流式中底部 1 行高的渐隐 mask**(生长边缘不硬切,panel alpha 渐变参数)。
- **animation**:围栏确认 → 底板先 wipe 入(`dur.panel`)→ 代码按**行**揭示(gate=Line,行内不逐字——代码读法不同于散文;survey §1 两遍 pass 惯例)+ 行 `stagger.item`;闭合后语法高亮一次性上色(色变 snap 合法)。
- **交互**:copy(已有);视口滚轮命中(已有);行号/折叠后置。

### 4.9 表格(0019 三风格已建模)

- 默认风格 3(全表门):网格 wipe 入(`dur.panel`,mask 沿 x)→ 表头字(`stagger.stage`)→ cell 按行 `stagger.item` 并行填;列宽变化 = panel colRatios + glyph 同源 morph(0018 §5 已定)。zebra/宽表滚动照 TODO 表格遗留 B/C 排期,不属本篇。

### 4.10 embed(图片/math/mermaid,O 相位)

- 占位框 = panel(估比例)+ 中心 loading orb(4.4 复用);Ready → 纹理 `Alpha` 渐入 `dur.block`,占位框留位防 reflow(0007 FSM 既定);Failed → 灰 chip + 重试命中。

### 4.11 hover / press / 选区(全局交互反馈层)

- hover 卡片:panel `aoStrength` +20% + tint +3%("抬起"),`dur.element`;press:scale 0.99,≤50ms 即时;链接/把手 hover 见各组件。**lensing/specular 高光**(survey §4 ⑦)= P3 高端档:hover 时 panel 加一条随 pointer 世界坐标移动的高光参数(纯参数,无纹理回读)。
- 选区 = 已有;多行选区圆角"墨团"(smin 并集)接 Plan 10 相位 4,观感目标 = macOS 选区。

### 4.12 背景 ambient(可选,默认关)

- 全屏极低对比 FBM 流场(Gemini/Stripe 语汇)诱惑大但**违反原则 4**(全屏活跃岛 + 周边运动 vection 风险)→ **默认不做**;仅留 ShaderBox 试验 shader-id 供 demo/film 用(plan17 已有 film 基建)。

---

## 5. SDF/shader 能力上限矩阵(本引擎"现在/加参数/新相位/不做")

| 能力 | 状态 | 载体 | 备注 |
|---|---|---|---|
| 字形进场(alpha/scale/rise/per-role profile) | ✅ 已有 | glyph.wgsl + enter_profile 表 | 本篇只改数值/加词级分组 |
| 几何 morph(reflow 不跳) | ✅ 已有 | 0016 Scene + PanelScene | 不动 |
| 面板圆角/网格/AO/行带/选中 | ✅ 已有 | panel.wgsl + storage buffer | 参数已数据驱动 |
| 骨架先行编排(gate×stage) | ✅ 机制已有 | 0019 调度器 + Plan 9 resolve | 本篇给各组件默认 plan |
| 小图标/checkbox/rule widget | ✅ 已有 | widget.wgsl(component-id) | 徽章/5 格条加 fn 即可 |
| 任意 fragment 画板(time/params) | ✅ 已有 | ShaderBox(glow_orb 等) | 五护栏已定 |
| **mask wipe 揭示(progress 参数)** | 🔧 加参数 | panel/widget 加 `reveal_progress` | 0025 AnimTarget 已列 MaskProgress,接线即可 |
| **边缘 glow / 流光描边** | 🔧 加参数 | panel 加 `edge_glow(强度,相位)` | 面积=描边,便宜;挂慢时钟 |
| **呼吸/慢时钟待机层** | 🔧 加参数 | ShaderBox 慢时钟已定(护栏 4);widget 光标加 time | 同屏 ≤4 活跃岛预算 |
| **图标 SDF morph(mix A→B)** | 🚧 新相位 | Plan 10 相位 4(面板/解析场 smin/mix) | 徽章态变首用,范围小 |
| **气泡 smin 融合 / 墨团选区** | 🚧 新相位 | Plan 10 相位 4 | 观感差异化点 |
| **thinking FBM 活性材质** | 🚧 新 shader-id | ShaderBox `thinking_flow` | 低对比、卡内裁剪 |
| **lensing/specular hover 高光** | 🚧 新参数(P3) | panel 高光项(pointer 世界坐标 uniform) | 不折射真实底层内容(那需纹理回读,不做),做参数化近似 |
| 字↔字 shape morph | ⏸ 后置 | 0025 相位 5(2× 采样) | 成本高,非本篇 |
| 全屏背景流场 | ❌ 默认不做 | — | 违反克制原则;留 demo |
| 文字进 fragment 特效(逐字扭曲等) | ❌ 不做 | — | 0018 铁律:shader 画容器,glyph 画字 |

**上限一句话**:除"字↔字 morph"与"真实折射"外,§4 全部组件规格都落在**已有图元 + 加参数**的范围内;真正的新工程只有 Plan 10 相位 4(smin/mix)和两三个小 shader fn。**上限卡在 fill-rate(大面积活跃 shader)与 resolve 缓存(0025 §4),不卡在表现力**——这正是 SDF 路线的红利。

---

## 6. 落地相位(建议拆 plan;每相位过 verify.sh + 黄金帧)

| 相位 | 内容 | 杠杆/风险 |
|---|---|---|
| **P0 token 收口**(快赢) | MotionTokens 表(dur/ease/stagger 进 style-config + CurveId 扩三条曲线);间距 token;现有 enter profile / panel 默认值改引 token;meta 行延迟显 | 纯数值,零新机制;页面观感立即统一 |
| **P1 节奏函数**(北极星本体) | RevealScheduler 匀速 → §3.2 分层函数(边界停顿+末延长先行,再 accent/微定时);三预设;调试面板六旋钮;重放 case 对拍(typewriter 预设 = 现状回归基线) | 核心价值;R8 seeded;native 可测(纯函数) |
| **P2 组件动效** | mask `reveal_progress` 参数(panel/widget)→ 骨架 wipe 全组件铺开;词级 spawn 分组 + 到达高亮;光标/orb 待机层(慢时钟);tool 卡 edge_glow;diff 5 格条;hover/press 反馈 | 每项独立可合;fill-rate 走 FrameStats 盯 |
| **P3 高端档** | Plan 10 相位 4(徽章 mix morph、墨团选区、气泡 smin);thinking_flow;lensing 高光 | 差异化;可各自延期不阻塞 |

**验收**(接 Plan 24 框架):TC-E02 节奏人审(三预设定帧序列)→ 过审转像素回归;TC-E01 无 raw 依旧全绿;新增「节奏确定性」native 测试:同 seed 同输入 → release 时刻序列逐 ms 相等(R8);FrameStats 新增活跃动画组计数,`?debug` 断言 ≤4。

## 7. 铁律自查

- **CR1**:节奏函数/边界表/词分组全在 core,纯函数,无平台依赖。
- **R8/R9**:微定时 seeded(块 key 为种);光标/orb 呼吸走注入时钟;重放/seek 稳定。
- **0018 边界**:文字不进 shader;新效果全是 panel/widget/ShaderBox 参数。
- **0028 五护栏**:所有常驻动效 = 小面积 + 慢时钟 + 离屏 cull + 静态冻;idle 页面回全冻结。
- **0032**:零新框架;交互全走三层。
- **锚定/稳定**:入场动画不动盒位;色变 snap 合法、几何绝不 snap(0016)。

> 评审焦点建议:① §3.2 默认值(尤其 Δ_base=180ms 与边界停顿表)是否合口味——好在全是旋钮;② 4.4 生命感载体三件套(光标/orb/指示条)是否过多,可裁;③ P2 的 edge_glow 与 P3 的 thinking_flow 谁先谁后。
