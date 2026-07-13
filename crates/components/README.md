# infinite-chat-components

part 组件库(ADR 0039):纯函数注册表「语义 → spans + decorations」(0033/0037 契约)。
只依赖 `infinite-chat-primitives`;markdown 解析经 `RenderCtx.parse` 注入(解析主体在 core)。
无 widget 对象、无保留树、无事件系统(0032)。独立消费示例:
`cargo run -p infinite-chat-components --example render_part_dump`。
