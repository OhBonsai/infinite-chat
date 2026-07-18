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
