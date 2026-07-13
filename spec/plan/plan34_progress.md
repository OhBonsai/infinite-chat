# Plan 34 · 进度账本(正确性/接入收口包)

> 逐 milestone 记录:做了什么 / 参考依据(file:行号)/ 遗留。
> GOAL:[plan34-correctness-integration-goal](./plan34-correctness-integration-goal.md)

## S1 · 滚动跟随 FSM(ADR + 三态 + 9 场景)

- **ADR**:[0038 scroll-follow-fsm](../decision/0038-scroll-follow-fsm.md)(GOAL 预计编号即中)——
  三态 Following/Released/Anchoring + 释放信号全集表 + 重进入条件 + 布局稳定不变量。
- **实现**(`crates/core/src/app.rs`):`FollowState` 枚举取代 `stick_to_bottom` 布尔;
  释放信号:`scroll_by(dy<0)`/`pan_by(dx≠0|dy<0)`/`zoom_by`/`set_selection(非空,新增)`/
  `scroll_to(jump)`;新 API `scroll_to_latest()`(Anchoring 平滑滚底)+ `follow_state()` 只读;
  wasm 导出同名两个(additive)。
- **修成文缺陷**:旧「距底 ≤48px 每帧被动复跟随」→ 重进入仅在 `scroll_down_intent`
  (用户主动向下,一帧有效)且抵底时发生;内容增长永不触发重进入。
- **行为等价**:Following == 旧 true 路径(smooth-damp/超一屏直跳原样);全 golden 不变
  (gate 全绿佐证);Anchoring 复用同一 damp 通道。
- **测试**:native 6 个 `follow_fsm_*` 覆盖 shadcn 9 场景(滚轮/键盘=scroll_by、
  滚动条/触摸=pan_by 两轴、缩放、选区、jump 六信号枚举 + 贴底/增高零位移/jump 回底/
  选区期间增长/重进入吸附);e2e 1 个(像素级零位移断言 + scroll_to_latest 恢复跟随)。
- **验收帧**:`test/results/plan34/s1-released-dpr{1,2}.png`。
- **踩坑**:①测试正文用 `\n\n` 分段(单 `\n` 被 markdown 并段 → 高度不超视口);
  ②e2e 必须 bootVisible 等 settle(固定 sleep 下 showcase 未放完,流式揭示被误判为位移)。
- **遗留**:jump-to-latest 按钮 UI(宿主侧;引擎 API 已备);新回合锚顶 peek(shadcn 3,
  记 research 增强档,非本 GOAL DoD)。

## S2 · 可点链接

(未开始)

## S3 · URL 白名单

(未开始)

## S4 · 流式 a11y 播报

(未开始)

## S5 · Vue harness

(未开始)

## DoD 对账

(未开始)
