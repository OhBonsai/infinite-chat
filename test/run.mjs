#!/usr/bin/env node
// test/run.mjs — 统一测试入口(类 e2e 框架):跑各层测试 → 聚合 → 产出结果与报告。
//
// 用法:
//   node test/run.mjs                      # 跑全部(gates + native + unit + e2e)
//   node test/run.mjs --suite native       # 只跑某层:gates|native|unit|e2e|all
//   node test/run.mjs --filter partrender  # 过滤(传给底层 runner)
//   node test/run.mjs --suite e2e --update-snapshots   # 更新 Playwright 黄金图
//   node test/run.mjs --open               # 跑完用默认浏览器打开 index.html
//
// 产出(test/results/,gitignored):
//   junit/*.xml      各层 JUnit
//   summary.json     机器可读汇总
//   TESTREPORT.md    给 dev agent 读的精简摘要(失败名+断言,接 Plan 20)
//   index.html       人可看的报告(蓝/黄/红,不用绿)
//
// 退出码:任一层失败 → 非 0(可作 CI 门 / `verify`)。
//
// 各层底层命令(本脚本只编排,不重复实现):
//   gates  : cargo fmt --check / clippy -D warnings / web tsc / wasm32 build
//   native : cargo test --workspace(含 partrender 契约闸 + F1–F12 重放)
//   unit   : web vitest(transport/sse-client)
//   e2e    : web playwright(Plan 24 各 spec)

import { spawnSync } from "node:child_process";
import { mkdirSync, writeFileSync, readFileSync, existsSync } from "node:fs";
import { resolve, dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const SELF = dirname(fileURLToPath(import.meta.url));
const ROOT = resolve(SELF, "..");
const WEB = join(ROOT, "web");
const RESULTS = join(SELF, "results");
const JUNIT = join(RESULTS, "junit");

// ───────────────────────── args ─────────────────────────
const argv = process.argv.slice(2);
const flag = (n) => argv.includes(n);
const val = (n, d) => {
  const i = argv.indexOf(n);
  return i >= 0 && argv[i + 1] && !argv[i + 1].startsWith("--") ? argv[i + 1] : d;
};
const SUITE = val("--suite", "all");
const FILTER = val("--filter", "");
const UPDATE = flag("--update-snapshots");
const OPEN = flag("--open");
const want = (n) => SUITE === "all" || SUITE === n;

mkdirSync(JUNIT, { recursive: true });

// ───────────────────────── helpers ─────────────────────────
function sh(cmd, args, cwd, env = {}) {
  const t0 = Date.now();
  const r = spawnSync(cmd, args, {
    cwd,
    env: { ...process.env, ...env },
    encoding: "utf8",
    shell: false,
  });
  const ms = Date.now() - t0;
  if (r.error && r.error.code === "ENOENT") {
    return { ok: false, missing: true, ms, out: `命令不存在: ${cmd}` };
  }
  return { ok: r.status === 0, code: r.status ?? -1, ms, out: (r.stdout || "") + (r.stderr || "") };
}

const tail = (s, n) => s.trim().split("\n").slice(-n).join("\n");
const esc = (s) => String(s).replace(/[&<>"]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" }[c]));

// 极简 JUnit 解析(正则,无依赖):统计 testcase + 抓 failure/error。
function parseJUnit(xml) {
  const cases = [];
  const caseRe = /<testcase\b([^>]*?)(\/>|>([\s\S]*?)<\/testcase>)/g;
  let m;
  while ((m = caseRe.exec(xml))) {
    const attrs = m[1];
    const body = m[3] || "";
    const name = (/\bname="([^"]*)"/.exec(attrs) || [])[1] || "?";
    const cls = (/\bclassname="([^"]*)"/.exec(attrs) || [])[1] || "";
    const failed = /<(failure|error)\b/.test(body);
    const skipped = /<skipped\b/.test(body);
    const msg = (/<(?:failure|error)\b[^>]*message="([^"]*)"/.exec(body) || [])[1] || "";
    cases.push({ name: cls ? `${cls} › ${name}` : name, failed, skipped, msg });
  }
  return cases;
}

// cargo test 文本解析:汇总 passed/failed + 失败用例名。
function parseCargo(out) {
  const failures = [];
  const failRe = /^test (\S+) \.\.\. FAILED$/gm;
  let m;
  while ((m = failRe.exec(out))) failures.push({ name: m[1], failed: true, msg: "" });
  let passed = 0,
    failed = 0;
  const resRe = /test result: \w+\. (\d+) passed; (\d+) failed/g;
  let r;
  while ((r = resRe.exec(out))) {
    passed += +r[1];
    failed += +r[2];
  }
  return { passed, failed, failures };
}

// ───────────────────────── runners ─────────────────────────
function runGates() {
  const steps = [
    ["fmt", "cargo", ["fmt", "--all", "--check"], ROOT],
    ["clippy", "cargo", ["clippy", "--workspace", "--all-targets", "--", "-D", "warnings"], ROOT],
    ["wasm-build", "cargo", ["build", "-p", "infinite-chat-wasm", "--target", "wasm32-unknown-unknown"], ROOT],
    ["tsc", "npx", ["tsc", "--noEmit"], WEB],
  ];
  const cases = [];
  let ms = 0;
  for (const [name, cmd, a, cwd] of steps) {
    const r = sh(cmd, a, cwd);
    ms += r.ms;
    cases.push({ name, failed: !r.ok, skipped: !!r.missing, msg: r.ok ? "" : tail(r.out, 25) });
  }
  return suiteOf("gates", cases, ms);
}

function runNative() {
  const a = ["test", "--workspace"];
  if (FILTER) a.push(FILTER);
  const r = sh("cargo", a, ROOT);
  writeFileSync(join(RESULTS, "native.log"), r.out);
  const p = parseCargo(r.out);
  // 无法解析(命令缺失/早退)时,整体作一个用例。
  const cases =
    p.passed + p.failed > 0
      ? [
          ...p.failures.map((f) => ({ name: f.name, failed: true, msg: "见 native.log" })),
          { name: `cargo test (${p.passed} passed)`, failed: false, msg: "" },
        ]
      : [{ name: "cargo test", failed: !r.ok, skipped: !!r.missing, msg: r.ok ? "" : tail(r.out, 40) }];
  return suiteOf("native", cases, r.ms, { passed: p.passed, failed: p.failed });
}

function runVitest() {
  const xml = join(JUNIT, "unit.xml");
  const a = ["vitest", "run", "--reporter=junit", `--outputFile=${xml}`];
  if (FILTER) a.push(FILTER);
  const r = sh("npx", a, WEB);
  const cases = existsSync(xml) ? parseJUnit(readFileSync(xml, "utf8")) : [{ name: "vitest", failed: !r.ok, skipped: !!r.missing, msg: tail(r.out, 30) }];
  return suiteOf("unit", cases, r.ms);
}

function runE2E() {
  // 不覆盖 reporter:复用 playwright.config(list+junit),仅用 env 把 junit 重定向到本目录。
  const xml = join(JUNIT, "e2e.xml");
  const a = ["playwright", "test"];
  if (FILTER) a.push(FILTER);
  if (UPDATE) a.push("--update-snapshots");
  const r = sh("npx", a, WEB, { PLAYWRIGHT_JUNIT_OUTPUT_NAME: xml });
  writeFileSync(join(RESULTS, "e2e.log"), r.out);
  const cases = existsSync(xml) ? parseJUnit(readFileSync(xml, "utf8")) : [{ name: "playwright", failed: !r.ok, skipped: !!r.missing, msg: tail(r.out, 40) }];
  return suiteOf("e2e", cases, r.ms);
}

function suiteOf(suite, cases, ms, counts) {
  const failed = counts ? counts.failed : cases.filter((c) => c.failed).length;
  const skipped = cases.filter((c) => c.skipped).length;
  const passed = counts ? counts.passed : cases.length - failed - skipped;
  return { suite, ms, passed, failed, skipped, cases, ok: failed === 0 && !cases.some((c) => c.skipped && c.failed) };
}

// ───────────────────────── run ─────────────────────────
const runs = [];
if (want("gates")) runs.push(runGates());
if (want("native")) runs.push(runNative());
if (want("unit")) runs.push(runVitest());
if (want("e2e")) runs.push(runE2E());

const totals = runs.reduce(
  (t, s) => ({ passed: t.passed + s.passed, failed: t.failed + s.failed, skipped: t.skipped + s.skipped, ms: t.ms + s.ms }),
  { passed: 0, failed: 0, skipped: 0, ms: 0 }
);
const RESULT = totals.failed === 0 && !runs.some((s) => s.cases.some((c) => c.skipped)) ? "PASS" : totals.failed > 0 ? "FAIL" : "PASS(有跳过)";
const commit = sh("git", ["rev-parse", "--short", "HEAD"], ROOT).out.trim() || "?";
const stamp = new Date().toISOString();

// ── summary.json ──
const summary = { result: RESULT, commit, stamp, totals, suites: runs };
writeFileSync(join(RESULTS, "summary.json"), JSON.stringify(summary, null, 2));

// ── TESTREPORT.md(给 agent,精简:只失败 + 计数)──
const failLines = runs
  .flatMap((s) => s.cases.filter((c) => c.failed || c.skipped).map((c) => `- [${s.suite}] ${c.name}${c.skipped ? " (跳过/命令缺失)" : ""}${c.msg ? `\n      ${c.msg.split("\n")[0]}` : ""}`))
  .join("\n");
writeFileSync(
  join(RESULTS, "TESTREPORT.md"),
  `# TESTREPORT — ${commit} @ ${stamp}\nRESULT: ${RESULT}\n\n## 各层\n${runs
    .map((s) => `${s.suite}: ${s.passed} passed / ${s.failed} failed${s.skipped ? ` / ${s.skipped} skipped` : ""}`)
    .join("\n")}\n${failLines ? `\n## 失败/跳过\n${failLines}\n` : "\n(全绿)\n"}`
);

// ── index.html(人看;蓝=过/红=败/黄=跳)──
writeFileSync(join(RESULTS, "index.html"), html(summary));

// ── 控制台 ──
const C = { PASS: "\x1b[34m", FAIL: "\x1b[31m", SKIP: "\x1b[33m", R: "\x1b[0m" };
console.log(`\n${C[RESULT.startsWith("PASS") ? "PASS" : "FAIL"]}● ${RESULT}${C.R}  ${totals.passed} passed / ${totals.failed} failed${totals.skipped ? ` / ${totals.skipped} skipped` : ""}  (${(totals.ms / 1000).toFixed(1)}s)`);
for (const s of runs) console.log(`  ${s.failed ? C.FAIL + "✗" : C.PASS + "✓"}${C.R} ${s.suite}: ${s.passed}/${s.passed + s.failed + s.skipped}`);
console.log(`→ test/results/TESTREPORT.md · index.html`);

if (OPEN) {
  const opener = process.platform === "darwin" ? "open" : process.platform === "win32" ? "start" : "xdg-open";
  sh(opener, [join(RESULTS, "index.html")], ROOT);
}

process.exit(totals.failed > 0 ? 1 : 0);

// ───────────────────────── report html ─────────────────────────
function html(s) {
  const badge = (c) => (c.failed ? "fail" : c.skipped ? "skip" : "pass");
  const rows = s.suites
    .map(
      (su) => `<section><h2>${su.suite} <small>${su.passed}✓ ${su.failed}✗ ${su.skipped ? su.skipped + "skip" : ""} · ${(su.ms / 1000).toFixed(1)}s</small></h2>
    ${su.cases
      .map((c) => `<div class="c ${badge(c)}"><span class="dot"></span><span class="n">${esc(c.name)}</span>${c.msg ? `<pre>${esc(c.msg)}</pre>` : ""}</div>`)
      .join("")}</section>`
    )
    .join("");
  return `<!doctype html><meta charset="utf8"><title>Test Report ${esc(s.commit)}</title>
<style>
:root{--pass:#3b82f6;--fail:#ef4444;--skip:#eab308;--bg:#0d0f17;--fg:#cdd3e0;--mut:#7b8499;--card:#161a24}
*{box-sizing:border-box}body{margin:0;background:var(--bg);color:var(--fg);font:14px/1.5 ui-sans-serif,system-ui,sans-serif;padding:24px}
h1{font-size:20px;margin:0 0 4px}h2{font-size:15px;margin:24px 0 8px}small{color:var(--mut);font-weight:400}
.top{display:flex;gap:16px;align-items:baseline;flex-wrap:wrap}
.result{font-weight:700;padding:2px 10px;border-radius:6px}.result.PASS{background:#1e3a8a;color:#bfdbfe}.result.FAIL{background:#7f1d1d;color:#fecaca}
.c{display:grid;grid-template-columns:14px 1fr;gap:8px;padding:5px 8px;border-radius:6px;background:var(--card);margin:3px 0;align-items:start}
.dot{width:9px;height:9px;border-radius:50%;margin-top:6px}
.pass .dot{background:var(--pass)}.fail .dot{background:var(--fail)}.skip .dot{background:var(--skip)}
.fail .n{color:#fca5a5}.skip .n{color:#fde047}.n{word-break:break-word}
pre{grid-column:2;margin:4px 0 0;padding:8px;background:#0a0c12;border-radius:4px;color:#f3a3a3;overflow:auto;font-size:12px}
</style>
<h1>infinite-chat 测试报告</h1>
<div class="top"><span class="result ${s.result.startsWith("PASS") ? "PASS" : "FAIL"}">${esc(s.result)}</span>
<small>${esc(s.commit)} · ${esc(s.stamp)} · ${s.totals.passed}✓ ${s.totals.failed}✗ ${s.totals.skipped}skip · ${(s.totals.ms / 1000).toFixed(1)}s</small></div>
${rows}
<p style="color:var(--mut);margin-top:32px">蓝=通过 · 黄=跳过/命令缺失 · 红=失败。机器可读见 summary.json / TESTREPORT.md。</p>`;
}
