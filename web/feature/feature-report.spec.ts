// feature-report.spec.ts — Plan 39 捕获 harness:逐 manifest 项拍照/录像 + 可见性断言。
// 铁律(GOAL §0):报告即测试 —— 任一项捕获/断言失败 = 该功能红(fail loud,不出残缺报告)。
// 产物:test/results/feature-assets/<id>.png|webm(+ .json 元数据);FEATURE_ONLY=<group|id> 增量。
import { expect, test, chromium, type Browser, type Page } from "@playwright/test";
import { copyFileSync, mkdirSync, readFileSync, statSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

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
  showcase: "/?replay=showcase&noinput",
  "chat-full": "/chat/?script=showcase-full&speed=8",
  empty: "/?empty&noinput",
  gallery: "/?empty&noinput&gallery",
  debug: "/?replay=showcase&noinput&debug",
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

for (const item of picked) {
  test(`${item.group}/${item.id} · ${item.title}`, async ({ browser }) => {
    if (item.kind === "png") {
      const dpr = item.capture.dpr ?? 1;
      const ctx = await browser.newContext({
        viewport: { width: 1280, height: 900 },
        deviceScaleFactor: dpr,
      });
      const page = await ctx.newPage();
      await boot(page, item.capture.scene);
      await runSteps(page, item);
      await assertVisible(page, item);
      const clip = item.capture.clip;
      const file = path.join(ASSETS, `${item.id}.png`);
      await page.screenshot({
        path: file,
        ...(clip ? { clip: { x: clip.x, y: clip.y, width: clip.w, height: clip.h } } : {}),
      });
      metaOut(item, file);
      await ctx.close();
    } else {
      // webm:独立 context recordVideo,段长 = durationMs(±编码噪声;确定性只要求脚本一致)。
      const ctx = await browser.newContext({
        viewport: { width: 1280, height: 720 },
        recordVideo: { dir: ASSETS, size: { width: 1280, height: 720 } },
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
