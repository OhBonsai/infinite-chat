// pages-nav.ts — Plan 40 官网四页共享导航(landing/chat/markdown/gallery + GitHub)。
// 纯 DOM 胶水(无引擎/业务逻辑,BR 精神):注入顶栏,金线标当前态。base 走 import.meta.env.BASE_URL
// (PAGES_BASE 子路径全通)。样式来自 pages-theme.css(.p-nav)。

type Page = "home" | "chat" | "markdown" | "gallery";

interface NavItem {
  key: Page;
  label: string;
  href: string; // 相对 BASE_URL 的路径
  external?: boolean;
}

const ITEMS: NavItem[] = [
  { key: "home", label: "首页", href: "" },
  { key: "chat", label: "对话", href: "chat/" },
  { key: "markdown", label: "Markdown", href: "markdown/" },
  { key: "gallery", label: "画廊", href: "gallery.html" },
];

/** 注入导航到页面顶部(prepend 到 body 或指定容器)。current 标当前页金线态。 */
export function mountNav(current: Page, container: HTMLElement = document.body): void {
  const base = import.meta.env.BASE_URL; // "/" 或 "/infinite-chat/"
  const nav = document.createElement("nav");
  nav.className = "p-nav";

  const brand = document.createElement("a");
  brand.className = "p-brand";
  brand.href = base;
  brand.textContent = "INFINITE-CHAT";
  nav.appendChild(brand);

  for (const it of ITEMS) {
    const a = document.createElement("a");
    a.href = base + it.href;
    a.textContent = it.label;
    if (it.key === current) a.setAttribute("aria-current", "page");
    nav.appendChild(a);
  }

  const gh = document.createElement("a");
  gh.href = "https://github.com/OhBonsai/infinite-chat";
  gh.textContent = "GitHub";
  gh.target = "_blank";
  gh.rel = "noopener";
  nav.appendChild(gh);

  container.prepend(nav);
}
