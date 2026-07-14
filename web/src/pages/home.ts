// home.ts — Plan 41 首页 = 全屏单 canvas 幕式播放器(引擎即宣传片)。薄壳编排:
// 装引擎(全屏 #home-canvas)→ 载母文档 → FilmDirector(七幕)+ HomePlayer(自动播/接管/循环)
// + chrome(进度点)+ 字幕层 + S7 入口浮层 + 降级(reduced-motion / 无 GPU H3)。
import "./pages-theme.css";
import { bootHero } from "../boot";
import { FilmDirector } from "../film/director";
import { HOME_SCENES, SUBTITLES, buildMasterDoc, type HomeSub } from "./home-scenes";
import { HomePlayer } from "./home-player";
import { mountHomeChrome } from "./home-chrome";

const base = import.meta.env.BASE_URL;
const reduce = window.matchMedia("(prefers-reduced-motion: reduce)").matches;

// 深链 + 品牌回首页(base 子路径全通)。
const setHref = (id: string, path: string): void => {
  const el = document.getElementById(id) as HTMLAnchorElement | null;
  if (el) el.href = base + path;
};
setHref("nav-brand", "");
setHref("o-chat", "chat/");
setHref("o-md", "markdown/");
setHref("o-gallery", "gallery.html");
const foot = document.getElementById("home-foot-ver");
if (foot) foot.textContent = `infinite-chat · ${base === "/" ? "local" : "pages"}`;

const subtitleEl = document.getElementById("home-subtitle") as HTMLElement;
const outroEl = document.getElementById("home-outro") as HTMLElement;
const playerEl = document.getElementById("home-player") as HTMLElement;

function renderSub(sub: HomeSub): string {
  const parts: string[] = [];
  if (sub.eyebrow) parts.push(`<div class="sub-eyebrow">${sub.eyebrow}</div>`);
  if (sub.title) parts.push(`<h2 class="sub-title">${sub.title}</h2>`);
  if (sub.note) parts.push(`<p class="sub-note">${sub.note}</p>`);
  if (sub.link) parts.push(`<p class="sub-note"><a class="sub-link" href="${base + sub.link.href}">${sub.link.text}</a></p>`);
  return parts.join("");
}

// 按幕 index 驱动 DOM(字幕/dolly/outro);只在幕变时改。
let prevIdx = -1;
function onScene(idx: number): void {
  if (idx === prevIdx) return;
  prevIdx = idx;
  const scene = HOME_SCENES[idx];
  const sub = SUBTITLES[scene.id] ?? {};
  const isOutro = scene.id === "outro";
  playerEl.classList.toggle("dolly", scene.id === "title");
  outroEl.classList.toggle("show", isOutro);
  if (isOutro || (!sub.eyebrow && !sub.title && !sub.note)) {
    subtitleEl.classList.remove("show");
  } else {
    subtitleEl.innerHTML = renderSub(sub);
    subtitleEl.classList.add("show");
  }
}

// 首屏 wasm 空窗:S1 字幕先行(引擎就绪前先显标题),引擎接管后照常。
onScene(0);

async function main(): Promise<void> {
  const canvas = document.getElementById("home-canvas") as HTMLCanvasElement | null;
  if (!canvas) return;
  const { chat, ok } = await bootHero({ canvasId: "home-canvas", rhythmPreset: "reader", glyphMode: 1 });
  if (!ok || !chat) {
    // 无 GPU:H3 填幻灯降级;此处先保底 —— canvas 隐藏,字幕 + 入口浮层仍在(静态封面)。
    canvas.style.display = "none";
    outroEl.classList.add("show");
    return;
  }

  // 引擎就绪门(start() 异步建 App;未就绪前 push_event/preset 落空)。就绪后载母文档 + 起播放器。
  const whenReady = (tries = 0): void => {
    const nameFn = (chat as unknown as { effect_preset_name?: () => string }).effect_preset_name;
    if (!nameFn || nameFn.call(chat) === "") {
      if (tries < 600) requestAnimationFrame(() => whenReady(tries + 1));
      return;
    }
    buildMasterDoc(chat);

    const noopTeaser = { show: () => {}, hide: () => {} };
    const dir = new FilmDirector(HOME_SCENES, chat as never, noopTeaser);
    const player = new HomePlayer(dir, { auto: !reduce, idleMs: 30_000 });
    const chrome = mountHomeChrome(document.getElementById("home-chrome") as HTMLElement, dir.marks, player);

    dir.onUpdate = (info) => {
      onScene(info.sceneIdx);
      chrome.update(info.sceneIdx);
    };
    player.onState = (s) => chrome.setPaused(s.paused);

    // 隐藏页冻引擎(rAF 本就被节流;显式 set_paused 保正确)。
    document.addEventListener("visibilitychange", () => {
      (chat as unknown as { set_paused?: (b: boolean) => void }).set_paused?.(
        document.hidden || player.isPaused(),
      );
    });

    player.start();
  };
  whenReady();
}

void main();
