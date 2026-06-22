#!/usr/bin/env node
// gen-showcase —— 生成 GitHub Pages 演示会话 web/public/cases/showcase.json。
// 一段「能力导览」markdown → 切成流式 delta(带时间戳)→ {steps:[{t,delta}]}(replay 格式)。
// 用法:node scripts/gen-showcase.mjs
import { writeFileSync, mkdirSync } from "node:fs";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "..");

// 注:LaTeX 反斜杠在 JS 模板串里要写成 \\。
const MD = `# infinite-chat — 用游戏引擎做 LLM 对话渲染 ✨

把大模型**流式吐字**渲染成一块**可无限缩放平移**的画布。下面这段对话本身正在**离线重放**——所见即引擎能力。

## 流式 markdown
支持 **粗体**、*斜体*、~~删除线~~、\`行内代码\`、[链接](https://github.com/OhBonsai/infinite-chat),以及行内数学 $e^{i\\pi}+1=0$。

> [!NOTE]
> 引用块与 GitHub Alert 都走 SDF 渲染,任意缩放都锐利。

### 列表 / 任务
- 无序项一
- 无序项二
  - 嵌套项
1. 有序第一
2. 有序第二

- [x] 已完成的任务
- [ ] 待办任务
- [ ] 流式出现时,复选框随单元浮现

### 代码块
\`\`\`rust
// GPU 实例化 + 块冻结:历史无限长也只算「可见的一屏」
fn render(frame: &Frame) {
    for node in frame.visible() {
        gpu.draw_sdf(node);
    }
}
\`\`\`

### 表格
| 能力 | 手法 | 规模 |
|:--|:-:|--:|
| 文字 | SDF / MSDF | 任意缩放锐利 |
| 流式 | 逐字 reveal | 全程无跳变 |
| 历史 | settled 冻结 | 100+ 轮依旧丝滑 |

### 显示数学
$$\\int_{-\\infty}^{\\infty} e^{-x^2}\\,dx = \\sqrt{\\pi}$$

---

CJK 与 emoji 混排也稳:你好,世界 🌍🚀。脚注同样支持[^1]。

[^1]: 这是一条脚注。

**滚轮缩放、拖拽平移** 试试看 —— 这是一块无边画布,不是 DOM。`;

// 按 code point 切(避免切断 emoji 代理对 / CJK)。每块 2–6 个字符,结构换行处停顿更久。
const cps = Array.from(MD);
const steps = [];
let t = 0;
let i = 0;
while (i < cps.length) {
  const n = 2 + Math.floor(Math.random() * 5); // 2..6
  const delta = cps.slice(i, i + n).join("");
  i += n;
  steps.push({ t: Math.round(t), delta });
  t += 28 + Math.random() * 26;                 // ~28–54ms/块
  if (delta.includes("\n\n") || /\n#{1,6}\s/.test(delta)) t += 160; // 段/标题边界停顿
  if (/```/.test(delta)) t += 120;
}

const out = {
  _doc: "GitHub Pages 演示会话(能力导览)。脚本生成:scripts/gen-showcase.mjs。VITE_DEMO 默认重放此 case。",
  sessionID: "showcase",
  messageID: "m1",
  partID: "p1",
  steps,
};

const dir = resolve(ROOT, "web/public/cases");
mkdirSync(dir, { recursive: true });
const path = resolve(dir, "showcase.json");
writeFileSync(path, JSON.stringify(out, null, 2));
console.log(`wrote ${path}: ${steps.length} steps, ~${Math.round(t / 1000)}s`);
