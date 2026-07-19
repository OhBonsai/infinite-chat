# Plan 47 · 进度账本(TUI TodoWrite 组件)

> 逐 milestone 记录:参考依据(file:行号)/ 符号映射 / native 数值 / 恒等验证 / 遗留。
> GOAL:[plan47-tui-todowrite-goal](./plan47-tui-todowrite-goal.md)
> mock 已就位:`web/public/chats/todo.json`(4 个 todowrite part,running 空 + completed 清单,清单随进度变化;`?script=todo` 跑)。

> **起点事实(2026-07-19 摸源码)**:
> - claude-code:V1 TodoWrite 无 TUI 清单(只回灌上下文);可见清单 = V2 `components/TaskListV2.tsx`:符号 `◻/◼/✔`(:220-241)、in_progress 加粗 / completed 删除线+dim(:313)、standalone 头 `N tasks (X done...)`(:194-205)、原序(:166)。字段 content/status/activeForm(`utils/todo/types.ts:4-14`),status 三态(:5)。
> - opencode:`todowrite` tool(`packages/core/src/tool/todowrite.ts:10`),item content/status/priority(`session/todo.ts:10-16`),status 四态含 cancelled(:13)。TUI `component/todo-item.tsx`:符号 `[✓]/[•]/[ ]`(:19),in_progress→warning 余→textMuted(:16/25),无删除线。消息流作 `<BlockTool title="# Todos">`(`routes/session/index.tsx:2464`),更新中→InlineTool `⚙ Updating todos...`(:2471-2479);`parseTodos`(:2612)读 input.todos/metadata.todos。
> - infinite-chat 落点:tool 走 `PartKind::Tool` 一级(`crates/components/src/partspecific.rs:28`),二级看工具名(`store.rs:545` `kind_tag: "tool:{name} · {status}"`);`parse_tool_tag`(:481)、`tool_render`(:267)、`tool_render_full`(:406)、`tui_tool_render_full`(:60)按 name match。`is_diff_tool`(:491)是同类谓词范例;`tool_display_name`(:469)已有 `todowrite=>"To-dos"`。

## P0 · 解析 + 谓词(components 纯函数)

✅ (2026-07-19,`53b28f8`)
- [x] `TodoStatus`(4 态:Pending/InProgress/Completed/Cancelled,未知→Pending)+ `Todo{content,status,priority?,active_form?}` + `parse_todos(payload_json)`(读 `input.todos`,兜底 `metadata.todos`/顶层;坏 JSON→空 AR12)。
- [x] `is_todo_tool(name)`(`todowrite|todoread|todo`),partspecific.rs near is_diff_tool。
- [x] native:`parse_todos_four_states_and_bad_json`(四态 + 未知→pending + 坏 JSON→空 + 无 todos→空)。

## P1 · 清单卡渲染器

✅ (2026-07-19,`53b28f8`)
- [x] `tool_render` 加 `is_todo_tool` 分支 → `todo_render(payload)`:header `Todos · X/N done`(X=completed 数)+ 逐 todo 行 `[符号] content`。
- [x] 符号 `[ ]/[~]/[✓]/[✗]`(§0 goal 定档;in_progress=`[~]`);completed/cancelled → `Reasoning`(dim)+ `StyledSpan::styled(strike=true)` 划除;in_progress → `ToolTitle` 强调;pending → `Normal`。
- [x] 目视 `?script=todo` completed 清单卡(dim+划除 completed、亮 in_progress、中性 pending;截图实证)。native `todowrite_renders_bracket_checklist`(X/N + 四符号 + 划除 + 强调)+ golden `tool_todo_list`。

## P2 · running 态 + flavor 双档

✅ (2026-07-19,`53b28f8`)
- [x] running/空 todos → 单行 `Updating todos…`;tui_tool_render_full 加 icon → `⚙ Updating todos…`(InlineTool)。`tui_tool_is_inline` 随态:running=inline(单行,参与兄弟贴合),completed=block(块清单卡,不贴合)。
- [x] flavor 双档:tui + rich 复用同 `todo_render`(方括号 + 划除,两 flavor 皆可渲);**rich 恒等** `replay_settled_frame_snapshot` 逐字节不变(todowrite 属新分支,旧剧本不触)。
- [x] native `todowrite_running_degrades_to_inline`(单行 + 态门)+ golden `tool_todo_running`(`⚙ Updating todos…`)。

## P3 · 收口

✅ (2026-07-19)
- [x] golden 双态:`tool_todo_list`(块清单卡:header + `[✓]a`/`[~]b`/`[ ]c`/`[✗]d`,dim/强调 role)+ `tool_todo_running`(`⚙ Updating todos…`)。insta role|text 快照。
- [x] native ≥4:parse 四态 + bracket 清单(符号/X-N/划除/强调)+ running 退化 + 两 golden = 4 项 aspect。
- [x] e2e ≥2:`web/tests/todo.spec.ts` —— ① 清单卡逐项(Todos·X/N + 5 content + ≥5 方括号 + 不露 JSON)② running→completed 替换零 pageerror + 内容不丢。
- [ ] 可选 showcase-full 并入 todowrite —— 未做(todo.json 独立 mock 已覆盖;记后续,避免动 showcase golden)。

## DoD 对账

| # | DoD 要求 | 状态 | 证据 |
|---|---|---|---|
| 1 | todo 解析(纯函数,四态,坏 JSON→空) | ✅ | parse_todos + TodoStatus;native parse_todos_four_states_and_bad_json |
| 2 | 清单渲染器(header X/N + 逐行符号/划除) | ✅ | todo_render;native bracket_checklist + golden tool_todo_list |
| 3 | running 态 InlineTool | ✅ | ⚙ Updating todos…;native running_degrades_to_inline + golden tool_todo_running |
| 4 | flavor 双档 + rich 恒等 | ✅ | tui/rich 复用 todo_render;replay_settled 逐字节 |
| 5 | mock 驱动(?script=todo) | ✅ | todo.json;e2e 清单卡逐项 |
| 6 | 门(五层 + rich 恒等 + tui todowrite golden + native≥4 + e2e≥2) | ◐ | native 4 / e2e 2 / golden 两态 / rich 恒等;五层门待跑 |
| 7 | progress 记账 + commit 不 push | ✅ | 本文件;`53b28f8`(P0-P2),P3 本轮;不 push |

**§3 客观判定**:清单正确 ✅(e2e 行数/X-N)· 态符号 ✅(native 四符号映射 + golden)· running 退化 ✅(native + golden)· rich 恒等 ✅(replay_settled 逐字节)· flavor ✅(tui/rich 复用)· 门 ◐(native/e2e/golden 绿;五层门跑中)。

**符号定档注**:goal §0 定 in_progress=`[~]`(本实现遵此);progress §0 摸源码记 opencode 用 `[•]`——以 goal 为准(`[~]` mono 更稳、与 `[✗]` 一致方括号族)。
