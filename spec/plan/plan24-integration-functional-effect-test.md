# Plan 24:汇总集成测试 —— 完整自动化方案(功能 + 效果)

- 日期:2026-06-24(2026-06-29 扩:§5 测试框架实现)
- 状态:**汇总验收通过(2026-06-29)** —— `node test/run.mjs` 四层全绿:**gates 4/4 · native 263/263 · unit 8/8 · e2e 29/29(共 304 测,0 失败)**。§3.1 完整自动化主体落地(PR-1/2/3 完成);PR-4(真 server 录像)/ PR-5(CI 接门)为后续。Plan 22(事件/FSM/边界/兜底)+ Plan 23(specific 渲染)落地后的**第一次端到端汇总验收**。
  - **本轮新增/补齐 spec(§3.1)**:`parts-render.spec`(TC-F05 tool 三态徽章 / F06 diff / F08 compaction / F09 畸形不崩 / F10 tool error)· `resilience.spec`(F13 stop 冻结 / F21 错误卡恒一张 / F22 ghost-abort)· `canvas-interact.spec`(F18 平移 + 锚底)· `integration-smoke.spec`(F01 历史可见 / F03 流式收尾)· `visual.spec` 扩(E01 无 raw 残留 / E05 滚动零跳变;E07=V2 已有)。已有覆盖:`render-fallback`(E3/F04/F07)· `dock`(F11/F12)· `copy/selection/find`(F15/F16/F17)· `text-layer-virtualized`· `sse-client.test`(E1 退避/超时/僵尸 + E2 重连 resync)。
  - **修**:`playwright.config` junit 改为尊重 `PLAYWRIGHT_JUNIT_OUTPUT_NAME` env → `test/run.mjs` 能收 e2e JUnit、计数准确(原 e2e 失真为 1/1,现 29/29)。
  - **仍为人工/后续(§3.3)**:真 opencode server + 真 LLM 冒烟(PR-4 录像资产);F02 乐观气泡(需 mock POST)/ F24 刷新(需 reload 夹具)/ F14 子 agent 手感(🟡)/ F23 切会话(无 `set_target_session` wasm API,native epoch 已测)/ F19 浏览器真断网(逻辑由 vitest E2 覆盖)。CI 接门(PR-5)。
- 范围:打开一个 session,通过 web 输入输出,验证整条链路:连接 → 发送 → 流式 → 全 part 渲染 → 交互(权限/反问/停止/复制/查找)→ 容错。本轮 = **功能(TC-F)+ 效果/观感(TC-E)**。

---

## 0. 两轴分类(关键:执行 ≠ 判定)

每个 TC 拆**两个独立的轴**,别混:

| 轴 | 含义 | 取值 |
|---|---|---|
| **执行(Run)** | 谁触发测试场景 | 🔵**自动**(脚本/CI 可造,含 mock/合成事件/录像)· 🔴**手动运行**(必须人去做:真断网、真机后台、真 LLM、跨 OS) |
| **判定(Oracle)** | 谁定 pass/fail | 🔵**自动断言**(断言/像素 diff)· 🟡**人工确认**(人眼/审美/感知) |

**三种归属**:
- **完整自动化 = 🔵执行 + 🔵判定** → 进 CI,无人值守(本轮主体)。
- **人工确认 = 🟡判定**(执行多为🔵自动) → 脚本跑出**产物**(截图/录像),人**审一次**;审过后像素回归转全自动。
- **手动运行 = 🔴执行** → 人必须亲自造场景。

> **关键杠杆 `push_event(raw)`**:wasm 暴露了直接喂事件的入口 → 测试可**合成任意 opencode 事件序列**(part.updated/delta、session.error、permission.asked、server.connected…)驱动整条 FSM/渲染,**不需要真 server**。这把一大批原本"难触发"的用例(各 part 渲染、Dock、容错 F1–F12)变成 **🔵自动执行 + 🔵自动判定**。真 server 只留作冒烟。

---

## 1. 功能测试(TC-F)

> 多数用 **合成事件(`push_event`)/ 录像 case** 自动驱动,无需真 server。

| id | 标题 | 执行 | 判定 | 怎么自动化 / 断言 |
|---|---|:--:|:--:|---|
| **TC-F01** | 连接 + 拉历史 | 🔵 | 🔵 | 录像/真 server;断言 `server.connected` 后 `visible_messages()` 非空(历史瞬显) |
| **TC-F02** | 发送 + 乐观气泡 | 🔵 | 🔵 | mock POST(route)发送;断言乐观 user 块立现;`prompt_async` 204 |
| **TC-F03** | 流式文本收尾 | 🔵 | 🔵 | 喂 part.delta×N + session.status idle;断言 assistant 块出现 + `session_status()=idle` |
| **TC-F04** | reasoning 区 | 🔵 | 🔵 | 喂 reasoning part;断言思考区块(角色/标签)出现 |
| **TC-F05** | tool 卡状态变迁 | 🔵 | 🔵 | 喂 tool part 三态(pending→running→completed);断言卡块 + 徽章随 status |
| **TC-F06** | diff 块 | 🔵 | 🔵 | 喂 tool(edit)带 `metadata.filediff`;断言 diff 块 + 增删行数 |
| **TC-F07** | file 附件 | 🔵 | 🔵 | 喂 file part;断言 chip/纹理占位块出现 |
| **TC-F08** | compaction | 🔵 | 🔵 | 喂 compaction part;断言分隔线块 + 标签 |
| **TC-F09** | 兜底/未知 part | 🔵 | 🔵 | 喂未注册 kind / 畸形 part;断言 `[标签]` + 内容在、不崩(render-fallback.spec) |
| **TC-F10** | tool error 卡 | 🔵 | 🔵 | 喂 tool state=error;断言红错误卡 |
| **TC-F11** | permission Dock | 🔵 | 🔵 | 喂 `permission.asked`;断言 Dock 弹出 + 输入框隐藏;点 Allow/Deny → `reply_permission` 调用 + Dock 消失(dock.spec) |
| **TC-F12** | question Dock | 🔵 | 🔵 | 喂 `question.asked`;断言问卷 + 答 → `reply_question`(dock.spec) |
| **TC-F13** | stop 冻结 | 🔵 | 🔵 | 流式中调 `stop_turn`;断言 `Stopped` + 之后喂同消息事件被丢弃 |
| **TC-F14** | 子 agent 跳转 | 🔵 | 🟡 | 喂 task tool + session.created{parent};断言 task 卡 + 点击触发导航(跳转**功能**自动;**手感**人工) |
| **TC-F15** | 每消息复制 | 🔵 | 🔵 | 点复制按钮;`clipboard.readText()` = 渲染纯文本(copy.spec) |
| **TC-F16** | 选区 + Cmd+C | 🔵 | 🔵 | 拖选 + Control+C;断言剪贴板 = 选中文本(selection.spec) |
| **TC-F17** | 跨历史查找 | 🔵 | 🔵 | Cmd+F 查屏外词;断言跳转 + 选中(find.spec) |
| **TC-F18** | 滚动/缩放/锚底 | 🔵 | 🔵 | `pan_by`/`zoom_at`/流式中;断言锚底跟随(底部新内容进可见)、离底不抢 |
| **TC-F19** | 断网重连对账 | 🔵 | 🔵 | **route abort / 关 fake EventSource** 模拟断网 → 恢复;断言重连 + resync,消息不丢不重、时间有序(F6) |
| **TC-F20** | busy 假死看门狗 | 🔵 | 🔵 | 喂 busy 后**停发事件** + 推进时钟(`tick`/`seek`);断言看门狗到点 → 对账 intent(F2) |
| **TC-F21** | 错误卡恒一张 | 🔵 | 🔵 | 喂 `idle` 先于 `message.updated(error)`;断言**恰一张**错误卡(F4);无 finish/error → 兜底卡(F8) |
| **TC-F22** | ghost-abort | 🔵 | 🔵 | 喂 `session.error(AbortError)` 且本轮无 part;断言无错误卡 + 删 temp + 回 Idle(F3) |
| **TC-F23** | 切会话不串台 | 🔵 | 🔵 | `set_target_session` 切换 + 喂旧 session 晚到事件;断言被 epoch 丢弃(F12) |
| **TC-F24** | 刷新不丢历史 | 🔵 | 🔵 | reload + 快照;断言历史完整、不重复 |

> **结论:TC-F 全部可 🔵自动执行 + 🔵自动判定**(借 `push_event` 合成事件 + route 控制 + 录像),仅 F14 的"手感"是 🟡。**这是完整自动化方案的主体。**

---

## 2. 效果 / 观感测试(TC-E)

> 效果分两类:**几何/像素可断言的(🔵判定)** vs **审美/感知的(🟡人工确认)**。执行普遍 🔵自动(定帧:`?replay`+`set_paused`+`seek_reveal`)。

| id | 标题 | 执行 | 判定 | 说明 |
|---|---|:--:|:--:|---|
| **TC-E01** | 流式不闪(无 raw) | 🔵 | 🔵 | 录像逐帧:结构块在 raw 抑制门前不出现 raw glyph(可断言:无 `\|`/半截 ``` 的 raw 角色块) |
| **TC-E02** | reveal 节奏美感 | 🔵 | 🟡 | 定帧序列截图,**人审节奏**(平滑/自主);无客观断言 |
| **TC-E03** | tool 卡观感 | 🔵 | 🟡 | 黄金帧截图,人审徽章配色/折叠;**过审后转像素回归(🔵)** |
| **TC-E04** | diff 观感 | 🔵 | 🟡→🔵 | 同上:首次人审,之后 `toHaveScreenshot` 回归 |
| **TC-E05** | 滚动零跳变 | 🔵 | 🔵 | `?verify` 块 AABB:滚动来回前后同块 y 不变(可断言) |
| **TC-E06** | SDF 缩放清晰 | 🔵 | 🟡 | zoom 截图,人审锐利度(感知,难客观) |
| **TC-E07** | 选区高亮不遮字 | 🔵 | 🔵 | 选区区域采样:文字色像素仍存在(可断言,visual.spec V2) |
| **TC-E08** | 暗主题观感 | 🔵 | 🟡 | 截图,人审配色/对比 |
| **TC-E09** | 规模丝滑(真会话) | 🔴 | 🔵 | **真 server 多轮含工具卡**(手动跑会话)→ `stats()` 断言 fps≥58、内存随可见;合成长会话部分可 🔵 |
| **TC-E10** | Dock 观感 | 🔵 | 🟡 | 截图,人审坞/阻塞态清晰 |

---

## 3. 三张清单(直接回答:哪些自动、哪些人审、哪些手动跑)

### 3.1 ✅ 完整自动化(🔵执行 + 🔵判定)→ 进 CI,无人值守

**TC-F01–F24(除 F14 手感)+ TC-E01 / E05 / E07**。落成 spec(复用 + 新增):

| spec 文件 | 覆盖 | 机制 |
|---|---|---|
| `integration-smoke.spec.ts`(新) | F01 F02 F03 F24 | 录像/route,冒烟链 |
| `parts-render.spec.ts`(新) | F04–F10 | **`push_event` 合成各 part** → 断言块出现 + 兜底(F09) |
| `dock.spec.ts`(已有,扩) | F11 F12 F13 | 合成 permission/question.asked + reply |
| `resilience.spec.ts`(新) | F19 F20 F21 F22 F23 | **合成事件 + route abort + `tick`/`seek` 推时钟** 造 F1–F12 |
| `copy.spec` / `selection.spec` / `find.spec`(已有) | F15 F16 F17 | |
| `canvas-interact.spec.ts`(新) | F18 | `pan_by`/`zoom_at` + 锚底断言 |
| `visual.spec.ts`(已有,扩) | E01 E05 E07 | `?verify` 几何 + 像素采样断言 |

→ `npm --prefix web run e2e` 一条命令全跑;JUnit reporter → `TESTREPORT.md`(Plan 20)。**这是你要的"完整自动化测试方案"。**

### 3.2 🟡 需要人工确认(🔵自动执行 + 🟡人工判定)→ 脚本出截图,人审一次

**TC-E02 节奏美感 · E03 tool 卡观感 · E04 diff 观感 · E06 缩放锐利 · E08 暗主题 · E10 Dock 观感 · F14 子 agent 手感**。

- 做法:`visual.spec` **自动定帧截图**存 `web/test-results/review/`;人审一轮 → 满意的存为黄金基线 → **之后 `toHaveScreenshot` 像素回归转全自动**(E03/E04 即此路径)。
- 即:**执行不用手动,但 pass/fail 需人眼签一次字**(审美无客观 oracle)。

### 3.3 🔴 需要手动运行(🔴执行)→ 人必须亲自造场景

**只有这几类真正要手动跑**:

1. **真 opencode server + 真 LLM 端到端冒烟**(每发布一次):跑一轮真对话,确认合成事件没漏掉真协议的边角(TC-F01–F13 的真服务版 + TC-E09 真规模)。
2. **真机后台节流**(WebView 挂起 2 分钟回前台):验真实节流下的僵尸/重连(F2 真版),浏览器节流 headless 造不出。
3. **跨 OS 剪贴板粘贴到外部 app**(Word/微信):text/html 保真,CI 剪贴板隔离、无法验真实粘贴。
4. **真多设备/多 OS 快捷键**(mac/Win 复制键差异)。

> **一句话:要手动运行的 = ① 真 server+真 LLM 冒烟 ② 真机后台节流 ③ 跨 app 粘贴 ④ 多设备。其余全部脚本自动执行;审美项自动出图、人审一次即可。**

---

## 4. 自动化 harness 设计(让"难触发"变可脚本)

| 难点 | 自动化手段 |
|---|---|
| 各 part 渲染 | `chat.push_event(JSON.stringify(event))` 直接喂 part.updated/delta,不依赖真 server/LLM |
| Dock(权限/反问) | 喂 `permission.asked`/`question.asked` 事件;断言 Dock + 调 `reply_*` |
| 容错 F1–F12 | 喂 `server.connected`/`session.error`/`instance.disposed` + route abort 断 SSE + `tick(dt)`/`seek` 推确定时钟 |
| 流式/节奏 | 录像 case + `set_paused`+`seek_reveal` **定帧**(确定性) |
| 真实协议边角 | 录一份**真 server 会话录像**(含 reasoning/tool/file/compaction/error)存 `web/public/replays/`,e2e 重放 |

录像 fixture(`web/public/replays/integration-*.json`):一次性从真 server 录,之后全自动重放 → **把"真 server 才有的事件"沉淀成可重复的自动化资产**(这也补强 §3.3 ① 的回归)。

---

## 5. 测试框架实现(`test/` 统一入口)

§3.1 要"一条命令全跑 + 出报告",需要一个**仓库级入口**把分散四层聚到一处。不另造测试框架,只做**编排 + 聚合 + 查看**层(类 e2e runner+reporter)。落地 [Plan 20](./plan20-minimal-test-pipeline.md) 的 `verify`。

### 5.1 目录结构

```
test/
├── run.mjs        # 入口:跑各层 → 解析 → 聚合 → 写产物。零依赖(纯 node)
├── README.md      # 用法 + 四层 + 产物 + 与 §3 三档边界
└── results/       # 产物(gitignored,仅留目录)
    ├── index.html     # 人看:蓝过/黄跳过或命令缺失/红败,失败带摘要
    ├── TESTREPORT.md  # agent 读:RESULT + 各层计数 + 失败名(接 Plan 20)
    ├── summary.json   # 机器:全量,CI 聚合/趋势门
    ├── junit/*.xml    # 各层原始 JUnit
    └── *.log          # native/e2e 失败时翻原始输出
```

### 5.2 四层编排(`run.mjs` 只调底层,不重复造)

| 层 | 底层命令 | cwd | 判定来源 |
|---|---|---|---|
| **gates** | `cargo fmt --check` · `clippy -D warnings` · wasm32 `build` · `tsc --noEmit` | root / web | 退出码 |
| **native** | `cargo test --workspace`(partrender 契约闸 + F1–F12 重放 + proptest) | root | 文本解析(passed/failed + 失败名) |
| **unit** | `vitest run --reporter=junit` | web | JUnit |
| **e2e** | `playwright test`(复用 `playwright.config.ts`:自动起 vite、headless WebGPU、单 worker) | web | JUnit(env 重定向到 `results/`,保留 `list` 实时输出) |

用法:`node test/run.mjs [--suite gates|native|unit|e2e|all] [--filter <pat>] [--update-snapshots] [--open]`。退出码随失败非 0 → 可直接当 CI 门 / pre-push。命令缺失(无 cargo/npx)降级为**黄(跳过)**不崩。

### 5.3 与 §3 三档的映射

- **§3.1 完整自动化** = 本入口默认门(`node test/run.mjs`,四层全绿才过)。
- **§3.2 人工确认** = `--suite e2e --filter visual` 自动出图存 `results/`,人审签字;过审转像素回归后并回默认门。**不进默认门**。
- **§3.3 手动运行** = 真 server / 真机 / 跨 app / 多设备,**不在此入口**,人按 §6 记录模板手跑。

### 5.4 落地步骤(PR 切分,每步端到端可跑)

| PR | 做什么 | 状态 |
|---|---|---|
| **PR-1 入口** | `test/run.mjs` 四层编排 + 三产物(html/md/json)+ README;两解析器(JUnit/cargo)单测 | 🔵**已完成**(2026-06-29) |
| **PR-2 补 spec** | `parts-render`(F05/F06/F08/F09/F10)·`resilience`(F13/F21/F22,合成事件驱动)·`integration-smoke`(F01/F03)·`canvas-interact`(F18) | 🔵**已完成**(2026-06-29;F02/F24 需 mock POST/reload 夹具,留后续) |
| **PR-3 扩已有** | `dock`(F11/F12 已覆盖)·`visual` 扩 E01/E05/E07(E07=V2)| 🔵**已完成**(2026-06-29) |
| **PR-4 录像资产** | 录一份真 server 会话 → `web/public/replays/integration-*.json`;smoke 脚本走重放(补 §3.3① 回归) | 🟡 待补 |
| **PR-5 CI 接门** | CI 跑 `node test/run.mjs`,上传 `results/` 制品;`summary.json` 作趋势门 | 🟡 待补 |

> **验收**:PR-2/3 后 `node test/run.mjs --suite e2e` 覆盖 §3.1 全部 TC-F + E01/E05/E07;`node test/run.mjs` 四层全绿 = 本轮汇总验收通过。

### 5.5 加新测的规矩

不改 `run.mjs`。新测加到对应层(`crates/*/tests`、`web/src/*.test.ts`、`web/tests/*.spec.ts`),入口自动纳入。**仅新增一整层时**才动 `run.mjs`。

---

## 6. 测试记录模板

```
轮次: 2026-__-__  环境: opencode <ver> @ localhost:4096 / Chrome <ver> WebGPU
自动(CI):  e2e PASS/FAIL  —  见 TESTREPORT.md
人工确认:  | TC | 截图 | 签字(OK/改) | 备注 |
手动运行:  | TC | 环境 | 结果 | 截图/录像 |
冒烟链(F01/F02/F03)= 必过门。
```

---

## 7. 非目标(本轮)

- 不重测规模回归基线(Plan 18/19 的 fps/内存曲线)。
- 不测 patch/step part 渲染(opencode SKIP)。
- 真 LLM 输出的「内容正确性」不测(测渲染/状态/容错,不测模型答得对不对)。

---

参考:[Plan 22](./plan22-opencode-events-and-fsm.md)(`push_event` 等接口)· [Plan 23](./plan23-part-render-implementation.md)· [0031](../decision/0031-event-fsm-resilience-and-js-rust-boundary.md)(F1–F12)· [0033](../decision/0033-part-render-contract.md)(兜底契约)· [Plan 21](./plan21-text-copy-and-selection.md)· [Plan 20](./plan20-minimal-test-pipeline.md)(TESTREPORT)· `spec/knowledge/opencode.md`。
