// report-format.mjs — TESTREPORT 机器可读摘要块(Plan 35 T1 / 调研 §4.1)。
// 独立模块以便 schema 测试(web/src/report-schema.test.ts);run.mjs 引用。

/** 机器摘要对象(schema 契约:result/commit/stamp/totals/suites[])。 */
export function machineSummary(result, commit, stamp, totals, runs) {
  return {
    result,
    commit,
    stamp,
    totals, // { passed, failed, skipped, ms }
    suites: runs.map((s) => ({
      suite: s.suite,
      passed: s.passed,
      failed: s.failed,
      skipped: s.skipped,
      ms: s.ms,
    })),
  };
}

/** TESTREPORT.md 里的 SUMMARY-JSON 段(围栏 json 块;单行紧凑)。 */
export function summaryJsonSection(summary) {
  return `## SUMMARY-JSON\n\n\`\`\`json\n${JSON.stringify(summary)}\n\`\`\`\n`;
}

/** 从 TESTREPORT.md 文本抽回机器摘要(消费端;找不到/坏 JSON → null,不 throw)。 */
export function parseSummaryJson(md) {
  const m = /## SUMMARY-JSON\s*\n+```json\n([\s\S]*?)\n```/.exec(md);
  if (!m) return null;
  try {
    return JSON.parse(m[1]);
  } catch {
    return null;
  }
}
