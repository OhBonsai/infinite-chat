# Plan 20:最小测试流水线(可迭代起点)

- 日期:2026-06-24
- 状态:**草案(待评审)**;从 [测试体系调研](../research/test-driven-dev-and-automation-survey.md) 抽出的**最小可用版**,刻意只做 20% 拿 80% 价值,其余升级档按需取用。
- 前置:[调研报告](../research/test-driven-dev-and-automation-survey.md)(全景,本 plan 是其精简落地)、[Plan 18](./plan18-scale-memory-verification.md)(已有 `?bench`)、AGENTS.md §7(已有卡口命令)。
- 目标:**一条命令、一份报告、逐个加测**。让"开发(agent)↔ 测试"解耦且数据可回传,且日常迭代**零心智负担**。

---

## 0. 一句话方案

> **`verify` 一条命令(独立进程跑)→ 产出一份 `TESTREPORT.md`(agent 只读它)→ 每完成一个 plan 顺手加 1 个 native 测试。** 就这些。其余(nextest/iai/Bencher/视觉/LLM 判图)都是**可选升级档**,想要再装。

为什么够用:本项目已有 `cargo test`+`proptest`、`?bench`、`?verify`、卡口命令——**散件齐了**,缺的只是"**一个入口 + 一份给 agent 的摘要**"。这就是本 plan 的全部增量。

---

## 1. 最小三件套(够用,不复杂)

| 维度 | 用什么 | 增量 |
|---|---|---|
| **功能** | `cargo test`(已有,含 `proptest`) | 0,继续用 |
| **确定性回归(护城河,最值)** | `insta` 快照:**1 个**重放 case 的末状态/`FrameData` | 引 1 个 dev-dep + 写 1 个测试 |
| **性能** | 复用已有 `?bench`(Plan 18) | 0,偶尔人工跑、记基线 |

**只新增两样东西**:① 一个 `verify` 脚本(聚合卡口 + 产报告);② `insta` 的第一个重放快照测试。别的都不动。

### 1.1 为什么"确定性重放快照"是第一个该加的测试

本项目 R8 确定性 + record/replay 已就位 → "**同 case 同 seed → 同末状态/同画面**"天然成立。于是**一个**测试就能守住一大类回归(闪/跳变/排版错/卡死):

```
录像 case → seek_reveal(固定 target_ms) → 序列化「末状态 + 关键帧 FrameData(redact 掉 spawn_time)」→ insta 快照
```

改了 reveal/layout/虚拟化后,`cargo insta review` 一眼看出"画面变没变、变得对不对"。**比写一堆断言便宜,覆盖面又大。** 这是调研里 §2/§7-L2 的精华,单点落地即可。

---

## 2. 数据契约:就一个文件 `report/TESTREPORT.md`

dev agent **只读这一个文件**,不读原始日志、不自己跑测试、不自评。`verify` 跑完写它:

```markdown
# TESTREPORT — <commit> @ <时间>
RESULT: PASS | FAIL

## 卡口
fmt: ok | clippy: ok | native-test: 178 passed / 0 failed | wasm-build: ok | tsc: ok

## 失败(若有)
- core::reveal::tier_monotonic  —  assertion failed: t1 < tp  (reveal.rs:553)
- ...(只列失败:名 + 断言 + file:line)

## 性能(可选,跑了 ?bench 才有)
fps@10k: 59 (基线 60, Δ-1)  |  retained_glyphs@10k: 2300 (基线 466650 ✅回落)

## 复现
seed=<...>  commit=<...>   # 失败可 replay 复现
```

要点(全部来自调研结论):
- **解耦**:`verify` 在**独立进程/CI** 跑,不在开发 session 里。
- **concise**:只蒸馏失败 + 关键指标,**不灌原始日志**(context 填满会降智)。
- **不自评**:写代码的 session 读报告判断;主观项(观感)以后交独立 subagent / 人。
- **可复现**:带 seed+commit,失败能 replay。

`report/` 进 `.gitignore`(临时产物);CI 里作 artifact。

---

## 3. `verify` 脚本(骨架,直接可抄)

放 `scripts/verify.sh`(或包成 `just verify` / `cargo xtask verify`)。**本地与 CI 同一条命令**(避免"CI 才暴露"):

```bash
#!/usr/bin/env bash
# verify — 一条命令跑全卡口 + 产 report/TESTREPORT.md。独立进程,dev agent 只读报告。
set -uo pipefail
mkdir -p report
fmt=$(cargo fmt --all --check     >/dev/null 2>&1 && echo ok || echo FAIL)
clp=$(cargo clippy --workspace --all-targets -- -D warnings >report/clippy.log 2>&1 && echo ok || echo FAIL)
tst=$(cargo test  --workspace      >report/test.log 2>&1 && echo ok || echo FAIL)
wsm=$(cargo build -p infinite-chat-wasm --target wasm32-unknown-unknown >/dev/null 2>&1 && echo ok || echo FAIL)
tsc=$( (cd web && npx tsc --noEmit) >/dev/null 2>&1 && echo ok || echo FAIL)
# 失败摘要(只抓失败行,喂 agent)
fails=$(grep -E 'FAILED|panicked|error\[' report/test.log report/clippy.log | head -40)
[ "$fmt$clp$tst$wsm$tsc" = "okokokokok" ] && R=PASS || R=FAIL
{
  echo "# TESTREPORT — $(git rev-parse --short HEAD) @ $(date '+%F %T')"
  echo "RESULT: $R"; echo
  echo "## 卡口"; echo "fmt:$fmt clippy:$clp native-test:$tst wasm-build:$wsm tsc:$tsc"; echo
  [ -n "$fails" ] && { echo "## 失败"; echo '```'; echo "$fails"; echo '```'; }
} > report/TESTREPORT.md
echo "→ report/TESTREPORT.md ($R)"
[ "$R" = PASS ]
```

(性能段可选:想加时,`verify` 末尾追一句"跑 `npm --prefix web run bench` 并把 fps/retained 摘进报告"。)

---

## 4. 迭代约定(零负担长大)

**每完成一个 plan,顺手加 1 个 native 测试**,纳入 `verify`。优先级:

1. **确定性重放快照**(改了渲染/reveal/虚拟化)——最高价值。
2. **proptest 不变量**(改了 store/reveal/boxlayout/虚拟化往返)——如 0029 的 release↔rebuild 逐字节等价。
3. **example 断言**(具体已知 case / 回归 pin)。

合同(借 SWE-bench,防糊弄):新增测试必须**红→绿**(证目标达成),且 `verify` 既有项**保持绿**(证无回归)。就这两条,够了。

---

## 5. 升级阶梯(想要再装,每格对应调研某节)

| 想要 | 装什么 | 调研出处 |
|---|---|---|
| 测试隔离 + JUnit + flaky 重试 | `cargo test` → **nextest** | §5 |
| 性能 CI 硬门(不被噪声坑) | `?bench` 热路径接 **iai-callgrind**(指令数);时间序列变更点门用 **Bencher** | §3 / §7-L3 |
| 帧时可信 | `?bench` 改报 **P95 帧时 + jank**,N≥5 取中位,`--disable-gpu-vsync` | §3.3 |
| 视觉不跳变 | `?verify` 截图 + **Playwright `toHaveScreenshot`**(紧容差,exact) | §1 / §7-L4 |
| 观感不回退(主观) | **LLM pairwise 判图**(软信号,人拍板) | §4.3 / §7-L4 |
| lint 进 GitHub | clippy 出 **SARIF** | §4.1 |
| agent 自动回灌 | Stop hook / `claude -p --output-format json` / MCP | §4.5 |

**原则:不一次性全装。** 起点只跑 §1–§4;遇到具体痛点(性能误报、视觉回归、观感漂移)再从本表取对应一格。

---

## 6. 落地步骤(一个小 PR 即可起步)

1. 写 `scripts/verify.sh`(§3)+ `report/` 进 `.gitignore`。
2. 引 `insta` dev-dep + 写**第一个**重放快照测试(§1.1)。
3. README/AGENTS.md 加一行:"提交前跑 `verify`,agent 读 `report/TESTREPORT.md`"。
4. (可选)CI 跑同一条 `verify`,`TESTREPORT.md` 作 artifact。

> 完成这 4 步 = 拥有"一条命令 + 一份 agent 可读报告 + 一个护城河测试"。**之后每个 plan 顺手 +1 测试,按 §5 需要才升级。**

---

参考:[测试体系调研](../research/test-driven-dev-and-automation-survey.md)(全景与取舍)· [Plan 18](./plan18-scale-memory-verification.md)(`?bench`)· AGENTS.md §7(卡口)· [insta](https://insta.rs/docs/)。
