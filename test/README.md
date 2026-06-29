# test/ —— 统一测试入口

仓库级测试**入口 + 结果 + 查看**。不重复实现测试,只**编排**已有各层并把结果聚到一处(类 e2e 框架的 runner+reporter)。落地 [Plan 20](../spec/plan/plan20-minimal-test-harness.md) 的 `verify`,跑 [Plan 24](../spec/plan/plan24-integration-functional-effect-test.md) 的自动化档。

## 跑

```bash
node test/run.mjs                 # 全部:gates + native + unit + e2e
node test/run.mjs --suite native  # 单层:gates | native | unit | e2e | all
node test/run.mjs --filter partrender          # 过滤(透传底层 runner)
node test/run.mjs --suite e2e --update-snapshots   # 更新 Playwright 黄金图
node test/run.mjs --open          # 跑完用浏览器打开 index.html
```

退出码:任一用例失败 → 非 0(可直接当 CI 门 / pre-push)。

## 四层(底层命令本入口只编排)

| 层 | 跑什么 | 底层 | 判定来源 |
|---|---|---|---|
| **gates** | fmt / clippy(-D warnings)/ wasm32 build / tsc | `cargo` · `npx tsc` | 退出码 |
| **native** | `crates/*` 全部单测(含 partrender 契约闸 + F1–F12 重放 + proptest) | `cargo test --workspace` | 文本解析 |
| **unit** | web 纯逻辑(transport / sse-client) | `vitest run` | JUnit |
| **e2e** | Plan 24 各 spec(render-fallback / copy / dock / find / selection / visual …) | `playwright test` | JUnit |

e2e 复用 `web/playwright.config.ts`:自动起 vite(`reuseExistingServer`)、headless Chromium WebGPU、串行单 worker(剪贴板单例)。本入口只用 `PLAYWRIGHT_JUNIT_OUTPUT_NAME` 把 junit 重定向进 `results/`,保留 `list` 实时输出。

## 结果(`test/results/`,gitignored)

| 文件 | 给谁 |
|---|---|
| `index.html` | **人看**——蓝=过 / 黄=跳过或命令缺失 / 红=败(不用绿),失败带摘要 |
| `TESTREPORT.md` | **dev agent 读**——RESULT + 各层计数 + 失败名/首行断言 |
| `summary.json` | 机器可读全量(CI 聚合 / 趋势门) |
| `junit/*.xml` | 各层原始 JUnit |
| `native.log` · `e2e.log` | 失败时翻原始输出 |

## 与 Plan 24 三档的关系

本入口覆盖 **§3.1 完整自动化**(执行🔵 + 判定🔵)。另两档不进默认门:

- **§3.2 人工确认**(判定🟡):`--suite e2e --filter visual` 自动出图,人审一轮签字 → 过审转像素回归再纳入。
- **§3.3 手动运行**(执行🔴):真 server 冒烟 / 真机后台节流 / 跨 app 粘贴 / 多设备快捷键 —— 不在此入口,见 Plan 24 §3.3。

## 加新测

不改本入口。把测试加到对应层(`crates/*/tests`、`web/src/*.test.ts`、`web/tests/*.spec.ts`),`run.mjs` 自动纳入。本入口只在**新增一整层**时才动。
