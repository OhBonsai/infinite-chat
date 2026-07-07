#!/usr/bin/env node
// Plan 28 R0:静态解析 opencode packages/ui/src/styles/theme.css 的 :root 变量表 →
// spec/reference/opencode-ui/tokens.global.json({ light: {...}, dark: {...} })。
// dark 块 = :root 内嵌 `@media (prefers-color-scheme: dark)`(CSS 嵌套);dark 缺省的键回落 light。
// 只做全局变量;组件级 computedStyle 由 live dump(dump-opencode-computed.mjs)补。
import { readFileSync, writeFileSync, mkdirSync } from "node:fs";
import { homedir } from "node:os";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const SRC = join(homedir(), "w/agentscode/opencode/packages/ui/src/styles/theme.css");
const OUT = join(
  dirname(fileURLToPath(import.meta.url)),
  "../spec/reference/opencode-ui/tokens.global.json",
);

const css = readFileSync(SRC, "utf8");

// 按行状态机:进入 dark media 块后计大括号深度,出块回 light。
const light = {};
const dark = {};
let inDark = false;
let depth = 0;
for (const line of css.split("\n")) {
  if (!inDark && /@media\s*\(prefers-color-scheme:\s*dark\)/.test(line)) {
    inDark = true;
    depth = 0;
  }
  if (inDark) {
    depth += (line.match(/{/g) ?? []).length - (line.match(/}/g) ?? []).length;
    if (depth <= 0 && line.includes("}")) inDark = false;
  }
  const m = line.match(/^\s*(--[\w-]+)\s*:\s*(.+?);\s*$/);
  if (m) (inDark ? dark : light)[m[1]] = m[2].trim();
}

const out = { source: "packages/ui/src/styles/theme.css", light, dark: { ...light, ...dark } };
mkdirSync(dirname(OUT), { recursive: true });
writeFileSync(OUT, JSON.stringify(out, null, 2));
console.log(
  `light ${Object.keys(light).length} vars, dark-overrides ${Object.keys(dark).length} → ${OUT}`,
);
