// resilience.spec.ts — Plan 24 PR-2(TC-F13/F21/F22):容错经 push_event 合成事件驱动(无需真 server)。
// 浏览器端验 F1–F12 的可观测面;wall-clock(F1/F2)逻辑另在 sse-client.test.ts(vitest)。
import { test, expect } from "@playwright/test";
import { bootVisible } from "./helpers";

test.use({ permissions: ["clipboard-read", "clipboard-write"] });

// 用 bootVisible:它等到 showcase 重放**耗尽 + 稳定**,此后引擎安静 → push_event 独占驱动 FSM
//(否则 showcase 残余 delta 会抢 FSM / 清掉合成错误卡,污染容错断言)。
test.beforeEach(async ({ page }) => {
  await bootVisible(page);
});

const countErrorCards = async (page: import("@playwright/test").Page): Promise<number> => {
  const runs = await page.evaluate(() => window.__chat.visible_text_runs());
  return (runs.match(/\[error\]/g) ?? []).length;
};

test("TC-F21 错误卡恒一张(F4)", async ({ page }) => {
  const err = JSON.stringify({
    type: "message.updated",
    properties: { info: { id: "m1", role: "assistant", error: { name: "APIError" } } },
  });
  await page.evaluate((e) => {
    window.__chat.push_event(e);
    window.__chat.push_event(e); // 重复错误
  }, err);
  await page.waitForTimeout(200);
  await page.evaluate(() => window.__chat.scroll_to(1)); // 错误卡 view
  await page.waitForTimeout(120);
  await expect.poll(() => countErrorCards(page)).toBe(1);
  expect(await page.evaluate(() => window.__chat.session_status())).toBe("errored");
});

test("TC-F22 ghost-abort 无错误卡(F3)", async ({ page }) => {
  await page.evaluate(() =>
    window.__chat.push_event(
      JSON.stringify({
        type: "session.error",
        properties: { sessionID: "s", error: { name: "AbortError" } },
      }),
    ),
  );
  await page.waitForTimeout(200);
  expect(await countErrorCards(page), "ghost-abort 不弹卡").toBe(0);
  expect(await page.evaluate(() => window.__chat.session_status())).toBe("idle");
});

test("TC-F13 stop 冻结消息(F11)", async ({ page }) => {
  // 进入流式(note_send → 首 part)→ stop → 之后同消息事件被丢弃。
  await page.evaluate(() => {
    window.__chat.note_send();
    window.__chat.push_event(
      JSON.stringify({
        type: "message.part.delta",
        properties: { messageID: "mz", partID: "pz", field: "text", delta: "AAA" },
      }),
    );
  });
  await expect
    .poll(() => page.evaluate(() => window.__chat.session_status()))
    .toBe("streaming");
  await page.evaluate(() => window.__chat.stop_turn());
  expect(await page.evaluate(() => window.__chat.session_status())).toBe("stopped");
  // 冻结后再喂同消息 → 丢弃(文本不增长)。
  await page.evaluate(() =>
    window.__chat.push_event(
      JSON.stringify({
        type: "message.part.delta",
        properties: { messageID: "mz", partID: "pz", field: "text", delta: "BBB" },
      }),
    ),
  );
  await page.waitForTimeout(200);
  await page.evaluate(() => window.__chat.scroll_to(1));
  await page.waitForTimeout(120);
  const runs = await page.evaluate(() => window.__chat.visible_text_runs());
  expect(runs, "冻结消息的后续段被丢弃").not.toContain("BBB");
  expect(runs, "停止前内容仍在").toContain("AAA");
});
