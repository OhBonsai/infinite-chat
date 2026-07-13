// embed-overlay.spec.ts — Plan 31 R3(0022):DOM overlay 与 GPU 世界**像素级跟随**。
// 判据(GOAL §3):pan/zoom 后 overlay 元素盒与引擎报告的屏幕矩形差 ≤1px(共享相机 world=px)。
import { test, expect, type Page } from "@playwright/test";
import { assertWebGpu } from "./helpers";

// 动画 SVG(SMIL)→ image-loader 嗅探为动图 → FrameEmbed → DOM overlay(canvas 块保持冻结)。
const ANIM_SVG =
  "data:image/svg+xml," +
  encodeURIComponent(
    `<svg xmlns='http://www.w3.org/2000/svg' width='60' height='40'><rect width='60' height='40' fill='tomato'><animate attributeName='opacity' values='0.4;1;0.4' dur='1s' repeatCount='indefinite'/></rect></svg>`,
  );

async function bootWithAnimEmbed(page: Page): Promise<void> {
  await page.goto("/?empty&noinput", { waitUntil: "domcontentloaded" });
  await assertWebGpu(page);
  await page.waitForFunction(() => !!(window as unknown as { __chat?: unknown }).__chat, null, {
    timeout: 60_000,
  });
  await page.evaluate((url) => {
    window.__chat.set_stream_rate(1e9);
    window.__chat.set_reveal_cps(1e9);
    window.__chat.push_event(
      JSON.stringify({
        type: "message.part.updated",
        properties: {
          part: {
            type: "text",
            id: "p-e",
            messageID: "m-e",
            text: `动图嵌入 ![anim](${url}) 尾部`,
          },
          time: 1,
        },
      }),
    );
  }, ANIM_SVG);
  await page.locator(".embed-overlay-probe, img[src^='data:image/svg']").first().waitFor({
    state: "attached",
    timeout: 20_000,
  });
}

/** overlay img 实际盒 vs 引擎屏幕矩形(设备 px → CSS px;相对 canvas 左上)。 */
async function overlayError(page: Page): Promise<{ dx: number; dy: number; dw: number; dh: number }> {
  return page.evaluate(() => {
    const dpr = window.devicePixelRatio || 1;
    const payload = JSON.parse(window.__chat.frame_embeds()) as {
      cam: { px: number; py: number; zoom: number };
      embeds: { wx: number; wy: number; ww: number; wh: number }[];
    };
    const e = payload.embeds[0];
    const { px, py, zoom } = payload.cam;
    const want = {
      x: ((e.wx - px) * zoom) / dpr,
      y: ((e.wy - py) * zoom) / dpr,
      w: (e.ww * zoom) / dpr,
      h: (e.wh * zoom) / dpr,
    };
    const canvas = document.getElementById("chat")!.getBoundingClientRect();
    const img = document.querySelector("img[src^='data:image/svg']")!.getBoundingClientRect();
    return {
      dx: Math.abs(img.left - canvas.left - want.x),
      dy: Math.abs(img.top - canvas.top - want.y),
      dw: Math.abs(img.width - want.w),
      dh: Math.abs(img.height - want.h),
    };
  });
}

test("R3 pan/zoom 中 overlay 与 GPU 锚点像素差 ≤1px", async ({ page }) => {
  await bootWithAnimEmbed(page);
  await page.waitForTimeout(500); // 首帧补间到位(120ms)+ 稳定
  const settle = () => page.waitForTimeout(250); // 补间只作用于布局变化;相机为直乘,一帧即随

  let err = await overlayError(page);
  expect(err.dx, `初始 dx=${err.dx}`).toBeLessThanOrEqual(1);
  expect(err.dy).toBeLessThanOrEqual(1);

  await page.evaluate(() => window.__chat.pan_by(0, 137));
  await settle();
  err = await overlayError(page);
  expect(err.dx, `pan 后 dx=${err.dx}`).toBeLessThanOrEqual(1);
  expect(err.dy, `pan 后 dy=${err.dy}`).toBeLessThanOrEqual(1);

  await page.evaluate(() => window.__chat.zoom_at(1.35, 200, 200));
  await settle();
  err = await overlayError(page);
  expect(err.dx, `zoom 后 dx=${err.dx}`).toBeLessThanOrEqual(1);
  expect(err.dy, `zoom 后 dy=${err.dy}`).toBeLessThanOrEqual(1);
  expect(err.dw, `zoom 后 dw=${err.dw}`).toBeLessThanOrEqual(1.5);
  expect(err.dh).toBeLessThanOrEqual(1.5);
});
