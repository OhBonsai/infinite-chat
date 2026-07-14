#!/usr/bin/env node
// build-feature-report.mjs — Plan 39 R3:manifest + 捕获资产 → 单文件自包含 HTML 体检报告。
//
// 读 test/feature-manifest.json + test/results/feature-assets/<id>.{png,webm}
// + test/results/summary.json(门状态)→ test/results/feature-report.html(base64 内嵌)。
// 铁律:资产缺失 = 非 0 退出(fail loud,不出残缺报告);体积 ≤80MB(超则记录并降级 poster)。
import { existsSync, readFileSync, statSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { execFileSync } from "node:child_process";

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const ASSETS = path.join(ROOT, "test", "results", "feature-assets");
const OUT = path.join(ROOT, "test", "results", "feature-report.html");
const BUDGET = 80 * 1024 * 1024;

const manifest = JSON.parse(readFileSync(path.join(ROOT, "test", "feature-manifest.json"), "utf8"));
const summary = existsSync(path.join(ROOT, "test", "results", "summary.json"))
  ? JSON.parse(readFileSync(path.join(ROOT, "test", "results", "summary.json"), "utf8"))
  : null;
const commit = (() => {
  try {
    return execFileSync("git", ["rev-parse", "--short", "HEAD"], { cwd: ROOT }).toString().trim();
  } catch {
    return "?";
  }
})();

// —— 资产齐全性校验(fail loud)——
const missing = [];
for (const it of manifest.items) {
  const f = path.join(ASSETS, `${it.id}.${it.kind}`);
  if (!existsSync(f)) missing.push(`${it.id} (${it.kind})`);
}
if (missing.length) {
  console.error(`feature-report: 资产缺失 ${missing.length} 项 → 先跑捕获(node test/feature-report.mjs):`);
  for (const m of missing) console.error(`  - ${m}`);
  process.exit(1);
}

const esc = (s) =>
  String(s).replace(/[&<>"]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" })[c]);
const dataUri = (id, kind) => {
  const b = readFileSync(path.join(ASSETS, `${id}.${kind}`));
  const mime = kind === "png" ? "image/png" : "video/webm";
  return `data:${mime};base64,${b.toString("base64")}`;
};

// —— 体积预算:超 80MB 则按 webm 大小降序丢到 poster 帧(webm 最占体积)——
let total = manifest.items.reduce(
  (n, it) => n + statSync(path.join(ASSETS, `${it.id}.${it.kind}`)).size,
  0,
);
const demoted = [];
if (total > BUDGET) {
  const webms = manifest.items
    .filter((it) => it.kind === "webm")
    .map((it) => ({ it, size: statSync(path.join(ASSETS, `${it.id}.webm`)).size }))
    .sort((a, b) => b.size - a.size);
  for (const { it, size } of webms) {
    if (total <= BUDGET) break;
    demoted.push(it.id);
    total -= size; // poster:webm 无独立首帧文件 → 降为纯文案条(记录)
  }
}

const groupSections = manifest.groups
  .map((g) => {
    const items = manifest.items.filter((i) => i.group === g.id);
    const cards = items
      .map((it) => {
        const media = demoted.includes(it.id)
          ? `<div class="poster">▶ 动效段(超体积预算,已降为占位;重跑 node test/feature-report.mjs 得完整视频)</div>`
          : it.kind === "png"
            ? `<img loading="lazy" src="${dataUri(it.id, "png")}" alt="${esc(it.title)}">`
            : `<video controls preload="none" loading="lazy"><source src="${dataUri(it.id, "webm")}" type="video/webm"></video>`;
        return `<article class="card" id="${esc(it.id)}">
  <h3>${esc(it.title)} <span class="kind ${it.kind}">${it.kind}</span></h3>
  <p class="desc">${esc(it.desc)}</p>
  <div class="media">${media}</div>
</article>`;
      })
      .join("\n");
    return `<section class="group" id="grp-${esc(g.id)}">
  <h2>${esc(g.title)}</h2>
  <p class="gdesc">${esc(g.desc)}</p>
  ${cards}
</section>`;
  })
  .join("\n");

const nav = manifest.groups
  .map((g) => {
    const sub = manifest.items
      .filter((i) => i.group === g.id)
      .map((it) => `<a href="#${esc(it.id)}">${esc(it.title)}</a>`)
      .join("");
    return `<div class="navgrp"><a class="navtop" href="#grp-${esc(g.id)}">${esc(g.title)}</a>${sub}</div>`;
  })
  .join("\n");

const gate = summary
  ? `<span class="${summary.result === "PASS" ? "ok" : "bad"}">${summary.result}</span> · ${summary.totals.passed} passed / ${summary.totals.failed} failed`
  : "(无 summary.json)";

const html = `<!doctype html>
<html lang="zh"><head><meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>infinite-chat · 功能体检报告</title>
<style>
:root{--bg:#131316;--card:#1d1d20;--fg:#ededed;--dim:#a0a0a0;--acc:#8cb0ff;--line:#2a2a30}
*{box-sizing:border-box}
body{margin:0;background:var(--bg);color:var(--fg);font:15px/1.6 system-ui,"PingFang SC",sans-serif;display:flex}
code,.kind{font-family:ui-monospace,"SF Mono",monospace}
nav{position:sticky;top:0;align-self:flex-start;width:280px;height:100vh;overflow:auto;padding:16px;border-right:1px solid var(--line);font-size:13px}
nav h1{font-size:15px;margin:0 0 4px}
nav .meta{color:var(--dim);font-size:11px;margin-bottom:16px}
.navgrp{margin-bottom:10px}
.navtop{display:block;color:var(--fg);font-weight:600;text-decoration:none;padding:3px 0}
.navgrp a:not(.navtop){display:block;color:var(--dim);text-decoration:none;padding:1px 0 1px 12px;font-size:12px;white-space:nowrap;overflow:hidden;text-overflow:ellipsis}
.navgrp a:hover{color:var(--acc)}
main{flex:1;max-width:960px;padding:24px 32px;margin:0 auto}
.hero{background:var(--card);border:1px solid var(--line);border-radius:14px;padding:20px 24px;margin-bottom:28px}
.hero h1{margin:0 0 8px;font-size:22px}
.hero .stats{color:var(--dim);font-size:13px}
.ok{color:#7ee787}.bad{color:#ff7b72}
.group{margin-bottom:40px;scroll-margin-top:16px}
.group>h2{font-size:19px;border-bottom:2px solid var(--line);padding-bottom:8px}
.gdesc{color:var(--dim);font-size:13px;margin-top:-4px}
.card{background:var(--card);border:1px solid var(--line);border-radius:14px;padding:16px 18px;margin:16px 0;scroll-margin-top:16px}
.card h3{margin:0 0 6px;font-size:16px;display:flex;align-items:center;gap:8px}
.kind{font-size:10px;padding:2px 6px;border-radius:6px;background:#2a2a30;color:var(--dim)}
.kind.webm{background:#243b5a;color:#8cb0ff}
.desc{color:var(--dim);font-size:13px;margin:0 0 12px}
.media img,.media video{max-width:100%;height:auto;border-radius:10px;border:1px solid var(--line);display:block;background:#000}
.poster{color:var(--dim);font-size:13px;padding:24px;border:1px dashed var(--line);border-radius:10px;text-align:center}
@media(max-width:820px){body{flex-direction:column}nav{position:static;width:auto;height:auto;border-right:none;border-bottom:1px solid var(--line)}}
</style></head><body>
<nav>
  <h1>功能体检</h1>
  <div class="meta">commit ${esc(commit)} · ${new Date().toISOString().slice(0, 16).replace("T", " ")}</div>
  ${nav}
</nav>
<main>
  <div class="hero">
    <h1>infinite-chat 功能体检报告</h1>
    <div class="stats">
      ${manifest.groups.length} 大项 / ${manifest.items.length} 小项(${manifest.items.filter((i) => i.kind === "png").length} 图 · ${manifest.items.filter((i) => i.kind === "webm").length} 视频)·
      资产 ${(total / 1024 / 1024).toFixed(1)}MB · 五层门 ${gate}
      ${demoted.length ? ` · ⚠ ${demoted.length} 段视频超预算已降级` : ""}
    </div>
    <p class="desc" style="margin-top:10px">plan28–38 这一大轮的全部人可感知功能。每项都由 e2e 逐项捕获并带可见性断言(报告即测试)。单文件自包含,双击即看;推荐 Chrome 打开(webm/VP8)。</p>
  </div>
  ${groupSections}
</main>
</body></html>`;

writeFileSync(OUT, html);
const finalSize = statSync(OUT).size;
console.log(
  `→ ${path.relative(ROOT, OUT)} (${(finalSize / 1024 / 1024).toFixed(1)}MB, ${manifest.items.length} 项${demoted.length ? `, ${demoted.length} 段降级` : ""})`,
);
if (finalSize > BUDGET) {
  console.error(`feature-report: 单文件 ${(finalSize / 1024 / 1024).toFixed(1)}MB 仍超 80MB 预算`);
  process.exit(1);
}
