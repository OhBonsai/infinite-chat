# LoL《Salvation》风格参考包(Plan 40 P0)—— 数值真值,零 Riot 素材

> 铁律(承 plan28 第一 / plan40 §0):**抄气质与数值,零版权素材**。本包提取《Salvation》
> (2026 赛季开场 / For Demacia)与 LoL 官网 UI 的**色相 / 节奏 / 字距 / 构图**数值;
> **不用** Riot 字体(Beaufort/Spiegel)、图像、音频、纹章。字体全部 OFL 免费近似。
> 每个数值下游 `web/src/pages/pages-theme.css` 引用时标出处行。

## 1. 色板(hex,含语义与出处)

`《Salvation》定帧`= trailer 关键帧的主色调观察;`LoL brand`= 长期公开品牌色(League 客户端/官网)。

| token | hex | 语义 | 出处 |
|---|---|---|---|
| `--bg-abyss` | `#0a0e14` | 最深底(hero 暗部/页背景) | plan40 GOAL 深底系 |
| `--bg-deep` | `#0a1428` | 次底(卡片/区块底,德玛西亚夜蓝) | LoL brand 深蓝 hextech-black |
| `--bg-panel` | `#0f1a2e` | 面板底(略提亮,悬停/激活) | 派生(bg-deep +6% L) |
| `--gold` | `#c8aa6e` | 鎏金主色(描边/强调/当前态) | plan40 GOAL 鎏金;LoL brand gold #C8AA6E |
| `--gold-bright` | `#f0e6d2` | 亮金白(大标题字/高光) | LoL brand grey-cool #F0E6D2 |
| `--gold-dim` | `#785a28` | 暗金(次级边/分隔) | LoL brand gold #785A28 |
| `--gold-deep` | `#463714` | 深金(阴影侧描边) | LoL brand #463714 |
| `--blue-royal` | `#0596aa` | 皇家青蓝(链接/hextech 光) | LoL brand hextech blue #0596AA |
| `--blue-bright` | `#0ac8b9` | 亮青(glow/交互高光) | LoL brand #0AC8B9 |
| `--blue-deep` | `#005a82` | 深青(渐变暗端) | LoL brand #005A82 |
| `--platinum` | `#cdbe91` | 白金(eyebrow 小字/次强调) | LoL brand grey #CDBE91 |
| `--crimson` | `#c8283c` | 绯红点缀(极克制:警示/一处强调) | plan40 GOAL 绯红点缀;LoL 红 #C8283C |
| `--ink` | `#a09b8c` | 正文灰(LoL 正文常用暖灰) | LoL brand grey #A09B8C |
| `--ink-dim` | `#5b5a56` | 弱文本(说明/页脚) | LoL brand grey #5B5A56 |

配比铁律(构图):底 70% 深蓝黑 / 中 20% 金蓝渐变 / 高 8% 亮金 / 点 2% 绯红。绯红**只在一处**。

## 2. 字形语汇(免费字体,OFL)

| 角色 | 字体 | 许可 | 气质出处 | 数值 |
|---|---|---|---|---|
| display 大标题 | **Cinzel** | OFL(免费可分发) | Trajan/Beaufort 罗马大写衬线气质近似(Beaufort 本体版权,不用) | 全大写,字距 +0.08em,字重 600–800 |
| eyebrow 小标 | **Cinzel** 或系统 | OFL | 全大写小字距金字幕 | 12–13px,字距 +0.24em,uppercase,--platinum |
| 中文标题/正文 | **LXGW WenKai** | OFL | 温润楷体(与衬线 display 调性合) | 字距 +0.02em |
| 正文 sans | 系统 UI 栈 | — | 工程站可读性(信息密度是内容主体,§6 风险) | 15px/1.7 |
| 代码 | ui-monospace | — | 引擎同款等宽 | 13px |

> 字体文件加载策略(P1/P5):Cinzel/LXGW 用 woff2 子集入库(仅标题字符集),体积预算记账;
> 缺失回退 `serif` / `"PingFang SC"`。**不引 Google Fonts CDN**(离线优先 + 无外部依赖)。

## 3. 动效语汇(时长/曲线数值)

| 动效 | 数值 | 出处 | 引擎映射 |
|---|---|---|---|
| 慢推镜(hero) | 缩放 1.0→1.06,**4.5s**,ease-in-out,循环 | Salvation 缓慢推镜观察 | 相机 zoom 注入 dt(可 golden 定帧) |
| 标题入场 | dissolve-in 900ms + 上浮 12px | plan36 dissolve / spring | set_effect_preset('expressive') |
| 金线描边入场 | stroke-dasharray 扫描 1.2s | LoL UI 金线勾边 | CSS(卡片边)/ 引擎 edge_glow |
| 雾层视差 | fbm noise 慢平移,0.02px/ms | plan36 noise.wgsl fbm | ShaderBox/背景 noise |
| 字幕式淡入(滚动) | opacity 0→1 + translateY 16px,600ms,IntersectionObserver | Salvation 字幕淡入 | CSS + IO |
| 光尘粒子 | 稀疏上浮,极慢 | Salvation 光尘 | 可选(引擎 idle 纪律 ≤4 动画组) |
| hover glow | box-shadow 金 8px→16px,200ms | LoL 卡悬停 | CSS |

## 4. 构图

- **hero**:全屏引擎 canvas(暗部三分留白),标题居中偏下三分线,eyebrow 在标题上方。
- **区块**:居中列 max-width 1100,鎏金细分隔线(1px --gold-dim,两端渐隐)。
- **卡片**:鎏金 1px 描边 + 深底,hover 金 glow;圆角小(4px,LoL 硬朗感,非圆润)。
- **导航**:顶部细条,当前态金线下划线。

## 5. 版权注记

《Salvation》/ League of Legends 为 Riot Games 商标,本包**仅数值化其公开气质**用于设计对照,
**站内零 Riot 资产**(字体/图像/音频/纹章)。grep 断言见 plan40 §6 pages smoke。
