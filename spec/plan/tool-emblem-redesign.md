# Tool 纹章重设计 —— 巴黎 2024 设计思路解读 + 生成提示词

- 日期:2026-06-22
- 缘起:第一版 tool icon(`tool-icons.frag/.wgsl`)进了 gallery 但太弱。重做方向:**极客 · 抽象 · 运动**,母语借**巴黎 2024 奥运纹章**。
- 产物:① 本文(设计思路解读 + 重设计方向)② `tool-emblem-prompt.md`(可直接喂模型的生成提示词)。

---

## 1. 巴黎 2024 到底做对了什么(设计思路解读)

巴黎 2024 把"象形图"(pictogram)直接废掉,改叫 **coat of arms / blason(纹章)**。品牌总监原话:"象形图是过去式"。不是画"一个人在做这个运动",而是给每个项目一枚**徽章**——像中世纪北欧盾徽,代表一个"运动家族"的荣誉、归属、价值。粉丝能戴在身上。

**三层合成法**(这是可复用的配方):

1. **器械/动作** —— 该项目的标志性装备或动作。
2. **场域** —— 该项目场地/赛道/球场的**独特特征**(网、跑道、泳道、靶环…)。
3. **对称轴** —— 把上面两层用**旋转/镜像对称**绕中心熔成一枚徽章。

再叠加的气质来源:**Art Deco(装饰艺术)的强轴对称 + 阶梯/放射**、**1920s 立体主义/奥费主义(Cubism/Orphism)的几何分面与同心节奏**、**1924 巴黎奥运**致敬、**法式规则花园**的对称。目标之一是**无性别/抽象**——所以全是几何,不是人形。

**代价(诚实记下)**:为了徽章美感,牺牲了"一眼看懂"。专家批评其可读性远低于传统象形图。⇒ **重设计要吸收它的合成法与气质,但补回可读性**(靠"运动=动势"这条我们独有的轴)。

> 来源见文末。

---

## 2. 为什么这套思路天然适配我们(shader + tool)

巴黎把"运动"编码成一条**静态轨迹线**(球的弧线、桨的划水)。我们是 **shader 原生**,可以让那条轨迹**真的动起来**——这正是 brief 里的"运动"。于是:

> **巴黎的纹章 = 静止的对称徽章;我们的纹章 = 会呼吸/旋转/扫描的对称徽章。** 动势既是装饰,又把可读性补回来(动作即动词,tool 恰好都是动词)。

"极客"的注入:把巴黎的"场域+器械"换成**计算机母题**——终端边框、命令提示符、光标块、花括号 `{}`、尖括号 `<>`、正则、diff 的 `±`、文件树星座、网格/位场、电路走线、扫描线。仍走 SDF 几何,仍轴对称,但符号系是 geek 的。

"抽象":不画写实物件,做**立体主义式的几何归约 + 奥费主义同心节奏**——重叠的几何面、同心弧、放射轴,1-bit 下靠形与轴说话。

---

## 3. 重设计配方(每个 tool 一枚纹章)

**四要素**(前三来自巴黎,第四是我们的轴):

| 层 | 含义 | shader 手法 |
|---|---|---|
| **场域 Field** | 该 tool 的"赛场"(缓冲/文件树/网络/终端) | 背景栅格/边框/场地纹理(`rectSDF` 框、网格 `fract`、跑道环) |
| **器械 Instrument** | 该 tool 的"装备"(光标/透镜/钥匙/±) | 主形(`circleSDF/triSDF/cross/raysSDF`…) |
| **对称轴 Axis** | 绕中心镜像/旋转熔成徽章 | 折叠坐标:`st.x=abs(st.x-.5)+.5`(竖镜)、`rotate` 后取 N 次对称 |
| **动势 Motion** | 把轨迹**真的动起来**(我们独有) | `iTime` 驱动 sweep/orbit/ping/draw-on/breathe(plan16 运动词典) |

**对称策略**(Art Deco 灵魂):每枚至少一条对称轴(竖镜最稳);进阶用 2 折/4 折旋转对称(`rotate(st, k·TAU/N)` 叠加)做"花园式"徽章。

**调色**:沿用 gallery 的双色(青 `#3DF5D0` 前景 / 靛 `#2E1A6B` 背景)或严格 1-bit,二选一走 params(0021 Palette),不写死。

**tool → 场域/器械 映射速查**(给生成时定锚):

| tool | 场域 Field | 器械 Instrument | 动势 Motion |
|---|---|---|---|
| shell | 终端边框/命令行 | 提示符 `>` + 光标块 | 光标闪、回显流 |
| read | 文件缓冲(行号栏) | 扫描头 | 扫描线纵扫 |
| write | 空缓冲 | 笔尖/光标 | 字符落、caret 推进 |
| edit | 文本块 | 选区/替换标 | 高亮滑动 + 反相 |
| glob | 文件树星座 | 通配符 `*` | 星号旋 + 命中波纹 |
| grep | 文本行阵 | 正则透镜 | 命中行逐行点亮 |
| task | 进程空间 | 派生节点 | 子 agent 环绕/分形 |
| webfetch | 地球经纬网 | URL 锚/箭 | 数据包向心 |
| websearch | 网络场 | 查询射线 | 声呐离心 |
| todowrite | 清单栏 | 复选框 | 勾选推进 |
| skill | 能力槽 | 钥匙/模块 | 插入解锁旋 |
| apply_patch | diff 视图 | `±` hunk | 增删行流入 |
| question | 对话框 | `?`/分支 | 等待脉冲 |
| lsp | 符号/AST 图 | `{}` 探针 | 诊断点跳 |
| plan | 蓝图栅格 | 节点路由 | 折线描出→播放 |
| invalid | 空集 `∅` | 禁止环 | glitch 抖 |

---

## 4. 落地

- 生成走 `tool-emblem-prompt.md`:把巴黎合成法 + geek 母题 + shader 约束打包成一段提示词,模型按"四要素"逐 tool 产出 **纹章解读 + WGSL case**。
- 仍接 plan16 §2.5 管线(`icon_id` 50–65、`base/sdf.wgsl` 工具箱、`dynamic=true`)。
- 验收:每枚成立的"徽章感"(有轴对称、有场域+器械两层)+ 动势正确 + 缩放锐利;contact-sheet 比对第一版。

## 5. 来源

- [Paris 2024: The coat of arms for the Olympic Games (Bootcamp/Medium)](https://medium.com/design-bootcamp/paris-2024-the-coat-of-arms-for-the-olympic-games-df702145b450)
- [Paris 2024 is "reinventing the pictogram" (It's Nice That)](https://www.itsnicethat.com/news/paris-2024-olympic-paralympic-pictograms-080223)
- [The Dramatic Shift of the 2024 Paris Olympic Pictograms (Medium)](https://elijahcobb.medium.com/the-dramatic-shift-of-the-2024-paris-olympic-pictograms-45613c02790)
- [Diving into Designers' Favorite Olympic Sport: the Pictograms (PRINT Magazine)](https://www.printmag.com/branding-identity-design/olympics-pictograms/)
- [Paris Olympic Games 2024: Pictograms ... Coat of Arms Explained (Outlook India)](https://www.outlookindia.com/sports/others/paris-olympic-games-2024-pictograms-to-know-and-their-importance-history-what-is-new-coat-of-arms-explained)
