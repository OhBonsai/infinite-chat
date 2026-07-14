// boot.ts — Plan 25 PR-A:共享装配。canvas/wasm init + ChatCanvas 构建 + 标准每帧泵
// (overlay/复制/文本层/选区/查找/Dock)+ 字体载入。`main.ts`(实时/演示)与 `/chat`(剧本)复用,
// **不复制粘贴**。param 相关(bench/debug/msdf/demo/film/SSE)留各自入口。

import init, { ChatCanvas } from "../pkg/infinite_chat_wasm.js";
import { layout, measure, FONT_SIZE } from "./layout-bridge";
import { rasterize } from "./glyph-raster";
import { attachCanvasInput } from "./input";

export interface BootOpts {
  /** 预录事件流(replay/剧本经它进引擎的 Player 通道);无则实时/合成。 */
  replay?: { t: number; raw: string }[];
  serverUrl?: string;
  sessionId?: string;
  /** 挂 Cmd+F 查找条(默认挂)。 */
  findBar?: boolean;
  /** 揭示节奏预设(Plan 25 M1;如 "reader")。**必须在暴露 `window.__chat` 之前**应用——
   * e2e 在 __chat 出现后立刻 `set_reveal_cps(1e9)`,若预设晚于它会把 tempo 打回慢档(竞态)。 */
  rhythmPreset?: string;
  /** 字形渲染方案(Plan 40 追加):0=Auto(MSDF→TinySDF)· 1=Bitmap(Canvas 覆盖率,1:1 最锐,
   * 官网默认清爽无锯齿)· 2=ForceTinySdf · 3=ForceMsdf。GPU 后端异步就绪后才生效(见 gateGlyphMode)。 */
  glyphMode?: number;
}

/** 引擎 App 由 `start()` 异步(spawn_local)建;就绪前诸多 setter 落空。用只读 `effect_preset_name()`
 *  非空作就绪信号,轮询到位再应用字形方案(否则 set_glyph_mode 早调 no-op → 仍是默认 Auto)。 */
function gateGlyphMode(chat: ChatCanvas, mode: number, tries = 0): void {
  if (chat.effect_preset_name() !== "") {
    chat.set_glyph_mode(mode);
    return;
  }
  if (tries < 600) requestAnimationFrame(() => gateGlyphMode(chat, mode, tries + 1));
}

export interface Booted {
  chat: ChatCanvas;
  canvas: HTMLCanvasElement;
  /** wasm 模块(读 `.memory` 线性内存,?bench 用)。 */
  wasmModule: Awaited<ReturnType<typeof init>>;
}

/** 装配 canvas + 引擎 + 渲染泵 + 字体。返回句柄供入口做各自的 param 特化。 */
export async function bootCanvas(opts: BootOpts): Promise<Booted> {
  const canvas = document.getElementById("chat") as HTMLCanvasElement;
  // HiDPI:后备缓冲 = CSS 尺寸 × devicePixelRatio;排版/光栅化同按设备像素(见 layout-bridge)。
  const dpr = window.devicePixelRatio || 1;
  const cssW = canvas.clientWidth || window.innerWidth;
  const cssH = canvas.clientHeight || window.innerHeight;
  canvas.width = Math.round(cssW * dpr);
  canvas.height = Math.round(cssH * dpr);

  // 画布矩形跟踪:overlay 容器对齐画布(可非全屏)+ 容器级 resize 桥到 window resize(喂 wasm 重配)。
  {
    const { watchCanvasRect } = await import("./canvas-rect");
    watchCanvasRect(canvas);
  }

  const wasmModule = await init();

  const chat = new ChatCanvas(canvas, {
    layout,
    measure,
    rasterize,
    serverUrl: opts.serverUrl,
    sessionId: opts.sessionId,
    replay: opts.replay,
  });
  chat.set_math_em(FONT_SIZE); // 数学字号 = 正文字号(含 DPR);显示数学 ×1.3(Plan 12)
  if (opts.rhythmPreset) chat.set_reveal_preset(opts.rhythmPreset); // 先于 __chat 暴露(见 BootOpts)
  chat.start();
  if (opts.glyphMode != null) gateGlyphMode(chat, opts.glyphMode); // 就绪后切字形方案(如官网默认 Bitmap)

  // 链接打开(Plan 34 S2 / 0038 边界):引擎只回调,宿主决定怎么开(默认新标签 + noopener)。
  chat.set_open_url_handler((url: string) => {
    window.open(url, "_blank", "noopener,noreferrer");
  });

  // 画布输入(滚轮/两指滚动/捏合缩放/拖拽平移)。
  attachCanvasInput(canvas, chat);

  // 图片嵌入(Plan 14 ③):轮询待解码图 → 浏览器解码/上传(重活在 JS)。
  {
    const { pumpImageLoads } = await import("./image-loader");
    setInterval(() => pumpImageLoads(chat), 120);
  }

  // Plan 26②:landmark + 读屏播报器(live region;按状态迁移粒度播,不逐 delta)。
  // S4(Plan 34):viewport 键盘可及(tabindex=0,shadcn R13)。
  canvas.setAttribute("role", "main");
  canvas.setAttribute("aria-label", "对话画布");
  canvas.setAttribute("tabindex", "0");
  {
    const { mountAnnouncer } = await import("./announcer");
    mountAnnouncer(chat);
  }

  // S4 + S1 收尾:jump-to-latest 按钮 —— Released 才可用;Following/Anchoring 无目标 →
  // inert + 隐藏(shadcn R13:无目标时不入 tab 序)。点击走 0038 正路 scroll_to_latest。
  {
    const jump = document.createElement("button");
    jump.className = "jump-latest";
    jump.textContent = "↓ 跳到最新";
    jump.setAttribute("aria-label", "跳到最新消息");
    jump.style.cssText =
      "position:fixed;right:24px;bottom:96px;z-index:60;padding:6px 12px;border-radius:16px;" +
      "border:1px solid #333;background:#1d1d20;color:#ededed;cursor:pointer;font-size:13px;" +
      "display:none";
    jump.addEventListener("click", () => chat.scroll_to_latest());
    document.body.appendChild(jump);
    const sync = () => {
      const released = chat.follow_state() === "released";
      jump.style.display = released ? "" : "none";
      jump.toggleAttribute("inert", !released);
      jump.tabIndex = released ? 0 : -1;
      requestAnimationFrame(sync);
    };
    requestAnimationFrame(sync);
  }

  // 每帧泵:动图 overlay(Plan 14)+ 复制按钮(Plan 21)+ 文本层/选区(Plan 21)+ Dock(Plan 22)。
  {
    const { pumpEmbedOverlay } = await import("./embed-overlay");
    const { pumpCopyButtons } = await import("./copy-button");
    const { pumpTextLayer, attachSelection } = await import("./text-layer");
    const { pumpAskOverlay } = await import("./ask-overlay");
    attachSelection(chat);
    if (opts.findBar !== false) {
      const { mountFindBar } = await import("./find-bar");
      mountFindBar(chat);
    }
    const tick = () => {
      pumpEmbedOverlay(chat);
      pumpCopyButtons(chat);
      pumpTextLayer(chat, canvas);
      pumpAskOverlay(chat); // Plan 27 B 路:pending question 的锚定表单(permission 走画布 tap)
      requestAnimationFrame(tick);
    };
    requestAnimationFrame(tick);
  }

  // 保活:挂 window,避免 chat 被 GC(否则帧循环/监听悬空)。
  (window as unknown as { __chat: unknown }).__chat = chat;

  // 数学字体(Plan 12):异步预载(非阻塞)→ 就绪后 refresh_fonts 让已现公式重栅。
  {
    const { loadMathMsdf } = await import("./msdf");
    const { loadMathFonts } = await import("./math-fonts");
    Promise.all([
      loadMathMsdf(chat).catch((e) => console.error("[math-msdf] load failed", e)),
      loadMathFonts().catch((e) => console.error("[math-fonts] load failed", e)),
    ]).then(() => chat.refresh_fonts());
  }

  return { chat, canvas, wasmModule };
}

// —— Plan 40 P2:hero 轻量装配(landing 背景 cinematic)——
// 只挂 canvas + 引擎 + 重放,**无 overlay/输入/nav**(hero 是背景氛围,零交互)。canvasId 可配。
export interface HeroBooted {
  chat: ChatCanvas;
  ok: boolean; // WebGPU/WebGL2 是否起来(否则调用方走静帧降级)
}
export async function bootHero(opts: {
  canvasId: string;
  rhythmPreset?: string;
  glyphMode?: number;
}): Promise<HeroBooted> {
  const canvas = document.getElementById(opts.canvasId) as HTMLCanvasElement | null;
  if (!canvas) return { chat: null as unknown as ChatCanvas, ok: false };
  const dpr = window.devicePixelRatio || 1;
  const cssW = canvas.clientWidth || window.innerWidth;
  const cssH = canvas.clientHeight || 480;
  canvas.width = Math.round(cssW * dpr);
  canvas.height = Math.round(cssH * dpr);
  try {
    const { watchCanvasRect } = await import("./canvas-rect");
    watchCanvasRect(canvas);
    await init();
    // 空 replay = 静默 Player 连接(否则无 replay/server 会落到合成演示流「你好!」,污染 hero 内容);
    // hero 文案全靠调用方 push_event 注入(同 chat/main.ts 注释)。
    const chat = new ChatCanvas(canvas, { layout, measure, rasterize, replay: [] });
    chat.set_math_em(FONT_SIZE);
    if (opts.rhythmPreset) chat.set_reveal_preset(opts.rhythmPreset);
    chat.start();
    if (opts.glyphMode != null) gateGlyphMode(chat, opts.glyphMode); // 就绪后切字形方案(官网默认 Bitmap)
    (window as unknown as { __hero: unknown }).__hero = chat; // 调试/烟测句柄
    return { chat, ok: true };
  } catch (e) {
    console.warn("[hero] 引擎起不来(无 WebGPU/WebGL2?)→ 静帧降级", e);
    return { chat: null as unknown as ChatCanvas, ok: false };
  }
}
