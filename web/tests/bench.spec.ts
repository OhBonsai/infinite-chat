// bench.spec.ts — Plan 18 §3.2/§4 浏览器侧规模/内存基线采集(无人值守)。
//
// 驱动 `?bench` 长会话页 → main.ts 采样器每 1s 写一行,内容停增长后导出 `window.__benchCSV`
// (含 fps / wasmMiB,native 测不到的两项)。本测把 CSV 落 `bench-results/` + 打印 + 基本断言。
import { test, expect } from "@playwright/test";
import { mkdirSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";

test("plan18 scale/memory before baseline (browser)", async ({ page }) => {
  const logs: string[] = [];
  page.on("console", (m) => logs.push(`[${m.type()}] ${m.text()}`));
  page.on("pageerror", (e) => logs.push(`[pageerror] ${e.message}`));

  // BENCH_QS 追加查询(如 sizefold=P1-off 对照、spread 调节);默认 spread=60。
  const qs = process.env.BENCH_QS || "spread=60";
  await page.goto(`/?bench&debug&${qs}`, { waitUntil: "domcontentloaded" });

  // 先确认 WebGPU 可用(否则引擎起不来、采样器永不跑 → 早失败给清晰诊断)。
  const gpu = await page.evaluate(async () => {
    const g = (navigator as unknown as { gpu?: { requestAdapter(): Promise<unknown> } }).gpu;
    if (!g) return { ok: false, why: "navigator.gpu 缺失" };
    try {
      const a = await g.requestAdapter();
      return { ok: !!a, why: a ? "ok" : "requestAdapter 返回 null" };
    } catch (e) {
      return { ok: false, why: String(e) };
    }
  });
  console.log(`[bench] WebGPU adapter: ${gpu.ok ? "OK" : "FAIL — " + gpu.why}`);

  // 等采样器导出 CSV(内容停增长后置 window.__benchCSV);载入 ~15s,给足 100s。
  let csv = "";
  try {
    await page.waitForFunction(
      () => typeof (window as unknown as { __benchCSV?: string }).__benchCSV === "string",
      null,
      { timeout: 100_000 },
    );
    csv = await page.evaluate(() => (window as unknown as { __benchCSV: string }).__benchCSV);
  } catch {
    console.log("[bench] 未拿到 __benchCSV。最近控制台:\n" + logs.slice(-25).join("\n"));
    throw new Error("bench CSV 未生成(多半 WebGPU 未在 headless 起来,见上方 adapter 行)");
  }

  // 末帧 stats(直接读 window.__chat,交叉校验)。
  const finalStats = await page.evaluate(() => {
    const c = (window as unknown as { __chat?: { stats(): Record<string, number> } }).__chat;
    return c ? c.stats() : null;
  });

  const dir = resolve(process.cwd(), "bench-results");
  mkdirSync(dir, { recursive: true });
  // BENCH_OUT 控输出文件(默认不覆盖 before 基线;P1/P2 after 跑各传自己的名)。
  const out = resolve(dir, process.env.BENCH_OUT || "plan18-before-browser.csv");
  writeFileSync(out, csv);

  // 稳态(turns 达峰的尾部行)取 fps / 帧时的 median + P95 —— headless fps 抖,不取单次(plan19 §2)。
  const rows = csv.trim().split("\n");
  const head = rows[0].split(",");
  const col = (name: string) => head.indexOf(name);
  const data = rows.slice(1).map((r) => r.split(",").map(Number));
  const maxTurns = Math.max(...data.map((r) => r[col("turns")]));
  const steady = data.filter((r) => r[col("turns")] === maxTurns);
  const pick = (name: string) =>
    steady.map((r) => r[col(name)]).sort((a, b) => a - b);
  const median = (xs: number[]) => xs[Math.floor(xs.length / 2)] ?? 0;
  const p95 = (xs: number[]) => xs[Math.min(xs.length - 1, Math.floor(xs.length * 0.95))] ?? 0;
  const fpsS = pick("fps");
  const fmsS = pick("frameMsAvg");
  const summary = {
    out: process.env.BENCH_OUT || "plan18-before-browser.csv",
    maxTurns,
    retainedGlyphs: finalStats?.retainedGlyphs,
    tierHot: finalStats?.tierHot,
    tierWarm: finalStats?.tierWarm,
    rebuilds_median: median(pick("rebuilds")),
    fps_median: median(fpsS),
    fps_p95_low: fpsS[0], // 排序后最低 fps = 最差帧
    frameMs_median: median(fmsS),
    frameMs_p95: p95(fmsS),
    // per-phase 中位(plan19 §2 归因:哪段是 fps 瓶颈)。
    ph_layout_median: median(pick("phBfLayout")),
    ph_grid_median: median(pick("phBfGrid")),
    ph_emit_median: median(pick("phBfEmit")),
    ph_advance_median: median(pick("phAdvance")),
    adv_ingest_median: median(pick("phAdvIngest")),
    adv_roles_median: median(pick("phAdvRoles")),
    adv_reveal_median: median(pick("phAdvReveal")),
    adv_ensure_median: median(pick("phAdvEnsure")),
    adv_schedule_median: median(pick("phAdvSchedule")),
    wasmMiB: steady.at(-1)?.[col("wasmMiB")],
    steadySamples: steady.length,
  };
  console.log(`\n[bench] CSV → ${out}`);
  console.log(`[bench] SUMMARY ${JSON.stringify(summary)}`);

  // 断言:全载(达 200 turn)+ 稳态有样本(让 fps 归因可信);具体 fps 门槛由调用方比对前后。
  expect(maxTurns).toBeGreaterThanOrEqual(150);
  expect(steady.length).toBeGreaterThanOrEqual(3);
  expect(median(fpsS)).toBeGreaterThan(0);

  // ── Plan 29 V4(DoD-1):滚动来回 → 内存回落 ≤ 稳态 ×1.10(0029 初心「内存跟可见屏走」)。
  // 滚顶(远端块降 Cold)→ 回底 → 采 wasm 线性内存与四档计数;before(plan18)此段只涨不降。
  const mem = () =>
    page.evaluate(() => {
      const w = window as unknown as {
        __wasmMemory?: { buffer: ArrayBuffer };
        __chat: { stats(): Record<string, number> };
      };
      return {
        mib: w.__wasmMemory ? w.__wasmMemory.buffer.byteLength / 1048576 : -1,
        st: w.__chat.stats(),
      };
    });
  const settle = async (frames: number) => page.waitForTimeout(frames * 16 + 200);
  const bottom = await mem();
  await page.evaluate(() => window.__chat.scroll_to(0));
  await settle(120); // 顶部停留:滞回带外的底部远端块逐级降档
  const atTop = await mem();
  await page.evaluate((n) => window.__chat.scroll_to(n), maxTurns - 1);
  await settle(120);
  const back = await mem();
  console.log(
    `[bench] roundtrip MiB bottom=${bottom.mib.toFixed(1)} top=${atTop.mib.toFixed(1)} back=${back.mib.toFixed(1)} ` +
      `tiers back=${back.st.tierHot}h/${back.st.tierWarm}w/${back.st.tierCold}c/${back.st.tierFrozen}f`,
  );
  if (bottom.mib > 0) {
    // wasm 线性内存不收缩(brk 语义)→ 判据:来回滚动**不使峰值继续攀升**(≤ 稳态 ×1.10),
    // 且驻留量(retainedGlyphs ∝ Hot 工作集)回底后与滚动前同量级。
    expect(back.mib).toBeLessThanOrEqual(bottom.mib * 1.1);
    expect(back.st.tierCold + back.st.tierFrozen).toBeGreaterThan(0);
  }
});
