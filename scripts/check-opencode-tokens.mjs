#!/usr/bin/env node
// Plan 28 遗留-7(§5 DoD-2):参考 token ↔ 我方实现 数值自动对照。
// 对照面:theme.rs 默认表 / glyph.wgsl 文字色 / motion.rs 间距 / boxlayout 常量 / layout-bridge 字号。
// 判定:色 ΔE*ab(1976)≤5;alpha ±0.05;标量按表内容差(px 级一致)。
// 产物:spec/reference/opencode-ui/CHECK.md + 退出码(有 FAIL → 1)。
import { readFileSync, writeFileSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..");
const read = (p) => readFileSync(join(ROOT, p), "utf8");
const live = JSON.parse(read("spec/reference/opencode-ui/tokens.live-dark.json")).vars;

// ── 颜色工具:css 色 → [r,g,b,a](0..1);sRGB→Lab→ΔE76 ──
function cssColor(v) {
  if (!v) return null;
  v = v.trim();
  let m = v.match(/^#([0-9a-f]{6})$/i);
  if (m) {
    const h = m[1];
    return [0, 2, 4].map((i) => parseInt(h.slice(i, i + 2), 16) / 255).concat([1]);
  }
  m = v.match(/^rgba?\(([^)]+)\)$/);
  if (m) {
    const p = m[1].split(",").map((x) => parseFloat(x));
    return [p[0] / 255, p[1] / 255, p[2] / 255, p.length > 3 ? p[3] : 1];
  }
  return null;
}
function lab([r, g, b]) {
  const f = (u) => (u > 0.04045 ? ((u + 0.055) / 1.055) ** 2.4 : u / 12.92);
  const [R, G, B] = [f(r), f(g), f(b)];
  const X = (R * 0.4124 + G * 0.3576 + B * 0.1805) / 0.95047;
  const Y = R * 0.2126 + G * 0.7152 + B * 0.0722;
  const Z = (R * 0.0193 + G * 0.1192 + B * 0.9505) / 1.08883;
  const t = (u) => (u > 0.008856 ? Math.cbrt(u) : 7.787 * u + 16 / 116);
  return [116 * t(Y) - 16, 500 * (t(X) - t(Y)), 200 * (t(Y) - t(Z))];
}
const dE = (a, b) => {
  const [L1, A1, B1] = lab(a);
  const [L2, A2, B2] = lab(b);
  return Math.hypot(L1 - L2, A1 - A2, B1 - B2);
};

// ── 我方值提取 ──
const themeSrc = read("crates/core/src/theme.rs");
function themeVal(field) {
  const m = themeSrc.match(new RegExp(`${field}: \\[\\s*([0-9][0-9.,\\s]*)\\]`));
  return m ? m[1].split(",").map((x) => parseFloat(x)) : null;
}
const wgsl = read("crates/render/src/shaders/base/glyph.wgsl");
function wgslCase(id) {
  const m = wgsl.match(new RegExp(`case ${id}u[^{]*\\{ return vec3<f32>\\(([^)]+)\\)`));
  return m ? m[1].split(",").map((x) => parseFloat(x)).concat([1]) : null;
}
function wgslDefault() {
  const m = wgsl.match(/default: \{ return vec3<f32>\(([^)]+)\)/);
  return m ? m[1].split(",").map((x) => parseFloat(x)).concat([1]) : null;
}
const motionSrc = read("crates/core/src/motion.rs");
const boxSrc = read("crates/core/src/boxlayout.rs");
const bridgeSrc = read("web/src/layout-bridge.ts");
const num = (src, re) => {
  const m = src.match(re);
  return m ? parseFloat(m[1]) : null;
};

// ── 对照表 ──
const rows = [];
function color(name, ours, refName, opts = {}) {
  const ref = refName === "transparent" ? [0, 0, 0, 0] : cssColor(live[refName]);
  if (!ours || !ref) {
    rows.push([name, "?", refName, "-", "MISS"]);
    return;
  }
  const alphaOk = Math.abs((ours[3] ?? 1) - ref[3]) <= 0.051 || opts.alphaFree;
  let pass;
  let detail;
  if (ref[3] === 0) {
    pass = (ours[3] ?? 1) === 0;
    detail = `alpha ${ours[3]}`;
  } else {
    const d = dE(ours, ref);
    pass = d <= 5 && alphaOk;
    detail = `ΔE=${d.toFixed(1)}${alphaOk ? "" : " alpha✗"}`;
  }
  rows.push([name, fmt(ours), `${refName}=${live[refName] ?? refName}`, detail, pass ? "PASS" : "FAIL"]);
}
function scalar(name, ours, ref, tolerance = 0.01) {
  const pass = ours != null && Math.abs(ours - ref) <= tolerance;
  rows.push([name, String(ours), String(ref), "", pass ? "PASS" : "FAIL"]);
}
const fmt = (c) => `[${c.map((x) => +x.toFixed(3)).join(",")}]`;

// theme.rs ↔ live tokens(映射依据 NOTES.md)
color("theme.code_bg", themeVal("code_bg"), "--surface-inset-base");
color("theme.code_chip(疑点)", themeVal("code_chip"), "transparent");
color("theme.quote_bar", themeVal("quote_bar"), "--border-weak-base");
color("theme.hr_rule", themeVal("hr_rule"), "transparent");
color("theme.table_header_bg", themeVal("table_header_bg"), "transparent");
color("theme.table_rule", themeVal("table_rule"), "--border-weak-base");
color("theme.user_bg", themeVal("user_bg"), "--surface-base");
color("theme.card_border", themeVal("card_border"), "--border-weak-base");
color("theme.diff_add_bg", themeVal("diff_add_bg"), "--surface-diff-add-base");
color("theme.diff_del_bg", themeVal("diff_del_bg"), "--surface-diff-delete-base");
color("theme.ask_button_bg", themeVal("ask_button_bg"), "--button-primary-base");
color("theme.selection(rgb)", themeVal("selection"), "--surface-interactive-base", { alphaFree: true });

// wgsl 文字色 ↔ text tokens
color("wgsl.Normal(default)", wgslDefault(), "--text-strong");
color("wgsl.Quote/ListMarker(8,9)", wgslCase(8), "--text-weak");
color("wgsl.Link(7)", wgslCase(7), "--text-interactive-base");
color("wgsl.InlineCode(4)", wgslCase(4), "--syntax-string");
color("wgsl.ToolArg(53)", wgslCase(53), "--text-weaker");
color("wgsl.Reasoning(51)", wgslCase(51), "--text-weak");
color("wgsl.ToolTitle(52)", wgslCase(52), "--text-strong");
color("wgsl.DiffAdded(56)", wgslCase(56), "--text-diff-add-base");
color("wgsl.DiffRemoved(57)", wgslCase(57), "--text-diff-delete-base");
color("wgsl.DiffCtx(59)", wgslCase(59), "--text-base");
color("wgsl.CodeKeyword(44)", wgslCase(44), "--syntax-keyword");
color("wgsl.CodeType(45)", wgslCase(45), "--syntax-type");
color("wgsl.CodeString(47)", wgslCase(47), "--syntax-string");
color("wgsl.CodeComment(48)", wgslCase(48), "--syntax-comment");

// 间距/布局/字号(标量;参考 = session-turn.css / message-timeline.tsx / theme.css)
scalar("motion.space_turn", num(motionSrc, /space_turn: ([\d.]+)/), 24);
scalar("motion.space_part", num(motionSrc, /space_part: ([\d.]+)/), 12);
scalar("boxlayout.CONTENT_MAX", num(boxSrc, /CONTENT_MAX: f32 = ([\d.]+)/), 800);
scalar("boxlayout.BUBBLE_RATIO", num(boxSrc, /BUBBLE_RATIO: f32 = ([\d.]+)/), 0.82);
scalar("bridge.BASE_FONT_CSS_PX", num(bridgeSrc, /BASE_FONT_CSS_PX = ([\d.]+)/), 14);
scalar("bridge.LINE_HEIGHT 倍率", num(bridgeSrc, /FONT_SIZE \* ([\d.]+)\)/), 1.6);

// ── 输出 ──
const fails = rows.filter((r) => r[4] !== "PASS");
const md = [
  "# CHECK — 参考 token ↔ 实现 数值对照(Plan 28 §5 / DoD-2)",
  "",
  `- 生成:scripts/check-opencode-tokens.mjs;判定:ΔE*ab ≤5、alpha ±0.05、标量 px 级。`,
  `- 结果:**${rows.length - fails.length}/${rows.length} PASS**${fails.length ? ` — ${fails.length} FAIL` : ""}`,
  "",
  "| 项 | 实现值 | 参考 | 偏差 | 判定 |",
  "|---|---|---|---|---|",
  ...rows.map((r) => `| ${r.join(" | ")} |`),
  "",
].join("\n");
writeFileSync(join(ROOT, "spec/reference/opencode-ui/CHECK.md"), md);
console.log(md);
process.exit(fails.length ? 1 : 0);
