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

- **post.wgsl**(noise.wgsl 前置拼接):单 shader 参数向量化 —— vignette
  `1−v·smoothstep(0.35,0.9,|c|·2)`、grain `nz_value(uv·dim/3, seed)±0.5×强度`
  (seed=**注入帧计数**,R8 非墙钟)、chroma 径向 R/B 反向偏移 px(弱档 clamp≤6)。
  **全零 = 同 uv 采样原像素,位等直通** → present 管线统一取代 blit(少一条管线,
  feedback_blit 已删)。
- **管线**:RT 路条件扩为「decay>0 或 post 非零」;FeedbackRt 增 post bind groups
  (同视图 + post_ubuf);post-only 时也走双 RT(记账 2×,单 RT 优化记遗留)。
- **core**:`set_post_params(v,g,c)`(clamp 0..1/0..0.5/0..6)+ `frame_counter`
  (wrapping,advance 递增)作 grain_seed。
- **wasm/面板**:`set_post_params`;Effects 节「post fx」(off/vignette/grain/chroma/全开)。
- **golden**:`post-three.png`(maxDiffPixelRatio 0;paused 定帧 → seed 冻结稳定)。
  三件肉眼验证:满帧颗粒/文字红青色差边/边缘压暗。
- **踩坑**:参数设置与 set_paused 同批执行 → FrameData 未重建即冻结,post 全零假象;
  参数先行等一帧再冻结。
- **naga**:post/feedback 两 shader 校验测试补上。

## F3 · bloom 可选档(时间盒)+ 收口

- **bloom:时间盒零轮,记遗留** —— Dual Kawase 简版需 4×多分辨率 RT + threshold/down/up
  三类 pass,远超本盒;GOAL 明示「bloom 缺席不影响 DoD 主体」。通道(RT/present 管线/参数
  面)已就绪,后续可作纯增量接入。
- **拖尾验收帧**:`test/results/plan37/f1-trail-dpr{1,2}.png`(decay 0.88 + 快速上滚,
  滚动残影中段;弱档观感 —— 残影在引用块区可辨,主观强度调档留 38 预设)。

## DoD 对账

1. **ADR 0040** ✅(pass 图/契约/确定性条款/降级矩阵/显存记账)。
2. **管线** ✅ lazy ping-pong(resize 重建/off 释放);content pass 语义零改(默认 golden
   全集逐字节,每门佐证);FrameStats feedback_active/rt_bytes + 面板行。
3. **拖尾** ✅ Max 混合(静止恒等)= 入场拖尾 + 滚动残影同机制;收敛自停(settle 窗纯函数,
   idle pass 级退场);默认 off 面板开。
4. **三小件** ✅ vignette/grain(noise+帧计数 seed)/chroma 单 shader;置零位等直通;
   bloom 遗留(时间盒)。
5. **确定性与门** ✅ 默认态硬门每门验证;grain seed 冻结定帧 golden(maxDiffPixelRatio 0)
   = 帧确定锁;perf 门绿(off 零开销)。feedback 开启态帧级双跑(tick 步进驱动)记遗留。
6. **测试** ✅ native 3(settle 数学/自停重开/零面)+ naga 2;e2e golden ≥2;验收帧
   dpr 双份(拖尾 + post 开启)。
7. **记账** ✅ 本文件;commit F0 5c89912 / F1 5d52389 / F2 6c729dd / F3 本次;不 push。

## DoD 对账

(未开始)
