//! partspecific — Plan 23 的 **specific 漂亮渲染器**(消费 0033 契约,覆盖兜底)。
//!
//! 设计定调(plan23 §0):
//! - **数据驱动两级分派**:`PartKind` 走 [`RenderRegistry`];tool 再按 *工具名* 在 [`tool_render`]
//!   内二级分派(bash/read/edit…),加工具 = 改这一处 match,不动 `build_frame` 骨架(0006)。
//! - **CR1 / R8**:每个渲染器是纯 [`RenderFn`](`fn(PartKind,&RenderPart,&RenderCtx)->Vec<StyledSpan>`),
//!   同输入同输出、零平台依赖、native 可测;且**每个都过** [`crate::partrender::assert_renderfn_conforms`]
//!   契约一致性闸(不 panic + 确定 + 非空输入非空输出)—— 保护与 Plan 22 并行不互相破坏。
//! - **SKIP 对齐 opencode**:patch/step part 不渲染;diff 来自 **tool 的 `metadata.filediff`**,不是 patch part。
//!
//! 视觉(卡底面板 / 左条 / diff 行底色 / 徽章色)由 `app.rs` 按 [`StyleRole`] 画装饰;本模块只决定
//! "是什么文字 + 什么角色",输出对齐既有 content→layout 契约(0001)。

#![allow(clippy::trivially_copy_pass_by_ref)] // RenderFn 统一签名按引用收 ctx(同 partrender.rs)

use serde_json::Value;

use crate::partrender::{PartKind, RenderCtx, RenderPart, RenderRegistry};
use infinite_chat_primitives::style::{StyleRole, StyledSpan};

/// Plan 23 起步注册表:在全兜底基础上 `register` 已实现的 specific 渲染器。
/// **加渲染器 = 这里多一行 + 一条 N3 契约测试**(plan23 §0/§4),其余 kind 继续走兜底 → UI 始终完整。
#[must_use]
pub fn default_registry() -> RenderRegistry {
    let mut reg = RenderRegistry::new();
    reg.register(PartKind::Reasoning, reasoning_render); // R1
    reg.register(PartKind::Compaction, compaction_render); // R1
    reg.register(PartKind::Tool, tool_render); // R2(spans 兜底;full 见下)
    reg.register_full(PartKind::Tool, tool_render_full); // Plan 32/0037:diff 装饰声明式首租
    reg.register(PartKind::Ask, ask_render); // Plan 27:流内问答卡(活跃期 + 落定答案卡)
    reg
}

/// Plan 45:tui flavor 的 part 渲染分派表(0033 additive 第二注册表)。R0 结论:opencode TUI 的
/// markdown/diff/table 渲法与 rich 同(@opentui 也富:语法高亮/词级 diff/grid 表)→ **复用 rich 渲染器**
/// (spans/装饰与主题无关,色在 emit 解析);TUI 观感差异走 theme(tui.json)+ 装饰扁平(flavor 门)+ mono 字体
/// + shader 色(flavor 位)。逐部件 TUI 特化形态(如 InlineTool 单行)= U3 收敛项,按需在此分叉。
pub fn tui_registry() -> RenderRegistry {
    default_registry()
}

// ───────────────────────── Plan 27:流内问答卡 ─────────────────────────

/// 流内 ask 卡(Plan 27):question/permission 的活跃期与落定期同一渲染器(按 payload.answer 分)。
/// - **permission pending**:标题 + prompt + 两个 [`StyleRole::AskButton`] 按钮字(`block_decorations`
///   据角色 run 画按钮面板 + 产命中盒,`tap` 路由应答 —— 0032 hit-test 层第一次实弹)。
/// - **question pending**:标题 + prompt + 编号选项 + 输入占位行 + `pad_lines` 补白(DOM 表单锚定在
///   这块矩形上,B 路;选项文字画布上也可读 → 滚动/缩放时表单未跟上仍有着落)。
/// - **answered**:落定答案卡 —— permission =「已允许 ✓ / 已拒绝 ✗」;question = 所选项 chip + 输入原文。
fn ask_render(_kind: PartKind, part: &RenderPart, _ctx: &RenderCtx) -> Vec<StyledSpan> {
    let obj = json_object(part.payload_json.as_deref());
    let kind = obj
        .as_ref()
        .and_then(|m| m.get("kind"))
        .and_then(Value::as_str)
        .unwrap_or("question");
    let answer = obj
        .as_ref()
        .and_then(|m| m.get("answer"))
        .filter(|v| !v.is_null());
    let options: Vec<String> = obj
        .as_ref()
        .and_then(|m| m.get("options"))
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default();
    let prompt = if part.text.is_empty() {
        if kind == "permission" {
            "需要权限确认"
        } else {
            "请回答"
        }
    } else {
        part.text.as_str()
    };

    let mut out: Vec<StyledSpan> = Vec::new();
    match (kind, answer) {
        ("permission", None) => {
            out.push(StyledSpan::new("❓ 权限请求\n", StyleRole::ToolTitle));
            out.push(StyledSpan::new(format!("{prompt}\n\n"), StyleRole::Normal));
            // 按钮字:AskButton 角色 run = 命中盒 + 面板锚(两 run 以普通空格隔开)。
            out.push(StyledSpan::new("  允许  ", StyleRole::AskButton));
            out.push(StyledSpan::new("   ", StyleRole::Normal));
            out.push(StyledSpan::new("  拒绝  ", StyleRole::AskButton));
            out.push(StyledSpan::new("\n", StyleRole::Normal));
        }
        ("permission", Some(a)) => {
            let allow = a.get("allow").and_then(Value::as_bool).unwrap_or(false);
            let title = if allow {
                "✓ 已允许\n"
            } else {
                "✗ 已拒绝\n"
            };
            out.push(StyledSpan::new(title, StyleRole::ToolTitle));
            out.push(StyledSpan::new(format!("{prompt}\n"), StyleRole::Reasoning));
        }
        ("question", None) => {
            out.push(StyledSpan::new("❓ 提问\n", StyleRole::ToolTitle));
            out.push(StyledSpan::new(format!("{prompt}\n"), StyleRole::Normal));
            for (i, o) in options.iter().enumerate() {
                out.push(StyledSpan::new(
                    format!("◻ {}. {o}\n", i + 1),
                    StyleRole::Normal,
                ));
            }
            out.push(StyledSpan::new("▸ 输入回答…\n", StyleRole::Reasoning));
            let pad = obj
                .as_ref()
                .and_then(|m| m.get("pad_lines"))
                .and_then(Value::as_u64)
                .unwrap_or(0);
            for _ in 0..pad {
                out.push(StyledSpan::new("\n", StyleRole::Normal)); // DOM 实测高补白(§3)
            }
        }
        ("question", Some(a)) => {
            out.push(StyledSpan::new("✓ 已回答\n", StyleRole::ToolTitle));
            out.push(StyledSpan::new(format!("{prompt}\n"), StyleRole::Reasoning));
            let picked: Vec<usize> = a
                .get("options")
                .and_then(Value::as_array)
                .map(|xs| {
                    xs.iter()
                        .filter_map(Value::as_u64)
                        .map(|x| x as usize)
                        .collect()
                })
                .unwrap_or_default();
            for i in &picked {
                if let Some(o) = options.get(*i) {
                    out.push(StyledSpan::new(format!(" {o} "), StyleRole::AskButton));
                    out.push(StyledSpan::new("  ", StyleRole::Normal));
                }
            }
            if !picked.is_empty() {
                out.push(StyledSpan::new("\n", StyleRole::Normal));
            }
            let text = a.get("text").and_then(Value::as_str).unwrap_or("");
            if !text.is_empty() {
                out.push(StyledSpan::new(format!("「{text}」\n"), StyleRole::Normal));
            }
        }
        _ => {}
    }
    if out.is_empty() {
        out.push(StyledSpan::new(
            format!("[{}]\n", part.kind_tag),
            StyleRole::Quote,
        ));
    }
    out
}

// ───────────────────────── R1:reasoning + compaction ─────────────────────────

/// 推理 / 思考区(0006):合成 `💭 Thinking` 标题行 + 弱化正文(markdown 结构保留,普通文走
/// [`StyleRole::Reasoning`] 弱化色)。折叠态由 ctx 控制:`folded` → 只出标题(降噪)。
fn reasoning_render(_kind: PartKind, part: &RenderPart, ctx: &RenderCtx) -> Vec<StyledSpan> {
    // Plan 28 R3:参考(message-part.css:280 / NOTES §3)= **裸弱化正文**,无标题、无卡、无折叠链;
    // "Thinking" 状态行是 turn 级合成(R4 shimmer),不属于 part 渲染。
    if ctx.folded || part.text.is_empty() {
        return vec![StyledSpan::new("Thinking\n", StyleRole::Reasoning)];
    }
    (ctx.parse)(&part.text)
        .into_iter()
        .map(dim_reasoning)
        .collect()
}

/// 把普通正文角色降级到 [`StyleRole::Reasoning`](弱化);代码/标题等结构角色保留以不丢语义。
fn dim_reasoning(span: StyledSpan) -> StyledSpan {
    match span.role() {
        StyleRole::Normal | StyleRole::Bold | StyleRole::Italic | StyleRole::BoldItalic => {
            StyledSpan::styled(
                span.text().to_owned(),
                StyleRole::Reasoning,
                span.is_struck(),
            )
        }
        _ => span,
    }
}

/// 上下文压缩通知(0026):标签行 + 整宽分隔线锚点(render 据 [`StyleRole::Rule`] 画细线)。
fn compaction_render(_kind: PartKind, _part: &RenderPart, _ctx: &RenderCtx) -> Vec<StyledSpan> {
    // Plan 28 R3.4:参考 MessageDivider = 分隔线 + 小字弱 label(--text-weaker;NOTES §7)。
    vec![
        StyledSpan::new("Context compacted\n", StyleRole::ToolArg),
        StyledSpan::new(" ", StyleRole::Rule),
    ]
}

// ───────────────────────── R2:通用 tool 卡(+ R3 diff 二级分派)─────────────────────────

/// 工具调用状态(plan23 §1.2;0031 已解码 `ToolState.status`)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ToolStatus {
    Pending,
    Running,
    Completed,
    Error,
    /// 未知 / 缺失:**不隐藏内容**(content-first),按 completed 展示。
    Unknown,
}

impl ToolStatus {
    fn parse(s: &str) -> Self {
        match s.trim().to_ascii_lowercase().as_str() {
            "pending" => Self::Pending,
            "running" => Self::Running,
            "completed" | "done" | "success" => Self::Completed,
            "error" | "failed" => Self::Error,
            _ => Self::Unknown,
        }
    }

    /// pending/running:隐藏 args/output(plan23 §1.2,降噪 + 防半截内容闪)。
    fn hides_body(self) -> bool {
        matches!(self, Self::Pending | Self::Running)
    }
}

/// 通用 tool 卡:`▸ <name>  [status]` 标题行 + 状态分派的 args/output;edit/write/apply_patch 走 diff。
/// 工具名二级分派在此(plan23 §0 数据驱动);未识别工具走"通用 args/output"分支(不丢内容)。
fn tool_render(_kind: PartKind, part: &RenderPart, ctx: &RenderCtx) -> Vec<StyledSpan> {
    // Plan 28 R3:对齐参考(basic-tool.css / message-part.css bash-output / NOTES §4):
    // trigger 行 = 「显示名(strong)+ 副题(weak)」,**无 ▸ 无 [badge] 文本徽章**(状态:
    // pending/running 藏正文 + R4 shimmer;completed 无标记;error 正文即红字风险区)。
    // bash 正文走**围栏代码**(复用 markdown 代码块管线 → inset 面板/等宽 13px,与参考同构)。
    let (name, status) = parse_tool_tag(&part.kind_tag);
    let obj = json_object(part.payload_json.as_deref());
    let subtitle = obj
        .as_ref()
        .and_then(|m| m.get("title"))
        .and_then(Value::as_str)
        .map(str::to_owned)
        .or_else(|| {
            obj.as_ref()
                .and_then(|m| m.get("input"))
                .filter(|v| !v.is_null())
                .map(compact_json)
        })
        .unwrap_or_default();
    // Plan 28 遗留-1:explored 聚合行(参考 "Explored N read, M searches";进行中 → "Exploring")。
    if name == "explored" {
        let reads = obj
            .as_ref()
            .and_then(|m| m.get("reads"))
            .and_then(Value::as_u64)
            .unwrap_or(0);
        let searches = obj
            .as_ref()
            .and_then(|m| m.get("searches"))
            .and_then(Value::as_u64)
            .unwrap_or(0);
        let title = if status.hides_body() {
            "Exploring"
        } else {
            "Explored"
        };
        let mut segs: Vec<String> = Vec::new();
        if reads > 0 {
            segs.push(format!("{reads} read"));
        }
        if searches > 0 {
            segs.push(format!(
                "{searches} search{}",
                if searches == 1 { "" } else { "es" }
            ));
        }
        return vec![
            StyledSpan::new(title, StyleRole::ToolTitle),
            StyledSpan::new(format!("  {}\n", segs.join(", ")), StyleRole::ToolArg),
        ];
    }
    let mut out = vec![StyledSpan::new(
        tool_display_name(name),
        StyleRole::ToolTitle,
    )];
    if subtitle.is_empty() {
        out.push(StyledSpan::new("\n", StyleRole::Normal));
    } else {
        out.push(StyledSpan::new(
            format!("  {subtitle}\n"),
            StyleRole::ToolArg,
        ));
    }

    // diff 工具:full 路径(tool_render_full)处理;此兜底出摘要文本(不产装饰)。
    if is_diff_tool(name) {
        if let Some(diff) = obj.as_ref().and_then(filediff_of) {
            out.extend(render_diff_full(&diff, 0, 0).spans);
            return out;
        }
    }

    if status.hides_body() {
        return out; // pending/running:只留 trigger 行(参考:args/output 隐藏、禁展开)
    }
    // Plan 28 R3.4:上下文噪音工具(read/glob/grep/list)**默认收起** —— 参考把它们折进
    // "Explored" 组(groupParts;完整分组行待 R4+,progress 有记)→ 先按「收起单行」对齐观感。
    if crate::partrender::is_context_tool(name) {
        return out;
    }

    match &obj {
        Some(map) => {
            let body = map
                .get("error")
                .or_else(|| map.get("output"))
                .and_then(Value::as_str)
                .unwrap_or("");
            if name == "bash" {
                // 参考 bash-output:`$ cmd` + 输出,mono 面板。围栏代码 = 复用整条代码块管线。
                let cmd = map
                    .get("input")
                    .and_then(|i| i.get("cmd").or_else(|| i.get("command")))
                    .and_then(Value::as_str)
                    .unwrap_or("");
                let mut pre = String::new();
                if !cmd.is_empty() {
                    use std::fmt::Write as _;
                    let _ = writeln!(pre, "$ {cmd}");
                }
                if !body.is_empty() {
                    if !pre.is_empty() {
                        pre.push('\n');
                    }
                    pre.push_str(body);
                }
                if !pre.is_empty() {
                    out.extend((ctx.parse)(&format!("```\n{pre}\n```")));
                }
            } else {
                // 通用:args(弱化)+ output/error(中性);不丢内容。
                if let Some(input) = map.get("input").filter(|v| !v.is_null()) {
                    out.push(StyledSpan::new(
                        format!("{}\n", compact_json(input)),
                        StyleRole::ToolArg,
                    ));
                }
                if !body.is_empty() {
                    out.push(StyledSpan::new(body.to_owned(), StyleRole::ToolOutput));
                }
            }
        }
        None => {
            if let Some(raw) = &part.payload_json {
                if !raw.is_empty() {
                    out.push(StyledSpan::new(raw.clone(), StyleRole::ToolArg));
                }
            }
        }
    }

    // 正文(若 part 自带 text,如 webfetch 摘要):兜在最后,markdown 渲染。
    if !part.text.is_empty() {
        out.extend((ctx.parse)(&part.text));
    }
    out
}

/// tool 的 full 渲染器(0037 首租):diff 工具产 spans + 装饰指令;其余 = tool_render 包装。
fn tool_render_full(
    kind: PartKind,
    part: &RenderPart,
    ctx: &RenderCtx,
) -> crate::partrender::RenderOutput {
    use crate::partrender::RenderOutput;
    let (name, status) = parse_tool_tag(&part.kind_tag);
    if is_diff_tool(name) && !status.hides_body() {
        let obj = json_object(part.payload_json.as_deref());
        if let Some(diff) = obj.as_ref().and_then(filediff_of) {
            // 前置 trigger 行(与 tool_render 同构)→ 其 grapheme 数 = diff 段 base 偏移。
            let subtitle = obj
                .as_ref()
                .and_then(|m| m.get("title"))
                .and_then(Value::as_str)
                .map(str::to_owned)
                .or_else(|| {
                    obj.as_ref()
                        .and_then(|m| m.get("input"))
                        .filter(|v| !v.is_null())
                        .map(compact_json)
                })
                .unwrap_or_default();
            let mut spans = vec![StyledSpan::new(
                tool_display_name(name),
                StyleRole::ToolTitle,
            )];
            if subtitle.is_empty() {
                spans.push(StyledSpan::new("\n", StyleRole::Normal));
            } else {
                spans.push(StyledSpan::new(
                    format!("  {subtitle}\n"),
                    StyleRole::ToolArg,
                ));
            }
            let base: u32 = spans
                .iter()
                .map(|sp| infinite_chat_primitives::text::graphemes(sp.text()).len() as u32)
                .sum();
            let mut diff_out = render_diff_full(&diff, base, ctx.unfolded);
            spans.append(&mut diff_out.spans);
            return RenderOutput {
                spans,
                decorations: diff_out.decorations,
            };
        }
    }
    RenderOutput::spans_only(tool_render(kind, part, ctx))
}

/// 工具显示名(参考 UI 的人读标签;NOTES §4 / TOOL_SAMPLES 观察):bash→Shell 等,未知首字母大写。
fn tool_display_name(name: &str) -> String {
    match name {
        "bash" => "Shell".to_owned(),
        "edit" => "Edit".to_owned(),
        "write" => "Write".to_owned(),
        "read" => "Read".to_owned(),
        "glob" => "Glob".to_owned(),
        "grep" => "Grep".to_owned(),
        "list" | "ls" => "List".to_owned(),
        "task" => "Task".to_owned(),
        "webfetch" => "Webfetch".to_owned(),
        "websearch" => "Web Search".to_owned(),
        "todowrite" => "To-dos".to_owned(),
        other => {
            let mut c = other.chars();
            match c.next() {
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                None => "Tool".to_owned(),
            }
        }
    }
}

/// 解析身份标签 `tool:<name> · <status>` → (工具名, 状态)。无 `tool:` 前缀/无 `·` 均稳健兜底。
fn parse_tool_tag(tag: &str) -> (&str, ToolStatus) {
    let body = tag.strip_prefix("tool:").unwrap_or(tag);
    let mut it = body.splitn(2, '·');
    let name = it.next().unwrap_or("").trim();
    let name = if name.is_empty() { "tool" } else { name };
    let status = ToolStatus::parse(it.next().unwrap_or(""));
    (name, status)
}

/// edit/write/apply_patch:diff 挂 tool(plan23 §1 / SKIP patch part)。
fn is_diff_tool(name: &str) -> bool {
    matches!(name, "edit" | "write" | "apply_patch" | "patch")
}

/// `metadata.filediff` 或顶层 `filediff` → diff 文本。
fn filediff_of(map: &serde_json::Map<String, Value>) -> Option<String> {
    let v = map
        .get("metadata")
        .and_then(Value::as_object)
        .and_then(|m| m.get("filediff"))
        .or_else(|| map.get("filediff"))?;
    v.as_str().map(str::to_owned)
}

/// JSON 字符串 → 顶层对象(非对象 / 不可解析 → None,调用方原样兜底)。
fn json_object(payload: Option<&str>) -> Option<serde_json::Map<String, Value>> {
    match serde_json::from_str::<Value>(payload?) {
        Ok(Value::Object(m)) => Some(m),
        _ => None,
    }
}

/// 紧凑 JSON(determinism:serde_json 对象按 key 有序;序列化失败回退空串,不 panic)。
fn compact_json(v: &Value) -> String {
    serde_json::to_string(v).unwrap_or_default()
}

// ───────────────────────── R3:diff 解析 + 渲染 ─────────────────────────

/// 一行 diff 的分类(朴素 unified-diff:按首字符)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffKind {
    /// 新增行(`+`,排除 `+++` 文件头)。
    Added,
    /// 删除行(`-`,排除 `---` 文件头)。
    Removed,
    /// hunk 头(`@@`)。
    Hunk,
    /// 上下文 / 文件头 / 其它。
    Context,
}

/// 一行 diff:分类 + 文本(去掉首个 +/- 标记符,文件头/hunk 保留原样)。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffLine {
    pub kind: DiffKind,
    pub text: String,
}

/// `filediff`(unified diff)→ 逐行分类(纯函数,N2)。朴素规则,确定:
/// `@@`→Hunk;`+++`/`---`→Context(文件头);`+`→Added;`-`→Removed;其余→Context。
#[must_use]
/// 一个 diff hunk(Plan 32 D1,Zed 数据模型语义:kind 由构成推导)。
#[derive(Debug, Clone, PartialEq)]
pub struct DiffHunk {
    /// hunk 头解析的起始行号(old, new);无头(裸 diff)从 1 起顺推。
    pub old_start: u32,
    pub new_start: u32,
    /// 行序列(渲染序:删除行在前,Zed「织入行流」语义)。
    pub lines: Vec<DiffLine>,
    /// 词级区间(Modified 且新旧行数相等且 ≤ MAX_WORD_DIFF_LINE_COUNT 才产;Zed 限流同值):
    /// (行下标 in lines, 字符区间 start..end)—— 区间保证行内且互不重叠。
    pub word_ranges: Vec<(usize, (usize, usize))>,
}

/// Zed `MAX_WORD_DIFF_LINE_COUNT` 同值:词级 diff 限流(防 O(n²) 失控)。
pub const MAX_WORD_DIFF_LINE_COUNT: usize = 5;
/// 连续上下文行超过此数 → 折叠(D4;research §2.2)。
pub const FOLD_CONTEXT_OVER: usize = 3;

/// 词级差异(v1:公共词前后缀,中段为变更区):返回 (old_range, new_range) 字符偏移。
/// 完整 LCS 词对齐记遗留(research §1.2);行长 > 512B 跳过(Zed 同思路限流)。
fn word_change_range(old: &str, new: &str) -> Option<((usize, usize), (usize, usize))> {
    if old.len() > 512 || new.len() > 512 || old == new {
        return None;
    }
    let ow: Vec<&str> = old.split_inclusive(char::is_whitespace).collect();
    let nw: Vec<&str> = new.split_inclusive(char::is_whitespace).collect();
    let mut p = 0;
    while p < ow.len() && p < nw.len() && ow[p] == nw[p] {
        p += 1;
    }
    let mut sfx = 0;
    while sfx < ow.len() - p
        && sfx < nw.len() - p
        && ow[ow.len() - 1 - sfx] == nw[nw.len() - 1 - sfx]
    {
        sfx += 1;
    }
    let o0: usize = ow[..p].iter().map(|w| w.len()).sum();
    let o1: usize = old.len() - ow[ow.len() - sfx..].iter().map(|w| w.len()).sum::<usize>();
    let n0: usize = nw[..p].iter().map(|w| w.len()).sum();
    let n1: usize = new.len() - nw[nw.len() - sfx..].iter().map(|w| w.len()).sum::<usize>();
    (o0 < o1 && n0 < n1).then_some(((o0, o1), (n0, n1)))
}

/// diff 源 → hunk 列表(Plan 32 D1,纯函数确定 R8;畸形输入兜底不 panic,AR12)。
/// `@@ -a,b +c,d @@` 头开启新 hunk 并解析行号;头外内容归当前(或首个隐式)hunk。
#[must_use]
pub fn diff_parse_hunks(diff: &str) -> Vec<DiffHunk> {
    let mut hunks: Vec<DiffHunk> = Vec::new();
    let mut cur: Option<DiffHunk> = None;
    for line in diff.lines() {
        if line.starts_with("@@") {
            if let Some(h) = cur.take() {
                hunks.push(h);
            }
            // 解析 `-a[,b] +c[,d]`(容错:任一缺失 → 1)。
            let num = |pat: char| -> u32 {
                line.split(pat)
                    .nth(1)
                    .and_then(|r| {
                        r.split(|c: char| c == ',' || c.is_whitespace())
                            .next()
                            .and_then(|n| n.parse::<u32>().ok())
                    })
                    .unwrap_or(1)
            };
            cur = Some(DiffHunk {
                old_start: num('-'),
                new_start: num('+'),
                lines: vec![DiffLine {
                    kind: DiffKind::Hunk,
                    text: line.to_owned(),
                }],
                word_ranges: Vec::new(),
            });
            continue;
        }
        let (kind, text) = if line.starts_with("+++") || line.starts_with("---") {
            (DiffKind::Context, line)
        } else if let Some(rest) = line.strip_prefix('+') {
            (DiffKind::Added, rest)
        } else if let Some(rest) = line.strip_prefix('-') {
            (DiffKind::Removed, rest)
        } else {
            (DiffKind::Context, line)
        };
        cur.get_or_insert_with(|| DiffHunk {
            old_start: 1,
            new_start: 1,
            lines: Vec::new(),
            word_ranges: Vec::new(),
        })
        .lines
        .push(DiffLine {
            kind,
            text: text.to_owned(),
        });
    }
    if let Some(h) = cur.take() {
        hunks.push(h);
    }
    // 词级:hunk 内连续 -段 紧跟 +段 且行数相等 ≤5 → 逐对产区间。
    for h in &mut hunks {
        let mut i = 0;
        while i < h.lines.len() {
            if h.lines[i].kind != DiffKind::Removed {
                i += 1;
                continue;
            }
            let del0 = i;
            while i < h.lines.len() && h.lines[i].kind == DiffKind::Removed {
                i += 1;
            }
            let add0 = i;
            while i < h.lines.len() && h.lines[i].kind == DiffKind::Added {
                i += 1;
            }
            let (dn, an) = (add0 - del0, i - add0);
            if dn == an && dn > 0 && dn <= MAX_WORD_DIFF_LINE_COUNT {
                for k in 0..dn {
                    if let Some((or, nr)) =
                        word_change_range(&h.lines[del0 + k].text, &h.lines[add0 + k].text)
                    {
                        h.word_ranges.push((del0 + k, or));
                        h.word_ranges.push((add0 + k, nr));
                    }
                }
            }
        }
    }
    hunks
}

pub fn diff_parse_lines(diff: &str) -> Vec<DiffLine> {
    diff.lines()
        .map(|line| {
            let (kind, text) = if line.starts_with("@@") {
                (DiffKind::Hunk, line)
            } else if line.starts_with("+++") || line.starts_with("---") {
                (DiffKind::Context, line)
            } else if let Some(rest) = line.strip_prefix('+') {
                (DiffKind::Added, rest)
            } else if let Some(rest) = line.strip_prefix('-') {
                (DiffKind::Removed, rest)
            } else {
                (DiffKind::Context, line)
            };
            DiffLine {
                kind,
                text: text.to_owned(),
            }
        })
        .collect()
}

/// diff 块渲染(Plan 32 D1/D2,0037 首租):`+a -d` 徽章 + hunk 化行流(双列行号 +
/// 折叠上下文)+ **声明式装饰指令**(行底 Band / hunk Gutter / 词级 CharBg / 折叠 D4)。
/// `base` = 本渲染输出在块 glyph 序列里的起始偏移(前置 spans 的 grapheme 数);
/// `unfolded` = 折叠区展开位图(bit i = 第 i 个折叠区展开;区序全局递增,确定性 R8)。
fn render_diff_full(diff: &str, base: u32, unfolded: u64) -> crate::partrender::RenderOutput {
    use crate::partrender::{DecorKind, DecorSlot, DecorationOp, RenderOutput};
    let hunks = diff_parse_hunks(diff);
    let added: usize = hunks
        .iter()
        .map(|h| h.lines.iter().filter(|l| l.kind == DiffKind::Added).count())
        .sum();
    let removed: usize = hunks
        .iter()
        .map(|h| {
            h.lines
                .iter()
                .filter(|l| l.kind == DiffKind::Removed)
                .count()
        })
        .sum();

    let mut out = RenderOutput::default();
    let mut cursor: u32 = base; // display glyph 计数(装饰锚定,0037)
    let mut push = |out: &mut RenderOutput, text: String, role: StyleRole| -> (u32, u32) {
        let n = infinite_chat_primitives::text::graphemes(&text).len() as u32;
        out.spans.push(StyledSpan::new(text, role));
        let span = (cursor, cursor + n);
        cursor += n;
        span
    };
    push(
        &mut out,
        format!("+{added} -{removed}\n"),
        StyleRole::ToolBadge,
    );

    let mut fold_no: u32 = 0; // 折叠区全局序号(D4;跨 hunk 递增,确定性)
    for (hi, h) in hunks.iter().enumerate() {
        let group = hi as u32;
        let (mut old_no, mut new_no) = (h.old_start, h.new_start);
        // hunk gutter 槽位:构成推导(Zed 语义:纯增/纯删/混合)。
        let has_add = h.lines.iter().any(|l| l.kind == DiffKind::Added);
        let has_del = h.lines.iter().any(|l| l.kind == DiffKind::Removed);
        let gutter_slot = match (has_add, has_del) {
            (true, false) => DecorSlot::DiffAddGutter,
            (false, true) => DecorSlot::DiffDelGutter,
            _ => DecorSlot::DiffModGutter,
        };
        let mut gutter_span: Option<(u32, u32)> = None;
        let mut li = 0;
        while li < h.lines.len() {
            let l = &h.lines[li];
            // 折叠:连续 Context(非 Hunk 头)> FOLD_CONTEXT_OVER → 一行「⋯ n unchanged lines」。
            if l.kind == DiffKind::Context && !l.text.starts_with("@@") {
                let run0 = li;
                while li < h.lines.len() && h.lines[li].kind == DiffKind::Context {
                    li += 1;
                }
                let n = li - run0;
                if n > FOLD_CONTEXT_OVER {
                    let region = fold_no;
                    fold_no += 1;
                    if region < 64 && unfolded & (1u64 << region) != 0 {
                        // 展开:照常逐行 + FoldRegion 指令(app 施加 scale 展开包络,D4)。
                        let mut span: Option<(u32, u32)> = None;
                        for k in run0..li {
                            let c = &h.lines[k];
                            let s0 = push(
                                &mut out,
                                format!("{old_no:>4} {new_no:>4} "),
                                StyleRole::CodeLineNum,
                            );
                            let s1 = push(&mut out, format!(" {}\n", c.text), StyleRole::DiffCtx);
                            span = Some(match span {
                                Some((a, _)) => (a, s1.1),
                                None => (s0.0, s1.1),
                            });
                            old_no += 1;
                            new_no += 1;
                        }
                        if let Some(sp) = span {
                            out.decorations.push(DecorationOp {
                                kind: DecorKind::FoldRegion,
                                span: sp,
                                slot: DecorSlot::DiffModGutter,
                                group: region,
                            });
                        }
                    } else {
                        // 折叠(默认):汇总行 + FoldSummary 指令(gutter 强圆角 marker +
                        // tap 命中盒;slot = hidden 语义色,opencode --surface-diff-hidden-stronger)。
                        let sp = push(
                            &mut out,
                            format!("      \u{22EF} {n} unchanged lines\n"),
                            StyleRole::DiffCtx,
                        );
                        out.decorations.push(DecorationOp {
                            kind: DecorKind::FoldSummary,
                            span: sp,
                            slot: DecorSlot::DiffModGutter,
                            group: region,
                        });
                        old_no += n as u32;
                        new_no += n as u32;
                    }
                    continue;
                }
                // 短上下文:照常逐行(回退 li 重放)。
                for k in run0..li {
                    let c = &h.lines[k];
                    push(
                        &mut out,
                        format!("{old_no:>4} {new_no:>4} "),
                        StyleRole::CodeLineNum,
                    );
                    push(&mut out, format!(" {}\n", c.text), StyleRole::DiffCtx);
                    old_no += 1;
                    new_no += 1;
                }
                continue;
            }
            match l.kind {
                DiffKind::Hunk => {
                    push(&mut out, format!("{}\n", l.text), StyleRole::DiffCtx);
                }
                DiffKind::Added | DiffKind::Removed => {
                    let is_add = l.kind == DiffKind::Added;
                    let nums = if is_add {
                        format!("     {new_no:>4} ")
                    } else {
                        format!("{old_no:>4}      ")
                    };
                    let ns = push(&mut out, nums, StyleRole::CodeLineNum);
                    let mark = if is_add { "+" } else { "-" };
                    let role = if is_add {
                        StyleRole::DiffAdded
                    } else {
                        StyleRole::DiffRemoved
                    };
                    let text_span = push(&mut out, format!("{mark}{}\n", l.text), role);
                    // 行底 Band:整行(含行号列;整宽矩形,锚定用于行定位)。
                    out.decorations.push(DecorationOp {
                        kind: DecorKind::Band,
                        span: (ns.0, text_span.1),
                        slot: if is_add {
                            DecorSlot::DiffAddBand
                        } else {
                            DecorSlot::DiffDelBand
                        },
                        group,
                    });
                    gutter_span = Some(match gutter_span {
                        Some((s0, _)) => (s0, text_span.1),
                        None => (ns.0, text_span.1),
                    });
                    // 词级 CharBg:word_ranges 里属本行的区间(字节 → grapheme 偏移;+1 跳 mark)。
                    for &(wl, (b0, b1)) in &h.word_ranges {
                        if wl == li {
                            let g0 = infinite_chat_primitives::text::graphemes(&l.text[..b0]).len()
                                as u32;
                            let g1 = infinite_chat_primitives::text::graphemes(&l.text[..b1]).len()
                                as u32;
                            out.decorations.push(DecorationOp {
                                kind: DecorKind::CharBg,
                                span: (text_span.0 + 1 + g0, text_span.0 + 1 + g1),
                                slot: if is_add {
                                    DecorSlot::DiffAddWord
                                } else {
                                    DecorSlot::DiffDelWord
                                },
                                group,
                            });
                        }
                    }
                    if is_add {
                        new_no += 1;
                    } else {
                        old_no += 1;
                    }
                }
                DiffKind::Context => unreachable!("上下文行已在 run 分支消费"),
            }
            li += 1;
        }
        if let Some(span) = gutter_span {
            out.decorations.push(DecorationOp {
                kind: DecorKind::Gutter,
                span,
                slot: gutter_slot,
                group,
            });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    /// 0039:测试 ctx = 注真 parse(快照/断言语义与生产一致;搬入 components 后改经 dev-dep core)。
    fn test_ctx() -> RenderCtx {
        RenderCtx {
            parse: infinite_chat_core::parse_markdown,
            ..RenderCtx::default()
        }
    }

    use super::{
        ask_render, compaction_render, default_registry, diff_parse_hunks, diff_parse_lines,
        reasoning_render, render_diff_full, tool_render, DiffKind,
    };
    use crate::partrender::{assert_renderfn_conforms, PartKind, RenderCtx, RenderPart};
    use infinite_chat_primitives::style::{StyleRole, StyledSpan};

    fn part(tag: &str, text: &str, json: Option<&str>) -> RenderPart {
        RenderPart {
            kind_tag: tag.to_owned(),
            text: text.to_owned(),
            payload_json: json.map(str::to_owned),
        }
    }

    fn joined(spans: &[StyledSpan]) -> String {
        spans.iter().map(StyledSpan::text).collect()
    }

    fn has_role(spans: &[StyledSpan], role: StyleRole) -> bool {
        spans.iter().any(|s| s.role() == role)
    }

    /// 渲染输出转可读快照串(N4):每行 `角色 | 文本`(\n 转义),稳定可 diff。
    fn dump(spans: &[StyledSpan]) -> String {
        spans
            .iter()
            .map(|s| format!("{:?} | {}", s.role(), s.text().replace('\n', "\\n")))
            .collect::<Vec<_>>()
            .join("\n")
    }

    // ── N3 契约一致性闸:每个 specific 渲染器必过(plan23 §3.7 N3 / 0033 §3)。
    #[test]
    fn reasoning_conforms() {
        assert_renderfn_conforms(reasoning_render);
    }
    #[test]
    fn compaction_conforms() {
        assert_renderfn_conforms(compaction_render);
    }
    #[test]
    fn tool_conforms() {
        assert_renderfn_conforms(tool_render);
        assert_renderfn_conforms(ask_render); // Plan 27
    }

    // ── R1
    #[test]
    fn reasoning_synthesizes_thinking_header_and_dims_body() {
        let p = part("reasoning", "先看 **要点** 再决定", None);
        let spans = reasoning_render(PartKind::Reasoning, &p, &test_ctx());
        let s = joined(&spans);
        // Plan 28 R3:参考 reasoning = 裸弱化正文,无合成标题。
        assert!(!s.contains("Thinking"), "不应再合成标题: {s}");
        assert!(s.contains("先看"), "正文丢了: {s}");
        assert!(has_role(&spans, StyleRole::Reasoning), "正文未弱化");
    }

    #[test]
    fn reasoning_folded_keeps_only_header() {
        let p = part("reasoning", "一大段思考过程", None);
        let ctx = RenderCtx {
            width: 0.0,
            folded: true,
            unfolded: 0,
            parse: infinite_chat_core::parse_markdown, // 测试注真 parse(0039)
        };
        let spans = reasoning_render(PartKind::Reasoning, &p, &ctx);
        assert_eq!(spans.len(), 1, "折叠态应只留一行 Thinking 占位");
        assert!(!joined(&spans).contains("一大段"), "折叠态不应出正文");
    }

    #[test]
    fn compaction_has_label_and_rule() {
        let spans = compaction_render(
            PartKind::Compaction,
            &part("compaction", "", None),
            &test_ctx(),
        );
        assert!(joined(&spans).contains("compacted"), "缺压缩标签");
        assert!(has_role(&spans, StyleRole::Rule), "缺分隔线锚点");
    }

    // ── R2
    #[test]
    fn tool_title_badge_and_output() {
        let p = part(
            "tool:bash · completed",
            "",
            Some(r#"{"input":{"cmd":"ls"},"output":"a.txt\nb.txt"}"#),
        );
        let spans = tool_render(PartKind::Tool, &p, &test_ctx());
        let s = joined(&spans);
        // Plan 28 R3:显示名 Shell + `$ cmd` 围栏代码正文(参考 bash-output)。
        assert!(s.contains("Shell"), "缺显示名: {s}");
        assert!(!s.contains("[done]"), "文本徽章应退役: {s}");
        assert!(s.contains("$ ls"), "缺 $ cmd 行: {s}");
        assert!(s.contains("a.txt"), "缺 output: {s}");
        assert!(has_role(&spans, StyleRole::ToolTitle));
    }

    #[test]
    fn tool_running_hides_body() {
        let p = part(
            "tool:bash · running",
            "",
            Some(r#"{"input":{"cmd":"ls"},"output":"x"}"#),
        );
        let spans = tool_render(PartKind::Tool, &p, &test_ctx());
        let s = joined(&spans);
        // Plan 28 R3:文本徽章退役;running 只留 trigger 行(标题+副题),藏正文。
        assert!(s.contains("Shell"), "缺显示名: {s}");
        assert!(!s.contains("[running]"), "文本徽章应退役: {s}");
        assert!(!s.contains("\"output\""), "运行中不应露 output: {s}");
    }

    #[test]
    fn tool_error_shows_error_text() {
        let p = part("tool:bash · error", "", Some(r#"{"error":"boom"}"#));
        let spans = tool_render(PartKind::Tool, &p, &test_ctx());
        let s = joined(&spans);
        assert!(s.contains("boom"), "错误未显: {s}");
    }

    #[test]
    fn tool_unparseable_json_keeps_content() {
        // 非对象 JSON → 原样兜底,不丢内容。
        let p = part("tool:custom · done", "", Some("[1,2,3]"));
        let spans = tool_render(PartKind::Tool, &p, &test_ctx());
        assert!(joined(&spans).contains("[1,2,3]"), "原始内容丢了");
    }

    // ── Plan 32 D1/D2(0037 首租)
    #[test]
    fn hunk_parse_snapshot_mixed() {
        // 混合 diff:hunk 头行号 / 增删改 / 折叠长上下文 —— 快照锁定(回归即红)。
        let d = "@@ -10,4 +10,5 @@\n ctx a\n-old line\n+new line\n ctx b\n ctx c\n ctx d\n ctx e\n ctx f\n@@ -30,1 +31,2 @@\n+added only\n";
        insta::assert_debug_snapshot!("diff_hunks_mixed", diff_parse_hunks(d));
    }

    #[test]
    fn word_ranges_only_for_equal_small_hunks_and_in_line() {
        // 词级限流(Zed MAX=5 同值)+ 不变式:区间在行内、互不重叠。
        let d = "@@ -1,1 +1,1 @@\n-let x = old_value;\n+let x = new_value;\n";
        let hs = diff_parse_hunks(d);
        let h = &hs[0];
        assert!(!h.word_ranges.is_empty(), "等行数小 hunk 应产词级区间");
        for &(li, (b0, b1)) in &h.word_ranges {
            assert!(b0 < b1 && b1 <= h.lines[li].text.len(), "区间在行内");
        }
        // 行数不等 → 不产。
        let d2 = "@@ -1,2 +1,1 @@\n-a\n-b\n+c\n";
        assert!(diff_parse_hunks(d2)[0].word_ranges.is_empty());
        // 超限(6 对)→ 不产。
        let many: String = (0..6)
            .map(|i| format!("-o{i}\n"))
            .chain((0..6).map(|i| format!("+n{i}\n")))
            .collect();
        assert!(diff_parse_hunks(&format!("@@ -1,6 +1,6 @@\n{many}"))[0]
            .word_ranges
            .is_empty());
    }

    #[test]
    fn render_diff_full_emits_three_layer_decorations() {
        // D2:行底 Band(每 ± 行)+ 每 hunk 一条 Gutter + 词级 CharBg;span 单调且在输出内。
        use crate::partrender::DecorKind;
        let d = "@@ -1,1 +1,1 @@\n-let x = 1;\n+let x = 2;\n";
        let out = render_diff_full(d, 0, 0);
        let total: u32 = out
            .spans
            .iter()
            .map(|sp| infinite_chat_primitives::text::graphemes(sp.text()).len() as u32)
            .sum();
        let bands = out
            .decorations
            .iter()
            .filter(|o| o.kind == DecorKind::Band)
            .count();
        let gutters = out
            .decorations
            .iter()
            .filter(|o| o.kind == DecorKind::Gutter)
            .count();
        let words = out
            .decorations
            .iter()
            .filter(|o| o.kind == DecorKind::CharBg)
            .count();
        assert_eq!(bands, 2, "两条 ± 行底");
        assert_eq!(gutters, 1, "每 hunk 一条 gutter");
        assert_eq!(words, 2, "词级两条(新旧各一)");
        for op in &out.decorations {
            assert!(
                op.span.0 < op.span.1 && op.span.1 <= total,
                "span 有界: {op:?}"
            );
        }
        // 确定性(R8):双跑逐字节一致。
        assert_eq!(out, render_diff_full(d, 0, 0));
    }

    #[test]
    fn long_context_folds_into_summary_row() {
        // D4 数据层:连续 Context > 3 → 「⋯ n unchanged lines」;行号跨折叠正确推进。
        let mut ctx = String::new();
        for i in 0..6 {
            use std::fmt::Write as _;
            let _ = writeln!(ctx, " c{i}");
        }
        let d = format!("@@ -1,8 +1,8 @@\n{ctx}-x\n+y\n");
        let out = render_diff_full(&d, 0, 0);
        let text: String = out.spans.iter().map(StyledSpan::text).collect();
        assert!(text.contains("6 unchanged lines"), "{text}");
        assert!(
            text.contains("   7      -x") && text.contains("        7 +y"),
            "折叠后新旧行号均应从 7 续: {text}"
        );
        // 折叠汇总行携 FoldSummary 指令(gutter marker + tap 命中,D4)。
        assert!(
            out.decorations
                .iter()
                .any(|d| d.kind == crate::partrender::DecorKind::FoldSummary && d.group == 0),
            "FoldSummary 指令在: {:?}",
            out.decorations
        );
    }

    #[test]
    fn unfolded_region_emits_rows_and_fold_region_op() {
        // D4:展开位图 bit0 置位 → 汇总行消失、上下文逐行(双列行号)、FoldRegion 指令圈住区间。
        let mut ctx = String::new();
        for i in 0..6 {
            use std::fmt::Write as _;
            let _ = writeln!(ctx, " c{i}");
        }
        let d = format!("@@ -1,8 +1,8 @@\n{ctx}-x\n+y\n");
        let out = render_diff_full(&d, 0, 1);
        let text: String = out.spans.iter().map(StyledSpan::text).collect();
        assert!(!text.contains("unchanged lines"), "展开后无汇总行: {text}");
        assert!(
            text.contains("   1    1   c0") && text.contains("   6    6   c5"),
            "上下文行带双列行号: {text}"
        );
        let region = out
            .decorations
            .iter()
            .find(|d| d.kind == crate::partrender::DecorKind::FoldRegion)
            .expect("FoldRegion 指令");
        assert_eq!(region.group, 0);
        assert!(region.span.0 < region.span.1);
        // 确定性(R8):同输入同输出。
        assert_eq!(out, render_diff_full(&d, 0, 1));
    }

    proptest::proptest! {
        #[test]
        fn hunk_parse_never_panics(input in proptest::string::string_regex("[@ +\\-a-z0-9,\\n]{0,200}").expect("regex")) {
            let _ = diff_parse_hunks(&input); // AR12:畸形输入不 panic
        }
    }

    // ── R3
    #[test]
    fn diff_parse_classifies_lines() {
        let d = "@@ -1,2 +1,2 @@\n ctx\n-old\n+new\n+++ b/f\n";
        let lines = diff_parse_lines(d);
        let kinds: Vec<DiffKind> = lines.iter().map(|l| l.kind).collect();
        assert_eq!(
            kinds,
            vec![
                DiffKind::Hunk,
                DiffKind::Context,
                DiffKind::Removed,
                DiffKind::Added,
                DiffKind::Context, // +++ 文件头不算新增
            ]
        );
        assert_eq!(lines[2].text, "old"); // 去掉 - 标记
        assert_eq!(lines[3].text, "new"); // 去掉 + 标记
    }

    #[test]
    fn tool_diff_renders_added_removed_and_summary() {
        let p = part(
            "tool:edit · completed",
            "",
            Some(r#"{"metadata":{"filediff":"@@ -1 +1 @@\n-old\n+new\n"}}"#),
        );
        let spans = tool_render(PartKind::Tool, &p, &test_ctx());
        let s = joined(&spans);
        assert!(s.contains("+1 -1"), "缺变更摘要: {s}");
        assert!(has_role(&spans, StyleRole::DiffAdded), "缺新增行角色");
        assert!(has_role(&spans, StyleRole::DiffRemoved), "缺删除行角色");
    }

    // R4(回合分组三桶 + context 折叠)由 Plan 22 的 `partrender::group_message_parts`(Bucket)
    // 提供并测试,本模块不再重复实现。

    // ── N6 覆盖:specific 注册 + 输出 ≠ 兜底
    #[test]
    fn registry_covers_and_differs_from_fallback() {
        let reg = default_registry();
        for kind in [PartKind::Reasoning, PartKind::Compaction, PartKind::Tool] {
            assert!(reg.has_specific(kind), "{kind:?} 未注册");
        }
        // specific 输出 ≠ 兜底(以 reasoning 为例:合成 Thinking 标题,非 "[reasoning]")。
        let p = part("reasoning", "x", None);
        let specific = joined(&reg.render(PartKind::Reasoning, &p, &test_ctx()));
        let fallback = joined(&crate::partrender::fallback_render(
            PartKind::Reasoning,
            &p,
            &test_ctx(),
        ));
        assert_ne!(specific, fallback, "specific 应区别于兜底");
        assert!(!specific.contains("[reasoning]"), "specific 不应有兜底标签");
    }

    // ── N4:各 kind 渲染输出快照(StyledSpan 角色+文本),随 status/折叠态确定(plan23 §3.7 N4)。
    #[test]
    fn render_snapshots() {
        let ctx = test_ctx();
        insta::assert_snapshot!(
            "reasoning",
            dump(&reasoning_render(
                PartKind::Reasoning,
                &part("reasoning", "先 `读码` 再 **决定**", None),
                &ctx
            ))
        );
        insta::assert_snapshot!(
            "compaction",
            dump(&compaction_render(
                PartKind::Compaction,
                &part("compaction", "", None),
                &ctx
            ))
        );
        insta::assert_snapshot!(
            "tool_bash_done",
            dump(&tool_render(
                PartKind::Tool,
                &part(
                    "tool:bash · completed",
                    "",
                    Some(r#"{"input":{"cmd":"ls -la"},"output":"a.txt\nb.txt"}"#)
                ),
                &ctx
            ))
        );
        insta::assert_snapshot!(
            "tool_running",
            dump(&tool_render(
                PartKind::Tool,
                &part(
                    "tool:grep · running",
                    "",
                    Some(r#"{"input":{"pattern":"foo"}}"#)
                ),
                &ctx
            ))
        );
        insta::assert_snapshot!(
            "tool_edit_diff",
            dump(&tool_render(
                PartKind::Tool,
                &part(
                    "tool:edit · completed",
                    "",
                    Some(r#"{"metadata":{"filediff":"@@ -1,2 +1,2 @@\n ctx\n-old\n+new\n"}}"#)
                ),
                &ctx
            ))
        );
    }

    use proptest::prelude::*;

    proptest! {
        // N2:diff 解析确定 + 增删计数 = 朴素 char 重数(R8)。
        #[test]
        fn diff_parse_deterministic_and_counts(diff in "[+\\- @a-z\\n]{0,200}") {
            let a = diff_parse_lines(&diff);
            let b = diff_parse_lines(&diff);
            prop_assert_eq!(&a, &b);
            // 朴素重数:行首 '+' 且非 "+++" / '-' 且非 "---" / 非 "@@"。
            let naive_add = diff.lines().filter(|l|
                l.starts_with('+') && !l.starts_with("+++") && !l.starts_with("@@")).count();
            let parsed_add = a.iter().filter(|l| l.kind == DiffKind::Added).count();
            prop_assert_eq!(naive_add, parsed_add);
        }
    }
}
