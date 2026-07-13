// perf-gate.test.ts — Plan 35 T2:perf 门红/绿判定自测(构造 CSV;调研 §3 确定性代理指标)。
import { describe, expect, it } from "vitest";
// @ts-expect-error 纯 mjs 模块无类型声明(测试构造输入验红绿,不需要类型)
import { compare, parseCsv, renderTable } from "../../test/perf-compare.mjs";

const HEADER = "retainedGlyphs,wasmMiB,frameMsAvg,phBfTotal,phAdvance,fps";
const csv = (rows: number[][]) => [HEADER, ...rows.map((r) => r.join(","))].join("\n");
// 基线:三行采样(耗时中位数取中行)。
const BASE = csv([
  [9000, 21.0, 5.0, 0.8, 0.5, 60],
  [9000, 21.0, 6.0, 0.9, 0.5, 60],
  [9000, 21.0, 7.0, 1.0, 0.6, 60],
]);

describe("perf-gate compare", () => {
  it("基线内 → 绿(含耗时 +25% 宽阈值内波动)", () => {
    const cur = csv([
      [9400, 22.0, 6.0, 1.0, 0.6, 58], // retained +4.4% / wasm +4.8% / frameMs 中位 +0…
      [9400, 22.0, 7.0, 1.1, 0.6, 58],
      [9400, 22.0, 7.4, 1.1, 0.6, 58],
    ]);
    const out = compare(cur, BASE);
    expect(out.pass, renderTable(out)).toBe(true);
  });

  it("构造 +15% 结构恶化 → 红(retainedGlyphs 超 +10% 门)", () => {
    const cur = csv([[10350, 21.0, 6.0, 0.9, 0.5, 60]]); // +15%
    const out = compare(cur, BASE);
    expect(out.pass).toBe(false);
    const r = out.results.find((x: { name: string }) => x.name === "retainedGlyphs");
    expect(r.ok).toBe(false);
  });

  it("fps 恶化不判负(只记录,headless fps 不可信)", () => {
    const cur = csv([[9000, 21.0, 6.0, 0.9, 0.5, 12]]); // fps 掉 80%
    expect(compare(cur, BASE).pass).toBe(true);
  });

  it("缺列/空 CSV → 指标跳过不假红;确定性:同输入同判定", () => {
    const out = compare("a,b\n1,2", BASE); // 全指标缺列
    expect(out.pass).toBe(true);
    expect(out.results.every((r: { ok: boolean | null }) => r.ok === null || r.ok === true)).toBe(true);
    expect(parseCsv("").rows).toEqual([]);
    const c = csv([[9400, 22.0, 6.0, 1.0, 0.6, 58]]);
    expect(compare(c, BASE)).toEqual(compare(c, BASE));
  });
});
