# 决策记录 0015:字形源解析与回退链 —— Bitmap / TinySDF / MSDF(可调试切换)

- 日期:2026-06-14
- 状态:**已采纳 + 已落地**(2026-06-15;两轴模型 / 源解析器 / 回退链 / baked MSDF / 调试器切换与逐源统计均就位。唯 §2.5 的「回退 TinySDF 用 LXGW @font-face 子集」走「退而求其次」留尾,见 §4)
- 前置:0009(字体)、0011 §3.3/§3.5(quad/kind/三源)、0012 §4(调试器)、0013(MSDF)、0014
- 来源:需求——两套渲染方案共存可调试切换;MSDF 编译字体(LXGW 常用字,体积可控);未覆盖字 fallback 到 TinySDF/Canvas

## 1. 背景与目标

已有:**Plan 2 Canvas 位图** + **Plan 3 TinySDF**(运行时单通道,从位图)。新需求:

1. **两套方案共存,调试器切换**:位图 vs SDF,A/B 对比。
2. **MSDF 编译字体**:离线把 **LXGW(`lxgw-wenkai-v1.522/LXGWWenKaiMono-Light.ttf`)的常用字集**烘成 MSDF(大字/拐角锐),**整体体积可控**。
3. **回退链**:某字不在 MSDF 烘集里 → fallback 到 **TinySDF**(运行时)→ 再到 **Canvas 位图**。

各源能力(回顾):位图(1× 锐、不可缩放/特效)、TinySDF(可缩放但大字软、任意字、运行时)、MSDF(任意尺度锐含拐角、需轮廓、固定字集)。

## 2. 决策:两轴模型 + 源解析器 + 回退链

### 2.1 两根轴

- **轴 A — 渲染方案(调试器全局切)**:`Bitmap` ↔ `SDF`。这就是"两套共存可切换"。
- **轴 B — SDF 内的源(逐字,带回退)**:`MSDF(LXGW 烘集命中)` → `TinySDF(运行时回退)`。
- emoji/彩字:始终 `RGBA`(与轴正交)。

### 2.2 源解析器(每字一次,在 atlas alloc 处)

```
resolve(glyph, mode):
  Bitmap 模式            → 位图(Canvas2D 覆盖率, kind=0)
  SDF 模式(默认):
     glyph ∈ MSDF 烘集   → MSDF(取 baked atlas, kind=2)
     否则                 → TinySDF(运行时 Canvas2D→EDT, kind=1)   # 回退
  emoji/彩字             → RGBA(kind=3)
```

回退落地:**MSDF 命中走静态 baked 图集;未命中即生成 TinySDF**(和现有 cache-miss 路径一致)。"Canvas 位图"是另一套方案(轴 A),也是最终兜底——`Bitmap` 模式下全部走它。

### 2.3 资源:离线 MSDF(LXGW 常用字)

- 离线用 **`msdf-bmfont-xml`**(npm,**预打包各平台 msdfgen,免编译**;`scripts/bake-msdf.mjs`)对 **LXGW** 烘 **常用字集**(ASCII + GB2312 一级 ~3760 常用汉字 + 常用标点;`TextDecoder('gbk')` 生成)→ `lxgw-msdf.png(RGB,可能多页)` + `lxgw-msdf.json`(**BMFont 格式**:`chars[]`={id, x/y/w/h, xoffset/yoffset, xadvance, page}、`common.lineHeight/base`、`distanceField.fieldType/distanceRange`、`kernings[]`)。`type=msdf`(三通道;msdf-bmfont-xml 仅 msdf/sdf/psdf,无 mtsdf)。
- **coverage = `chars[].id` 集合**;metrics(advance/bbox)来自 BMFont chars。(canonical 的 msdf-atlas-gen 需源码编译,brew 无 formula,故选 npm 版。)
- 体积:~3500 字 × ~40px MSDF cell(RGB)≈ 数 MB,**bounded**;生僻字**零资产**(运行时 TinySDF)。
- ship `web/public/`,**懒加载**(SDF 模式首次需要时拉)。
- 加载时把 coverage(`chars[].id`)建成 Set/bitset → 解析器 O(1) 判命中。

### 2.4 渲染(复用 0011,不新增管线)

- tile `kind` ∈ {0 位图覆盖率, 1 TinySDF, 2 MSDF, 3 RGBA}(0011 §3.5),片元按 kind 分支:位图 `cov=r`;TinySDF `smoothstep`;MSDF `median(r,g,b)` 再 smoothstep;RGBA 直采。
- atlas:**MSDF baked = 静态页**(启动/懒加载灌入)+ **R8 动态页**(位图/TinySDF 运行时,LRU)+ RGBA 页;实例 `layer` 选页、`kind` 选采样。

### 2.5 metrics 一致性(关键坑)

MSDF 用 LXGW 的 advance;TinySDF/位图用 Canvas2D 测。**同一行混排时 advance 要一致**,否则回退字错位。

- **推荐**:**回退 TinySDF 也用 LXGW 光栅**(`@font-face` 一份 LXGW **子集 woff2**)→ 与 MSDF 同源,advance/字形一致。
- **退而求其次**:生僻字(回退集)用系统字体,接受轻微不一致(因为是稀有字)。
- 决定:**正文字体统一指向 LXGW**(MSDF 烘 LXGW、回退 TinySDF 也 LXGW),正文不再用系统字体栈——把"字形一致"摆在"白嫖系统字体"前(本条**收窄 0009/0011 的'正文系统字体'**:有了自带 LXGW 后,正文统一 LXGW 更一致;系统字体仅作 LXGW 未加载时的兜底 fallback 链尾)。

### 2.6 调试器(0012 / plan4 4C)

- setter `set_glyph_mode(mode)`:`Auto`(MSDF→TinySDF 回退)/ `Bitmap` / `ForceTinySDF`(禁 MSDF,验回退)/ `ForceMSDF`(看覆盖空洞)。
- `FrameStats` 加**逐源计数** `{msdf, tinysdf, bitmap, rgba}` → DOM 面板显示 **MSDF 命中率**,用来调烘集。

## 3. 不变量与影响

- **契约/kind/atlas/片元不变**(0011);本 ADR 只新增 **① 源解析策略 ② 全局 mode(调试器)③ LXGW MSDF baked 资源 + coverage 判定 ④ 逐源统计**。
- **体积可控**:MSDF 仅常用字集(数 MB,懒加载);生僻字零资产。
- **正文字体统一 LXGW**(2.5):比"系统字体栈"一致,但要打包 LXGW(MSDF atlas + 回退用子集 woff2);权衡见 2.5。

## 4. 落地拆解(实现状态 2026-06-15)

1. **离线 baking 工具** ✅:`scripts/bake-msdf.mjs`(`msdf-bmfont-xml`)→ LXGW 常用字集 → BMFont png+json,可重跑。烘焙产物默认 gitignore(~11MB,可复现),本地 dev 直接服务;要 out-of-box 翻 `.gitignore` 几行。
2. **资源加载** ✅:`web/src/msdf.ts` fetch BMFont json → 紧凑 typed array(ids/cells)+ 解码 PNG 页 → `ChatCanvas.load_msdf`;render `MsdfAtlas`(RGBA D2Array 静态页)灌入 + 重建 bind group。懒加载(切到 MSDF 模式或 `?msdf` 时拉)。
3. **源解析器 + 回退** ✅:`GpuSink::resolve` 按 mode/coverage 选源——MSDF 命中(单码点 ∈ 烘集)→ baked;否则 Auto 回退 TinySDF / ForceMSDF 留空洞;Bitmap 全位图。MSDF quad 几何由方格 + BMFont metrics 反推(`msdf_instance`)。
4. **回退 TinySDF 用 LXGW @font-face**(子集 woff2)❌ **留尾**:当前回退 TinySDF/位图仍用系统字体栈,advance 由 system measureText 给 → MSDF(LXGW)字形落在系统度量槽位(§2.5「退而求其次」;LXGW 等宽集影响小)。要精确一致需烘一份 LXGW 子集 woff2 + 正文统一指向它(fonttools subset,另起)。
5. **调试器** ✅:`set_glyph_mode`(auto/bitmap/tinysdf/msdf 循环)+ `stats()` 逐源计数 `{bitmap,tinysdf,msdf,rgba}` → 面板 `src B/T/M` 行(MSDF 命中率)。
6. **渲染管线** ✅:`GpuInstance.kind`(0 位图/1 TinySDF/2 MSDF/3 RGBA),glyph.wgsl 统一控制流采两源后按 kind 选(median(rgb) for MSDF)。`TILE_PX=128` 对 TinySDF 仍有效;MSDF 走 baked 不受此限。

## 5. 重新评估触发

- 烘集命中率太低(大量回退到软 TinySDF)→ 扩充烘集 / 改运行时 MSDF(fdsm + LXGW .ttf,0013)。
- 不想打包 LXGW(回到小包体优先)→ 退回"系统字体 + TinySDF",放弃 MSDF 锐度(0009/0011 原路)。

## 6. 来源 / 链接

- 字体:`lxgw-wenkai-v1.522/LXGWWenKaiMono-Light.ttf`(OFL)
- 相关:0009(字体)、0011 §3.3/§3.5(kind/三源/atlas)、0012 §4(调试器)、0013(MSDF 离线/实时)、[TODO K′](../../TODO.md)、[design/thinking](../design/thinking.md)
- 工具:**`msdf-bmfont-xml`(npm,已用,`scripts/bake-msdf.mjs`)** → BMFont 输出;备选 msdf-atlas-gen(需编译)/ fdsm(运行时 MSDF,0013)
