# Plan 36 · GOAL:效果值层地基 —— noise.wgsl 噪声库 + 距离带效果补全 + 首批消费(agent 直跑,无人介入)

- 日期:2026-07-14
- 状态:**GOAL(可执行任务书)** —— 给 Claude Code 的完整执行合同,读完即可自主跑到 DoD。
- 执行方式:**顺序执行,master 工作区直跑**。当前 master = plan28–35 全量成果(五 crate 分层:primitives/components/core/render/wasm;437 测;ADR 至 0039)。
- 定位:**TODO2-A 效果系统第一期(值层)**。[research/shader-effects-catalog](../research/shader-effects-catalog.md) §8 判定的三缺口之一:**噪声家族入库**是溶解/扰动/一切"机理类"效果的前置;顺路把"一步之遥"档的距离带效果(glow/解析阴影/条纹)与首批消费效果(dissolve/hit-flash/spring 曲线)落掉。
- 效果系统总链:plan36(值层地基)→ plan37(反馈通道)→ plan38(预设库)。本 GOAL 不依赖 37/38。
- 前置:
  - [research/shader-effects-catalog](../research/shader-effects-catalog.md)(§1–§3 值层目录 + §8 对照,**本 GOAL 的选型真值,数值/公式从引用链接抄**)
  - [0018 panel 图元](../decision/0018-sdf-panel-decoration-primitive.md)(效果=参数)· [0026 shader 模块化](../decision/0026-modular-shader-organization.md)(include_str! 拼接 + naga 校验——noise.wgsl 照此入库)· [0037 装饰契约](../decision/0037-declarative-part-decoration-contract.md)(DecorSlot 语义槽)· [0025 SDF 节点动画](../decision/0025-sdf-node-animation-system.md)(anim slot ≤4)· [0002 §5.1](../decision/0002-event-driven-pipeline.md)(效果是数据;profile full/reduced/off = 参数置零非分支)
  - 现有资产:`crates/render/src/shaders/` 的 sdf.wgsl(op_smin 已有)/rect.wgsl(gloop 已有)/glyph.wgsl(apply_curve 三曲线);`primitives` 的 motion.rs(CurveId)/theme.rs;glow_orb/shimmer(plan25/28)

---

## 0. 铁律(第一条):增强层,不改默认观感

所有新效果**默认参数 = 恒等**(置零直通)。**全部效果关闭时,现有 golden 全集逐字节不变**——这是本 GOAL 的硬门(同 plan33 纯搬家判据)。效果只经参数开启(0018"效果=数据"),开启态另立新 golden。plan28 的 opencode 参考观感是默认态,效果库是其上的 opt-in 增强层(与节奏系统同定位)。

## 1. GOAL 与完成定义(DoD)

**GOAL**:(A) `noise.wgsl` 噪声函数库入 render(hash/value/gradient/fbm/voronoi,确定性 seed 化);(B) 距离带效果补全进 rect/panel 参数(外发光 glow、解析 drop shadow、等距条纹/cel-band);(C) 首批消费:dissolve 溶解(退场通道)、hit-flash 强调混色、Spring 闭式曲线入 CurveId;(D) 调试面板"效果试衣间"(逐效果旋钮 + debug case)。

**DoD(全满足才算完)**:

1. **noise.wgsl**:hash(Dave Hoskins hash-without-sine 或 PCG 整数 hash,**选定一个并记依据**——跨平台位稳定优先)+ value noise + gradient noise + fbm(≤4 octave,octave 数是参数)+ voronoi F1;全部签名带 `seed` 参数;naga 构建期校验过;一个可视化 debug case(`?noise` 或面板开关,四象限展示)。
2. **确定性**:噪声 = 纯函数 of (seed, uv, time');time' 来自注入时钟(R8);同 seed 同帧输入 → 帧逐字节一致(golden 双跑);WebGL2 路(naga→GLSL)与 WebGPU 路数值一致性抽查(同帧对拍,容差 0,若 GLSL 精度差异则记录并改整数 hash)。
3. **距离带三件**:① glow(`exp(-k·d)` 外发光,panel/rect 参数:色/强度/半径);② **解析 drop shadow**(Evan Wallace 闭式,catalog §2 链接:erf 近似高斯×圆角矩形,单 pass);③ 等距条纹/cel-band(`fract(d/L)` 周期化,参数:间距/占空比)。各配 native 参数快照 + 开启态 golden(dpr=1/2)。
4. **dissolve**:noise 阈值裁剪 + 阈值窄带发光边(Febucci 配方,catalog §4);进度由 0025 slot 驱动(注入 dt,收敛后恒等 AR3);挂到**块退场通道**(0016 exit 的首个真消费者——顺路补 0016 留尾的 exit 淡出:dissolve 是其超集,普通淡出 = dissolve 强度 0 的退化);默认 off。
5. **hit-flash + Spring**:DecorSlot 增强调混色参数(RGB mix 到指定色,时间包络 cubicPulse);`CurveId::Spring`(闭式 `1 - e^{-kt}·cos(wt)`,k/w 进 motion token,纯函数可重放)——glyph 入场可选 spring 曲线(reveal stagger 已有,只换曲线)。
6. **profile 联动**:全部新效果参数受 full/reduced/off 调制(off=置零恒等,0002 §5.1);**动画组 ≤4 / idle 全退场断言保持**(dissolve/flash 收敛后退出活跃集)。
7. `node test/run.mjs` 五层全绿(含 perf 门);**默认态 golden 全集逐字节不变**(DoD 硬门);新增效果开启态 golden ≥4 帧(noise 可视化/glow+shadow/dissolve 中间帧/flash 峰值),dpr=1/2;native ≥4 新测。
8. `spec/plan/plan36_progress.md` 逐 milestone 记账;**按 milestone commit,不 push**,commit 前全门绿。

**启动**:`claude --permission-mode acceptEdits "读 spec/plan/plan36-noise-and-distance-effects-goal.md,按其执行到 DoD 全满足"`(中断重跑同句,读 progress 续跑)。

## 2. Milestones

- **N1 · noise.wgsl 库 + 可视化**:hash 选型(WebGL2 位稳定验证先行)→ value/gradient/fbm/voronoi F1 → 0026 include 拼接 + naga → debug 四象限 case → 双跑帧一致测试。
- **N2 · 距离带三件**:glow / 解析 shadow / 条纹带 → rect+panel 参数扩(storage buffer 布局按 0018 §2 增量通道;参数不够则扩 stride 并记账)→ DecorSlot 语义槽接 0037 → 逐效果 golden。
- **N3 · dissolve 退场 + hit-flash**:dissolve(noise 消费者第一号)接 0016 exit 通道 + 0025 slot;hit-flash 混色 + cubicPulse 包络;两者默认 off,面板可开;native:进度端点恒等/双跑一致。
- **N4 · Spring 曲线 + 面板试衣间**:CurveId::Spring 闭式 + motion token(k/w 保守默认);style-panel 增"Effects"节(逐效果开关+旋钮);验收帧 dpr 双份;DoD 对账。

## 3. 客观判定

| 维度 | 判据 |
|---|---|
| 默认态零变化 | 全效果 off:golden 全集逐字节(硬门) |
| 噪声确定性 | 同 seed 双跑帧一致;WebGPU vs WebGL2 对拍一致(或记录差异并改整数 hash) |
| 参数恒等 | 每效果参数置零 → 输出 == 无效果路径(native 断言) |
| 收敛恒等 | dissolve/flash 结束后 age≥dur 恒等(AR3);idle 活跃组归零 |
| 视觉验收 | 开启态 golden + dpr 双帧;shadow 与现 panel AO 并排人检(不糊不脏) |

## 4. 铁律(承 AGENTS.md / dev-practices §4)

- **AR1/AR2/AR3** 渲染只读、效果只写 presentation、收敛恒等 · **AR10** 参数走既有每帧批量通道。
- **CR1** core 只持参数与进度(纯数据);噪声/效果全在 WGSL(render)· **R8/R9** seed 注入、时钟注入,shader 内无真随机。
- **0026 共识**:noise.wgsl 独立文件 include 复用,不复制粘贴进各 shader;**0→1 一次做对**:hash 选型一次定(位稳定优先),不留双 hash。
- **效果=数据**(0002 §5.1):新效果零新图元、零 if 分支档位(置零即关)。
- 改 render 先 `Skill(render-write)`;**按 milestone commit,不 push**。

## 5. 变更边界

- **主战场**:`crates/render/src/shaders/`(noise.wgsl 新建、rect/panel/glyph.wgsl 参数扩)、`primitives`(motion.rs CurveId/theme 效果色槽)、core 的效果参数/进度通道(0025 slot 消费)、web style-panel。
- **非目标域**:反馈缓冲/后处理(plan37)、预设库/场景绑定(plan38)、partrender/partspecific 语义(效果经 DecorSlot 参数进,不改渲染器逻辑)。
- **与 plan37/38 衔接**:noise.wgsl 是 37 的 grain/热浪、38 的一切机理效果的共享库;dissolve/glow/flash 的参数面即 38 预设表的词汇——**参数命名与默认值在本 GOAL 定稿后 38 只引用不改**。

## 6. 风险与预案

| 风险 | 预案 |
|---|---|
| WebGL2 浮点 hash 位漂移 | 首选 PCG 整数 hash(u32 全整数运算,跨后端位稳定);float hash 仅作对照 |
| storage buffer 参数位不够 | 0018 §2 增量通道扩 stride;实在紧张 → 效果参数并入 DecorSlot 索引表(间接层) |
| dissolve 与 tier 回收撞(退场中块被回收) | 退场动画期锁 Hot(参与 idle 判定);收敛即解锁——native 断言退场块不被中途回收 |
| 解析 shadow 与既有 panel AO 视觉打架 | shadow 仅挂新 DecorSlot,默认 off;38 预设期再定谁当默认 |
| perf 门红(效果开启态) | 默认 off 不影响门;开启态 bench 单独记录,不入门槛(38 期定预算) |

> 一句话:**把"机理"(噪声)和"装饰"(距离带)两个词汇表补进 SDF 世界,并用 dissolve/flash/spring 三个首批消费者证明"效果=参数"的通道端到端可用**——默认观感一个字节不变,增强层从此有了原料库。
