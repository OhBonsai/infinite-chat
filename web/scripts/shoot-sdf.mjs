// Plan 28 闭环工具:起我方 dev server 页(需已运行 :5173),截 SDF 侧帧供与参考并排。
// 用法:node web/scripts/shoot-sdf.mjs <url-path> <out.png> [wait-ms]
import { chromium } from "@playwright/test";

const [, , path = "/?replay=showcase&noinput", out = "sdf.png", waitMs = "9000"] = process.argv;
const browser = await chromium.launch({
  channel: "chromium",
  args: [
    "--enable-unsafe-webgpu",
    "--enable-features=Vulkan",
    "--enable-webgpu-developer-features",
    "--ignore-gpu-blocklist",
    "--use-angle=default",
  ],
});
const page = await browser.newPage({ viewport: { width: 1280, height: 900 }, deviceScaleFactor: Number(process.env.SHOT_DPR ?? 1) });
await page.goto(`http://localhost:5173${path}`, { waitUntil: "domcontentloaded" });
await page.waitForFunction(() => !!window.__chat, null, { timeout: 60000 });
await page.evaluate(() => {
  window.__chat.set_stream_rate(1e9);
  window.__chat.set_reveal_cps(1e9);
});
await page.waitForTimeout(Number(waitMs));
// settled:1e9 cps 下等自然收敛(不用 seek —— seek+pause 曾产重影假象,见 plan28 progress)。
await page.evaluate(() => window.__chat.scroll_to(0));
await page.waitForTimeout(1500);
await page.screenshot({ path: out });
console.log("→", out);
await browser.close();
