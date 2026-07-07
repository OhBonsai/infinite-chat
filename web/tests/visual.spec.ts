// visual.spec.ts — Plan 21 P2 视觉回归(0030):选区高亮黄金帧 + "高亮不遮文字"像素证。
//
// 钉死现有 headless Chromium + WebGPU(playwright.config.ts)。截图区**裁掉左侧头像 glow-orb 列**
// (那是 dynamic shaderbox,每帧脉动 → 非确定);裁出的正文 + 选区在 settled 后是静态的 → 可比对。
import { test, expect, type Page } from "@playwright/test";
import { PNG } from "pngjs";
import { assertWebGpu, bootVisible, readRuns } from "./helpers";

// 正文区;settled 后静态。Plan 25 内容列居中(默认视口 1280 → 列左缘 (1280-760)/2 = 260,
// 动态 orb 头像在 220..252)→ 裁剪从列左缘起,继续排除 orb 列(每帧脉动 → 非确定)。
const CLIP = { x: 260, y: 8, width: 520, height: 200 };

async function applySelection(page: Page, start: number, end: number): Promise<void> {
  await page.evaluate(
    ([s, e]) => window.__chat.set_selection(new Uint32Array([0, s, e])),
    [start, end],
  );
  await expect
    .poll(() => page.evaluate(() => window.__chat.stats().selRects), { timeout: 8_000 })
    .toBeGreaterThan(0);
  await page.waitForTimeout(120);
}

test("E01 渲染无 raw markdown 残留(结构块已成型)", async ({ page }) => {
  await bootVisible(page); // showcase 全揭示 settled
  const text = (await readRuns(page)).map((r) => r.text).join("\n");
  // 已渲染的可见文本里不应有裸 markdown 语法残留:成对粗体 **、围栏 ```、表格分隔行 |---。
  expect(text, "无裸 ``` 围栏").not.toContain("```");
  expect(text, "无裸 ** 粗体标记").not.toContain("**");
  expect(text, "无裸表格分隔行").not.toMatch(/\|\s*-{3,}/);
});

test("E05 滚动来回零跳变(块 y 复位,无漂移)", async ({ page }) => {
  await bootVisible(page);
  const y0 = (await readRuns(page))[0]?.y ?? 0;
  await page.evaluate(() => window.__chat.pan_by(0, 300));
  await page.waitForTimeout(120);
  await page.evaluate(() => window.__chat.pan_by(0, -300)); // 来回 → 相机复位
  await page.waitForTimeout(150);
  const y1 = (await readRuns(page))[0]?.y ?? -999;
  // 同块在相机复位后 y 必回原位(不累积漂移 / 不跳变)。
  expect(Math.abs(y1 - y0), "滚动来回后块 y 复位").toBeLessThan(2);
});

test("V1 选区高亮黄金帧", async ({ page }) => {
  await bootVisible(page);
  await applySelection(page, 6, 120); // 跨多行,确保墨团充满裁剪区(可见回归才有意义)
  // 首次跑生成基线;容差容 GPU 抗锯齿亚像素噪声(0030/Plan21 §3.3)。
  await expect(page).toHaveScreenshot("sel-highlight.png", {
    clip: CLIP,
    maxDiffPixelRatio: 0.02,
    animations: "disabled",
  });
});

test("V3 SDF 墨团黄金帧(P3 多行选区)", async ({ page }) => {
  await bootVisible(page);
  // 跨多行选区 → 逐行圆角墨团(P3)。范围足够大以覆盖正文区多行。
  await applySelection(page, 6, 220);
  await expect(page).toHaveScreenshot("ink-blob.png", {
    clip: CLIP,
    maxDiffPixelRatio: 0.02,
    animations: "disabled",
  });
});

// ── Plan 25 M3:组件代表帧黄金图(设计收敛后的像素回归)──
// 全部 `?empty` 干净画布 + push_event 合成 + set_paused/seek_reveal 定帧(含流光相位/呼吸相位,
// 时间 = 注入时钟 → 完全确定);clip 从内容列左缘起(默认视口 1280 → 列左缘 260,orb 在 220..252
// 之外)。fixture 与 design-review S3/S6/S7/S9 同源。
const partUpd = (p: Record<string, unknown>) =>
  JSON.stringify({ type: "message.part.updated", properties: { part: p, time: 1 } });
const msgUpd = (id: string, role: string) =>
  JSON.stringify({
    type: "message.updated",
    properties: { info: { id, sessionID: "s-v", role, time: { created: 1 } } },
  });

async function bootEmptyAndPush(page: Page, events: string[]): Promise<void> {
  await page.goto("/?empty&noinput", { waitUntil: "domcontentloaded" });
  await assertWebGpu(page);
  await page.waitForFunction(() => !!(window as unknown as { __chat?: unknown }).__chat, null, {
    timeout: 60_000,
  });
  await page.evaluate(() => {
    window.__chat.set_stream_rate(1e9);
    window.__chat.set_reveal_cps(1e9);
  });
  for (let attempt = 0; attempt < 5; attempt++) {
    await page.evaluate((evs) => {
      for (const e of evs) window.__chat.push_event(e);
    }, events);
    try {
      await expect
        .poll(() => page.evaluate(() => window.__chat.visible_turns() !== "[]"), {
          timeout: 5_000,
          intervals: [200],
        })
        .toBe(true);
      break;
    } catch {
      /* 引擎未就绪 → 重推(事件幂等,AR4) */
    }
  }
}

const GOLD_CLIP = { x: 260, y: 8, width: 640, height: 300 };

test("V4 tool 卡四态黄金帧(徽章 + running 流光,定相位)", async ({ page }) => {
  const tool = (id: string, status: string) =>
    partUpd({
      type: "tool",
      id,
      messageID: "m-a1",
      tool: "bash",
      state: {
        status,
        input: { cmd: `echo ${status}` },
        ...(status === "completed" ? { output: "ok" } : {}),
        ...(status === "error" ? { error: "exit 1" } : {}),
      },
    });
  await bootEmptyAndPush(page, [
    msgUpd("m-a1", "assistant"),
    tool("t-p", "pending"),
    tool("t-r", "running"),
    tool("t-d", "completed"),
    tool("t-e", "error"),
  ]);
  await page.evaluate(() => {
    window.__chat.set_paused(true);
    window.__chat.seek_reveal(30_000); // 全 settled;流光相位 = f(30s 慢时钟)确定
  });
  await expect(page).toHaveScreenshot("tool-cards.png", {
    clip: GOLD_CLIP,
    maxDiffPixelRatio: 0.02,
    animations: "disabled",
  });
});

test("V5 表格骨架中点黄金帧(风格 3 网格先行)", async ({ page }) => {
  const table =
    "对比:\n\n| 能力 | 手法 | 规模 |\n|---|---|---|\n| 文字 | SDF | 任意缩放 |\n| 流式 | 逐字 | 无跳变 |\n| 历史 | 冻结 | 100+ 轮 |\n\n收。";
  await bootEmptyAndPush(page, [
    msgUpd("m-a1", "assistant"),
    partUpd({ type: "text", id: "p-t", messageID: "m-a1", text: table }),
  ]);
  await page.evaluate(() => {
    (window.__chat as unknown as { set_reveal_preset(n: string): void }).set_reveal_preset("reader");
    window.__chat.set_rhythm(JSON.stringify({ tempo_ms: 60, microtiming: 0 }));
    window.__chat.set_paused(true);
    window.__chat.seek_reveal(700); // 网格骨架 + 表头字揭示中(microtiming 关 → 逐 ms 确定)
  });
  await expect(page).toHaveScreenshot("table-skeleton.png", {
    clip: GOLD_CLIP,
    maxDiffPixelRatio: 0.02,
    animations: "disabled",
  });
});

test("V6 diff 卡黄金帧(行带 + 摘要 + 5 格条)", async ({ page }) => {
  await bootEmptyAndPush(page, [
    msgUpd("m-a1", "assistant"),
    partUpd({
      type: "tool",
      id: "t-ed",
      messageID: "m-a1",
      tool: "edit",
      state: {
        status: "completed",
        metadata: {
          filediff:
            "@@ -1,4 +1,5 @@\n ctx line\n-old_alpha\n-old_beta\n+new_alpha\n+new_beta\n+new_gamma\n ctx tail\n",
        },
      },
    }),
  ]);
  await page.evaluate(() => {
    window.__chat.set_paused(true);
    window.__chat.seek_reveal(30_000);
  });
  await expect(page).toHaveScreenshot("diff-card.png", {
    clip: GOLD_CLIP,
    maxDiffPixelRatio: 0.02,
    animations: "disabled",
  });
});

test("V7 整页布局黄金帧(居中列 + user 弱底 + 回合分组)", async ({ page }) => {
  await bootEmptyAndPush(page, [
    msgUpd("m-u1", "user"),
    partUpd({ type: "text", id: "p-u1", messageID: "m-u1", text: "帮我看看这个仓库的测试怎么跑?" }),
    msgUpd("m-a1", "assistant"),
    partUpd({
      type: "text",
      id: "p-a1",
      messageID: "m-a1",
      text: "测试入口是 `test/run.mjs`,四层门:gates、native、unit、e2e,全绿才过。",
    }),
    msgUpd("m-u2", "user"),
    partUpd({ type: "text", id: "p-u2", messageID: "m-u2", text: "单跑 e2e 呢?" }),
    msgUpd("m-a2", "assistant"),
    partUpd({
      type: "text",
      id: "p-a2",
      messageID: "m-a2",
      text: "加 `--suite e2e`;更新黄金图再加 `--update-snapshots`。",
    }),
  ]);
  await page.evaluate(() => {
    window.__chat.set_paused(true);
    window.__chat.seek_reveal(30_000);
  });
  await expect(page).toHaveScreenshot("page-layout.png", {
    clip: { x: 260, y: 8, width: 640, height: 420 },
    maxDiffPixelRatio: 0.02,
    animations: "disabled",
  });
});

test("V2 高亮不遮文字(像素证)", async ({ page }) => {
  await bootVisible(page);

  // 无选区帧。
  await page.evaluate(() => window.__chat.set_selection(new Uint32Array(0)));
  await page.waitForTimeout(120);
  const png0 = PNG.sync.read(await page.screenshot({ clip: CLIP }));

  // 有选区帧(覆盖正文区前若干字)。
  await applySelection(page, 0, 80);
  const png1 = PNG.sync.read(await page.screenshot({ clip: CLIP }));

  expect(png1.width).toBe(png0.width);
  expect(png1.height).toBe(png0.height);

  // 统计:正文是浅色字,暗底;选区是偏蓝半透明叠底。
  const isBright = (d: Buffer, i: number) => d[i] > 175 && d[i + 1] > 175 && d[i + 2] > 175;
  let bright0 = 0;
  let bright1 = 0;
  let blueShift = 0;
  for (let i = 0; i < png0.data.length; i += 4) {
    if (isBright(png0.data, i)) bright0 += 1;
    if (isBright(png1.data, i)) bright1 += 1;
    // 选区叠底 → 蓝通道相对无选区帧抬升(且非变暗)。
    if (png1.data[i + 2] - png0.data[i + 2] > 14) blueShift += 1;
  }

  expect(bright0, "正文区本应有浅色文字像素").toBeGreaterThan(50);
  expect(blueShift, "有选区帧应出现偏蓝高亮像素(高亮已渲染)").toBeGreaterThan(50);
  // 关键:文字像素在加了高亮后**基本保留**(若被高亮遮盖会大幅消失)→ 证文字画在高亮之上。
  expect(bright1, "高亮之上文字像素应基本保留(未被遮)").toBeGreaterThan(bright0 * 0.5);
});
