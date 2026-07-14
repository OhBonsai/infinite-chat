#!/usr/bin/env node
// gen-pages-assets.mjs — Plan 40 P3:landing 功能全览资产导出(单命令)。
//
// 为什么入库:CI 无 GPU 不能现场捕获 → landing 卡片的截图/短片必须本地生成并**提交**。
// 与 plan39 体检同源同 DSL 同确定性 —— 只捕 manifest 里 `pages: true` 的精选项(FEATURE_PAGES=1),
// 拷到 web/public/pages-assets/ + 写 index.json(home.ts 据此渲九锚点卡片)。
//
// 用法:
//   node scripts/gen-pages-assets.mjs            # 捕获精选 → 拷贝 → 写卡片清单 → 体积对账
//   node scripts/gen-pages-assets.mjs --copy-only # 跳过捕获,用现有 feature-assets 拷贝(调试卡片)
//
// 体积预算:≤15MB(超则非 0 退出,提示删/换 webm)。webm 段数由 manifest 精选控制(设计 ≤6)。
import { spawnSync } from "node:child_process";
import { copyFileSync, mkdirSync, readFileSync, readdirSync, rmSync, statSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const SRC = path.join(ROOT, "test", "results", "feature-assets");
const DEST = path.join(ROOT, "web", "public", "pages-assets");
const BUDGET_MB = 15;
const copyOnly = process.argv.includes("--copy-only");

const manifest = JSON.parse(readFileSync(path.join(ROOT, "test", "feature-manifest.json"), "utf8"));
const groups = manifest.groups; // 有序:九大项
const picks = manifest.items.filter((it) => it.pages === true);
if (picks.length === 0) {
  console.error("gen-pages-assets: manifest 无 pages:true 项 —— 先标记精选");
  process.exit(1);
}

function run(cmd, args, opts = {}) {
  const r = spawnSync(cmd, args, { cwd: ROOT, stdio: "inherit", ...opts });
  if (r.status !== 0) process.exit(r.status ?? 1);
}

// 1) 捕获(FEATURE_PAGES=1 → spec 只跑 pages 精选;报告即测试,任一项失败即非 0)。
if (!copyOnly) {
  run("npx", ["playwright", "test", "--config", "feature/playwright.config.ts"], {
    cwd: path.join(ROOT, "web"),
    env: { ...process.env, FEATURE_PAGES: "1" },
  });
}

// 2) 清空并重建 DEST(避免旧精选残留),按 manifest 顺序拷贝命中项。
rmSync(DEST, { recursive: true, force: true });
mkdirSync(DEST, { recursive: true });

const cards = [];
const missing = [];
for (const it of picks) {
  const asset = `${it.id}.${it.kind}`;
  const from = path.join(SRC, asset);
  try {
    statSync(from);
  } catch {
    missing.push(asset);
    continue;
  }
  copyFileSync(from, path.join(DEST, asset));
  cards.push({
    id: it.id,
    group: it.group,
    kind: it.kind,
    title: it.pagesTitle ?? it.title, // landing 专属标题(如整幅 Markdown 全览卡)覆盖体检项标题
    desc: it.pagesDesc ?? it.desc ?? "",
    file: asset,
  });
}
if (missing.length) {
  console.error(`gen-pages-assets: 精选资产缺失 ${missing.length} 项(先跑捕获,去掉 --copy-only):`);
  for (const m of missing) console.error(`  - ${m}`);
  process.exit(1);
}

// 3) 写卡片清单 index.json:九大项分区(有序)+ 每项精选卡(有序);home.ts 直接渲染。
const sections = groups
  .map((g) => ({
    id: g.id,
    title: g.title,
    desc: g.desc,
    cards: cards.filter((c) => c.group === g.id),
  }))
  .filter((s) => s.cards.length > 0);
writeFileSync(
  path.join(DEST, "index.json"),
  JSON.stringify({ version: 1, sections }, null, 2) + "\n",
);

// 4) 体积对账(≤15MB 预算;超则失败,提示收敛)。
let total = 0;
const sizes = [];
for (const f of readdirSync(DEST)) {
  const sz = statSync(path.join(DEST, f)).size;
  total += sz;
  sizes.push([f, sz]);
}
sizes.sort((a, b) => b[1] - a[1]);
const mb = (n) => (n / 1024 / 1024).toFixed(2);
console.log(`\npages-assets: ${cards.length} 卡 · ${sections.length} 区 · 合计 ${mb(total)}MB / ${BUDGET_MB}MB`);
for (const [f, sz] of sizes.slice(0, 6)) console.log(`  ${mb(sz).padStart(6)}MB  ${f}`);
if (total > BUDGET_MB * 1024 * 1024) {
  console.error(`\ngen-pages-assets: 超体积预算 ${mb(total)}MB > ${BUDGET_MB}MB —— 删项或把 webm 降段/降码率`);
  process.exit(1);
}
console.log("gen-pages-assets: OK");
