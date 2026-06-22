# Plan 16 附 — opencode 内置 tool 的 animated icon 设计

- 日期:2026-06-22
- 前置:[plan16 §2.5 内置 animated icon 库](./plan16-shaderbox.md)、[0028 ShaderBox](../decision/0028-shaderbox-primitive.md)
- 源:`opencode/packages/opencode/src/tool/registry.ts`(builtin 清单)+ 各 `*.txt` 描述
- 一句话:为每个内置 tool 设计一枚 **1-bit 黑白、SDF 自画、shader 驱动**的 animated icon,沿用 PixelSpiritDeck 调性(`stroke`/`fill`/`step`、`S/C` 呼吸、`iTime` 旋转/morph);落地为 `icon_id ≥ 50` 追加进 `icons.wgsl`(plan16 §2.5c),先以 GLSL contact-sheet 验收。

---

## 0. 来源:内置 tool 清单(registry.ts §219–236)

按 registry `builtin` 数组顺序(+ 条件开关):

| # | tool id | 语义(来自 `*.txt`) |
|---|---|---|
| 1 | `shell` | 跑命令(bash) |
| 2 | `read` | 读文件/目录 |
| 3 | `glob` | 文件名 pattern 匹配(`**/*.ts`) |
| 4 | `grep` | 文件**内容**正则搜索 |
| 5 | `edit` | 精确字符串替换 |
| 6 | `write` | 写文件 |
| 7 | `task` | 派生 subagent 跑多步任务 |
| 8 | `webfetch` | 取单个 URL 内容 |
| 9 | `todowrite` | 维护结构化任务清单 |
| 10 | `websearch` | 联网搜索(条件:provider 支持) |
| 11 | `skill` | 载入专用 skill(注入指令/资源) |
| 12 | `apply_patch` | diff 格式编辑(gpt 系) |
| 13 | `question` | 向用户提问 |
| 14 | `lsp` | LSP 代码智能(experimental) |
| 15 | `plan-enter` / `plan-exit` | 进/出 plan 模式(experimental·cli) |
| 16 | `invalid` | 内部错误/未知 tool(兜底) |

> `edit`/`write` 与 `apply_patch` 互斥(同一模型只挂其一),但 icon 都设计;`lsp`/`plan`/`question` 条件挂载;`invalid` 内部用,设计作彩蛋。

---

## 1. 头脑风暴:为什么这样选(shader-first 原则)

**约束即灵感**。这套图标不是"画好的 SVG 塞进去",而是**一段片元逐像素算出来**。所以设计时反推 shader 能廉价表达什么:

1. **1-bit 是特征不是限制**。只有"亮/暗",于是语义全靠**形状 + 运动**,不靠颜色/描边粗细层级。这正是 PixelSpiritDeck 的母语:`step`/`stroke`/`fill` 出锐利硬边,缩放永远清晰(SDF 本性)。
2. **运动是第二语义轴**。静态图标靠"长什么样"区分,animated 图标多一条轴:"怎么动"。读=扫描线下移、搜=透镜扫过、派生=卫星环绕——**动作本身就是动词**,而 tool 恰好都是动词。这是把"图标"升维成"图示"的关键。
3. **每个动作映射一条廉价 shader 习语**(见 §2 运动词典)。不引入噪声/多 pass/raymarch——全是 `fract`、`rotate`、`atan` 一两行,符合 plan16 五护栏的"便宜片元"。
4. **同源工具箱**。所有图标只用 plan16 §2.5a 的 `circleSDF/rectSDF/triSDF/stroke/fill/rotate/raysSDF/crossSDF/polySDF/vesicaSDF/rhombSDF/spiralSDF/scale` + `S/C`,外加一个 `seg()`(点到线段距离,画折线/路由用)。零新依赖。
5. **灵感杂食**:终端光标的闪烁、声呐的扩散环、地球仪的经线、钥匙开锁的微转、流程图的判定菱形、no-entry 禁止符的抖动——都是**集体记忆里的强符号**,1-bit 下依然一眼认出。借符号,不抄具体作品。

**区分度是第一难点**(16 个挤在一套黑白语言里别撞):
- `read`/`write`/`edit` 同为"纸面操作" → 用**不同动作**分开:read=扫描线**下移**、write=笔尖**横移留迹**、edit=高亮框**滑动并反相**。
- `glob`/`grep` 同为"找" → glob=**星号 `*`** 旋转(文件名通配符,符号直给)、grep=**透镜**扫内容行(放大镜=内容搜索)。
- `webfetch`/`websearch` 同为"网" → fetch=地球仪 + **向心**飞入一个点(取回单个)、search=**离心**声呐扩散环(广播查询)。方向相反,语义自明。
- `task`/`plan` 同为"编排" → task=**卫星环绕**(并发派生)、plan=**折线路由**逐段画出(顺序规划)。

---

## 2. 运动词典(shader 习语 → 语义)

每个 icon 从这张表里取 1–2 个驱动,保证便宜且协调(统一动效时钟,plan16 §2.3 护栏4,30fps):

| 习语 | 一行 shader | 读作 |
|---|---|---|
| **breathe 呼吸** | `S=1+sin(t)/4`、`C=1+cos(t)/4` 乘到半径/阈值 | 待命/存在感(环境光) |
| **sweep 扫掠** | `y = a + b*fract(t*k)` 当 stroke 阈值 | 扫描/逐行处理 |
| **rotate 旋转** | `rotate(st, t*k)` | 转动/运转/通配 |
| **blink 闪烁** | `step(.5, fract(t*k))` | 光标/等待输入 |
| **orbit 环绕** | `vec2(cos(a),sin(a))*r`,`a=t*k+i·TAU/n` | 并发/派生 |
| **ping 扩散** | 循环 `stroke(r, fract(t+i/n), w·(1-phase))` | 广播/查询外部 |
| **draw-on 描出** | `mix(a,b,clamp(t…))` + `seg()` 揭示 | 逐步构建(规划/写入) |
| **flip 反相** | `flip(c, mask)`(deck 原语) | 替换/选中/对调 |
| **jitter 抖动** | `step(.5, fract(t*6+sin(t*13)))` | 错误/失效 |

---

## 3. 逐 tool icon 设计(id 从 50 起,接 plan16 §2.5c)

> 格式:**主形(SDF 原语)** · **运动(词典)** · 一句立意。contact-sheet tile 号见括号。

**50 `read`**(tile 0)— 纵向纸张 `rectSDF` 描边 + 一条水平**扫描线** `stroke(st.y, …)`,阈值 `sweep` 从顶到底循环,仅在纸内显示。*读 = 一行行扫过页面。*

**51 `write`**(tile 1)— 同纸张轮廓 + 一支**笔尖**(短竖 caret)沿基线 `sweep` 横移,**身后留下一条逐长的 stroke**(`step(st.x, penX)` 揭示),笔尖 `blink`。*写 = 落笔留迹。*

**52 `edit`**(tile 2)— 三条文本 `stroke` 横线 + 一个**高亮框** `rectSDF` 描边沿中行 `sweep` 滑动,框内 `flip` 反相底下的线。*编辑 = 选中一段、就地替换。*

**53 `grep`**(tile 3)— 居中**放大镜**:`circleSDF` 镜环 + `rotate(-45°)` 出的斜柄 stroke;镜内文本行 `sweep` 向上滚动(只在镜内 `fill` 掩码显示)。*grep = 透镜下逐行扫内容。*

**54 `glob`**(tile 4)— **星号 `*`**:`raysSDF(st,6)` 取 `stroke(…,.5)` 出 6 条辐;`rotate(st, t)` 慢转;圆盘 `fill` 掩半径 + 中心点;四角小方块 `blink` 暗示"命中的文件"。*glob = 文件名通配符在转。*

**55 `shell`**(tile 5)— **`>` 尖括号**(两条对角 `stroke` 在中点相交,源自 deck branch/hanged-man 习语)+ 右侧**下划线 caret** `blink`。*shell = 终端提示符在等命令。*

**56 `task`**(tile 6)— 中心**枢纽** `fill(circleSDF)` + 一圈淡 `stroke` 轨道 + **3 颗卫星** `orbit`(`rotate` 后落 circle)。*task = 主体派生并发子 agent。*

**57 `webfetch`**(tile 7)— **地球仪**:`circleSDF` 轮廓 + `scale(x)` 压扁的经线椭圆 + 赤道线(都 `fill` 掩在球内)+ 一个**点从右向心飞入**(`sweep` 反向)。*fetch = 从网上取回一份内容。*

**58 `websearch`**(tile 8)— **声呐**:中心发射点 + 循环 `ping` 扩散圆环(`stroke(r, fract(t+i/n), w·(1-phase))`,越外越细越淡)。*search = 向外广播一次查询。*(与 fetch 方向相反:离心 vs 向心。)

**59 `todowrite`**(tile 9)— 三行 **复选框 `rectSDF` + 任务线 `stroke`**;顶行的对勾用两段 `seg`/对角 stroke **`draw-on` 描出**(完成进度)。*todo = 勾掉一项。*

**60 `skill`**(tile 10)— **钥匙**:`circleSDF` 环(匙头)+ 横 `stroke` 匙杆 + 两道齿;整体 `rotate(sin(t)·小角)` 微转。*skill = 解锁一项专门能力。*

**61 `apply_patch`**(tile 11)— **diff `±`**:淡框 + 上行 `crossSDF` 的 **`+`**、下行单 `stroke` 的 **`−`**,二者 `blink` 交替加重(正在应用增删)。*patch = 加几行、删几行。*

**62 `question`**(tile 12)— 一个**问号 `?`**:`circleSDF` 上弧(用 `atan` 角度掩出 3/4 环)+ 短竖杆 + 下方点;整体 `breathe`。*question = 在问你。*

**63 `plan-enter`**(tile 13)— **折线路由**:3 个节点 `circleSDF`,节点间连线用 `seg()` **`draw-on`** 从左到右逐段画出(节点随线到达浮现)。*plan-enter = 把步骤铺出来。*

**64 `plan-exit`**(tile 14)— 同路由已画完(静态)+ 末端一个**播放三角** `triSDF`(`rotate` 指向右)`breathe`。*plan-exit = 计划就绪,开跑。*

**65 `invalid`**(tile 15)— **禁止符**:`circleSDF` 环 + 一道对角 `stroke` 斜杠(球内),整体 `jitter` 抖动。*invalid = 未知/失效 tool 的兜底。*

> **lsp** 复用 `question`/`grep` 家族:可作 **66 `lsp`** = 放大镜内嵌一个代码花括号 `{}`(两条对称 `stroke` 弧)+ 一个跳动诊断点。本轮先出上面 16 个核心,`lsp` 列为下一批(它 experimental)。

---

## 4. 落地 / 验收

- **先 GLSL contact-sheet**(本目录 `tool-icons.frag`):4×4 共 16 tile,复用 deck 同一套 toolbox + 网格 harness,可直接丢进 shadertoy 或本仓 shaderbox 源里跑。**验收**:每格形状成立、运动正确、缩放锐利。
- **后 WGSL**:按 plan16 §2.5 契约把 `draw()` 的 `switch` 译成 `icons.wgsl`,`icon_id` 50–65 追加在 deck 50 支之后;`dynamic` 标志——**16 个全 animated = true**(均用 time)。
- **注册表**:`IconId` 追加 `Read=50 … Invalid=65`;功能别名直接映射(`Copy`/`Check` 仍可指向 deck 内既有 morph,见 plan16 §2.5c)。
- **护栏**:全是便宜片元(无噪声/march),小盒 dynamic ≈ 可忽略(0028 §4 成本模型);多个同屏(工具条/调用流)时守"屏上活跃 box × 像素"上限(plan16 §6 Open ⑦)。

## 5. Scope · 不做

- ❌ 颜色/glow/灰阶(本轮严格 1-bit;调色后续走 0021 Palette)。
- ❌ `?` 等曲线字形追求像素级完美(SDF 近似即可,够认就行)。
- ❌ 一次铺满 lsp/更多 MCP tool(先 16 核心,留 66+ 续)。
- ❌ 鼠标 hover 预览(plan16 `iMouse` defer)。
