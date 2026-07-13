# infinite-chat-primitives

图元词汇 + 视觉 token(ADR 0039 分层最底层):帧指令(frame)、ShaderBox 图元、主题/动效
token(theme/motion)、样式角色与 span(style)、装饰指令(decoration,0037)、grapheme
切分(text)。**纯数据 + 纯函数,零 workspace 依赖**(CR1');components/core/render 均依赖
本 crate,禁反向。
