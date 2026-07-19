// flavor.spec.ts — Plan 45 U4:观感 flavor(rich ⇄ tui)e2e。?tui 生效 + 运行时切换重渲(DoD-7 e2e ≥2)。
import { test, expect } from "@playwright/test";
import { assertWebGpu } from "./helpers";

type FlavorWin = { __chat: { render_flavor(): string; visible_turns(): string } };
const flavor = (page: import("@playwright/test").Page) =>
  page.evaluate(() => (window as unknown as FlavorWin).__chat.render_flavor());

test("flavor:?tui 生效(引擎 render_flavor=tui + 面板档位切 off/typewriter)", async ({ page }) => {
  const errors: string[] = [];
  page.on("pageerror", (e) => errors.push(e.message));
  await page.goto("/chat/?script=showcase-full&at=30000&tui", { waitUntil: "domcontentloaded" });
  await assertWebGpu(page);
  await page.waitForFunction(() => !!(window as unknown as { __chat?: unknown }).__chat, null, { timeout: 60_000 });
  // 引擎观感切到 tui(applyFlavor 生效)。
  await expect.poll(() => flavor(page), { timeout: 15_000, message: "?tui 应把引擎切到 tui" }).toBe("tui");
  // 动效档:tui 默认 effect off + rhythm typewriter(面板下拉如实)。
  await expect(page.locator("#flavor-sel")).toHaveValue("tui");
  await expect(page.locator("#effect-sel")).toHaveValue("off");
  await expect(page.locator("#rhythm-sel")).toHaveValue("typewriter");
  expect(errors, `flavor 页错误: ${errors.join("; ")}`).toHaveLength(0);
});

// Plan 46:tui flavor 逐字位置进 Rust cell 网格 —— 强制重排(refresh_fonts)后,JS `layout` 回调不涨
// (Rust celllayout 接管);rich 同触发则 JS `layout` 涨。measureText 仅开机校准一次,稳态零调用。
type LayoutWin = { __chat: { refresh_fonts(): void; visible_turns(): string }; __layoutCounts(): { layout: number; measure: number } };
const relayoutDelta = async (page: import("@playwright/test").Page): Promise<number> => {
  const before = await page.evaluate(() => (window as unknown as LayoutWin).__layoutCounts().layout);
  await page.evaluate(() => (window as unknown as LayoutWin).__chat.refresh_fonts());
  await page.waitForTimeout(600); // 重排若干帧
  const after = await page.evaluate(() => (window as unknown as LayoutWin).__layoutCounts().layout);
  return after - before;
};

test("cell:tui 逐字位置进 Rust(强制重排 JS layout 回调 delta==0;rich 对照 > 0)", async ({ page }) => {
  await page.goto("/chat/?script=showcase-full&at=30000&tui", { waitUntil: "domcontentloaded" });
  await assertWebGpu(page);
  await page.waitForFunction(() => !!(window as unknown as { __chat?: unknown }).__chat, null, { timeout: 60_000 });
  await expect.poll(() => flavor(page), { timeout: 15_000 }).toBe("tui");
  await page.evaluate(() => (window as unknown as { __chat: { set_reveal_cps(n: number): void } }).__chat.set_reveal_cps(1e9));
  await page.waitForTimeout(1500); // settle
  const tuiDelta = await relayoutDelta(page);
  expect(tuiDelta, "tui 重排:JS layout 回调不涨(Rust cell 布局接管)").toBe(0);
  // 内容不丢(cell 布局出了块)。
  const turns = await page.evaluate(() => JSON.parse((window as unknown as LayoutWin).__chat.visible_turns()).length);
  expect(turns, "cell 布局后仍有可见块").toBeGreaterThan(0);
});

test("cell:rich 对照 —— 逐字实测走 JS(强制重排 JS layout 回调 > 0)", async ({ page }) => {
  await page.goto("/chat/?script=showcase-full&at=30000", { waitUntil: "domcontentloaded" });
  await assertWebGpu(page);
  await page.waitForFunction(() => !!(window as unknown as { __chat?: unknown }).__chat, null, { timeout: 60_000 });
  await page.evaluate(() => (window as unknown as { __chat: { set_reveal_cps(n: number): void } }).__chat.set_reveal_cps(1e9));
  await page.waitForTimeout(1500);
  const richDelta = await relayoutDelta(page);
  expect(richDelta, "rich 重排:JS layout 回调涨(逐字实测在 JS)").toBeGreaterThan(0);
});

test("flavor:运行时切换重渲(tui → rich 同内容换皮,引擎 flavor 跟随)", async ({ page }) => {
  await page.goto("/chat/?script=showcase-full&at=30000&tui", { waitUntil: "domcontentloaded" });
  await assertWebGpu(page);
  await page.waitForFunction(() => !!(window as unknown as { __chat?: unknown }).__chat, null, { timeout: 60_000 });
  await expect.poll(() => flavor(page), { timeout: 15_000 }).toBe("tui");
  // 运行时切回 rich(面板下拉)→ 引擎 flavor 跟随,内容仍在(重渲不塌)。
  await page.selectOption("#flavor-sel", "rich");
  await expect.poll(() => flavor(page), { timeout: 10_000, message: "切 rich 后引擎应回 rich" }).toBe("rich");
  await expect(page.locator("#effect-sel")).toHaveValue("subtle");
  // 内容未丢(切换 = 缓存失效重渲,非清空)。
  const turns = await page.evaluate(() => JSON.parse((window as unknown as FlavorWin).__chat.visible_turns()).length);
  expect(turns, "重渲后仍有可见块").toBeGreaterThan(0);
});
