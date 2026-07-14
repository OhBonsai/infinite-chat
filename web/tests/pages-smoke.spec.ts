// pages-smoke.spec.ts — Plan 40 P6:四页官网收口烟测(进 e2e 门)。
// 断言每页「能起、能渲、结构在」—— landing/markdown 引擎经 window.__hero 出图(bootHero 暴露);
// chat 完整谱由 chat-route.spec 覆盖,这里只查 chrome;gallery 是静态生成页(无引擎)。
import { test, expect } from "@playwright/test";
import { assertWebGpu } from "./helpers";

// bootHero 挂的调试/烟测句柄(landing/markdown 用);只取渲染读接口。
const heroTurns = (page: import("@playwright/test").Page) =>
  page.evaluate(() => (window as unknown as { __hero: { visible_turns(): string } }).__hero.visible_turns());

test("landing:hero 引擎出图 + 导航 + 九大功能分区", async ({ page }) => {
  const errors: string[] = [];
  page.on("pageerror", (e) => errors.push(e.message));

  await page.goto("/", { waitUntil: "domcontentloaded" });
  await assertWebGpu(page);
  // hero 引擎就绪并喂入导览流 → 有可见回合(就绪门 + 两步事件序列真渲)。
  await page.waitForFunction(
    () => !!(window as unknown as { __hero?: unknown }).__hero,
    null,
    { timeout: 60_000 },
  );
  await expect
    .poll(async () => JSON.parse(await heroTurns(page)).length, { timeout: 60_000, message: "hero 无可见回合" })
    .toBeGreaterThan(0);

  // 统一导航:四页 + GitHub(nav 由 pages-nav 注入,当前页金线态)。
  const navLinks = await page.locator(".p-nav a").count();
  expect(navLinks, "nav 链接数(四页 + GitHub)").toBeGreaterThanOrEqual(5);
  await expect(page.locator('.p-nav a[aria-current="page"]')).toHaveCount(1);

  // 功能全览:九大项锚点分区(pages-assets/index.json → 动态渲染)。
  await expect
    .poll(() => page.locator(".feat-sec").count(), { timeout: 30_000, message: "功能分区未渲染" })
    .toBe(9);
  // 至少若干精选卡 + 媒体(截图/短片)入位。
  expect(await page.locator(".feat-card").count(), "精选卡数").toBeGreaterThanOrEqual(9);
  expect(await page.locator(".feat-media").count(), "卡片媒体数").toBeGreaterThanOrEqual(9);

  expect(errors, `页面错误: ${errors.join("; ")}`).toHaveLength(0);
});

test("markdown:引擎轮播出图 + playground 结构", async ({ page }) => {
  const errors: string[] = [];
  page.on("pageerror", (e) => errors.push(e.message));

  await page.goto("/markdown/", { waitUntil: "domcontentloaded" });
  await assertWebGpu(page);
  await page.waitForFunction(
    () => !!(window as unknown as { __hero?: unknown }).__hero,
    null,
    { timeout: 60_000 },
  );
  // 全类型轮播:引擎就绪后喂样例 → 有可见回合。
  await expect
    .poll(async () => JSON.parse(await heroTurns(page)).length, { timeout: 60_000, message: "markdown 无可见回合" })
    .toBeGreaterThan(0);

  // playground 结构:输入框 + 播放按钮在位(可玩)。
  await expect(page.locator("#md-input")).toHaveCount(1);
  await expect(page.locator("#md-run")).toHaveCount(1);
  // 点播放不炸(合成流喂引擎;内容语义由 markdown.ts 单测面覆盖,这里只保不抛)。
  await page.click("#md-run");
  await page.waitForTimeout(500);

  expect(errors, `页面错误: ${errors.join("; ")}`).toHaveLength(0);
});

test("gallery:静态页 + 统一导航条", async ({ page }) => {
  await page.goto("/gallery.html", { waitUntil: "domcontentloaded" });
  // 生成页内联的导航条(gen-gallery 注入):四页 + GitHub 链接可见。
  const navLinks = page.locator('a[href*="chat"], a[href*="markdown"], a[href*="github"]');
  expect(await navLinks.count(), "gallery 导航链接").toBeGreaterThanOrEqual(2);
});
