// Plan 31 R1(DoD-2/0035 §4):RaTeX 排版 ↔ KaTeX 参考 数值对照。
// 取三个可跨系统直接对齐的度量(em 归一):
//   ① x^2 上标抬升(base 基线 − sup 基线)     ② \frac{a}{b} 分子基线→杠 / 杠→分母基线
//   ③ x_1 下标下沉
// RaTeX 侧:cargo dump(math.rs dump_math_layouts);KaTeX 侧:**其度量常数**(dist katex.mjs
// fontMetrics sigmas —— DOM span 盒含行高/strut,不能当基线用,首版实测偏差全来自量测法)。
// 参考式(TeX Appendix G):supRaise=sup2;fracNumGap=num1−axis−rule/2;
// fracDenGap=denom1+axis+rule/2;subDrop=sub1。容差 ≤3%。
// 产物:spec/reference/math-katex-CHECK.md + restore-diff/math-katex-side.png。
import { chromium } from "@playwright/test";
import { execSync } from "node:child_process";
import { readFileSync, writeFileSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "../..");

// ── RaTeX 侧 ──
const out = execSync(
  "cargo test -p infinite-chat-core --release dump_math_layouts -- --ignored --nocapture 2>&1",
  { cwd: ROOT, encoding: "utf8" },
);
const line = out.split("\n").find((l) => l.startsWith("MATHDUMP "));
if (!line) throw new Error("MATHDUMP 未产出:\n" + out.slice(-800));
const [supD, fracD, subD] = JSON.parse(line.slice("MATHDUMP ".length));
const g = (d, ch) => d.glyphs.find((x) => x.ch === ch);
const ratex = {
  supRaise: g(supD, "x").dy - g(supD, "2").dy,
  fracNumGap: fracD.rules[0].y - g(fracD, "a").dy, // 杠 y − 分子基线(y 向下,杠在下 → 正)
  fracDenGap: g(fracD, "b").dy - fracD.rules[0].y,
  subDrop: g(subD, "1").dy - g(subD, "x").dy,
};

// ── KaTeX 侧:度量常数(参考值)+ 并排参考图(DOM 真渲染,仅供肉眼复核)──
const dist = readFileSync(join(ROOT, "web/node_modules/katex/dist/katex.mjs"), "utf8");
const sigma = (name) => {
  const m = dist.match(new RegExp(`${name}: \\[([0-9.]+)`));
  if (!m) throw new Error(`sigma ${name} 未找到`);
  return parseFloat(m[1]);
};
const [sup2, sub1, num1, denom1, axis, ruleTh] = [
  sigma("sup2"),
  sigma("sub1"),
  sigma("num1"),
  sigma("denom1"),
  sigma("axisHeight"),
  sigma("defaultRuleThickness"),
];
const katex = {
  supRaise: sup2,
  fracNumGap: num1 - axis - ruleTh / 2,
  fracDenGap: denom1 + axis + ruleTh / 2,
  subDrop: sub1,
};
{
  const b = await chromium.launch();
  const p = await b.newPage();
  await p.setContent(
    `<!DOCTYPE html><html><body><div id="m" style="font-size:100px;padding:60px"></div></body></html>`,
  );
  await p.addStyleTag({ path: join(ROOT, "web/node_modules/katex/dist/katex.min.css") });
  await p.addScriptTag({ path: join(ROOT, "web/node_modules/katex/dist/katex.min.js") });
  await p.evaluate(() => {
    const host = document.getElementById("m");
    const wrap = document.createElement("div");
    wrap.style.cssText = "display:flex;gap:60px;align-items:center";
    for (const [t, d] of [["x^2", false], ["\\frac{a}{b}", true], ["x_1", false]]) {
      const cell = document.createElement("div");
      // eslint-disable-next-line no-undef
      window.katex.render(t, cell, { displayMode: d });
      wrap.appendChild(cell);
    }
    host.appendChild(wrap);
  });
  await p.screenshot({ path: join(ROOT, "test/results/restore-diff/math-katex-ref.png") });
  await b.close();
}

// ── 对照 ──
const rows = [];
const cmp = (name, ours, ref) => {
  const tol = Math.max(0.005, Math.abs(ref) * 0.03);
  const pass = Math.abs(ours - ref) <= tol;
  rows.push([name, ours.toFixed(4), ref.toFixed(4), (ours - ref).toFixed(4), pass ? "PASS" : "FAIL"]);
};
cmp("sup 抬升(x^2)", ratex.supRaise, katex.supRaise);
cmp("frac 分子→杠", ratex.fracNumGap, katex.fracNumGap);
cmp("frac 杠→分母", ratex.fracDenGap, katex.fracDenGap);
cmp("sub 下沉(x_1)", ratex.subDrop, katex.subDrop);
const fails = rows.filter((r) => r[4] !== "PASS").length;
const md = [
  "# math ↔ KaTeX 参考对照(Plan 31 R1 / 0035 §4)",
  "",
  `- 生成:web/scripts/check-math-katex.mjs;容差 max(0.04em, 3%)。`,
  `- 结果:**${rows.length - fails}/${rows.length} PASS**`,
  "",
  "| 度量(em) | RaTeX | KaTeX | Δ | 判定 |",
  "|---|---|---|---|---|",
  ...rows.map((r) => `| ${r.join(" | ")} |`),
  "",
].join("\n");
writeFileSync(join(ROOT, "spec/reference/math-katex-CHECK.md"), md);
console.log(md);
process.exit(fails ? 1 : 0);
