// pages-smoke.spec.ts — Plan 40 P6:四页官网收口烟测(进 e2e 门)。
// 断言每页「能起、能渲、结构在」—— landing/markdown 引擎经 window.__hero 出图(bootHero 暴露);
// chat 完整谱由 chat-route.spec 覆盖,这里只查 chrome;gallery 是静态生成页(无引擎)。
import { test, expect } from "@playwright/test";
import { assertWebGpu } from "./helpers";

// bootHero 挂的调试/烟测句柄(landing/markdown 用);只取渲染读接口。
type HeroWin = { __hero: { visible_turns(): string } };
const heroTurns = (page: import("@playwright/test").Page) =>
  page.evaluate(() => (window as unknown as HeroWin).__hero.visible_turns());
// 当前框取顶部可见文本(幂等对拍用:同一幕 enter 应产同一框取,与到达路径无关)。
const frameTop = (page: import("@playwright/test").Page) =>
  page.evaluate(() => {
    const ts = (JSON.parse((window as unknown as HeroWin).__hero.visible_turns()) as { y: number; text?: string }[])
      .filter((x) => x.y >= -5)
      .sort((a, b) => a.y - b.y);
    return (ts[0]?.text ?? "").replace(/\s+/g, " ").slice(0, 24);
  });
const curDot = (page: import("@playwright/test").Page) =>
  page.evaluate(() => [...document.querySelectorAll("#home-chrome .dot")].findIndex((d) => d.classList.contains("on")));

test("landing:幕式播放器(单实例真渲染 + 自动播 + 键盘 + 直跳幂等 + S7 深链)", async ({ page }) => {
  const errors: string[] = [];
  page.on("pageerror", (e) => errors.push(e.message));

  await page.goto("/", { waitUntil: "domcontentloaded" });
  await assertWebGpu(page);
  await page.waitForFunction(() => !!(window as unknown as { __hero?: unknown }).__hero, null, { timeout: 60_000 });
  await expect
    .poll(async () => JSON.parse(await heroTurns(page)).length, { timeout: 60_000, message: "母文档无可见回合" })
    .toBeGreaterThan(0);

  // 单引擎实例(单 canvas 多幕铁律):唯一引擎 canvas #home-canvas(#home-bg 是纯装饰 WebGL 流光,非引擎)。
  expect(await page.locator("#home-canvas").count(), "单引擎 canvas").toBe(1);
  // chrome:七幕点 + S1 字幕 + 极简 nav。
  await expect.poll(() => page.locator("#home-chrome .dot").count(), { timeout: 30_000 }).toBe(8);
  await expect(page.locator("#home-subtitle .sub-title")).toContainText("INFINITE CHAT");
  expect(await page.locator("#home-nav a").count()).toBeGreaterThanOrEqual(2);

  // 自动播:不介入,过 S1 时长(5s)后当前点前移。
  await expect
    .poll(() => curDot(page), { timeout: 12_000, message: "未自动推进" })
    .toBeGreaterThan(0);

  // 键盘接管:← 回 S1,→ 进 S2。
  await page.keyboard.press("ArrowLeft");
  await expect.poll(() => curDot(page)).toBe(0);
  await page.keyboard.press("ArrowRight");
  await expect.poll(() => curDot(page)).toBe(1);

  // 直跳幂等(plan17 契约):直跳第 3 幕的框取,与「先跳第 6 幕再跳第 3 幕」一致。
  await page.locator("#home-chrome .dot").nth(2).click();
  await expect.poll(() => curDot(page)).toBe(2);
  await page.waitForTimeout(700);
  const direct = await frameTop(page);
  await page.locator("#home-chrome .dot").nth(5).click();
  await page.waitForTimeout(500);
  await page.locator("#home-chrome .dot").nth(2).click();
  await page.waitForTimeout(700);
  const viaOther = await frameTop(page);
  expect(viaOther, "直跳幂等:同幕框取与路径无关").toBe(direct);

  // S7 直跳:入口浮层显、字幕隐、四链接深链(base 前缀)。
  await page.locator("#home-chrome .dot").last().click();
  await expect(page.locator("#home-outro")).toHaveClass(/show/);
  await expect(page.locator("#home-subtitle")).not.toHaveClass(/show/);
  expect(await page.locator("#home-outro .outro-card").count()).toBe(4);
  await expect(page.locator("#o-chat")).toHaveAttribute("href", /chat\/$/);
  await expect(page.locator("#o-md")).toHaveAttribute("href", /markdown\/$/);

  expect(errors, `页面错误: ${errors.join("; ")}`).toHaveLength(0);
});

test("landing 降级:?slides 无 GPU 幻灯(同 chrome / 幕数 / 字幕,canvas 隐)", async ({ page }) => {
  await page.goto("/?slides", { waitUntil: "domcontentloaded" });
  // 幻灯容器显、canvas 隐、每幕一张 slide、七幕点、S1 字幕。
  await expect.poll(() => page.locator("#home-fallback .fb-slide").count(), { timeout: 20_000 }).toBeGreaterThan(0);
  await expect(page.locator("#home-fallback")).toBeVisible();
  await expect(page.locator("#home-canvas")).toBeHidden();
  expect(await page.locator("#home-chrome .dot").count()).toBe(8);
  await expect(page.locator("#home-subtitle .sub-title")).toContainText("INFINITE CHAT");
  // 点第 3 幕 → 该幕 slide 激活(幻灯也走同一 chrome/字幕)。
  await page.locator("#home-chrome .dot").nth(2).click();
  await expect.poll(() => page.locator("#home-fallback .fb-slide.on").count()).toBe(1);
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
