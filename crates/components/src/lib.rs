//! infinite-chat-components — part 组件库(0039 分层):纯函数「语义 → spans + decorations」。
//!
//! 只依赖 primitives(0032:无 widget 对象/无保留树/无事件系统);markdown 解析经
//! [`partrender::RenderCtx::parse`] 注入(解析主体在 core)。core 注入注册表消费本 crate;
//! 第二宿主可独立消费(见 `examples/render_part_dump.rs`)。

pub mod codeblock;
pub mod highlight;
pub mod partrender;
pub mod partspecific;
