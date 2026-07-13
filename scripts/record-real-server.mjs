#!/usr/bin/env node
// record-real-server.mjs — Plan 35 T3(plan24 PR-4):对真 opencode server 录全事件流录像。
//
// 真网络只发生在**录制时**(T3 铁律的合规路径);录像入 test/replays/real-server/,
// 每卷配 native 重放测试(双跑一致 R8 + 末态快照)。录制脚本本身可不确定,录像必须自洽。
//
// 用法(server 需已起,kill-reconnect 场景自起自管 4097):
//   node scripts/record-real-server.mjs --scenario qa-short
//   node scripts/record-real-server.mjs --scenario markdown-long | tool-call | kill-reconnect
//   MODEL=aliyuntokenplan/qwen3.7-max SERVER=http://localhost:4096 可覆盖。
//
// 输出:test/replays/real-server/<scenario>.json —— [{ t, raw }](t = 录制起点毫秒偏移),
// 与 web/public/replays 同格式;含 heartbeat/connected 噪音(引擎须忽略,AR12 的真世界样本)。

import { spawn } from "node:child_process";
import { mkdirSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const OUT_DIR = path.join(ROOT, "test", "replays", "real-server");
const argv = process.argv.slice(2);
const opt = (n, d) => {
  const i = argv.indexOf(n);
  return i >= 0 ? argv[i + 1] : d;
};
const SCENARIO = opt("--scenario", "qa-short");
const MODEL_STR = process.env.MODEL ?? "aliyuntokenplan/qwen3.7-max";
const slash = MODEL_STR.indexOf("/");
const MODEL = { providerID: MODEL_STR.slice(0, slash), modelID: MODEL_STR.slice(slash + 1) };

// ── 录音机:SSE → [{t,raw}];断线自动重连(重连本身也是录像内容:server.connected 再现)──
function makeRecorder(server) {
  const t0 = Date.now();
  const events = [];
  let stop = false;
  let lastEventAt = Date.now();
  (async function pump() {
    while (!stop) {
      try {
        const res = await fetch(`${server}/event`, { headers: { accept: "text/event-stream" } });
        const reader = res.body.getReader();
        const dec = new TextDecoder();
        let buf = "";
        for (;;) {
          const { done, value } = await reader.read();
          if (done || stop) break;
          buf += dec.decode(value, { stream: true });
          let i;
          while ((i = buf.indexOf("\n\n")) >= 0) {
            const chunk = buf.slice(0, i);
            buf = buf.slice(i + 2);
            const data = chunk
              .split("\n")
              .filter((l) => l.startsWith("data:"))
              .map((l) => l.slice(5).trim())
              .join("");
            if (data) {
              events.push({ t: Date.now() - t0, raw: data });
              lastEventAt = Date.now();
            }
          }
        }
      } catch {
        // 断线(kill 场景预期):稍候重连,重连成功后 server.connected 会进录像。
      }
      if (!stop) await new Promise((r) => setTimeout(r, 500));
    }
  })();
  return {
    events,
    stop: () => {
      stop = true;
    },
    quietFor: (ms) => Date.now() - lastEventAt >= ms,
  };
}

async function post(server, p, body) {
  const res = await fetch(`${server}${p}`, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify(body ?? {}),
  });
  const text = await res.text();
  if (!res.ok) throw new Error(`${res.status} @ ${p}: ${text.slice(0, 200)}`);
  try {
    return JSON.parse(text);
  } catch {
    return text;
  }
}

/** 等到「安静 quietMs 且已见任意 assistant part」或超时。 */
async function waitDone(rec, { quietMs = 10_000, capMs = 180_000 } = {}) {
  const t0 = Date.now();
  for (;;) {
    const sawParts = rec.events.some((e) => e.raw.includes("message.part"));
    if (sawParts && rec.quietFor(quietMs)) return;
    if (Date.now() - t0 > capMs) return;
    await new Promise((r) => setTimeout(r, 500));
  }
}

/** 录像期间自动应答 permission(tool-call 场景;应答动作产生的事件也入录像)。 */
function autoApprove(server, rec) {
  const seen = new Set();
  const timer = setInterval(async () => {
    for (const e of rec.events) {
      if (!e.raw.includes("permission")) continue;
      try {
        const env = JSON.parse(e.raw);
        const id = env?.properties?.id ?? env?.properties?.permissionID;
        const sid = env?.properties?.sessionID;
        if (!id || seen.has(id)) continue;
        seen.add(id);
        await post(server, `/session/${sid}/permissions/${id}`, { response: "always" }).catch(() =>
          post(server, `/permission/${id}/reply`, { response: "always" }),
        );
      } catch {
        /* 非 JSON / 无 id:跳过 */
      }
    }
  }, 800);
  return () => clearInterval(timer);
}

const PROMPTS = {
  "qa-short": "用一句话回答:SSE(Server-Sent Events)是什么?",
  "markdown-long":
    "写一篇 markdown 小文介绍 Rust 所有权:包含一个二级标题、一个 3 行表格、一个 rust 代码块、一个无序列表和一个指向 https://doc.rust-lang.org 的链接。控制在 300 字内。",
  "tool-call": "运行 bash 命令 `echo hello-recording && date +%Y` 并把输出原样告诉我。",
  "kill-reconnect":
    "请分五段慢慢介绍 TCP 三次握手,每段至少三句话。", // 长回复给 kill 窗口
};

async function main() {
  mkdirSync(OUT_DIR, { recursive: true });
  let server = process.env.SERVER ?? "http://localhost:4096";
  let child = null;

  if (SCENARIO === "kill-reconnect") {
    // 自起自管(4097):录到中途 kill -9 → 重启 → 观察 server.connected 再现与后续对账。
    server = "http://localhost:4097";
    const start = () =>
      spawn("opencode", ["serve", "--port", "4097"], { stdio: "ignore", detached: false });
    child = start();
    await waitUp(server);
    const rec = makeRecorder(server);
    const stopApprove = autoApprove(server, rec);
    const s = await post(server, "/session", {});
    const sid = s.id ?? s.sessionID;
    // 异步发(不等回完),等首个 delta 后 2s kill。
    post(server, `/session/${sid}/message`, {
      parts: [{ type: "text", text: PROMPTS[SCENARIO] }],
      model: MODEL,
    }).catch(() => {});
    const t0 = Date.now();
    while (!rec.events.some((e) => e.raw.includes("message.part.delta")) && Date.now() - t0 < 60_000)
      await new Promise((r) => setTimeout(r, 200));
    await new Promise((r) => setTimeout(r, 2000));
    child.kill("SIGKILL");
    await new Promise((r) => setTimeout(r, 3000)); // 死机窗口(录像里是静默 + 断线)
    child = start();
    await waitUp(server);
    await new Promise((r) => setTimeout(r, 4000)); // 重连 → server.connected 入卷
    // 重启后补一问,证明会话可续(快照对账素材)。
    await post(server, `/session/${sid}/message`, {
      parts: [{ type: "text", text: "刚才断了。请用一句话总结你上面讲到哪了。" }],
      model: MODEL,
    }).catch(() => {});
    await waitDone(rec);
    stopApprove();
    rec.stop();
    child.kill("SIGKILL");
    dump(rec.events);
    return;
  }

  const rec = makeRecorder(server);
  const stopApprove = autoApprove(server, rec);
  const s = await post(server, "/session", {});
  const sid = s.id ?? s.sessionID;
  await post(server, `/session/${sid}/message`, {
    parts: [{ type: "text", text: PROMPTS[SCENARIO] }],
    model: MODEL,
  });
  await waitDone(rec);
  stopApprove();
  rec.stop();
  dump(rec.events);

  function noop() {}
  noop(child);
}

async function waitUp(server) {
  for (let i = 0; i < 60; i++) {
    if (await fetch(`${server}/doc`).then((r) => r.ok).catch(() => false)) return;
    await new Promise((r) => setTimeout(r, 500));
  }
  throw new Error(`server 未就绪: ${server}`);
}

function dump(events) {
  const out = path.join(OUT_DIR, `${SCENARIO}.json`);
  writeFileSync(out, JSON.stringify(events, null, 0));
  console.log(`→ ${out}  (${events.length} events, ${(events.at(-1)?.t ?? 0) / 1000}s)`);
}

main()
  .then(() => process.exit(0)) // SSE fetch 不可取消地占着事件循环 → 显式退出
  .catch((e) => {
    console.error(e);
    process.exit(1);
  });
