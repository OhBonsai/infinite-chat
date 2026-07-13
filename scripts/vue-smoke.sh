#!/usr/bin/env bash
# Plan 34 S5:Vue harness smoke 一键跑(装依赖 → smoke;dev server 由 spec 自起自杀)。
set -euo pipefail
cd "$(dirname "$0")/.."
(cd web/examples/vue && npm install)
(cd web && npx playwright test tests/vue-smoke.spec.ts)
