// ask.spec.ts — Plan 27 §5 E2E:流内问答卡。① question DOM 表单锚定/提交/落定 ② permission
// SDF 按钮真实 click(命中矩形中心)+ hover cursor ③ 落定卡文本可复制(text-layer)。
// (旧底部 Dock 已退役,dock.spec 由本文件替代。)
import { test, expect, type Page } from "@playwright/test";
import { assertWebGpu, bootVisible } from "./helpers";

test.use({ permissions: ["clipboard-read", "clipboard-write"] });

async function boot(page: Page): Promise<void> {
  await bootVisible(page); // showcase 耗尽 → 引擎安静,ask 独占驱动
}

test("① question:流内表单锚定块矩形 → 勾选+输入+确认 → 原位落定 + 解阻", async ({ page }) => {
  await boot(page);
  await page.evaluate(() =>
    window.__chat.push_event(
      JSON.stringify({
        type: "question.asked",
        properties: { sessionID: "s", question: "阈值要改吗?", options: ["保持", "提高"] },
      }),
    ),
  );
  await expect
    .poll(() => page.evaluate(() => window.__chat.session_status()))
    .toBe("blocked:question");
  // pending 块在视口外(bootVisible 已滚顶)→ 滚到它(真实流内:锚底会带入;测试显式跳)。
  await page.evaluate(() => {
    const st = JSON.parse(window.__chat.ask_state()) as { block: number };
    window.__chat.scroll_to(st.block);
  });

  // 表单出现且锚定 pending 块矩形(<8px:表单贴块顶;canvas-rect 已对齐画布)。
  const form = page.locator(".ask-form");
  await form.waitFor({ state: "visible", timeout: 10_000 });
  const anchored = await page.evaluate(() => {
    const r = JSON.parse(window.__chat.ask_rect()) as { x: number; y: number } | null;
    if (!r) return null;
    const dpr = window.devicePixelRatio || 1;
    const canvas = document.getElementById("chat")!.getBoundingClientRect();
    const f = document.querySelector(".ask-form")!.getBoundingClientRect();
    return { dx: Math.abs(f.left - (canvas.left + r.x / dpr)), dy: Math.abs(f.top - (canvas.top + r.y / dpr)) };
  });
  expect(anchored, "有块矩形").not.toBeNull();
  expect(anchored!.dx, "表单 x 锚定块").toBeLessThan(8);
  expect(anchored!.dy, "表单 y 锚定块").toBeLessThan(8);
  // 焦点入表单(26② 语义)。
  await expect(page.locator(".ask-form input").first()).toBeFocused();

  // 勾选 0 + 输入 + 确认(真表单路径)。
  await page.locator('.ask-option[value="0"]').check();
  await page.locator(".ask-text").fill("先保持");
  await page.locator(".ask-submit").click();

  // 表单撤、FSM 解阻、答案卡原位落定(所选项 + 输入原文可见;未选项收起)。
  await form.waitFor({ state: "detached", timeout: 10_000 });
  await expect
    .poll(() => page.evaluate(() => window.__chat.session_status()))
    .not.toBe("blocked:question");
  await page.evaluate(() => window.__chat.scroll_to(1));
  const runs = () => page.evaluate(() => window.__chat.visible_text_runs());
  await expect.poll(runs, { timeout: 10_000 }).toContain("已回答");
  expect(await runs()).toContain("保持");
  expect(await runs()).toContain("先保持");
  expect(await runs(), "未选项收起").not.toContain("提高");
});

test("② permission:SDF 按钮矩形中心真实 click → 解阻 + 落定;hover cursor:pointer", async ({ page }) => {
  await boot(page);
  await page.evaluate(() =>
    window.__chat.push_event(
      JSON.stringify({
        type: "permission.asked",
        properties: { sessionID: "s", title: "允许写文件?" },
      }),
    ),
  );
  await expect
    .poll(() => page.evaluate(() => window.__chat.session_status()))
    .toBe("blocked:permission");
  await page.evaluate(() => {
    const st = JSON.parse(window.__chat.ask_state()) as { block: number };
    window.__chat.scroll_to(st.block);
  });

  // 命中盒非空(A 路观测点;仅可见块产命中盒 —— 屏外按钮不可点,by design)。
  interface T { id: string; x: number; y: number; w: number; h: number }
  await expect
    .poll(() => page.evaluate(() => (JSON.parse(window.__chat.ask_hit_targets()) as T[]).length), {
      timeout: 10_000,
    })
    .toBe(2);
  const t = (await page.evaluate(() => window.__chat.ask_hit_targets()).then((s) => JSON.parse(s) as T[]))
    .find((x) => x.id === "allow")!;

  // hover 按钮中心 → cursor:pointer(web input.ts 手感线)。
  const dpr = await page.evaluate(() => window.devicePixelRatio || 1);
  const canvas = await page.locator("#chat").boundingBox();
  const cx = canvas!.x + (t.x + t.w / 2) / dpr;
  const cy = canvas!.y + (t.y + t.h / 2) / dpr;
  await page.mouse.move(cx, cy);
  await expect
    .poll(() => page.evaluate(() => (document.getElementById("chat") as HTMLElement).style.cursor))
    .toBe("pointer");

  // 真实 click(≤4px tap 判定)→ 应答 + 落定卡。
  await page.mouse.click(cx, cy);
  await expect
    .poll(() => page.evaluate(() => window.__chat.session_status()))
    .not.toBe("blocked:permission");
  await page.evaluate(() => window.__chat.scroll_to(1));
  await expect
    .poll(() => page.evaluate(() => window.__chat.visible_text_runs()), { timeout: 10_000 })
    .toContain("已允许");
});

test("③ 落定卡文本可复制(text-layer 选区路径)", async ({ page }) => {
  await boot(page);
  await page.evaluate(() =>
    window.__chat.push_event(
      JSON.stringify({
        type: "permission.asked",
        properties: { sessionID: "s", title: "允许访问网络?" },
      }),
    ),
  );
  // 事件经注入队列下一帧才被 ingest → 等阻塞态就绪再直答(本用例测落定卡可复制,非交互路径)。
  await expect
    .poll(() => page.evaluate(() => window.__chat.session_status()))
    .toBe("blocked:permission");
  await page.evaluate(() => window.__chat.reply_permission(true));
  await page.evaluate(() => window.__chat.scroll_to(1));
  await expect
    .poll(() => page.evaluate(() => window.__chat.visible_text_runs()), { timeout: 10_000 })
    .toContain("已允许");
  // 程序化选中落定卡文本 → 选区高亮出现(可复制性由 selection/copy 全家桶保证)。
  await page.evaluate(() => window.__chat.set_selection(new Uint32Array([1, 0, 10])));
  await expect
    .poll(() => page.evaluate(() => window.__chat.stats().selRects), { timeout: 8_000 })
    .toBeGreaterThan(0);
});
