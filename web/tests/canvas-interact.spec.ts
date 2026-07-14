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
  await page.goto("/dev.html?replay=showcase&noinput", { waitUntil: "domcontentloaded" });
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

// Plan 34 S1(0038):Released 态布局稳定 —— 脱离跟随后流式增高,视口内既有内容像素级零位移。
test("S1 released 态流式增高零位移(0038 布局稳定不变量)", async ({ page }) => {
  await bootVisible(page); // showcase 全揭示 settled(稳定等待,非固定 sleep)
  // 上滚脱离(释放信号)。既有内容此后应零位移;新内容只出现在视口下方屏外
  //(0038:Released 不冻内容,只冻相机)。
  await page.evaluate(() => window.__chat.pan_by(0, -260));
  await page.waitForTimeout(400);
  expect(await page.evaluate(() => window.__chat.follow_state())).toBe("released");
  const clip = { x: 300, y: 100, width: 480, height: 300 }; // 视口中部既有内容
  const before = (await page.screenshot({ clip })).toString("base64");
  // 追加新消息(增高);Released 下不得推动视口。
  await page.evaluate(() =>
    window.__chat.push_event(
      JSON.stringify({
        type: "message.part.updated",
        properties: {
          part: { type: "text", id: "grow", messageID: "mgrow", text: "released growth tail ".repeat(40) },
          time: 2,
        },
      }),
    ),
  );
  await page.waitForTimeout(600);
  expect(await page.evaluate(() => window.__chat.follow_state()), "增长不触发重进入").toBe(
    "released",
  );
  const after = (await page.screenshot({ clip })).toString("base64");
  expect(after, "既有内容像素零位移").toBe(before);
  // jump-to-latest 正路:平滑滚底并恢复跟随。
  await page.evaluate(() => window.__chat.scroll_to_latest());
  await expect
    .poll(() => page.evaluate(() => window.__chat.follow_state()), { timeout: 5_000 })
    .toBe("following");
});
