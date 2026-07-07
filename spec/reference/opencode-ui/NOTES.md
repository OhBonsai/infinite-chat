# opencode UI 参考真值注记(Plan 28 R0)

- 参考仓库:`~/w/agentscode/opencode` @ `4ddfa7c6`;样式真值 = `packages/ui`(web/desktop 同源)。
- 捕获方式:storybook **Timeline Playground**(`packages/ui/src/components/timeline-playground.stories.tsx`,
  渲染**生产级** `SessionTurn` + 官方夹具)→ Playwright 截图 + `getComputedStyle` dump。
  脚本:`web/scripts/capture-opencode-ref.mjs`;dark scheme;1280×900 dpr=1;动画禁用后截 settled 帧。
- 产物:`tokens.global.json`(theme.css :root 静态解析,light+dark)/ `tokens.live-dark.json`
  (live 解析值,**以此为准** —— class-based dark 与 media-query 块有差异,如 `--background-base`
  live=#121212 vs 静态 #101010)/ `computed/<scene>.json`(逐节点盒+样式)/ `shots/`(8 场景;
  超视口场景为切片目录)。
- **许可**:opencode MIT。本包只收数值/结构/截图对拍用,不搬其源码与资产;图标形状 SDF 自绘。

## 捕获注意事项(复现/复核用)

1. storybook 起法:`cd ~/w/agentscode/opencode/packages/storybook && bun run storybook`(:6006)。
   冷启动 vite 按需编译大,首载给足预热(脚本已含 5s 预热页)。
2. **本地夹具修补**(不影响样式):`timeline-playground.stories.tsx:1223` 的 `provider.all`
   story 给了数组而 `message-part.tsx:1073` 期望 Map(`provider?.all?.get`)→ 本地改为 Map,
   未提交到参考仓库。
3. `content-visibility:auto` + defer 懒挂载 → 屏外内容空白;长场景截图用**视口切片**(滚动真滚动
   容器 = playground 主预览区,`stories.tsx:1964`),每片 100px 重叠。
4. **已知缺口**:playground 的 `FileComponentProvider` 用 stub → edit/write 卡 body 显示
   "File viewer stub",**diff 视图像素真值缺**;R3-diff 时从 `packages/app` 的 file/diff 组件
   源码静态提取(`session-turn-diff-view` 的 CSS 在 `session-turn.css:196`)。

## 0. 页面骨架(真 app,非 playground 包装)

- 时间线列:**居中,md+ `max-w-200`(=50rem=800px),2xl `max-w-[1000px]`,`mx-auto`**
  (`packages/app/src/pages/session/message-timeline.tsx:1095-1097`)。行侧 padding
  `px-4 md:px-5`(16/20px,`:1118`)。1280 视口 → 列 800px 含 20px 侧衬。
- turn 间距:`[data-slot="session-turn-list"] { gap: 24px }`(`session-turn.css:230`)。
- assistant 一轮内 part 间距:`session-turn-assistant-content { gap: 12px }`(`session-turn.css:86`)
  + `assistant-message { gap: 12px }`(`message-part.css:1-8`)。
- 背景:列容器 `--background-stronger`(dark live=#151515);正文底无卡(文字直接铺底)。

## 1. user 消息(`message-part.css:10-208`)

右对齐气泡,**无头像列**(参考无 orb —— R1 移除我方 orb 列):

| 项 | 值 |
|---|---|
| 盒 | `width:fit-content; max-width:min(82%, 64ch); margin-left:auto` |
| 底/框 | `background: var(--surface-base)`(dark live=rgba(255,255,255,0.031));`border:1px solid var(--border-weak-base)` |
| padding / 圆角 | `8px 12px` / `6px` |
| 字 | sans 14px(`--font-size-base`)regular,`line-height: 150%`,色 `--text-strong` |
| 附件 | 气泡上方,右对齐 flex-wrap,gap 8px;图 48×48,文件 220×48 圆角 6 |
| meta/copy 行 | 气泡下 `margin-top:4px; min-height:24px`,右对齐,**hover 才显**(opacity 0→1,0.15s) |

## 2. assistant text part(`message-part.css:210-252` + `markdown.css`)

- **无卡无底**:正文直接排,`text-part` 仅是 markdown 容器。
- markdown(`markdown.css:1-266`):正文 14px/150% `--text-base`(dark live≈rgba(255,255,255,0.618));
  **strong/标题用 `--text-strong`**(更亮 —— 层级靠亮度不靠字号跳变);h1-h4 字号接近正文
  (见 computed s2-markdown dump);行内 code mono 13px 无底色、亮色;代码块:
  `--surface-inset-base`(dark live=rgba(0,0,0,0.5))圆角 6 边框 `--border-weak-base`,
  内衬 12px,mono 13px;引用:左 2px 竖线 `--border-weak-base` + 文弱化;列表标记 `--text-weaker`。
  (精确到 px 的逐元素值 → `computed/s2-markdown.json`。)

## 3. thinking / reasoning

- **"Thinking" 状态行 = turn 级合成**(非 part):spinner 16×16 + shimmer 文字,行高 20px,
  gap 8px,`margin-top:12px`,sans 14px medium `--text-weak`;heading 部分 `--text-weaker` regular
  (`session-turn.css:44-70`;TextReveal travel=25 duration=700,`message-timeline.tsx:166`)。
- **reasoning part 正文**(`message-part.css:280-312`):同 markdown 但整体弱化(`--text-weak` 系),
  无折叠、无 copy、无 label(极简;调研 §2)。

## 4. tool 卡(`basic-tool.css` + `message-part.css:313-556`)

结构 = `Collapsible`:trigger 行(图标+title+subtitle+args+chevron)+ body(output 面板)。

| 项 | 值 |
|---|---|
| trigger 行 | `tool-trigger`(`basic-tool.css:1`):flex gap 8px,title sans 14px medium `--text-strong`,subtitle `--text-weak`,args `--text-weaker`;行本身**无底**(hover `--surface-base`,0.15s) |
| bash 输出 | `bash-output`(`message-part.css:358`):`$ cmd` + 输出 mono 13px,面板 `--surface-inset-base` 底、`--border-weak-base` 1px 框、圆角 6、内衬 12px;`[data-scrollable]` 纵向裁切 |
| edit/write | trigger = Edit/Write + 文件名(strong)+ 路径(weak)+ `DiffChanges`(+N 绿 `--text-diff-add-base` / −N 红 `--text-diff-delete-base`,`diff-changes.css`);body = 文件卡(标题条 + 内容区) |
| 四态 | pending/running:title 走 **TextShimmer**、args/output 隐、禁展开;completed:静;error:`tool-error-card` 红系(`--text-on-critical-base`/`--surface-critical-weak`) |
| task(子代理) | 胶囊卡 `task-tool-card`(`basic-tool.css:173`):agent 名着色(tone),边框圆角胶囊,running 有 Spinner |
| webfetch/websearch | 单行:label strong + url/query `--text-base`,外链图标;无卡无底 |

## 5. context 折叠组(`message-part.css:623-657`)

连续 read/glob/grep/list → 一行 "Explored N read, M searches"(settled 文案;进行中 "Exploring…"):
trigger sans 14px,"Explored" medium strong + 计数 `--text-weak`;展开列表
`context-tool-group-list`:缩进列表,每行一个工具调用,弱化色。**无卡无底**。

## 6. permission/question 卡(`message-part.css:717-1148`)

`dock-prompt`(叠在 composer 上方的坞,**非流内**;我方 plan27 已流内化 —— 观感抄卡体,不抄位置):
- `[data-kind="permission"]`(`:717`):卡 `--surface-raised-base` 底、`--border-weak-base` 框、
  圆角 8、内衬 12-16px;标题行 strong;按钮排右下:Allow(primary 深底白字)/ Deny(secondary)。
- `[data-kind="question"]`(`:829`):同卡体 + 选项列表(radio/checkbox 行,hover `--surface-base`)
  + 自由输入 input(`--input-base` 底 圆角 6)+ 底部按钮行;`question-answers`(`:1127`)=
  已答 chips(小胶囊,`--surface-weak` 底)。
- 精确数值 → `computed/s6-question.json`(playground 的 question 工具卡形态)。

## 7. compaction(`message-part.css:253-279`)

`MessageDivider`:整宽分隔线 + 居中小字 label(`--text-weaker` 13px),上下留白;无卡。

## 8. 动效清单(R4 准绳;映射到我方 0025/0028)

| 参考动效 | 参数(file:行) | 我方映射 |
|---|---|---|
| PacedMarkdown 逐字揭示 | 24ms/批,词/标点边界 snap(`message-part.tsx:193-209`,调研§0.1) | 已超集:节奏系统(reader 预设)保留为增强层 |
| TextShimmer(pending title/Thinking) | step 45ms/字,sweep 1200ms,swap 220ms,spread 5.2ch,base `--text-weak` → peak `--text-strong`(`text-shimmer.css:1-18`) | WIDGET_PULSE/自定 shader 相位扫描 |
| TextReveal(thinking heading 换字) | travel 25px,duration 700ms(`message-timeline.tsx:166`) | 0016 morph / glyph 进场曲线 |
| hover 过渡 | 0.15s ease 通用(collapsible.css:7/29/38,accordion.css:44) | dur_element token ≈150ms |
| fadeUp 进场 | 0.4s ease-out,上移淡入(`animations.css:36-48`) | enter 曲线 + 6px rise(已有) |
| pulse-opacity / pulse-scale | `animations.css:6-35`(呼吸类) | WIDGET_PULSE |
| collapsible 展开 | 结构切换即时(slideUp/Down 被注释,`collapsible.css:92-99`),仅 chevron rotate 0.15s | 折叠即时重排 + morph 平滑 |

## 9. 状态矩阵(截图索引)

| 场景 | 覆盖 | 截图 |
|---|---|---|
| s1-user-variants | user 短/中/长气泡 | `shots/s1-user-variants.png` |
| s2-markdown | 标题/列表/代码/表格/混排 | `shots/s2-markdown/`(3 片) |
| s3-reasoning | thinking 行 + reasoning 正文 + 完整轮 | `shots/s3-reasoning/`(2 片) |
| s4-tools | bash/edit/write/task/webfetch/websearch/skill/todo | `shots/s4-tools.png` |
| s5-context-group | Explored 折叠组 | `shots/s5-context-group.png` |
| s6-question | question 工具卡(选项+已答) | `shots/s6-question.png` |
| s7-read-single | 单 read 工具 | `shots/s7-read-single.png` |
| s9-kitchen-sink | 全谱混排(页面节奏) | `shots/s9-kitchen-sink/`(7 片) |
