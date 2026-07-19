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

// Plan 46:tui flavor 逐字位置进 Rust cell 网格 —— 强制重排(refresh_fonts)后,**文本块** JS `layout`
// 回调不涨(Rust celllayout 接管);唯**结构表格块**仍走 JS 表引擎(placeTable 列宽/网格在桥侧,收敛轮
// 决策)→ tui 重排 delta 远小于 rich(rich 逐字全走 JS)。measureText 仅开机校准一次,稳态近零。
type LayoutWin = { __chat: { refresh_fonts(): void; visible_turns(): string; set_reveal_cps(n: number): void }; __layoutCounts(): { layout: number; measure: number } };
const settle = async (page: import("@playwright/test").Page) => {
  await page.evaluate(() => (window as unknown as LayoutWin).__chat.set_reveal_cps(1e9));
  await page.waitForTimeout(1500);
};
const relayoutDelta = async (page: import("@playwright/test").Page): Promise<number> => {
  const before = await page.evaluate(() => (window as unknown as LayoutWin).__layoutCounts().layout);
  await page.evaluate(() => (window as unknown as LayoutWin).__chat.refresh_fonts());
  await page.waitForTimeout(600); // 重排若干帧
  const after = await page.evaluate(() => (window as unknown as LayoutWin).__layoutCounts().layout);
  return after - before;
};

test("cell:tui 文本逐字进 Rust(重排 JS layout 远小于 rich;仅表格块留 JS)", async ({ page }) => {
  await page.goto("/chat/?script=showcase-full&at=30000&tui", { waitUntil: "domcontentloaded" });
  await assertWebGpu(page);
  await page.waitForFunction(() => !!(window as unknown as { __chat?: unknown }).__chat, null, { timeout: 60_000 });
  await expect.poll(() => flavor(page), { timeout: 15_000 }).toBe("tui");
  await settle(page);
  const tuiDelta = await relayoutDelta(page);
  // 切回 rich 同内容对照(切换本身的重排在 settle 里消化,不进 delta 窗口)。
  await page.selectOption("#flavor-sel", "rich");
  await expect.poll(() => flavor(page), { timeout: 10_000 }).toBe("rich");
  await settle(page);
  const richDelta = await relayoutDelta(page);
  // 核心证明:tui 文本层进 Rust → 重排时 JS layout 回调远少于 rich(rich 逐字全走 JS)。
  expect(tuiDelta, `tui 重排 JS layout(${tuiDelta})应远小于 rich(${richDelta})`).toBeLessThan(richDelta);
  // 稳态近零:tui 仅结构表格/tile 块打 JS(showcase-full 1 表)→ 个位数,不随正文规模涨。
  expect(tuiDelta, "tui 重排仅结构块打 JS(个位数)").toBeLessThanOrEqual(3);
  expect(richDelta, "rich 逐字实测在 JS(涨)").toBeGreaterThan(0);
  const turns = await page.evaluate(() => JSON.parse((window as unknown as LayoutWin).__chat.visible_turns()).length);
  expect(turns, "cell 布局后仍有可见块").toBeGreaterThan(0);
});

// Plan 46 L5a:cell 网格「渲染同源 + 双宽」硬门 —— tui 下 cell_w 必须 = LXGW MSDF 拉丁 baked advance
// (量宽==渲染源,非 measureText),且 CJK advance = 2×cell_w(双宽 → 网格严丝合缝,无过宽字间距)。
test("cell:tui 校准渲染同源 + 双宽(cell_w = LXGW 拉丁 advance;CJK = 2×)", async ({ page }) => {
  await page.goto("/chat/?script=showcase-full&at=16000&tui", { waitUntil: "domcontentloaded" });
  await assertWebGpu(page);
  await page.waitForFunction(() => !!(window as unknown as { __chat?: unknown }).__chat, null, { timeout: 60_000 });
  await expect.poll(() => flavor(page), { timeout: 15_000 }).toBe("tui");
  // applyFlavor(tui) 会 await loadMsdf → 轮询到 MSDF 就位。
  await expect
    .poll(async () => page.evaluate(async () => (await import("/src/msdf.ts")).msdfLoaded()), { timeout: 20_000 })
    .toBe(true);
  const m = await page.evaluate(async () => {
    const msdf = await import("/src/msdf.ts");
    const lb = await import("/src/layout-bridge.ts");
    const FS = lb.FONT_SIZE;
    return {
      cellW: lb.calibrateCellMetrics().cellW,
      latin: msdf.msdfAdvancePx("M".codePointAt(0)!, FS),
      cjk: msdf.msdfAdvancePx("中".codePointAt(0)!, FS),
    };
  });
  expect(m.cellW, "cell_w = LXGW 拉丁 baked advance(渲染同源,非 measureText)").toBe(m.latin);
  expect(m.cjk, "CJK advance = 2×cell_w(双宽 → 网格对齐,无过宽字间距)").toBe(2 * m.cellW!);
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
  // 内容未丢(切换 = 缓存失效重渲,非清空)。poll:重渲后帧循环需追上(机器高负载下略慢,
  // 内容本就在 store,只等投影上屏),故轮询而非单次读,避免 CPU 饱和时假阴。
  await expect
    .poll(() => page.evaluate(() => JSON.parse((window as unknown as FlavorWin).__chat.visible_turns()).length), {
      timeout: 10_000,
      message: "重渲后仍有可见块",
    })
    .toBeGreaterThan(0);
});
