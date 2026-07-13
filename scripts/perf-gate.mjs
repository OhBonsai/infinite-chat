#!/usr/bin/env node
// perf-gate.mjs — 性能回归门 CLI(Plan 35 T2)。
//
// 用法:
//   node scripts/perf-gate.mjs                      # 默认:web/bench-results/plan18-before-browser.csv
//                                                   #       vs test/perf-baselines/browser.csv
//   node scripts/perf-gate.mjs --current a.csv --baseline b.csv
//   node scripts/perf-gate.mjs --update-baseline    # 显式更新基线(同 golden 惯例,绝不自动)
//
// 退出码:0 = 过 / 1 = 恶化超阈值 / 2 = 输入缺失(采集不存在 → 调用方按黄跳处理)。
import { copyFileSync, existsSync, readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { compare, renderTable } from "../test/perf-compare.mjs";

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const args = process.argv.slice(2);
const opt = (name, dflt) => {
  const i = args.indexOf(name);
  return i >= 0 ? args[i + 1] : dflt;
};
const currentPath = path.resolve(ROOT, opt("--current", "web/bench-results/plan18-before-browser.csv"));
const baselinePath = path.resolve(ROOT, opt("--baseline", "test/perf-baselines/browser.csv"));

if (!existsSync(currentPath)) {
  console.error(`perf-gate: 采集不存在: ${currentPath}(先跑 e2e bench)`);
  process.exit(2);
}
if (args.includes("--update-baseline")) {
  copyFileSync(currentPath, baselinePath);
  console.log(`perf-gate: 基线已更新 ← ${currentPath}`);
  process.exit(0);
}
if (!existsSync(baselinePath)) {
  console.error(`perf-gate: 基线不存在: ${baselinePath}(--update-baseline 固化一次)`);
  process.exit(2);
}

const outcome = compare(readFileSync(currentPath, "utf8"), readFileSync(baselinePath, "utf8"));
console.log(renderTable(outcome));
process.exit(outcome.pass ? 0 : 1);
