// input(Plan 6)— 画布输入在 web 层统一处理,经 `ChatCanvas.pan_by/zoom_at` 调入 wasm。
//
// 之前 wheel 在 wasm 里(只有纵向滚 + ctrl 缩放,无横向、无拖拽)。挪到 TS:改手感不必重编 wasm。
// 覆盖:① 触摸板两指滚动(deltaX 横 + deltaY 纵)② 捏合 / ctrl+滚轮 缩放 ③ 鼠标/触摸板按住拖拽平移。
//
// 单位:相机吃**设备像素**(viewport = canvas.width = CSS×dpr);DOM 事件给 CSS px / 行设备无关 px,
// 故一律 ×dpr。方向:wheel 的 deltaY>0(下滚)= 看更新内容,直接喂正 dy;拖拽与内容同向 → 取负 movement。

import type { ChatCanvas } from "../pkg/infinite_chat_wasm.js";
import { LINE_HEIGHT } from "./layout-bridge";

/// 在 canvas 上挂滚轮 + 指针拖拽,平移/缩放经 chat 调入。返回卸载函数。
export function attachCanvasInput(canvas: HTMLCanvasElement, chat: ChatCanvas): () => void {
  const dpr = () => window.devicePixelRatio || 1;
  // 代码块行窗纵滚累加器(Plan 15 ④):触摸板小增量累成整行,不丢小滚。
  let codeScrollAccum = 0;

  // 滚轮 / 触摸板两指滚动 + 捏合缩放(捏合在浏览器里 = ctrlKey + wheel)。
  const onWheel = (e: WheelEvent) => {
    e.preventDefault(); // 阻止页面滚动 / 浏览器缩放
    const d = dpr();
    if (e.ctrlKey) {
      // 捏合 / ctrl+滚轮 → 围绕指针缩放。exp 让缩放平滑且对称(放大/缩小步长一致)。
      const factor = Math.exp(-e.deltaY * 0.01);
      const r = canvas.getBoundingClientRect();
      chat.zoom_at(factor, (e.clientX - r.left) * d, (e.clientY - r.top) * d);
    } else {
      // 两指滚动:先看指针是否在某代码块行窗内 → 滚块(纵=行、横=px),不滚画布(Plan 15 ④)。
      const r = canvas.getBoundingClientRect();
      const sx = (e.clientX - r.left) * d;
      const sy = (e.clientY - r.top) * d;
      const key = chat.code_block_at_screen(sx, sy);
      if (key) {
        // CSS px deltaY → 行(累加小增量,触摸板平滑);横向直接喂 px。
        codeScrollAccum += e.deltaY / LINE_HEIGHT;
        const lines = Math.trunc(codeScrollAccum);
        codeScrollAccum -= lines;
        if (lines !== 0 || e.deltaX !== 0) chat.scroll_code_block(key, e.deltaX * d, lines);
      } else {
        // 横纵都喂画布(横向滚宽表、纵向滚消息流)。
        chat.pan_by(e.deltaX * d, e.deltaY * d);
      }
    }
  };
  canvas.addEventListener("wheel", onWheel, { passive: false });

  // 画布坐标(设备像素;ask tap/hover 用)。
  const canvasXY = (e: PointerEvent): [number, number] => {
    const r = canvas.getBoundingClientRect();
    const d = dpr();
    return [(e.clientX - r.left) * d, (e.clientY - r.top) * d];
  };

  // 指针拖拽(鼠标左键 / 触摸板按住)→ 平移。setPointerCapture 保证拖出画布仍跟手。
  // Plan 27:down 处记起点,up 时**位移 ≤4px(CSS)判 tap** → `chat.tap` 路由(ask 按钮应答);
  // 超阈值 = 拖拽,绝不误触按钮(§8 风险)。down 命中按钮先给按压视觉态(SDF 变深)。
  let dragging = false;
  let downAt: [number, number] | null = null; // CSS px(tap 阈值判定)
  const onDown = (e: PointerEvent) => {
    if (e.button !== 0) return; // 仅主键
    dragging = true;
    downAt = [e.clientX, e.clientY];
    const [sx, sy] = canvasXY(e);
    chat.ask_press(sx, sy); // 命中 ask 按钮 → 按压态(未命中内部 noop)
    chat.set_pointer_down(true); // M2f press 视觉(卡片微缩)
    canvas.setPointerCapture(e.pointerId);
    canvas.style.cursor = "grabbing";
  };
  const onMove = (e: PointerEvent) => {
    const [sx, sy] = canvasXY(e);
    chat.set_pointer(sx, sy); // M2f hover 视觉(卡片提亮;引擎侧命中,SDF 参数反馈)
    if (!dragging) {
      // hover:命中 ask 按钮 → cursor:pointer(每次 move 一次 hit 查询,纯几何,便宜)。
      canvas.style.cursor = chat.ask_hit_at(sx, sy) ? "pointer" : "";
      return;
    }
    const d = dpr();
    // 拖拽与内容同向:向右拖 → 内容右移 → 相机左移(负 dx);纵向同理。
    chat.pan_by(-e.movementX * d, -e.movementY * d);
  };
  const onLeave = () => chat.set_pointer(-1, -1); // 离开画布 → 清 hover
  const onUp = (e: PointerEvent) => {
    chat.set_pointer_down(false); // M2f 清 press
    if (!dragging) return;
    dragging = false;
    if (canvas.hasPointerCapture(e.pointerId)) canvas.releasePointerCapture(e.pointerId);
    canvas.style.cursor = "";
    chat.ask_release(); // 清按压态
    // tap 判定:总位移 ≤4 CSS px → 路由给画布 hit-test 层(0032 第一个口)。
    if (downAt) {
      const dist = Math.hypot(e.clientX - downAt[0], e.clientY - downAt[1]);
      downAt = null;
      if (dist <= 4) {
        const [sx, sy] = canvasXY(e);
        chat.tap(sx, sy);
      }
    }
  };
  canvas.addEventListener("pointerdown", onDown);
  canvas.addEventListener("pointermove", onMove);
  canvas.addEventListener("pointerup", onUp);
  canvas.addEventListener("pointercancel", onUp);
  canvas.addEventListener("pointerleave", onLeave);

  return () => {
    canvas.removeEventListener("wheel", onWheel);
    canvas.removeEventListener("pointerdown", onDown);
    canvas.removeEventListener("pointermove", onMove);
    canvas.removeEventListener("pointerup", onUp);
    canvas.removeEventListener("pointercancel", onUp);
    canvas.removeEventListener("pointerleave", onLeave);
  };
}
