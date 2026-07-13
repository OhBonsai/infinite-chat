# Plan 35 · 进度账本(测试工程体系收口)

> 逐 milestone 记录:做了什么 / 对账表 / 录像清单 / 门阈值 / 遗留。
> GOAL:[plan35-test-engineering-goal](./plan35-test-engineering-goal.md)

## T1 · plan20 对账转正 + TESTREPORT 机器可读摘要

### 对账表(草案 vs 现状,2026-07-14)

| 草案条目 | 现状 | 结论 |
|---|---|---|
| `verify` 一条命令 | `node test/run.mjs` 四层门(plan24)天天在用;`scripts/verify.sh` 在,作单机聚合别名 | ✅(入口换名,语义达成) |
| `report/TESTREPORT.md` | `test/results/TESTREPORT.md`(gitignored,每跑刷新) | ✅(路径差异,已记 plan20 头) |
| agent 只读 TESTREPORT | AGENTS.md §7 / test/AGENTS.md 已成规 | ✅ |
| insta 确定性重放快照 | `crates/core/tests/replay.rs` + 众多 insta 快照(render_snapshots 等) | ✅(远超 1 个) |
| proptest 不变量 | hunk/协议/store 多处 proptest | ✅ |
| 每 plan +1 native 测 | 惯例成立(plan32 +7 / plan34 +13) | ✅ |
| 失败摘要 concise(名+断言+file:line) | TESTREPORT「失败/跳过」节 | ✅ |
| **机器可读摘要(调研 §4.1)** | 原只有 summary.json 旁文件 | **本次补齐**:TESTREPORT 内嵌 SUMMARY-JSON 块 |
| 性能段(可选) | 无门(CSV 靠人眼) | → T2 落 perf-gate |
| nextest/iai/Bencher 升级档 | 未装 | 明确不做(升级档按需,GOAL 亦禁 Bencher) |

### 本次改动

- `test/report-format.mjs`(新):`machineSummary`/`summaryJsonSection`/`parseSummaryJson`
  纯函数;run.mjs 引用,TESTREPORT 末尾内嵌 `## SUMMARY-JSON` 围栏 json 块
  (result/commit/stamp/totals/suites[{suite,passed,failed,skipped,ms}];cases 明细
  不入 —— concise 契约,细节留 summary.json)。
- `web/src/report-schema.test.ts`(新,unit 层自动纳入):生成→抽回往返 + 键集断言 +
  缺段/坏 JSON 容错(消费端不 throw)。
- plan20 头部状态行:草案 → **已落地(对账转正)**,差异三行记录在案。
- **遗留**:scripts/verify.sh 未接 SUMMARY-JSON(它是 run.mjs 之外的旧别名;是否退役
  待作者定夺,不擅删)。

## T2 · 性能回归门(perf-gate + 基线)

- **基线固化**:`test/perf-baselines/browser.csv` = T1 gate(commit 91f4a4e 前夕)在本机的
  bench 采集(plan29 之后的当前 master 稳态;「基线即当前」判据)。
- **比较逻辑**(`test/perf-compare.mjs`,纯函数):结构指标 `retainedGlyphs`/`wasmMiB`
  取末行、阈值 **+10%**;耗时 `frameMsAvg`/`phBfTotal`/`phAdvance` 取**中位数**、宽阈值
  **+25%**(T10:不给单次 wall-time 设门);`fps` 只记录不设门(调研 §3.3 headless 不可信);
  缺列/空 CSV → 指标跳过不假红(采集面变化不该红)。
- **CLI**(`scripts/perf-gate.mjs`):默认比 `web/bench-results/plan18-before-browser.csv`
  (bench.spec 每次 e2e 的输出)vs 基线;`--update-baseline` 显式固化(同 golden 惯例);
  退出码 0/1/2(过/恶化/输入缺失)。
- **run.mjs 第五层 `perf`**(新增一整层,按规可动 run.mjs):e2e 后跑,消费同一门里
  bench 刚写的采集;**陈旧输入守卫**——采集 mtime 早于本次运行启动(单跑/被跳/读到 git
  里的旧 CSV)→ 黄跳显式标注,不假红不静默。
- **红绿自测**(`web/src/perf-gate.test.ts`,unit 层自动纳入):基线内绿 / 构造 +15% 结构
  恶化红 / fps 恶化不判负 / 缺列容错 + 同输入同判定(确定性)。
- **文档**:AGENTS.md §7 加 perf 门一行(基线更新命令)。
- **踩坑**:git 里 tracked 的 plan18-before-browser.csv 是**陈旧**采集(bench.spec 每跑覆写、
  commit 前 restore 的习惯由此而来)—— 曾被门误读假红,故加 mtime 守卫。
- **遗留**:误报率判据(同 build 三连跑不翻转)待 T4 CI 环境验证;结构指标在不同机器
  应一致,耗时中位数跨机器阈值可能需按 runner 分基线(记 T4)。

## T3 · PR-4 真 server 录像(常规/错误/压缩/慢流)

### 录像清单(test/replays/real-server/,[{t,raw}] 同 web replay 格式)

| 卷 | 事件数 | 时长 | 覆盖 |
|---|---|---|---|
| qa-short | 127 | ~64s | 短问答;45×plugin.added 等未知事件(AR12 真世界样本) |
| markdown-long | 158 | ~190s | 表格/代码块/列表/链接的长 markdown 流式 |
| tool-call | 78 | ~180s | bash 工具调用(tool part 17×updated;权限自动放行) |
| kill-reconnect | 187 | ~123s | **kill -9 中断 → 静默 → 重启 → server.connected ×2 → 续问 + 「上一条消息未送达」** |

- **录制器**:`scripts/record-real-server.mjs`(SSE 全录含 heartbeat 噪音;权限轮询自动
  放行;kill-reconnect 自起自管 4097)。真网络只在录制发生(T3 合规);模型
  aliyuntokenplan/qwen3.7-max(本机 provider)。
- **重放测试**(`crates/core/tests/real_server_replay.rs`,native 层自动计数 +4):每卷
  双跑逐字节一致(R8)+ 末态 insta 快照;kill 卷验断前内容不丢 + 重连事件重放稳定。
- **踩坑**:①SSE fetch 无法优雅取消 → 录制器 main 后必须 `process.exit`(否则进程挂死,
  `&&` 链断);②`waitDone` 的 quiet 判定被 ~10s 心跳刷新 → 实际等满 180s cap 才收卷
  (录像变长无害,时间成本记录在案);③kill 卷重放末态 status=streaming(录像止于
  session.idle 前的对账窗,快照如实锁定)。
- **取舍(按 GOAL 允许记录)**:压缩触发 —— 无 summarize/compact 端点,须自然长会话
  (token 成本高)→ 不录,记 plan22 人工留验;慢流 —— tc/代理需 root/额外依赖 → 不录
  (kill 卷的断线静默已覆盖"事件间大空洞"这一重放形态)。
- **plan22 留验闭环**:「真实 server 联调」可自动化部分就此闭环;「后台挂起真机」仍人工。

## T4 · PR-5 CI 接门

- **run.mjs**:`SKIP_E2E=1` → e2e/perf 两层**显式降级**(黄跳,跳过条目带原因文本入
  TESTREPORT/summary —— Fail loud,禁静默;perf 随 e2e 因为它消费 bench 采集)。
- **workflow**(`.github/workflows/verify.yml` 升级):原 verify.sh 五卡口 → 统一门
  `SKIP_E2E=1 node test/run.mjs`(cargo/npm 双缓存;wasm32 build 走 cargo 无需 wasm-pack);
  test/results(TESTREPORT + summary.json + junit)恒上传 artifact;push/PR/手动触发。
  ubuntu runner 无 headless WebGPU 是降级根因(调研 §3.3 同源)。
- **本地等效**:`scripts/ci.sh` = 完整门(含 e2e/perf;先还原 bench CSV);README 加
  verify 徽章;test/AGENTS.md 加 CI/perf 说明。
- **验证**:`SKIP_E2E=1 node test/run.mjs --suite e2e` → PASS(有跳过)+ 显式标注 ✅;
  完整门本地绿(commit 前照常)。
- **遗留**:①workflow 在真 GitHub runner 上的首绿待 push 后观察(本 goal 不 push,
  规则如此);②有 GPU 的 self-hosted runner 矩阵(跑全量 e2e)记升级档;③耗时类 perf
  阈值跨机器基线(T2 遗留)在 CI 因 SKIP_E2E 暂不触发。

## T5 · 可选档(感知 diff / LLM-judge,时间盒)

- **感知 diff**(`scripts/perceptual-diff.mjs`,零新依赖,pngjs 自 web/):逐像素色差容忍
  (默认 ≤12/255)+ 变化占比阈值(默认 ≤0.5%);自比 PASS 0%、折叠 vs 展开 FAIL 26.8%
  —— 红绿都验。**不进门槛**(GOAL 明示仅新增;golden 仍逐字节)。
- **LLM-judge**(`scripts/llm-judge.mjs`):两样张 data-URL file part → 本地 opencode
  盲评 A/B。管线全通(script→opencode→provider),但配置模型 qwen3.7-max 纯文本,
  上游拒图(`InternalError.Algo.InvalidParameter: Unexpected item type in content`)。
  **遗留**:配 vision 模型(如 qwen-vl)后原脚本即用;时间盒到此止。

## DoD 对账

1. **plan20 转正** ✅ 对账表全条目(✅/补齐/明确不做);TESTREPORT 内嵌 SUMMARY-JSON,
   schema 测试锁定(T1,commit 91f4a4e)。
2. **PR-4 真 server 录像** ✅ 4 卷(常规×2/tool/kill-重连)双跑逐字节一致 + insta 快照;
   重连卷含对账自愈素材;压缩/慢流不可行取舍已记(T3,commit 901cb60)。
3. **PR-5 CI** ✅ verify.yml 统一门 + SKIP_E2E 显式降级 + scripts/ci.sh + README 徽章
   (T4,commit 474e00b);真 runner 首绿待下次 push 观察(本 goal 不 push)。
4. **perf 门** ✅ 第五层 + 基线固化 + 红绿自测 + --update-baseline 惯例(T2,commit
   2e76892);实跑绿(437/437 含 perf 1/1)。误报率三连跑判据记 CI 环境遗留。
5. **可选档** ✅(时间盒内)感知 diff 红绿通;LLM-judge 管线通、缺 vision 模型记遗留。
6. **门自测** ✅ perf-gate 构造 CSV 红/绿(unit);录像重放入 native 计数(+4)。
7. **记账** ✅ 本文件(对账表/录像清单/门阈值/遗留俱全)。
8. **commit 政策** ✅ T1 91f4a4e · T2 2e76892 · T3 901cb60 · T4 474e00b · T5 本次;不 push。
