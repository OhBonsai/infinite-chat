# Plan 37 · GOAL:反馈缓冲与后处理通道 —— 拖尾/余晖/后处理一族的管线钥匙(agent 直跑,无人介入)

- 日期:2026-07-14
- 状态:**GOAL(可执行任务书)** —— 给 Claude Code 的完整执行合同,读完即可自主跑到 DoD。
- 执行方式:**顺序执行,master 工作区直跑**。前置 plan36 完成(noise.wgsl 在库——grain/热浪消费它)。
- 定位:**TODO2-A 效果系统第二期(通道层)**。catalog §8 判定三缺口之二:**feedback buffer 通道**是拖尾/motion blur/CRT 余晖/流体一整族效果的唯一钥匙;顺路落屏幕空间后处理 pass(vignette/grain/色差/bloom 可选档)——0003 当年留过 overlay pass 位,本 GOAL 兑现成正式第二 pass。
- 前置:
  - [research/shader-effects-catalog](../research/shader-effects-catalog.md)(§4 拖尾/bloom/CRT/§6 trail 原理与链接;Dual Kawase 参考 blog.frost.kiwi/dual-kawase)
  - [0003 容错与降级](../decision/0003-fault-tolerance.md)(RenderBackend trait + pass 组织)· [0002 §5.1](../decision/0002-event-driven-pipeline.md)(profile 置零非分支)· [0034 canvas 运行时 resize](../decision/0034-canvas-runtime-resize-flicker-free.md)(RT 尺寸/dpr 变化的既有处理,feedback RT 跟随)
  - 现有:render 单 pass(content);`FrameStats` 度量面;plan29 内存红线意识(RT 是新增显存大头,必须记账)

---

## 0. 铁律(第一条):呈现历史 ≠ 逻辑状态

feedback 纹理是**纯呈现态**:只进 shader、**永不回读进 core 逻辑**(AR1/AR2 的管线级延伸)。hit-test/选区/滚动/golden 语义全部与 feedback 无关。off 时**零成本**:不分配 RT、不加 pass(lazy alloc,首次开启才建)。默认 off——现有 golden 全集逐字节不变是硬门(同 plan36 §0)。

## 1. GOAL 与完成定义(DoD)

**GOAL**:(A) 新 ADR(取当时下一空闲编号,预计 0040):反馈/后处理通道设计——RT ping-pong、pass 顺序、确定性契约、降级矩阵;(B) 管线落地:content pass → (可选)feedback mix → (可选)post pass → present,off 全直通;(C) 首批消费:字符入场拖尾 + 滚动残影(弱档);(D) 后处理三小件(vignette/grain/色差)+ bloom 可选档(时间盒)。

**DoD(全满足才算完)**:

1. **ADR 就位**:pass 图(content→feedback→post)、`FeedbackParams{decay, mix_mode}` 数据契约、**确定性条款**(feedback 内容 = 帧序列的纯函数:注入 dt 序列固定 → 任意帧逐字节可复现;重放含 feedback 双跑一致)、off=零分配零 pass、WebGL2 全支持(RT/blit 无特性依赖)、显存记账(2×全屏 RT×dpr²,进 FrameStats)。
2. **管线**:lazy RT ping-pong(resize/dpr 变随 0034 重建);pass 插入不改 content pass 任何语义;`FrameStats += feedback_active/rt_bytes`;debug 面板显示。
3. **拖尾首消费**:新字符 spawn 处写入强度 → feedback 衰减叠加(`prev*decay + cur`),形成淡拖尾;滚动残影同通道弱档;两者默认 off,面板开;**恒等收敛**:内容静止 + N 帧后 feedback 影响 < 1/255 → 自动停 pass 回直通(idle 全退场断言扩展到 pass 级)。
4. **后处理三小件**:vignette(中心距压暗)/ grain(noise.wgsl 消费,帧 seed 注入)/ 色差(径向 RGB 偏移,弱档上限);全参数置零恒等;**(可选档,时间盒 ≤2 轮)**bloom:Dual Kawase 简版(2 down + 2 up),超时间盒记遗留。
5. **确定性与门**:含 feedback 的场景注入固定 dt 序列 → 第 N 帧 golden 双跑逐字节;默认态 golden 全集不变(硬门);perf 门绿(off 态零开销;on 态单独记录不入门);动画组 ≤4 断言保持。
6. native ≥3 新测(decay 数学纯函数/收敛自停/RT 记账)+ e2e ≥2(开启拖尾帧确定/off 逐字节);验收帧 dpr=1/2(拖尾中间帧 + 后处理三件开启态)。
7. `spec/plan/plan37_progress.md` 逐 milestone 记账;**按 milestone commit,不 push**,commit 前全门绿。

**启动**:`claude --permission-mode acceptEdits "读 spec/plan/plan37-feedback-pass-goal.md,按其执行到 DoD 全满足"`(中断重跑同句,读 progress 续跑)。

## 2. Milestones

- **F0 · ADR + 直通骨架**:ADR 落稿 → RenderBackend 加可选 pass 接缝(CR3 trait 路,不 cfg)→ off 全直通(golden 硬门先验证)→ FrameStats/面板。
- **F1 · feedback ping-pong + 拖尾**:lazy RT → decay mix shader → spawn 强度写入 → 收敛自停 → 双跑帧确定测试 → golden(开启态)。
- **F2 · 后处理三小件**:vignette/grain/色差(single post shader,参数向量化)→ 置零恒等断言 → golden。
- **F3 · bloom 可选档(时间盒)+ 收口**:Dual Kawase 简版;到盒记遗留;DoD 对账 + 验收帧。

## 3. 客观判定

| 维度 | 判据 |
|---|---|
| off 零变化/零成本 | golden 全集逐字节;off 态无 RT 分配(FrameStats 断言 rt_bytes==0)、帧耗时差 < 噪声 |
| 帧确定 | 固定 dt 序列第 N 帧双跑逐字节(WebGPU;WebGL2 抽查) |
| 收敛自停 | 静止后 feedback pass 自动退出(FrameStats feedback_active 归零,e2e 断言) |
| 恒等 | 每后处理参数置零 == 直通(native shader 参数级断言 + 帧对拍) |
| 显存记账 | rt_bytes == 2×w×h×4×dpr²;resize 后正确重建(0034 路) |

## 4. 铁律

- **AR1/AR2 管线级**:feedback 纯呈现,零回读;core 不知道 feedback 存在(只发参数)。
- **CR3** pass 走 RenderBackend trait 不用 cfg · **R8** 帧序列纯函数(dt 注入)· **0→1**:pass 图按 ADR 一次定,不留"临时挂在 content pass 里"的双轨。
- **内存红线**(承 plan29):RT lazy + off 即释放;长会话 bench 复跑确认 wasmMiB 无回归(perf 门覆盖)。
- 改 render 先 `Skill(render-write)`;**按 milestone commit,不 push**。

## 5. 变更边界

- **主战场**:`crates/render/src/`(backend pass 组织、feedback/post shader 新文件)、core 仅加参数字段与收敛判定(纯数据)、web 面板。
- **非目标域**:噪声库本体(plan36 已定稿,只消费)、预设/场景绑定(plan38)、CRT 整套/流体(远期,通道就绪后 38+ 再选)。
- **与 plan38 衔接**:`FeedbackParams`/post 参数面定稿后 38 只引用;拖尾/残影的"何时开"由 38 预设决定,本 GOAL 只给面板手动开关。

## 6. 风险与预案

| 风险 | 预案 |
|---|---|
| feedback 破坏 golden 体系 | 默认 off 硬门 + 开启态用"固定 dt 第 N 帧"新 golden 家族,与既有互不干扰 |
| 移动端/低端机双全屏 RT 压力 | reduced profile 下 feedback 强制 off(profile 联动);RT 可半分辨率档(参数) |
| WebGL2 blit/格式差异 | 全用 RGBA8 + 标准 sampler,不用 float RT;naga 转译已验路(plan32 D3) |
| bloom 失控(多 pass 复杂) | 明确可选档 + 时间盒;核心交付是 feedback 通道本身,bloom 缺席不影响 DoD 主体 |

> 一句话:**给渲染管线装上"记忆"(feedback)和"最后一笔"(post)两个插槽,off 时不存在、on 时可复现**——拖尾/余晖/氛围一整族效果从此有了管线级入口,而 golden/确定性/内存三条底线一根不松。
