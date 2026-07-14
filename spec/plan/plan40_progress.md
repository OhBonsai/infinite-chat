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

- **精选标记(manifest)**:feature-manifest.json 给 13 项加 `pages: true` —— 九大项全覆盖
  (streaming×2 · markdown×1 · structure×1 · cards×2 · canvas×1 · interaction×1 · effects×3 ·
  theme×1 · reliability×1),webm 4 段(≤6 预算)。markdown 从 3 项(md-codeblock/table/math,
  同一 showcase 全幅截图 → 三卡雷同)收敛为 1 张「Markdown 全类型」整幅长卡(行内/Alert/列表/
  代码/表格/公式一屏滚完),用可选 `pagesTitle`/`pagesDesc` 覆盖体检项窄标题。
- **精选原则(踩坑)**:卡片整幅呈现截图 → 挑**视觉可辨**项。showcase 全幅项彼此雷同(都是同一
  文档);empty 场景项(tool-states/diff/shimmer/fx-*)push 独立内容 → 各异,最强。**debug 面板
  clip 项(reveal-presets/theme-panel)体检里是折叠态 → 几乎空白**,换成有内容的
  thinking-shimmer(工具卡流光)/ theme-tokens(改色整幅文档)。
- **导出脚本 `scripts/gen-pages-assets.mjs`(单命令)**:`FEATURE_PAGES=1` → feature-report.spec
  只捕 pages:true 精选(spec 加此过滤 + ManifestItem.pages?)→ 拷 test/results/feature-assets →
  清空重建 web/public/pages-assets/ → 写 index.json(九区有序 × 卡片,pagesTitle/Desc 覆盖)→
  体积对账(≤15MB,超则非 0)。`--copy-only` 跳捕获调卡片。**CI 无 GPU → 资产本地生成入库**。
  实测 13 卡 / 9 区 / **3.18MB**(fx-post 全幅 1.69MB 最大)。
- **landing 渲染(home.ts + index.html CSS)**:fetch pages-assets/index.json → 九锚点分区
  (`#feat-<group>`)× 鎏金描边卡(hover:金边 + 上浮 + glow)。**masonry(CSS columns)整幅呈现**
  (不裁成同质缩略,展示引擎真实输出);png `loading=lazy`;**webm 进视口才 play、离开即 pause**
  (videoIO,省电 + 不抢 hero 引擎帧)。frame max-height 620px + overflow 裁超长(md 全类型长卡)。
- **验证**:headless 可见页烟测 —— 9 区 / 13 卡 / 13 媒体 / 0 错;目视各区(md 全类型整幅、tool/
  diff 卡清晰、shimmer 工具卡、theme 改色文档、a11y 播报)成立;webm 滚动自动播(landing 动起来 =
  引擎即宣传片)。资产入库不 gitignore;manifest schema 单测不因新增字段破(只查既有字段,不拒未知键)。

## P4 · chat 页(完整对话 + 重放控制条)

- **已有(player-chrome / P1)**:播放/暂停、可拖进度轨(rail/fill/knob)、时间码、倍速按钮
  (0.5/1/2/4×)、画布宽度滑块 —— 进度 + 倍速本就齐,P4 只补预设双下拉。
- **预设双下拉(新)**:右面板加「呈现」区 —— ①节奏预设 `<select>`(typewriter 打字机 / reader
  朗读 / flow 流动 → `chat.set_reveal_preset`),默认 reader 与 boot 一致;②效果预设 `<select>`
  (off / subtle / expressive → `chat.set_effect_preset`),初值取 `chat.effect_preset_name()`
  当前档。均引擎公开接口即时生效,只改后续呈现不动已播语义(AR2 model/presentation 分离)。
  select 深色描边 + teal hover,与面板一致;每项带一行 small 说明。
- **验证**:headless 烟测 —— showcase-full(?speed=8)从头到尾播满(0:27,含推理/表格/diff 卡/
  ask 交互卡/输入盒),两下拉 3×3 选项、默认 reader+subtle,改 effect→expressive 即
  `effect_preset_name()==expressive`,0 错。目视:完整对话可看可玩,控制条(播放/进度/倍速/双预设/
  宽度)齐全。

## P5 · markdown 页(轮播 + playground)

- **引擎挂载**:markdown.ts `bootHero("md-canvas", reader)` + 就绪门(同 hero)后 expressive。
  无 GPU → 隐藏 canvas + 提示(Chrome 打开)。
- **全类型轮播(替换式,单回合)**:6 条样例(行内强调 / 列表任务 / 代码块 / 表格 / 数学 / 引用+Alert)
  每 7s 循环流式揭示。**关键坑**:同一 part id upsert 新文本**不重播**(沿用已揭示态);先加
  **新 part id** 才从头揭示,再 `message.part.removed` 删上一条 → 画布**始终只一条**、循环重现、
  不累积(省内存)。引擎无「删整条 message」事件,故用「单 message + 换 part id + 删旧 part」。
- **实时 playground**:textarea 粘贴任意 markdown → 「流式播放」停轮播 + feed 这段(同替换路径)。
- **验证**:headless 烟测 —— t=3s 样例0(行内)、t=10s 样例1(列表,**已换**、仍 n=1)、playground
  换成用户内容(n=1);目视 playground 输入「数学+表格」→ 引擎渲行内 E=mc² + 块级 ∫ + SDF 表格,
  流式揭示;0 错。可看(轮播)可玩(playground)成立。

## P6 · 收口(smoke / 降级 / 体积 / 文档)

(未开始)

## DoD 对账

(未开始)
