// integration-smoke.spec.ts — Plan 24 PR-2(TC-F01/F03):冒烟链。连接/历史可见 + 流式文本收尾。
import { test, expect } from "@playwright/test";
import { assertWebGpu, bootVisible, readTurns } from "./helpers";

test("TC-F01 连接 + 历史可见", async ({ page }) => {
  await page.goto("/dev.html?replay=showcase&noinput", { waitUntil: "domcontentloaded" });
  await assertWebGpu(page);
  await page.waitForFunction(() => !!(window as unknown as { __chat?: unknown }).__chat, null, {
    timeout: 60_000,
  });
  // 重放 = 历史灌入;可见消息非空。
  await expect.poll(() => readTurns(page).then((t) => t.length), { timeout: 30_000 }).toBeGreaterThan(0);
});

test("TC-F03 流式文本 + idle 收尾", async ({ page }) => {
  await bootVisible(page); // 等 showcase 重放耗尽 → 引擎安静,push 的流式独占驱动 FSM
  await page.evaluate(() => {
    window.__chat.note_send();
    // 流式增量(同 message/part)。
    for (const d of ["流式", "文本", "收尾XYZ"]) {
      window.__chat.push_event(
        JSON.stringify({
          type: "message.part.delta",
          properties: { messageID: "ms", partID: "ps", field: "text", delta: d },
        }),
      );
    }
  });
  await expect
    .poll(() => page.evaluate(() => window.__chat.session_status()))
    .toBe("streaming");
  // idle 收尾。
  await page.evaluate(() =>
    window.__chat.push_event(
      JSON.stringify({ type: "session.status", properties: { status: { type: "idle" } } }),
    ),
  );
  await expect.poll(() => page.evaluate(() => window.__chat.session_status())).toBe("idle");
  // 流式文本进入可见。
  await page.evaluate(() => window.__chat.scroll_to(1));
  await page.waitForTimeout(150);
  expect(await page.evaluate(() => window.__chat.visible_text_runs())).toContain("收尾XYZ");
});
