# Plan 34 · GOAL:正确性/接入收口包 —— 滚动跟随 FSM + 可点链接 + URL 白名单 + 流式 a11y 播报 + Vue harness(agent 直跑,无人介入)

- 日期:2026-07-13
- 状态:**GOAL(可执行任务书)** —— 给 Claude Code 的完整执行合同,读完即可自主跑到 DoD。
- 执行方式:**顺序执行,master 工作区直跑**(不开 worktree)。当前 master = plan28–31 全量成果(+ plan32 进行中/plan33 待跑——**避开 plan33 搬家窗口**,先后皆可但不交替)。
- 定位:**P3 整层收口**。五项彼此小而独立(每项一个 milestone 一个 commit),打包成一份 GOAL;全部来自 research 已点名、但从未裂成计划的欠账。
- 前置:
  - [research/chat-ui-libs-ai-elements-shadcn](../research/chat-ui-libs-ai-elements-shadcn.md)(**R10 滚动跟随 FSM / R12 URL 白名单 / R13 a11y 播报 / R8 Vue harness**——本 GOAL 的四条直接来源)
  - [research/streaming-chat-ux-standards §1/§3](../research/streaming-chat-ux-standards.md)(滚动三态 = 业界收敛地基;shadcn 9 条场景可直接当测试清单)
  - [research/markdown-parser-comparison §5](../research/markdown-parser-comparison.md)(可点链接:`StyleRole::Link` 补 URL 数据 + hit-test,**不换解析器**)
  - [decision/0031-event-fsm-resilience](../decision/0031-event-fsm-resilience-and-js-rust-boundary.md)(滚动 FSM 并入或引用)· [0030 选区/a11y 镜像](../decision/0030-text-copy-selection-and-a11y-mirror.md) + plan26(**读屏级镜像已落**——本 GOAL 只补流式播报语义,不重做镜像)· [0032 交互架构](../decision/0032-interaction-architecture-sdf-world-not-component-framework.md)(链接命中走它的 ② 路)· [0006](../decision/0006-inline-tags-and-extensibility.md)/[0007](../decision/0007-rich-media-embeds.md)(白名单策略落点)
  - 现状核实:锚底/底部跟随已有(0002 §6 阈值 ~48px);但**无显式三态 FSM、无完整释放信号集**;`StyleRole::Link` 仅配色无 URL/命中(2026-07-13 grep)。

---

## 0. 排序与打包原则

五项独立、无相互依赖,按"越靠近引擎核心越先"排:S1 滚动 FSM(相机层)→ S2 可点链接(内容+命中)→ S3 URL 白名单(安全策略,S2 的守门员,紧随)→ S4 流式 a11y 播报(镜像补强)→ S5 Vue harness(纯外部验证)。每个 milestone 独立 commit、全门绿,中断可续。

## 1. GOAL 与完成定义(DoD)

**GOAL**:把 P3 层五项正确性/接入欠账一次收口——滚动跟随显式 FSM 化(新 ADR)、链接可点且有 URL 安全白名单、读屏按"完成的消息"播报而非逐 token、同一份 wasm 在 Vue 下 smoke 通过。

**DoD(全满足才算完)**:

1. **S1 滚动跟随 FSM**:新 ADR(取当时下一空闲编号,预计 0038;并入或引用 0031)——状态 `Following / Released / Anchoring`;**释放信号全集**:滚轮/触摸/键盘滚动键/拖滚动条/选区拖拽/显式 jump;`scrollToEnd`/jump-to-latest 回到 Following;流式增高时 Released 下**读者视口内容零位移**(布局稳定,streaming-chat-ux §1②)。shadcn 9 条场景落成 native/e2e 测试(R10 原文清单),现有锚底行为作回归基线(Following 态 == 旧行为)。
2. **S2 可点链接**:`content` 给 `StyleRole::Link` 挂 URL sidecar(0020 节点区间同款,不进扁平 glyph 流,AR10);hover → pointer 光标 + 下划线加强(装饰走 0037 DecorationOp);点击命中(0032 ②,settled 几何 AR3)→ 经 `api` 回调 `onOpenUrl(url)` 交宿主(引擎不直接 `window.open`,0000 §2.2 边界);e2e:流式中链接闭合后可点、半截不可点(plan30 raw 抑制不回退)。
3. **S3 URL 白名单**(shadcn R12):图片加载与链接打开前过 **prefix 白名单**(可配置 `allowedLinkPrefixes/allowedImagePrefixes/defaultOrigin`;默认:仅 `https:`/`http:`/`data:image`,拒 `javascript:`/`file:`/未知协议);拒绝路径:链接不派发回调 + 视觉降级为普通文本色,图片走 Failed 兜底(plan14);策略写进 0006/0007 的策略节(append 小节,不新开 ADR);native:协议矩阵测试(恶意 URL 全拒且不 panic,AR12)。
4. **S4 流式 a11y 播报**(shadcn R13,0030 步骤 3 补强):plan26 已落的读屏镜像上补——容器 `role="log"` + `aria-relevant="additions"`;流式中 `aria-busy=true`,**按"完成的 part/消息"追加播报,绝不逐 token**(settle 事件驱动,AR5 投影);viewport `role=region`+`aria-label`+`tabindex=0`;jump 按钮无目标时 `inert`。自动化验 ARIA 属性 + 播报单元粒度(镜像 DOM 断言);真实 VoiceOver 体验记人工清单(plan21 §7 同款)。
5. **S5 Vue harness**(shadcn R8):`web-vue/`(或 `web/examples/vue/`)最小接入——同一份 wasm/pkg,挂 canvas + 输入框 + onOpenUrl/turnSettled 回调;一个 Playwright smoke(加载、流式渲染、滚动跟随、链接点击回调触发);**零改 wasm API**(改了 = 框架耦合,判负)。
6. `node test/run.mjs` 四层全绿;每个 S ≥1 新测红→绿;golden 不因 S1 回归(Following == 旧行为逐字节);验收帧 dpr=1/2(S1 Released 态 + S2 hover 态)。
7. `spec/plan/plan34_progress.md` 逐 milestone 记录(做了什么 / 参考 file:行号 / 遗留)。
8. **按 milestone 正常 commit,不 push**;每个 commit 前全门绿。

**启动**:`claude --permission-mode acceptEdits "读 spec/plan/plan34-correctness-integration-goal.md,按其执行到 DoD 全满足"`(中断重跑同句,读 plan34_progress 续跑)。

## 2. Milestones(S1→S5,每个独立 commit)

- **S1 · 滚动跟随 FSM**:先 ADR(状态图 + 释放信号全集 + 重进入条件 + 布局稳定不变量)→ 相机/input 层实现三态(现锚底逻辑重构进 Following,行为等价)→ shadcn 9 条场景测试化(参考 streaming-chat-ux §1 清单:流式贴底跟随 / 上滚脱离 / 脱离后增高不推视口 / jump 回底 / 选区期间不跟随 / 键盘滚动释放 / 拖滚动条释放 / 触摸释放 / 重进入吸附)。
- **S2 · 可点链接**:content 产 URL sidecar → hit-test 命中(0032 ②)→ hover pointer + 装饰(0037)→ `api.onOpenUrl` 回调 + TS 类型 → e2e(点击派发 / 半截不可点 / 选区拖拽不误触发点击)。
- **S3 · URL 白名单**:配置项(wasm setter + 默认值)→ 链接派发前过滤 + 图片 loader 前过滤 → 拒绝降级视觉 → 协议矩阵 native 测 → 0006/0007 补策略节。
- **S4 · 流式 a11y 播报**:镜像容器语义(log/additions/busy)→ settle 驱动的播报单元(part 完成才入镜像追加区)→ 属性/粒度自动化断言 → 人工 VoiceOver 清单进 progress 遗留。
- **S5 · Vue harness**:最小 Vue 工程(vite,不进主 build)→ 接 wasm 同款初始化 → smoke spec → README 注明"框架无关性回归保护"(0021 划界)。

## 3. 客观判定

| 维度 | 判据 |
|---|---|
| FSM 完备 | 9 场景测试全绿;状态迁移表与 ADR 一致(native 枚举遍历);Following == 旧行为(golden 逐字节) |
| 布局稳定 | Released 态流式增高:视口内既有内容屏幕位移 == 0(e2e 像素断言) |
| 链接 | 闭合可点/半截不可点/回调收到精确 URL;选区拖拽不误触 |
| 白名单 | 协议矩阵:恶意样本全拒不 panic;白名单外图片走 Failed 兜底 |
| a11y | 镜像属性断言 + 播报单元 = settled part(绝无逐 token 节点追加) |
| 框架无关 | Vue smoke 全绿且 wasm API 零改动(git diff 断言 pkg 接口文件) |

## 4. 铁律(承 AGENTS.md / dev-practices §4)

- **AR1/AR3** 命中/滚动只读 settled 几何 · **AR5** 播报单元 = FSM settle 投影 · **AR10** URL sidecar 旁传不进 glyph 流 · **AR12** 恶意输入兜底不 panic。
- **CR1** FSM/白名单判定纯逻辑进 core · **CR5/0000 §2.2** 引擎不执行导航,只回调宿主 · **R8/R9** FSM 迁移确定性(重放含滚动输入)。
- **T1** 测试无 sleep(释放信号显式注入)· **T6** native 优先 · **T9** 先复现再改。
- **新 ADR 而非改旧**(滚动 FSM);0006/0007 只 append 策略节。
- **按 milestone commit,不 push**;改 input/camera 过 AR 清单;改 `web/` 先 `Skill(bridge-write)`。

## 5. 变更边界

- **主战场**:input/camera 滚动态、`content.rs` Link sidecar(不动解析主体)、`api`/wasm 回调面(additive)、web 镜像层、新 `web-vue/`。
- **非目标域**:`partspecific.rs`/装饰(plan32)、crate 结构(plan33)、math/embed(plan31 已收)、节奏(plan30)。
- **与 plan32/33 衔接**:S2 hover 装饰走 0037 DecorationOp(plan32 D0 已落,直接消费);若 plan33 已跑完则文件路径按新 crate 布局,GOAL 语义不变。

## 6. 风险与预案

| 风险 | 预案 |
|---|---|
| FSM 重构破坏现锚底手感 | Following 态先做行为等价重构(golden + 既有 e2e 回归),再加新态 |
| 链接命中与选区拖拽冲突 | 按下即拖 = 选区、松开无位移 = 点击(阈值 ~4px);e2e 双场景 |
| 白名单误伤 data:image 内联图 | 默认白名单显式含 `data:image/`;svg data URI 已有分流(plan31 R4)不受影响 |
| 逐 token 播报难禁绝(镜像增量粒度) | 播报追加区与选区镜像分离:选区镜像可逐块,`aria-live` 区只收 settled part |
| Vue harness 引依赖膨胀 | 独立子工程独立 lockfile,不进主 `web/package.json`;CI 只跑 smoke |

> 一句话:**滚动有状态机、链接能点且守门、读屏不吵、框架不绑**——P3 整层一次收口,全部是研究点过名的确定性欠账,小步快跑五个 commit 完事。
