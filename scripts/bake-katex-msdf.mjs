#!/usr/bin/env node
// bake-katex-msdf(Plan 12 ④)— 把 **RaTeX 用的 KaTeX 字体**(Computer Modern 系列)编译成 MSDF,
// 合并成**单页 atlas**供运行时数学 SDF 锐利缩放(MSDF,任意缩放无锯齿)。
//
// 两步:① 逐 KaTeX 字族(`scripts/katex/charset/*.txt`,RaTeX 语料收集)跑 msdf-bmfont 出 per-font
// MSDF;② pngjs 把各族字形**拼进一张 2048² 页**,字形 id 改用**合成键 `role*0x110000 + codepoint`**
// (role = StyleRole 26–40)→ 与正文 codepoint key 空间不撞,可与 lxgw 共存。产物
// `web/public/fonts/katex-msdf.{json, .0.png}`(BMFont 格式,gitignore,可重生)。
//
// 用法:
//   1) 收集字符集(改语料后):cargo test -p infinite-chat-core --test dump_katex_charset -- --ignored
//   2) 编译:node scripts/bake-katex-msdf.mjs   (需 RaTeX/ 工作区 + 全局 msdf-bmfont + pngjs)
//
// 运行时(web `msdf.ts loadMathMsdf` → wasm `load_msdf`):atlas 进 MSDF D2Array;`resolve` 对数学
// 角色按合成键查 MSDF(命中)→ 未命中回退 TinySDF;`msdf_node` 用本 atlas 的 baked size。

import { execFileSync } from "node:child_process";
import { mkdirSync, existsSync, readFileSync, writeFileSync, rmSync, mkdtempSync } from "node:fs";
import { resolve, dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { tmpdir } from "node:os";
import { createRequire } from "node:module";

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "..");
// pngjs 装在 web/node_modules(本仓唯一的 node 工程);从 web/ 解析。
const { PNG } = createRequire(resolve(ROOT, "web/package.json"))("pngjs");
const FONTS = resolve(ROOT, "RaTeX/fonts");
const CHARSET = resolve(ROOT, "scripts/katex/charset");
const OUT = resolve(ROOT, "web/public/fonts");
const SIZE = 42; // 每字形 MSDF 像素
const RANGE = 4; // 距离场像素范围
const ATLAS = 2048; // 合并页尺寸(与 lxgw-msdf 一致 → 可同一 D2Array 共存)
const PAD = 2; // 拼接字形间距
const CLI = "msdf-bmfont";

// KaTeX 字族基名 → StyleRole 数值(须与 crate::content::StyleRole / math::katex_font_base 一致)。
const ROLE = {
  "Main-Regular": 26, "Main-Bold": 27, "Main-Italic": 28, "Main-BoldItalic": 29,
  "Math-Italic": 30, "AMS-Regular": 31,
  "Size1-Regular": 32, "Size2-Regular": 33, "Size3-Regular": 34, "Size4-Regular": 35,
  "Caligraphic-Regular": 36, "Fraktur-Regular": 37, "SansSerif-Regular": 38,
  "Script-Regular": 39, "Typewriter-Regular": 40,
};
const SYN_BASE = 0x110000; // 合成键基:role*SYN_BASE + codepoint(codepoint < 0x110000)

if (!existsSync(FONTS)) fail(`找不到 ${FONTS} —— 需本地 RaTeX/ 工作区(见 plan12)。`);
const manifestPath = resolve(CHARSET, "manifest.txt");
if (!existsSync(manifestPath))
  fail(`找不到 ${manifestPath} —— 先跑:cargo test --test dump_katex_charset -- --ignored`);

const bases = readFileSync(manifestPath, "utf8").split("\n").map((s) => s.trim()).filter(Boolean);
const tmp = mkdtempSync(join(tmpdir(), "katexmsdf-"));
mkdirSync(OUT, { recursive: true });

// 合并页(RGBA,透明底)+ 行式打包游标。
const page = new PNG({ width: ATLAS, height: ATLAS });
page.data.fill(0);
let cx = PAD, cy = PAD, rowH = 0;
const chars = [];

for (const base of bases) {
  if (!(base in ROLE)) { console.warn(`⚠ 未知字族(无 role 映射):${base}`); continue; }
  const ttf = resolve(FONTS, `KaTeX_${base}.ttf`);
  const charset = resolve(CHARSET, `${base}.txt`);
  if (!existsSync(ttf) || !existsSync(charset)) { console.warn(`⚠ 缺 ${base} 字体/字符集`); continue; }
  const stem = join(tmp, base);
  try {
    execFileSync(CLI, [ttf, "-i", charset, "-m", "1024,1024", "-s", String(SIZE),
      "-r", String(RANGE), "-t", "msdf", "-f", "json", "-o", stem], { stdio: "pipe" });
  } catch (e) { console.warn(`⚠ 烘 ${base} 失败:${(e.message || "").split("\n")[0]}`); continue; }

  const meta = JSON.parse(readFileSync(join(tmp, `KaTeX_${base}.json`), "utf8"));
  const src = PNG.sync.read(readFileSync(`${stem}.png`));
  // 整张子图先放到合并页的一个块(块 = 该字族 atlas),记录块偏移 → 各字形坐标 += 偏移。
  if (cx + src.width + PAD > ATLAS) { cx = PAD; cy += rowH + PAD; rowH = 0; }
  if (cy + src.height + PAD > ATLAS) fail(`合并页 ${ATLAS}² 放不下(字形过多)——增大 ATLAS。`);
  PNG.bitblt(src, page, 0, 0, src.width, src.height, cx, cy);
  const ox = cx, oy = cy;
  cx += src.width + PAD;
  rowH = Math.max(rowH, src.height);
  const role = ROLE[base];
  for (const c of meta.chars) {
    chars.push({
      id: role * SYN_BASE + c.id, // 合成键
      x: c.x + ox, y: c.y + oy, width: c.width, height: c.height,
      xoffset: c.xoffset, yoffset: c.yoffset, xadvance: c.xadvance, page: 0,
    });
  }
  console.log(`✓ ${base}(role ${role}): ${meta.chars.length} 字形 @ (${ox},${oy})`);
}

// 写合并 atlas(BMFont 格式,单页)+ png。
const out = {
  info: { face: "KaTeX-MSDF", size: SIZE },
  common: { lineHeight: SIZE, scaleW: ATLAS, scaleH: ATLAS, pages: 1 },
  distanceField: { fieldType: "msdf", distanceRange: RANGE },
  pages: ["katex-msdf.0.png"],
  chars,
};
writeFileSync(join(OUT, "katex-msdf.json"), JSON.stringify(out));
writeFileSync(join(OUT, "katex-msdf.0.png"), PNG.sync.write(page));
rmSync(tmp, { recursive: true, force: true });
console.log(`\n✓ 合并 ${chars.length} 数学字形 → ${join(OUT, "katex-msdf.{json,.0.png}")}(${ATLAS}² 单页,合成键)`);

function fail(m) { console.error(`✗ ${m}`); process.exit(1); }
