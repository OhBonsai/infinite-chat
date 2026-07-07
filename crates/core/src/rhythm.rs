//! rhythm(M5/M6 / Plan 25 M1)— 分层节奏函数:把 [`RevealScheduler`](crate::RevealScheduler)
//! 的匀速 cps 升级为 design §3.2 的「朗读」节奏(reveal-rhythm 七层的参数化):
//!
//! ```text
//! cost(unit) = Δ_base × lengthen × accel_scale × accent_slow   // ①④⑤ tempo/末延长/重音自身
//!            + rest(boundary) × rest_knob × scale              // ③ 边界停顿(大杠杆)
//!            + accent_pause × scale                            // ⑤ 重音拍前停
//!            + ε(seeded)                                       // ⑥ 微定时(1/f 近似)
//! ```
//!
//! - **单位**:拉丁词一拍(词首计 Δ,词内 glyph 0 成本 = 整词一帧上屏)、CJK 单字一拍、
//!   标点/空白 0 成本(吸附前一单元;停顿计给**下一**单元)。
//! - **额外项等比 tempo 缩放**(`scale = tempo/180`):表值 = 默认 tempo(180ms)下的绝对毫秒
//!   (design §3.2 边界停顿表)。tempo→0(如 e2e `set_reveal_cps(1e9)`)⇒ 全项→0 ⇒ 即时揭示,
//!   既有测试/重放不回归。
//! - **确定性**(R8/R9):微定时 = 以(块 seed, glyph 下标)哈希的两倍频噪声(1/f 白噪近似,
//!   见 plan25 遗留),无墙钟无全局态;同输入同 seed → 成本表逐字节相等。
//! - CR1:纯函数,native 可测;边界来自标点 + [0020 节点树](crate::nodes)。

use crate::nodes::{NodeKind, NodeTree};
use serde::Deserialize;

/// 节奏参数(六旋钮,design §3.2;预设见 [`RhythmParams::preset`])。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RhythmParams {
    /// ① tempo:每单元基准间隔 ms(词/CJK 字)。`INFINITY` = 跟内容到达(不限速,旧默认)。
    pub tempo_ms: f32,
    /// ③ 边界停顿倍率(0 = 关;1 = design 表值)。
    pub rest: f32,
    /// ④ 末延长:边界前最后一单元 × 此值(1 = 关;默认 1.4)。
    pub final_lengthening: f32,
    /// ⑤ 重音强度(0 = 关):强调/标题单元前 +100ms×此值,自身 ×(1+0.2×此值)。
    pub accent: f32,
    /// ⑥ 微定时幅度(0..1):ε = 噪声 × min(0.2Δ, 50ms×scale) × 此值。
    pub microtiming: f32,
    /// ⑦ 句内加速(0..1):句首→句尾 Δ 渐缩(至 -35%×此值)。
    pub accelerando: f32,
    /// 单元粒度:true = 拉丁词一拍(design「拍点在词不在字」,reader/flow);
    /// false = 逐 glyph 一拍(typewriter = 旧匀速逐字,回归等价)。
    pub word_unit: bool,
}

impl Default for RhythmParams {
    /// 默认 = `typewriter`(旧匀速行为等价:跟到达,零节奏层)——Engine 级不回归(重放/native);
    /// 产品层默认档(reader)由 web harness 启动时 `set_reveal_preset` 切(plan25 M1 决策)。
    fn default() -> Self {
        Self::preset(RhythmPreset::Typewriter)
    }
}

/// 预设三档(design §3.2;一键切,兼当回归对拍基线)。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RhythmPreset {
    /// 现状等价:匀速(tempo 由 `set_reveal_cps` 决定,默认跟到达),零节奏层。
    Typewriter,
    /// 「朗读」感(设计默认档):句读呼吸、末延长、重音、微定时。
    Reader,
    /// 「rap flow」:更快、重音更强、微定时满幅、句内加速。
    Flow,
}

impl RhythmPreset {
    /// 名字 ↔ 预设(wasm `set_reveal_preset`);未知名 → None。
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "typewriter" => Some(RhythmPreset::Typewriter),
            "reader" => Some(RhythmPreset::Reader),
            "flow" => Some(RhythmPreset::Flow),
            _ => None,
        }
    }
}

impl RhythmParams {
    /// 预设参数表(design §3.2)。
    pub fn preset(p: RhythmPreset) -> Self {
        match p {
            RhythmPreset::Typewriter => Self {
                tempo_ms: f32::INFINITY,
                rest: 0.0,
                final_lengthening: 1.0,
                accent: 0.0,
                microtiming: 0.0,
                accelerando: 0.0,
                word_unit: false,
            },
            RhythmPreset::Reader => Self {
                tempo_ms: 180.0,
                rest: 1.0,
                final_lengthening: 1.4,
                accent: 1.0,
                microtiming: 1.0,
                accelerando: 0.0,
                word_unit: true,
            },
            RhythmPreset::Flow => Self {
                tempo_ms: 120.0,
                rest: 0.6,
                final_lengthening: 1.2,
                accent: 1.5,
                microtiming: 1.0,
                accelerando: 1.0,
                word_unit: true,
            },
        }
    }
}

/// 六旋钮**局部覆盖**(wasm `set_rhythm`,同 Theme/MotionTokens 的缺字段友好模式,但基底是
/// **当前值**而非默认——旋钮逐个拧,不互相回弹)。字段名即 JSON 键。
#[derive(Debug, Default, Clone, Deserialize)]
#[serde(default)]
pub struct RhythmOverrides {
    pub tempo_ms: Option<f32>,
    pub rest: Option<f32>,
    pub final_lengthening: Option<f32>,
    pub accent: Option<f32>,
    pub microtiming: Option<f32>,
    pub accelerando: Option<f32>,
    pub word_unit: Option<bool>,
}

impl RhythmOverrides {
    /// 从 JSON 解析(未知/缺字段忽略)。
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// 覆盖到 `base`(仅出现的键)。
    pub fn apply(&self, base: &mut RhythmParams) {
        if let Some(v) = self.tempo_ms {
            base.tempo_ms = if v > 0.0 && v.is_finite() {
                v
            } else {
                f32::INFINITY
            };
        }
        if let Some(v) = self.rest {
            base.rest = v.max(0.0);
        }
        if let Some(v) = self.final_lengthening {
            base.final_lengthening = v.max(1.0);
        }
        if let Some(v) = self.accent {
            base.accent = v.max(0.0);
        }
        if let Some(v) = self.microtiming {
            base.microtiming = v.clamp(0.0, 1.0);
        }
        if let Some(v) = self.accelerando {
            base.accelerando = v.clamp(0.0, 1.0);
        }
        if let Some(v) = self.word_unit {
            base.word_unit = v;
        }
    }
}

/// 参考 tempo(ms):停顿表/重音拍的绝对毫秒在此 tempo 下成立,其他 tempo 等比缩放。
const REF_TEMPO_MS: f32 = 180.0;
/// 边界停顿表(design §3.2;× rest 旋钮 × tempo 缩放)。
const REST_MINOR_PUNCT: f32 = 100.0; // 逗号/顿号/分号
const REST_MAJOR_PUNCT: f32 = 250.0; // 句号/问叹号
const REST_LIST_ITEM: f32 = 150.0; // 列表项间
const REST_PARAGRAPH: f32 = 400.0; // 段落间
const REST_BLOCK: f32 = 600.0; // 块间(表格/代码/标题)
/// 重音拍:强调单元前的额外停顿(ms,tempo 缩放 × accent 旋钮)。
const ACCENT_PAUSE_MS: f32 = 100.0;
/// 微定时封顶(ms,tempo 缩放)。
const MICRO_CLAMP_MS: f32 = 50.0;

/// 拉丁词字符(词内连写不折拍):ASCII 字母数字 + 撇号。
fn is_word_cluster(c: &str) -> bool {
    !c.is_empty() && c.chars().all(|ch| ch.is_ascii_alphanumeric() || ch == '\'')
}

/// 空白(含换行):零成本、断词。
fn is_space_cluster(c: &str) -> bool {
    c.chars().all(char::is_whitespace) && !c.is_empty()
}

/// 次级标点(短停 100ms):ASCII `, ; :` + 全角 `,、;:`。
fn is_minor_punct(c: &str) -> bool {
    matches!(
        c,
        "," | ";" | ":" | "\u{3001}" | "\u{FF0C}" | "\u{FF1B}" | "\u{FF1A}"
    )
}

/// 主级标点(长停 250ms,兼句边界 = accelerando 复位):ASCII `. ! ?` + 全角 `。!?…`。
fn is_major_punct(c: &str) -> bool {
    matches!(
        c,
        "." | "!" | "?" | "\u{3002}" | "\u{FF01}" | "\u{FF1F}" | "\u{2026}"
    )
}

/// 卡片标题角色(tool/reasoning 的 `▸ name` / `💭 Thinking` 行):**整段一个单元**——
/// 标题是标签不是正文,逐字/逐词揭示只会闪(用户实测);一拍整体上屏 + 一次 enter 动画。
fn is_card_title_role(role: u32) -> bool {
    role == crate::content::StyleRole::ToolTitle as u32
}

/// 重音角色(粗体/标题/表格强调):design §3.2 ⑤「强调单元」的 v1 信号源。
fn is_accent_role(role: u32) -> bool {
    use crate::content::StyleRole as R;
    role == R::Bold as u32
        || role == R::BoldItalic as u32
        || role == R::Heading as u32
        || (R::Heading2 as u32..=R::Heading6 as u32).contains(&role)
        || role == R::TableStrong as u32
}

/// (seed, g) → [-1, 1] 确定性噪声(splitmix64 → 折叠)。
fn hash_noise(seed: u64, g: u64) -> f32 {
    let mut z = seed
        .wrapping_mul(0x9E37_79B9_7F4A_7C15)
        .wrapping_add(g.wrapping_mul(0xBF58_476D_1CE4_E5B9));
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^= z >> 31;
    // 折到 [-1,1](取高 24 位保均匀)。
    ((z >> 40) as f32) / f32::from(1u16 << 12).powi(2) - 1.0
}

/// 每 glyph 的释放成本表(ms)。仅**单元首 glyph** 有非零成本(词首/CJK 字);词内、空白、
/// 标点为 0(与首拍同帧上屏/吸附)。`tempo_ms` = 生效基准(有限值;∞ 的「不限速」由调度器
/// 层短路,不进本函数)。`seed` = 块级种子(R8:块 key 为种)。
pub fn cost_table(
    clusters: &[String],
    roles: &[u32],
    tree: &NodeTree,
    p: &RhythmParams,
    tempo_ms: f32,
    seed: u64,
) -> Vec<f32> {
    let n = clusters.len();
    let mut cost = vec![0.0f32; n];
    if n == 0 || !tempo_ms.is_finite() || tempo_ms <= 0.0 {
        return cost;
    }
    let scale = tempo_ms / REF_TEMPO_MS;

    // ── 节点边界 rest(0020 树):节点区间首 glyph 记边界级(取最大)。──
    let mut node_rest = vec![0.0f32; n];
    for (idx, node) in tree.nodes().iter().enumerate() {
        let g0 = node.range.0 as usize;
        if g0 >= n {
            continue;
        }
        // 顶层子节点 = 块边界:普通段落 400,结构块/标题 600;嵌套层只认 列表项/表格行 150。
        let r = if node.parent == 0 && idx != 0 {
            if node.kind == NodeKind::Paragraph {
                REST_PARAGRAPH
            } else {
                REST_BLOCK
            }
        } else {
            match node.kind {
                NodeKind::ListItem | NodeKind::TableRow => REST_LIST_ITEM,
                _ => 0.0,
            }
        };
        node_rest[g0] = node_rest[g0].max(r);
    }

    // ── 扫单元:词首/CJK 字计拍;跟踪标点/句位置。──
    let mut prev_word = false; // 上一非空白 cluster 是否词字符(词内判定)
    let mut prev_title = false; // 上一 cluster 是否卡片标题角色(标题整段一拍)
    let mut pending: f32 = 0.0; // 待付边界停顿(标点/节点边界,计给下一单元;块区间从前导 \n 起
                                // → 节点 rest 常落在空白位,须传导)
    let mut units_since_sentence = 0u32; // accelerando:句内单元计数
    let mut first_unit = true;
    let mut unit_starts: Vec<usize> = Vec::new();
    for g in 0..n {
        pending = pending.max(node_rest[g]);
        let c = clusters[g].as_str();
        if is_space_cluster(c) {
            prev_word = false;
            // 标题内空格(role 仍 ToolTitle)不断段;换行(role 随行)自然结束标题段。
            if !roles.get(g).copied().is_some_and(is_card_title_role) {
                prev_title = false;
            }
            continue;
        }
        if is_minor_punct(c) {
            pending = pending.max(REST_MINOR_PUNCT);
            prev_word = false;
            continue;
        }
        if is_major_punct(c) {
            pending = pending.max(REST_MAJOR_PUNCT);
            units_since_sentence = 0;
            prev_word = false;
            continue;
        }
        let word = is_word_cluster(c);
        let title = roles.get(g).copied().is_some_and(is_card_title_role);
        // 词拍(reader/flow):词字符仅词首计拍;逐字拍(typewriter):每个非空白/标点都计;
        // 卡片标题(ToolTitle 连续段):恒一个单元(空白也不断——整行一拍)。
        let unit_start = if title {
            !prev_title
        } else {
            !p.word_unit || !word || !prev_word
        };
        prev_word = word && !title;
        prev_title = title;
        if !unit_start {
            continue;
        }
        unit_starts.push(g);

        // ① Δ_base ×(⑦ 句内加速)×(⑤ 重音自身略慢)。
        let accel_r = (units_since_sentence as f32 / 24.0).min(1.0);
        let mut delta = tempo_ms * (1.0 - 0.35 * p.accelerando * accel_r);
        units_since_sentence += 1;
        let accent = p.accent > 0.0 && roles.get(g).copied().is_some_and(is_accent_role);
        if accent {
            delta *= 1.0 + 0.2 * p.accent.min(2.0);
        }

        // ③ 边界停顿:待付边界(标点/节点,已沿空白传导)× rest 旋钮 × scale。首单元不停(文首)。
        let boundary = pending;
        pending = 0.0;
        let rest = if first_unit {
            0.0
        } else {
            boundary * p.rest * scale
        };
        if boundary >= REST_PARAGRAPH {
            units_since_sentence = 0; // 段/块边界也复位句内加速
        }
        first_unit = false;

        // ⑤ 重音拍前停。
        let accent_pause = if accent {
            ACCENT_PAUSE_MS * p.accent * scale
        } else {
            0.0
        };

        // ⑥ 微定时:两倍频 hash 噪声(1/f 白噪近似)× 封顶。
        let eps = if p.microtiming > 0.0 {
            let noise =
                0.7 * hash_noise(seed, g as u64) + 0.3 * hash_noise(seed ^ 0xA5A5, (g / 4) as u64);
            noise * (0.2 * delta).min(MICRO_CLAMP_MS * scale) * p.microtiming
        } else {
            0.0
        };

        cost[g] = (delta + rest + accent_pause + eps).max(0.0);
    }

    // ── ④ 末延长:边界(标点/节点 rest)前最后一单元 Δ × final_lengthening。──
    // 以单元序回看:单元 u 的「下一事件」若是标点或带节点 rest 的单元 → u 的 Δ 部分乘系数。
    // 简化:给成本加 (len-1)×tempo 的近似(Δ 已含 accel/accent 缩放,误差可忽略;v1 取 1 单元)。
    if p.final_lengthening > 1.0 {
        for (ui, &g) in unit_starts.iter().enumerate() {
            // 找 g 之后、直到**下一单元首**(含)之间是否有边界信号(标点,或落在块间 \n /
            // 下一单元首上的节点 rest)。
            let next_g = unit_starts.get(ui + 1).copied().unwrap_or(n);
            let has_boundary = (g + 1..(next_g + 1).min(n)).any(|k| {
                node_rest[k] > 0.0
                    || is_minor_punct(clusters[k].as_str())
                    || is_major_punct(clusters[k].as_str())
            });
            if has_boundary || next_g >= n {
                cost[g] += tempo_ms * (p.final_lengthening - 1.0);
            }
        }
    }
    cost
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(src: &str) -> (Vec<String>, Vec<u32>, NodeTree) {
        let (spans, _t, tree, _e) = crate::content::parse_markdown_nodes(src, 0);
        let mut clusters = Vec::new();
        let mut roles = Vec::new();
        for s in &spans {
            for g in crate::support::graphemes(s.text()) {
                clusters.push(g.to_owned());
                roles.push(s.role().as_u32());
            }
        }
        (clusters, roles, tree)
    }

    fn reader() -> RhythmParams {
        RhythmParams::preset(RhythmPreset::Reader)
    }

    #[test]
    fn deterministic_same_seed_same_costs() {
        // M1 native ①(成本层):同 seed 同输入 → 成本表逐位相等;换 seed → 微定时不同。
        let (c, r, t) = parse("你好,世界。第二句 hello world 结束。\n\n新段落开始");
        let p = reader();
        let a = cost_table(&c, &r, &t, &p, 180.0, 42);
        let b = cost_table(&c, &r, &t, &p, 180.0, 42);
        assert_eq!(a, b, "同 seed 必须逐位相等(R8/R9)");
        let d = cost_table(&c, &r, &t, &p, 180.0, 43);
        assert_ne!(a, d, "换 seed 微定时应不同");
    }

    #[test]
    fn typewriter_zero_layers_uniform() {
        // M1 native ②(成本层):typewriter 预设(有限 tempo)= 纯匀速——词/字首恒 Δ,无停顿差。
        let (c, r, t) = parse("你好,世界。abc def\n\n新段");
        let p = RhythmParams {
            tempo_ms: 10.0,
            ..RhythmParams::preset(RhythmPreset::Typewriter)
        };
        let costs = cost_table(&c, &r, &t, &p, 10.0, 7);
        for (g, &x) in costs.iter().enumerate() {
            assert!(
                x == 0.0 || (x - 10.0).abs() < 1e-4,
                "typewriter 成本只有 0 或 Δ: g={g} cost={x}"
            );
        }
    }

    #[test]
    fn boundary_rest_monotone_with_level() {
        // M1 native ③:边界停顿单调 ∝ 层级——逗号 < 句号 < 段落 < 块。
        let (c, r, t) = parse("甲,乙。丙\n\n丁\n\n# 戊");
        let p = RhythmParams {
            microtiming: 0.0, // 关噪声,纯比层级
            ..reader()
        };
        let costs = cost_table(&c, &r, &t, &p, 180.0, 1);
        let at = |ch: &str| {
            let g = c.iter().position(|x| x == ch).expect("字符必在样本内");
            costs[g]
        };
        // 乙在逗号后、丙在句号后、丁在段落边界、戊在块边界(标题,且顶层)。
        assert!(
            at("乙") < at("丙"),
            "逗号 < 句号: {} {}",
            at("乙"),
            at("丙")
        );
        assert!(
            at("丙") < at("丁"),
            "句号 < 段落: {} {}",
            at("丙"),
            at("丁")
        );
        assert!(at("丁") <= at("戊"), "段落 ≤ 块: {} {}", at("丁"), at("戊"));
        assert!(at("甲") <= 180.0 * 1.4 + 1e-3, "首单元无边界停顿");
    }

    #[test]
    fn latin_word_is_one_beat() {
        // 词单位:词首计拍,词内 0(整词一拍);CJK 逐字计拍。
        let (c, r, t) = parse("hello 世界");
        let p = RhythmParams {
            microtiming: 0.0,
            ..reader()
        };
        let costs = cost_table(&c, &r, &t, &p, 180.0, 1);
        let g_h = c.iter().position(|x| x == "h").expect("h");
        assert!(costs[g_h] > 0.0, "词首计拍");
        for (k, &x) in costs.iter().enumerate().take(g_h + 5).skip(g_h + 1) {
            assert!(
                x.abs() < f32::EPSILON,
                "词内零成本(e/l/l/o): k={k} cost={x}"
            );
        }
        let g_shi = c.iter().position(|x| x == "世").expect("世");
        let g_jie = c.iter().position(|x| x == "界").expect("界");
        assert!(costs[g_shi] > 0.0 && costs[g_jie] > 0.0, "CJK 逐字计拍");
    }

    #[test]
    fn rest_scales_with_tempo_instant_at_zero() {
        // 额外项等比 tempo:tempo→~0(e2e set_reveal_cps(1e9))⇒ 全成本→~0(即时,不回归)。
        let (c, r, t) = parse("甲。乙\n\n丙");
        let costs = cost_table(&c, &r, &t, &reader(), 1e-6, 1);
        assert!(
            costs.iter().all(|&x| x < 0.01),
            "tempo≈0 ⇒ 成本≈0: {costs:?}"
        );
    }

    #[test]
    fn card_title_is_single_beat() {
        // 用户实测:标题逐字/逐词揭示会闪 → ToolTitle 连续段(含内部空格)= 一个单元:
        // 仅首字计拍,其余(含 "Thinking" 词字符与空格后字符)全 0 成本(同帧上屏)。
        let spans = vec![
            crate::content::StyledSpan::new(
                "💭 Thinking
",
                crate::content::StyleRole::ToolTitle,
            ),
            crate::content::StyledSpan::new("正文思考内容", crate::content::StyleRole::Reasoning),
        ];
        let mut clusters = Vec::new();
        let mut roles = Vec::new();
        for sp in &spans {
            for g in crate::support::graphemes(sp.text()) {
                clusters.push(g.to_owned());
                roles.push(sp.role().as_u32());
            }
        }
        let total: u32 = clusters.len() as u32;
        let tree = {
            // 扁平树(specific 渲染路径同款):单 Run 覆盖全部。
            let (_s, _t, tree, _e) = crate::content::parse_markdown_nodes(&clusters.concat(), 0);
            let _ = total;
            tree
        };
        let p = RhythmParams {
            microtiming: 0.0,
            ..RhythmParams::preset(RhythmPreset::Reader)
        };
        let costs = cost_table(&clusters, &roles, &tree, &p, 180.0, 1);
        let title_role = crate::content::StyleRole::ToolTitle.as_u32();
        let mut title_beats = 0;
        for (g, &r) in roles.iter().enumerate() {
            if r == title_role && clusters[g] != "\n" && costs[g] > 0.0 {
                title_beats += 1;
            }
        }
        assert_eq!(title_beats, 1, "标题整段应只计 1 拍(首字),其余同帧");
    }

    #[test]
    fn accent_bold_costs_more_than_plain() {
        // ⑤ 重音:粗体单元(前停 + 自身慢)成本 > 同位置普通词。
        let (cb, rb, tb) = parse("平文 **重点** 平文");
        let p = RhythmParams {
            microtiming: 0.0,
            ..reader()
        };
        let costs = cost_table(&cb, &rb, &tb, &p, 180.0, 1);
        let g_zhong = cb.iter().position(|x| x == "重").expect("重");
        let g_ping = cb.iter().position(|x| x == "平").expect("平");
        assert!(
            costs[g_zhong] > costs[g_ping],
            "重音单元应更贵: {} vs {}",
            costs[g_zhong],
            costs[g_ping]
        );
    }
}
