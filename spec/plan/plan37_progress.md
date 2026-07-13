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

(未开始)

## F2 · 后处理三小件(vignette / grain / 色差)

(未开始)

## F3 · bloom 可选档(时间盒)+ 收口

(未开始)

## DoD 对账

(未开始)
