---
last_reviewed: 2026-06-29
layer: L0
domain: cross
---

# test/ · AGENTS — 测试怎么用(AI 代理入口)

> 仓库**唯一测试入口**。改完代码 → 跑这一条门确认没回退。详细设计见 [README.md](./README.md);测试体系决策见 [Plan 20](../spec/plan/plan20-minimal-test-pipeline.md)(`verify`)/ [Plan 24](../spec/plan/plan24-integration-functional-effect-test.md)(汇总验收 + 三档)。

## 1. 一条命令(默认门)

```bash
node test/run.mjs            # 跑全部四层(gates + native + unit + e2e),全绿才过
```

- 退出码:任一用例失败 → 非 0 → 可直接当 **CI 门 / pre-push / 提交前自检**。
- 产物在 `test/results/`(gitignored,每次刷新):
  - **`index.html`** = 人看(浏览器开;蓝过/黄跳过/红败)。
  - **`TESTREPORT.md`** = agent 读(`RESULT` + 各层计数 + 失败名)。**先读它,别翻原始日志。**
  - `summary.json`(机器)· `junit/*.xml` · `native.log`/`e2e.log`(失败时翻)。
- 跑完看报告:`node test/run.mjs --open`(自动开 index.html)。

## 2. 四层 + 单跑(迭代时只跑相关层,快)

| 层 | 测什么 | 单跑 |
|---|---|---|
| **gates** | fmt / clippy(-D warnings)/ wasm32 build / tsc | `node test/run.mjs --suite gates` |
| **native** | `crates/*` Rust 单测(解码/FSM/兜底/F1–F12 重放/proptest/契约闸) | `node test/run.mjs --suite native` |
| **unit** | web 纯逻辑(transport `sse-client` 等,vitest) | `node test/run.mjs --suite unit` |
| **e2e** | 浏览器 Playwright(全 part 渲染/Dock/复制选区查找/容错/视觉) | `node test/run.mjs --suite e2e` |

- 过滤:`node test/run.mjs --suite native --filter partrender`(透传底层 runner)。
- 更新视觉黄金图:`node test/run.mjs --suite e2e --update-snapshots`。
- 命令缺失(无 cargo/npx)→ 该层**黄(跳过)**不崩。
- **CI**(Plan 35 T4):`.github/workflows/verify.yml` 跑同一条统一门;ubuntu 无 headless
  WebGPU → `SKIP_E2E=1` 显式降级(e2e/perf 黄跳带原因入 TESTREPORT,gates/native/unit 硬门)。
  本地完整等效:`scripts/ci.sh`。**perf 层**(第五层):e2e bench 采集 vs
  `test/perf-baselines/`,基线更新走 `node scripts/perf-gate.mjs --update-baseline`。

> **改 Rust(crates/) → 至少 `--suite gates` + `--suite native`;改 web/ TS → `--suite gates`(含 tsc)+ `--suite unit`/`--suite e2e`。** 提交前跑一次完整 `node test/run.mjs`。

## 3. 加测的规矩(不动 `run.mjs`)

测试加到**对应层的常规位置**,入口自动纳入(无需改 `run.mjs`):

| 要测的东西 | 放哪 | 怎么写 |
|---|---|---|
| Rust 纯逻辑 / 协议 / FSM / store / 容错 | `crates/<crate>/src/*.rs` 的 `#[cfg(test)]` 或 `crates/<crate>/tests/*.rs` | 确定性、native 可测(守 CR1/R8);见 `/test-write` |
| web 纯 TS 逻辑(无浏览器) | `web/src/*.test.ts` | vitest;可注入 fake 时钟/EventSource(见 `sse-client.test.ts`) |
| 浏览器端到端 / 渲染 / 交互 | `web/tests/*.spec.ts` | Playwright;**用 `chat.push_event(raw)` 合成 opencode 事件驱动,不需真 server**(见下) |
| 视觉黄金帧 | `web/tests/visual.spec.ts` + `*-snapshots/` | `toHaveScreenshot`;裁掉动效列(头像 orb),定帧后比对 |

**仅新增一整层时**才动 `run.mjs`。

### e2e 关键杠杆:`push_event` 合成事件
浏览器测试不依赖真 opencode server——`window.__chat.push_event(JSON.stringify(event))` 直接喂任意事件(part.updated/delta、session.error、permission.asked、server.connected…)驱动整条 FSM/渲染。配 `session_status()`/`stop_turn()`/`reply_permission()`/`scroll_to()` 断言。容错(F1–F12)、各 part 渲染、Dock 全靠它变成🔵自动。

> e2e 配置共性(`web/tests/helpers.ts`):`assertWebGpu`(headless WebGPU 必须起)、`bootVisible`(等 showcase 重放**耗尽**后再 push_event,否则残余 delta 抢 FSM)。

## 4. 排查失败 / 看执行(有头 / 调试)

`test/run.mjs` 是无人值守;**要看 Chrome 怎么跑,用 web 的 e2e 脚本**(经 `npm --prefix web run`,**任何 cwd 都安全**):

```bash
npm --prefix web run e2e:ui                                  # UI 模式:时间旅行 + 单条重跑(最好用)
npm --prefix web run e2e:headed -- parts-render.spec.ts      # 开真 Chrome 窗口看(WebGPU 原生可用)
npm --prefix web run e2e -- dock.spec.ts:7 --headed          # 只跑某行那条用例(行号定位)
npm --prefix web run e2e -- dock.spec.ts --debug             # Inspector 单步
npm --prefix web exec playwright show-report                 # 跑完回看 HTML(每步截图/trace)
```

> **⚠️ 必须在 `web/` cwd 跑 Playwright(或用上面的 `npm --prefix web run`)。** 直接在仓库根 `npx playwright test …` 会**找不到 `web/playwright.config.ts`** → Playwright 用默认配置递归扫描,误把 `web/src/*.test.ts`(vitest 文件)当 e2e 加载 → 报 *"did not expect test()/two versions of @playwright/test"*。等价手动法:`cd web && npx playwright test …`。

- `--headed` 覆盖 config 的 `headless:true`,无需改文件;config 那串 WebGPU flags 是给 headless 的,有头无害。`-g "<正则>"` 按标题过滤(如 `-g permission`)。
- native 失败:读 `test/results/native.log` 或 `cargo test -p <crate> <name> -- --nocapture`。

## 5. 三档边界(哪些自动 / 人审 / 手动)— 详见 [Plan 24 §3](../spec/plan/plan24-integration-functional-effect-test.md)

- **🔵 完整自动化** = 本入口默认门(四层全绿)。**这是日常验收。**
- **🟡 人工确认**(审美:reveal 节奏 / 卡片观感 / 暗主题 / Dock 观感):`--suite e2e --filter visual` 自动出图,人审一次签字;过审转像素回归后并回默认门。**不进默认门。**
- **🔴 手动运行**(`run.mjs` 之外,人亲自造):① 真 opencode server + 真 LLM 冒烟 ② 真机后台节流(僵尸/重连真版)③ 跨 app 粘贴(text/html 保真)④ 多设备/多 OS 快捷键。记录走 [Plan 24 §6 模板](../spec/plan/plan24-integration-functional-effect-test.md)。
