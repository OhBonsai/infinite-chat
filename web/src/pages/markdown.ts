// markdown.ts — Plan 40 markdown 页入口(P5):引擎挂 #md-canvas 跑全类型流式揭示轮播 +
// textarea→合成流实时 playground。**引擎零特判**:样例/粘贴内容都经 push_event 进引擎(同 chat 页)。
import "./pages-theme.css";
import { mountNav } from "./pages-nav";
import { bootHero } from "../boot";

mountNav("markdown");

// 全类型样例轮播:每条覆盖一类富文本(行内/列表/代码/表格/公式/引用+Alert/任务),循环流式揭示。
const SAMPLES: { label: string; md: string }[] = [
  {
    label: "行内与强调",
    md: "# 行内样式\n\n**粗体**、*斜体*、~~删除线~~、`行内代码`,以及 [链接](https://github.com/OhBonsai/infinite-chat)。\n\n中英混排也严丝合缝:SDF 文字图元,任意缩放锐利。",
  },
  {
    label: "列表与任务",
    md: "# 列表与任务框\n\n- 无序一\n- 无序二\n  - 嵌套子项\n\n1. 有序一\n2. 有序二\n\n- [x] 已完成的任务\n- [ ] 待办任务",
  },
  {
    label: "代码块",
    md: "# 代码块(语法高亮)\n\n```rust\nfn render(frame: &Frame) {\n    for node in frame.visible() {\n        node.draw(); // GPU 实例化 + 视口裁剪\n    }\n}\n```\n\n超长代码折叠成可滚动行窗,右上角有 copy 按钮。",
  },
  {
    label: "表格",
    md: "# 表格(SDF 网格严丝合缝)\n\n| 能力 | 手法 | 规模 |\n| --- | --- | --- |\n| 文字 | SDF / MSDF | 任意缩放锐利 |\n| 流式 | 逐字 reveal | 全程无跳变 |\n| 历史 | settled 派绘 | 100+ 轮回旧丝滑 |",
  },
  {
    label: "数学公式",
    md: "# 数学公式(SDF 排版)\n\n行内:质能 $E = mc^2$,欧拉恒等式 $e^{i\\pi} + 1 = 0$。\n\n块级求和:\n\n$$\\sum_{i=1}^{n} i = \\frac{n(n+1)}{2}$$",
  },
  {
    label: "引用与 Alert",
    md: "# 引用块与 GitHub Alert\n\n> 普通引用:引擎在跑,你看到的就是宣传片本身。\n\n> [!NOTE]\n> GitHub Alert 也走 SDF 渲染,任意缩放都锐利。\n\n> [!WARNING]\n> 流式前沿留 hold 区,不提交歧义语法字节。",
  },
];

function fatal(msg: string): void {
  const wrap = document.querySelector(".md-canvas-wrap");
  if (!wrap) return;
  const el = document.createElement("div");
  el.style.cssText =
    "margin:auto;padding:24px;color:var(--ink-dim);font:13px/1.7 system-ui;text-align:center;max-width:30em";
  el.textContent = msg;
  wrap.appendChild(el);
}

async function main(): Promise<void> {
  const canvas = document.getElementById("md-canvas") as HTMLCanvasElement | null;
  if (!canvas) return;
  const { chat, ok } = await bootHero({ canvasId: "md-canvas", rhythmPreset: "reader" });
  if (!ok || !chat) {
    canvas.style.display = "none";
    fatal("此浏览器无法启动 WebGPU/WebGL2 —— Markdown 演示需要引擎。用 Chrome 打开可看全类型流式揭示。");
    return;
  }

  // 合成流:单回合**替换**式轮播 —— 建一次 assistant 消息,每次喂**新 part id**(干净重播:同 id
  //   upsert 会沿用已揭示态、不重现;换 id 才从头揭示)+ 删上一条 part(画布始终只一条,不累积)。
  const SID = "md";
  const MID = "md-msg";
  let msgCreated = false;
  let seq = 0;
  let prevPID = "";
  const feed = (md: string): void => {
    if (!msgCreated) {
      chat.push_event(
        JSON.stringify({
          type: "message.updated",
          properties: { info: { id: MID, role: "assistant", sessionID: SID } },
        }),
      );
      msgCreated = true;
    }
    const pid = `md-p-${seq++}`;
    chat.push_event(
      JSON.stringify({
        type: "message.part.updated",
        properties: {
          part: { type: "text", id: pid, messageID: MID, sessionID: SID, text: md },
          time: 1,
        },
      }),
    );
    if (prevPID) {
      chat.push_event(
        JSON.stringify({
          type: "message.part.removed",
          properties: { sessionID: SID, messageID: MID, partID: prevPID },
        }),
      );
    }
    prevPID = pid;
  };

  const CAROUSEL_MS = 7000;
  let timer = 0;
  let idx = 0;
  const stopCarousel = (): void => {
    if (timer) window.clearInterval(timer);
    timer = 0;
  };
  const startCarousel = (): void => {
    stopCarousel();
    const step = (): void => {
      if (document.hidden) return; // 隐藏页不推进(下方 visibilitychange 兜暂停)
      feed(SAMPLES[idx % SAMPLES.length].md);
      idx++;
    };
    step();
    timer = window.setInterval(step, CAROUSEL_MS);
  };

  // 引擎就绪门(同 hero:start() 异步建 App,未就绪前 set_effect_preset 返回 false 皆 no-op)。
  const whenReady = (tries = 0): void => {
    if (!chat.set_effect_preset("expressive")) {
      if (tries < 600) requestAnimationFrame(() => whenReady(tries + 1));
      return;
    }
    startCarousel();
  };
  whenReady();

  // Playground:粘贴任意 markdown → 点「流式播放」合成流喂引擎(停轮播,专注这段)。
  const input = document.getElementById("md-input") as HTMLTextAreaElement | null;
  const run = document.getElementById("md-run");
  run?.addEventListener("click", () => {
    stopCarousel();
    const md = input?.value ?? "";
    if (md.trim()) feed(md);
  });

  // 页签隐藏:暂停引擎帧泵 + 停轮播;回来恢复。
  document.addEventListener("visibilitychange", () => {
    chat.set_paused(document.hidden);
    if (document.hidden) stopCarousel();
    else if (!timer) startCarousel();
  });
}

void main();
