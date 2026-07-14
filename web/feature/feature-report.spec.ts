// feature-report.spec.ts — Plan 39 捕获 harness:逐 manifest 项拍照/录像 + 可见性断言。
// 铁律(GOAL §0):报告即测试 —— 任一项捕获/断言失败 = 该功能红(fail loud,不出残缺报告)。
// 产物:test/results/feature-assets/<id>.png|webm(+ .json 元数据);FEATURE_ONLY=<group|id> 增量。
import { expect, test, chromium, type Browser, type Page } from "@playwright/test";
import { copyFileSync, mkdirSync, readFileSync, statSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

// 自动裁高(plan39 用户反馈「像手机屏,截全部或部分高度」):无显式 clip 的项截完整窄视口后,
// 从底部向上裁掉纯底色空白 —— 内容多截多、内容少截少,不留大片黑边。底色 = 左上角像素。
async function autoCropBottom(buf: Buffer): Promise<Buffer> {
  const { PNG } = await import("pngjs");
  const png = PNG.sync.read(buf);
  const { width: w, height: h, data } = png;
  const [br, bg, bb] = [data[0], data[1], data[2]];
  let lastInk = 0;
  for (let y = 0; y < h; y++) {
    let rowInk = false;
    for (let x = 0; x < w; x += 4) {
      const i = (y * w + x) * 4;
      if (
        Math.abs(data[i] - br) + Math.abs(data[i + 1] - bg) + Math.abs(data[i + 2] - bb) >
        24
      ) {
        rowInk = true;
        break;
      }
    }
    if (rowInk) lastInk = y;
  }
  const keep = Math.min(h, lastInk + 24); // 内容底 + 24px 边距
  if (keep >= h - 2) return buf; // 满屏内容,不裁
  const out = new PNG({ width: w, height: keep });
  data.copy(out.data, 0, 0, w * keep * 4);
  return PNG.sync.write(out);
}

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const ASSETS = path.join(ROOT, "test", "results", "feature-assets");
const manifest = JSON.parse(readFileSync(path.join(ROOT, "test", "feature-manifest.json"), "utf8")) as {
  groups: { id: string }[];
  items: ManifestItem[];
};

interface ManifestItem {
  id: string;
  group: string;
  kind: "png" | "webm";
  title: string;
  capture: {
    scene: string;
    steps: string[];
    assert: { textVisible?: string; ink?: boolean };
    clip?: { x: number; y: number; w: number; h: number };
    dpr?: number;
    durationMs?: number;
  };
}

const SCENE_URL: Record<string, string> = {
  // Plan 40:harness 从 / 迁到 /dev.html(/ 现为 landing 页);chat-full 走 /chat/。
  showcase: "/dev.html?replay=showcase&noinput",
  "chat-full": "/chat/?script=showcase-full&speed=8",
  empty: "/dev.html?empty&noinput",
  gallery: "/dev.html?empty&noinput&gallery",
  debug: "/dev.html?replay=showcase&noinput&debug",
};

const ONLY = process.env.FEATURE_ONLY ?? "";
const picked = manifest.items.filter(
  (it) => !ONLY || it.id === ONLY || it.group === ONLY,
);

const LAUNCH_ARGS = [
  "--enable-unsafe-webgpu",
  "--enable-features=Vulkan",
  "--enable-webgpu-developer-features",
  "--ignore-gpu-blocklist",
  "--use-angle=default",
];

async function boot(page: Page, scene: string): Promise<void> {
  await page.goto(SCENE_URL[scene], { waitUntil: "domcontentloaded" });
  await page.waitForFunction(() => !!(window as unknown as { __chat?: unknown }).__chat, null, {
    timeout: 60_000,
  });
  if (scene !== "chat-full") {
    // showcase/empty:加速载入 + 即时揭示(定帧惯例);chat-full 剧本自带 speed 参数。
    await page.evaluate(() => {
      window.__chat.set_stream_rate(1e9);
      window.__chat.set_reveal_cps(1e9);
    });
  }
}

/** 等 visible_turns 稳定(同 helpers.waitStable;此处独立避免跨目录 import)。 */
async function settle(page: Page): Promise<void> {
  const hist: string[] = [];
  await expect
    .poll(
      async () => {
        const cur = await page.evaluate(() => window.__chat.visible_turns());
        hist.push(cur);
        if (hist.length > 3) hist.shift();
        return hist.length === 3 && hist[0] === hist[1] && hist[1] === hist[2] && cur !== "[]";
      },
      { timeout: 90_000, intervals: [300] },
    )
    .toBe(true);
}

async function runSteps(page: Page, item: ManifestItem): Promise<void> {
  for (const step of item.capture.steps) {
    const [op] = step.split(":", 1);
    const arg = step.slice(op.length + 1);
    if (op === "settle") await settle(page);
    else if (op === "waitMs") await page.waitForTimeout(Number(arg));
    else if (op === "pause") await page.evaluate(() => window.__chat.set_paused(true));
    else if (op === "panUp")
      await page.evaluate((px) => window.__chat.pan_by(0, -px), Number(arg));
    else if (op === "pushText")
      await page.evaluate(
        (t) =>
          window.__chat.push_event(
            JSON.stringify({
              type: "message.part.updated",
              properties: { part: { type: "text", id: "px", messageID: "mx", text: t }, time: 1 },
            }),
          ),
        arg,
      );
    else if (op === "pushTool")
      await page.evaluate(
        (json) =>
          window.__chat.push_event(
            JSON.stringify({
              type: "message.part.updated",
              properties: { part: JSON.parse(json), time: 1 },
            }),
          ),
        arg,
      );
    else if (op === "removePart")
      await page.evaluate(
        (id) =>
          window.__chat.push_event(
            JSON.stringify({
              type: "message.part.removed",
              properties: { sessionID: "s", messageID: "mx", partID: id },
            }),
          ),
        arg,
      );
    else if (op === "tapFold")
      await page.evaluate(() => {
        const t = JSON.parse(window.__chat.fold_hit_targets())[0] as
          | { x: number; y: number; w: number; h: number }
          | undefined;
        if (!t) throw new Error("无折叠命中盒");
        window.__chat.tap(t.x + t.w / 2, t.y + t.h / 2);
      });
    else if (op === "tapLink")
      await page.evaluate(() => {
        const t = JSON.parse(window.__chat.link_hit_targets())[0] as
          | { x: number; y: number; w: number; h: number }
          | undefined;
        if (!t) throw new Error("无链接命中盒");
        window.__chat.tap(t.x + t.w / 2, t.y + t.h / 2);
      });
    else if (op === "js") await page.evaluate(`(async () => { ${arg} })()`);
    else throw new Error(`未知步骤: ${step}`);
  }
}

/** 可见性断言(报告即测试):textVisible 走引擎可见文本;ink 走像素非空占比。 */
async function assertVisible(page: Page, item: ManifestItem): Promise<void> {
  const a = item.capture.assert;
  if (a.textVisible !== undefined && a.textVisible !== "") {
    const text = await page.evaluate(() => window.__chat.visible_text_runs());
    expect(text, `${item.id}: 文本可见`).toContain(a.textVisible);
  } else if (a.textVisible === "") {
    const runs = await page.evaluate(() => JSON.parse(window.__chat.visible_text_runs()).length);
    expect(runs, `${item.id}: 有可见文本`).toBeGreaterThan(0);
  }
  if (a.ink) {
    const clip = item.capture.clip;
    const buf = await page.screenshot(
      clip ? { clip: { x: clip.x, y: clip.y, width: clip.w, height: clip.h } } : {},
    );
    const { PNG } = await import("pngjs");
    const png = PNG.sync.read(buf);
    let ink = 0;
    const [br, bg, bb] = [png.data[0], png.data[1], png.data[2]]; // 左上角作底色采样
    for (let i = 0; i < png.data.length; i += 4) {
      const d =
        Math.abs(png.data[i] - br) + Math.abs(png.data[i + 1] - bg) + Math.abs(png.data[i + 2] - bb);
      if (d > 24) ink++;
    }
    const ratio = ink / (png.width * png.height);
    expect(ratio, `${item.id}: 墨迹占比 ${ratio}`).toBeGreaterThan(0.001);
  }
}

function metaOut(item: ManifestItem, file: string): void {
  writeFileSync(
    path.join(ASSETS, `${item.id}.json`),
    JSON.stringify({ id: item.id, kind: item.kind, file, at: new Date().toISOString() }),
  );
}

mkdirSync(ASSETS, { recursive: true });

// 默认捕获画幅(plan39 用户反馈):对话是居中一条 ~545px 窄列,不是整块浏览器视口。
// 默认走「手机竖屏」600×1200 + dpr=2(内容列居中占满、两侧小边距、字清晰);需要更宽
// 画幅的项(样式面板/画廊格栅)在 manifest 里用 capture.viewport 显式覆盖。
const DEFAULTS = (manifest as { defaults?: { viewportW: number; viewportH: number; dpr: number } })
  .defaults ?? { viewportW: 600, viewportH: 1200, dpr: 2 };
function viewportOf(item: ManifestItem): { width: number; height: number; dpr: number } {
  const v = (item.capture as { viewport?: { width: number; height: number } }).viewport;
  return {
    width: v?.width ?? DEFAULTS.viewportW,
    height: v?.height ?? DEFAULTS.viewportH,
    dpr: item.capture.dpr ?? DEFAULTS.dpr,
  };
}

for (const item of picked) {
  test(`${item.group}/${item.id} · ${item.title}`, async ({ browser }) => {
    const vp = viewportOf(item);
    if (item.kind === "png") {
      const ctx = await browser.newContext({
        viewport: { width: vp.width, height: vp.height },
        deviceScaleFactor: vp.dpr,
      });
      const page = await ctx.newPage();
      await boot(page, item.capture.scene);
      await runSteps(page, item);
      await assertVisible(page, item);
      const clip = item.capture.clip;
      const file = path.join(ASSETS, `${item.id}.png`);
      if (clip) {
        // 显式 clip:精确区域(画廊格/面板等),原样。
        await page.screenshot({
          path: file,
          clip: { x: clip.x, y: clip.y, width: clip.w, height: clip.h },
        });
      } else {
        // 无 clip:整窄视口 → 自动裁掉底部空白(手机屏合身高度)。
        const shot = await page.screenshot();
        writeFileSync(file, await autoCropBottom(shot));
      }
      metaOut(item, file);
      await ctx.close();
    } else {
      // webm:独立 context recordVideo,段长 = durationMs(±编码噪声;确定性只要求脚本一致)。
      const ctx = await browser.newContext({
        viewport: { width: vp.width, height: vp.height },
        recordVideo: { dir: ASSETS, size: { width: vp.width, height: vp.height } },
      });
      const page = await ctx.newPage();
      await boot(page, item.capture.scene);
      await runSteps(page, item);
      await assertVisible(page, item);
      const video = page.video();
      await ctx.close(); // 关 context 落盘视频
      const tmp = await video!.path();
      const file = path.join(ASSETS, `${item.id}.webm`);
      copyFileSync(tmp, file);
      expect(statSync(file).size, `${item.id}: webm 非空`).toBeGreaterThan(10_000);
      metaOut(item, file);
    }
  });
}
