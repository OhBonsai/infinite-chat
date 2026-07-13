# CHECK — 参考 token ↔ 实现 数值对照(Plan 28 §5 / DoD-2)

- 生成:scripts/check-opencode-tokens.mjs;判定:ΔE*ab ≤5、alpha ±0.05、标量 px 级。
- 结果:**32/32 PASS**

| 项 | 实现值 | 参考 | 偏差 | 判定 |
|---|---|---|---|---|
| theme.code_bg | [0.126,0.126,0.126,1] | --surface-inset-base=#202020 | ΔE=0.0 | PASS |
| theme.code_chip(疑点) | [0,0,0,0] | transparent=transparent | alpha 0 | PASS |
| theme.quote_bar | [0.157,0.157,0.157,1] | --border-weak-base=#282828 | ΔE=0.0 | PASS |
| theme.hr_rule | [0,0,0,0] | transparent=transparent | alpha 0 | PASS |
| theme.table_header_bg | [0,0,0,0] | transparent=transparent | alpha 0 | PASS |
| theme.table_rule | [0.157,0.157,0.157,1] | --border-weak-base=#282828 | ΔE=0.0 | PASS |
| theme.user_bg | [0.11,0.11,0.11,1] | --surface-base=#1C1C1C | ΔE=0.0 | PASS |
| theme.card_border | [0.157,0.157,0.157,1] | --border-weak-base=#282828 | ΔE=0.0 | PASS |
| theme.diff_add_bg | [0,0.078,0.004,1] | --surface-diff-add-base=#001401 | ΔE=0.0 | PASS |
| theme.diff_del_bg | [0.141,0.008,0,1] | --surface-diff-delete-base=#240200 | ΔE=0.0 | PASS |
| theme.ask_button_bg | [0.929,0.91,0.894,1] | --button-primary-base=#ede8e4 | ΔE=0.0 | PASS |
| theme.selection(rgb) | [0.008,0.184,0.651,0.45] | --surface-interactive-base=#022fa6 | ΔE=0.0 | PASS |
| wgsl.Normal(default) | [0.929,0.929,0.929,1] | --text-strong=#EDEDED | ΔE=0.0 | PASS |
| wgsl.Quote/ListMarker(8,9) | [0.439,0.439,0.439,1] | --text-weak=#707070 | ΔE=0.0 | PASS |
| wgsl.Link(7) | [0.753,0.831,0.984,1] | --text-interactive-base=#c0d4fb | ΔE=0.0 | PASS |
| wgsl.InlineCode(4) | [0,0.808,0.726,1] | --syntax-string=#00ceb9 | ΔE=0.1 | PASS |
| wgsl.ToolArg(53) | [0.314,0.314,0.314,1] | --text-weaker=#505050 | ΔE=0.0 | PASS |
| wgsl.Reasoning(51) | [0.439,0.439,0.439,1] | --text-weak=#707070 | ΔE=0.0 | PASS |
| wgsl.ToolTitle(52) | [0.929,0.929,0.929,1] | --text-strong=#EDEDED | ΔE=0.0 | PASS |
| wgsl.DiffAdded(56) | [0.769,1,0.753,1] | --text-diff-add-base=#c4ffc0 | ΔE=0.0 | PASS |
| wgsl.DiffRemoved(57) | [0.926,0.184,0.078,1] | --text-diff-delete-base=#ec2f14 | ΔE=0.1 | PASS |
| wgsl.DiffCtx(59) | [0.628,0.628,0.628,1] | --text-base=#A0A0A0 | ΔE=0.1 | PASS |
| wgsl.CodeKeyword(44) | [0.929,0.698,0.945,1] | --syntax-keyword=#edb2f1 | ΔE=0.0 | PASS |
| wgsl.CodeType(45) | [0.988,0.835,0.228,1] | --syntax-type=#fcd53a | ΔE=0.1 | PASS |
| wgsl.CodeString(47) | [0,0.808,0.726,1] | --syntax-string=#00ceb9 | ΔE=0.1 | PASS |
| wgsl.CodeComment(48) | [0.561,0.561,0.561,1] | --syntax-comment=#8f8f8f | ΔE=0.0 | PASS |
| motion.space_turn | 24 | 24 |  | PASS |
| motion.space_part | 12 | 12 |  | PASS |
| boxlayout.CONTENT_MAX | 800 | 800 |  | PASS |
| boxlayout.BUBBLE_RATIO | 0.82 | 0.82 |  | PASS |
| bridge.BASE_FONT_CSS_PX | 14 | 14 |  | PASS |
| bridge.LINE_HEIGHT 倍率 | 1.6 | 1.6 |  | PASS |
