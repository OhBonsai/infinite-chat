//! infinite-chat-primitives — 图元词汇 + 视觉 token(0039 分层:最底层,零 workspace 依赖)。
//!
//! 内容 = 「纯数据 + 纯函数」:帧指令(frame)、ShaderBox 图元、主题/动效 token、
//! 渲染样式角色与 span(style)、装饰指令(decoration,0037)、grapheme 切分(text)。
//! **无引擎机制**(store/fsm/reveal 在 core)、**无平台依赖**(CR1');
//! components/core/render 均依赖本 crate,禁反向。

pub mod decoration;
pub mod effects;
pub mod frame;
pub mod motion;
pub mod shaderbox;
pub mod style;
pub mod text;
pub mod theme;
