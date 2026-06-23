# Plan 17 — 产品介绍片(引擎自驱多幕剧 + 播放器进度条)

- 日期:2026-06-23
- 前置:[导演表](../design/intro-film-director-notes.md)(创意 bible,决议 v0.2)、Plan 8(reveal)、Plan 13(chat 布局)、0022(DOM 叠加)、0003(容错)、plan16(shaderbox)
- 一句话:把导演表落成**一页引擎自驱的介绍片**——一个"导演时钟"按时间轴推进、在每个镜头入点调 `ChatCanvas` API,底部一条视频播放器进度条(播放/暂停/调速/拖动 seek/幕刻度)。**不是 AE 合成视频,是产品实时在跑。**

> 作用域:`web/` 层为主(导演/场景/播放器全是 TS + 引擎 API 编排),**不动 core/render**。已上线能力直接驱动;路线图能力用"即将到来"teaser 卡(DOM)占位,不伪造效果。

---

## 1. 架构(三件套)

```
FilmDirector(导演时钟)  ──驱动──▶  ChatCanvas(引擎,已有 API)
      │  按 scene.timeline 在入点/cue 调 API(reveal/zoom/pan/paused/seek)
      │
      ├── Scene[](场景表 = 导演表逐幕的可执行版)
      │     each: { id, title, durationMs, content?, enter(chat), cues[{at, fn}], camera? }
      │
      └── FilmPlayer(DOM 播放器)  ◀──绑定──▶  FilmDirector
            ▶/⏸ · 0.5/1/2× · 进度条 + 9 段幕刻度 · 拖动 seek
```

- **FilmDirector**:维护 `clock`(ms,受 `rate` 调速)、`currentScene`、`playing`。每 rAF `tick(dt*rate)`:推进 clock;到 scene 边界 → `exit` 旧 + `enter` 新;在 scene 内按 `cues` 的 `at` 触发一次性引擎调用。
- **Scene**:声明式。`enter(chat)` 负责把引擎**置到该幕起点**(载内容 / 设相机 / 设 reveal 参数 / restart_reveal)——这是 **seek 可跳到任意幕**的关键(每幕自包含、可重入)。
- **FilmPlayer**:纯 DOM chrome(0022 风格),固定屏幕坐标;只读/写 director 的 clock 与 rate。

## 2. 关键机制

### 2.1 调速(节奏优先)
`rate ∈ {0.5,1,2}` 同时作用两处:① 导演时钟 `dt*rate`(镜头快慢);② 映射 `set_reveal_cps`/`set_reveal_slow`(流式吐字快慢),使"调速"对内容揭示也生效。

### 2.2 暂停 / 播放
`set_paused(true/false)`(已上线)。暂停时导演时钟也停 → 画面定格(符合"每镜暂停值得截图")。

### 2.3 拖动 seek(跨幕)
进度条拖到 t → 找到所属 scene → 调该 scene 的 `enter(chat)` 把引擎重建到该幕起点 → 再用 `seek_reveal(局部偏移)` 对齐幕内进度。**因此每幕 enter 必须幂等可重入**(载自己的内容、设自己的相机)。跨幕状态跳转不追求逐帧精确,只求"跳到这一幕开头看得对"。

### 2.4 相机运镜
`zoom_at(factor,sx,sy)` / `pan_by(dx,dy)`。运镜 cue 用"每帧增量"在一段时间内累加(如 0.8s 内匀速推近到 8×),对称缩回避免漂移(同 reel zoomPulse)。每幕 enter 时**复位相机**(记录基准 → 反向操作归位,或提供一个 `reset_camera` 钩子,见 §5 待办)。

### 2.5 规模幕的"长河"(双现:惊鸿 + 全爆发)
需要一条 **100+ 轮的长历史**瞬间在场(不能慢慢流)。两条路:
- **A(优先,免改 core)**:预生成长历史 replay,用**极高 cps**(`set_reveal_cps(1e6)`)+ 大步 `tick` 让它"秒到全屏",再相机拉远。
- **B(若 A 太抖)**:core 加一个"种子历史"入口(直接落 settled 块,跳过揭示)——`[需 core 改动]`,记为路线图。
v1 先走 A。

### 2.6 路线图 teaser(决议 4:全提上)
glow-orb / 3D·raymarch / 多实例 / a11y 镜像未上线 → 用 **DOM teaser 卡**(0022 叠加层风格):一块半透明面板 + Mono 标题 `COMING SOON` + 一句话 + 极简静态示意(不伪造动效)。诚实,且把"未来"讲出来。

## 3. 文件map(全新增,web 层)

```
web/src/film/
├── director.ts     # FilmDirector:时钟/调速/seek/scene 推进
├── scenes.ts       # Scene[] = 导演表 v0.2 九拍的可执行版(content + enter + cues)
├── player.ts       # FilmPlayer:DOM 进度条/按钮/幕刻度
├── teaser.ts       # 路线图 COMING SOON 卡(DOM 叠加)
└── film.css.ts     # 播放器/teaser 内联样式(零依赖)
web/public/cases/film/*.json   # 各幕内容 fixture(gen-film.mjs 生成)
scripts/gen-film.mjs            # 生成各幕 fixture(含长河 long-history)
```
入口:`main.ts` 在 `?film=1` 或 VITE_DEMO 主页 → `mountFilm(chat)`(取代当前 reel 导演)。

## 4. 相位

| 相位 | 交付 | 验证 |
|---|---|---|
| **① 骨架** | director.ts + player.ts + scenes 占位(3 幕:开场/流式/收尾)+ main.ts 接线 | tsc 过;本地 `?film=1` 能播放/暂停/调速/拖动 |
| **② 内容 fixture** | gen-film.mjs:各幕内容(代码/表/数学/图/猫线/长河) | 重放各幕内容正确上屏 |
| **③ 运镜 + 规模** | zoom/pan cue;规模幕长河拉远(§2.5 路 A);相机复位 | GPU 人工:推拉锐利、拉远 fps 稳 |
| **④ 全九幕 + 容错** | 九拍全接;容错幕(断网/重连/刷新)编排;高潮规模全爆发 | GPU 人工:整片连贯、高潮成立 |
| **⑤ 路线图 teaser + 收尾** | teaser 卡(glow-orb/3D/多实例/a11y)+ 收尾 chat + slogan | GPU 人工:teaser 诚实、slogan 落地 |
| **⑥ 调参 + 部署** | timing/cps/相机参数调到顺;接 Pages(主页=film) | 线上整片顺滑;进度条/seek 可用 |

> 沙箱可验:tsc + 文件结构 + fixture 重建。**运镜/规模/整片观感须 GPU 人工。**

## 5. 待办 / Open(可能需 core 小改)

- **相机复位 API**:seek 跨幕要把相机归零。现有只有相对 `zoom_at/pan_by`。① v1 用"记录基准+反向归位"在 web 层凑;② 若不稳,core 加 `reset_camera()` / `fit_camera()`(小改)。
- **种子历史**(§2.5 路 B):规模幕若拉远太抖,core 加"直接落 settled 历史"入口。
- **glow-orb / 3D / 多实例 / a11y**:本片只作 teaser;真实现各有自己的 plan。
- **seek 幕内精度**:跨幕只保证"跳到幕首对",幕内逐帧精确 defer。

## 6. Done

一页引擎自驱介绍片:FilmDirector + 九幕 Scene + FilmPlayer 进度条(播放/暂停/0.5·1·2×/拖动 seek/幕刻度)落地;规模双现(惊鸿+全爆发高潮)、容错编排、路线图 teaser、收尾 chat + slogan「一条永不结束的对话」;主页(VITE_DEMO/`?film`)= 介绍片;markdown demo 降为 `?replay=showcase`;tsc 全绿;线上整片顺滑。
