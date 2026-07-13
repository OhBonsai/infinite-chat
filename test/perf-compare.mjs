// perf-compare.mjs — 性能回归门比较逻辑(Plan 35 T2,纯函数;调研 §3)。
//
// 确定性优先(T10):结构指标(retainedGlyphs/wasmMiB)取**末行**(会话载满稳态)、
// 阈值 +10%;耗时指标(frameMsAvg/phBfTotal/phAdvance)取**中位数**、宽阈值 +25%。
// fps **只记录不设门**(headless fps 不可信,调研 §3.3)。

/** 解析 bench CSV → { header: string[], rows: number[][] }(空/畸形 → rows=[])。 */
export function parseCsv(text) {
  const lines = String(text).trim().split(/\r?\n/);
  if (lines.length < 2) return { header: [], rows: [] };
  const header = lines[0].split(",").map((h) => h.trim());
  const rows = lines
    .slice(1)
    .map((l) => l.split(",").map(Number))
    .filter((r) => r.length === header.length && r.every((x) => Number.isFinite(x)));
  return { header, rows };
}

function col(parsed, name) {
  const i = parsed.header.indexOf(name);
  if (i < 0) return [];
  return parsed.rows.map((r) => r[i]);
}

function median(xs) {
  if (!xs.length) return NaN;
  const s = [...xs].sort((a, b) => a - b);
  const m = s.length >> 1;
  return s.length % 2 ? s[m] : (s[m - 1] + s[m]) / 2;
}

function last(xs) {
  return xs.length ? xs[xs.length - 1] : NaN;
}

/** 门指标表:name → { pick(样本列→标量), threshold(允许恶化比例), gated } */
export const METRICS = [
  { name: "retainedGlyphs", pick: last, threshold: 0.1, gated: true },
  { name: "wasmMiB", pick: last, threshold: 0.1, gated: true },
  { name: "frameMsAvg", pick: median, threshold: 0.25, gated: true },
  { name: "phBfTotal", pick: median, threshold: 0.25, gated: true },
  { name: "phAdvance", pick: median, threshold: 0.25, gated: true },
  { name: "fps", pick: median, threshold: Infinity, gated: false }, // 只记录(§3.3)
];

/**
 * 对比 current vs baseline → { pass, results: [{name, base, cur, delta, limit, gated, ok}] }。
 * 缺列/NaN → 该指标跳过(记 ok=null,不判负 —— 采集面变化不该假红);全部 gated 指标
 * ok!==false 才 pass。
 */
export function compare(currentCsv, baselineCsv, metrics = METRICS) {
  const cur = parseCsv(currentCsv);
  const base = parseCsv(baselineCsv);
  const results = metrics.map(({ name, pick, threshold, gated }) => {
    const b = pick(col(base, name));
    const c = pick(col(cur, name));
    if (!Number.isFinite(b) || !Number.isFinite(c)) {
      return { name, base: b, cur: c, delta: NaN, limit: threshold, gated, ok: null };
    }
    // 恶化比例:指标全部「越小越好」;基线为 0 时用绝对余量(0 → 允许到阈值同名绝对小量)。
    const delta = b > 0 ? (c - b) / b : c > 0 ? Infinity : 0;
    const ok = !gated || delta <= threshold;
    return { name, base: b, cur: c, delta, limit: threshold, gated, ok };
  });
  const pass = results.every((r) => r.ok !== false);
  return { pass, results };
}

/** 结果表 → 控制台/报告一段文本。 */
export function renderTable(outcome) {
  const lines = outcome.results.map((r) => {
    const d = Number.isFinite(r.delta) ? `${(r.delta * 100).toFixed(1)}%` : "n/a";
    const mark = r.ok === false ? "✗" : r.ok === null ? "–" : "✓";
    const gate = r.gated ? `≤+${(r.limit * 100).toFixed(0)}%` : "(记录)";
    return `${mark} ${r.name}: ${r.base} → ${r.cur}  Δ${d} ${gate}`;
  });
  return `${outcome.pass ? "PERF PASS" : "PERF FAIL"}\n${lines.join("\n")}`;
}
