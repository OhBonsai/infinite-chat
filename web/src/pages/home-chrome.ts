// home-chrome.ts — Plan 41 首页播放器 chrome:幕进度点 × 幕数 + 播放/暂停。
// 纯 DOM,取值全走 pages-theme token(样式在 index.html)。返回 update() 供 home.ts 按幕驱动。
import type { SceneMark } from "../film/director";
import type { HomePlayer } from "./home-player";

export interface HomeChrome {
  update: (sceneIdx: number) => void;
  setPaused: (paused: boolean) => void;
}

export function mountHomeChrome(container: HTMLElement, marks: SceneMark[], player: HomePlayer): HomeChrome {
  const inner = document.createElement("div");
  inner.className = "chrome-inner";

  const pp = document.createElement("button");
  pp.className = "pp";
  pp.setAttribute("aria-label", "播放/暂停");
  pp.textContent = "⏸";
  pp.addEventListener("click", () => player.togglePause());

  const dotsWrap = document.createElement("div");
  dotsWrap.className = "dots";
  const dots: HTMLElement[] = [];
  marks.forEach((m, i) => {
    const d = document.createElement("button");
    d.className = "dot";
    d.setAttribute("aria-label", `第 ${i + 1} 幕:${m.title}`);
    d.title = m.title;
    d.addEventListener("click", () => player.gotoScene(i));
    dots.push(d);
    dotsWrap.appendChild(d);
  });

  inner.append(pp, dotsWrap);
  container.appendChild(inner);

  return {
    update: (sceneIdx: number) => {
      dots.forEach((d, i) => d.classList.toggle("on", i === sceneIdx));
    },
    setPaused: (paused: boolean) => {
      pp.textContent = paused ? "▶" : "⏸";
    },
  };
}
