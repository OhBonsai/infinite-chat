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
  "c06-all",
  "c06-table",
  "c06b-table-align",
  "c06c-table-cjk",
  "c06d-table-inline",
  "c06e-table-ragged",
  "c06f-table-wide",
  "c07-setext",
  "c08-quote-alert",
  "c09-mixed-long",
  "c10-cjk",
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
    return {
      case: typeof c.case === "string" && c.case ? c.case : null,
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
