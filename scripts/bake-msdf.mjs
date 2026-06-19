// bake-msdf.mjs — 离线把 LXGW 常用字集烘成 MSDF atlas(0015)。
//
// 产出:web/public/fonts/lxgw-msdf.png(MSDF 图集,可能多页 .0.png/.1.png)
//       + lxgw-msdf.json(BMFont 格式:chars[].id = coverage 集 / xadvance / atlas bounds,
//         common.lineHeight,distanceField.distanceRange)。
// 运行时(0015):SDF 模式命中 coverage(chars[].id)→ MSDF;未命中 → 回退 TinySDF。
//
// 用法:  node scripts/bake-msdf.mjs
//        node scripts/bake-msdf.mjs --font <ttf> --out <dir> --size 40 --extra scripts/charset-extra.txt
//
// 依赖:msdf-bmfont-xml(npm,**预打包各平台 msdfgen 二进制,无需编译**)
//   npm i -g msdf-bmfont-xml      # 装完得到 `msdf-bmfont` 命令
//   (备选 canonical 工具 msdf-atlas-gen 需从源码 cmake 编译,brew 无 formula;故默认用 npm 版)
// 字符集:ASCII 可打印 + 常用 CJK 标点 + GB2312 一级常用字(~3760,Node TextDecoder('gbk') 生成,无额外依赖)
//        + 可选 --extra 文件。生僻字不烘 → 运行时 TinySDF 回退。

import { execFileSync } from "node:child_process";
import { mkdirSync, writeFileSync, existsSync, readFileSync, statSync, readdirSync, renameSync } from "node:fs";
import { dirname, resolve, basename } from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "..");

// ---- 配置(命令行可覆盖)----
const args = parseArgs(process.argv.slice(2));
const FONT = resolve(ROOT, args.font ?? "lxgw-wenkai-v1.522/LXGWWenKaiMono-Light.ttf");
const OUT_DIR = resolve(ROOT, args.out ?? "web/public/fonts");
const CELL = Number(args.size ?? 40); // 每字形 MSDF 像素;40 ≈ 体积/质量平衡
const RANGE = Number(args.range ?? 4); // 距离场像素范围(distanceRange)
const TYPE = args.type ?? "msdf"; // msdf-bmfont-xml 仅支持 msdf / sdf / psdf(无 mtsdf)
const TEX = args.texture ?? "2048,2048"; // 单页纹理尺寸;字多会自动分多页
const NAME = args.name ?? "lxgw-msdf";
const CLI = args.bin ?? "msdf-bmfont";

if (!existsSync(FONT)) fail(`字体不存在:${FONT}`);

// ---- 1) 组字符集 → 写文件 ----
const chars = buildCharset(args.extra ? resolve(ROOT, args.extra) : null);
console.log(`[bake-msdf] 字符集 ${chars.length} 字(ASCII + CJK 标点 + GB2312 一级 + extra)`);
mkdirSync(OUT_DIR, { recursive: true });
const charsetPath = resolve(OUT_DIR, `${NAME}.charset.txt`);
writeFileSync(charsetPath, chars.join("")); // msdf-bmfont -i 吃"字符直接拼接"的文件

// ---- 2) 跑 msdf-bmfont-xml ----
const outBase = resolve(OUT_DIR, NAME); // 产出 <NAME>.json + <NAME>.png / .0.png…
const cliArgs = [
  FONT,
  "-o", `${outBase}.png`,
  "-f", "json",
  "-t", TYPE,
  "-i", charsetPath,
  "-s", String(CELL),
  "-r", String(RANGE),
  "-m", TEX,
  "--pot",
];
console.log(`[bake-msdf] ${CLI} ${cliArgs.join(" ")}`);
try {
  execFileSync(CLI, cliArgs, { stdio: "inherit" });
} catch (e) {
  if (e.code === "ENOENT") {
    fail(
      `找不到 ${CLI}。装一下(无需编译):\n` +
        `  npm i -g msdf-bmfont-xml\n` +
        `(或 npx msdf-bmfont …;canonical 的 msdf-atlas-gen 需源码 cmake 编译,brew 无 formula。)\n` +
        `或用 --bin <可执行路径> 指定。`,
    );
  }
  fail(`${CLI} 失败:${e.message}`);
}

// ---- 3) 归一 json 名 + 报告 + coverage ----
// msdf-bmfont-xml 怪癖:纹理用 -o 名(lxgw-msdf.*.png),但 config json 用**字体名**。
// 统一重命名成 ${NAME}.json(运行时用稳定名;pages 字段已正确指向 lxgw-msdf.*.png)。
// msdf-bmfont-xml 把 config json 写成**字体名**.json(不是 -o 名)。无条件把它改名到
// ${NAME}.json,**覆盖**任何陈旧同名(换字体重烘必须;旧实现仅在 jsonOut 不存在时改名 →
// 重烘会留旧 json + 新 png 错配,致字形错位)。
const jsonOut = `${outBase}.json`;
const fontJson = resolve(OUT_DIR, `${basename(FONT).replace(/\.[^.]+$/, "")}.json`);
if (existsSync(fontJson) && fontJson !== jsonOut) {
  renameSync(fontJson, jsonOut); // renameSync 在 POSIX 覆盖已存在目标
  console.log(`[bake-msdf] 重命名 ${rel(fontJson)} → ${NAME}.json(覆盖陈旧)`);
}
if (!existsSync(jsonOut)) fail(`未生成 ${rel(jsonOut)}(检查 ${CLI} 输出)`);
const bm = JSON.parse(readFileSync(jsonOut, "utf8"));
const covered = bm.chars?.length ?? 0;
const pages = readdirSync(OUT_DIR).filter((f) => f.startsWith(NAME) && f.endsWith(".png"));
const mb = pages.reduce((s, f) => s + statSync(resolve(OUT_DIR, f)).size, 0) / 1024 / 1024;
console.log(`\n[bake-msdf] 完成:`);
console.log(`  ${rel(jsonOut)}  coverage=${covered} glyphs  fieldType=${bm.distanceField?.fieldType ?? TYPE}`);
console.log(`  纹理页 ${pages.length} 张,共 ${mb.toFixed(2)} MB:${pages.join(", ")}`);
console.log(`  运行时(0015):chars[].id 建 Set 判命中;png 进静态 atlas 页;common.lineHeight / distanceField.distanceRange 喂 shader。`);
console.log(`  注:产物是构建资源;不想入库就在 .gitignore 排除 ${NAME}.png/.json/.charset.txt,改 CI 烘。`);

// ================= helpers =================

function buildCharset(extraPath) {
  const set = new Set();
  for (let c = 0x20; c <= 0x7e; c++) set.add(c); // ASCII 可打印
  for (const ch of "　、。〃々〈〉《》「」『』【】〔〕〖〗！？，；：（）“”‘’…—·～％＃＠＆＊＋－＝／＼｜") set.add(ch.codePointAt(0));
  // GB2312 一级常用字(~3760):lead 0xB0–0xD7,trail 0xA1–0xFE,TextDecoder('gbk') 解码,无额外依赖。
  const dec = new TextDecoder("gbk", { fatal: true });
  for (let hi = 0xb0; hi <= 0xd7; hi++) {
    for (let lo = 0xa1; lo <= 0xfe; lo++) {
      try {
        const ch = dec.decode(new Uint8Array([hi, lo]));
        if (ch && ch !== "�") set.add(ch.codePointAt(0));
      } catch {
        /* 空位跳过 */
      }
    }
  }
  if (extraPath && existsSync(extraPath)) {
    for (const ch of readFileSync(extraPath, "utf8")) {
      const cp = ch.codePointAt(0);
      if (cp > 0x20) set.add(cp);
    }
  }
  return [...set].sort((a, b) => a - b).map((cp) => String.fromCodePoint(cp));
}

function parseArgs(argv) {
  const o = {};
  for (let i = 0; i < argv.length; i++) {
    if (argv[i].startsWith("--")) o[argv[i].slice(2)] = argv[i + 1]?.startsWith("--") ? true : argv[++i];
  }
  return o;
}
function rel(p) {
  return p.startsWith(ROOT) ? p.slice(ROOT.length + 1) : p;
}
function fail(msg) {
  console.error(`[bake-msdf] ✗ ${msg}`);
  process.exit(1);
}
