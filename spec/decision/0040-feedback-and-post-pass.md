# 决策记录 0040:反馈缓冲与后处理通道 —— content → feedback → post 三段 pass 图

- 日期: 2026-07-14(Plan 37 F0)
- 状态: 已采纳
- 前置: [0003](0003-render-backend-abstraction.md)(RenderBackend trait,当年留过 overlay pass 位)·
  [0026](0026-modular-shader-organization.md)(shader 组织)· [0034](0034-canvas-runtime-resize-flicker-free.md)
  (resize 重建路)· [0002 §5.1](0002-event-driven-pipeline.md)(效果=数据,off=参数置零)·
  [research/shader-effects-catalog §5/§8](../research/shader-effects-catalog.md)
- 范围: 渲染管线新增两个**可选** pass 插槽;off 时零分配零 pass(默认),on 时帧序列确定可复现。

## 1. 背景 / 问题

拖尾/余晖/motion blur/CRT/流体一整族效果的唯一钥匙是 **feedback buffer**(上一帧结果作本帧输入);
vignette/grain/色差/bloom 则需要**屏幕空间后处理**。两者都不是"往 content pass 里加几行"能落的
——需要管线级 RT 与 pass 顺序契约,一次定清(0→1,不留临时双轨)。

## 2. 决策

```
content pass ──→ [feedback mix pass]? ──→ [post pass]? ──→ present
   (现状不动)      prev*decay + cur          vignette/grain/色差(单 shader 参数向量化)
                   RT ping-pong(A/B 交替)     读 feedback 输出(或 content 直读)
```

- **数据契约**:`FeedbackParams { decay: f32, mix_mode: u32 }` + `PostParams { vignette, grain,
  chroma, _pad }`(core 只发参数,**不知道 pass 存在** —— AR1/AR2 管线级;全零 = 两 pass 均
  不插入,present 直通 content,像素与现状逐字节一致)。
- **确定性条款(R8 管线级)**:feedback 内容 = 帧序列的纯函数 —— RT 初值全零、mix 仅依赖
  (prev RT, 本帧 content, decay),时间只经注入 dt 进 content;固定 dt 序列 → 第 N 帧逐字节
  可复现(重放含 feedback 双跑一致)。grain 的帧 seed = 注入帧计数,非墙钟。
- **off = 零成本**:RT lazy 分配(首次 on 才建),off 即刻释放;`FrameStats += feedback_active,
  rt_bytes`(off 断言 rt_bytes==0)。
- **WebGL2 全支持**:RGBA8 + 标准 sampler,不用 float RT/blit 特性;naga 转译路已验(plan32 D3)。
- **显存记账**:2 × w × h × 4 × dpr²(ping-pong 双 RT);resize/dpr 变随 0034 重建。
- **降级矩阵**:reduced profile → feedback 强制 off;RT 可半分辨率档(参数,后置);
  post 三小件参数置零即恒等,无需分支。
- **收敛自停**:内容静止且 feedback 残差 < 1/255 连续 N 帧 → pass 自动退出回直通
  (idle 全退场断言扩展到 pass 级)。

## 3. 备选与否决理由

- **挂在 content pass 里做拖尾**(per-instance 历史):违 0→1(双轨)且做不了滚动残影/后处理;否决。
- **float RT 提精度**:WebGL2 特性依赖 + 带宽翻倍;RGBA8 + decay 下限(≥1/255 可感)够用;否决。
- **每效果独立 post pass**:pass 数随效果线性涨;单 post shader 参数向量化(置零恒等)足够;否决。

## 4. 影响

- 正:拖尾/余晖/氛围一族获得管线级入口;plan38 预设只发参数;确定性/golden/内存三线不破。
- 负:backend 增两条 pipeline + RT 生命周期管理;以 off 硬门(golden 逐字节)+ FrameStats
  记账 + 收敛自停三道断言兜住。

## 5. 实现入口

`crates/render/src/backend.rs`(pass 组织/RT)· `shaders/post/feedback.wgsl` ·
`shaders/post/post.wgsl` · core 参数字段(纯数据);记账见 [plan37_progress](../plan/plan37_progress.md)。
