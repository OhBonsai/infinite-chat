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

(未开始)

## H2 · 内容幕全量(S2–S6 + 转场)

(未开始)

## H3 · 降级与移动端

(未开始)

## H4 · 收口(smoke / 每幕 golden / 幂等对拍 / 文档)

(未开始)

## DoD 对账

(未开始)
