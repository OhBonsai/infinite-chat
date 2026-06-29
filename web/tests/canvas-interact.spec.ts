// canvas-interact.spec.ts — Plan 24 PR-2(TC-F18):滚动/缩放/锚底。
import { test, expect } from "@playwright/test";
import { bootVisible, readTurns } from "./helpers";

test("TC-F18 平移改变相机上报位置(滚动生效)", async ({ page }) => {
  await bootVisible(page); // 滚到顶、内容载满
  const dpr = await page.evaluate(() => window.devicePixelRatio || 1);
  const before = (await readTurns(page))[0];
  await page.evaluate(() => window.__chat.pan_by(240, 0)); // 水平平移(不被纵向锚底取消)
  await page.waitForTimeout(150);
  const after = (await readTurns(page)).find((t) => t.id === before.id);
  expect(after, "消息仍可见").toBeTruthy();
  expect(Math.abs(after!.x - before.x), "相机平移后上报屏幕 x 变化").toBeGreaterThan(20 * dpr * 0.5);
});

test("TC-F18 锚底:流式新内容进入可见(底部跟随)", async ({ page }) => {
  await page.goto("/?replay=showcase&noinput", { waitUntil: "domcontentloaded" });
  await page.waitForFunction(() => !!(window as unknown as { __chat?: unknown }).__chat, null, {
    timeout: 60_000,
  });
  await page.evaluate(() => {
    window.__chat.set_stream_rate(1e9);
    window.__chat.set_reveal_cps(1e9);
  });
  // 注入一条带独特词的新消息 → 锚底应跟随,使其进入可见窗。
  await page.evaluate(() =>
    window.__chat.push_event(
      JSON.stringify({
        type: "message.part.updated",
        properties: {
          part: { type: "text", id: "tail", messageID: "mtail", text: "ANCHORWORDXYZ" },
          time: 1,
        },
      }),
    ),
  );
  await expect
    .poll(() => page.evaluate(() => window.__chat.visible_text_runs()), { timeout: 10_000 })
    .toContain("ANCHORWORDXYZ");
});
