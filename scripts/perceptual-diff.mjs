#!/usr/bin/env node
// perceptual-diff.mjs — Plan 35 T5(可选档):golden 对比的感知模式(仅新增,不替换逐字节)。
// 逐像素色差容忍(默认 ≤12/255)+ 变化占比阈值(默认 ≤0.5%)→ 抗 AA/驱动微抖,仍抓真回归。
// 用法:node scripts/perceptual-diff.mjs a.png b.png [--tolerance 12] [--max-ratio 0.005]
// 退出码:0 = 感知等同 / 1 = 超阈值 / 2 = 输入错误。**不进门槛**(跑通即可,升级档)。
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";

const require = createRequire(import.meta.url);
const { PNG } = require("../web/node_modules/pngjs");

const [a, b] = process.argv.slice(2).filter((x) => !x.startsWith("--"));
const opt = (n, d) => {
  const i = process.argv.indexOf(n);
  return i >= 0 ? Number(process.argv[i + 1]) : d;
};
const TOL = opt("--tolerance", 12);
const MAX = opt("--max-ratio", 0.005);
if (!a || !b) {
  console.error("用法: perceptual-diff.mjs a.png b.png [--tolerance N] [--max-ratio R]");
  process.exit(2);
}
const pa = PNG.sync.read(readFileSync(a));
const pb = PNG.sync.read(readFileSync(b));
if (pa.width !== pb.width || pa.height !== pb.height) {
  console.error(`尺寸不同: ${pa.width}x${pa.height} vs ${pb.width}x${pb.height}`);
  process.exit(1);
}
let changed = 0;
for (let i = 0; i < pa.data.length; i += 4) {
  const d = Math.max(
    Math.abs(pa.data[i] - pb.data[i]),
    Math.abs(pa.data[i + 1] - pb.data[i + 1]),
    Math.abs(pa.data[i + 2] - pb.data[i + 2]),
  );
  if (d > TOL) changed++;
}
const ratio = changed / (pa.width * pa.height);
const ok = ratio <= MAX;
console.log(
  `${ok ? "PERCEPTUAL PASS" : "PERCEPTUAL FAIL"} changed=${changed}px ratio=${(ratio * 100).toFixed(3)}% (tol=${TOL}, max=${(MAX * 100).toFixed(2)}%)`,
);
process.exit(ok ? 0 : 1);
