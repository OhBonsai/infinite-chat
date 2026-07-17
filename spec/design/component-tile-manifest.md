# Plan 44 · 组件 Tile 墙 —— 全清单 + 尺寸设计表(T0 交付,作者过目点)

- 日期: 2026-07-17
- 契约: [decision 0042](../decision/0042-tile-grid-layout-mode.md)(tile 网格布局模式)
- 用途: `/components/` 两面墙的 manifest 真值(内容写死 = R8 确定性)+ 首页 S3/S4 幕的局部框取素材。
- 基网格: **4 列**,`gap` 统一,固定 `row_h`;span 异构、边缘落网格线(0042 §2.1)。

## 0. 尺寸档(容量四问定档,写死)

| 档 | span (列×行) | 用途 | 容量判据(huashu 四问) |
|---|---|---|---|
| 大格 | 4×2 | diff 卡 / 表格 / 代码块 | 需宽(多列/长行)+ 高(多行);一屏一个焦点 |
| 中格 | 2×2 | tool-bash / ask / 块级公式 / 代码折叠 | 需高不需满宽;命令输出/选项/公式竖向展开 |
| 横条 | 4×1 | compaction / hr 族 | 满宽一线,天然横向 |
| 标准 | 2×1 | 多数(气泡/正文/列表/引用/Alert…) | 半宽一段,墙的主旋律 |
| 小格 | 1×1 | 链接 / 行内公式 / emoji | 一词一符,点睛 |

## 1. Agent 卡墙(9 tile,≥9 达标)

> 内容 = `message.updated`(建消息)+ `message.part.updated`(part.time=1)注入;tile `id` = messageID。

| # | id | span | 标题(Mono) | 内容(写死) |
|---|---|---|---|---|
| 1 | `a-user` | 2×1 | USER | user 气泡:`把首页六幕重构成组件墙,tile 要 SDF 真渲染。` |
| 2 | `a-asst` | 2×1 | ASSISTANT | assistant 正文(markdown):`已拆成 **4 列基网格** + skyline 摆位。每格是真 panel 图元,缩放锐利。` |
| 3 | `a-reason` | 2×1 | REASONING | reasoning 卡:`先确认 span 设计表 → 再写布局器 → 恒等硬门先验。` |
| 4 | `a-bash` | 2×2 | TOOL · BASH | tool bash `completed`:input `{cmd:"cargo test -p core tile"}`,output `test result: ok. 12 passed`(多行) |
| 5 | `a-read` | 2×1 | TOOL · READ | tool read `running`(shimmer 流光):input `{path:"crates/core/src/tilelayout.rs"}` |
| 6 | `a-diff` | 4×2 | TOOL · EDIT | tool edit `completed` + `metadata.filediff`(折叠态,**可点展开**):`src/tilelayout.rs` 的 skyline 补丁(+8/−2,含上下文行) |
| 7 | `a-ask` | 2×2 | ASK | ask 交互卡(permission):question `允许写入 tilelayout.rs?` + allow/deny 真按钮矩形 |
| 8 | `a-compact` | 4×1 | COMPACTION | compaction 分隔:`— 上下文已压缩 · 保留 12 条 —` |
| 9 | `a-error` | 2×1 | ERROR | 错误兜底卡:`APIError: rate limited (429) — 已重试 3 次` |

**摆位(skyline,4 列)**:r0 `a-user`(c0-1)+`a-asst`(c2-3);r1 `a-reason`(c0-1)+`a-read`(c2-3);
r2-3 `a-bash`(c0-1)+`a-ask`(c2-3);r4-5 `a-diff`(c0-3);r6 `a-compact`(c0-3);r7 `a-error`(c0-1)。

## 2. Markdown 墙(19 tile,≥16 达标)

> 内容 = 单 text part(markdown 源写死);tile `id` = messageID。代码/表格/公式/图片/SVG 走引擎既有富渲染。

| # | id | span | 标题 | 内容(markdown 源,写死) |
|---|---|---|---|---|
| 1 | `m-head` | 2×1 | HEADINGS | `# H1 标题\n## H2\n### H3` |
| 2 | `m-emph` | 2×1 | EMPHASIS | `**粗** · *斜* · ~~删~~ · \`行内码\`` |
| 3 | `m-link` | 1×1 | LINK | `[GitHub ↗](https://github.com/OhBonsai/infinite-chat)`(白名单内可点) |
| 4 | `m-ul` | 2×1 | LIST · UL | `- 一\n- 二\n  - 嵌套` |
| 5 | `m-ol` | 2×1 | LIST · OL | `1. 一\n2. 二\n3. 三` |
| 6 | `m-task` | 2×1 | TASKS | `- [x] 已完成\n- [ ] 待办` |
| 7 | `m-quote` | 2×1 | QUOTE | `> 引擎在跑,你看到的就是宣传片本身。` |
| 8 | `m-note` | 2×1 | ALERT · NOTE | `> [!NOTE]\n> SDF 渲染,任意缩放锐利。` |
| 9 | `m-warn` | 2×1 | ALERT · WARN | `> [!WARNING]\n> 前沿留 hold 区,不提交歧义字节。` |
| 10 | `m-code` | 4×2 | CODE BLOCK | `rust` 高亮 + 行号:`fn place(t:&Tile)->Rect{ /* skyline */ }`(~8 行) |
| 11 | `m-codefold` | 2×2 | CODE · FOLD | 超长代码块(~40 行)→ 折叠成可滚行窗(视口内滚动) |
| 12 | `m-table` | 4×2 | TABLE | 4×3 表(对齐 + CJK):能力/手法/规模(同 plan43 S3 表) |
| 13 | `m-hr` | 4×1 | RULE | `分节线 ——`\n`---`\n(hr + 标题线) |
| 14 | `m-foot` | 2×1 | FOOTNOTE | `正文[^1]\n\n[^1]: 脚注内容。` |
| 15 | `m-imath` | 1×1 | MATH · INLINE | `质能 $E=mc^2$` |
| 16 | `m-bmath` | 2×2 | MATH · BLOCK | `$$\sum_{i=1}^{n} i = \frac{n(n+1)}{2}$$` |
| 17 | `m-img` | 2×1 | IMAGE | `![tile](data:image/… 内嵌小图)`(浏览器解码 → 纹理) |
| 18 | `m-svg` | 2×1 | SVG | 内嵌 SVG 矢量(streaming-svg,0036) |
| 19 | `m-emoji` | 1×1 | EMOJI · CJK | `字 🌸 混排 ✅` |

**摆位(skyline,4 列)**:标准 2×1 双双成行;`m-link`/`m-imath`/`m-emoji` 1×1 填缝;`m-code`/`m-table` 4×2
各自独占两行;`m-codefold`/`m-bmath` 2×2 成对;`m-hr` 4×1 横贯。具体行号由布局器 skyline 确定性算出(native 断言逐 tile 校验四边落网格线)。

## 3. 降级(无 GPU)

同设计表的 **pages-assets 网格幻灯**:每墙一张网格截图(T2 用 design-review harness 出图),布局对齐同 manifest。

## 4. 待作者过目点

- **span 设计表**(§0 五档 + §1/§2 逐 tile span)是否符合「异构但对齐、一处 120%」的画面预期?
- **内容清单**(§1 九卡 / §2 十九型)是否覆盖需求且无冗余?
- 过目通过 → T1 引擎布局器动工(0042 契约已冻结)。
