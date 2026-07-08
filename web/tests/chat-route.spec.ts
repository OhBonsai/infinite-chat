// chat-route.spec.ts — Plan 25:/chat 剧本回放页端到端。快放跑完剧本,断言里程碑序列。
// 剧本本身即 Plan 24 §4 想要的「全事件回归资产」。
import { test, expect } from "@playwright/test";
import { assertWebGpu } from "./helpers";

test("showcase-full 全事件谱:11 场里程碑顺序 + 终态 idle(Plan 25 §3)", async ({ page }) => {
  const errors: string[] = [];
  page.on("pageerror", (e) => errors.push(e.message));

  await page.goto("/chat/?script=showcase-full&speed=12", { waitUntil: "domcontentloaded" });
  await assertWebGpu(page);
  await page.waitForFunction(() => !!(window as unknown as { __chat?: unknown }).__chat, null, {
    timeout: 60_000,
  });
  await page.evaluate(() => {
    window.__chat.set_stream_rate(1e9);
    window.__chat.set_reveal_cps(1e9);
  });

  // 快放下逐里程碑轮询(持久内容;Dock 瞬态由 dock.spec/mini 覆盖)。滚到底跟随最新。
  const runs = () => page.evaluate(() => window.__chat.visible_text_runs());
  const milestones = [
    "偶发超时", // 场1 用户气泡
    "连接池", // 场2 reasoning 正文
    "p99", // 场3 流式 markdown 表格
    "Read", // 场4 tool 三态(终态显示名,Plan 28 R3)
    "指数退避", // 场6 diff 块(新增行)
    "perf-report", // 场7 file 附件
    "12 passed", // 场9 错误后重试 completed
    "Context compacted", // 场10 compaction
    "CHANGELOG", // 场11 二轮追问
  ];
  for (const m of milestones) {
    await expect.poll(runs, { timeout: 60_000, message: `里程碑未出现: ${m}` }).toContain(m);
  }
  // 终态 idle;全程无页面错误。
  await expect
    .poll(() => page.evaluate(() => window.__chat.session_status()), { timeout: 60_000 })
    .toBe("idle");
  expect(errors, `页面错误: ${errors.join("; ")}`).toHaveLength(0);
});

test("mini 剧本端到端:打字→用户气泡→流式→Dock 自动应答→工具卡→idle", async ({ page }) => {
  const errors: string[] = [];
  page.on("pageerror", (e) => errors.push(e.message));

  await page.goto("/chat/?script=mini&speed=8", { waitUntil: "domcontentloaded" });
  await assertWebGpu(page);
  await page.waitForFunction(() => !!(window as unknown as { __chat?: unknown }).__chat, null, {
    timeout: 60_000,
  });
  // 快放下即时揭示,免等吐字动画。
  await page.evaluate(() => {
    window.__chat.set_stream_rate(1e9);
    window.__chat.set_reveal_cps(1e9);
  });

  const runs = () => page.evaluate(() => window.__chat.visible_text_runs());

  // 里程碑 1:用户气泡上屏(剧本事件路径,非打字机本身)。
  await expect.poll(runs, { timeout: 30_000 }).toContain("测试怎么跑");
  // 里程碑 2:Dock 曾弹出并被剧本自动应答(阻塞态来去)。轮询捕捉 blocked 或直接看到已解除后的工具卡。
  // (dock allow 后 FSM 离开 blocked;若快放跳过了 blocked 窗口,则以工具卡为准。)
  // 里程碑 3:工具卡(bash · done)可见。
  await expect.poll(runs, { timeout: 30_000 }).toContain("Shell"); // Plan 28 R3 显示名
  // 里程碑 4:assistant 结语可见。
  await expect.poll(runs, { timeout: 30_000 }).toContain("run.mjs");
  // 里程碑 5:终态 idle(session.status idle 收尾)。
  await expect
    .poll(() => page.evaluate(() => window.__chat.session_status()), { timeout: 30_000 })
    .toBe("idle");
  // 播放器 chrome 存在;全程无页面错误。
  await expect(page.locator(".chat-player")).toBeVisible();
  expect(errors, `页面错误: ${errors.join("; ")}`).toHaveLength(0);
});

test("画布列宽:?w= 生效 + overlay 对齐画布 + resize 后引擎/overlay 跟随", async ({ page }) => {
  await page.goto("/chat/?script=mini&speed=8&w=500", { waitUntil: "domcontentloaded" });
  await assertWebGpu(page);
  await page.waitForFunction(() => !!(window as unknown as { __chat?: unknown }).__chat, null, {
    timeout: 60_000,
  });
  await page.evaluate(() => {
    window.__chat.set_stream_rate(1e9);
    window.__chat.set_reveal_cps(1e9);
  });

  // ?w=500 生效(border-box 含 1px 虚线边框)。
  const cw = await page.evaluate(() => document.getElementById("chat")!.clientWidth);
  expect(cw, "?w=500 应让画布列宽 ≈500").toBeLessThanOrEqual(500);
  expect(cw).toBeGreaterThan(460);

  // 等文本层出 span 后:overlay 容器必须对齐画布矩形(canvas-rect),不再假设视口 (0,0)。
  await page.locator(".text-layer span").first().waitFor({ state: "attached", timeout: 30_000 });
  const align = () =>
    page.evaluate(() => {
      const c = document.getElementById("chat")!.getBoundingClientRect();
      const l = document.querySelector(".text-layer")!.getBoundingClientRect();
      return { dl: Math.abs(l.left - c.left), dt: Math.abs(l.top - c.top), dw: Math.abs(l.width - c.width) };
    });
  let a = await align();
  expect(a.dl, "text-layer 左缘对齐画布").toBeLessThan(2);
  expect(a.dt, "text-layer 上缘对齐画布").toBeLessThan(2);
  expect(a.dw, "text-layer 宽度对齐画布").toBeLessThan(2);

  // 模拟拖拽 resize(直接改 #stage 宽,同原生手柄效果):引擎后备缓冲与 overlay 下一帧跟随。
  await page.evaluate(() => {
    document.getElementById("stage")!.style.width = "760px";
  });
  await expect
    .poll(
      () =>
        page.evaluate(() => {
          const canvas = document.getElementById("chat") as HTMLCanvasElement;
          const dpr = window.devicePixelRatio || 1;
          // wasm on_resize 应已把后备缓冲重设为新 clientWidth×dpr(canvas-rect 桥发的 resize)。
          return Math.abs(canvas.width - canvas.clientWidth * dpr);
        }),
      { timeout: 10_000, message: "resize 后 wasm 未重配后备缓冲" },
    )
    .toBeLessThan(2);
  // overlay 由 rAF 泵逐帧跟随(canvas-rect):stage 居中 → resize 会平移画布左缘(500→760 即
  // -130px),单次采样会撞「泵还差一帧」的窗口(负载下必现)→ 按 T1 用显式条件 poll 收敛值。
  await expect
    .poll(async () => (await align()).dl, {
      timeout: 5_000,
      message: "resize 后 text-layer 应跟随对齐",
    })
    .toBeLessThan(2);
  a = await align();
  expect(a.dw, "resize 后 text-layer 宽度跟随").toBeLessThan(2);
});
