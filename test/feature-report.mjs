#!/usr/bin/env node
// feature-report.mjs — Plan 39 R4:功能体检一键入口(捕获 spec + 生成报告)。
//
// 用法:
//   node test/feature-report.mjs                 # 全量捕获 → 生成单文件报告
//   node test/feature-report.mjs --only <group|id>  # 增量重捕某组/项 → 重新生成
//   node test/feature-report.mjs --report-only   # 跳过捕获,仅用现有资产重出报告
//
// 常驻资产:每个大 plan 收口后重跑一次,报告即新。产物 test/results/feature-report.html
// (gitignored,重跑即得);不进五层门(manifest schema 测在 unit 层保清单不腐)。
import { spawnSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const argv = process.argv.slice(2);
const only = (() => {
  const i = argv.indexOf("--only");
  return i >= 0 ? argv[i + 1] : "";
})();
const reportOnly = argv.includes("--report-only");

function run(cmd, args, opts = {}) {
  const r = spawnSync(cmd, args, { cwd: ROOT, stdio: "inherit", ...opts });
  if (r.status !== 0) process.exit(r.status ?? 1);
}

if (!reportOnly) {
  // 捕获(独立 Playwright project;报告即测试 —— 任一项失败即非 0 退出)。
  run("npx", ["playwright", "test", "--config", "feature/playwright.config.ts"], {
    cwd: path.join(ROOT, "web"),
    env: { ...process.env, ...(only ? { FEATURE_ONLY: only } : {}) },
  });
}
// 生成单文件报告(资产缺失 → 非 0 退出)。
run("node", [path.join(ROOT, "scripts", "build-feature-report.mjs")]);
