//! Plan 20 §1.1:确定性重放快照(护城河测试)。
//!
//! 本项目 R8 确定性 + record/replay 就位 → "同 case 同 seed → 同末状态/同画面"天然成立。
//! 于是**一个**测试就能守住一大类回归(闪/跳变/排版错/卡死):
//!
//! ```text
//! 录像 case → 载满内容 → seek_reveal(固定 target_ms) → 序列化「末状态 FrameData(redact spawn_time)」→ insta 快照
//! ```
//!
//! 改了 reveal/layout/虚拟化后,`cargo insta review` 一眼看出"画面变没变、变得对不对"。
//! 比写一堆断言便宜,覆盖面又大。
//!
//! 快照**刻意 redact 掉时序/相机字段**(`spawn_time`/`time_ms`/`cam_*`):它们是呈现时刻细节,
//! 与"末状态画面"(几何 + 文本 + 样式 + 块身份)正交;redact 后快照只盯排版/内容回归,
//! 不被 reveal 节奏微调刷红。

use std::fmt::Write as _;

use infinite_chat_core::{CollectSink, Engine, FrameData, MonospaceLayout, Player};

/// 构造一条 `message.part.delta` 原始事件(与 core 单测同源,protocol 解码路径一致)。
fn delta(part: &str, text: &str) -> String {
    format!(
        r#"{{"type":"message.part.delta","properties":{{"sessionID":"s","messageID":"m","partID":"{part}","field":"text","delta":{text:?}}}}}"#
    )
}

/// 末状态 FrameData → 确定性文本摘要(redact 时序/相机)。
///
/// 浮点四舍五入到 1 位小数,抹去 f32 末位噪声;只留与回归相关的:
/// 字形(文本 + 几何 + 样式 + 块身份 + anim/alpha)、面板(几何 + 网格比例 + 揭示)、各图元计数、末状态可见文本。
fn redacted_frame(f: &FrameData) -> String {
    let r1 = |x: f32| (x * 10.0).round() / 10.0;
    let r2 = |x: f32| (x * 100.0).round() / 100.0;
    let mut out = String::new();

    let _ = writeln!(out, "glyphs: {}", f.glyphs.len());
    for g in &f.glyphs {
        let _ = writeln!(
            out,
            "  {:?} pos=[{},{}] size=[{},{}] style={} block={} idx={} anim={} alpha={}",
            g.cluster,
            r1(g.pos[0]),
            r1(g.pos[1]),
            r1(g.size[0]),
            r1(g.size[1]),
            g.style,
            g.block_seq,
            g.glyph_idx,
            g.anim,
            r2(g.alpha),
        );
    }

    let _ = writeln!(out, "rects: {}", f.rects.len());
    let _ = writeln!(out, "panels: {}", f.panels.len());
    for p in &f.panels {
        let cols: Vec<f32> = p.col_ratios.iter().map(|&x| r2(x)).collect();
        let rows: Vec<f32> = p.row_ratios.iter().map(|&x| r2(x)).collect();
        let _ = writeln!(
            out,
            "  id={} pos=[{},{}] size=[{},{}] reveal={} cols={:?} rows={:?}",
            p.id,
            r1(p.pos[0]),
            r1(p.pos[1]),
            r1(p.size[0]),
            r1(p.size[1]),
            r2(p.reveal),
            cols,
            rows,
        );
    }

    let _ = writeln!(out, "images: {}", f.images.len());
    let _ = writeln!(out, "embeds: {}", f.embeds.len());
    let _ = writeln!(out, "widgets: {}", f.widgets.len());
    let _ = writeln!(out, "shaderboxes: {}", f.shaderboxes.len());

    // 末状态:可见文本(按 glyph 顺序拼接,顺序无损)。
    let text: String = f.glyphs.iter().map(|g| g.cluster.as_str()).collect();
    let _ = writeln!(out, "text: {text:?}");
    out
}

/// 一条覆盖多块类型的合成 case:标题 / 段落 / 列表 / 表格(出 panel)/ 代码块。
/// 单条 part、一次性投递 → 完全确定;`seek_reveal` 重放揭示到 settled 末态。
#[test]
fn replay_settled_frame_snapshot() {
    let md = "# Title\n\nHello world, streaming text.\n\n- item one\n- item two\n\n| a | b |\n|---|---|\n| 1 | 2 |\n\n```rust\nfn main() {}\n```\n";

    // base_cps 极大 → 几帧载满内容(规模/末态测试惯例,Plan 18);max_width 800。
    let player = Player::from_pairs(vec![(0.0, delta("p1", md))], 16.0);
    let mut eng = Engine::new(
        player,
        MonospaceLayout::default(),
        CollectSink::default(),
        100_000.0,
        800.0,
    );
    // 先把内容流入并排版冻结。
    for _ in 0..40 {
        eng.frame(16.0);
    }
    // 再确定性重放揭示到固定时间轴末态(默认不限速 → 全部揭示)。
    eng.seek_reveal(2000.0);

    let frame = eng.sink().last().expect("应有末帧");
    insta::assert_snapshot!(redacted_frame(frame));
}

/// Plan 45:tui flavor 末帧摘要 —— 含 panel `radius/ao/glow`(锁「装饰扁平」= 全零)+ 计数。
/// 与 rich `redacted_frame` 分开(不动其 golden);geometry 同 rich(flavor 只改色/圆角),此摘要盯扁平。
fn redacted_frame_flat(f: &FrameData) -> String {
    let r1 = |x: f32| (x * 10.0).round() / 10.0;
    let mut out = String::new();
    let _ = writeln!(out, "glyphs: {}", f.glyphs.len());
    let _ = writeln!(out, "rects: {}", f.rects.len());
    let _ = writeln!(
        out,
        "panels: {} (radius/ao/glow → tui 全零)",
        f.panels.len()
    );
    for p in &f.panels {
        let _ = writeln!(
            out,
            "  id={} pos=[{},{}] size=[{},{}] radius={} ao={} glow={}",
            p.id,
            r1(p.pos[0]),
            r1(p.pos[1]),
            r1(p.size[0]),
            r1(p.size[1]),
            r1(p.radius),
            r1(p.ao),
            r1(p.edge_glow),
        );
    }
    let text: String = f.glyphs.iter().map(|g| g.cluster.as_str()).collect();
    let _ = writeln!(out, "text: {text:?}");
    out
}

/// tui flavor golden 家族:同内容 tui 观感 → 装饰扁平(圆角/AO/glow=0),确定性末帧。
#[test]
fn replay_tui_flavor_frame_snapshot() {
    let md = "# Title\n\nHello world, streaming text.\n\n- item one\n- item two\n\n| a | b |\n|---|---|\n| 1 | 2 |\n\n```rust\nfn main() {}\n```\n";
    let player = Player::from_pairs(vec![(0.0, delta("p1", md))], 16.0);
    let mut eng = Engine::new(
        player,
        MonospaceLayout::default(),
        CollectSink::default(),
        100_000.0,
        800.0,
    );
    assert!(eng.set_render_flavor("tui"), "tui 接受");
    for _ in 0..40 {
        eng.frame(16.0);
    }
    eng.seek_reveal(2000.0);
    let frame = eng.sink().last().expect("应有末帧");
    insta::assert_snapshot!(redacted_frame_flat(frame));
}
