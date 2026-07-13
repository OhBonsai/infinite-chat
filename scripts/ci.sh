#!/usr/bin/env bash
# ci.sh — CI 的本地等效入口(Plan 35 T4):完整统一门(含 e2e/perf,本机有 WebGPU)。
# CI(ubuntu 无 headless WebGPU)跑的是 SKIP_E2E=1 的降级版,见 .github/workflows/verify.yml。
set -euo pipefail
cd "$(dirname "$0")/.."
git checkout -- web/bench-results/plan18-before-browser.csv 2>/dev/null || true
exec node test/run.mjs "$@"
