#!/usr/bin/env node
// 复制 KaTeX woff2 字体(数学,Plan 12)从本地 RaTeX 工作区 → web/public/fonts/katex/。
//
// 数学字形栅化需 KaTeX 字体(web/src/math-fonts.ts 懒加载)。字体源 = RaTeX/platforms/web/fonts/
// (RaTeX 是本地 gitignore 的工作区)。产物 web/public/fonts/katex/ 同样 gitignore(可重生,守仓库
// 体积,与 lxgw-msdf 同策)。math-fonts.ts 只需 16 个 Regular/Bold/Italic 变体里**用到的**那些。
//
// 用法:node scripts/copy-katex-fonts.mjs   (需先有 ./RaTeX 工作区)

import { mkdirSync, copyFileSync, existsSync, readdirSync } from "node:fs";
import { join } from "node:path";

const SRC = "RaTeX/platforms/web/fonts";
const DST = "web/public/fonts/katex";

// math-fonts.ts FACES 用到的 woff2(基名)。
const NEEDED = [
  "KaTeX_Main-Regular", "KaTeX_Main-Bold", "KaTeX_Main-Italic", "KaTeX_Main-BoldItalic",
  "KaTeX_Math-Italic", "KaTeX_Math-BoldItalic", "KaTeX_AMS-Regular",
  "KaTeX_Size1-Regular", "KaTeX_Size2-Regular", "KaTeX_Size3-Regular", "KaTeX_Size4-Regular",
  "KaTeX_Caligraphic-Regular", "KaTeX_Fraktur-Regular", "KaTeX_SansSerif-Regular",
  "KaTeX_Script-Regular", "KaTeX_Typewriter-Regular",
];

if (!existsSync(SRC)) {
  console.error(`✗ 找不到 ${SRC} —— 需本地 RaTeX 工作区(git clone RaTeX 到项目根,见 plan12)。`);
  process.exit(1);
}
mkdirSync(DST, { recursive: true });
const avail = new Set(readdirSync(SRC));
let n = 0;
for (const base of NEEDED) {
  const f = `${base}.woff2`;
  if (avail.has(f)) {
    copyFileSync(join(SRC, f), join(DST, f));
    n++;
  } else {
    console.warn(`⚠ 缺 ${f}`);
  }
}
console.log(`✓ 复制 ${n}/${NEEDED.length} 个 KaTeX woff2 → ${DST}`);
