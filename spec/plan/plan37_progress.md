# Plan 37 · 进度账本(反馈缓冲与后处理通道)

> 逐 milestone 记录:做了什么 / 参考依据 / 显存与确定性数值 / 遗留。
> GOAL:[plan37-feedback-pass-goal](./plan37-feedback-pass-goal.md)
> 前置:plan36 完成(noise.wgsl 在库)。

## F0 · ADR + 直通骨架

- **ADR**:[0040-feedback-and-post-pass](../decision/0040-feedback-and-post-pass.md) 已落
  (pass 图/FeedbackParams+PostParams 契约/确定性条款(RT 零初值+注入帧计数 seed)/
  off 零分配/RGBA8 无特性依赖/2×RT 显存记账/reduced 强制 off/收敛自停)。
- **进行中**:backend pass 接缝 + off 直通骨架(golden 硬门先验证)+ FrameStats
  feedback_active/rt_bytes + 面板显示。

## F1 · feedback ping-pong + 拖尾

- **管线**(backend):lazy 双 RT(RGBA8 同 surface 格式,resize 尺寸失配即重建,off 置
  None 零常驻);content pass 画到 RT[cur] → 同 pass 末尾全屏叠 RT[prev]×decay(**Max 混合**
  —— `max(scene, scene·decay) == scene`,静止内容恒等;运动/消失像素留衰减残影,这就是
  拖尾/滚动残影的同一机制)→ blit pass 上屏 → ping-pong 翻转。`post/feedback.wgsl`
  (全屏三角 + fs_prev/fs_blit 双入口)。
- **core**:`set_feedback_decay(0..0.98)`;画面活动追踪(内容到达/字形释放/相机 pan 变/
  退场中/折叠动画,全经注入时钟 R8)→ 静止超 settle 窗(`feedback_settle_ms`:
  decay^n<1/255 的 n 帧折 ms,纯函数)→ 发射 decay=0(**收敛自停**,pass 退出回直通,
  idle 全退场断言扩到 pass 级);活动恢复即重开。FrameStats 实值(active/2×w×h×4)。
- **wasm/面板**:`set_feedback_decay` 导出;Effects 节「feedback trail」(off/0.7/0.85)。
- **踩坑**:活动探针放 advance 早段会错过 1e9 cps 的瞬时 settle —— 活动标记直接落在
  内容到达(enqueue)与字形释放(schedule)两个事实点。
- **native 3**:settle 数学(0.5→8 帧)/自停+重开+记账(双跑一致)/F0 默认零(升级为
  off 面)。
- **遗留**:开启态拖尾 golden(固定 dt 第 N 帧家族)与 WebGL2 抽查随 F3 验收帧一并;
  半分辨率 RT 档(ADR 预留参数)未做。

## F2 · 后处理三小件(vignette / grain / 色差)

(未开始)

## F3 · bloom 可选档(时间盒)+ 收口

(未开始)

## DoD 对账

(未开始)
