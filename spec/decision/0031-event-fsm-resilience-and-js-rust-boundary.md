# 决策记录 0031:事件 FSM 韧性 + JS/Rust 边界(transport 移 TS)

- 日期:2026-06-24
- 状态:**提议中(草案,待评审)**
- 前置:[0001](0001-canvas-architecture.md)(协议/契约)、[0002](0002-event-driven-pipeline.md)(事件管线)、[0003](0003-fault-tolerance.md)(容错/catch-up/resync,本篇扩其韧性)、[0005](0005-turn-aggregation-and-settlement.md)(回合收尾,本篇扩其 FSM)、[0021](0021-js-rust-boundary-and-configurable-render.md)(JS/Rust 边界,本篇据其划线)、`seam::Connection`(已是注入接缝)、`spec/knowledge/opencode.md`(接口真相)
- 外部参考:钉钉《OpenCode Sessions 页面 — 干净实现计划》(失败模式目录 F1–F12;[链接](https://alidocs.dingtalk.com/i/nodes/mweZ92PV6M9Lb2mxUMPa5RPaWxEKBD6p))。**仅取其"解决了哪些问题(F 目录)",不取其 React 缝法。**
- 定位:一份**「韧性 + 边界」合一蓝图**。回答两件其实是一体两面的事:(1)**transport 移 TS** 后,逻辑与 I/O 在 JS/Rust 间怎么划线;(2)**事件 FSM 怎么从「解禁 loading」升级到「全韧性覆盖」**,把 F1–F12 归到 Rust 侧并提炼成确定性重放测试。

---

## 1. 背景:两个一体两面的缺口

- **transport 无韧性**:`crates/wasm/transport.rs` 是 Plan1 stub,代码自述"不做重连/看门狗(留 Plan2,0003)"——只 subscribe + 入队,**无重连/心跳看门狗/连接超时/僵尸检测/cache-bust**。`Connected`/`Heartbeat` 事件解码了但**没接任何韧性逻辑**。
- **FSM 偏粗**:`fsm.rs::TurnTracker` 只有 `Idle/Active/Stalled/Settled`,够"解禁忘了 idle 的 loading"(0005),但**缺** AwaitingAck(发了 204 没回)、Blocked(权限/提问)、Stopped(用户中止)、Errored(错误卡)等状态,F1/F3/F4/F8/F9/F11 都无对应。
- **protocol 偏窄**:`Event` 只有 7 种,**缺** `removed/session.error/compacted/permission/question/created/disposed`——而用户明确要"**从 opencode 消息类型扩展**"。

这两件事一体两面:transport 在 TS、FSM 在 Rust,**正确的划线让韧性逻辑落在可确定性重放的 Rust 核**,而 I/O 落在迭代飞快的 TS。

## 2. 划线原则:按「副作用 / 确定性」,不按「性能」

> **不要**按"性能无关 → TS"划线。`nextStatus`/`reduceMessages` 性能无关,但**恰恰不该**去 TS——它们是 F1–F12 韧性测试的被测对象,放 Rust 才能用录像重放当 oracle 逐字节断言。

| 类别 | 判据 | 归属 |
|---|---|---|
| **副作用 / 平台 I/O** | 真实网络/时钟/不确定 | **TS**:SSE 连接、重连、心跳、连接超时、僵尸检测、cache-bust、`fetch` 快照、abort POST、UI 对话框 |
| **需确定性重放的纯逻辑** | 同输入逐字节同输出、要穷尽 match | **Rust core**:protocol 解码、store 对账、**FSM 转移**、reconcile 合并规则、F1–F12 判定 |
| **纯展示 / 宿主交互** | 渲染/输入框/面板 | **TS**(已是) |

## 3. 边界契约(transport 移 TS)

`Connection` 已是 seam trait(`core::seam::Connection`),transport.rs 只是它的 wasm 实现。移 TS = 把 Rust `SseConnection` 换成 **TS 喂事件队列**,与现有 `replay.ts` 喂重放事件**完全对称**(录制点不变 → 重放确定性不受影响)。

```
TS(web)                                   ║  Rust(core,经 wasm 薄胶水)
transport.ts                              ║  protocol / store / fsm / reconcile
─ EventSource 连接 + subscribe            ║  ─ decode: raw → Event/Part(未知→Ignored,AR12)
─ 重连退避 / 连接超时 / 僵尸检测 / cache-bust ║  ─ store: 三表对账(AR4)
─ 心跳喂"活着"时间戳                       ║  ─ fsm: nextStatus 转移(F1–F12 判定)
─ server.connected → chat.resync(...)     ║  ─ reconcile: 非破坏性 merge + 排序 + epoch
─ 收 Rust 的 reconcile/probe intent → 做 I/O ║  ─ 发 intent(需 probe/reconnect/abort)给 TS
        │ push raw 事件字符串 + 连接生命周期信号 ──▶  消费(与 replay 同入口)
        ◀── intent(reconcileNeeded / probeStatus / abort)
```

**契约**:TS 只产「**raw 事件字符串 + 连接生命周期信号(connected/disconnected/heartbeatTimeout)**」;Rust 产「**解码 + 状态 + 韧性判定 + 给 TS 的 intent**」。

### 3.1 timer 双层(守 R8)

- **确定性逻辑 timer(Rust,`dt_ms` 驱动)**:回合看门狗(F2)、no-reply 计时(F1)、idle 防抖(F7)。它们**只决策状态**,到点发 intent(如 `reconcileNeeded`),**不自己做 I/O**。
- **wall-clock I/O timer(TS)**:重连退避、连接超时 10s、流空闲僵尸 35s、cache-bust。它们**只做 I/O**。
- **桥接**:Rust 发 intent → TS `fetch` 快照 → `chat.resync_from_snapshot(raw)` → Rust 走确定性对账。**决策确定、I/O 在 TS**,两全。

## 4. TS 侧职责(`transport.ts`,照搬钉钉 sse-client)

直接移植钉钉 §6.1 参数(都对应真实事故):指数退避 `min(1000·2^(n-1), 60s)` 连上归零;连接超时 10s;流空闲僵尸 35s(漏 3 次心跳 → abort 重连);cache-bust(WebKit 复用坏连接);`disconnect()`(停重连)vs `forceReconnect()`(保留)分离;`server.connected` → 触发 `resync`;按 Rust intent 执行 `probe /session/status`、`POST /session/:id/abort`、`fetch /session/:id/message`。

## 5. Rust 侧 FSM enrichment(韧性归核)

### 5.1 扩 protocol `Event`(承接"扩展 opencode 消息类型")

新增变体(未知仍 → `Ignored`,守 AR12 向前兼容):`PartRemoved`、`SessionError{name,data}`、`SessionCompacted`、`PermissionAsked/Replied`、`QuestionAsked/Replied`、`SessionCreated{parent_id}`、`InstanceDisposed`。**这是 FSM 韧性的事件燃料**;自定义扩展类型同样走"加变体 + 加转移",不动骨架。

### 5.2 `TurnStatus` → 富 `SessionStatus`(投影式,R8 不变)

```rust
pub enum SessionStatus {
    Idle,
    AwaitingAck { sent_at_ms: f64 },          // 发了 prompt,等首个事件(F1)
    Streaming { message_id: String },
    Retrying { attempt: u32 },                // session.status: retry(F8)
    Blocked { on: Blocked },                  // permission / question(F9 配额也走此)
    Stalled,                                  // soft 超时(保留)
    Stopped,                                  // 用户中止(F11)
    Errored { error: MessageError },          // F3/F4/F8
}
// 派生而非存储:is_active / can_send / streaming_id 全从 status 算(杜绝散落 boolean)
```

### 5.3 `nextStatus` 转移 + timer 即副作用

纯函数 `nextStatus(cur, ev) -> SessionStatus`(穷尽 match,可 proptest);**进入某态即起/停对应 dt_ms timer**(AwaitingAck→no-reply;Streaming/心跳→重置看门狗;离开 active→清 timer),集中一处管理。

### 5.4 reconcile 精炼(扩 0003)

- **`Connected` → resync**:重连即触发 `resync_from_snapshot`(当前 `Connected` 解码了却没接,**直接缺口**)。
- **hard-watchdog → 先 resync 再决策(F2)**:静默超时不再直接 Settle 注卡;**先对账**——服务端其实回完了(终结事件丢了)就补出回复**且不叠错误卡**,否则才注超时卡。
- **非破坏性 merge(F6)**:server 为准但**保留**尚未持久化的 SSE-only chunk,按 `time.created` 排序。
- **epoch 守卫(F12)**:`set_target_session` 切会话即 `epoch++`;晚到的旧 session 快照/intent 按 epoch 丢弃。

### 5.5 错误/中止/去重(纯 Rust)

- **错误卡去重(F4/F8)**:合成卡用 `error-` 前缀;真 error 到清陈旧卡 → **恒一张**;idle 收尾若末条 assistant 既无 finish 又无 error → 兜底注卡。
- **ghost-abort(F3)**:`SessionError(AbortError)` 且本轮无 assistant part → 删 `temp-` 用户消息、**不弹错误卡**、回 Idle。
- **temp 去重(F10)**:真用户事件回来,仅当文本匹配 `temp-` 才视为回显并移除,否则按 assistant。
- **stop 冻结(F11)**:冻结 id 集 + `stopped_at` → 之后丢弃针对冻结消息的事件,**resync 也跳过**(不复活已中止内容);abort 父+子会话由 TS 执行。
- **子会话(F5 相关)**:`SessionCreated{parent_id == 当前}` → 登记子会话,影响事件过滤与 abort 范围。

## 6. F1–F12 归属矩阵(给 Claude Code:每条 = 一个状态/转移 + 一个测试)

| F | 失败模式 | 主逻辑在 | TS 做的 I/O | FSM 状态/转移 |
|---|---|---|---|---|
| F1 | 204 后结果永不到 | **Rust** | probe /session/status | `AwaitingAck` + no-reply timer → resync/注卡 |
| F2 | busy 假死/僵尸流 | **Rust** | 35s 僵尸→重连;probe | hard-watchdog → **先 resync 再决策** |
| F3 | ghost-abort(无 part) | **Rust** | — | `SessionError(Abort)` → 删 temp、无卡、回 Idle |
| F4 | idle 先于 error | **Rust** | — | 错误卡 `error-` 去重 → 恒一张 |
| F5 | 切会话丢首 delta | **Rust**(store 全留 + target 过滤) | — | 子会话登记 + epoch |
| F6 | 重连丢事件 | **Rust**(merge) | connected→resync | 非破坏性 merge + 时间排序 |
| F7 | 多步间瞬时 idle | **Rust** | — | settle ~300ms 防抖(dt_ms) |
| F8 | 503 耗尽无 error | **Rust** | — | idle 收尾兜底注卡;`Retrying` |
| F9 | 额度耗尽(两路径) | **Rust**(抽因) | 配额对话框 | 统一 `Blocked`/回调 + 发送前预检 |
| F10 | temp 去重 | **Rust** | — | 文本匹配才回显移除 |
| F11 | Stop 协议 | **Rust**(冻结/跳过) | abort POST 父+子 | `Stopped` + 冻结集 + resync 跳过 |
| F12 | 切会话串台 | **Rust**(epoch)+ TS(会话生命周期) | — | `epoch++` 丢弃过期快照/intent |

> 12 条里 **10 条主逻辑在 Rust**;TS 只负责 I/O 动作(probe/reconnect/abort/fetch)+ UI 对话框 + 会话生命周期。这正是"韧性归核、I/O 归 TS"。

## 7. 测试(韧性 = 确定性重放,你的护城河)

- **Rust(主力)**:F1–F12 每条 = 一个 **事件序列重放 / proptest** 用例(native 毫秒,录像重放作 oracle)。例:F4 `[SEND, IDLE, MESSAGE_UPDATED(error)]` → 断言"恰一张错误卡";F11 `[STREAMING, STOP, PART(冻结)]` → "该 part 被丢弃";F2 `[SEND,(120s 静默)]`×{busy/idle 有回复/idle 无回复} 三分支。属性不变量:消息按时间有序、错误卡至多一张、temp 终被去重或转正。
- **TS**:`transport.ts` 用 vitest + fake EventSource 测重连退避/连接超时/35s 僵尸自愈/cache-bust;Playwright 测真断网→自动重连→`server.connected`→resync 端到端。
- 接 [Plan 20](../plan/plan20-minimal-test-pipeline.md) `verify` + `TESTREPORT.md`。**钉钉文档"知识提炼成测试"在你这儿是更强版本——你有确定性重放,它只有手搓事件序列。**

## 8. 决策与不决定的

**采纳**:**按「副作用/确定性」划线**(非性能);**transport 移 TS**(Connection seam 喂队列,与 replay 对称);**timer 双层**(确定性逻辑 timer 在 Rust 发 intent,wall-clock I/O timer 在 TS);**扩 protocol Event** 承接 opencode 消息类型扩展(未知→Ignored);**TurnStatus → 富 SessionStatus 投影联合**;**reconcile 精炼**(Connected→resync、hard-watchdog 先对账、非破坏 merge、epoch);**F1–F12 韧性主逻辑归 Rust 并提炼成确定性重放测试**。

**理由**:I/O 在 TS 迭代快、照搬钉钉 sse-client 零成本、调试直观;韧性逻辑在 Rust 得确定性重放 oracle + 穷尽 match + proptest;边界契约让两者解耦且录制点不变,重放不受影响。

**不决定的(留实现/后续)**:具体 timer 数值(no-reply 180s / 僵尸 35s / idle 防抖 300ms 等,实测调);`Blocked` 与现有 permission/question 宿主回调的接法;子会话(subagent)渲染呈现;`SessionCompacted` 后整列失效重拉的触发粒度;intent 通道用回调 vs 轮询 flag;是否升级到完整 reducer/状态机库(0005 升级路径,本篇是其前置)。

## 9. 落地清单(分阶段;边界先行 → FSM enrichment → F 测试)

1. **边界切换**:`Connection` seam 改 TS 喂队列;`transport.ts`(照搬 sse-client:重连/超时/僵尸/cache-bust);删 `transport.rs` 的 SSE 实现(或留 native stub)。端到端:断网→自动重连→`server.connected`→`resync`。
2. **protocol 扩枚举**(§5.1):加 Event 变体 + 解码 + 单测(未知→Ignored 回归)。
3. **FSM enrichment**(§5.2/5.3):`SessionStatus` 联合 + `nextStatus` + timer 副作用;派生量从 status 算。
4. **reconcile 精炼**(§5.4/5.5):Connected→resync、hard-watchdog 先对账、非破坏 merge、epoch、错误卡去重/ghost-abort/temp/stop。
5. **F1–F12 测试**(§7):Rust 重放/proptest 全绿 + TS transport vitest + Playwright 端到端。
6. (可选)0005 升级路径:`nextStatus` 喂 XState,局部替换。

## 10. 参考

钉钉《OpenCode Sessions 干净实现计划》F1–F12 失败模式目录([链接](https://alidocs.dingtalk.com/i/nodes/mweZ92PV6M9Lb2mxUMPa5RPaWxEKBD6p))· [0003](0003-fault-tolerance.md)(容错,本篇扩)· [0005](0005-turn-aggregation-and-settlement.md)(回合收尾,本篇扩 FSM)· [0021](0021-js-rust-boundary-and-configurable-render.md)(JS/Rust 边界)· `core::seam::Connection`(注入接缝)· `web/src/replay.ts`(TS 喂事件先例,与本篇对称)· [Plan 20](../plan/plan20-minimal-test-pipeline.md)(测试流水线)· [测试体系调研](../research/test-driven-dev-and-automation-survey.md)(确定性重放作 oracle)。

> 可信度:transport.rs/fsm.rs/protocol.rs 现状为本仓源码核实(高);钉钉 F 目录为外部经验参考(其自述源自线上事故,作"待规避的坑",非本仓事实)。
