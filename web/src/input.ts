// input(Plan 6)— 画布输入在 web 层统一处理,经 `ChatCanvas.pan_by/zoom_at` 调入 wasm。
//
// 之前 wheel 在 wasm 里(只有纵向滚 + ctrl 缩放,无横向、无拖拽)。挪到 TS:改手感不必重编 wasm。
// 覆盖:① 触摸板两指滚动(deltaX 横 + deltaY 纵)② 捏合 / ctrl+滚轮 缩放 ③ 鼠标/触摸板按住拖拽平移。
//
// 单位:相机吃**设备像素**(viewport = canvas.width = CSS×dpr);DOM 事件给 CSS px / 行设备无关 px,
// 故一律 ×dpr。方向:wheel 的 deltaY>0(下滚)= 看更新内容,直接喂正 dy;拖拽与内容同向 → 取负 movement。

import type { ChatCanvas } from "../pkg/infinite_chat_wasm.js";

/// 在 canvas 上挂滚轮 + 指针拖拽,平移/缩放经 chat 调入。返回卸载函数。
export function attachCanvasInput(canvas: HTMLCanvasElement, chat: ChatCanvas): () => void {
  const dpr = () => window.devicePixelRatio || 1;

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
      // 两指滚动:横纵都喂(横向滚宽表、纵向滚消息流)。
      chat.pan_by(e.deltaX * d, e.deltaY * d);
    }
  };
  canvas.addEventListener("wheel", onWheel, { passive: false });

  // 指针拖拽(鼠标左键 / 触摸板按住)→ 平移。setPointerCapture 保证拖出画布仍跟手。
  let dragging = false;
  const onDown = (e: PointerEvent) => {
    if (e.button !== 0) return; // 仅主键
    dragging = true;
    canvas.setPointerCapture(e.pointerId);
    canvas.style.cursor = "grabbing";
  };
  const onMove = (e: PointerEvent) => {
    if (!dragging) return;
    const d = dpr();
    // 拖拽与内容同向:向右拖 → 内容右移 → 相机左移(负 dx);纵向同理。
    chat.pan_by(-e.movementX * d, -e.movementY * d);
  };
  const onUp = (e: PointerEvent) => {
    if (!dragging) return;
    dragging = false;
    if (canvas.hasPointerCapture(e.pointerId)) canvas.releasePointerCapture(e.pointerId);
    canvas.style.cursor = "";
  };
  canvas.addEventListener("pointerdown", onDown);
  canvas.addEventListener("pointermove", onMove);
  canvas.addEventListener("pointerup", onUp);
  canvas.addEventListener("pointercancel", onUp);

  return () => {
    canvas.removeEventListener("wheel", onWheel);
    canvas.removeEventListener("pointerdown", onDown);
    canvas.removeEventListener("pointermove", onMove);
    canvas.removeEventListener("pointerup", onUp);
    canvas.removeEventListener("pointercancel", onUp);
  };
}
