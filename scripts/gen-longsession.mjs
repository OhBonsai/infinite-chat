// gen-longsession.mjs — Plan 18 §3.1:合成长会话重放 case(N 行,多 turn)。
//
// 产物 = `web/public/replays/longsession-<lines>.json`,格式 = ChatCanvas 直吃的 records
// `[{ t, raw }]`(raw = opencode `message.part.delta` 信封)。每个 turn 一个独立 part/message
// (partID=p{i} / messageID=m{i}),`t=i+0.5` 配 `?bench` 的 Player step → 每帧释放一个 turn。
//
// 内容混合(覆盖各 BlockCache 子结构):60% 段落 / 15% 列表 / 15% 代码 / 10% 表格。
// **与 crates/core app.rs `bench_turn_md` 保持一致** → native(retained_*)与浏览器(fps/内存)可对比。
//
// 用法:node scripts/gen-longsession.mjs --lines 10000 --turns 200

import { writeFileSync, mkdirSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const __dir = dirname(fileURLToPath(import.meta.url));

function arg(name, def) {
  const i = process.argv.indexOf(`--${name}`);
  return i >= 0 && process.argv[i + 1] ? Number(process.argv[i + 1]) : def;
}

const TURNS = arg("turns", 200);
const LINES = arg("lines", 10000);
const linesPerTurn = Math.max(4, Math.round(LINES / TURNS));

/// 一个 turn 的混合 markdown(与 app.rs bench_turn_md 逐字对应)。
function turnMd(turn, lines) {
  const para = Math.floor((lines * 6) / 10);
  const list = Math.floor((lines * 15) / 100);
  const code = Math.floor((lines * 15) / 100);
  const tableRows = Math.max(2, lines - para - list - code);
  let s = `# Turn ${turn}\n\n`;
  for (let i = 0; i < para; i++)
    s += `This is paragraph line ${i} of turn ${turn}, some **bold** and \`code\` words.\n`;
  s += "\n";
  for (let i = 0; i < list; i++) s += `- list item ${i} in turn ${turn}\n`;
  s += "\n```rust\n";
  for (let i = 0; i < code; i++) s += `let x${i} = ${i} + turn_${turn};\n`;
  s += "```\n\n| col_a | col_b |\n| --- | --- |\n";
  for (let i = 0; i < tableRows; i++) s += `| r${i}a | r${i}b |\n`;
  s += "\n";
  return s;
}

const records = [];
for (let i = 0; i < TURNS; i++) {
  const raw = JSON.stringify({
    type: "message.part.delta",
    properties: {
      sessionID: "bench",
      messageID: `m${i}`,
      partID: `p${i}`,
      field: "text",
      delta: turnMd(i, linesPerTurn),
    },
  });
  records.push({ t: i + 0.5, raw });
}

const outDir = resolve(__dir, "../web/public/replays");
mkdirSync(outDir, { recursive: true });
const tag = LINES >= 1000 ? `${Math.round(LINES / 1000)}k` : `${LINES}`;
const out = resolve(outDir, `longsession-${tag}.json`);
writeFileSync(out, JSON.stringify(records));
const bytes = records.reduce((a, r) => a + r.raw.length, 0);
console.log(
  `[gen-longsession] ${TURNS} turns × ${linesPerTurn} 行 ≈ ${TURNS * linesPerTurn} 行,` +
    `${(bytes / 1024).toFixed(0)} KiB → ${out}`,
);
