# 研究:2D SDF / 平面 shader 特效全局目录 —— 效果系统选型的全景视野

- 日期:2026-07-13
- 类型:research(深研目录;5 路并行调研汇总,链接均经抓取或搜索交叉验证)
- 目的:为 TODO2-A「效果系统」建立**全局视野**——历史上业界出现过的 2D/UI 向 shader 能力尽量全收,每项给可点开看效果/源码的地址。作者自看效果用,后续效果 plan 从这里选型。
- 验证标注:✓ = 调研中实际抓取正文核验;○ = 搜索结果标题/多来源交叉确认(Shadertoy 页面是 JS 应用,fetch 拿不到正文,只能交叉验证 ID)。
- 阅读法:§1–§7 是**能力目录**(值层:画什么);§8 是**与本引擎对照**(哪些已有/一步之遥/远期);§9 是**必看清单**(自看效果的最短路径)。

---

## 0. 全景一句话

一切 2D shader 特效 = **场(SDF/噪声/纹理)× 映射(距离带/阈值/扭曲)× 时间(缓动/弹簧/反馈)**。SDF 给"形",噪声给"机理",距离带给"装饰",域操作给"复制与变形",反馈缓冲给"历史(拖尾/流体)",逐实例属性给"编排"。本引擎已握有前四样的地基(0018/0025/0026/0028),缺的主要是**噪声家族入库、反馈缓冲通道、以及效果预设库本身**。

## 1. SDF 本体能力(词汇表层)

| 能力 | 一句话原理 | 链接 |
|---|---|---|
| 2D 图元全表(~70 个) | circle/box/segment/triangle/pie/arc/vesica/star/polygon/bezier/heart… 全有闭式 SDF | ✓ https://iquilezles.org/articles/distfunctions2d/ ;单图元 demo 例 ○ https://www.shadertoy.com/view/XdXcRB |
| 布尔/域操作总表 | min/max=并/差/交;opElongate/opRound/opOnion/opSym/opRep/opTwist/opCheapBend/opDisplace | ✓ https://iquilezles.org/articles/distfunctions/ |
| smin 全变体 | 多项式(C1/C2)/指数(N 元序无关)/power/root + 返回 blend factor 版(混材质) | ✓ https://iquilezles.org/articles/smin/ ;讲解 ○ https://www.shadertoy.com/view/Ml3Gz8 |
| interior distance | min/max 组合内部只是 bound;exact 内距靠轮廓分段/凹图元 | ✓ https://iquilezles.org/articles/interiordistance/ |
| rounding / onion | `d-r` 圆角;`abs(d)-t` 抽壳成环(可嵌套多层轮廓) | ✓ distfunctions 页内小节 |
| domain repetition | `p-c*round(p/c)` 一行无限平铺;有限重复 clamp 索引保 exact | ✓ https://iquilezles.org/articles/sdfrepetition/ ;demo ○ https://www.shadertoy.com/view/3tyBDW |
| 缓动小函数库 | almostIdentity/expImpulse/cubicPulse/expStep/gain/parabola/sinc(动画曲线的"标准零件") | ✓ https://iquilezles.org/articles/functions/ |
| domain warping | `f(p+fbm(p+fbm(p)))` 递归扭曲,云雾/大理石机理 | ✓ https://iquilezles.org/articles/warp/ ;demo ○ https://www.shadertoy.com/view/lsl3RH |
| distance estimation | 任意隐函数 `d≈|f|/|∇f|`,等宽描边/函数绘图 | ✓ https://iquilezles.org/articles/distance/ |
| 解析梯度图元表 | 图元同时返回 d 与 ∇d(AA/法线/流场) | ○ https://iquilezles.org/articles/distgradfunctions2d/ |
| fwidth AA | `smoothstep(±0.5*fwidth(d))` 标准抗锯齿 | ✓ https://www.ronja-tutorials.com/post/034-2d-sdf-basics/ |
| LYGIA 库 | sdf 模块(circle/cross/heart/hex/spiral/star/superShape…+op*)+ draw 模块(fill/stroke/flip/bridge) | ✓ https://github.com/patriciogonzalezvivo/lygia/tree/main/sdf · https://lygia.xyz/sdf |
| SDF 文字 | Valve 单通道 SDF → msdfgen 多通道(median 保尖角) | ✓ https://github.com/Chlumsky/msdfgen ;论文 ○ Valve SIGGRAPH2007 Green |
| 形状 morph | 逐像素 `mix(dA,dB,t)` 再阈值——SDF 的免费福利 | ○ https://www.shadertoy.com/view/3sSGDz · https://www.shadertoy.com/view/3dVyWz |
| metaball | 多圆 smin 融合成粘连团块 | ○ https://www.shadertoy.com/view/wd3SzS · IQ https://www.shadertoy.com/view/ld2GRz |
| Ronja 2D SDF 四部曲 | basics/combination(round/chamfer/stairs)/space manipulation(swirl/镜像/旋转重复)/shadows | ✓ ronja-tutorials.com post/034 ○ 035 · 036 · 037 |
| 综合工具箱 | 图元+combiner+outline+2D 光照+阴影一页全 | ○ https://www.shadertoy.com/view/4dfXDn (Shapes 2D) |

## 2. 距离带效果(装饰层——**你引擎效果系统的主通道**)

| 效果 | 原理 | 链接 |
|---|---|---|
| 描边 stroke/outline | `abs(d-w)<t` 取带 | ○ https://www.shadertoy.com/view/4lfyR2 ;LYGIA draw/stroke ✓ |
| 内/外发光 glow | 对 d 做 `k/d`、`exp(-k·d)` 衰减 | ○ https://www.shadertoy.com/view/3s3GDn (GLOW TUTORIAL)· https://www.shadertoy.com/view/WdK3Dz (NEON LOVE) |
| 解析软阴影 drop shadow | 高斯×box 卷积闭式解(erf),O(1) 单 pass | ✓ https://madebyevan.com/shaders/fast-rounded-rectangle-shadows/ (Evan Wallace,**Zed/Figma 同款,必读**) |
| 2D raymarch 软阴影 | 沿光线 march SDF 记 `min(k·d/t)` 半影 | ○ ronja post/037 · https://www.shadertoy.com/view/wtSBDG |
| AO 近似 | 邻域 d 映射为遮蔽 `clamp(d/r,0,1)` | 你引擎 panel 已有;理论 ✓ IQ distance 文 |
| 等距条纹 banding | `abs(fract(d/L+0.5)-0.5)*L` 周期化距离 | ✓ ronja 034 Height Lines 小节 |
| 色阶带 cel-band | `floor(d/L)` 量化查调色板 | ○ 4dfXDn contour 模式 |

## 3. 噪声与程序化家族(机理层)

| 能力 | 原理 | 链接 |
|---|---|---|
| value / gradient(Perlin) | 格点值/梯度插值 | ✓ https://thebookofshaders.com/11/ ;✓ https://iquilezles.org/articles/gradientnoise/ ;ref ○ shadertoy lsf3WH / XdXGW8 |
| simplex | 单纯形网格,N+1 角点,无方向伪影 | ✓ https://github.com/stegu/webgl-noise (Ashima 官方延续)+ 新版 https://github.com/stegu/psrdnoise/ |
| voronoi 族 | Worley F1/F2;smooth voronoi;voronoi edges;voronoise(noise↔voronoi 统一) | ✓ https://thebookofshaders.com/12/ ;✓ IQ smoothvoronoi / voronoise;○ https://www.shadertoy.com/view/ldl3W8 |
| fbm / turbulence / ridged | 多八度叠加;abs 变体出沟谷/山脊 | ✓ https://thebookofshaders.com/13/ ;✓ https://iquilezles.org/articles/fbm/ |
| curl noise | 势场取旋度 → 无散度流场(粒子永不聚死) | ○ Bridson 2007 论文;拆解 ○ https://emildziewanowski.com/curl-noise/ |
| blue noise | 高频随机,观感最匀;抖动/采样 | ✓ https://momentsingraphics.de/BlueNoise.html (CC0 纹理库) |
| hash | Dave Hoskins hash-without-sine(跨平台稳);PCG 整数 hash | ○ https://www.shadertoy.com/view/4djSRW ;论文 ○ jcgt.org/published/0009/03/02 |
| 图案:truchet/hex/极坐标/棋盘 | 网格 hash 旋转瓦片;错位双格取最近;atan 周期 | ○ https://www.shadertoy.com/view/llBBWG ;IQ hex ○ https://www.shadertoy.com/view/Xd2GR3 ;✓ thebookofshaders.com/09 |
| 生成美学经典(待机/背景灵感) | plasma(sin 叠加)/tunnel(极坐标 1/r)/Star Nest(Kaliset 分形)/aurora/水焦散 | ○ 4dccWj · Ms2SWW · **XlfGRj(Star Nest)** · XtGGRt(Auroras)· MdlXz8(Tileable Water Caustic) |
| 有序抖动 | Bayer 矩阵阈值;Obra Dinn 风 = Bayer+blue noise | ✓ wikipedia Ordered_dithering;○ https://danielilett.com/2020-02-26-tut3-9-obra-dithering/ |

## 4. 2D 游戏 fragment 特效(动作/反馈层)

| 效果 | 原理 | 链接(godotshaders 单页均含完整源码+live preview) |
|---|---|---|
| **溶解 dissolve** | noise<threshold 裁剪 + 阈值窄带发光边 | ✓ https://blog.febucci.com/2018/09/dissolve-shader/ ;✓ https://godotshaders.com/shader/2d-dissolve-with-burn-edge/ |
| 冲击波 shockwave/ripple | 到波心距离在扩张环带内径向偏移 UV | ○ godotshaders.com/shader/positioned-shockwave/ · /ripple-shader/ |
| 受击闪白 hit-flash | RGB mix 到白,保 alpha,快速开关 | ✓ https://godotshaders.com/shader/hit-flash-effect-shader/ |
| outline 三路 | alpha 外扩 / Sobel / SDF 加宽 | ○ godotshaders /2d-outline-inline/ · /sobel-outline-shader/ |
| squash & stretch | 顶点非均匀缩放,面积守恒 | ○ godotshaders /squash-and-stretch/ |
| CRT/扫描线 | 桶形弯曲+scanline+shadow mask | ○ **Lottes CRT** https://www.shadertoy.com/view/XsjSzR ;可读源码 ○ github libretro/glsl-shaders crt-lottes.glsl |
| glitch | 条带横移+RGB split+噪点 | ○ godotshaders /glitch-effect-shader/ |
| 色差 chromatic aberration | RGB 三通道异向偏移采样 | ○ godotshaders /radial-chromatic-aberration/ |
| pixelate/halftone/dither | UV 量化;亮度→点径;Bayer 阈值 | ○ godotshaders /simple-ordered-dithering-and-screen-pixelation/ · /halftone/ |
| grain/vignette/posterize/LUT | 帧噪声;中心距压暗;色阶量化;查表调色 | ○ godotshaders /film-grain-shader/ 等 |
| bloom / Dual Kawase | 阈值提亮→mip 链 down/up 廉价模糊→加回 | ○ https://blog.frost.kiwi/dual-kawase/ ;godotshaders /dual-kawase-blur-down/ |
| motion blur / 拖尾 | 沿速度多采样;或 feedback buffer 衰减叠加 | ○ godotshaders /2d-motion-blur/ |
| 2D 光照/法线贴图 | sprite normal map 逐像素 N·L | ○ gdquest.com lighting-with-normal-maps |
| 2D SDF 阴影/光照 | 屏幕 SDF raymarch 软影 | ○ godotshaders /dynamic-2d-lights-and-soft-shadows/ ;文章 ○ slembcke.github.io/SuperFastSoftShadows |
| 水/热浪 distortion | 滚动噪声 RG 作 UV 偏移 | ○ godotshaders /heat-haze-shader/ · /2d-water-distortion-effect-godot-4/ |
| 雨滴玻璃 | 网格 hash 水滴 + 偏移扭曲背景 + mip 模糊 | ○ **Heartfelt** https://www.shadertoy.com/view/ltffzl (BigWings,本类事实标准) |
| 雪/雾/焦散 | 多层视差 hash 粒子;fbm 遮罩;voronoi 亮边 | ○ shadertoy 4lfcz4;godotshaders /2d-fog-overlay/;○ MdKXDm(2Tweet Caustic) |
| 闪电 | 到扰动中线的距离 + k/d 发光 | ○ https://www.shadertoy.com/view/ldjXDD |
| 漩涡/portal | 极坐标旋转角随半径衰减 | ○ godotshaders /2d-swirling-vortex-portal/ |
| 旗帜/草摆动 | 顶点 sin(TIME+pos),UV.y 加权 | ○ godotshaders /2d-wind-sway/ |
| **屏幕转场全库** | `transition(uv,progress)` 统一范式:噪声 wipe/iris/百叶窗… **80+ 个 GLSL,在线可玩** | ✓ https://github.com/gl-transitions/gl-transitions ;画廊 https://gl-transitions.com/ (**必看**) |

## 5. UI 界面 shader 先例(同行怎么用)

| 先例 | 用在哪 | 链接 |
|---|---|---|
| Zed GPUI | 整个 UI = quad/shadow/sprite 原语 + SDF shader(和你同构) | ○ https://zed.dev/blog/videogame ;shader 源码 `crates/gpui/src/platform/mac/shaders.metal`(本机 zed 仓库可直接读) |
| Warp 终端 | <250 行 shader 画全部 UI(圆角/毛玻璃) | ○ https://www.warp.dev/blog/how-to-draw-styled-rectangles-using-the-gpu-and-metal |
| makepad Sdf2d | 样式即 shader,live edit;gloop 选区已被 plan32 借走 | ✓ docs.rs makepad-draw DrawQuad;本机仓库 `~/w/agentscode/makepad` |
| alacritty | glyph 双源混合子像素合成(文字 shader 教科书) | ✓ https://github.com/alacritty/alacritty/blob/master/alacritty/res/glsl3/text.f.glsl |
| Flutter | FragmentProgram 正门 + flutter_shaders + Wonderous 生产级用例 | ○ docs.flutter.dev fragment-shaders;github gskinnerTeam/flutter-wonderous-app |
| **paper-shaders** | 零依赖 npm shader 组件(mesh gradient/dithering/halftone/grain 做卡片背景) | ○ https://github.com/paper-design/shaders ;画廊 https://shaders.paper.design/ |
| Stripe mesh gradient | 品牌级 shader 背景(fbm 扭曲四色网格) | ○ 拆解 https://www.bram.us/2021/10/13/how-to-create-the-stripe-website-gradient-effect/ ;生成器 https://whatamesh.vercel.app/ |
| React Bits | Aurora/Silk/LiquidChrome 背景 + Shader Reveal 组件 | ○ https://reactbits.dev/backgrounds/aurora |
| CSS Houdini | paint worklet 案例库 | ○ https://houdini.how/ |
| Codrops hover/distortion 系列 | 图片/文字 hover 扭曲(displacement UV) | ○ tympanus.net 2018 WebGL Distortion Hover · 2020 Interactive Hover |
| shimmer/skeleton | 渐变随时间平移(你 plan28 已有 GPU 相位版) | ○ docs.flutter.dev cookbook shimmer-loading |
| Siri/Apple Intelligence 辉光 | 屏缘 SDF 描边 + 流动渐变 + bloom | ○ codepen.io/gxt/details/pvzMwLR ;shadertoy lcdGz7 |
| ChatGPT voice orb | 噪声位移球 + fresnel 发光(音频驱 uniform) | ○ https://github.com/aguscruiz/voiceorb ;demo voice-orb.netlify.app |
| Telegram 动态壁纸 | 4 控制点渐变流动(消息发送时推进) | ✓ 源码 DrKLO/Telegram MotionBackgroundDrawable.java;○ telegram.org/blog/animated-backgrounds |
| iOS 26 Liquid Glass 复刻 | 背景采样 + SDF 法线折射 + 色散 | ○ https://www.shadertoy.com/view/WftXD2 ;汇总 github carolhsiaoo/awesome-liquid-glass |
| Android AGSL | RuntimeShader 进 View/Compose | ○ developer.android.com AGSL 文档 |

## 6. 粒子 / 逐实例 / 文字级动效

| 能力 | 原理 | 链接 |
|---|---|---|
| 文字↔粒子聚散 | 采样文字像素为目标位,粒子插值/受力迁移 | ○ Codrops 2019 Interactive Particles;codepen XWNjBdb |
| **MSDF 文字溶解成尘/花瓣** | dissolve mask + 边缘发射粒子(2026 Codrops,和你技术栈最贴) | ○ tympanus.net 2026/01/28 webgpu-gommage-effect |
| per-glyph stagger/spring 入场 | 逐字符错峰弹簧(translateY+opacity+blur) | ○ Codrops 2020 stagger reveal;✓ splitting.js.org/guide.html |
| 弹簧物理 | mass-tension-friction 二阶积分;闭式 `e^{-dt}cos(wt)` 近似 | ✓ https://www.joshwcomeau.com/animation/a-friendly-introduction-to-spring-physics/ (**交互式,必读**) |
| GPGPU/FBO 粒子 | 位置/速度存纹理,ping-pong 更新(万级粒子唯一正确架构) | ✓ https://barradeau.com/blog/?p=621 (FBO particles 圣经) |
| curl noise 流场粒子 | 无散度场推粒子,"呼吸的雾" | ○ github spite/polygon-shredder;拆解 emildziewanowski.com/curl-noise |
| attractor 聚散 | 引力+切向扰动,轨道/逃逸节奏 | ○ threejs.org/examples/webgpu_tsl_compute_attractors_particles.html |
| 拖尾 trail(feedback) | 不清屏,上帧×衰减回写 | ○ danilw.github.io GPU particles 文;shadertoy tdK3zD |
| confetti | CPU 数百片重力+wobble | ✓ https://github.com/catdad/canvas-confetti |
| 烟花/火焰/烟雾 | hash 爆点公式化轨迹;noise 上升域;fbm+warp | ○ shadertoy lscGRl(BigWings Fireworks)· **MdX3zr(XT95 Flame,火焰第一经典)** |
| 2D 流体 | 半拉格朗日 NS 全 fragment 求解 | ✓ https://github.com/PavelDoGreat/WebGL-Fluid-Simulation (16k★,**指针交互生命感天花板**) |
| boids/swarm text | 三规则集群;文字作弱吸引 → "思考中"隐喻 | ○ threejs webgl_gpgpu_birds;shadertoy XsdyRj |
| thanos snap 退场 | 元素栅格化分层随机位移淡出 | ○ redstapler.co thanos-snap;github mttetc/thanos.snap |
| 按钮粒子/爆散 | 点击点发射高速粒子(integrate 反向 = 凝聚入场) | ○ tympanus.net 2018 particle-effects-for-buttons |

## 7. 值得整库收藏的四个源

1. **gl-transitions**(80+ 转场,统一 `transition(uv)` 契约,在线画廊)— https://gl-transitions.com/
2. **LYGIA**(多语言含 WGSL 的函数库,Prosperity/Patron 双许可,0025 已引)— https://lygia.xyz
3. **godotshaders.com**(每页 live preview + 完整源码 + CC0/MIT)— https://godotshaders.com/shader/
4. **The Book of Shaders**(ch07 shapes / 09 patterns / 10-13 noise 全家)— https://thebookofshaders.com/

## 8. 与本引擎现状对照(效果 plan 的选型底图)

**已有**(0018 panel/0025 anim/0026 模块化/0028 shaderbox/plan32 落的):圆角/描边/网格/AO/选中(panel 参数)、smin+gloop(rect 邻接融合)、glyph spawn 淡入+apply_curve 三曲线、shimmer 相位扫描、glow_orb、fold scale 折叠、morph move(CPU-mix)、raymarch 3D 图标(shaderbox)、msdf 文字。

**一步之遥**(现有图元加参数即得,零新管线;≈ 0018"效果=数据"的直接兑现):
- 距离带家族补全:**外发光 glow / 解析 drop shadow(Evan Wallace 闭式)/ 等距条纹 / cel-band** → panel/rect 加 2-3 个参数;
- **hash + value/gradient noise + fbm 入 WGSL 库**(sdf.wgsl 旁新 noise.wgsl,Dave Hoskins hash 保确定性 R8)——这是溶解/扰动/纹理化一切效果的**前置地基**;
- **溶解 dissolve**(noise 阈值 + 发光边):退场/删除动效,消费 noise 库 + 现有 alpha 通道;
- **hit-flash / 强调闪色**:DecorSlot 一个混色参数;
- **文字揭示升级**:per-glyph blur-in(近似:alpha+scale)+ stagger 已有节奏系统,补 spring 闭式近似(`e^{-kt}cos(wt)`,纯函数可重放)。

**中期**(需要一条新通道,但价值高):
- **feedback buffer 通道**(上帧纹理输入)→ 解锁拖尾/motion blur/流体/CRT 余晖一整族;与 R8 确定性要对齐(feedback 内容不进逻辑,只进呈现);
- **屏幕空间后处理 pass**(bloom Dual Kawase / vignette / grain / 色差):0003 的 overlay pass 位留过;
- **gl-transitions 式转场契约**:会话切换/主题切换的 `transition(uv,progress)`;
- 粒子层(逐实例已有,加 per-instance 速度/寿命 = confetti/火花/文字凝聚轻量版)。

**远期/看灵感**(TODO2-A 上限):GPGPU 万级粒子、2D 流体待机背景、liquid glass 折射卡片、swarm"思考中"、2D SDF 光照阴影全场景化。

**接入通道对照**:值层 → `noise.wgsl`/`sdf.wgsl`(0026 include 拼接);装饰参数 → 0018 panel/rect + 0037 DecorationOp;编排/时序 → 0025 slots + rhythm/motion token;身份 → 0020/0016(morph 已通)。**全部无需新架构。**

## 9. 必看清单(自看效果最短路径,20 条)

打开就能玩的:
1. gl-transitions 画廊(80 转场一页看完)https://gl-transitions.com/
2. paper-shaders 画廊(UI 背景效果族)https://shaders.paper.design/
3. WebGL Fluid(指针交互流体)https://paveldogreat.github.io/WebGL-Fluid-Simulation/
4. godotshaders dissolve(带 live preview + 源码)https://godotshaders.com/shader/2d-dissolve-with-burn-edge/

Shadertoy 经典(每个都是一类效果的代表作):
5. Heartfelt(雨滴玻璃)ltffzl · 6. Flame(火焰)MdX3zr · 7. Star Nest(星云)XlfGRj · 8. Auroras XtGGRt · 9. Tileable Water Caustic MdlXz8 · 10. Lottes CRT XsjSzR · 11. NEON LOVE(bezier 霓虹)WdK3Dz · 12. GLOW TUTORIAL 3s3GDn · 13. Shapes 2D 工具箱 4dfXDn · 14. Fireworks lscGRl · 15. Metaballs Quintic ld2GRz · 16. Apple Liquid Glass WftXD2(均为 shadertoy.com/view/ 后缀)

文章级(源码齐全):
17. Evan Wallace 解析圆角阴影 https://madebyevan.com/shaders/fast-rounded-rectangle-shadows/
18. IQ smin https://iquilezles.org/articles/smin/ + functions https://iquilezles.org/articles/functions/
19. Josh Comeau 弹簧物理(交互式)https://www.joshwcomeau.com/animation/a-friendly-introduction-to-spring-physics/
20. Febucci dissolve https://blog.febucci.com/2018/09/dissolve-shader/

---

> 与既有文档的关系:[sdf-animation-system](./sdf-animation-system.md) 讲"动画系统怎么组织"(控制层),本篇讲"能画出什么"(值层全目录);[animation-system-survey](./animation-system-survey.md) 的分期建议仍有效。下一步:从 §8"一步之遥"里挑 5-8 个效果 + noise.wgsl 地基,可裂出「效果库 plan」(TODO2-A 的第一期)。
