# Plan 41 · 进度账本(首页播放器模式)

> 逐 milestone 记录:幕剧本 / film 复用审计(file:行号)/ 幂等对拍 / 降级记录 / 遗留。
> GOAL:[plan41-homepage-player-mode-goal](./plan41-homepage-player-mode-goal.md)
> 前置:plan40 完成(风格 token / 四页 IA / pages-assets 转 fallback)。

## H0 · 幕剧本定稿 + film 复用审计(交作者过目)

### film 体系复用审计(file:line)

现成 `web/src/film/`(plan17)基本就是 plan41 想要的骨架 —— **复用为主,少量 additive**。

| 件 | 位置 | 契约 / 复用点 |
|---|---|---|
| Scene 接口 | `film/director.ts:15-21` | `{ id, title, durationMs, enter?(ctx), cues?[{at,fn}] }`;`enter` 幂等可重入(为 seek 设计,注释「可重入,供 seek」)——**正是单 canvas 直跳幂等的基石** |
| FilmDirector | `film/director.ts:30-154` | 全局时钟 `tick(dtMs)`(`:101-123`,**到尾停不循环** `:104-107`)· `seek(ms)`(`:93-98`)· `enterScene(idx,fromSeek)`(`:131-143`,seek 时把 `at<=local` 的 cue 标记已触发不重放)· `sceneAt` 线性扫 `starts[]` |
| 引擎派发 | `film/director.ts:59-72` | `ctx.call(name,...args)` 守卫式字符串派发到 ChatCanvas(缺方法只 warn,版本漂移安全);`FilmCtx={call,rate,teaser}` |
| rAF 泵 | `film/scenes.ts:83-90` | `mountFilm` 内 rAF loop 算 dt 喂 `dir.tick`;返回 destroy |
| FilmPlayer chrome | `film/player.ts:9-86` | 纯 DOM:播放/暂停 + seek 轨(rail/fill/knob)+ **逐幕刻度**(`dir.marks`)+ 时间码 + 0.5/1/2×;只需 `{marks,totalMs,toggle,setRate,seek,onUpdate}` |
| teaser 叠层 | `film/teaser.ts` | 半透明卡 CSS 淡入(.35s)——**字幕层的基底** |
| 现有 9 幕 | `film/scenes.ts:29-74` | open/scale-glimpse/stream/sdf/rich/fx/resilient/scale-peak/outro;**在一份 showcase 文档上编排节奏/相机/teaser**(不各自注内容) |
| 当前消费者 | `web/src/main.ts:221-225` | `demo && !explicitReplay` → `mountFilm(chat)`;即 `/dev.html` 默认首页。**plan41 不动 dev.html 这条**,另写首页 film |

引擎 API(经 `ctx.call`):`set_reveal_cps/slow`·`restart_reveal`·`zoom_at`·`pan_by`·`set_shaderbox_gallery`·`set_paused`;另 `scroll_to(view)`(`lib.rs:840`,**绝对滚到某回合** = 幕分段框取的关键)、`set_effect_preset`(`lib.rs:1091`,审计误判「不存在」,实存)、`push_event`。

### 内容架构决策(关键)

引擎**无 reset/clear/换 session 公开 API**(仅 `apply_part_removed` 删单 part)→ 逐幕注入新内容再清屏这条**不走**(累积、破幂等)。

**决策:单份「首页母文档」上编排**(沿用 plan17 现成幕架构,幂等且零清屏):
- 一份内容丰富、**分段有序**的母文档(自述段 → 功能卡:tool 三态/diff 折叠/表格/公式 → markdown 全类型段),开机一次性载入(instant catch-up,AR6);内容**复用** showcase 段 + plan40 markdown 样例 + 一段新自述。
- 每幕 `enter()` = `scroll_to(该段回合)` 绝对框取 + reveal 节奏 + 效果预设 + 字幕;**引擎缩放恒定 1.0**(S1 慢推镜走 **CSS transform**,同 plan40 hero;zoom 脉冲对称回 1.0)→ enter 幂等,直跳=`seek→enterScene→scroll_to` 到段首。
- 「真渲染」(DoD-4)由母文档全走引擎图元满足;各幕「不同内容」= 母文档**不同段**的相机框取,非清屏重注。

### 七幕剧本(定稿,时长/节奏走 token 可调)

| 幕 | id | 时长 | 内容(引擎真渲,母文档段) | 相机/效果 | 转场入 | 字幕层(DOM) |
|---|---|---|---|---|---|---|
| S1 标题 | `title` | 5s | hero 氛围导览流(品牌氛围),expressive dissolve/glow | CSS 慢推镜 dolly;expressive | 首屏淡入(wasm 空窗:字幕先行,引擎就绪 canvas 接管) | eyebrow `RUST · WASM · WEBGPU · SDF` + H1 `INFINITE CHAT` + 一句话定位 |
| S2 是什么 | `what` | 6s | 自述段流式(reader):Rust+wasm+WebGPU 的 SDF 对话渲染引擎… | scroll 到自述段;reader 节奏 | dissolve 400ms | eyebrow「是什么」+ 三点要点字幕 |
| S3 功能巡礼 | `features` | 8s | 功能段:tool 卡三态→diff 折叠→表格网格→公式;cue 每 ~2s `scroll_to` 推进 | 分段 scroll + 对称 zoom 脉冲看细节;subtle | dissolve | 「功能巡礼」+ 分段小字幕(工具卡/diff/表格/公式) |
| S4 完整对话 | `conversation` | 6s | 母文档作为一场对话整体快放(高 cps ×4 感) | scroll 跟随;fast reveal | dissolve | 「完整对话 · 进入完整版 →」深链 `/chat/` |
| S5 markdown | `markdown` | 6s | markdown 全类型段连播(行内/列表/代码/表格/公式/引用 Alert) | scroll 到 md 段;reader | dissolve | 「Markdown 全类型 · 打开 playground →」深链 `/markdown/` |
| S6 效果秀 | `effects` | 5s | 同屏内容上 off→subtle→expressive 切 + dissolve/拖尾/glow;cue 切预设 | 对称 zoom 脉冲 + shaderbox 惊鸿;预设切换 | dissolve | 「效果系统 · off → subtle → expressive」 |
| S7 尾幕入口 | `outro` | 停留 | 引擎静场背景(效果退场,零活跃动画) | 静场,zoom 1.0 | dissolve | **DOM 入口卡浮层**:chat/markdown/gallery/GitHub 四链接 + 页脚(技术栈 + commit) |

自动播总长 ≈ 36s + S7 停留;循环回 S1。

### 需 additive 的增量(film 体系之上,不动 dev.html 那套)

- **director**:`loop` 选项(现「到尾停」→ 首页要循环回 S1;在 rAF loop 到尾 `seek(0)` 或加 flag)· prev/next 幕助手(键盘/滚轮/点用)。
- **首页 chrome(新)**:进度**点 × 幕数**可点(点=`seek` 到幕首)+ 播放/暂停 + 键盘 ←/→ + 滚轮/滑动切幕 + **接管后停自动、空闲 30s 恢复** + `visibilitychange` 暂停。基于 `player.ts` 改（刻度→离散点）。
- **字幕层(新)**:DOM 每幕 eyebrow/标题,随幕 index CSS 淡入淡出(绑 `dir.onUpdate`);基于 teaser 样式。
- **转场**:DOM dissolve 遮罩,`enterScene` 期淡入淡出盖住重建帧(300–500ms,token);风险表「遮罩盖重建帧」对策。
- **母文档**:`web/` 侧构建(复用 showcase 段 + markdown 样例 + 自述),开机 push_event 载入。
- **降级**:无 GPU → pages-assets 截图/webm 幻灯(同幕结构同 chrome,媒体替 canvas)· reduced-motion → 不自动播 + 静帧 + 手动 · 移动端 → 相机 fit 宽或幻灯(H3 定稿)。

### 变更边界(重申)
主战场 `web/`(index/home 重构、首页 film 剧本 module、首页 chrome、字幕层、母文档);**不动** chat/markdown/gallery/dev 四页、引擎渲染语义、pages.yml。plan40 pages-assets/index.json 保留 → 职责转 fallback 数据源。

### Open / 待观察
- `scroll_to(view)` 的 view 语义(回合序号)需 H1 实测确认(母文档段→view 映射)。
- S1 首屏 wasm 空窗:字幕 DOM 先行 + 引擎就绪 canvas 淡入(plan40 `startWhenReady` 坑已趟)。
- 幕内 reveal 与 scroll 框取的协同(母文档 instant 载入后各幕是否 restart_reveal 局部)——H2 调。

## H1 · 播放器骨架(S1+S7 先通)

- **index.html 重构**:plan40 hero/功能卡区/大页脚**全移除** → 全屏单 canvas(`#home-canvas`)+ 薄壳:
  顶部渐晕/底部压暗 vignette、右上极简 nav(品牌 + GitHub)、字幕层(`#home-subtitle`,下三分)、
  S7 入口浮层(`#home-outro`,四链接卡 + 页脚,径向 abyss 遮罩)、chrome 容器、无 GPU fallback 容器。
  CSS 全走 pages-theme token;dolly 慢推镜走 **CSS transform**(`.dolly` 类,引擎 zoom 恒 1.0)。
- **模块(新,web 薄胶水)**:
  - `home-scenes.ts`:7 幕 `HOME_SCENES`(enter 只碰引擎:`set_effect_preset`/`scroll_to`/`set_reveal_cps`+
    `restart_reveal`,幂等)+ `SUBTITLES` 字幕文案 + `buildMasterDoc(chat)`(3 段 assistant 回合,
    push_event 真事件形状;H1 文本占位,H2 换 tool 卡/diff/表真富内容)+ `SECTION` 段→view 映射。
  - `home-player.ts`:`HomePlayer` 围绕现成 `FilmDirector`(**不改 director**)—— 自动推进 + **循环**
    (director 到尾停 → 这里 `seek(0)` 回 S1)· 手动接管(←/→ 键 / 滚轮节流 / 进度点)· 接管后停自动、
    空闲 30s 恢复 · 暂停/播放 · `auto:false`(reduced-motion:静帧只手动)。
  - `home-chrome.ts`:`mountHomeChrome` —— 幕进度点 × 幕数(点=`gotoScene`)+ 播放/暂停;返回 update()。
  - `home.ts` 重写:装引擎全屏(bootHero glyphMode 1)→ 就绪门载母文档 → `FilmDirector(HOME_SCENES)` +
    `HomePlayer` + chrome + 字幕/dolly/outro 按幕 index 驱动(`dir.onUpdate`)+ 深链 base 前缀 +
    visibilitychange 冻引擎 + 无 GPU 保底(canvas 隐 + 入口浮层)。首屏 wasm 空窗:S1 字幕 DOM 先行。
- **复用**:`FilmDirector`(director.ts 原样)· `Scene`/`FilmCtx` 类型 · `ctx.call` 守卫派发。
- **验证(headless)**:7 进度点、S1 当前点、字幕大标题「INFINITE CHAT」+ eyebrow + tagline、母文档
  3 回合真渲;点末点 → S7 入口浮层显 + 字幕隐 + dot6 亮(直跳行为面);0 错。pages-smoke landing 用例
  改播放器断言(幕数/字幕/S7 直跳/入口卡),3/3 绿。
- **遗留 → H2**:母文档短(整幅上屏,scroll_to 分段未视觉区分);S2–S6 内容/转场/scroll 框取 H2 补;
  S7 静场背景仍显母文档(H2/H3 调静场帧)。

## H2 · 内容幕全量(S2–S6 + 转场)

- **母文档换真富内容(对话式)**:`buildMasterDoc` 从 3 段文本 → **一场真对话**(user 问 → assistant 答),
  三回合:①自述文本;②tool 卡三态(bash completed/running · edit pending)+ diff 卡(edit
  completed + metadata.filediff)+ 表格 + 行内/块级公式;③markdown 全类型(粗斜删码链接/列表/
  Alert/代码块)。**关键**:user 消息分隔 turn(无 user 则所有 assistant 塌成一个 turn,scroll_to
  框不到段);对话式也让 S4「完整对话」名副其实。tool/diff part 形状取自 feature-manifest。
- **段框取(踩坑 × 3,全实测)**:
  1. **c.call 脱 this bug(致命)**:`FilmCtx.call` 以 `chat[name](...)` 脱 this 调 wasm-bindgen 方法 →
     抛 `__wbg_ptr` undefined,被 director try/catch 吞成静默 no-op → **整条 film 编排全失效**(dev film
     恐同样)。修:home.ts 传**绑定 Proxy**(方法自动 `bind(chat)`),不改 director.ts / 不动 dev film。
  2. **follow 覆盖**:引擎 following 态每帧滚到底,`pan_by` 定位被拉回;`scroll_to(view)` 会 set
     `follow=Released`(app.rs:1717)故能定住 → **一律用 scroll_to**(pan 仅作段内增量巡礼)。
  3. **cps 触发 follow**:幕 enter 里 `set_reveal_cps(1e9)` 会让 reveal 完成 → follow 回底,又覆盖
     scroll_to。修:**母文档 boot 一次性全揭示**(home.ts set_reveal_cps 1e9),幕 enter **不碰 cps**。
  view 校准(part 级、视口无关):0=自述问 · 3=功能答(其下 tool/diff/表/公式)· 9=markdown。
  末段(markdown)因其下无内容,scroll_to 会 clamp → 显于视口下半(可接受)。
- **幕内容**:S1 title(expressive + scroll 0 + CSS dolly)· S2 what(scroll 0)· S3 features(scroll 3 +
  pan cue 下推过 diff/表格)· S4 conversation(scroll 0 看整场)· S5 markdown(scroll 9)· S6 effects
  (scroll 9 + cue off→subtle→expressive)· S7 outro(scroll 0 + 入口浮层)。
- **转场**:`#home-transition` abyss 层,幕切 `flashTransition()` 重放 dissolve(opacity 0→0.6→0,
  var(--t-diss) 450ms)盖住 scroll 跳帧;reduced-motion 关。
- **可读性**:底部 vignette 加深(48%→86% at 90%)+ 字幕 text-shadow → 鎏金大标题在引擎内容上清晰。
- **验证**:7 幕逐一 dot 直跳截图 —— 各幕框取正确(intro 顶 / features diff+表格+公式 / markdown /
  outro 浮层)+ 字幕深链;0 错。

## H3 · 降级与移动端

- **无 GPU 幻灯降级**(`home-fallback.ts`):bootHero ok:false(WebGPU+WebGL2 皆无)或 `?slides` 强制 →
  canvas 隐,`mountFallbackSlides` 用 plan40 入库 pages-assets 每幕一张代表图/短片(title→reveal-typing
  webm · what→canvas-zoom · features→card-tool-states · conversation→card-diff · markdown→md-codeblock ·
  effects→fx-preset-switch webm)。**同幕结构 / 同 chrome / 同字幕**:复用 FilmDirector + HomePlayer,
  空引擎 {}(幕 enter 的 ctx.call 全 guard 成 no-op),onScene 切幻灯(webm 进场自动播、离场暂停)。
- **共用接线抽出** `wirePlayer(engine, onHidden)`:引擎路 / 幻灯路都建 director+player+chrome+onScene,
  仅 engine(绑定引擎 vs 空 {})与 hidden 处理不同。
- **reduced-motion**:HomePlayer `auto:false` → 不自动推进(静帧 S1)+ CSS dolly/transition 关;手动
  (键/点/滑)仍可切。实测:dot@1s=0 dot@8s=0(无自动播)· canvasAnim=none · 手动 dot3→dot=3。
- **移动端**(R0 拍板):保留引擎播放器(真渲染 > 静态幻灯);390×844 实测播放器可用、字幕 clamp 缩放、
  入口卡单列、chrome 可达 + **触摸横滑切幕**(HomePlayer touchstart/end,|dx|>48 且 >|dy|)。**取舍**:
  宽内容(diff/代码块)按固定 world 宽渲染 → 移动端右侧溢出(可读,不裁字);要更贴合可后续加相机 fit 宽
  (但破 zoom 恒 1.0 幂等),故 R0 选保留引擎 + 接受溢出(无 GPU 移动端自动走幻灯,contain 不溢出)。
- **验证**:?slides 幻灯 6 slide/1 on/7 dot/字幕在,S3 显 card-tool-states 截图 + 同字幕/chrome;移动端
  引擎播放器 7 dot 可用;reduced-motion 无自动播 + 手动可切;0 错。

## H4 · 收口(smoke / 幂等对拍 / 文档 / DoD)

- **pages-smoke 扩断言**(进 e2e 门):landing 用例覆盖 —— 单 canvas 实例(单实例铁律)· 母文档真渲回合 ·
  七幕点 + S1 字幕 + nav · **自动播**(不介入过 S1 5s 后当前点前移)· **键盘**(←回 S1 / →进 S2)·
  **直跳幂等**(直跳第 3 幕框取 == 先跳第 6 幕再跳第 3 幕框取,plan17 契约行为面)· **S7 深链**(入口浮层显 +
  字幕隐 + 四卡 + chat/markdown href base 前缀)。新增**幻灯降级用例**(?slides:canvas 隐 + fb-slide +
  同 chrome/幕数/字幕 + 点幕切 slide)。本地 4/4 绿。
- **幂等对拍取「态对拍」而非「像素 golden」**:DoD 原提「每幕 golden dpr=1/2」。评估后取**状态对拍**
  (visible_turns 顶部框取文本比对):①GPU 逐像素 golden 对动画片跨机/跨驱动易脆(现有 visual golden 靠
  窄 clip + off-profile 才稳,首页 7 幕全屏动画不具备);②plan17 幂等契约的本质是「同幕 enter 产同一
  引擎态」,态对拍**直接**验此契约,比像素更本质且稳。记此取舍(同 plan40「无 GPU fallback 按构造判定」)。
- **多页构建**:PAGES_BASE build 五页(index 播放器 / chat / markdown / dev / gallery)+ pages-assets
  14 档拷入 dist;home-fallback/player/scenes 入包。
- **文档**:DEPLOY-pages.md landing 条改「幕式播放器 + 幻灯降级」+ pages-assets 职责转 fallback;
  README 在线官网条同步。
- **四页不回退回归**:chat/markdown/gallery/dev 既有 smoke 原样绿(pages-smoke 后三例 + chat-route 等)。

## DoD 对账

| DoD | 状态 | 证据 |
|---|---|---|
| 1 幕剧本(6–7 幕) | ✅ | 7 幕 HOME_SCENES(title/what/features/conversation/markdown/effects/outro),H0 定稿 commit |
| 2 播放器行为 | ✅ | 自动播循环 + ←/→/进度点/滚轮/触摸接管 + 暂停 + 接管停自动/空闲 30s 恢复 + visibilitychange + reduced-motion 静帧;smoke 断言自动播/键盘/直跳 |
| 3 幕切换(单 canvas / enter 幂等 / 转场 / 直跳一致) | ✅ | 单 FilmDirector + scroll_to 框段 + dissolve 转场;直跳幂等态对拍绿 |
| 4 DOM 薄壳(字幕/chrome/S7/极简 nav;卡区移除) | ✅ | index.html 全屏 canvas + 字幕层 + 进度点 chrome + S7 浮层 + 品牌/GitHub;plan40 卡区/大页脚删 |
| 5 降级(无 GPU 幻灯 / reduced-motion / 移动端) | ✅ | home-fallback 幻灯(pages-assets 同 chrome)+ auto:false + 移动端引擎+触摸(R0 记账) |
| 6 门与验收(smoke/门绿/回退/PAGES_BASE) | ✅ | pages-smoke 扩(4 例)+ 五层门绿 + 四页 smoke 不动 + PAGES_BASE 五页构建通;每幕像素 golden → 态对拍(记账) |
| 7 progress 记账 + milestone commit 不 push | ✅ | 本文件 H0–H4;H0–H4 逐 milestone commit,**不 push** |

| 客观判据 | 状态 | 证据 |
|---|---|---|
| 单实例 | ✅ | smoke 断言 canvas 数 == 1;幕切不重建 wasm(FilmDirector 仅 scroll/preset) |
| 幂等直跳 | ✅ | 态对拍:直跳第 3 幕框取 == 经第 6 幕再跳第 3 幕框取 |
| 播放器行为 | ✅ | 自动推进/键盘/进度点/接管/隐藏各断言;空闲恢复由 idleMs 逻辑(30s,非 smoke 长跑) |
| 真渲染 | ✅ | S2–S6 内容来自引擎 visible_turns(tool/diff/表/公式/markdown 图元);仅 ?slides fallback 用 img/video |
| 风格延续 | ✅ | chrome/字幕/浮层全 pages-theme token;grep 无新硬编码色值(vignette/scrim 用 rgba abyss 同底色) |
| 不回退 | ✅ | 五层门 464/464;chat/markdown/gallery/dev 四页 smoke 原样;引擎 core/render 未改,golden 全集不触 |

**遗留 / 取舍**:
- 末段(markdown)scroll_to clamp 显于视口下半(其下无内容);要顶置需母文档更高,H2 判定可接受。
- 移动端宽内容(diff/代码)按固定 world 宽右溢(可读不裁字);相机 fit 宽会破 zoom 恒 1.0 幂等,R0 保留引擎。
- 每幕像素 golden 未落(GPU 动画片跨机脆)→ 以直跳幂等**态对拍**替(更本质);像素回归仍靠既有 visual golden(dev.html)。
- `FilmCtx.call` 脱 this bug 用绑定 Proxy 局部绕过(未改 director.ts);dev.html 的 film 同 bug 仍在(非本 plan 目标域,未动)。
