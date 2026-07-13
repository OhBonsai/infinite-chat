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

(未开始)

## T3 · PR-4 真 server 录像(常规/错误/压缩/慢流)

(未开始)

## T4 · PR-5 CI 接门

(未开始)

## T5 · 可选档(感知 diff / LLM-judge,时间盒)

(未开始)

## DoD 对账

(未开始)
