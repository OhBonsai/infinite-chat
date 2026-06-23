#!/usr/bin/env node
// gen-reel —— 生成「能力 reel」会话 web/public/cases/reel.json。
// 不是 markdown 文档,而是表现力内容:大标题 + cat 分割线(---)+ 数学 + emoji,
// 由 main.ts 的 reel 导演反复以不同 reveal 节奏重放 + zoom + shaderbox 特效。
// 用法:node scripts/gen-reel.mjs
import { writeFileSync, mkdirSync } from "node:fs";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "..");

// LaTeX 反斜杠在 JS 模板串里写 \\。--- 会被引擎渲成「猫」分割线(rule_cat)。
const MD = `# 任何文字 · 都是 SDF

# 无限缩放 · 永远锐利

---

## 不同节奏 · 流式涌现

快 — 慢 — 顿 — 涌 — 字随节奏长出来

---

## 数学是一等公民

$$e^{i\\pi} + 1 = 0$$

$$\\sum_{i=1}^{n} i = \\frac{n(n+1)}{2}$$

---

## 中英混排 · emoji 也稳

rendering engine · 渲染引擎 🚀🌍✨

---

# 这不是 DOM

# 这是一块 GPU 画布`;

const cps = Array.from(MD);
const steps = [];
let t = 0;
let i = 0;
while (i < cps.length) {
  const n = 2 + Math.floor(Math.random() * 4); // 2..5
  const delta = cps.slice(i, i + n).join("");
  i += n;
  steps.push({ t: Math.round(t), delta });
  t += 26 + Math.random() * 22;
  if (delta.includes("\n\n") || /\n#{1,6}\s/.test(delta)) t += 180;
}

const out = {
  _doc: "能力 reel(非 markdown 文档)。脚本生成:scripts/gen-reel.mjs。main.ts reel 导演反复重放 + 变节奏 + zoom + shaderbox。",
  sessionID: "reel",
  messageID: "m1",
  partID: "p1",
  steps,
};

const dir = resolve(ROOT, "web/public/cases");
mkdirSync(dir, { recursive: true });
const path = resolve(dir, "reel.json");
writeFileSync(path, JSON.stringify(out, null, 2));
console.log(`wrote ${path}: ${steps.length} steps, ~${Math.round(t / 1000)}s`);
