// canvas-rect.ts — 画布矩形缓存:overlay 层(0022 家族)的坐标基准从「视口原点」升级为「画布矩形」。
//
// 动机(Plan 25 迭代):画布不再必然全屏(/chat 可设宽/鼠标 resize)。text-layer / copy-button /
// embed-overlay 的容器原是 `position:fixed;inset:0`,内部元素用画布设备像素坐标 ÷ DPR 定位——
// 隐含假设「画布左上角 = 视口 (0,0)」。本模块把容器对齐到画布真实矩形,假设消除;
// 主页画布仍全屏 → rect 恒等于视口,行为零变化。
//
// 性能:**不逐帧读 getBoundingClientRect**(强制 reflow)。rect 只在 ResizeObserver / window
// resize 时刷新并递增版本号;`syncLayerToCanvas` 逐帧调但仅版本变化时写样式(零成本快路径)。
//
// resize 桥:容器拖拽改画布尺寸不触发 window resize,而 wasm 的后备缓冲/surface/max_width 重配
// 挂在 window resize 上(lib.rs on_resize)→ RO 探测到画布尺寸真变时**代派发**一次 resize 事件。
// 派发后画布尺寸不再变 → RO 不再触发 → 无回环。

let rect = { left: 0, top: 0, width: 0, height: 0 };
let version = 0;
let watched: HTMLCanvasElement | null = null;

function refresh(): void {
  if (!watched) return;
  const r = watched.getBoundingClientRect();
  if (
    r.left !== rect.left ||
    r.top !== rect.top ||
    r.width !== rect.width ||
    r.height !== rect.height
  ) {
    rect = { left: r.left, top: r.top, width: r.width, height: r.height };
    version += 1;
  }
}

/** boot 时调一次:开始跟踪画布矩形 + 桥接容器级 resize 到 window resize(喂 wasm 重配路径)。
 * 直发不去抖:wasm on_resize 重配后**同步重画一帧**(lib.rs,合成前缓冲已满)→ 无空白帧;
 * RO 每帧至多触发一次(规范)→ 重排/重画天然 ≤ 帧率。拖动全程实时清晰重排。 */
export function watchCanvasRect(canvas: HTMLCanvasElement): void {
  watched = canvas;
  refresh();
  let lastW = canvas.clientWidth;
  let lastH = canvas.clientHeight;
  if (typeof ResizeObserver !== "undefined") {
    new ResizeObserver(() => {
      refresh(); // rect 缓存即时刷新 → overlay 逐帧跟随
      if (canvas.clientWidth !== lastW || canvas.clientHeight !== lastH) {
        lastW = canvas.clientWidth;
        lastH = canvas.clientHeight;
        window.dispatchEvent(new Event("resize"));
      }
    }).observe(canvas);
  }
  // window resize 本身也可能平移画布(居中布局)→ 刷新缓存(wasm 侧自有监听,互不依赖)。
  window.addEventListener("resize", refresh);
}

/** 把一个 `position:fixed` 的 overlay 容器对齐到画布矩形。逐帧调安全:版本未变即返回。 */
export function syncLayerToCanvas(el: HTMLElement): void {
  if (el.dataset.rectV === String(version)) return;
  el.dataset.rectV = String(version);
  el.style.left = `${rect.left}px`;
  el.style.top = `${rect.top}px`;
  el.style.width = `${rect.width}px`;
  el.style.height = `${rect.height}px`;
}
