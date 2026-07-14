// home.ts — Plan 40 landing 页入口(P1 骨架:导航 + 入口路由 + 滚动淡入 + 页脚;
// P2 填 hero cinematic 引擎场景,P3 填功能全览卡片)。纯 DOM 胶水。
import "./pages-theme.css";
import { mountNav } from "./pages-nav";

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

// 页脚版本(构建期无 git,用 BASE_URL 提示环境;真 commit 由 CI 注入,P6 再接)。
const foot = document.getElementById("foot-ver");
if (foot) foot.textContent = `infinite-chat · ${base === "/" ? "local" : "pages"}`;

// P2:hero 引擎 cinematic 场景在此挂载(先占位:canvas 保持渐变背景 = CSS 已给)。
