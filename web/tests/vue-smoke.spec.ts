// vue-smoke.spec.ts — Plan 34 S5:框架无关性回归(research R8 / 0021 划界)。
// 同一份 wasm + 同一套桥在 Vue 壳下:加载 / 流式渲染 / 滚动跟随 / 链接回调 四步 smoke。
// Vue 子工程依赖未装时**显式 skip**(fail-loud,报告可见;跑法见 web/examples/vue/README)。
import { test, expect } from "@playwright/test";
import { spawn, type ChildProcess } from "node:child_process";
import { existsSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const VUE_DIR = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../examples/vue");
const VUE_URL = "http://localhost:5174";
const depsInstalled = existsSync(path.join(VUE_DIR, "node_modules", ".bin", "vite"));

let server: ChildProcess | null = null;

test.beforeAll(async () => {
  test.skip(!depsInstalled, "vue harness 依赖未装(cd web/examples/vue && npm install)— 显式跳过");
  // 已有 5174 在跑则复用;否则起 dev server。
  const alive = await fetch(VUE_URL).then(() => true).catch(() => false);
  if (!alive) {
    server = spawn("npm", ["run", "dev"], { cwd: VUE_DIR, stdio: "ignore", detached: false });
    for (let i = 0; i < 120; i++) {
      if (await fetch(VUE_URL).then(() => true).catch(() => false)) return;
      await new Promise((r) => setTimeout(r, 500));
    }
    throw new Error("vue dev server 未在 60s 内就绪");
  }
});

test.afterAll(() => {
  server?.kill();
});

test("S5 Vue 壳 smoke:加载/流式渲染/滚动跟随/链接回调,零改 wasm API", async ({ page }) => {
  test.skip(!depsInstalled, "vue harness 依赖未装 — 显式跳过");
  await page.goto(VUE_URL, { waitUntil: "domcontentloaded" });
  // ① 加载:同一份 wasm 起来,Vue 状态条渲染出会话态。
  await page.waitForFunction(() => !!(window as unknown as { __chat?: unknown }).__chat, null, {
    timeout: 60_000,
  });
  // 无 replay/server 时 bootCanvas 走合成 demo 流 → 状态即会话态(streaming/idle 均可);
  // Vue 响应式状态条渲染出「状态 · 跟随态」即证加载成功。
  await expect(page.locator("[data-test=status]")).toContainText("·");
  // ② 流式渲染:push 事件 → 画布可见文本(与主 harness 同一探针)。
  await page.evaluate(() => {
    window.__chat.set_stream_rate(1e9);
    window.__chat.set_reveal_cps(1e9);
    window.__chat.push_event(
      JSON.stringify({
        type: "message.part.updated",
        properties: {
          part: {
            type: "text",
            id: "p1",
            messageID: "m1",
            text: "Vue 壳里流式渲染,链接见 [官网](https://example.com/vue) 结束。".repeat(20),
          },
          time: 1,
        },
      }),
    );
  });
  await expect
    .poll(() => page.evaluate(() => window.__chat.visible_text_runs()), { timeout: 15_000 })
    .toContain("Vue 壳里流式渲染");
  // ③ 滚动跟随:上滚释放 → Released;scroll_to_latest → Following(0038 FSM 同一行为)。
  await page.evaluate(() => window.__chat.pan_by(0, -200));
  await expect
    .poll(() => page.evaluate(() => window.__chat.follow_state()), { timeout: 5_000 })
    .toBe("released");
  await page.evaluate(() => window.__chat.scroll_to_latest());
  await expect
    .poll(() => page.evaluate(() => window.__chat.follow_state()), { timeout: 5_000 })
    .toBe("following");
  // ④ 链接回调:tap 命中 → Vue 响应式状态收到精确 URL(onOpenUrl 交宿主)。
  await expect
    .poll(
      () =>
        page.evaluate(() =>
          (JSON.parse(window.__chat.link_hit_targets()) as Array<{ url: string }>).some(
            (t) => t.url === "https://example.com/vue",
          ),
        ),
      { timeout: 10_000 },
    )
    .toBe(true);
  const tapped = await page.evaluate(() => {
    const t = (
      JSON.parse(window.__chat.link_hit_targets()) as Array<{
        url: string;
        x: number;
        y: number;
        w: number;
        h: number;
      }>
    ).find((x) => x.url === "https://example.com/vue");
    return t ? window.__chat.tap(t.x + t.w / 2, t.y + t.h / 2) : false;
  });
  expect(tapped, "tap 命中链接").toBeTruthy();
  await expect(page.locator("[data-test=opened-url]")).toHaveText("https://example.com/vue");
});
