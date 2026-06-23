// replay-config(Plan 5D)— 重放配置(case + speed)持久化到 localStorage。
// 调试面板里选择,reload 生效;不进 URL。main.ts 启动时读取。
//
// 注:CASES 列表与 web/public/cases/*.json 手动保持一致(调试工具,不值得加 manifest 端点)。

export const REPLAY_CASES = [
  "c01-plaintext",
  "c02-bold-close",
  "c03-inline-code",
  "c04-list",
  "c05-fence",
  "c06-all", // 表格全场景(对齐/CJK/内联/残缺/宽),其余 c06* 已并入
  "c07-setext",
  "c08-quote-alert",
  "c09-mixed-long",
  "c10-cjk",
  "n-all", // 节点树调试(Plan 7 / 0020:标题/行内/列表嵌套/引用/代码/表格全含)。配 ?debug 看节点框
  // Plan 9 揭示门控人工验收(切表格风格 + transport 慢放对比;见 plan9 §9):
  "g-table", // 单表逐行流入 + 末段闭合:整表=等闭合才骨架→填;行框=逐行;原始=逐字
  "g-nest", // 嵌套列表逐项:验文档序逐项/逐层
  "g-mixed", // 段落→列表→表格→段落:验块间自上而下 + 每块自风格
  "g-choreo", // 内容瞬到:验编排压过内容(transport 慢放看编排)
  "g-tasks", // 任务复选框(Plan 11):marker → SDF 方框/对勾;混普通列表
  "g-md-all", // 全 markdown 流式综合验收(含复选框/表格/代码/脚注/数学/CJK/emoji)
  "showcase", // GitHub Pages markdown 导览(次要入口 ?replay=showcase)。脚本生成,见 scripts/gen-showcase.mjs
  "reel", // 能力 reel(VITE_DEMO 主页默认 + reel 导演驱动)。脚本生成,见 scripts/gen-reel.mjs
] as const;

/// 慢放倍率选项(1 = 实时,<1 越慢)。
export const REPLAY_SPEEDS = [1, 0.5, 0.25, 0.1, 0.05] as const;

export interface ReplayConfig {
  /// 选中的 case 名;null = 不重放(走合成流/实时)。
  case: string | null;
  /// 回放倍率(见 REPLAY_SPEEDS)。
  speed: number;
}

const KEY = "infinite-chat.replayConfig";
const DEFAULT: ReplayConfig = { case: null, speed: 1 };

export function loadReplayConfig(): ReplayConfig {
  try {
    const raw = localStorage.getItem(KEY);
    if (!raw) return { ...DEFAULT };
    const c = JSON.parse(raw) as Partial<ReplayConfig>;
    // 校验 case 在白名单内:已删除/改名的 case → 当作 null(否则 fetch 命中 SPA 回退、
    // 返回 index.html,`.json()` 撞 `<!doctype` 崩 init)。
    const valid = typeof c.case === "string" && (REPLAY_CASES as readonly string[]).includes(c.case);
    return {
      case: valid ? (c.case as string) : null,
      speed: Number(c.speed) > 0 ? Number(c.speed) : 1,
    };
  } catch {
    return { ...DEFAULT };
  }
}

export function saveReplayConfig(c: ReplayConfig): void {
  try {
    localStorage.setItem(KEY, JSON.stringify(c));
  } catch {
    /* localStorage 不可用(隐私模式等)→ 忽略,退回默认 */
  }
}
