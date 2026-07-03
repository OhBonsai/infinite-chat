# 0034:画布运行时 resize 的无闪烁方案 —— 同步重画 vs 去抖拉伸

- 日期:2026-07-02
- 状态:**已采纳(已实现)** —— 采纳方案 B(同步重画);方案 A(去抖+CSS 拉伸)作为记录在案的备选/降级;相机补偿路线**已否决**
- 前置:[0011](0011-gpu-text-as-sdf-primitive.md)(SDF 世界,排版按设备像素)· [0029](0029-session-virtualization-and-glyph-working-set.md)/[Plan 19](../plan/plan19-session-virtualization.md)(重排成本模型)· [0022](0022-dom-overlay-layer.md)(overlay 家族,坐标基准)· R8 确定性(dt 注入)
- 背景:/chat 页画布改为**可变宽列**(容器 div 定宽,滑块/`?w=` 实时调,Plan 25 迭代)。拖动调宽时画布内容闪烁。本 ADR 记录根因、两套候选方案与取舍,以及为什么"假 resize(相机补偿)"不可行。

---

## 1. 根因:`canvas.width=` 清空缓冲 + 重画晚一帧

HTML 规范:对 canvas 的 `width`/`height` 属性赋值**立即将绘制缓冲重置为透明空白**。原 resize 链:

```
容器宽变 → RO → 派发 window resize → wasm on_resize:
  canvas.set_width(w)   ← 缓冲瞬间清空
  surface 重配 + set_max_width(整屏重折行)
  (结束,无渲染)
→ 浏览器合成点:缓冲还是空的 → 合成一帧空白  ← 闪
→ 下个 rAF:engine.frame(dt) 才画回来
```

连续拖动 = 每帧一次「清空→空白帧→重画」循环 → 闪烁。`set_max_width` 全量重折行的开销可能再掉帧,把空白窗拉得更长。

## 2. 两套方案

### 方案 A:去抖 + CSS 拉伸(曾短暂上线,记录在案)

拖动期间**不碰后备缓冲**:容器/CSS 宽实时变,浏览器把旧位图拉伸填充(内容连续但单轴变形、微模糊);尺寸稳定 ~150ms 后才派发一次 resize,一次性清晰重配+重折行。

- 优点:实现最小(纯 TS 一个 setTimeout);重排只发生一次,任意内容规模都不掉帧。
- 缺点:拖动期间文字**横向变形**(非等比),观感"糊一下再跳清晰";overlay 坐标(engine 旧宽)与视觉(新宽)在拖动期间轻微脱钩。

### 方案 B:同步重画(采纳)✅

把「重配 + 重画」做成**同一任务内的原子操作**——`on_resize` 里重配后立刻 `engine.frame(0.0)`:

```
RO → 派发 resize → wasm on_resize:
  set_width(清空) → surface 重配 → set_max_width(重折行) → engine.frame(0)  ← 同步画满
→ 合成点:缓冲已是完整新画面 → 无空白帧,拖动全程实时、清晰、真重折行
```

- `frame(0.0)`:dt=0 不推进时间 → reveal/动画零前进,**R8 确定性无损**,纯"重排+重画"。
- 频率天然有界:RO 回调规范上每帧至多一次 → 重折行 ≤ 帧率。
- TS 侧(canvas-rect)恢复**直发**,无去抖。
- 缺点:拖动期间每帧全量重折行。对 /chat 级内容无感;**超长会话**(10k 行)下拖动可能掉帧(掉帧≠闪烁:帧完整,只是慢)。届时降级 = A/B 混合:拖动中降档(去抖/拉伸),停手精排——留作触发条件明确的后手,不预做。

### 已否决:假 resize(相机补偿)

「拖动期间不动缓冲,用引擎相机伪装宽度变化,停手再真重配」——**几何上不成立 + 语义上不值**:

1. 拖宽是**单轴**形变(横向 600→700,纵向不变),相机 `zoom` 是**等比**变换,等比无法补偿各向异性——补横毁纵。
2. 即便引入各向异性相机,得到的也只是"旧折行画面的缩放视图",不是**重折行**——与方案 A 中浏览器免费提供的 CSS 拉伸是同一视觉等价类,却要新造假相机态,且拖动期间 overlay/hit-test/选区的坐标基准全部随之造假(0022 家族逐帧对齐的是真相机)。高成本复刻已有效果,否。

## 3. 决定

**采纳 B(同步重画)**:`crates/wasm/lib.rs::on_resize` 末尾 `app.engine.frame(0.0)`;`web/src/canvas-rect.ts` RO 桥直发不去抖。A 记录为超长会话下的降级预案(触发条件:拖动重折行实测掉帧到肉眼可感)。

## 4. 不变量

- **无空白合成帧**:凡改 `canvas.width/height` 的路径,必须在同一任务内完成一次完整渲染(新增 resize 入口时遵守此约定)。
- **dt=0 重画无副作用**:`frame(0)` 不推进 reveal/动画/时钟(R8);同帧多次调用等幂。
- **宽度单源**:尺寸只写在外层容器 div 上;canvas/输入盒/overlay 全部跟随(见 canvas-rect,0022 延伸)。

## 5. 验证

- e2e `chat-route.spec`「画布列宽」:改 `#stage` 宽 → 断言后备缓冲重配(`canvas.width == clientWidth×dpr`)+ text-layer 对齐(<2px)。
- 人工(🟡):拖动滑块/手柄,全程无闪烁、无模糊、逐帧重折行跟手。
- 回归口:若未来 resize 又闪,先查是否有新路径改了 canvas 尺寸而没同步重画(§4 不变量 1)。

---

参考:HTML spec(canvas 尺寸赋值重置位图)· ResizeObserver 每帧至多一次回调 · Figma/Google Docs 等 canvas 应用的 resize-then-draw-in-same-task 惯例 · [Plan 19](../plan/plan19-session-virtualization.md)(重折行成本,降级预案的触发依据)。
