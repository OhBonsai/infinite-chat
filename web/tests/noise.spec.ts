// noise.spec.ts — Plan 36 N1:噪声库确定性(DoD-2)—— 同 seed 双次独立加载,
// noise 四象限格(静态,time 恒 0)像素**逐字节一致**;并留 golden 锁可视化回归。
// 格坐标同 app.rs push_shaderbox_gallery:TILE 40/GAP 8/MARGIN 16,index = ICON_COUNT+2 = 68。
import { test, expect, type Page } from "@playwright/test";

const TILE = 40, GAP = 8, MARGIN = 16, IDX = 68;

function noiseTileClip(viewportW: number): { x: number; y: number; width: number; height: number } {
  const pitch = TILE + GAP;
  const cols = Math.max(1, Math.floor((viewportW - 2 * MARGIN) / pitch));
  const col = IDX % cols, row = Math.floor(IDX / cols);
  return { x: MARGIN + col * pitch, y: MARGIN + row * pitch, width: TILE, height: TILE };
}

async function shootNoiseTile(page: Page): Promise<string> {
  await page.goto("/dev.html?empty&noinput&gallery", { waitUntil: "domcontentloaded" });
  await page.waitForFunction(() => !!(window as unknown as { __chat?: unknown }).__chat, null, {
    timeout: 60_000,
  });
  await page.waitForTimeout(1200); // 上屏若干帧(静态格 time=0,帧数无关)
  const clip = noiseTileClip(page.viewportSize()!.width);
  return (await page.screenshot({ clip })).toString("base64");
}

test("N1 噪声四象限:双次独立加载像素逐字节一致(同 seed 确定性)", async ({ page }) => {
  const a = await shootNoiseTile(page);
  const b = await shootNoiseTile(page);
  expect(a.length, "格内有内容").toBeGreaterThan(100);
  expect(b, "双跑逐字节一致(R8)").toBe(a);
});

test("N1 噪声四象限 golden(可视化回归锁)", async ({ page }) => {
  await page.goto("/dev.html?empty&noinput&gallery", { waitUntil: "domcontentloaded" });
  await page.waitForFunction(() => !!(window as unknown as { __chat?: unknown }).__chat, null, {
    timeout: 60_000,
  });
  await page.waitForTimeout(1200);
  const clip = noiseTileClip(page.viewportSize()!.width);
  await expect(page).toHaveScreenshot("noise-quadrants.png", { clip, maxDiffPixelRatio: 0 });
});

test("N2 距离带三件 golden(glow / 解析 shadow / 条纹带)", async ({ page }) => {
  await page.goto("/dev.html?empty&noinput&gallery", { waitUntil: "domcontentloaded" });
  await page.waitForFunction(() => !!(window as unknown as { __chat?: unknown }).__chat, null, {
    timeout: 60_000,
  });
  await page.waitForTimeout(1200);
  // 演示条钉视口 (16,480) 起,三块 120×72 + 间距;含 glow/shadow 外扩留边。
  await expect(page).toHaveScreenshot("fx-distance-band.png", {
    clip: { x: 0, y: 460, width: 440, height: 120 },
    maxDiffPixelRatio: 0,
  });
});

test("F2 后处理三小件 golden(paused 定帧;grain seed=帧计数 → 冻结稳定)", async ({ page }) => {
  await page.goto("/dev.html?empty&noinput", { waitUntil: "domcontentloaded" });
  await page.waitForFunction(() => !!(window as unknown as { __chat?: unknown }).__chat, null, {
    timeout: 60_000,
  });
  await page.evaluate(() => {
    window.__chat.set_stream_rate(1e9);
    window.__chat.set_reveal_cps(1e9);
    window.__chat.push_event(
      JSON.stringify({
        type: "message.part.updated",
        properties: {
          part: { type: "text", id: "p1", messageID: "m1", text: "后处理三小件定帧样张:vignette + grain + chroma。" },
          time: 1,
        },
      }),
    );
  });
  await page.waitForTimeout(2000);
  await page.evaluate(() => window.__chat.set_post_params(0.35, 0.12, 2));
  await page.waitForTimeout(300); // 参数进 FrameData(至少一帧)
  await page.evaluate(() => window.__chat.set_paused(true)); // 再冻结 → grain seed 恒定
  await page.waitForTimeout(300);
  await expect(page).toHaveScreenshot("post-three.png", {
    clip: { x: 200, y: 0, width: 640, height: 220 },
    maxDiffPixelRatio: 0,
  });
});

// ── Plan 38 E2:效果预设切换(0041)──

test("E2 预设切换:expressive 生效 / off 压平 / 自定义 JSON 加载 / 坏名拒绝", async ({ page }) => {
  await page.goto("/dev.html?empty&noinput", { waitUntil: "domcontentloaded" });
  await page.waitForFunction(() => !!(window as unknown as { __chat?: unknown }).__chat, null, {
    timeout: 60_000,
  });
  const r = await page.evaluate(() => {
    const c = window.__chat;
    const out = {
      initial: c.effect_preset_name(),
      hitExpressive: c.set_effect_preset("expressive"),
      afterExpressive: c.effect_preset_name(),
      rejectUnknown: c.set_effect_preset("nope"),
      afterUnknown: c.effect_preset_name(),
      hitOff: c.set_effect_preset("off"),
      afterOff: c.effect_preset_name(),
      customOk: c.set_effect_preset_json(
        JSON.stringify({ name: "custom", exit: { dissolve_ms: 250 } }),
      ),
      afterCustom: c.effect_preset_name(),
      badRejected: !c.set_effect_preset_json("{oops"),
      afterBad: c.effect_preset_name(),
    };
    return out;
  });
  expect(r.initial, "默认 subtle").toBe("subtle");
  expect(r.hitExpressive && r.afterExpressive === "expressive", "切 expressive").toBeTruthy();
  expect(!r.rejectUnknown && r.afterUnknown === "expressive", "未知名不动当前档").toBeTruthy();
  expect(r.hitOff && r.afterOff === "off", "切 off").toBeTruthy();
  expect(r.customOk && r.afterCustom === "custom", "自定义 JSON 生效(数据驱动零编译)").toBeTruthy();
  expect(r.badRejected && r.afterBad === "custom", "坏 JSON 整份拒绝不半应用").toBeTruthy();
});

test("E3 expressive 开启态 golden(paused 定帧,双跑一致)", async ({ page }) => {
  await page.goto("/dev.html?empty&noinput", { waitUntil: "domcontentloaded" });
  await page.waitForFunction(() => !!(window as unknown as { __chat?: unknown }).__chat, null, {
    timeout: 60_000,
  });
  await page.evaluate(() => {
    window.__chat.set_effect_preset("expressive");
    window.__chat.set_stream_rate(1e9);
    window.__chat.set_reveal_cps(1e9);
    window.__chat.push_event(
      JSON.stringify({
        type: "message.part.updated",
        properties: {
          part: { type: "text", id: "p1", messageID: "m1", text: "expressive 档定帧样张:spring 入场已收敛,静态观感与 subtle 一致(差异在动态)。" },
          time: 1,
        },
      }),
    );
  });
  await page.waitForTimeout(2500); // 揭示 + spring 收敛(t≥1 恰 1,AR3)
  await page.evaluate(() => window.__chat.set_paused(true));
  await page.waitForTimeout(300);
  await expect(page).toHaveScreenshot("expressive-settled.png", {
    clip: { x: 200, y: 0, width: 640, height: 160 },
    maxDiffPixelRatio: 0,
  });
});
