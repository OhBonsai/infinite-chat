# Plan 25 · progress(GOAL 执行日志)

- GOAL: [plan25-chat-page-design-goal.md](./plan25-chat-page-design-goal.md)
- 设计: [design/chat-page-design.md](../design/chat-page-design.md)
- 开跑: 2026-07-06
- 协议: 每 milestone 记录「做了什么 / 验证方式 / 遗留」;视觉评审记录在 test/results/design-review/REVIEW-*.md,结论汇入本文。

## M0 · 基建 + P0 token 收口

- 状态: **完成**(2026-07-07;四层全绿 gates4/native272/unit44/e2e37,视觉自评 r3 达标)
- 做了什么:
  - **前置**:修 master 自带 3 个 clippy pedantic 警告(boxlayout/nodes/partspecific),基线 gates 回绿。
  - **截图 harness**:`web/tests/design-review.spec.ts`(execution-only,S1 节奏四帧+8连帧 / S2 预设对比(feature-detect `set_reveal_preset`,M1 前只拍 current)/ S9 整页两帧);产物 `test/results/design-review/<scene>/<name>.png`。定帧一律 `set_paused(true)` + `seek_reveal(ms)`(确定性)。playwright.config `testIgnore` 挡出默认门,跑法 `DESIGN_REVIEW=1 npm --prefix web run e2e -- design-review.spec.ts`。`main.ts` 加 `?empty`(一条 noop 未知事件占 Player 通道,AR12 → Ignored)= 干净画布。
  - **MotionTokens**:`crates/core/src/motion.rs`(dur/ease-id/stagger/space 一张表,serde 局部覆盖照 Theme 模式)+ Engine `motion` 字段 + wasm `set_motion(json)` + web style-config `motion` 覆盖 + style-panel「Motion」节 11 个旋钮(实时,不重排)。
  - **曲线**:`glyph.wgsl::apply_curve` 扩 id 2/3/4 = enter/exit/expressive(bezier 数值拟合:`1-(1-t)^1.9` / `.5t+.5t^2.1` / `1-(1-t^1.7)^3.4`,maxErr ≤.035;t=1 恒 1 守 RD4)。0025 的 CurveId 尚无 Rust 枚举 → 决策:不建枚举,motion.rs 钉 `CURVE_*` id 常量与 wgsl 同值,快照测试锁。
  - **enter profile 默认**:id0 由「0.1→1 大缩放 pop」收敛为 design §4.3 的「Alpha + 上浮 6px + Scale 0.96→1 + ease.enter」;fade 时长 = `dur_glyph`(200ms)经新 `FrameData.fade_ms` 传 sink(EffectProfile 档位语义保留)。
  - **间距 + layout**:boxlayout 走 token(turn 40 / part 16),内容列 760 **水平居中**(design §2.1,两侧留白≥48)+ 顶/底留白 `space_turn/2`;user 气泡 max = 85% 列宽。
  - **user 弱底面板**:theme 加 `user_bg`,build_frame 对 user view 发 FramePanel(圆角 10、无网格/AO,outset `space_inset` 比例,不动盒位守锚定;id 低 32 位 `0xFFFF_FFFF` 防撞表格面板)。
  - **meta 延迟显**:copy 按钮移到消息盒**下缘外**(顺带修复「按钮压 user 文字」的既有缺陷,S9 基线截图可见)+ streaming 中末条消息不显(Complete 态才现)+ 240ms/延迟 60ms 渐显(镜像 token 默认)。
  - **native 测试**:`motion.rs` token 快照 + JSON 局部覆盖 2 条;boxlayout 新增列居中测试;既有测试按新布局意图更新(右缘平移 200→100、orb 半可见 → 先 pan 入屏);核心几何快照(replay_snapshot)重录 = 纯 +[20,20] 平移(居中 20 + 顶部留白 20)。
- 视觉自评(3 轮,≤5 上限内收敛;样张 test/results/design-review/,记录见下):
  - **r1**(token+面板+居中后):发现 3 硬伤——①正文折行宽=整文档宽,长段溢出居中列(pre-existing,居中后更刺眼);②头像 orb 逐 part 一个,S9 同屏 5 个呼吸违「活跃动效组 ≤4」;③copy 按钮逐 part 一个,一屏 8 个漂浮。
  - **r2**(修①折行宽=列宽 wrap_width、②orb 每回合一个、③按钮按回合×角色聚合):S1 折行进列宽 ✓、S9 3 orb ✓ 3 按钮 ✓;残留 2 项——④H2 标题底线错画在整块底部(pre-existing);⑤user 按钮与下张卡视觉贴近。
  - **r3**(修④head_rule 贴标题行底、⑤user meta 改 hover 才显 per design §4.1):rubric 客观项 1/2/3/6 全 ✓(4/5 wipe 类 M0 无对象);观感项 11/12 ✓。**达标**。rubric 7(settled 帧全静态)✗:orb 呼吸常驻——按 GOAL 属 M2「idle 全退场」项,不在 M0 修。
  - 顺带修复的 pre-existing 缺陷:copy 按钮压 user 文字、head_rule 错位、MonospaceLayout 不镜像真桥「代码不折行」(seam 失真)。
- 验证方式: `node test/run.mjs` 四层(gates/native/unit 已绿,e2e 黄金图按新观感重录后绿)+ design-review S1/S2/S9 截图自评 3 轮(rubric §5)。
- 遗留:
  - `space_block_gap` 默认保 8 非设计示意的 12:块内真实间距由块间 `\n`(行高)驱动,未数据化;token 只接了数学溢出补偿位点。机制化留 M2+ 或后续。
  - `space_inset` 尚未接卡片内边距(tool 卡 inset 在 partrender 硬编),M2 edge_glow 时一并收编。
  - settled/idle 帧非全静态(头像 orb 常驻呼吸)→ M2 待机三件套 + idle 全退场时解决(rubric 7)。
  - user 消息 meta(copy)现为 hover 才显的 DOM 按钮,但按钮不可见时不可发现;M2 交互反馈(hit-test hover)再打磨。

## M1 · P1 节奏函数

- 状态: **完成**(2026-07-07;四层全绿 gates4/native280/unit44/e2e37,视觉自评 r1 达标)
- 做了什么:
  - **`crates/core/src/rhythm.rs`(新)**:分层节奏成本函数(design §3.2 七层参数化)——
    `cost(unit) = Δ×lengthen×accel×accent_slow + rest(boundary)×knob×scale + accent_pause + ε(seeded)`。
    单元 = 拉丁词一拍(词首计 Δ、词内 0 = 「拍点在词不在字」)/ CJK 单字一拍;标点/空白 0 成本吸附。
    边界信号 = 标点(次级 100/主级 250)+ 0020 节点树(列表项/表格行 150、段落 400、顶层块 600),
    节点 rest 沿块间 `\n` 传导给下一单元。微定时 = splitmix64 两倍频 hash 噪声(1/f 白噪近似),
    seed = 块下标,clamp ±min(0.2Δ, 50ms×scale)(R8/R9)。
  - **关键取舍——额外项等比 tempo 缩放**(scale=tempo/180):表值 = 默认 tempo 下的绝对毫秒;
    tempo→0(e2e `set_reveal_cps(1e9)`)⇒ 全项→0 ⇒ 即时揭示,全部既有测试/重放零改动过。
  - **预设三档**:typewriter(∞ 跟到达、零层、逐 glyph 拍 = 旧匀速**逐字**等价——`word_unit`
    开关保回归)/ reader(180ms、全层、词拍;设计默认档)/ flow(120ms、rest 0.6、accent 1.5、
    accelerando 1)。**Engine 默认 typewriter**(native/重放不回归),**产品默认 reader** 在
    main.ts 启动切(bench 模式豁免——采集指标不受节奏拖慢)。
  - **调度器改毫秒赤字制**(reveal.rs):`advance_clock` 攒 ms(×slow,封顶 32ms≈2 帧防倾泻),
    释放判据 `budget≥0`、扣整段成本可透支 → 长停顿(段落 400ms)自然等足、永不死锁;
    `idle_reset` 只没收正余额(赤字 = 上一单元停顿未走完,保留)。`set_reveal_cps` 降级为
    `tempo=1000/cps` 别名(旧旋钮/测试全兼容)。
  - **wasm API**:`set_reveal_preset(name)` + `set_rhythm(json)`(六旋钮局部覆盖**当前值**,
    RhythmOverrides Option 字段)。
  - **web**:style-panel「Rhythm」节 = preset 下拉 + 六旋钮(切档清旋钮覆盖防污染;改完
    restart_reveal 立即重看);style-config 持久化。
  - **harness 升级**:S1/S2 改为 set_reveal_preset + `set_rhythm({tempo_ms:60})` 统一 tempo
    (对比节奏层/groove 而非速度;此前 seekShot 强设 cps 会抹平预设差异——r1 发现的 harness bug)。
  - **native 测试(GOAL ①②③ + DoD5)**:rhythm.rs 6 条(同 seed 逐位相等/typewriter 匀速/
    边界停顿单调 逗<句<段≤块/词一拍/tempo→0 即时/重音更贵)+ reveal.rs 调度器 4 条改写
    (毫秒制等价)+ app.rs 整机 `rhythm_reader_release_times_deterministic_same_seed`
    (reader 含微定时 spawn 序列逐 ms 相等 + 释放间隔有节奏差)。
- 视觉自评(S2 同刻三预设,test/results/design-review/s2-presets/):reader 停在列表项首
  (边界呼吸可见)、typewriter 匀速推进无停顿(机械基线)、flow 最快最密(低 rest+accelerando)
  ——rubric 8 ✓;拉丁词整词上屏(如 "(tempo)")——rubric 9 ✓;新揭示字带淡入(enter 动画中)✓。
- **竞态修复(e2e 4 红根因)**:main.ts 在 bootCanvas 之后才 set_reveal_preset("reader"),而
  e2e 在 `window.__chat` 出现即 set_reveal_cps(1e9)——预设晚到把 tempo 打回 180ms → V1/V3
  黄金帧拍到揭示中途、TC-F13 的 AAA 排在未揭完的 showcase 之后被预算饿死、ask 表单等不到锚。
  修法:preset 进 BootOpts,在 `window.__chat` 暴露**之前**应用(boot.ts),e2e 覆盖稳赢。
  另补 native 复现 `stop_freezes_future_but_pre_stop_content_reveals_under_reader`(TC-F13 流程
  在 reader+1e9 下的核内等价,先证 core 正确 → 定位到浏览器侧竞态)。
- 验证方式:`node test/run.mjs` 四层 + S1/S2 连帧自评。
- 遗留:
  - 微定时用两倍频 hash 白噪近似 1/f(真 1/f 需滤波状态,违纯函数;听感差异在 ±50ms 内可忽略)。
  - 行内结构闭合(链接/行内码)的 80ms 边界未接(v1 从简;节点树 Run 不区分行内闭合位)。
  - accelerando 的句位序用「句内单元计数/24」近似(无真句长前瞻)。

## M2 · P2 组件动效

- 状态: **完成**(2026-07-07;S3–S8/S10 场景矩阵全出图,各子循环 1 轮达标;e2e 新增两断言;评审 REVIEW-M2-r1)
- **M2a 待机生命感三件套 + 动画组预算**(完成,视觉自评 S8 一轮达标):
  - `WIDGET_PULSE`(markdown/pulse.wgsl):呼吸圆角条 SDF(0.15Hz、幅度 ≤±8%,非 blink);
    行内光标(揭示前沿右侧小方块)+ 回合左缘 2px 指示条,**只在 view 未 settled 时发射** →
    settled/idle 全退场(rubric 7);挂 globals.time_ms(注入时钟,seek 确定)。
  - 头像 orb:**等待/流式中才 dynamic**,settled → `dynamic=false` 冻结(0028 护栏 2)。
  - `FrameStats.anim_groups`(dynamic orb + 呼吸 widget + wipe/流光面板)→ wasm `stats().animGroups`;
    e2e `motion-idle.spec.ts` 两断言:「idle 归零」「流式 ≥1 且 ≤4」(GOAL M2 验收项)。
  - native `anim_groups_alive_while_streaming_zero_when_settled`。
  - S8 样张:流式相位两帧(呼吸可见、幅度小)+ settled/idle 帧(全退场)✓。
- **M2b 卡片 wipe + edge_glow + 徽章**(完成,S3/S4 一轮达标):
  - FramePanel 加 `edge_glow`/`glow_phase` 参数(wgsl 布局 [22]/[23],ratios 平移到 [24];
    wasm 打包同步;naga 拼接测试锁)。流光 = |d|≈0 的 1.8px 光带 × 沿周长 cos(θ−phase) 流动,
    面积=描边(0028 成本模型便宜);相位挂慢时钟。
  - tool/reasoning 卡底从双 FrameRect 升级为 **SDF 面板**(0018 §6 收编):`reveal` 由首个已揭字
    spawn 驱动 0→1(dur.panel × ease.expressive,CPU 喂少量面板参数,非逐字 RD2 不涉;
    catch-up spawn=-1e9 → 恒 1 守 AR6;走完恒 1 守 AR3)。卡片面板槽位 id 低 32 = 0xFFFF_FFFE。
  - `BlockCache.card_status`(自 RenderPart.kind_tag 提取 pending/running/done/error)→
    running = edge_glow 0.85 + 实心点徽章;`WIDGET_BADGE`(markdown/badge.wgsl):环/点/勾/叉
    SDF(select 收敛无控制流,fwidth 安全);error 红叉(色变 snap 合法)。
  - native `card_wipe_enters_running_glows_done_freezes`(wipe<1→恒1 / running 流光 / done 勾 /
    动效组归零);既有 2 测按新意图更新(orb settled 静态、卡底 rect→panel)。
  - S3 样张:四态徽章一眼可辨、running 流光、wipe 中点框先行 ✓;S4:Thinking 骨架先行 ✓。
- **M2c+d 代码/表格验帧 + diff 5 格条**(完成,S5/S6/S7 一轮达标):
  - S5 代码块:底板/行号/语法色随行揭示生长 + 光标(行窗 tail 跟随属 Plan 15 既有设计);
    S6 表格风格 3:网格骨架先现 → 表头 stage → cell 填充(几何连续)✓。
  - DiffChanges 5 格条:diff 带扫描顺路按行 y 量化数增/删行 → 卡右上 5 个 7×7 圆角小格
    (增绿/删红/余灰,**theme 色全数据驱动的 FrameRect** —— 与 design「widget 小 fn」等视觉、
    零新 shader,取舍记档);r2r3 native 断言 5 格。S7 样张:行带 + `+3 −2` 摘要 + 3绿2红格条 ✓。
- **M2e 词级装饰层**(完成):词内 glyph spawn 依次 +k×stagger.glyph(25ms,token 置 0 即关);
  到达高亮 = glyph.wgsl enter 期 `tint ×(1+arrive_boost×(1−e))`(随 e→1 衰减归位,AR3),
  `arrive_boost` 进 MotionTokens(默认 0.08,可关)→ FrameData → backend Globals(pad 槽改名)。
- **M2f hover/press**(完成,S10 一轮达标):Engine `pointer/pointer_down`(presentation-only)
  → wasm `set_pointer(±)/set_pointer_down` → input.ts pointermove/down/up/leave;卡片 hover =
  fill +0.03 提亮 + 边框 alpha +0.2,press = 面板中心微缩 0.99(即时 ≤50ms,盒位/命中不动守锚定);
  native `pointer_hover_lifts_card_press_scales`(hover 提亮/press 微缩/离开复原)。
- M2 验收:场景矩阵 S1–S10 全出图 ✓;e2e 新增「idle 回冻结」「动画组 ≤4」两断言 ✓(motion-idle.spec);
  四层门见 TESTREPORT(M2 收口跑)。
- M2 遗留(候后续小循环):
  - 流光颜色暂硬编主题蓝(panel 参数已数据化,色值收 theme token 留后续)。
  - 徽章不随卡片 wipe 裁切(独立 widget);观感无碍。
  - reasoning auto-collapse / 卡片折叠交互未实现(S3/S4 折叠中点帧缺席)→ 记 GOAL 范围内遗留。

## M3 · 黄金帧固化 + 汇总验收

- 状态: **完成**(2026-07-07)
- 做了什么:
  - 代表帧黄金图 4 张迁入 `visual.spec.ts`(默认门,`--update-snapshots` 入册后连跑两次验证确定性):
    **V4** tool 卡四态(徽章 + running 流光定相位)· **V5** 表格骨架中点(风格 3 网格先行,
    `microtiming:0` 关噪声保逐 ms 确定)· **V6** diff 卡(行带 + 摘要 + 5 格条)· **V7** 整页布局
    (居中列 + user 弱底 + 回合分组)。全部 `?empty` 干净画布 + push_event 合成 + `set_paused` +
    `seek_reveal` 定帧(相位/呼吸皆注入时钟 → 完全确定);clip 从内容列左缘(260)起裁掉 orb 列。
  - e2e 由 37 → 43(+4 黄金 +2 motion-idle);TODO.md「节奏美感升级」条目标注已落地 → 指向本 plan。
- 验证方式: `--update-snapshots` 录制后紧接普通跑 = 43/43 全绿(截图两连一致 = 确定性)。

## M4 · P3 高端档(best-effort)

- 状态: **部分完成 + defer**(2026-07-07,按 GOAL「允许 defer,记录理由」)
- **✅ 徽章 SDF mix morph(Plan 10 相位 4 最小件)**:红→绿全流程——
  - 红测 `badge_morphs_on_status_transition` 先证 bug:tool 状态迁移经 part.updated **全量重写**,
    旧 cache 被重建 → 从 cache 读 prev 恒 None → morph 永不触发。
  - 修:PartView 加 **view 级** `last_card_status`(跨 cache 重置存活)检测迁移 → `badge_morph =
    (from, at)`;徽章 params 升级为 `[from_shape, stroke, morph_t, to_shape]`,wgsl `d =
    mix(d_from, d_to, ease_expressive(t))` = 0025 值层 MixT 首用;稳态 from==to、t=1 恒等(AR3);
    形变中计入 anim_groups。
- **✅(已有)墨团选区(smin 并集)**:Plan 21 P3 已落地(V3 `ink-blob` 黄金帧),无需重做。
- **⏸ defer:thinking_flow FBM 活性材质** —— 理由:新 shaderbox shader-id + pipeline + reasoning
  卡 running 态判定(reasoning 无 status 语义,需 FSM 侧新信号)属新机制面;GOAL 风险表「2 轮不达标
  即 defer」,为守整体 DoD 优先度让位。留:0028 shader-id 注册表 + 卡片流光通道均已就位,后续加
  `thinking_flow.wgsl` + 发射条件即可。
- **⏸ defer:lensing/specular hover 高光** —— 理由:需 panel 增 pointer 世界坐标 uniform 与高光
  参数(又一次 params 布局迁移);M2f 已交付 hover 提亮/press 微缩的基础反馈,增量观感收益低于
  布局迁移回归风险(0018 参数契约两侧同步成本)。留:pointer 已进 Engine,panel 参数通道成熟。

## 追记 · 作者人工核查反馈(2026-07-07)

- **`/chat` 剧本页接 reader 预设**:此前只有主 harness 有产品默认档,剧本页(展示面)走
  typewriter → 节奏观感缺席;`chat/main.ts` bootCanvas 加 `rhythmPreset:"reader"`
  (e2e 照旧 cps 1e9 覆盖,chat-route/ask 全绿)。
- **修:HiDPI 列宽锁死**(作者实测,slider 拖到 770px 内容不动)——根因:M0 的
  `CONTENT_MAX=760` 以 world px 计,而 world unit = **设备像素**(HiDPI 排版 ×dpr)→ retina 上
  列宽被钉在 760 设备 px ≈ 380 CSS px。修:内容列上限 ×dpr(Engine `set_dpr`,wasm 启动/resize
  注入 `devicePixelRatio`;native/无头 dpr=1 → 全部测试与黄金帧零影响);顺带修掉 AsstBox
  `max_size` 残留的未缩放 CONTENT_MAX。
- **修:Thinking/tool 卡标题逐字闪**(作者实测观感差)——标题是标签不是正文,逐词/逐字揭示 +
  词内 stagger 在标题上呈现为闪烁。修:ToolTitle 连续段(含内部空格)在节奏层记 **一个单元**
  (仅首字计拍),调度层豁免词内 stagger → 整行同帧、一次 SDF enter 动画(alpha+上浮+scale)
  整体入场,与 design §4.5「标题 stage 整体入场」对齐。native `card_title_is_single_beat` 锁行为。
- `.gitignore` 加 `.qoder/`(Qoder IDE 本地配置)。

## DoD 对账(GOAL §0)

1. **`node test/run.mjs` 四层全绿(含新增测试)** ✅ —— 终态代码:gates 4/4、native 285(vs
   基线 269,新增节奏/token/动效/morph 等 16 条)、unit 44、e2e 43(vs 基线 37:+4 黄金帧
   +2 motion-idle 断言)。注:一次性完整单命令跑约 10–12 分钟;终验按层单跑各自全绿 +
   完整跑记录见 TESTREPORT。
2. **M1–M3(P0–P2)验收清单逐项勾;M4 完成或记 defer** ✅ —— M0/M1/M2/M3 各节全勾;
   M4:徽章 morph ✅、墨团选区已有 ✅、thinking_flow/lensing defer(理由在案)。
3. **design-review.spec.ts 产全场景截图 + rubric 自评 ≥ 达标线,评审入 progress** ✅ ——
   S1–S10 全出图(execution-only,不进默认门);评审 REVIEW-M0-r3 / M1-r1 / M2-r1 / M3-M4
   (test/results/design-review/,transient;结论全部汇入本文各节)。M0 三轮、其余一轮收敛,
   未触 5 轮上限。
4. **收敛关键帧固化为 visual.spec 黄金图,进默认门像素回归** ✅ —— V4–V7 四张 +
   既有 V1/V3 按新观感重录;录制后连跑一致(确定性检过)。
5. **节奏确定性 native 测试(同 seed 逐 ms 相等,R8)** ✅ ——
   `rhythm_reader_release_times_deterministic_same_seed`(整机)+ rhythm.rs 成本层同 seed 逐位相等。
6. **progress 记录每 milestone 做了什么/验证方式/遗留** ✅ —— 本文即是。

### 交付总览(供作者跑完后决定提交)

- 核心新文件:`crates/core/src/{motion.rs,rhythm.rs}`、`crates/render/src/shaders/markdown/{pulse,badge}.wgsl`、
  `web/tests/{design-review,motion-idle}.spec.ts`。
- 主要改动面:reveal.rs(毫秒赤字调度)、app.rs(词拍释放/生命感/卡片面板/5 格条/hover/morph/
  anim_groups)、boxlayout(居中列+token 间距)、glyph/panel/widget.wgsl(曲线/流光/呼吸/徽章)、
  wasm(set_motion/set_rhythm/set_reveal_preset/set_pointer*/settled 透出)、web(style-panel
  Motion+Rhythm 节、copy 按钮 meta 化、boot 预设、input hover)。
- 全程未 commit(GOAL 铁律);工作区即交付。
