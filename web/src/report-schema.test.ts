// report-schema.test.ts — Plan 35 T1:TESTREPORT 机器摘要块 schema 锁定(调研 §4.1)。
// 生成 → 抽回往返;缺段/坏 JSON 容错(消费端不 throw)。
import { describe, expect, it } from "vitest";
// @ts-expect-error 纯 mjs 模块无类型声明(测试内往返验证 schema,不需要类型)
import { machineSummary, summaryJsonSection, parseSummaryJson } from "../../test/report-format.mjs";

const runs = [
  { suite: "gates", passed: 4, failed: 0, skipped: 0, ms: 3000, cases: [] },
  { suite: "native", passed: 300, failed: 1, skipped: 2, ms: 90_000, cases: [] },
];
const totals = { passed: 304, failed: 1, skipped: 2, ms: 93_000 };

describe("TESTREPORT SUMMARY-JSON", () => {
  it("生成 → 抽回往返,schema 键齐全", () => {
    const sum = machineSummary("FAIL", "abc1234", "2026-07-14T00:00:00Z", totals, runs);
    const md = `# TESTREPORT — x\nRESULT: FAIL\n\n## 各层\n…\n\n${summaryJsonSection(sum)}`;
    const back = parseSummaryJson(md);
    expect(back).toEqual(sum);
    expect(back.result).toBe("FAIL");
    expect(back.totals).toEqual(totals);
    expect(back.suites).toHaveLength(2);
    for (const s of back.suites) {
      expect(Object.keys(s).sort()).toEqual(["failed", "ms", "passed", "skipped", "suite"]);
    }
    // cases 明细不入摘要(concise 契约:细节留 summary.json)。
    expect(JSON.stringify(back)).not.toContain("cases");
  });

  it("缺段 / 坏 JSON → null(消费端不 throw)", () => {
    expect(parseSummaryJson("# TESTREPORT\n无摘要段")).toBeNull();
    expect(parseSummaryJson("## SUMMARY-JSON\n\n```json\n{oops\n```")).toBeNull();
  });
});
