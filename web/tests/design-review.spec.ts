// design-review.spec.ts — Plan 25 视觉自评截图 harness(execution-only,**不断言**)。
//
// 不进默认门:playwright.config.ts 在无 DESIGN_REVIEW 环境变量时 testIgnore 本文件。
// 跑法:DESIGN_REVIEW=1 npm --prefix web run e2e -- design-review.spec.ts
// 产物:test/results/design-review/<scene>/<name>.png(gitignored),文件名含场景 + seek 毫秒。
//
// 确定性(GOAL §2):内容用 push_event 合成(stream_rate=1e9 即时到达);定帧一律
// set_paused(true) 后 seek_reveal(ms)(core 侧 16ms 微步重跑,R8 确定)——不用 waitForTimeout 定帧。
// 场景矩阵是终态(GOAL §3),本文件随实现长大:M0 先建 S1/S2/S9。
import { expect, test, type Page } from "@playwright/test";
import fs from "node:fs";
import path from "node:path";
import { assertWebGpu, waitStable } from "./helpers";

// run.mjs / npm run e2e 的 cwd 都是 web/ → 产物落仓库根 test/results/design-review/。
const OUT_ROOT = path.resolve(process.cwd(), "..", "test", "results", "design-review");

function shot(scene: string, name: string): string {
  const dir = path.join(OUT_ROOT, scene);
  fs.mkdirSync(dir, { recursive: true });
  return path.join(dir, `${name}.png`);
}

// ---- 事件合成(shape 见 spec/knowledge/opencode.md §3.1 + protocol.rs 单测)----

const msgUpdated = (id: string, role: string) =>
  JSON.stringify({
    type: "message.updated",
    properties: { info: { id, sessionID: "s-dr", role, time: { created: 1 } } },
  });
const partUpdated = (p: Record<string, unknown>) =>
  JSON.stringify({ type: "message.part.updated", properties: { part: p, time: 1 } });
const textPart = (id: string, messageID: string, text: string) =>
  partUpdated({ type: "text", id, messageID, text });

/** 空画布启动(`?empty` = noop Player,无 synthetic demo、无残余 delta)+ 即时到达。 */
async function bootEmpty(page: Page): Promise<void> {
  await page.goto("/?empty&noinput", { waitUntil: "domcontentloaded" });
  await assertWebGpu(page);
  await page.waitForFunction(() => !!(window as unknown as { __chat?: unknown }).__chat, null, {
    timeout: 60_000,
  });
  await page.evaluate(() => {
    window.__chat.set_stream_rate(1e9);
    window.__chat.set_reveal_cps(1e9);
  });
}

async function pushAll(page: Page, events: string[]): Promise<void> {
  // `window.__chat` 挂上时引擎 state 可能尚未就绪(异步 start),push_event 会被静默丢弃
  // (冷启首个测试易踩)。事件全是 message.updated/part.updated 全量对账语义(AR4,幂等)
  // → 推送后短轮询不非空就整包重推,最多 5 轮。
  for (let attempt = 0; attempt < 5; attempt++) {
    await page.evaluate((evs) => {
      for (const e of evs) window.__chat.push_event(e);
    }, events);
    try {
      await expect
        .poll(() => page.evaluate(() => window.__chat.visible_turns() !== "[]"), {
          timeout: 5_000,
          intervals: [200],
        })
        .toBe(true);
      break;
    } catch {
      // 引擎还没就绪 → 重推
    }
  }
  await waitStable(page); // 内容全落地 + 全揭示(1e9)+ 相机停
}

/** 定帧:暂停 → seek 到 ms → core 已 render_now,直接截图。节奏参数由调用方先设
 * (set_reveal_preset / set_rhythm),本函数不动 tempo —— M1 后 cps 只是 tempo 的别名,
 * 在这里覆盖会抹平预设差异。 */
async function seekShot(page: Page, ms: number, file: string): Promise<void> {
  await page.evaluate(
    (ms) => {
      window.__chat.set_paused(true);
      window.__chat.seek_reveal(ms);
    },
    ms,
  );
  await page.screenshot({ path: file });
}

/** 设预设 + 统一 tempo(对比的是节奏层/groove,不是速度)。 */
async function setPreset(page: Page, preset: string, tempoMs: number): Promise<void> {
  await page.evaluate(
    ({ preset, tempoMs }) => {
      const c = window.__chat as unknown as {
        set_reveal_preset(n: string): void;
        set_rhythm(j: string): void;
      };
      c.set_reveal_preset(preset);
      c.set_rhythm(JSON.stringify({ tempo_ms: tempoMs }));
    },
    { preset, tempoMs },
  );
}

// ---- 素材 ----

// S1/S2 共用长 markdown(中英混排、句读/段落/列表/行内结构齐全 → 节奏边界可感知)。
const LONG_MD = [
  "## 流式渲染的节奏设计",
  "",
  "好的节奏不是匀速打字机,而是像朗读一样呼吸:句中稳定推进,标点处稍作停顿,段落间留出明显的空白拍。",
  "读者的眼睛跟随揭示前沿移动,**结构性停顿**帮助他们分句、分段地消化内容,而不是被均匀的字符流推着走。",
  "",
  "实现上分三层:",
  "- 基础速度(tempo):每个词或字的基准间隔",
  "- 边界停顿(rest):逗号、句号、段落各有层级",
  "- 微定时(microtiming):seeded 1/f 噪声,幅度极小",
  "",
  "最后,`typewriter` 预设保持旧的匀速行为,作为回归对拍的基线;`reader` 预设是默认;`flow` 更快但保留 groove。",
  "The quick brown fox jumps over the lazy dog, and pauses — briefly — at every boundary.",
].join("\n");

const S1_EVENTS = [
  msgUpdated("m-u1", "user"),
  textPart("p-u1", "m-u1", "讲讲流式渲染的节奏设计?"),
  msgUpdated("m-a1", "assistant"),
  textPart("p-a1", "m-a1", LONG_MD),
];

// S9 三轮完整对话(user + assistant 多 part:text/reasoning/tool/代码/表格)。
const S9_EVENTS = [
  msgUpdated("m-u1", "user"),
  textPart("p-u1", "m-u1", "帮我看看这个仓库的测试怎么跑?"),
  msgUpdated("m-a1", "assistant"),
  partUpdated({ type: "reasoning", id: "p-r1", messageID: "m-a1", text: "先找测试入口,再看运行器。" }),
  partUpdated({
    type: "tool",
    id: "p-t1",
    messageID: "m-a1",
    tool: "bash",
    state: { status: "completed", input: { cmd: "ls test/" }, output: "AGENTS.md\nrun.mjs\nresults" },
  }),
  textPart("p-a1", "m-a1", "测试入口是 `test/run.mjs`,四层门:gates、native、unit、e2e,全绿才过。"),
  msgUpdated("m-u2", "user"),
  textPart("p-u2", "m-u2", "单跑 e2e 呢?给个例子。"),
  msgUpdated("m-a2", "assistant"),
  textPart(
    "p-a2",
    "m-a2",
    ["单跑某层加 `--suite`:", "", "```bash", "node test/run.mjs --suite e2e --filter visual", "```", "", "常用组合:", "", "| 改动 | 至少跑 |", "|---|---|", "| Rust | gates + native |", "| web TS | gates + e2e |"].join("\n"),
  ),
  msgUpdated("m-u3", "user"),
  textPart("p-u3", "m-u3", "明白了,谢谢!"),
  msgUpdated("m-a3", "assistant"),
  textPart("p-a3", "m-a3", "不客气。改完记得提交前跑一次完整 `node test/run.mjs`。"),
];

test.use({ viewport: { width: 1000, height: 900 } });

// 对比用统一 tempo(ms/拍):压快 3 倍便于截帧,节奏层(rest/accent/ε)随 tempo 等比缩放,
// groove 差异保持。总时长估算:单元数(CJK 字 + 拉丁词)× tempo × ~1.4(停顿摊销)。
const TEMPO = 60;
const UNITS = LONG_MD.replace(/[a-zA-Z0-9']+/g, "w").replace(/\s/g, "").length;
const TOTAL = Math.round(UNITS * TEMPO * 1.4);

// ---- S1 正文节奏(reader 预设,产品默认档)----
test("S1 正文节奏:20/50/80/settled 四帧 + 400ms 连帧", async ({ page }) => {
  await bootEmpty(page);
  await pushAll(page, S1_EVENTS);
  await setPreset(page, "reader", TEMPO);
  for (const f of [0.2, 0.5, 0.8]) {
    const ms = Math.round(TOTAL * f);
    await seekShot(page, ms, shot("s1-cadence", `seek-${ms}ms-${Math.round(f * 100)}pct`));
  }
  await seekShot(page, TOTAL * 3, shot("s1-cadence", "settled"));
  // 连续 8 帧(400ms 间隔 ≈ 6-7 拍/帧)从 40% 处起:看帧间揭示量的分布(句读呼吸)。
  const t0 = Math.round(TOTAL * 0.4);
  for (let k = 0; k < 8; k++) {
    const ms = t0 + k * 400;
    await seekShot(page, ms, shot("s1-cadence", `seq-${String(k).padStart(2, "0")}-${ms}ms`));
  }
});

// ---- S2 预设对比:同文本 × 同 tempo × 三预设(差异 = 节奏层)----
test("S2 预设对比:同文本 × 各预设 50% 一帧 + 8 连帧", async ({ page }) => {
  await bootEmpty(page);
  await pushAll(page, S1_EVENTS);
  for (const preset of ["typewriter", "reader", "flow"]) {
    await setPreset(page, preset, TEMPO);
    const mid = Math.round(TOTAL * 0.5);
    await seekShot(page, mid, shot("s2-presets", `${preset}-seek-${mid}ms`));
    for (let k = 0; k < 8; k++) {
      const ms = Math.round(TOTAL * 0.4) + k * 400;
      await seekShot(page, ms, shot("s2-presets", `${preset}-seq-${String(k).padStart(2, "0")}-${ms}ms`));
    }
  }
});

// ---- S3 tool 卡(M2b:四态徽章 + running 流光相位两帧 + wipe 入场中点)----
const toolPart = (id: string, mid: string, status: string, extra = "") =>
  partUpdated({
    type: "tool",
    id,
    messageID: mid,
    tool: "bash",
    state: {
      status,
      input: { cmd: `echo ${status}` },
      ...(status === "completed" ? { output: "ok" } : {}),
      ...(status === "error" ? { error: "exit 1" } : {}),
      ...JSON.parse(extra || "{}"),
    },
  });

test("S3 tool 卡:四态 + 流光相位 + wipe 中点", async ({ page }) => {
  await bootEmpty(page);
  const events = [
    msgUpdated("m-a1", "assistant"),
    toolPart("t-p", "m-a1", "pending"),
    toolPart("t-r", "m-a1", "running"),
    toolPart("t-d", "m-a1", "completed"),
    toolPart("t-e", "m-a1", "error"),
  ];
  await pushAll(page, events);
  await setPreset(page, "reader", TEMPO);
  // wipe 入场中点:重启揭示后 ~dur.panel/2(210ms)。
  await seekShot(page, 210, shot("s3-tool-card", "wipe-mid-210ms"));
  // 全 settled:四态徽章(环/点/勾/叉)+ running 流光。
  await seekShot(page, 60_000, shot("s3-tool-card", "all-states-settled"));
  // running 流光相位两帧:相位差 π = π/2.5 rad·s⁻¹ ≈ 1257ms(慢时钟随引擎时间推进)。
  await seekShot(page, 61_257, shot("s3-tool-card", "glow-phase-b"));
});

// ---- S4 reasoning 卡(M2b:骨架 wipe 入场中点 / settled;auto-collapse 未实现 → 遗留)----
test("S4 reasoning 卡:骨架入场中点 + settled", async ({ page }) => {
  await bootEmpty(page);
  await pushAll(page, [
    msgUpdated("m-a1", "assistant"),
    partUpdated({
      type: "reasoning",
      id: "p-r",
      messageID: "m-a1",
      text: "先分析问题的三个层面,再决定实现顺序。首先是数据结构,其次是调度时序,最后是视觉参数。",
    }),
  ]);
  await setPreset(page, "reader", TEMPO);
  await seekShot(page, 210, shot("s4-reasoning", "enter-mid-210ms"));
  await seekShot(page, 60_000, shot("s4-reasoning", "settled"));
});

// ---- S5 代码块(底板随行揭示生长 / 逐行揭示中 / 闭合上色后)----
test("S5 代码块:揭示中两帧 + settled", async ({ page }) => {
  await bootEmpty(page);
  const code = [
    "看这段实现:",
    "```rust",
    "fn reveal(units: &[Unit]) -> Plan {",
    "    let mut t = 0.0;",
    "    for u in units {",
    "        t += cost(u); // 节奏成本",
    "    }",
    "    Plan::at(t)",
    "}",
    "```",
    "以上。",
  ].join("\n");
  await pushAll(page, [msgUpdated("m-a1", "assistant"), textPart("p-c", "m-a1", code)]);
  await setPreset(page, "reader", TEMPO);
  await seekShot(page, 900, shot("s5-code", "reveal-early"));
  await seekShot(page, 2600, shot("s5-code", "reveal-mid"));
  await seekShot(page, 60_000, shot("s5-code", "settled"));
});

// ---- S6 表格(风格 3 全表门:网格 wipe → 表头 → cell 填充 → settled)----
test("S6 表格:骨架/表头/cell/settled 四帧", async ({ page }) => {
  await bootEmpty(page);
  const table = "对比:\n\n| 能力 | 手法 | 规模 |\n|---|---|---|\n| 文字 | SDF | 任意缩放 |\n| 流式 | 逐字 | 无跳变 |\n| 历史 | 冻结 | 100+ 轮 |\n\n收。";
  await pushAll(page, [msgUpdated("m-a1", "assistant"), textPart("p-t", "m-a1", table)]);
  await setPreset(page, "reader", TEMPO);
  // 全表门:表格在「收。」到达(表闭合)后才揭;骨架(网格 wipe)→ 表头 → cell。
  await seekShot(page, 700, shot("s6-table", "skeleton-grid"));
  await seekShot(page, 1200, shot("s6-table", "header-stage"));
  await seekShot(page, 2600, shot("s6-table", "cells-filling"));
  await seekShot(page, 60_000, shot("s6-table", "settled"));
});

// ---- S7 diff(edit tool filediff:行带 + 摘要 + 5 格条)----
test("S7 diff:行带揭示中 + settled(5 格条)", async ({ page }) => {
  await bootEmpty(page);
  const filediff =
    "@@ -1,4 +1,5 @@\n ctx line\n-old_alpha\n-old_beta\n+new_alpha\n+new_beta\n+new_gamma\n ctx tail\n";
  await pushAll(page, [
    msgUpdated("m-a1", "assistant"),
    partUpdated({
      type: "tool",
      id: "t-ed",
      messageID: "m-a1",
      tool: "edit",
      state: { status: "completed", metadata: { filediff } },
    }),
  ]);
  await setPreset(page, "reader", TEMPO);
  await seekShot(page, 900, shot("s7-diff", "bands-revealing"));
  await seekShot(page, 60_000, shot("s7-diff", "settled-with-bar"));
});

// ---- S8 待机生命感(M2a:光标/orb/指示条 —— 相位两帧 + settled/idle 帧)----
test("S8 待机生命感:流式中相位两帧 + idle 帧", async ({ page }) => {
  await bootEmpty(page);
  await pushAll(page, S1_EVENTS);
  await setPreset(page, "reader", TEMPO);
  const mid = Math.round(TOTAL * 0.5);
  // 0.15Hz 半周期 = 3333ms → 两帧呼吸 alpha 差最大(rubric 10:有可见变化但幅度小)。
  await seekShot(page, mid, shot("s8-idle-life", "streaming-phase-a"));
  await seekShot(page, mid + 3333, shot("s8-idle-life", "streaming-phase-b"));
  // settled/idle:光标/指示条退场、orb 冻结 → 全静态(rubric 7)。
  await seekShot(page, TOTAL * 3, shot("s8-idle-life", "settled-idle"));
});

// ---- S10 交互反馈(M2f:卡片 hover 提亮 / press 微缩)----
test("S10 交互反馈:hover 前后 + press 帧", async ({ page }) => {
  await bootEmpty(page);
  await pushAll(page, [
    msgUpdated("m-a1", "assistant"),
    toolPart("t-h", "m-a1", "completed"),
  ]);
  await page.evaluate(() => {
    window.__chat.set_paused(true);
    window.__chat.seek_reveal(60_000);
  });
  await page.screenshot({ path: shot("s10-interact", "idle") });
  // hover 卡片中心(canvas 布满窗口,卡在列内顶部)。
  await page.mouse.move(500, 80);
  await page.evaluate(() => window.__chat.tick(16));
  await page.screenshot({ path: shot("s10-interact", "hover") });
  await page.mouse.down();
  await page.evaluate(() => window.__chat.tick(16));
  await page.screenshot({ path: shot("s10-interact", "press") });
  await page.mouse.up();
});

// ---- S9 整页 layout(3 轮对话:间距/分组/meta)----
test("S9 整页 layout:settled 整页一帧 + 滚动后一帧", async ({ page }) => {
  await bootEmpty(page);
  await pushAll(page, S9_EVENTS);
  await page.evaluate(() => {
    window.__chat.set_paused(true);
    window.__chat.seek_reveal(60_000); // 全 settled
  });
  await page.screenshot({ path: shot("s9-layout", "settled-top") });
  // 滚动后一帧:往下滚 400px(看回合分组在滚动中段的观感)。
  await page.evaluate(() => {
    window.__chat.pan_by(0, 400);
    window.__chat.tick(0);
  });
  await page.screenshot({ path: shot("s9-layout", "scrolled-400") });
});
