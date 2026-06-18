// transport.ts — 调试"视频播放器":回放当前消息的**揭示**时间轴(0019 渲染编排)。
//
// 为什么需要:正常 replay 一次性按墙钟跑完就冻住,reveal 编排(整表骨架 / 行框 / 逐字)一闪而过、
// 肉眼看不清。本控件把揭示当成一段视频:播放/暂停、调速、拖拽 scrubber 定位、单步——由 JS 掌钟,
// 驱动 wasm `tick(dt)`(增量推进)与 `seek_reveal(target)`(确定性跳转,向后也对)。
//
// 模型:内容已由 replay 加载(冻结块);揭示基速固定 `CPS`(让时间轴时长稳定),"速度"只改观看
// 快慢(dt×倍率),不改编排。时长 ≈ glyph 数 / CPS。换表格风格后用本控件播放即可看清差异。

const CPS = 60; // 固定揭示基速(glyph/s)→ 时间轴稳定
const SPEEDS = [0.25, 0.5, 1, 2, 4];

/// 仅声明本控件用到的 ChatCanvas 方法(避免耦合生成类型)。
interface TransportChat {
  set_paused(paused: boolean): void;
  set_reveal_cps(cps: number): void;
  set_reveal_slow(slow: number): void;
  tick(dt_ms: number): void;
  seek_reveal(target_ms: number): void;
  stats(): unknown;
}

export function mountTransport(chat: TransportChat, parent: HTMLElement): void {
  // 固定揭示基速;倍率默认 1。reveal_slow 由倍率经 tick 的 dt 实现,这里恒 1。
  chat.set_reveal_cps(CPS);
  chat.set_reveal_slow(1);

  let active = false; // 首次交互才接管时钟(此前让 replay 自然流式加载完内容)
  let playing = false;
  let speed = 1;
  let playhead = 0; // ms,时间轴位置
  let duration = 4000; // ms,按 glyph 数估算
  let lastWall = 0;
  let raf = 0;
  let seekPending = -1; // 待应用的拖拽目标(rAF 合并,避免每个 input 都重跑)

  // ── UI ──
  const wrap = document.createElement("div");
  wrap.style.cssText = "display:flex;flex-direction:column;gap:6px;margin-top:6px";
  const row = document.createElement("div");
  row.style.cssText = "display:flex;gap:6px;align-items:center";
  const mkBtn = (label: string) => {
    const b = document.createElement("button");
    b.textContent = label;
    b.style.cssText =
      "flex:0 0 auto;min-width:30px;font:12px ui-monospace,monospace;color:#cdd6f4;background:#313244;border:0;border-radius:4px;padding:3px 8px;cursor:pointer";
    return b;
  };
  const toStartBtn = mkBtn("⏮");
  const playBtn = mkBtn("▶");
  const stepBackBtn = mkBtn("⏪");
  const stepFwdBtn = mkBtn("⏩");
  const speedSel = document.createElement("select");
  speedSel.style.cssText =
    "flex:1;font:11px ui-monospace,monospace;color:#cdd6f4;background:#313244;border:0;border-radius:4px;padding:3px;cursor:pointer";
  for (const s of SPEEDS) {
    const o = document.createElement("option");
    o.value = String(s);
    o.textContent = `${s}×`;
    o.selected = s === 1;
    speedSel.appendChild(o);
  }
  row.append(toStartBtn, stepBackBtn, playBtn, stepFwdBtn, speedSel);

  const bar = document.createElement("input");
  bar.type = "range";
  bar.min = "0";
  bar.max = String(duration);
  bar.value = "0";
  bar.step = "1";
  bar.style.cssText = "width:100%;accent-color:#89b4fa;cursor:pointer";

  const label = document.createElement("div");
  label.style.cssText = "font:11px ui-monospace,monospace;color:#9399b2";

  wrap.append(row, bar, label);
  parent.appendChild(wrap);

  // ── 辅助 ──
  const STEP = 1000 / 30; // 单步 = 一帧(~33ms)
  const fmt = (ms: number) => `${(ms / 1000).toFixed(2)}s`;
  const updLabel = () => {
    label.textContent = `${fmt(playhead)} / ${fmt(duration)}  ${playing ? "▶" : "⏸"} ${speed}×`;
  };
  const estimateDuration = () => {
    const total = (chat.stats() as { glyphsTotal?: number })?.glyphsTotal ?? 0;
    duration = Math.max(800, Math.round(total > 0 ? (total / CPS) * 1000 + 800 : 4000));
    bar.max = String(duration);
  };
  const setHead = (t: number) => {
    playhead = Math.max(0, Math.min(duration, t));
    bar.value = String(Math.round(playhead));
    updLabel();
  };
  // 首次交互:接管墙钟(冻结 wasm 自走帧)+ 估时长。此前内容已由 replay 流式加载完。
  const ensureActive = () => {
    if (active) return;
    active = true;
    chat.set_paused(true);
    estimateDuration();
  };
  const applySeek = () => {
    if (seekPending < 0) return;
    const t = seekPending;
    seekPending = -1;
    chat.seek_reveal(t); // 确定性重跑揭示到 t(向后也对)
    setHead(t);
  };

  // ── 播放循环 ──
  const loop = (now: number) => {
    if (!playing) return;
    const dt = Math.min(64, now - lastWall) * speed; // 墙钟差 × 倍率,封顶防大跳
    lastWall = now;
    if (seekPending >= 0) applySeek();
    if (playhead >= duration) {
      pause();
      return;
    }
    chat.tick(dt); // 增量推进揭示(不重跑)
    setHead(playhead + dt);
    raf = requestAnimationFrame(loop);
  };
  const play = () => {
    ensureActive();
    if (playing) return;
    playing = true;
    playBtn.textContent = "⏸";
    if (playhead >= duration) {
      chat.seek_reveal(0);
      setHead(0);
    } // 到尾再播 → 回到开头
    lastWall = performance.now();
    raf = requestAnimationFrame(loop);
  };
  function pause() {
    playing = false;
    playBtn.textContent = "▶";
    cancelAnimationFrame(raf);
    updLabel();
  }

  // ── 事件 ──
  playBtn.onclick = () => (playing ? pause() : play());
  toStartBtn.onclick = () => {
    ensureActive();
    pause();
    chat.seek_reveal(0);
    setHead(0);
  };
  stepFwdBtn.onclick = () => {
    ensureActive();
    pause();
    chat.tick(STEP);
    setHead(playhead + STEP);
  };
  stepBackBtn.onclick = () => {
    ensureActive();
    pause();
    setHead(playhead - STEP);
    chat.seek_reveal(playhead); // 向后只能重跑
  };
  speedSel.onchange = () => {
    speed = Number(speedSel.value) || 1;
    updLabel();
  };
  bar.addEventListener("input", () => {
    ensureActive();
    pause();
    seekPending = Number(bar.value);
    requestAnimationFrame(applySeek); // 合并连续拖拽,一帧只跑一次 seek
  });

  // 初次:轮询估时长(内容流式期间 glyph 数在变,稳定后定),默认停在末尾(全显)。
  let polls = 0;
  const id = setInterval(() => {
    estimateDuration();
    if (++polls >= 6) {
      clearInterval(id);
      setHead(duration);
    }
  }, 400);
  updLabel();
}
