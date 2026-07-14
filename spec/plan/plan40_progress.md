# Plan 40 · 进度账本(GitHub Pages cinematic 官网)

> 逐 milestone 记录:风格参考依据 / 资产清单与体积 / 降级验证 / 遗留。
> GOAL:[plan40-github-pages-cinematic-goal](./plan40-github-pages-cinematic-goal.md)

## P0 · 风格参考包(lol-style tokens)

见下

### P0 记账
- 详见本节

## P1 · 站点骨架 + 导航(四页 IA)

- **四页 IA**:新 `index.html` = landing(hero + 功能全览占位 + 四入口卡 + 页脚);
  原开发 harness `git mv index.html → dev.html`(`?dev` 用途,不上导航);
  `markdown/index.html` 新建(canvas + playground textarea 骨架);chat 页加导航。
- **共享导航**(`web/src/pages/pages-nav.ts`):四页 + GitHub,金线当前态(aria-current);
  base 走 import.meta.env.BASE_URL(PAGES_BASE 子路径全通)。gallery(静态生成页)在
  `gen-gallery.mjs` 注入内联 nav bar(同风格,重生成 public/gallery.html)。
- **vite 多入口**:input += markdown + dev(landing/chat/markdown/dev 四入口)。
- **入口 ts**:home.ts(nav + 入口路由 + 滚动淡入 IntersectionObserver + 页脚)、markdown.ts
  骨架;各页 import pages-theme.css。
- **e2e 迁移(必要连带)**:index.html 从 harness 变 landing → 19 处 `goto("/?...")` +
  feature SCENE_URL 批量改指 `/dev.html?...`(harness 入口迁移);chat-full 走 /chat/ 不变。
- **验证**:PAGES_BASE build 产 index/chat/markdown/dev/gallery 五页;preview 子路径四页
  全 200;landing 目视 = LoL 气质(深蓝黑 + 鎏金衬线大写标题 + 金线导航)。

## P2 · hero cinematic(引擎即宣传片)

(未开始)

## P3 · 功能全览区(pages-assets 精选)

(未开始)

## P4 · chat 页(完整对话 + 重放控制条)

(未开始)

## P5 · markdown 页(轮播 + playground)

(未开始)

## P6 · 收口(smoke / 降级 / 体积 / 文档)

(未开始)

## DoD 对账

(未开始)
