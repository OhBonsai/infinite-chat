// 九幕场景表(导演表 v0.2 可执行版)+ mountFilm 入口。
// v1 骨架:在「一份 film 文档」上,用 reveal 节奏 + 相机 + teaser 编排九幕(plan17)。
// 内容相关的精确取景(定位到某张表/代码块)与长河历史 = Phase ②(gen-film.mjs)。
import { FilmDirector, type Scene, type EngineApi, type FilmCtx } from "./director";
import { mountPlayer } from "./player";
import { createTeaser } from "./teaser";

const sleep = (ms: number) => new Promise<void>((r) => setTimeout(r, ms));

// 对称 zoom 脉冲:放大/缩小到 F 再原样还原(同屏点 → 不漂移)。F<1 = 拉远。
async function zoomPulse(ctx: FilmCtx, F: number, hold = 900) {
  const dpr = window.devicePixelRatio || 1;
  const cx = (window.innerWidth * dpr) / 2;
  const cy = window.innerHeight * dpr * 0.42;
  const steps = 26;
  const f = Math.pow(F, 1 / steps);
  for (let i = 0; i < steps; i++) { ctx.call("zoom_at", f, cx, cy); await sleep(20); }
  await sleep(hold);
  for (let i = 0; i < steps; i++) { ctx.call("zoom_at", 1 / f, cx, cy); await sleep(20); }
}

const reveal = (ctx: FilmCtx, cps: number, slow: number) => {
  ctx.call("set_reveal_cps", cps);
  ctx.call("set_reveal_slow", slow);
  ctx.call("restart_reveal");
};

// 九幕(~40s,节奏优先)。at = 幕内 ms。
export const SCENES: Scene[] = [
  {
    id: "open", title: "开场", durationMs: 4000,
    enter: (c) => reveal(c, 12, 1.0), // caret + 首条消息,慢起
  },
  {
    id: "scale-glimpse", title: "规模·惊鸿", durationMs: 4000,
    cues: [{ at: 400, fn: (c) => void zoomPulse(c, 0.5, 1400) }], // 拉远一瞥:这条对话很长
  },
  {
    id: "stream", title: "流式", durationMs: 5000,
    enter: (c) => reveal(c, 9, 0.5),                 // 慢打字
    cues: [{ at: 2600, fn: (c) => reveal(c, 46, 1.0) }], // 转快涌:同句不同节奏
  },
  {
    id: "sdf", title: "SDF·数学", durationMs: 5000,
    enter: (c) => reveal(c, 28, 1.0),
    cues: [{ at: 700, fn: (c) => void zoomPulse(c, 3.0, 1300) }], // 推近仍锐利(含公式)
  },
  {
    id: "rich", title: "富内容", durationMs: 5000,
    enter: (c) => reveal(c, 24, 1.0), // 文档含代码/表/数学/猫分割线
  },
  {
    id: "fx", title: "特效·路线图", durationMs: 5000,
    enter: (c) => c.call("set_shaderbox_gallery", true),
    cues: [
      { at: 1600, fn: (c) => { c.call("set_shaderbox_gallery", false); c.teaser.show("Agent 发光身份", "assistant 的 glow-orb 身份环 · 流式时加速脉冲(plan16 §2.6)"); } },
      { at: 3000, fn: (c) => c.teaser.show("3D 相机 · raymarch SDF", "光栅 SDF + raymarch 混合渲染,文字走进三维(决策 0024)") },
      { at: 4200, fn: (c) => c.teaser.show("多实例同步 · a11y 镜像", "多标签页状态同步(0008)· canvas 的无障碍 DOM 镜像") },
    ],
  },
  {
    id: "resilient", title: "容错", durationMs: 5000,
    // v1 占位:用 teaser 叙述"断网不慌 / 刷新不丢历史"(真实断网编排 = Phase ④ 接 0003)
    enter: (c) => c.teaser.show("断网不慌 · 刷新不丢历史", "弱网丢包:已上屏内容纹丝不动;EventSource 自动重连 + 快照 resync 对账(决策 0003)"),
  },
  {
    id: "scale-peak", title: "规模·全爆发", durationMs: 4000, // 高潮
    cues: [{ at: 300, fn: (c) => void zoomPulse(c, 0.32, 1800) }], // 拉到极远:100+ 轮长河,fps 稳
  },
  {
    id: "outro", title: "收尾 · Chat", durationMs: 3000,
    enter: (c) => { reveal(c, 30, 1.0); c.teaser.show("一条永不结束的对话", "infinite-chat · Rust · WebAssembly · WebGPU · React/Vue import"); },
  },
];

/// 挂载介绍片:teaser + director + 播放器 + rAF 主循环。返回销毁函数。
export function mountFilm(chat: unknown) {
  const teaser = createTeaser();
  const dir = new FilmDirector(SCENES, chat as EngineApi, teaser);
  mountPlayer(dir);
  let last = performance.now();
  let alive = true;
  const loop = (now: number) => {
    if (!alive) return;
    dir.tick(now - last);
    last = now;
    requestAnimationFrame(loop);
  };
  requestAnimationFrame((t) => { last = t; requestAnimationFrame(loop); });
  dir.play();
  return () => { alive = false; };
}
