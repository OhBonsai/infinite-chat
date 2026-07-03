# Plan 27:流内问答卡(question/permission 入对话列)+ 活跃期双实现 A/B

- 日期:2026-07-02
- 状态:**已落地(2026-07-03)**。PR-A~E 全部完成;统一门 `node test/run.mjs` 全绿。
  - **PR-A**:store `PartExtra::Ask`(Pending→Answered 原位落定,0020 身份不变)+ `push_ask`/`pending_ask`/`answer_pending_ask`(幂等)/`set_ask_pad_lines`;ingest asked→流内块(FSM 0031 语义不变);engine/wasm:`ask_state`/`ask_rect`/`reply_permission_with(allow)`/`reply_question_with`/`set_ask_height`;`PartKind::Ask` + 兜底 + `ask_render`(pending/answered 双态,契约闸)。native p27×3。
  - **PR-B(B 路)**:`ask-overlay.ts` 锚定表单(canvas-rect;首次可见才聚焦——display:none 聚焦会丢;实测高单向回报);落定答案卡(所选项 chip + 输入原文,未选项收起)。
  - **PR-C(A 路)**:`StyleRole::AskButton`(58)+ wgsl;按钮面板与命中盒同源(`ask_button_runs`);`tap`(input.ts ≤4px 判定)/hover cursor/按压深色;影子 a11y 真按钮(text-layer);**pending permission 块的透明 span 开 pointer 洞**(否则吞点击——实弹踩坑,已记 0032 附录 A)。
  - **PR-D**:剧本 `dock` 指令退役 → `ask`(permission=tap 真按钮矩形中心 / question=填真表单);mini/showcase-full 场5/8 改造;`dock.ts` 删除、`dock.spec`→`ask.spec`(①②③)、a11y 焦点断言迁表单/影子按钮。
  - **PR-E**:A/B 客观结论 + 判据表回写 [0032 附录 A](../decision/0032-interaction-architecture-sdf-world-not-component-framework.md);手感对比 🟡 人工。
- 前置:[0032](../decision/0032-interaction-architecture-sdf-world-not-component-framework.md)(三层交互模型——本 plan 是它的第一次实弹检验)· [0031](../decision/0031-event-fsm-resilience-and-js-rust-boundary.md)(FSM Blocked 语义,不变)· [0033](../decision/0033-part-render-contract.md)(落定卡走 partrender)· [0022](../decision/0022-dom-overlay-layer.md)(DOM 锚定机制,embed-overlay 先例)· [0011 §3.3④](../decision/0011-gpu-text-as-sdf-primitive.md)/[0020](../decision/0020-content-node-identity-model.md)(CPU hit-test + 节点身份)· [0029 §3](../decision/0029-session-virtualization-and-glyph-working-set.md)(布局稳定)· [Plan 25](./plan25-chat-scripted-showcase.md)(剧本/showcase,需扩展)· 设计准则:0→1 不留后向兼容
- 交互形态(参考 Claude Cowork AskUserQuestion):question/permission 不再是底部浮层 Dock,而是**对话流内的卡片**——活跃期展示选项(checkbox 多选 + 自由输入)/按钮,应答后**选项收起,只留所选答案,以卡片形式留在对话历史里**。

---

## 0. 目标 / 非目标 / 决策背景

**目标**:
1. question/permission 从「会话级 fixed 浮层」改为「**流内块**」:占位、随滚动、虚拟化、应答后原位落定成答案卡(永久对话历史的一部分)。
2. 活跃期**双实现 A/B**:**permission = wasm 原生**(SDF 按钮 + hit-test,试炼 0032 hit-test 层——子 agent 卡片将来必经之路);**question = DOM overlay 锚定**(checkbox+文本输入,IME 硬约束)。做完在 /chat 逐帧对比,结论回写 0032。

**为什么这样切**(需求陈情已确认):
- 落定期无争议 = wasm SDF 卡(与 tool 卡同类:排版/滚动/虚拟化/可复制/partrender)。
- 活跃期的分歧点是**自由文本输入必须 DOM**(IME 组合输入 canvas 自绘 = 深坑,0030 已论证)→ question 锁 DOM;permission 纯两键无文本 → 唯一能公平试 wasm 原生交互的对象。

**非目标**:不做通用表单框架(0032 铁律:不造组件系统);不做 ask 卡滚出视野时的"回到问题"引导(P2);旧底部 Dock **退役不保留**(0→1 准则,dock.ts 删除,相关测试改写)。

---

## 1. 共同结构(core,两边共用——本 plan 真正的结构活)

### 1.1 流内 ask 块

- `permission.asked` / `question.asked` 事件 → Store 追加一个 **ask 块**(伪 part,流内、有身份 0020):携带 `kind(permission|question)`、`prompt` 文本、`options[]`(question)、`state: Pending | Answered{...}`。事件 properties 里缺字段 → 容忍并用占位文案(AR12)。
- FSM `Blocked{permission|question}` 语义**不变**(0031);变的只是呈现位置。锚底跟随使新 ask 块自然进入可见(现有行为)。
- **应答** → ask 块 `state=Answered`(存所选项/输入文本)→ 块内容原位替换为答案卡(选项收起,高度变化走正常 reflow;0020 身份不变 → 其余块稳定)→ FSM 解阻。

### 1.2 core/wasm 新接口(最小集)

| 接口 | 作用 |
|---|---|
| `ask_state() -> JSON` | 当前 pending ask(kind/prompt/options/块 id);无则空。DOM 侧与剧本/e2e 的真相源 |
| `ask_rect() -> JSON` | pending ask 块的屏幕矩形(设备像素,随相机每帧变)——DOM 表单锚定用(同 frame_embeds 惯例) |
| `ask_hit_targets() -> JSON` | (permission)SDF 按钮命中盒列表 `{id:"allow"\|"deny", x,y,w,h}` ——tap 路由 + 剧本/e2e 定位用 |
| `tap(x, y)` | 画布点击路由:命中 ask 按钮 → 应答;未命中 → 忽略(为 0032 hit-test 层开的第一个口) |
| `reply_question_with(json)` | `{options:[idx...], text?}` → Answered + 解阻(替代现 `reply_question()`) |
| `reply_permission_with(allow: bool)` | 同上(现 `reply_permission()` 不分允许/拒绝,升级) |

### 1.3 落定卡渲染(0033 注册表)

- partrender 注册 ask 专门渲染:**question 答案卡** = prompt(弱化)+ 所选项(高亮 chip)+ 自由输入原文;**permission 卡** = 「已允许 ✓ / 已拒绝 ✗」+ 权限摘要。兜底(未注册)= 标签+JSON 丑骨架,契约闸照跑。

---

## 2. A 路:permission = wasm 原生活跃期

- **画**:pending 时块内 SDF 画两个按钮(允许/拒绝):圆角面板(0028 shaderbox 或 FrameRect)+ 文字,主题色走 Theme(26①)。
- **点**:web `input.ts` 把 click(非拖拽结束的 tap)转 `chat.tap(x,y)`(设备像素,canvas 相对);core 用 `ask_hit_targets` 命中 → `reply_permission_with`。
- **手感(试验的观测点)**:hover 高亮 + cursor:pointer——由 web 侧 mousemove 查 `ask_hit_targets`(节流)切 CSS cursor;按压态 SDF 变暗一帧。**这些成本就是 A/B 要量的东西,如实做、如实记。**
- **a11y 成本(试验的一部分)**:SDF 按钮读屏不可见 → text-layer 镜像里为 pending permission 补一对隐藏真按钮(点击转 reply)。记录这份"影子 DOM"的维护成本进对比结论。

## 3. B 路:question = DOM overlay 锚定活跃期

- `web/src/ask-overlay.ts`(新):pending question 时按 `ask_state()` 建表单——prompt + **checkbox 多选** + 自由输入 `<input>` + 确认键;容器绝对定位,每帧按 `ask_rect()` 对齐(embed-overlay 同款相机同步;canvas-rect 已保证画布偏移正确)。
- 占位:core 侧 ask 块 pending 期按**估算高度**占位(选项数 × 行高 + 输入行);DOM 表单实测高与估算差 >1 行时,DOM 回报 `set_ask_height(px)` → 块高微调一次(防遮挡/空洞)。
- 提交 → `reply_question_with({options, text})` → overlay 卸载 → 落定卡原位接管。间隙无闪:落定卡在 overlay 卸载同帧已渲出(状态先行,DOM 后撤)。
- a11y:真表单天生可达;announcer 播报沿用 26②。

## 4. 剧本 / showcase / Dock 退役

- 剧本指令扩展(Plan 25 §1):`dock` 指令**替换**为 `ask` 指令——`{ "ask": { "allow": true } }`(permission,驱动走 `tap` 命中真按钮矩形)/ `{ "ask": { "options": [1], "text": "保持 1s" } }`(question,驱动填真 DOM 表单后点确认)。**两边都走真交互路径,不直调 reply**。
- `showcase-full.json` 场5(permission)/场8(question)改走新交互;落定卡成为对话历史一部分(场11 收尾后仍可回滚看到)。
- `dock.ts` 删除;`dock.spec.ts` 改写为 `ask.spec.ts`;a11y.spec 的 Dock 焦点断言迁到新表单/隐藏按钮。

## 5. 测试(进 test/ 默认门)

- **native**:ask 块状态机(Pending→Answered 原位替换、高度 reflow 稳定 0029§3、身份不变 0020);`ask_hit_targets` 命中几何;`reply_*_with` 幂等(重复应答忽略);渲染契约闸覆盖 ask kind。
- **e2e `ask.spec.ts`**:① question:合成事件 → 流内表单出现(锚定块矩形 <2px)→ 勾选+输入+确认 → 表单消失、答案卡在原位、FSM 解阻;滚动时表单跟块走。② permission:合成事件 → SDF 按钮出现(`ask_hit_targets` 非空)→ 在按钮矩形中心真实 click → 解阻 + 落定卡;hover 时 cursor 变 pointer。③ 落定卡文本可复制(text-layer)。
- **chat-route.spec**:里程碑更新(场5/场8 走新路径)。
- **人工确认(🟡)**:A/B 手感对比——滚动跟手、hover/按压反馈、主题一致性、缩放表现(SDF 卡随 zoom 缩放,DOM 表单不随 zoom——**这是预期差异,重点观察**)。

## 6. A/B 评审与回写(收尾必做)

对比维度:实现/维护成本(影子 a11y、hover 手写)· 滚动/缩放跟手 · 观感一致性 · 测试可断言性。**结论回写 0032**:形成「交互放哪层」的实测判据表(如:纯按钮 ≤N 个 → SDF 层;含文本输入/复杂表单 → DOM 锚定层),供子 agent 卡、future 组件决策直接引用。

## 7. 落地顺序(PR 切分)

1. **PR-A 共同结构**:core ask 块 + FSM 接线 + `ask_state/ask_rect/reply_*_with` + 兜底渲染 + native 测试。(此后 B、C 可并行)
2. **PR-B question DOM**:`ask-overlay.ts` + `set_ask_height` + 落定卡专门渲染 + e2e①③。
3. **PR-C permission wasm**:SDF 按钮 + `ask_hit_targets`/`tap` + hover/cursor + 影子 a11y 按钮 + e2e②。
4. **PR-D 剧本+showcase**:`ask` 指令(schema+driver+vitest)+ showcase 场5/8 改造 + chat-route 里程碑 + dock 退役(删 dock.ts、改写 spec)。
5. **PR-E 评审回写**:🟡 人工 A/B 对比 → 结论写入 0032 附录。

## 8. 风险

- **blocked 时块在屏外**:用户往回滚历史时问题卡不可见 → FSM 仍阻塞,输入框已有 blocked 提示兜底;"跳回问题"引导留 P2(非目标)。
- **DOM 表单不随 zoom**:B 路表单是 CSS 像素,画布 zoom 时块矩形变但表单字号不变——锚定位置正确、大小观感差异记入 A/B 评审(embed-overlay 动图同款既有行为,非新问题)。
- **估高误差**:§3 的 `set_ask_height` 单向微调一次,不做双向反馈环(防抖动)。
- **tap 与拖拽冲突**:input.ts 需区分 click 与 drag-end(位移阈值 ~4px),别把平移误判成按钮点击。
