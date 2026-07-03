// a11y.spec.ts — Plan 26②(0030 步骤3)E2E:ARIA 镜像 / live region 播报 / Dock 焦点管理。
import { test, expect } from "@playwright/test";
import { assertWebGpu, bootVisible } from "./helpers";

test("ARIA 镜像:log 容器 + article 块(角色描述/posinset/setsize)", async ({ page }) => {
  await bootVisible(page);
  // 容器语义。
  const layer = page.locator(".text-layer");
  await expect(layer).toHaveAttribute("role", "log");
  await expect(layer).toHaveAttribute("aria-label", "对话");
  // 块级 article(虚拟化:仅可见块;posinset 保序)。
  const articles = page.locator('.text-layer [role="article"]');
  expect(await articles.count()).toBeGreaterThan(0);
  const first = articles.first();
  await expect(first).toHaveAttribute("aria-roledescription", /消息/);
  const pos = Number(await first.getAttribute("aria-posinset"));
  const size = Number(await first.getAttribute("aria-setsize"));
  expect(pos).toBeGreaterThan(0);
  expect(size).toBeGreaterThanOrEqual(pos);
  // landmark:画布 main。
  await expect(page.locator('canvas[role="main"]')).toBeAttached();
});

test("live region:状态迁移按粒度播报(不逐 delta);阻塞态 assertive", async ({ page }) => {
  await bootVisible(page); // showcase 耗尽 → 引擎安静
  const region = page.locator(".sr-announcer");
  await expect(region).toBeAttached();

  // 发送 → 播"等待回复";首包 → "正在回复"。
  await page.evaluate(() => {
    window.__chat.note_send();
  });
  await expect.poll(() => region.textContent(), { timeout: 5_000 }).toContain("等待回复");
  await page.evaluate(() =>
    window.__chat.push_event(
      JSON.stringify({
        type: "message.part.delta",
        properties: { messageID: "ma", partID: "pa", field: "text", delta: "hi" },
      }),
    ),
  );
  await expect.poll(() => region.textContent(), { timeout: 5_000 }).toContain("正在回复");

  // 权限请求 → assertive + 文案;idle 收尾 → "回复完成"。
  await page.evaluate(() =>
    window.__chat.push_event(JSON.stringify({ type: "permission.asked", properties: { sessionID: "s" } })),
  );
  await expect.poll(() => region.textContent(), { timeout: 5_000 }).toContain("权限");
  await expect(region).toHaveAttribute("aria-live", "assertive");
  await page.evaluate(() => {
    window.__chat.reply_permission(true);
    window.__chat.push_event(
      JSON.stringify({ type: "session.status", properties: { status: { type: "idle" } } }),
    );
  });
  await expect.poll(() => region.textContent(), { timeout: 5_000 }).toContain("回复完成");
  await expect(region).toHaveAttribute("aria-live", "polite");
});

test("ask 焦点与影子 a11y:question 表单焦点入/还原;permission 影子真按钮可达", async ({ page }) => {
  await page.goto("/?replay=showcase&noinput", { waitUntil: "domcontentloaded" });
  await assertWebGpu(page);
  await page.waitForFunction(() => !!(window as unknown as { __chat?: unknown }).__chat, null, {
    timeout: 60_000,
  });
  // 先给一个已知焦点(查找条输入框)。
  await page.keyboard.press("Control+F");
  await expect(page.locator(".find-input")).toBeFocused();

  // question → 流内表单弹出,焦点入表单首控件。
  await page.evaluate(() =>
    window.__chat.push_event(
      JSON.stringify({
        type: "question.asked",
        properties: { sessionID: "s", question: "选一个?", options: ["A", "B"] },
      }),
    ),
  );
  const form = page.locator(".ask-form");
  await form.waitFor({ state: "visible", timeout: 10_000 });
  await expect(page.locator(".ask-form input").first()).toBeFocused();

  // 提交 → 表单撤 + 焦点还原到查找输入框(26② 语义,Plan 27 迁移)。
  await page.locator(".ask-submit").click();
  await form.waitFor({ state: "detached", timeout: 10_000 });
  await expect(page.locator(".find-input")).toBeFocused();

  // permission → 影子 a11y 真按钮出现(SDF 按钮读屏不可见的镜像);点击可应答。
  await page.evaluate(() =>
    window.__chat.push_event(
      JSON.stringify({ type: "permission.asked", properties: { sessionID: "s", title: "允许?" } }),
    ),
  );
  const shadow = page.locator(".ask-shadow-allow");
  await shadow.waitFor({ state: "attached", timeout: 10_000 });
  // sr-only(1px clip)元素 Playwright 可视性检查不过 → dispatchEvent(读屏激活本质也是派发 click)。
  await shadow.dispatchEvent("click");
  await expect
    .poll(() => page.evaluate(() => window.__chat.session_status()))
    .not.toBe("blocked:permission");
  // 落定后影子按钮撤(下一帧泵)。
  await page.locator(".ask-shadow").waitFor({ state: "detached", timeout: 10_000 });
});
