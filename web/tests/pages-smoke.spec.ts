// pages-smoke.spec.ts — Plan 40 P6:四页官网收口烟测(进 e2e 门)。
// 断言每页「能起、能渲、结构在」—— landing/markdown 引擎经 window.__hero 出图(bootHero 暴露);
// chat 完整谱由 chat-route.spec 覆盖,这里只查 chrome;gallery 是静态生成页(无引擎)。
import { test, expect } from "@playwright/test";
import { assertWebGpu } from "./helpers";

// bootHero 挂的调试/烟测句柄(landing/markdown 用);只取渲染读接口。
const heroTurns = (page: import("@playwright/test").Page) =>
  page.evaluate(() => (window as unknown as { __hero: { visible_turns(): string } }).__hero.visible_turns());

test("landing:全屏 canvas 幕式播放器(引擎出图 + 幕数 + S1 字幕 + S7 入口 + 直跳)", async ({ page }) => {
  const errors: string[] = [];
  page.on("pageerror", (e) => errors.push(e.message));

  await page.goto("/", { waitUntil: "domcontentloaded" });
  await assertWebGpu(page);
  // 引擎就绪 + 母文档载入 → 有可见回合(单 canvas 真渲染)。
  await page.waitForFunction(
    () => !!(window as unknown as { __hero?: unknown }).__hero,
    null,
    { timeout: 60_000 },
  );
  await expect
    .poll(async () => JSON.parse(await heroTurns(page)).length, { timeout: 60_000, message: "母文档无可见回合" })
    .toBeGreaterThan(0);

  // 播放器 chrome:七幕进度点 + S1 起手当前点 + 大标题字幕。
  await expect.poll(() => page.locator("#home-chrome .dot").count(), { timeout: 30_000 }).toBe(7);
  await expect(page.locator("#home-subtitle")).toHaveClass(/show/);
  await expect(page.locator("#home-subtitle .sub-title")).toContainText("INFINITE CHAT");
  // 极简 nav:品牌 + GitHub。
  expect(await page.locator("#home-nav a").count(), "nav 链接(品牌 + GitHub)").toBeGreaterThanOrEqual(2);

  // 直跳末幕(进度点)→ S7 入口浮层显、字幕隐(幂等直跳的行为面)。
  await page.locator("#home-chrome .dot").last().click();
  await expect(page.locator("#home-outro")).toHaveClass(/show/);
  await expect(page.locator("#home-subtitle")).not.toHaveClass(/show/);
  // 入口四链接就位。
  expect(await page.locator("#home-outro .outro-card").count(), "S7 入口卡").toBe(4);

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
