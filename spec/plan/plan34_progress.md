# Plan 34 · 进度账本(正确性/接入收口包)

> 逐 milestone 记录:做了什么 / 参考依据(file:行号)/ 遗留。
> GOAL:[plan34-correctness-integration-goal](./plan34-correctness-integration-goal.md)

## S1 · 滚动跟随 FSM(ADR + 三态 + 9 场景)

- **ADR**:[0038 scroll-follow-fsm](../decision/0038-scroll-follow-fsm.md)(GOAL 预计编号即中)——
  三态 Following/Released/Anchoring + 释放信号全集表 + 重进入条件 + 布局稳定不变量。
- **实现**(`crates/core/src/app.rs`):`FollowState` 枚举取代 `stick_to_bottom` 布尔;
  释放信号:`scroll_by(dy<0)`/`pan_by(dx≠0|dy<0)`/`zoom_by`/`set_selection(非空,新增)`/
  `scroll_to(jump)`;新 API `scroll_to_latest()`(Anchoring 平滑滚底)+ `follow_state()` 只读;
  wasm 导出同名两个(additive)。
- **修成文缺陷**:旧「距底 ≤48px 每帧被动复跟随」→ 重进入仅在 `scroll_down_intent`
  (用户主动向下,一帧有效)且抵底时发生;内容增长永不触发重进入。
- **行为等价**:Following == 旧 true 路径(smooth-damp/超一屏直跳原样);全 golden 不变
  (gate 全绿佐证);Anchoring 复用同一 damp 通道。
- **测试**:native 6 个 `follow_fsm_*` 覆盖 shadcn 9 场景(滚轮/键盘=scroll_by、
  滚动条/触摸=pan_by 两轴、缩放、选区、jump 六信号枚举 + 贴底/增高零位移/jump 回底/
  选区期间增长/重进入吸附);e2e 1 个(像素级零位移断言 + scroll_to_latest 恢复跟随)。
- **验收帧**:`test/results/plan34/s1-released-dpr{1,2}.png`。
- **踩坑**:①测试正文用 `\n\n` 分段(单 `\n` 被 markdown 并段 → 高度不超视口);
  ②e2e 必须 bootVisible 等 settle(固定 sleep 下 showcase 未放完,流式揭示被误判为位移)。
- **遗留**:jump-to-latest 按钮 UI(宿主侧;引擎 API 已备);新回合锚顶 peek(shadcn 3,
  记 research 增强档,非本 GOAL DoD)。

## S2 · 可点链接

- **sidecar**(`content.rs`):`LinkRegion{range,url}` + `extract_links(&spans)` —— 从 jcode 的
  ` (url)` Dim 后缀配对提取(「Link run + 紧邻后缀」),**spans 不改**(URL 不进 glyph 流,
  AR10 旁传);流式半截链接被 prepare_stream 剥除 → 无 sidecar 天然不可点。
- **vendor 修复**(`jcode-render-core/markdown.rs`):段落路径链接文字此前漏 Link 角色
  (只有表格路径有)→ 镜像表格逻辑补上;链接文字自此显 --text-interactive-base #c0d4fb
  (native 快照零变化 = 既有夹具无段落链接)。
- **命中**(`app.rs`):`BlockCache.links`;build_frame **只收 settled 块**(AR3)按行分段登记
  `link_targets`;tap 第三级命中(ask → fold → link)→ `pending_open_url`(core 不导航,
  CR5/0000 §2.2);hover(pointer 在行段内)→ 下划线 FrameRect(theme 新字段
  `link_underline` #c0d4fb,serde default 兼容旧 JSON)。
- **wasm**(additive):`set_open_url_handler(f)`(tap 命中链接 → 取走 pending → 回调
  onOpenUrl)、`link_hit_at`(hover cursor)、`link_hit_targets`(e2e/调试 JSON)。
- **宿主**(boot.ts):默认 handler = `window.open(url,"_blank","noopener,noreferrer")`。
- **踩坑(架构级)**:0030 透明文本镜像层的 span `pointer-events:auto` **截获了文本处的全部
  指针事件** —— 链接恰在文本上,canvas 的 input.ts 永远收不到。修:text-layer 增
  click(≤4px 同规)+ pointermove 转发(`TextHost.tap/set_pointer/link_hit_at` 可选成员),
  拖拽(>4px)= 选区不转发;span cursor 命中链接时切手型。
- **测试**:native 3(sidecar 提取/半截不产/engine tap→pending 取即清);e2e 2
  (真实鼠标点击 → 回调收精确 URL + 半截无命中盒;拖选链接文字不误触发)。
- **验收帧**:`test/results/plan34/s2-link-hover-dpr{1,2}.png`(hover 下划线可见)。
- **遗留**:表格内链接 jcode 不产 url 后缀 → 本期不可点;autolink `<url>` 未验(应同路)。

## S3 · URL 白名单

- **core**(`urlpolicy.rs`,新模块,纯函数 CR1):`UrlPolicy{link_prefixes,image_prefixes,
  default_origin}`;默认表仅 `https:`/`http:`/`data:image/`。规范化堵绕过:剥 `\t\n\r`
  (`java\tscript:` 注入)、scheme 大小写不敏感、相对 URL 仅配 defaultOrigin 才放行。
- **链接拒绝路径**(ensure_layouts):白名单外不进 sidecar(无命中盒/无 hover)+ 区间角色
  降级 Normal(视觉普通文本);tap 层 belt-and-suspenders 再验一次(策略热切窗口)。
- **图片拒绝路径**(sync_image_registry):登记即 `Failed`(alt 兜底,plan14 既有路),
  `take_pending_images` 永不吐白名单外 URL。
- **wasm**:`set_url_policy(json)`(camelCase 三键,缺省并默认表;非法 JSON 忽略 AR12);
  改表使全部块缓存失效(降级裁决在排版期)。
- **文档**:0006 §10 + 0007 尾节 append(策略节,不新开 ADR,GOAL 指定)。
- **测试**:native 3 —— 协议矩阵(16 恶意/5 放行,含大小写/控制字符/相对/超长不 panic)、
  defaultOrigin/自定义表/空表全拒、engine 级(恶意链接无盒+降级+好链仍 Link;file:// 图
  不进待解码队列)。
- **遗留**:defaultOrigin 只做「放行相对 URL」,实际解析交宿主(未拼接);blob:/about: 默认拒。

## S4 · 流式 a11y 播报

(未开始)

## S5 · Vue harness

(未开始)

## DoD 对账

(未开始)
