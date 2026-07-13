//! 独立消费示例(plan33 DoD-4):不链 core/render/wasm —— 喂一个 tool/diff RenderPart,
//! 打印 spans + decorations。这是「组件可被第二宿主消费」的最小证明。
//! 跑法:`cargo run -p infinite-chat-components --example render_part_dump`

use infinite_chat_components::partrender::{PartKind, RenderCtx, RenderPart};
use infinite_chat_components::partspecific::default_registry;
use infinite_chat_primitives::style::plain;

fn main() {
    let part = RenderPart {
        kind_tag: "tool:edit · done".into(),
        text: String::new(),
        payload_json: Some(
            r#"{"tool":"edit","state":{"status":"completed","input":{"path":"a.rs"},
                "metadata":{"filediff":"@@ -1,3 +1,3 @@\n ctx\n-old\n+new\n"}}}"#
                .into(),
        ),
    };
    let ctx = RenderCtx {
        width: 640.0,
        folded: false,
        unfolded: 0,
        parse: plain, // 第二宿主可注入自己的 markdown 解析;此处纯文本直通
    };
    let out = default_registry().render_full(PartKind::Tool, &part, &ctx);
    println!("── spans({}) ──", out.spans.len());
    for sp in &out.spans {
        println!("{:>12?} | {:?}", sp.role(), sp.text());
    }
    println!("── decorations({}) ──", out.decorations.len());
    for d in &out.decorations {
        println!("{d:?}");
    }
}
