// home.ts — Plan 40 landing 页入口(P2:hero cinematic 引擎场景 + 导航 + 入口路由 +
// 滚动淡入 + 页脚;P3 填功能全览卡片)。纯 DOM 胶水。
import "./pages-theme.css";
import { mountNav } from "./pages-nav";
import { bootHero } from "../boot";

const base = import.meta.env.BASE_URL;
mountNav("home");

// 入口卡路由(base 子路径全通)。
const setHref = (id: string, path: string) => {
  const el = document.getElementById(id) as HTMLAnchorElement | null;
  if (el) el.href = base + path;
};
setHref("e-chat", "chat/");
setHref("e-md", "markdown/");
setHref("e-gallery", "gallery.html");

// 滚动字幕式淡入(LoL 风;reduced-motion 由 CSS 直接显示)。
const io = new IntersectionObserver(
  (entries) => {
    for (const e of entries) if (e.isIntersecting) e.target.classList.add("p-in");
  },
  { threshold: 0.12 },
);
for (const el of document.querySelectorAll(".p-reveal")) io.observe(el);

// —— P3:功能全览区 —— 九大项锚点分区 × plan39 精选卡(截图/短片,pages-assets 入库)。
// 卡片鎏金描边 + hover glow;媒体懒加载(png loading=lazy;webm 进视口才 play,离开即 pause 省电)。
interface FeatCard {
  id: string;
  group: string;
  kind: "png" | "webm";
  title: string;
  desc: string;
  file: string;
}
interface FeatSection {
  id: string;
  title: string;
  desc: string;
  cards: FeatCard[];
}

// webm 可视即播、离开即停(省电 + 不抢 hero 引擎帧)。
const videoIO = new IntersectionObserver(
  (entries) => {
    for (const e of entries) {
      const v = e.target as HTMLVideoElement;
      if (e.isIntersecting) void v.play().catch(() => {});
      else v.pause();
    }
  },
  { threshold: 0.25 },
);

function cardMedia(base: string, c: FeatCard): HTMLElement {
  const src = `${base}pages-assets/${c.file}`;
  if (c.kind === "webm") {
    const v = document.createElement("video");
    v.src = src;
    v.muted = true;
    v.loop = true;
    v.playsInline = true;
    v.preload = "metadata"; // 首帧作静态封面;真播由 videoIO 触发
    v.setAttribute("aria-label", c.title);
    v.className = "feat-media";
    videoIO.observe(v);
    return v;
  }
  const img = document.createElement("img");
  img.src = src;
  img.loading = "lazy";
  img.decoding = "async";
  img.alt = c.title;
  img.className = "feat-media";
  return img;
}

async function renderFeatures(): Promise<void> {
  const host = document.getElementById("features");
  if (!host) return;
  let data: { sections: FeatSection[] };
  try {
    const r = await fetch(`${base}pages-assets/index.json`);
    if (!r.ok) throw new Error(`HTTP ${r.status}`);
    data = await r.json();
  } catch (e) {
    console.warn("[features] 精选清单载入失败(未跑 gen-pages-assets?)", e);
    return; // 无资产:全览区留空(eyebrow 仍在),不阻塞其它区块
  }
  const frag = document.createDocumentFragment();
  for (const sec of data.sections) {
    const section = document.createElement("section");
    section.className = "feat-sec p-reveal";
    section.id = `feat-${sec.id}`;
    const h = document.createElement("h2");
    h.className = "feat-h p-h2";
    h.textContent = sec.title;
    const d = document.createElement("p");
    d.className = "feat-secdesc p-cjk";
    d.textContent = sec.desc;
    const grid = document.createElement("div");
    grid.className = "feat-grid";
    for (const c of sec.cards) {
      const card = document.createElement("figure");
      card.className = "feat-card p-card";
      const frame = document.createElement("div");
      frame.className = "feat-frame";
      frame.appendChild(cardMedia(base, c));
      const cap = document.createElement("figcaption");
      cap.className = "feat-cap";
      const ct = document.createElement("h3");
      ct.textContent = c.title;
      const cd = document.createElement("p");
      cd.className = "p-cjk";
      cd.textContent = c.desc;
      cap.append(ct, cd);
      card.append(frame, cap);
      grid.appendChild(card);
    }
    section.append(h, d, grid);
    frag.appendChild(section);
    io.observe(section); // 滚动淡入(动态节点须补观察)
  }
  host.appendChild(frag);
}
void renderFeatures();

// 页脚版本(构建期无 git,用 BASE_URL 提示环境;真 commit 由 CI 注入,P6 再接)。
const foot = document.getElementById("foot-ver");
if (foot) foot.textContent = `infinite-chat · ${base === "/" ? "local" : "pages"}`;

// —— P2:hero cinematic —— 「引擎即宣传片」。canvas 跑一段富文本流式揭示作氛围背景
// (expressive 效果预设 = dissolve/glow/trail),CSS 前景标题叠在上层;慢推镜(dolly)与
// 底部渐隐 mask 都在 index.html 的 CSS 里(reduced-motion 由 CSS 关)。此处只管:装引擎、
// 喂内容、循环重播、页签隐藏时暂停;无 GPU/引擎起不来 → 静帧降级(CSS 渐变已在,补提示)。
const reduce = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
// hero 背景文案:短、有节奏,展示富渲染(标题/强调/行内码);双语克制,呼应 Salvation 台词感。
const HERO_STREAM = [
  "# 每一个 token,都是一次入场\n\n",
  "流式吐字不是打印——是**揭示**。\n\n",
  "字符带着 `dissolve` 与微光落定,",
  "像尘埃在光里缓缓成形。\n\n",
  "> 引擎在跑,你看到的就是宣传片本身。\n",
].join("");

function mountHeroFallback(msg: string): void {
  const fb = document.getElementById("hero-fallback");
  if (!fb) return;
  fb.style.display = "flex";
  fb.innerHTML =
    `<div class="hero-fb-inner"><span class="hero-fb-dot"></span>${msg}</div>`;
}

async function bootHeroScene(): Promise<void> {
  const canvas = document.getElementById("hero-canvas") as HTMLCanvasElement | null;
  if (!canvas) return;
  const { chat, ok } = await bootHero({ canvasId: "hero-canvas", rhythmPreset: "reader" });
  if (!ok || !chat) {
    // 引擎起不来(无 WebGPU/WebGL2):canvas 隐藏,留 CSS 渐变 + 一行提示。
    canvas.style.display = "none";
    mountHeroFallback("此浏览器无法启动 WebGPU/WebGL2 —— 展示静态封面。用 Chrome 打开可看引擎实时演绎。");
    return;
  }
  // 喂内容 = 两步事件序列(先建 message 再填 part,同 chats/mini.json 真实 opencode 事件形状):
  // 光 part.updated 不会建 turn —— 引擎需要 message.updated 先立起 role/sessionID。
  const HERO_SID = "hero";
  const HERO_MID = "hero-msg";
  const feed = () => {
    chat.push_event(
      JSON.stringify({
        type: "message.updated",
        properties: { info: { id: HERO_MID, role: "assistant", sessionID: HERO_SID } },
      }),
    );
    chat.push_event(
      JSON.stringify({
        type: "message.part.updated",
        properties: {
          part: {
            type: "text",
            id: "hero-part",
            messageID: HERO_MID,
            sessionID: HERO_SID,
            text: HERO_STREAM,
          },
          time: 1,
        },
      }),
    );
  };

  // 引擎就绪门:start() 把 GPU/后端初始化 spawn 成异步任务,App 未建好前 set_effect_preset
  // 返回 false(state 尚空 → 一切调用皆 no-op)。轮询到预设命中(App 就绪)再喂内容 + 起循环,
  // 否则内容落空(曾致 hero 全黑)。
  const startWhenReady = (tries = 0): void => {
    if (!chat.set_effect_preset("expressive")) {
      if (tries < 600) requestAnimationFrame(() => startWhenReady(tries + 1));
      return;
    }
    feed();
    if (reduce) return; // 减少动效:揭示一次即定帧,不循环、不推镜(CSS 已关 dolly)。

    // 循环重播:每 ~9s 重跑揭示动画(内容不变,只重放 dissolve/glow 的入场)。
    const LOOP_MS = 9000;
    let timer = window.setInterval(() => {
      if (!document.hidden) chat.restart_reveal();
    }, LOOP_MS);

    // 页签隐藏 → 暂停引擎帧泵(省电,回来即恢复);同时停/续重播定时器。
    document.addEventListener("visibilitychange", () => {
      chat.set_paused(document.hidden);
      if (document.hidden) {
        window.clearInterval(timer);
        timer = 0;
      } else if (!timer) {
        chat.restart_reveal();
        timer = window.setInterval(() => {
          if (!document.hidden) chat.restart_reveal();
        }, LOOP_MS);
      }
    });
  };
  startWhenReady();
}
void bootHeroScene();
