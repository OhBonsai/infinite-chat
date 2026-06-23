// FilmPlayer —— 底部视频播放器进度条(DOM chrome)。绑定 FilmDirector。
import type { FilmDirector } from "./director";

const fmt = (ms: number) => {
  const s = Math.floor(ms / 1000);
  return `${Math.floor(s / 60)}:${String(s % 60).padStart(2, "0")}`;
};

export function mountPlayer(dir: FilmDirector, parent: HTMLElement = document.body) {
  const bar = document.createElement("div");
  bar.style.cssText =
    "position:fixed;left:0;right:0;bottom:0;z-index:9997;display:flex;align-items:center;gap:14px;" +
    "padding:12px 18px;background:linear-gradient(#0d0f1700,#0d0f17ee);" +
    "font:12px/1 'JetBrains Mono',monospace;color:#cdd3e0;user-select:none;";

  const btn = document.createElement("button");
  btn.style.cssText =
    "all:unset;cursor:pointer;width:30px;height:30px;border-radius:50%;border:1px solid #3df5d066;" +
    "color:#3df5d0;text-align:center;line-height:30px;flex:0 0 auto;";
  btn.textContent = "▶";

  const track = document.createElement("div");
  track.style.cssText =
    "position:relative;flex:1;height:18px;cursor:pointer;display:flex;align-items:center;";
  const rail = document.createElement("div");
  rail.style.cssText = "position:absolute;left:0;right:0;height:3px;background:#222838;border-radius:2px;";
  const fill = document.createElement("div");
  fill.style.cssText = "position:absolute;left:0;height:3px;width:0;background:#3df5d0;border-radius:2px;";
  const knob = document.createElement("div");
  knob.style.cssText =
    "position:absolute;width:11px;height:11px;border-radius:50%;background:#3df5d0;left:0;transform:translateX(-50%);";
  track.append(rail, fill, knob);

  // 幕刻度
  for (const m of dir.marks) {
    const tick = document.createElement("div");
    const pct = (m.startMs / dir.totalMs) * 100;
    tick.title = m.title;
    tick.style.cssText =
      `position:absolute;left:${pct}%;width:2px;height:9px;background:#7c8499;opacity:.6;transform:translateX(-50%);`;
    track.appendChild(tick);
  }

  const time = document.createElement("span");
  time.style.cssText = "flex:0 0 auto;opacity:.7;min-width:74px;text-align:right;";

  const speeds = document.createElement("div");
  speeds.style.cssText = "display:flex;gap:6px;flex:0 0 auto;";
  const speedBtns: Record<number, HTMLButtonElement> = {};
  for (const r of [0.5, 1, 2]) {
    const sb = document.createElement("button");
    sb.style.cssText =
      "all:unset;cursor:pointer;padding:3px 7px;border-radius:5px;font-size:11px;color:#9aa3b5;border:1px solid #ffffff14;";
    sb.textContent = `${r}×`;
    sb.onclick = () => dir.setRate(r);
    speedBtns[r] = sb;
    speeds.appendChild(sb);
  }

  bar.append(btn, track, time, speeds);
  parent.appendChild(bar);

  btn.onclick = () => dir.toggle();
  const seekFromEvent = (e: MouseEvent) => {
    const rect = track.getBoundingClientRect();
    const pct = Math.max(0, Math.min(1, (e.clientX - rect.left) / rect.width));
    dir.seek(pct * dir.totalMs);
  };
  let dragging = false;
  track.addEventListener("mousedown", (e) => { dragging = true; seekFromEvent(e); });
  window.addEventListener("mousemove", (e) => { if (dragging) seekFromEvent(e); });
  window.addEventListener("mouseup", () => { dragging = false; });

  dir.onUpdate = (info) => {
    const pct = info.total ? info.clock / info.total : 0;
    fill.style.width = `${pct * 100}%`;
    knob.style.left = `${pct * 100}%`;
    btn.textContent = info.playing ? "⏸" : "▶";
    time.textContent = `${fmt(info.clock)} / ${fmt(info.total)}`;
    for (const r of [0.5, 1, 2]) {
      const on = info.rate === r;
      speedBtns[r].style.color = on ? "#3df5d0" : "#9aa3b5";
      speedBtns[r].style.borderColor = on ? "#3df5d066" : "#ffffff14";
    }
  };
}
