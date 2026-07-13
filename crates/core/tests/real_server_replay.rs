//! 真 server 录像重放(Plan 35 T3 / plan24 PR-4)。
//!
//! 录像由 `scripts/record-real-server.mjs` 对真 opencode server 录制(真网络只在录制时
//! 发生一次,T3 合规);此处离线重放:**双跑逐字节一致(R8)** + 末态 insta 快照。
//! 录像含真实事件噪音(plugin.added/catalog.updated/heartbeat…)—— 未知事件不炸(AR12)
//! 的真世界样本;kill-reconnect 卷含断线静默 + server.connected 再现(0031 韧性素材)。

use infinite_chat_core::{CollectSink, Engine, MonospaceLayout, Player};

fn load(name: &str) -> Vec<(f64, String)> {
    let path = format!(
        "{}/../../test/replays/real-server/{name}.json",
        env!("CARGO_MANIFEST_DIR")
    );
    #[allow(clippy::panic)] // reason: 测试夹具缺失即测试失败,panic 携复现指引正是本意
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("读录像 {path}: {e}(先跑 scripts/record-real-server.mjs)"));
    let evs: Vec<serde_json::Value> = serde_json::from_str(&text).expect("录像 JSON");
    evs.into_iter()
        .map(|e| {
            (
                e["t"].as_f64().expect("t"),
                e["raw"].as_str().expect("raw").to_owned(),
            )
        })
        .collect()
}

/// 重放一卷 → (最终可见文本, 会话状态标签)。揭示即时化(测内容与状态,不测节奏)。
fn run(recs: Vec<(f64, String)>) -> (String, &'static str) {
    let last_t = recs.last().map_or(0.0, |r| r.0);
    let mut eng = Engine::new(
        Player::from_pairs(recs, 16.0),
        MonospaceLayout::default(),
        CollectSink::default(),
        1_000_000.0,
        800.0,
    );
    eng.set_reveal_cps(1.0e9);
    let frames = (last_t / 16.0) as usize + 200; // 播完 + 收尾余量
    for _ in 0..frames {
        eng.frame(16.0);
    }
    (eng.sink().visible_text(), eng.session_status_tag())
}

fn replay_deterministic(name: &str) -> String {
    let (a, sa) = run(load(name));
    let (b, sb) = run(load(name));
    assert_eq!(a, b, "{name}: 双跑可见文本逐字节一致(R8)");
    assert_eq!(sa, sb, "{name}: 双跑会话状态一致");
    assert!(!a.trim().is_empty(), "{name}: 有可见内容");
    format!("status={sa}\n---\n{a}")
}

#[test]
fn real_qa_short_replays_deterministically() {
    let out = replay_deterministic("qa-short");
    insta::assert_snapshot!("real_qa_short", out);
}

#[test]
fn real_markdown_long_replays_deterministically() {
    let out = replay_deterministic("markdown-long");
    insta::assert_snapshot!("real_markdown_long", out);
}

#[test]
fn real_tool_call_replays_deterministically() {
    let out = replay_deterministic("tool-call");
    insta::assert_snapshot!("real_tool_call", out);
}

#[test]
fn real_kill_reconnect_replays_deterministically() {
    // 断线卷:kill -9 静默段 + 重启后 server.connected 再现 + 续问 —— 重放必须稳定走完,
    // 且断前内容不丢(对账/韧性,0031;快照锁定末态)。
    let out = replay_deterministic("kill-reconnect");
    insta::assert_snapshot!("real_kill_reconnect", out);
}
