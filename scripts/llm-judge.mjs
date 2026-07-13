#!/usr/bin/env node
// llm-judge.mjs — Plan 35 T5(可选档):LLM pairwise 判图最小脚本(跑通即可,不进门槛)。
// 把两张渲染样张(data URL file part)交本地 opencode 模型,盲问 A/B 哪张观感更好 + 理由。
// 用法:node scripts/llm-judge.mjs a.png b.png [--model provider/model] → 打印判词。
import { readFileSync } from "node:fs";

const [a, b] = process.argv.slice(2).filter((x) => !x.startsWith("--"));
const mi = process.argv.indexOf("--model");
const MODEL_STR = mi >= 0 ? process.argv[mi + 1] : (process.env.MODEL ?? "aliyuntokenplan/qwen3.7-max");
const slash = MODEL_STR.indexOf("/");
const MODEL = { providerID: MODEL_STR.slice(0, slash), modelID: MODEL_STR.slice(slash + 1) };
const SERVER = process.env.SERVER ?? "http://localhost:4096";
if (!a || !b) {
  console.error("用法: llm-judge.mjs a.png b.png");
  process.exit(2);
}
const dataUrl = (p) => `data:image/png;base64,${readFileSync(p).toString("base64")}`;
const res = await fetch(`${SERVER}/session`, { method: "POST", headers: { "content-type": "application/json" }, body: "{}" });
const sid = (await res.json()).id;
const msg = await fetch(`${SERVER}/session/${sid}/message`, {
  method: "POST",
  headers: { "content-type": "application/json" },
  body: JSON.stringify({
    model: MODEL,
    parts: [
      { type: "text", text: "你是 UI 视觉评审。下面两张是同一聊天界面的渲染样张(A 先 B 后)。盲评:哪张排版/密度/层级观感更好?只答「A」或「B」+ 不超过三条理由。" },
      { type: "file", mime: "image/png", filename: "A.png", url: dataUrl(a) },
      { type: "file", mime: "image/png", filename: "B.png", url: dataUrl(b) },
    ],
  }),
});
const out = await msg.json();
const text = (out.parts ?? []).filter((p) => p.type === "text").map((p) => p.text).join("\n");
if (text) {
  console.log(text);
} else {
  console.log("无文本回复;info.error =", JSON.stringify(out?.info?.error ?? null));
}
