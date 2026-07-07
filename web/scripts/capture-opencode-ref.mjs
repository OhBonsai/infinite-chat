// Plan 28 R0:参考真值捕获 —— 驱动 opencode storybook Timeline Playground(真实 SessionTurn +
// 官方夹具),逐场景截图 + getComputedStyle dump。
//
// 前置:opencode storybook dev server 在 :6006(`cd ~/w/agentscode/opencode/packages/storybook
// && bun run storybook`)。运行:`node web/scripts/capture-opencode-ref.mjs`(repo 根)。
// 产物:spec/reference/opencode-ui/shots/<scene>.png + computed/<scene>.json。
// 主题:dark(plan28 基准决策;storybook 全局 `theme`)。视口 1280×900 dpr=1;动画禁用后截 settled 帧。
import { chromium } from "@playwright/test";
import { mkdirSync, writeFileSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "../..");
const REF = join(ROOT, "spec/reference/opencode-ui");
const URL = "http://localhost:6006/iframe.html?id=playground-timeline--basic&globals=theme:dark";

// 场景 = 按钮点击序列(按钮文案来自 timeline-playground.stories.tsx 的 USER_VARIANTS label /
// MARKDOWN_SAMPLES 键 / TOOL_SAMPLES 键 / 复合按钮)。对齐我方 design-review S1–S10。
const SCENES = [
  { name: "s1-user-variants", clicks: ["short", "medium", "long"] },
  { name: "s2-markdown", clicks: ["short", "headings", "lists", "code", "table", "mixed"] },
  { name: "s3-reasoning", clicks: ["full turn"] },
  { name: "s4-tools", clicks: ["short", "bash", "edit", "write", "task", "webfetch", "websearch", "skill", "todowrite"] },
  { name: "s5-context-group", clicks: ["context group"] },
  { name: "s6-question", clicks: ["short", "question"] },
  { name: "s7-read-single", clicks: ["short", "read"] },
  { name: "s9-kitchen-sink", clicks: ["kitchen sink"] },
];

// dump 的样式属性(§3:盒模型 + 排版 + 配色 + 圆角描边)。
const PROPS = [
  "display", "paddingTop", "paddingRight", "paddingBottom", "paddingLeft",
  "marginTop", "marginRight", "marginBottom", "marginLeft", "gap", "rowGap", "columnGap",
  "fontFamily", "fontSize", "fontWeight", "lineHeight", "letterSpacing",
  "color", "backgroundColor", "borderColor", "borderWidth", "borderRadius",
  "maxWidth", "width", "height", "opacity", "boxShadow", "textAlign", "alignSelf",
];

const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1280, height: 900 }, deviceScaleFactor: 1 });
page.on("pageerror", (e) => console.log("  [pageerror]", e.message.slice(0, 200)));
mkdirSync(join(REF, "shots"), { recursive: true });
mkdirSync(join(REF, "computed"), { recursive: true });

// 预热(vite dev 冷启动按需编译大量模块,首载可能超时出错误浮层)+ live 全局变量 dump:
// 以静态解析的变量名清单逐一 getPropertyValue —— class-based dark 的实际值可能异于
// media-query 块(实测 --background-base:#121212 vs 静态 #101010),live 为准。
await page.goto(URL, { waitUntil: "networkidle" });
await page.waitForTimeout(5000);
{
  const { readFileSync } = await import("node:fs");
  const names = Object.keys(JSON.parse(readFileSync(join(REF, "tokens.global.json"), "utf8")).light);
  const live = await page.evaluate((ns) => {
    const cs = getComputedStyle(document.documentElement);
    const out = {};
    for (const n of ns) {
      const v = cs.getPropertyValue(n).trim();
      if (v) out[n] = v;
    }
    return { scheme: document.documentElement.className, vars: out };
  }, names);
  writeFileSync(join(REF, "tokens.live-dark.json"), JSON.stringify(live, null, 1));
  console.log(`live vars (${live.scheme}): ${Object.keys(live.vars).length}`);
}

for (const scene of SCENES) {
  await page.goto(URL, { waitUntil: "networkidle" });
  await page.waitForTimeout(1200); // 热身后仍留渲染余量
  // 动画全禁(shimmer/spinner/transition)→ settled 帧确定。
  await page.addStyleTag({
    content: "*,*::before,*::after{animation:none!important;transition:none!important}",
  });
  for (const label of scene.clicks) {
    const btn = page.getByRole("button", { name: label, exact: true }).first();
    await btn.click({ timeout: 8000 });
    await page.waitForTimeout(150); // Solid 渲染 + 懒挂载(defer rAF)
    if (await page.locator(".sb-errordisplay").isVisible().catch(() => false)) {
      console.log(`  [overlay] after click "${label}" in ${scene.name}`);
    }
  }
  await page.waitForTimeout(800); // PacedMarkdown 揭示完
  // 屏外内容不渲染(content-visibility:auto + defer 懒挂载)→ 逐块滚入触发挂载,再强制可见。
  await page.addStyleTag({ content: "*{content-visibility:visible!important}" });
  await page.evaluate(async () => {
    const blocks = document.querySelectorAll('[data-component="assistant-message"],[data-component="user-message"]');
    for (const el of blocks) {
      el.scrollIntoView({ block: "center" });
      await new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r)));
    }
    window.scrollTo(0, 0);
  });
  await page.waitForTimeout(1200); // 懒挂载 rAF 链 + 图片/字体
  // 截时间线列。高于视口的场景不能用元素拼接截图(defer/content-visibility 屏外区域会空白)——
  // 改视口切片:滚动逐屏截真像素(shots/<scene>/000.png…);短场景仍单张。
  const list = page.locator('[data-slot="session-turn-list"]');
  // 真滚动容器 = 主预览区(flex:1 overflow:auto,timeline-playground.stories.tsx:1964)。
  const scroller = list.locator("xpath=ancestor::div[contains(@style,'overflow')][1]");
  const total = await list.evaluate((el) => el.scrollHeight);
  const vh = 900;
  if (total <= vh + 100) {
    await list.screenshot({ path: join(REF, "shots", `${scene.name}.png`) });
  } else {
    mkdirSync(join(REF, "shots", scene.name), { recursive: true });
    const step = vh - 100; // 100px 重叠防切字
    const steps = Math.ceil(total / step);
    for (let i = 0; i < steps; i++) {
      await scroller.evaluate((el, y) => el.scrollTo(0, y), i * step);
      await page.waitForTimeout(400); // 懒挂载
      const box = await list.boundingBox();
      await page.screenshot({
        path: join(REF, "shots", scene.name, `${String(i).padStart(3, "0")}.png`),
        clip: { x: box.x, y: 0, width: box.width, height: vh },
      });
    }
  }

  // computedStyle dump:时间线内全部 data-component/data-slot 节点。
  const dump = await list.evaluate((rootEl, props) => {
    const listRect = rootEl.getBoundingClientRect();
    const out = [];
    for (const el of rootEl.querySelectorAll("[data-component],[data-slot]")) {
      const r = el.getBoundingClientRect();
      if (r.width === 0 && r.height === 0) continue;
      const cs = getComputedStyle(el);
      const style = {};
      for (const p of props) style[p] = cs[p];
      out.push({
        component: el.getAttribute("data-component") ?? undefined,
        slot: el.getAttribute("data-slot") ?? undefined,
        // 面向还原:相对时间线列左上角的盒(参考侧滚动无关)。
        rect: {
          x: +(r.x - listRect.x).toFixed(1),
          y: +(r.y - listRect.y).toFixed(1),
          w: +r.width.toFixed(1),
          h: +r.height.toFixed(1),
        },
        text: (el.textContent ?? "").slice(0, 60),
        style,
      });
    }
    return { listWidth: listRect.width, nodes: out };
  }, PROPS);
  writeFileSync(join(REF, "computed", `${scene.name}.json`), JSON.stringify(dump, null, 1));
  console.log(`${scene.name}: ${dump.nodes.length} nodes, list ${dump.listWidth}px`);
}

await browser.close();
console.log("done →", REF);
