# Plan 35 · GOAL:测试工程体系收口 —— plan20 转正 + 真 server 录像 + CI 接门 + 性能回归门(agent 直跑,无人介入)

- 日期:2026-07-13
- 状态:**GOAL(可执行任务书)** —— 给 Claude Code 的完整执行合同,读完即可自主跑到 DoD。
- 执行方式:**顺序执行,master 工作区直跑**(不开 worktree;避开 plan33 搬家窗口)。
- 定位:**P4 工程体系整层收口**。测试地基(plan20 最小管线、plan24 汇总验收)已在且四层门天天在用,但三块欠账从未裂成计划:plan20 至今是"草案待评审"、plan24 的 PR-4(真 server 录像)/PR-5(CI 接门)未做、性能回归靠人眼看 CSV 无门。
- 前置:
  - [plan20-minimal-test-pipeline](./plan20-minimal-test-pipeline.md)(草案;`scripts/verify.sh` + TESTREPORT 已实际落地——本 GOAL 对账转正)
  - [plan24-integration-functional-effect-test](./plan24-integration-functional-effect-test.md)(四层门已绿;**PR-4/PR-5 明确列为后续 = 本 GOAL**)
  - [plan22-opencode-events-and-fsm §人工留验](./plan22-opencode-events-and-fsm.md)(真实 server 联调 / 后台挂起——PR-4 顺带覆盖其可自动化部分)
  - [research/test-driven-dev-and-automation-survey](../research/test-driven-dev-and-automation-survey.md)(§2 确定性重放 = 最强武器 · §3 性能门:阈值 + 变更点检测,headless fps 不可信 → 用确定性代理指标 · §4 agent 可消费输出)
  - [knowledge/opencode.md](../knowledge/opencode.md)(起真 server 的接口知识,动 M1/M2 前必读)
  - 现状:`web/bench-results/*.csv` 已积累(plan18/19/29 基线);replay 库全部是合成事件流;CI 未接。

---

## 0. 原则(承调研 §7 与项目铁律)

- **确定性优先**:一切门槛断言建立在确定性代理指标上(glyph 数/耗时中位数/内存字节/重放快照),**不给 headless fps 设硬门**(调研 §3.3:headless fps 不可信,只作参考记录)。
- **agent 只读 TESTREPORT**(plan20/AGENTS.md §7 既定):新门全部并入 `scripts/verify.sh` → `report/TESTREPORT.md`,不另开报告出口。
- 真网络只在录制时发生一次(T3),录下来永久重放(T3 铁律的合规路径:测试禁真网络,录像不是测试)。

## 1. GOAL 与完成定义(DoD)

**GOAL**:(A) plan20 从"草案"对账转正(已落地项打勾、缺件补齐或明确降级);(B) PR-4:对真 opencode server 录一套全事件真流录像(含难合成的错误/重试/压缩),入 replay 库;(C) PR-5:CI 工作流跑统一门;(D) 性能回归门:基于既有 bench CSV 的阈值 + 历史对比断言。

**DoD(全满足才算完)**:

1. **plan20 转正**:逐条对账草案 vs 现状(verify.sh/TESTREPORT/四层门/每 plan +1 native 惯例),已落地打 ✅、未落地项或补齐或记"明确不做+理由";plan20 头部状态行改「已落地(vN 对账)」;缺件里**必补**的:TESTREPORT 增加机器可读摘要块(JSON:各层 pass/fail/数量/耗时,调研 §4.1)。
2. **PR-4 真 server 录像**:起本地 `opencode serve`(knowledge §7 流程)→ 跑一组真会话脚本(短问答 / 长 markdown / tool 调用 / **人为制造错误:杀 server 重连、压缩触发、慢流**)→ 录成 replay 录像入库(`test/replays/real-server/`);每卷录像配一条重放测试(双跑一致 R8 + 最终 Store 快照断言);plan22 的「真实 server 联调」留验项就此闭环(「后台挂起真机」不可自动化 → 记人工清单,不算欠账)。
3. **PR-5 CI 接门**:GitHub Actions workflow(或等效本地 `scripts/ci.sh`,按仓库托管现状定)——push/PR 触发 `node test/run.mjs` 四层门;headless WebGPU 不可用的 runner 上 native+gates+unit 必跑、e2e 降级跳过并**显式标注**(Fail loud,不静默);工作流文件 + 徽章/说明入 README。
4. **性能回归门**:`scripts/perf-gate.mjs`——读本次 bench 采集(确定性指标:`retained_glyphs`/`frameMs_median`/`wasmMiB`/`ph_*` 分相耗时)对比入库基线 CSV(plan29 after 即基线),超阈值(默认 +10%)即红;门并入 verify.sh/TESTREPORT;基线更新须显式 `--update-baseline`(同 golden 惯例)。Bencher/变更点检测 SaaS **不引**(调研 §3.2 记为升级档,本地对比够用)。
5. **(可选档,时间盒 ≤2 轮)感知 diff + LLM-judge**:golden 对比从逐字节升级可选感知模式(odiff/pixelmatch 任一,阈值宽松,仅新增不替换);LLM pairwise 判图(plan21/23 留的观感主观项)搭一个最小脚本跑一对样张出报告——**跑通即可,不进门槛**;超时间盒直接记遗留。
6. `node test/run.mjs` 四层全绿(含新 perf 门);新增门自身有测试(perf-gate 对构造 CSV 的红/绿判定);录像重放进 native 层计数。
7. `spec/plan/plan35_progress.md` 逐 milestone 记录(做了什么 / 对账表 / 录像清单 / 门阈值 / 遗留)。
8. **按 milestone 正常 commit,不 push**;每个 commit 前全门绿。

**启动**:`claude --permission-mode acceptEdits "读 spec/plan/plan35-test-engineering-goal.md,按其执行到 DoD 全满足"`(中断重跑同句,读 plan35_progress 续跑)。

## 2. Milestones(T1→T5,每个独立 commit)

- **T1 · plan20 对账转正**:草案逐条 vs 现状表格进 progress;补 TESTREPORT 机器可读摘要块;plan20 状态行更新(草案 → 已落地/差异记录)。
- **T2 · 性能回归门**:perf-gate 脚本 + 基线固化(plan29 after CSV 复制为 `test/perf-baselines/`)+ 并入 verify.sh + 红/绿自测 + 文档(AGENTS.md §7 卡口清单加一行)。
- **T3 · PR-4 真 server 录像**:起 server(路径前缀/事件面按 knowledge Tier0/1;版本漂移则回源并回填 knowledge)→ 录制脚本(可重跑,录像文件确定命名)→ 错误场景注入(kill -9 重连 / 长会话触发压缩 / tc 或代理模拟慢流,按可行性取舍并记录)→ 每卷配重放测试 → 入库。
- **T4 · PR-5 CI 接门**:workflow 文件(缓存 cargo/npm;矩阵:native 必跑 + e2e 条件跑)→ headless WebGPU 探测与显式降级标注 → 本地等效入口 `scripts/ci.sh` → README/AGENTS.md 说明。
- **T5 · 可选档(时间盒)**:感知 diff 模式 + LLM-judge 最小脚本;到盒即止,遗留记账。

## 3. 客观判定

| 维度 | 判据 |
|---|---|
| plan20 转正 | 对账表全条目有结论(✅/补齐/明确不做);TESTREPORT 含 JSON 摘要且 schema 有测试 |
| 录像质量 | ≥4 卷真流录像(常规/错误/压缩/慢流);每卷双跑 byte 级一致;重连卷含对账自愈断言(AR4) |
| CI | workflow 在干净环境跑绿(或 ci.sh 本地全绿);e2e 降级路径有显式标注输出 |
| perf 门 | 构造 +15% 恶化 CSV → 门红;基线内 → 绿;误报率:连续 3 次重跑同 build 不翻转(确定性指标保证) |
| 不回退 | 四层门全绿;perf 门对当前 master 绿(基线即当前) |

## 4. 铁律(承 AGENTS.md / dev-practices §4)

- **T3 测试禁真网络**——真 server 只出现在录制脚本(非测试);录像即防腐层。
- **T10** flaky 找根因(perf 门用确定性指标,不给 wall-time 单次值设门;耗时类取中位数 + 宽阈值)。
- **R8/R9** 录像重放确定性是全部新门的地基;录制脚本本身可不确定,录像必须自洽。
- **Fail loud**:CI 任何降级/跳过必须显式标注进 TESTREPORT,禁静默。
- **agent 只读 TESTREPORT**(不自跑不自评);**按 milestone commit,不 push**。
- 动 M1/M2 相关(录制)前必读 knowledge/opencode.md(AGENTS.md §4.1);发现协议漂移回填 Tier1。

## 5. 变更边界

- **主战场**:`scripts/`(verify/ci/perf-gate/录制)、`test/`(replays/perf-baselines/run.mjs 接线)、`.github/workflows/`(若适用)、plan20 状态行、AGENTS.md §7 一行。
- **非目标域**:引擎代码零改动(除非录像重放暴露真 bug——那按 T9 先红后绿单独修,commit 分开);不动 golden 既有断言;不引 Bencher/SaaS。
- **与 plan33 衔接**:先后皆可;若 plan33 先跑,路径按新 crate 布局。

## 6. 风险与预案

| 风险 | 预案 |
|---|---|
| 本机 opencode server 版本与 knowledge 漂移 | 回源 Tier2 重取 + 回填 Tier1(knowledge §7 既定流程);录像以实录为准 |
| CI runner 无 GPU → e2e 全跳 | 显式降级标注 + native/gates/unit 仍是硬门;e2e 留本地跑(plan25/28 同款人机分工) |
| 慢流/压缩场景难触发 | 按可行性取舍:压缩用超长会话逼出;慢流用代理节流;实在不行记遗留并保留注入脚本 |
| perf 门误报(环境抖动) | 只用确定性计数指标设硬门;耗时类中位数 + 宽阈值(+10%)+ 连跑不翻转自测 |
| 时间盒失控(T5) | 硬时间盒 ≤2 轮迭代,到点记遗留收工 |

> 一句话:**plan20 转正、真流录成防腐层、CI 把门自动化、性能有基线红线**——测试体系从"能用"到"闭环",此后每个 plan 的门槛都免费变严。
