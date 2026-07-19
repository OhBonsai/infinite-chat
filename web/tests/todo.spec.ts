// todo.spec.ts — Plan 47 P3:TUI TodoWrite 组件 e2e。?script=todo 清单卡逐项 + running→completed 不炸。
import { test, expect } from "@playwright/test";
import { assertWebGpu } from "./helpers";

type Win = {
  __chat: { visible_text_runs(): string; visible_turns(): string; render_flavor(): string };
};
const ready = (page: import("@playwright/test").Page) =>
  page.waitForFunction(() => !!(window as unknown as { __chat?: unknown }).__chat, null, { timeout: 60_000 });

test("todo:tui 清单卡逐项渲染(行数==todos + Todos·X/N + 方括号符号)", async ({ page }) => {
  await page.goto("/chat/?script=todo&tui&at=30000", { waitUntil: "domcontentloaded" });
  await assertWebGpu(page);
  await ready(page);
  await expect.poll(() => page.evaluate(() => (window as unknown as Win).__chat.render_flavor()), { timeout: 15_000 }).toBe("tui");
  await page.waitForTimeout(1500); // settle 到末态清单
  const runs = JSON.parse(await page.evaluate(() => (window as unknown as Win).__chat.visible_text_runs()));
  const all = (runs as { text: string }[]).map((r) => r.text).join("\n");
  // 卡 header:Todos · X/N done。
  expect(all, `应有 Todos·X/N 表头: ${all.slice(0, 200)}`).toMatch(/Todos\s*·\s*\d+\/\d+\s*done/);
  // 末态 5 条 todo content 全在(mock todo.json 末次 todowrite)。
  for (const c of ["复现高并发下的超时", "定位重试是否无退避", "改成指数退避", "补压测覆盖登录路径", "更新超时阈值文档"]) {
    expect(all, `缺 todo: ${c}`).toContain(c);
  }
  // 逐 todo 方括号符号(≥5),且不露原始 JSON。
  const marks = (all.match(/\[[ ~✓✗]\]/g) ?? []).length;
  expect(marks, `逐 todo 方括号符号 ≥5,实 ${marks}`).toBeGreaterThanOrEqual(5);
  expect(all, "清单卡不应露原始 JSON").not.toContain('"todos"');
});

test("todo:running→completed 两态替换不炸(全程零 pageerror + 内容不丢)", async ({ page }) => {
  const errors: string[] = [];
  page.on("pageerror", (e) => errors.push(e.message));
  await page.goto("/chat/?script=todo&tui&at=30000", { waitUntil: "domcontentloaded" });
  await assertWebGpu(page);
  await ready(page);
  await page.waitForTimeout(2000); // 走完 running→completed 替换全程
  expect(errors, `回放报错: ${errors.join("; ")}`).toHaveLength(0);
  const turns = JSON.parse(await page.evaluate(() => (window as unknown as Win).__chat.visible_turns()));
  expect((turns as unknown[]).length, "两态替换后仍有可见块").toBeGreaterThan(0);
});
