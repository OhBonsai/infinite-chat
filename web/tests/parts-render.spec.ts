// parts-render.spec.ts — Plan 24 PR-2(TC-F05/F08/F09/F10):各 part 经 push_event 合成 → 兜底渲染。
// 不依赖真 server:wasm `push_event` 直接喂 part.updated 驱动 store→渲染。
import { test, expect, type Page } from "@playwright/test";
import { assertWebGpu } from "./helpers";

const partUpdated = (p: unknown) =>
  JSON.stringify({ type: "message.part.updated", properties: { part: p, time: 1 } });

// 注入一个 part(独立 message → 独立 view),滚到它,返回该 view 的可见文本。
async function pushAndRead(page: Page, view: number, p: unknown): Promise<string> {
  await page.evaluate((raw) => window.__chat.push_event(raw), partUpdated(p));
  await page.waitForTimeout(120);
  await page.evaluate((v) => window.__chat.scroll_to(v), view);
  await page.waitForTimeout(120);
  return page.evaluate(() => window.__chat.visible_text_runs());
}

test.beforeEach(async ({ page }) => {
  await page.goto("/?replay=showcase&noinput", { waitUntil: "domcontentloaded" });
  await assertWebGpu(page);
  await page.waitForFunction(() => !!(window as unknown as { __chat?: unknown }).__chat, null, {
    timeout: 60_000,
  });
  await page.evaluate(() => {
    window.__chat.set_stream_rate(1e9);
    window.__chat.set_reveal_cps(1e9);
  });
});

test("TC-F05 tool 卡状态变迁(pending→done,徽章随 status,旧态不残留)", async ({ page }) => {
  // Plan 23 tool 卡:`▸ <name>  [badge]`,badge = [pending]/[running]/[done]/[error]。
  const tool = (status: string) =>
    partUpdated({
      type: "tool",
      id: "t1",
      messageID: "mtool",
      tool: "bash",
      state: { status, input: { cmd: "ls" } },
    });
  await page.evaluate((raw) => window.__chat.push_event(raw), tool("pending"));
  await page.waitForTimeout(120);
  await page.evaluate((v) => window.__chat.scroll_to(v), 1);
  await expect
    .poll(() => page.evaluate(() => window.__chat.visible_text_runs()))
    .toContain("[pending]");

  await page.evaluate((raw) => window.__chat.push_event(raw), tool("completed"));
  await page.waitForTimeout(150);
  const runs = await page.evaluate(() => window.__chat.visible_text_runs());
  expect(runs, "工具名 + 完成徽章可见").toContain("▸ bash");
  expect(runs, "完成态徽章 [done]").toContain("[done]");
  expect(runs, "旧态徽章不残留(重写重渲)").not.toContain("[pending]");
});

test("TC-F06 diff 块(tool edit metadata.filediff)", async ({ page }) => {
  const runs = await pushAndRead(page, 1, {
    type: "tool",
    id: "ted",
    messageID: "med",
    tool: "edit",
    state: {
      status: "completed",
      metadata: { filediff: "@@ -1,2 +1,2 @@\n ctx\n-oldline\n+newline\n" },
    },
  });
  expect(runs, "edit 工具卡").toContain("▸ edit");
  expect(runs, "新增行可见").toContain("newline");
  expect(runs, "删除行可见").toContain("oldline");
});

test("TC-F08 compaction 分隔线 + 标签", async ({ page }) => {
  const runs = await pushAndRead(page, 1, { type: "compaction", id: "c1", messageID: "mc" });
  expect(runs, "压缩标签可见").toContain("上下文已压缩");
});

test("TC-F10 tool error 卡(state=error)", async ({ page }) => {
  const runs = await pushAndRead(page, 1, {
    type: "tool",
    id: "te",
    messageID: "mte",
    tool: "bash",
    state: { status: "error", error: "boom" },
  });
  expect(runs, "工具名可见").toContain("▸ bash");
  expect(runs, "错误徽章可见").toContain("[error]");
  expect(runs, "错误正文可见").toContain("boom");
});

test("TC-F09 畸形/未知 part 不崩(兜底)", async ({ page }) => {
  // 未知 part 类型 → Other(噪音,不承载);畸形 part → 解码健壮。关键:不崩、应用仍可用。
  await page.evaluate(() => {
    window.__chat.push_event(
      JSON.stringify({ type: "message.part.updated", properties: { part: { type: "totally-unknown", id: "u1" }, time: 1 } }),
    );
    window.__chat.push_event("{ not valid json");
    window.__chat.push_event(JSON.stringify({ type: "message.part.updated", properties: {} }));
  });
  await page.waitForTimeout(150);
  // 仍能取状态 + 已有内容仍在(没崩)。
  expect(await page.evaluate(() => window.__chat.session_status())).toBeTruthy();
  const runs = await page.evaluate(() => window.__chat.visible_text_runs());
  expect(runs.length, "应用仍渲染(未崩)").toBeGreaterThan(2);
});
