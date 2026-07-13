// links.spec.ts — Plan 34 S2:可点链接(闭合可点 / 半截不可点 / 选区拖拽不误触)。
// 引擎不导航(0000 §2.2):点击只经 onOpenUrl 回调交宿主 —— 测试注册捕获回调断言 URL。
import { test, expect, type Page } from "@playwright/test";

declare global {
  interface Window {
    __openedUrl?: string;
  }
}

async function bootEmpty(page: Page): Promise<void> {
  await page.goto("/?empty&noinput", { waitUntil: "domcontentloaded" });
  await page.waitForFunction(() => !!(window as unknown as { __chat?: unknown }).__chat, null, {
    timeout: 60_000,
  });
  await page.evaluate(() => {
    window.__chat.set_stream_rate(1e9);
    window.__chat.set_reveal_cps(1e9);
    window.__chat.set_open_url_handler((u: string) => {
      window.__openedUrl = u;
    });
  });
}

function pushText(page: Page, id: string, text: string): Promise<void> {
  return page.evaluate(
    ([pid, t]) =>
      window.__chat.push_event(
        JSON.stringify({
          type: "message.part.updated",
          properties: { part: { type: "text", id: pid, messageID: "m1", text: t }, time: 1 },
        }),
      ),
    [id, text],
  );
}

test("S2 闭合链接可点:回调收到精确 URL;半截链接不可点", async ({ page }) => {
  await bootEmpty(page);
  await pushText(page, "p1", "文档见 [官网](https://example.com/docs) 结尾。");
  // settled 后命中盒才登记(AR3)。
  await expect
    .poll(() => page.evaluate(() => JSON.parse(window.__chat.link_hit_targets()).length), {
      timeout: 10_000,
    })
    .toBe(1);
  const t = await page.evaluate(
    () => JSON.parse(window.__chat.link_hit_targets())[0] as { x: number; y: number; w: number; h: number },
  );
  // 真实鼠标点击(走 input.ts click-vs-drag 判定路径);坐标为设备像素 → 换算 CSS px。
  const dpr = await page.evaluate(() => window.devicePixelRatio || 1);
  const canvasBox = (await page.locator("canvas").first().boundingBox())!;
  await page.mouse.click(
    canvasBox.x + (t.x + t.w / 2) / dpr,
    canvasBox.y + (t.y + t.h / 2) / dpr,
  );
  await expect
    .poll(() => page.evaluate(() => window.__openedUrl), { timeout: 5_000 })
    .toBe("https://example.com/docs");

  // 半截链接(流式成形中)不产命中盒。
  await pushText(page, "p2", "键入中 [半截](https://exa");
  await page.waitForTimeout(800);
  const urls = await page.evaluate(
    () => (JSON.parse(window.__chat.link_hit_targets()) as Array<{ url: string }>).map((x) => x.url),
  );
  expect(urls, "半截链接无命中盒").toEqual(["https://example.com/docs"]);
});

test("S2 选区拖拽不误触发点击", async ({ page }) => {
  await bootEmpty(page);
  await pushText(page, "p1", "拖选这段 [链接文字](https://example.com/drag) 不应打开。");
  await expect
    .poll(() => page.evaluate(() => JSON.parse(window.__chat.link_hit_targets()).length), {
      timeout: 10_000,
    })
    .toBe(1);
  const t = await page.evaluate(
    () => JSON.parse(window.__chat.link_hit_targets())[0] as { x: number; y: number; w: number; h: number },
  );
  const dpr = await page.evaluate(() => window.devicePixelRatio || 1);
  const canvasBox = (await page.locator("canvas").first().boundingBox())!;
  const cx = canvasBox.x + (t.x + t.w / 2) / dpr;
  const cy = canvasBox.y + (t.y + t.h / 2) / dpr;
  // 按下 → 拖出 60px → 松开:input.ts 应判定为拖拽(选区),不派发 tap。
  await page.mouse.move(cx, cy);
  await page.mouse.down();
  await page.mouse.move(cx + 60, cy, { steps: 8 });
  await page.mouse.up();
  await page.waitForTimeout(400);
  expect(await page.evaluate(() => window.__openedUrl), "拖拽不触发打开").toBeUndefined();
});
