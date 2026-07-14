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

- **轻量装配 `bootHero`(boot.ts,additive)**:只挂 canvas + wasm init + ChatCanvas +
  `chat.start()`,**无 overlay/输入/nav/find/announcer**(hero 是零交互氛围背景)。canvasId 可配;
  `replay: []` = 静默 Player(否则落合成演示流「你好!」污染 hero);挂 `window.__hero` 调试/烟测句柄;
  WebGPU/WebGL2 起不来 → try/catch 返回 `ok:false` 供调用方走静帧降级。
- **home.ts hero 场景**:`bootHero("hero-canvas", rhythmPreset:"reader")` → `set_effect_preset
  ("expressive")`(dissolve/glow/trail)→ 喂一段富文本导览流(标题/强调/行内码/引用,双语克制,
  呼应 Salvation 台词感)。喂内容 = **两步事件序列**(`message.updated` 建 assistant 消息 →
  `message.part.updated` 填 text part,含 role/sessionID,同 chats/mini.json 真实 opencode 形状;
  光 part.updated 不建 turn)。循环:每 9s `restart_reveal()` 重放揭示;`visibilitychange` →
  `set_paused` 暂停帧泵 + 停/续重播定时器。reduced-motion:喂一次即定帧,不循环。
- **就绪门(关键坑)**:`chat.start()` 把 GPU/后端初始化 `spawn_local` 成异步任务 —— App 未建好前
  `set_effect_preset` 返回 false(state 空,一切调用 no-op),立即喂内容会**全部落空**(hero 全黑)。
  修:`startWhenReady` 轮询 `set_effect_preset("expressive")` 命中(App 就绪)再喂内容 + 起循环。
- **hero CSS(index.html)**:canvas 作氛围背景 —— opacity 0.9 + `transform-origin 50% 18%` +
  `hero-dolly` 慢推镜(scale 1.0→1.06,var(--t-push)=4.5s ease alternate 循环)+ 顶实底渐隐 mask
  (让前景 CSS 标题突出);`.hero::after` 极淡径向金晕 + 上下压暗叠层增纵深(纯 CSS,零 Riot 资产);
  `prefers-reduced-motion` 关 animation/transform;无 GPU → canvas 隐藏 + `.hero-fallback` 金点提示。
- **验证**:headless Chromium(dpr2,WebGPU 参数)烟测 —— 自动路径 `effect_preset_name()=expressive`
  + `visible_turns()` 含 hero turn(标题/强调/dissolve 行内码/引用四行真渲染)。目视合成:top-left 引擎
  实时富文本流(rich markdown 真渲染)+ 居中鎏金 Cinzel 大标题 + eyebrow + 副标题 = cinematic 一体。
  **踩坑**:自动化 tab 后台 → `document.hidden=true` → rAF 全冻(3s 内 0 帧),画布不可在隐藏 tab 目视,
  须用 Playwright 可见页(visibilityState=visible)烟测。

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
