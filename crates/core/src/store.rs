//! store(M3)— 世界状态唯一真相,归一化三表 + 对账(AR4)+ 快照灌入 + session 归属。
//!
//! - `delta` 乐观追加;`message.part.updated` 全量覆盖该 part(AR4:丢字自愈)。
//! - `apply_snapshot` 批量灌历史(catch-up,Phase F):带 sessionID,建立 part/message→session 映射。
//! - **session 归属**:delta 实测不带 sessionID,靠 `partID→messageID→sessionID` 解析
//!   (snapshot/updated 建映射),供 `?session=` 过滤。
//! - 一切按 part_id upsert,首见即记录顺序;幂等(R8/确定性:同序列 → 同状态)。

use std::collections::HashMap;
use std::fmt::Write as _;

use crate::partrender::{PartKind, RenderPart};
use crate::protocol::{Part, SnapshotMessage};

/// 消息角色(0005 / Plan 13 §2):chat 级左右分栏的依据。`"user"` → [`Role::User`](右),其余
/// (assistant/system/…)→ [`Role::Assistant`](左)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Role {
    User,
    #[default]
    Assistant,
}

impl Role {
    /// 协议 role 串 → 角色(`"user"` = User;其余皆 Assistant)。
    pub fn from_proto(s: &str) -> Role {
        if s.eq_ignore_ascii_case("user") {
            Role::User
        } else {
            Role::Assistant
        }
    }
}

/// 非文本 part 的结构化载荷(Plan 22 P1/P3 承载)。文本/推理只用 `PartRow.text`(delta 累积);
/// 其余类型把渲染所需投影存这里 → `display_source` 据此产兜底 markdown(标签 + 内容,丑但完整)。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
enum PartExtra {
    /// 普通 text part。
    #[default]
    Text,
    /// 推理 / 思考区(0006):正文在 `text`。
    Reasoning,
    /// 工具调用:工具名 + 状态 + state 的 JSON dump(input/output/metadata)。
    Tool {
        name: String,
        status: String,
        payload_json: String,
    },
    /// 文件附件。
    File {
        filename: String,
        mime: String,
        url: String,
    },
    /// 上下文压缩通知。
    Compaction,
    /// 合成错误卡(Plan 22 P4 / F4):消息在 `text`,渲染为 `[error]` 标签块。
    Error,
    /// 流内问答块(Plan 27):question/permission 入对话列。`prompt` 在 `text`;
    /// 活跃期展示选项/按钮,应答后原位落定成答案卡(0020 身份不变 → 其余块稳定)。
    Ask {
        kind: AskKind,
        /// question 的候选项(permission 恒空)。
        options: Vec<String>,
        /// `None` = Pending(活跃);`Some` = Answered(落定)。
        answer: Option<AskAnswer>,
        /// DOM 表单实测高回报的补白行数(Plan 27 §3 `set_ask_height`,单向微调一次)。
        pad_lines: u32,
    },
}

/// 问答种类(Plan 27)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AskKind {
    /// 权限请求:两键(允许/拒绝),wasm 原生活跃期(A 路)。
    Permission,
    /// 提问:多选 + 自由输入,DOM overlay 活跃期(B 路)。
    Question,
}

/// 应答载荷(落定内容)。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AskAnswer {
    Permission { allow: bool },
    Question { options: Vec<usize>, text: String },
}

/// 当前 pending ask 的只读投影(host/剧本/e2e 的真相源)。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingAsk {
    pub part_id: String,
    pub kind: AskKind,
    pub prompt: String,
    pub options: Vec<String>,
}

/// part 行是否「上下文噪音工具」(read/glob/grep/list;Plan 28 遗留-1 Explored 分组)。
fn row_is_context_tool(row: &PartRow) -> bool {
    matches!(&row.extra, PartExtra::Tool { name, .. } if crate::partrender::is_context_tool(name))
}

/// 应答 → 紧凑 JSON(兜底展示 + 渲染投影 payload 共用;纯函数确定)。
fn ask_answer_json(a: &AskAnswer) -> String {
    match a {
        AskAnswer::Permission { allow } => format!(r#"{{"allow":{allow}}}"#),
        AskAnswer::Question { options, text } => serde_json::json!({
            "options": options,
            "text": text,
        })
        .to_string(),
    }
}

/// 合成错误卡的固定 part id(F4「恒一张」:同 session 始终复用同一 id → upsert 即替换)。
const ERROR_CARD_ID: &str = "error-card";
/// 合成消息 id(错误卡归属)。
const ERROR_CARD_MSG: &str = "error-card-msg";

/// Thinking 状态行合成 part id(Plan 28 遗留-1:turn 级「等待首包」指示,ERROR_CARD 同模式;
/// 参考 session-turn-thinking:无可见 part 时 spinner+shimmer。settled/首包即清,不留历史)。
const THINKING_ROW_ID: &str = "thinking-row";
const THINKING_ROW_MSG: &str = "thinking-row-msg";

/// 单个 part 的累积状态(Plan 22:文本 + 分类载荷)。
#[derive(Debug, Clone, PartialEq, Eq)]
struct PartRow {
    message_id: String,
    /// Plan 29 FrozenFar:正文已释放(等 resync 重取);apply_* 全量写入时清。
    text_released: bool,
    /// 已知的归属 session(snapshot/updated 带;delta 不带 → 靠 message 映射补)。
    session_id: Option<String>,
    /// 当前文本 = delta 累积(对账后被全量覆盖);text/reasoning 用,非文本 part 为空。
    text: String,
    /// 非文本分类载荷(默认 `Text`)。
    extra: PartExtra,
}

/// 归一化文档表(Plan2 text 子集)。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Store {
    /// part_id 首见顺序(渲染按此纵向堆叠)。
    order: Vec<String>,
    parts: HashMap<String, PartRow>,
    /// messageID → sessionID(snapshot/updated 建立),用于解析 delta 的归属。
    message_session: HashMap<String, String>,
    /// messageID → 角色(snapshot 的 `info.role` 建立;Plan 13 §4.3)。live delta 不带 role →
    /// 未知默认 Assistant(流式多为 assistant;user 消息经 snapshot/resync 校正)。
    message_role: HashMap<String, Role>,
    /// ask 块自增序号(Plan 27:每次 asked 事件一个新块,id 稳定)。
    ask_seq: u32,
}

impl Store {
    pub fn new() -> Self {
        Self::default()
    }

    /// 文本增量追加(AR4 乐观路径)。非 `text` field 忽略。`message_id` 用于 session 解析。
    pub fn apply_delta(&mut self, part_id: &str, message_id: &str, field: &str, delta: &str) {
        if field != "text" {
            return;
        }
        let known = self.message_session.get(message_id).cloned();
        let row = self.ensure(part_id, message_id);
        row.text.push_str(delta);
        if row.session_id.is_none() {
            row.session_id = known;
        }
    }

    /// 全量对账(AR4):以 `part.updated` 为准,覆盖文本 + 分类载荷;若带 sessionID 则建立映射。
    /// Plan 22 P1:承载全分类 part(text/reasoning/tool/file/compaction);`Other`(噪音)忽略。
    pub fn apply_part_updated(&mut self, part: &Part) {
        let (id, message_id, session_id, text, extra) = match part {
            Part::Text {
                id,
                message_id,
                text,
                session_id,
            } => (id, message_id, session_id, text.clone(), PartExtra::Text),
            Part::Reasoning {
                id,
                message_id,
                text,
                session_id,
            } => (
                id,
                message_id,
                session_id,
                text.clone(),
                PartExtra::Reasoning,
            ),
            Part::Tool {
                id,
                message_id,
                session_id,
                tool,
                state,
            } => {
                let status = state
                    .get("status")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("pending")
                    .to_owned();
                let payload_json =
                    serde_json::to_string_pretty(state).unwrap_or_else(|_| state.to_string());
                (
                    id,
                    message_id,
                    session_id,
                    String::new(),
                    PartExtra::Tool {
                        name: tool.clone(),
                        status,
                        payload_json,
                    },
                )
            }
            Part::File {
                id,
                message_id,
                session_id,
                filename,
                mime,
                url,
            } => (
                id,
                message_id,
                session_id,
                String::new(),
                PartExtra::File {
                    filename: filename.clone(),
                    mime: mime.clone(),
                    url: url.clone(),
                },
            ),
            Part::Compaction {
                id,
                message_id,
                session_id,
            } => (
                id,
                message_id,
                session_id,
                String::new(),
                PartExtra::Compaction,
            ),
            Part::Other => return, // 噪音 part(step/snapshot/patch)不承载(0002/AR12)
        };
        if !session_id.is_empty() {
            self.message_session
                .insert(message_id.clone(), session_id.clone());
        }
        let sid = (!session_id.is_empty()).then(|| session_id.clone());
        let row = self.ensure(id, message_id);
        row.text = text;
        row.text_released = false; // Plan 29:resync 全量覆盖 → FrozenFar 释放态自愈
        row.extra = extra;
        if sid.is_some() {
            row.session_id = sid;
        }
    }

    /// 批量灌入快照历史(Phase F,catch-up)。带 sessionID,建立映射。
    pub fn apply_snapshot(&mut self, messages: &[SnapshotMessage]) {
        for msg in messages {
            self.message_session
                .insert(msg.message_id.clone(), msg.session_id.clone());
            self.message_role
                .insert(msg.message_id.clone(), Role::from_proto(&msg.role));
            for tp in &msg.text_parts {
                let row = self.ensure(&tp.part_id, &msg.message_id);
                row.text.clone_from(&tp.text);
                row.session_id = Some(msg.session_id.clone());
            }
        }
    }

    /// 删 part(`message.part.removed`,Plan 22 P1)。从顺序表 + parts 表移除;不存在 = noop(幂等)。
    pub fn apply_part_removed(&mut self, part_id: &str) {
        if self.parts.remove(part_id).is_some() {
            self.order.retain(|id| id != part_id);
        }
    }

    /// 记录 live 消息角色(`message.updated` 的 `info.role`;Plan 13 §4.3 左右分栏的 live 来源)。
    /// 空 id/role 忽略(损坏事件不污染)。session 顺带建映射(若带)。
    pub fn set_message_role(&mut self, message_id: &str, role: &str, session_id: &str) {
        if message_id.is_empty() || role.is_empty() {
            return;
        }
        self.message_role
            .insert(message_id.to_owned(), Role::from_proto(role));
        if !session_id.is_empty() {
            self.message_session
                .insert(message_id.to_owned(), session_id.to_owned());
        }
    }

    /// 某 part 的当前文本(源文本;非文本 part 为空)。
    pub fn part_text(&self, part_id: &str) -> Option<&str> {
        self.parts.get(part_id).map(|r| r.text.as_str())
    }

    /// 是否文本类 part(text;含未知缺省)。非文本(tool/file/...)的 `display_source` 可整体重写
    /// (tool pending→completed),需重置重渲(append-only 流式不适用),app 据此判定。
    #[must_use]
    pub fn part_is_text(&self, part_id: &str) -> bool {
        self.parts
            .get(part_id)
            .is_none_or(|r| matches!(r.extra, PartExtra::Text))
    }

    /// 某 part 的**兜底显示源**(Plan 22 P3):喂渲染管线的 markdown。文本/推理 = 正文(+ 标签);
    /// 非文本 = `**[标签]**` + 内容(JSON/url)。这就是"丑骨架":每 part 标了身份、内容完整。
    /// 复用既有 `parse_markdown_nodes` 全管线(表格/代码块/数学/reveal/虚拟化),不另起渲染路径。
    /// Plan 23 的 specific 漂亮渲染器经 0033 registry 覆盖,不走这里。
    #[must_use]
    /// (Plan 28 遗留-1)part 在「连续 context 工具组」里的角色:
    /// `Some(true)` = 组首(出聚合行),`Some(false)` = 组员(塌空),`None` = 不在组。
    /// 连续 = store.order 相邻 + 同 message。单个 context 工具也算组首(N=1 聚合行)。
    fn context_group_role(&self, part_id: &str) -> Option<bool> {
        let row = self.parts.get(part_id)?;
        if !row_is_context_tool(row) {
            return None;
        }
        let idx = self.order.iter().position(|id| id == part_id)?;
        let lead = idx == 0
            || self
                .parts
                .get(&self.order[idx - 1])
                .is_none_or(|prev| prev.message_id != row.message_id || !row_is_context_tool(prev));
        Some(lead)
    }

    /// (Plan 28)组首起的连续组统计:(reads, searches, 全部 completed?)。
    fn context_group_stats(&self, lead_id: &str) -> (usize, usize, bool) {
        let (mut reads, mut searches, mut done) = (0usize, 0usize, true);
        let Some(start) = self.order.iter().position(|id| id == lead_id) else {
            return (0, 0, true);
        };
        let msg = self.parts[&self.order[start]].message_id.clone();
        for id in &self.order[start..] {
            let Some(row) = self.parts.get(id) else { break };
            if row.message_id != msg || !row_is_context_tool(row) {
                break;
            }
            if let PartExtra::Tool { name, status, .. } = &row.extra {
                match name.as_str() {
                    "glob" | "grep" => searches += 1,
                    _ => reads += 1, // read/list
                }
                if status != "completed" && status != "done" && status != "error" {
                    done = false;
                }
            }
        }
        (reads, searches, done)
    }

    pub fn display_source(&self, part_id: &str) -> Option<String> {
        let row = self.parts.get(part_id)?;
        // Plan 28 遗留-1:context 工具组员 → 空(块塌 0 高,聚合行在组首;参考 groupParts)。
        if self.context_group_role(part_id) == Some(false) {
            return Some(String::new());
        }
        Some(match &row.extra {
            PartExtra::Text => row.text.clone(),
            PartExtra::Reasoning => {
                if row.text.is_empty() {
                    "*[reasoning]*".to_owned()
                } else {
                    format!("*[reasoning]*\n\n{}", row.text)
                }
            }
            PartExtra::Tool {
                name,
                status,
                payload_json,
            } => format!("**[tool:{name} · {status}]**\n\n```json\n{payload_json}\n```"),
            PartExtra::File { filename, url, .. } => {
                format!("**[file:{filename}]**\n\n{url}")
            }
            PartExtra::Compaction => "---\n\n*[上下文已压缩]*".to_owned(),
            PartExtra::Error => {
                if row.text.is_empty() {
                    "**[error]**".to_owned()
                } else {
                    format!("**[error]**\n\n{}", row.text)
                }
            }
            PartExtra::Ask {
                kind,
                options,
                answer,
                ..
            } => {
                // 兜底(未注册 specific 时):标签 + prompt + 选项/答案,丑但完整(0033)。
                let tag = match (kind, answer.is_some()) {
                    (AskKind::Permission, false) => "ask:permission · pending",
                    (AskKind::Permission, true) => "ask:permission · answered",
                    (AskKind::Question, false) => "ask:question · pending",
                    (AskKind::Question, true) => "ask:question · answered",
                };
                let mut out = format!("**[{tag}]**\n\n{}", row.text);
                for (i, o) in options.iter().enumerate() {
                    let _ = write!(out, "\n{}. {o}", i + 1);
                }
                if let Some(a) = answer {
                    let _ = write!(out, "\n\n```json\n{}\n```", ask_answer_json(a));
                } else if let PartExtra::Ask { pad_lines, .. } = &row.extra {
                    // pending 补白(§3 set_ask_height):进显示源 → 字节长度变化触发重排(与
                    // specific 渲染器的 payload pad 同步)。
                    for _ in 0..*pad_lines {
                        out.push('\n');
                    }
                }
                out
            }
        })
    }

    /// 追加一个流内 ask 块(Plan 27:`permission.asked`/`question.asked` → 对话列新块)。
    /// 返回块 part_id(`ask-<seq>`,append-only 稳定,0020 身份)。
    pub fn push_ask(
        &mut self,
        kind: AskKind,
        prompt: &str,
        options: Vec<String>,
        session_id: &str,
    ) -> String {
        self.ask_seq += 1;
        let part_id = format!("ask-{}", self.ask_seq);
        let msg_id = format!("ask-msg-{}", self.ask_seq);
        self.message_role.insert(msg_id.clone(), Role::Assistant);
        if !session_id.is_empty() {
            self.message_session
                .insert(msg_id.clone(), session_id.to_owned());
        }
        let row = self.ensure(&part_id, &msg_id);
        prompt.clone_into(&mut row.text);
        row.extra = PartExtra::Ask {
            kind,
            options,
            answer: None,
            pad_lines: 0,
        };
        if !session_id.is_empty() {
            row.session_id = Some(session_id.to_owned());
        }
        part_id
    }

    /// 当前 pending ask(最新一个未应答的;FSM Blocked 语义下同时至多一个)。
    #[must_use]
    pub fn pending_ask(&self) -> Option<PendingAsk> {
        self.order.iter().rev().find_map(|id| {
            let row = self.parts.get(id)?;
            match &row.extra {
                PartExtra::Ask {
                    kind,
                    options,
                    answer: None,
                    ..
                } => Some(PendingAsk {
                    part_id: id.clone(),
                    kind: *kind,
                    prompt: row.text.clone(),
                    options: options.clone(),
                }),
                _ => None,
            }
        })
    }

    /// 应答当前 pending ask(种类须匹配)→ 原位落定成答案卡(块身份不变,0020)。
    /// **幂等**:无匹配 pending → `None`(重复应答忽略)。返回被应答块的 part_id。
    pub fn answer_pending_ask(&mut self, a: AskAnswer) -> Option<String> {
        let want = match a {
            AskAnswer::Permission { .. } => AskKind::Permission,
            AskAnswer::Question { .. } => AskKind::Question,
        };
        let pending = self.pending_ask()?;
        if pending.kind != want {
            return None;
        }
        if let Some(row) = self.parts.get_mut(&pending.part_id) {
            if let PartExtra::Ask { answer, .. } = &mut row.extra {
                *answer = Some(a);
                return Some(pending.part_id);
            }
        }
        None
    }

    /// DOM 表单实测高回报(Plan 27 §3):给 pending ask 块补 `n` 行占位(单向,一次)。
    pub fn set_ask_pad_lines(&mut self, part_id: &str, n: u32) {
        if let Some(row) = self.parts.get_mut(part_id) {
            if let PartExtra::Ask { pad_lines, .. } = &mut row.extra {
                *pad_lines = n;
            }
        }
    }

    /// Plan 23 渲染投影:把结构化 part 映射成 [`RenderPart`] + [`PartKind`],喂 `RenderRegistry`
    /// 的 specific 漂亮渲染器(0033 契约)。**只投影有 specific 渲染器的种类**(reasoning/tool/
    /// compaction);text/file/error 返回 `None` → 走 Plan 22 的 `display_source` markdown 兜底。
    pub(crate) fn render_part(&self, part_id: &str) -> Option<(PartKind, RenderPart)> {
        let row = self.parts.get(part_id)?;
        // Plan 28 遗留-1:context 工具组 —— 组员无投影(display_source 已空);组首投影成聚合行
        // `tool:explored`(payload = {reads,searches};pending → tool_render shimmer "Explored")。
        match self.context_group_role(part_id) {
            Some(false) => return None,
            Some(true) => {
                let (reads, searches, done) = self.context_group_stats(part_id);
                let status = if done { "completed" } else { "pending" };
                return Some((
                    PartKind::Tool,
                    RenderPart {
                        kind_tag: format!("tool:explored · {status}"),
                        text: String::new(),
                        payload_json: Some(format!(r#"{{"reads":{reads},"searches":{searches}}}"#)),
                    },
                ));
            }
            None => {}
        }
        let proj = match &row.extra {
            PartExtra::Reasoning => (
                PartKind::Reasoning,
                RenderPart {
                    kind_tag: "reasoning".to_owned(),
                    text: row.text.clone(),
                    payload_json: None,
                },
            ),
            PartExtra::Tool {
                name,
                status,
                payload_json,
            } => (
                PartKind::Tool,
                RenderPart {
                    kind_tag: format!("tool:{name} · {status}"),
                    text: String::new(),
                    payload_json: Some(payload_json.clone()),
                },
            ),
            PartExtra::Compaction => (
                PartKind::Compaction,
                RenderPart {
                    kind_tag: "compaction".to_owned(),
                    text: String::new(),
                    payload_json: None,
                },
            ),
            PartExtra::Ask {
                kind,
                options,
                answer,
                pad_lines,
            } => {
                let k = match kind {
                    AskKind::Permission => "permission",
                    AskKind::Question => "question",
                };
                let state = if answer.is_some() {
                    "answered"
                } else {
                    "pending"
                };
                let payload = serde_json::json!({
                    "kind": k,
                    "options": options,
                    "answer": answer.as_ref().map(|a| serde_json::from_str::<serde_json::Value>(&ask_answer_json(a)).unwrap_or(serde_json::Value::Null)),
                    "pad_lines": pad_lines,
                })
                .to_string();
                (
                    PartKind::Ask,
                    RenderPart {
                        kind_tag: format!("ask:{k} · {state}"),
                        text: row.text.clone(),
                        payload_json: Some(payload),
                    },
                )
            }
            PartExtra::Text | PartExtra::File { .. } | PartExtra::Error => return None,
        };
        Some(proj)
    }

    /// Plan 29 V2(FrozenFar):释放某 part 的正文驻留(Store 语义放宽为「可由 server 快照
    /// 重取的缓存」,0029 §5)。`released` 置位 → display_source 空;resync 全量覆盖时自愈。
    pub fn release_text(&mut self, part_id: &str) {
        if let Some(row) = self.parts.get_mut(part_id) {
            row.text = String::new();
            row.text_released = true;
        }
    }

    /// 该 part 正文是否已被 FrozenFar 释放(等待 resync 重取)。
    #[must_use]
    pub fn text_released(&self, part_id: &str) -> bool {
        self.parts.get(part_id).is_some_and(|r| r.text_released)
    }

    /// Thinking 状态行(Plan 28 遗留-1):发送后、首包前的合成指示行。投影成
    /// `tool:thinking · pending` → tool_render 出 "Thinking" 标题 + R4 shimmer,零新管线。
    pub fn upsert_thinking_row(&mut self, session_id: &str) {
        if self.parts.contains_key(THINKING_ROW_ID) {
            return;
        }
        self.message_role
            .insert(THINKING_ROW_MSG.to_owned(), Role::Assistant);
        if !session_id.is_empty() {
            self.message_session
                .insert(THINKING_ROW_MSG.to_owned(), session_id.to_owned());
        }
        let row = self.ensure(THINKING_ROW_ID, THINKING_ROW_MSG);
        row.extra = PartExtra::Tool {
            name: "thinking".to_owned(),
            status: "pending".to_owned(),
            payload_json: String::new(),
        };
        if !session_id.is_empty() {
            row.session_id = Some(session_id.to_owned());
        }
    }

    /// 撤 Thinking 行(首包/终态)。返回是否存在过。
    pub fn clear_thinking_row(&mut self) -> bool {
        if self.parts.remove(THINKING_ROW_ID).is_some() {
            self.order.retain(|id| id != THINKING_ROW_ID);
            return true;
        }
        false
    }

    /// 幂等 upsert 合成错误卡(Plan 22 P4 / F4:**恒一张**)。固定 id → 替换旧卡;附到 `session` 末尾。
    /// `message` 为人读错误文案。返回是否新建(false = 替换既有)。
    pub fn upsert_error_card(&mut self, session_id: &str, message: &str) -> bool {
        let existed = self.parts.contains_key(ERROR_CARD_ID);
        if !existed {
            self.message_role
                .insert(ERROR_CARD_MSG.to_owned(), Role::Assistant);
        }
        if !session_id.is_empty() {
            self.message_session
                .insert(ERROR_CARD_MSG.to_owned(), session_id.to_owned());
        }
        let row = self.ensure(ERROR_CARD_ID, ERROR_CARD_MSG);
        message.clone_into(&mut row.text);
        row.extra = PartExtra::Error;
        if !session_id.is_empty() {
            row.session_id = Some(session_id.to_owned());
        }
        !existed
    }

    /// 清掉合成错误卡(F4:真回复到 / ghost-abort → 清陈旧卡)。无卡 = noop。
    pub fn clear_error_card(&mut self) {
        self.apply_part_removed(ERROR_CARD_ID);
    }

    /// 当前是否有合成错误卡(测试/可观测:F4「恒一张」断言)。
    #[must_use]
    pub fn has_error_card(&self) -> bool {
        self.parts.contains_key(ERROR_CARD_ID)
    }

    /// 某 part 的角色(Plan 13 §4.3):partID → messageID → 角色;未知默认 Assistant。
    pub fn part_role(&self, part_id: &str) -> Role {
        self.parts
            .get(part_id)
            .and_then(|r| self.message_role.get(&r.message_id))
            .copied()
            .unwrap_or_default()
    }

    /// 某 part 的归属 session(直接已知,或经 message 映射解析)。
    pub fn part_session(&self, part_id: &str) -> Option<&str> {
        let row = self.parts.get(part_id)?;
        match &row.session_id {
            Some(s) => Some(s.as_str()),
            None => self
                .message_session
                .get(&row.message_id)
                .map(String::as_str),
        }
    }

    /// part 所属 messageID(Plan 44:tile 模式按 message 反查 tile → 每 tile = 一条消息)。
    pub fn part_message(&self, part_id: &str) -> Option<&str> {
        self.parts.get(part_id).map(|r| r.message_id.as_str())
    }

    /// 按首见顺序遍历 (part_id, text)。
    pub fn parts_in_order(&self) -> impl Iterator<Item = (&str, &str)> {
        self.order
            .iter()
            .filter_map(move |id| self.parts.get(id).map(|r| (id.as_str(), r.text.as_str())))
    }

    /// 真相源文本总量(Σ 各 part 文本字节数;Plan 18 §2.1 `store_chars` 度量)。历史规模代理,
    /// 用 `text.len()`(字节,O(1)/part)→ 每帧累加廉价(ASCII/拉丁文 ≈ 字符数)。
    pub fn char_count(&self) -> usize {
        self.parts.values().map(|r| r.text.len()).sum()
    }

    /// 用于断言/对账的快照(part_id, text),按顺序。
    pub fn snapshot(&self) -> Vec<(String, String)> {
        self.parts_in_order()
            .map(|(id, text)| (id.to_owned(), text.to_owned()))
            .collect()
    }

    fn ensure(&mut self, part_id: &str, message_id: &str) -> &mut PartRow {
        if !self.parts.contains_key(part_id) {
            self.order.push(part_id.to_owned());
            self.parts.insert(
                part_id.to_owned(),
                PartRow {
                    message_id: message_id.to_owned(),
                    session_id: None,
                    text: String::new(),
                    text_released: false,
                    extra: PartExtra::default(),
                },
            );
        }
        self.parts.get_mut(part_id).expect("just inserted above") // reason: 上面已确保存在;非生产 panic 路径
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::TextPartData;

    fn text_part(id: &str, mid: &str, sid: &str, text: &str) -> Part {
        Part::Text {
            id: id.into(),
            message_id: mid.into(),
            text: text.into(),
            session_id: sid.into(),
        }
    }

    #[test]
    fn delta_appends_in_order() {
        let mut s = Store::new();
        s.apply_delta("p1", "m1", "text", "Hel");
        s.apply_delta("p1", "m1", "text", "lo");
        assert_eq!(s.part_text("p1"), Some("Hello"));
    }

    #[test]
    fn non_text_field_ignored() {
        let mut s = Store::new();
        s.apply_delta("p1", "m1", "reasoning", "x");
        assert_eq!(s.part_text("p1"), None);
    }

    #[test]
    fn part_updated_overwrites_lost_delta() {
        // AR4:delta 丢了一截,updated 全量对账后自愈。
        let mut s = Store::new();
        s.apply_delta("p1", "m1", "text", "Hel");
        s.apply_part_updated(&text_part("p1", "m1", "s1", "Hello world"));
        assert_eq!(s.part_text("p1"), Some("Hello world"));
        assert_eq!(s.part_session("p1"), Some("s1"));
    }

    #[test]
    fn order_is_first_seen() {
        let mut s = Store::new();
        s.apply_delta("b", "m", "text", "B");
        s.apply_delta("a", "m", "text", "A");
        s.apply_delta("b", "m", "text", "B2");
        let ids: Vec<_> = s.parts_in_order().map(|(id, _)| id.to_owned()).collect();
        assert_eq!(ids, vec!["b", "a"]);
    }

    #[test]
    fn snapshot_loads_history_with_session() {
        let mut s = Store::new();
        s.apply_snapshot(&[SnapshotMessage {
            session_id: "sX".into(),
            message_id: "m1".into(),
            role: "assistant".into(),
            text_parts: vec![TextPartData {
                part_id: "p1".into(),
                text: "历史回复".into(),
            }],
        }]);
        assert_eq!(s.part_text("p1"), Some("历史回复"));
        assert_eq!(s.part_session("p1"), Some("sX"));
    }

    #[test]
    fn delta_session_resolved_via_message_map() {
        // delta 不带 sessionID,但同 message 已被 snapshot/updated 建立映射 → 可解析。
        let mut s = Store::new();
        s.apply_snapshot(&[SnapshotMessage {
            session_id: "sX".into(),
            message_id: "m1".into(),
            role: "assistant".into(),
            text_parts: vec![],
        }]);
        s.apply_delta("p9", "m1", "text", "live");
        assert_eq!(s.part_session("p9"), Some("sX"));
    }

    fn snap(part: &str, mid: &str, text: &str) -> SnapshotMessage {
        SnapshotMessage {
            session_id: "s".into(),
            message_id: mid.into(),
            role: "a".into(),
            text_parts: vec![TextPartData {
                part_id: part.into(),
                text: text.into(),
            }],
        }
    }

    #[test]
    fn final_state_converges_to_snapshot_under_faults() {
        // Phase J/AR4 总不变量:乱序 + 重复 + 丢失的 delta,经快照对账后收敛到权威态。
        let mut s = Store::new();
        s.apply_delta("p1", "m1", "text", "Hel");
        s.apply_delta("p2", "m2", "text", "wor");
        s.apply_delta("p1", "m1", "text", "Hel"); // 重复
        s.apply_delta("p2", "m2", "text", "ld");
        s.apply_snapshot(&[snap("p1", "m1", "Hello"), snap("p2", "m2", "world")]);
        assert_eq!(s.part_text("p1"), Some("Hello"));
        assert_eq!(s.part_text("p2"), Some("world"));
    }

    #[test]
    fn snapshot_apply_is_idempotent() {
        let msgs = [snap("p1", "m1", "abc")];
        let mut a = Store::new();
        a.apply_snapshot(&msgs);
        let mut b = Store::new();
        b.apply_snapshot(&msgs);
        b.apply_snapshot(&msgs); // 再来一次(重连重放)
        assert_eq!(a.snapshot(), b.snapshot());
    }

    #[test]
    fn part_removed_is_idempotent_and_drops_order() {
        let mut s = Store::new();
        s.apply_delta("a", "m", "text", "A");
        s.apply_delta("b", "m", "text", "B");
        s.apply_part_removed("a");
        s.apply_part_removed("a"); // 再删一次 = noop(幂等)
        s.apply_part_removed("zzz"); // 删不存在 = noop
        assert_eq!(s.part_text("a"), None, "已删 part 取不到");
        let ids: Vec<_> = s.parts_in_order().map(|(id, _)| id.to_owned()).collect();
        assert_eq!(ids, vec!["b"], "顺序表也移除了 a");
    }

    use proptest::prelude::*;

    proptest! {
        // Plan 22 N2:任意 delta/removed 序列 → 三表幂等(同序列两遍同态)+ 顺序仅含存活 part。
        #[test]
        fn store_part_upsert_removed_idempotent(
            ops in prop::collection::vec(
                (0u8..4, prop::sample::select(vec!["p1","p2","p3"]), "[a-c]{0,4}"),
                0..30,
            ),
        ) {
            let run = || {
                let mut s = Store::new();
                for (op, pid, txt) in &ops {
                    match op {
                        0 | 1 => s.apply_delta(pid, "m", "text", txt), // upsert/append
                        2 => s.apply_part_removed(pid),
                        _ => { let _ = s.part_text(pid); } // read(无副作用)
                    }
                }
                s
            };
            let a = run();
            let b = run();
            prop_assert_eq!(a.snapshot(), b.snapshot(), "同序列两遍应同态(确定性)");
            // 顺序表无重复、且每个 id 都能取到文本(无悬挂 order 项)。
            let ids: Vec<String> = a.parts_in_order().map(|(id,_)| id.to_owned()).collect();
            let mut uniq = ids.clone(); uniq.sort(); uniq.dedup();
            prop_assert_eq!(ids.len(), uniq.len(), "order 无重复");
        }

        // T7/AR4 不变量:无论中途追加多少任意 delta,一次全量对账后文本必等于对账值。
        #[test]
        fn reconciliation_is_authoritative(
            deltas in prop::collection::vec("[\\PC]{0,8}", 0..12),
            authoritative in "[\\PC]{0,32}",
        ) {
            let mut s = Store::new();
            for d in &deltas {
                s.apply_delta("p", "m", "text", d);
            }
            s.apply_part_updated(&text_part("p", "m", "s", &authoritative));
            prop_assert_eq!(s.part_text("p"), Some(authoritative.as_str()));
        }
    }
}
