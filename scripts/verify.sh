#!/usr/bin/env bash
# verify — 一条命令跑全卡口 + 产 report/TESTREPORT.md(Plan 20)。
#
# 设计(见 spec/plan/plan20-minimal-test-pipeline.md):
#   - 独立进程跑(不在开发 session 里);dev agent **只读 report/TESTREPORT.md**,不读原始日志、不自己跑。
#   - 本地与 CI 同一条命令(避免"CI 才暴露")。
#   - 报告只蒸馏失败 + 关键指标(concise),原始日志落 report/*.log 供需要时回溯。
#
# 退出码:PASS→0,任一卡口 FAIL→1(可作 CI gate)。
set -uo pipefail
cd "$(git rev-parse --show-toplevel)" || exit 2
mkdir -p report

# 卡口(失败也继续,收齐全部状态再汇总;原始输出落 report/*.log)。
fmt=$(cargo fmt --all --check >report/fmt.log 2>&1 && echo ok || echo FAIL)
clp=$(cargo clippy --workspace --all-targets --all-features -- -D warnings >report/clippy.log 2>&1 && echo ok || echo FAIL)
tst=$(cargo test --workspace >report/test.log 2>&1 && echo ok || echo FAIL)
wsm=$(cargo build -p infinite-chat-wasm --target wasm32-unknown-unknown >report/wasm.log 2>&1 && echo ok || echo FAIL)
tsc=$( (cd web && npm run --silent typecheck) >report/tsc.log 2>&1 && echo ok || echo FAIL)

# native-test 计数(passed/failed),便于报告一眼看体量。
testline=$(grep -hE '^test result:' report/test.log 2>/dev/null \
  | awk '{p+=$4; f+=$6} END{printf "%d passed / %d failed", p+0, f+0}')
[ -n "$testline" ] || testline="$tst"

# 失败摘要(只抓失败行,喂 agent;不灌原始日志)。
fails=$(grep -hE 'FAILED|panicked|error\[|error:' report/test.log report/clippy.log report/tsc.log 2>/dev/null | head -40)

[ "$fmt$clp$tst$wsm$tsc" = "okokokokok" ] && R=PASS || R=FAIL

{
  echo "# TESTREPORT — $(git rev-parse --short HEAD) @ $(date '+%F %T')"
  echo "RESULT: $R"
  echo
  echo "## 卡口"
  echo "fmt: $fmt | clippy: $clp | native-test: $testline | wasm-build: $wsm | tsc: $tsc"
  echo
  if [ -n "$fails" ]; then
    echo "## 失败"
    echo '```'
    echo "$fails"
    echo '```'
    echo
  fi
  echo "## 复现"
  echo "commit=$(git rev-parse HEAD)"
  echo "# 失败可 replay 复现:确定性录像 → seek_reveal,见 plan20 §1.1 / replay-debug。"
} > report/TESTREPORT.md

echo "→ report/TESTREPORT.md ($R)"
[ "$R" = PASS ]
