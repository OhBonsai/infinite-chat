// replay(Plan 5D)— 载 case(带时间戳的 text delta 序列)→ 包成 opencode `message.part.delta`
// 信封 → `ChatCanvas` 的 `replay` 配置(经 core `Player` 喂,替代 synthetic;不连服务端)。
//
// case 文件:`web/public/cases/<name>.json` = `{ steps: [{t, delta}], sessionID?, messageID?, partID? }`。
// 用法:`?replay=<name>`(见 main.ts)。逐条覆盖 5C 规格表,验"全程无跳变"。

export interface ReplayStep {
  /// 该 delta 的相对时间戳(ms)。
  t: number;
  /// 追加的文本片段(拼接 = 完整 markdown)。
  delta: string;
}

export interface ReplayCase {
  sessionID?: string;
  messageID?: string;
  partID?: string;
  steps: ReplayStep[];
}

/// 传给 `ChatCanvas` 的记录(与 wasm `Record{t,raw}` 一致)。
export interface ReplayRecord {
  t: number;
  raw: string;
}

/// case → Record[]:每步包成一条 `message.part.delta` 信封(opencode 实测格式)。
/// `speed`:回放倍率,<1 = 慢放(把时间戳拉长,便于肉眼/单步看过渡)。默认 1。
export function buildReplay(c: ReplayCase, speed = 1): ReplayRecord[] {
  const sessionID = c.sessionID ?? "replay";
  const messageID = c.messageID ?? "m";
  const partID = c.partID ?? "p1";
  const k = speed > 0 ? 1 / speed : 1; // speed=0.2 → t×5(5 倍慢)
  return c.steps.map((s) => ({
    t: Math.round(s.t * k),
    raw: JSON.stringify({
      type: "message.part.delta",
      properties: { sessionID, messageID, partID, field: "text", delta: s.delta },
    }),
  }));
}

/// 拉 case 文件并构造 replay records。`speed`<1 慢放(见 buildReplay)。
export async function loadCase(name: string, speed = 1, base = "/cases"): Promise<ReplayRecord[]> {
  const c: ReplayCase = await fetch(`${base}/${encodeURIComponent(name)}.json`).then((r) => {
    if (!r.ok) throw new Error(`replay case 不存在: ${name} (${r.status})`);
    return r.json();
  });
  if (!Array.isArray(c.steps) || c.steps.length === 0) {
    throw new Error(`replay case ${name} 无 steps`);
  }
  return buildReplay(c, speed);
}
