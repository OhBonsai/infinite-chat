// formation.spec.ts — Plan 42 字阵编队 e2e。像素差证:编队显著改变渲染,散场后近似恒等。
// 不用逐像素 committed golden(GPU 动画片跨机脆);用「同帧双拍差比」证确定性 + 通道生效 + 散场还原。
import { expect, test } from "@playwright/test";

import { assertWebGpu } from "./helpers";

interface FormWin {
  __chat: {
    set_stream_rate(cps: number): void;
    set_reveal_cps(cps: number): void;
    formation_begin(json: string): boolean;
    formation_progress(p: number): void;
    formation_end(): void;
    set_wind(x: number, y: number, radius: number, strength: number): void;
    formation_petals(count: number): void;
  };
}

// dev.html?replay=showcase 的内容 = 顶部一段导览(x240 起,y12+);编队方格也居此域。
const CLIP = { x: 320, y: 10, width: 420, height: 150 };

async function diffRatio(a: Buffer, b: Buffer): Promise<number> {
  const { PNG } = await import("pngjs");
  const pa = PNG.sync.read(a);
  const pb = PNG.sync.read(b);
  let diff = 0;
  const n = Math.min(pa.data.length, pb.data.length);
  for (let i = 0; i < n; i += 4) {
    const d =
      Math.abs(pa.data[i] - pb.data[i]) +
      Math.abs(pa.data[i + 1] - pb.data[i + 1]) +
      Math.abs(pa.data[i + 2] - pb.data[i + 2]);
    if (d > 30) diff++;
  }
  return diff / (pa.width * pa.height);
}

test("formation:编队改变渲染 + 散场还原(VS 通道 + 确定性)", async ({ page }) => {
  const errors: string[] = [];
  page.on("pageerror", (e) => errors.push(e.message));

  await page.goto("/dev.html?replay=showcase&noinput", { waitUntil: "domcontentloaded" });
  await assertWebGpu(page);
  await page.waitForFunction(() => !!(window as unknown as { __chat?: unknown }).__chat, null, { timeout: 60_000 });
  // 满揭示定态(去流式/去 catch-up 动画)→ 编队前后可比。
  await page.evaluate(() => {
    const w = window as unknown as FormWin;
    w.__chat.set_stream_rate(1e9);
    w.__chat.set_reveal_cps(1e9);
  });
  await page.waitForTimeout(1500);

  const base = await page.screenshot({ clip: CLIP });

  // 开始方格编队 + 满进度 → 字飞成阵。
  const ok = await page.evaluate(() =>
    (window as unknown as FormWin).__chat.formation_begin(
      JSON.stringify({ shape: "grid", cols: 20, cell: 16, seed: 1 }),
    ),
  );
  expect(ok, "grid shape 合法 → begin true").toBe(true);
  await page.evaluate(() => (window as unknown as FormWin).__chat.formation_progress(1.0));
  await page.waitForTimeout(500);
  const formed = await page.screenshot({ clip: CLIP });
  const dFormed = await diffRatio(base, formed);
  expect(dFormed, "编队应显著改变渲染").toBeGreaterThan(0.02);

  // 散场:progress 0 + end → 还原(与 base 的差应远小于成形态,证退场恒等收敛)。
  await page.evaluate(() => {
    const w = window as unknown as FormWin;
    w.__chat.formation_progress(0.0);
    w.__chat.formation_end();
  });
  await page.waitForTimeout(500);
  const after = await page.screenshot({ clip: CLIP });
  const dAfter = await diffRatio(base, after);
  expect(dAfter, "散场后应明显靠回 base").toBeLessThan(dFormed * 0.35);

  expect(errors, `页面错误: ${errors.join("; ")}`).toHaveLength(0);
});

test("formation:指针风场推开字 + 松手回弹", async ({ page }) => {
  const errors: string[] = [];
  page.on("pageerror", (e) => errors.push(e.message));
  await page.goto("/dev.html?replay=showcase&noinput", { waitUntil: "domcontentloaded" });
  await assertWebGpu(page);
  await page.waitForFunction(() => !!(window as unknown as { __chat?: unknown }).__chat, null, { timeout: 60_000 });
  await page.evaluate(() => {
    const w = window as unknown as FormWin;
    w.__chat.set_stream_rate(1e9);
    w.__chat.set_reveal_cps(1e9);
  });
  await page.waitForTimeout(1500);
  const base = await page.screenshot({ clip: CLIP });
  // 风场吹在内容域(设备 px ≈ 顶部导览);大半径 + 强推力,稀疏内容也明显位移。
  await page.evaluate(() => (window as unknown as FormWin).__chat.set_wind(460, 70, 340, 260));
  await page.waitForTimeout(300);
  const windy = await page.screenshot({ clip: CLIP });
  const dWindy = await diffRatio(base, windy);
  expect(dWindy, "风场应推开字").toBeGreaterThan(0.012);
  // 松手 → 回弹(闭式偏移归 0):与 base 的残差应远小于成风态。
  await page.evaluate(() => (window as unknown as FormWin).__chat.set_wind(0, 0, 0, 0));
  await page.waitForTimeout(300);
  const calm = await page.screenshot({ clip: CLIP });
  expect(await diffRatio(base, calm), "松手明显回弹").toBeLessThan(dWindy * 0.2);
  expect(errors, `页面错误: ${errors.join("; ")}`).toHaveLength(0);
});

test("formation:花瓣从花心撒下(rect fx petal)", async ({ page }) => {
  await page.goto("/dev.html?replay=showcase&noinput", { waitUntil: "domcontentloaded" });
  await assertWebGpu(page);
  await page.waitForFunction(() => !!(window as unknown as { __chat?: unknown }).__chat, null, { timeout: 60_000 });
  await page.evaluate(() => {
    const w = window as unknown as FormWin;
    w.__chat.set_stream_rate(1e9);
    w.__chat.set_reveal_cps(1e9);
  });
  await page.waitForTimeout(1500);
  await page.evaluate(() => {
    const w = window as unknown as FormWin;
    w.__chat.formation_begin(JSON.stringify({ shape: "rose", a: 120, k: 5 }));
    w.__chat.formation_progress(1.0);
  });
  await page.waitForTimeout(300);
  const noPetals = await page.screenshot({ clip: CLIP });
  await page.evaluate(() => (window as unknown as FormWin).__chat.formation_petals(80));
  await page.waitForTimeout(500);
  const withPetals = await page.screenshot({ clip: CLIP });
  expect(await diffRatio(noPetals, withPetals), "撒花瓣应改变渲染").toBeGreaterThan(0.01);
});

test("formation:首页绽放高潮幕(点 bloom → 字幕 + 编队/花瓣跑不炸)", async ({ page }) => {
  const errors: string[] = [];
  page.on("pageerror", (e) => errors.push(e.message));
  await page.goto("/", { waitUntil: "domcontentloaded" });
  await assertWebGpu(page);
  await page.waitForFunction(() => !!(window as unknown as { __hero?: unknown }).__hero, null, { timeout: 60_000 });
  await page.waitForTimeout(2000);
  // 直跳绽放幕(Plan 43 六幕:title/claim/table/card/bloom/door → bloom = 第 5 点,index 4)。
  await page.locator("#home-chrome .dot").nth(4).click();
  await expect(page.locator("#home-subtitle .sub-title")).toContainText("每个字都是一等图元");
  await page.waitForTimeout(3500); // 聚花 + 撒瓣 + 驱动器推进
  // 离开绽放幕(回 S1)→ 散场收尾不炸。
  await page.locator("#home-chrome .dot").first().click();
  await page.waitForTimeout(600);
  expect(errors, `绽放幕页面错误: ${errors.join("; ")}`).toHaveLength(0);
});

test("formation:坏 shape JSON 整拒(AR12)", async ({ page }) => {
  await page.goto("/dev.html?empty&noinput", { waitUntil: "domcontentloaded" });
  await assertWebGpu(page);
  await page.waitForFunction(() => !!(window as unknown as { __chat?: unknown }).__chat, null, { timeout: 60_000 });
  const bad = await page.evaluate(() => (window as unknown as FormWin).__chat.formation_begin("not json"));
  expect(bad, "坏 JSON → false").toBe(false);
  const bad2 = await page.evaluate(() =>
    (window as unknown as FormWin).__chat.formation_begin(JSON.stringify({ shape: "nope" })),
  );
  expect(bad2, "未知 shape → false").toBe(false);
});
