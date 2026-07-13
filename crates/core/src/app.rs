//! app(M13)— 每帧编排循环,串起 conn→protocol→store→smoother→content→layout→render。
//!
//! 严格分相(AR1):事件改状态(`apply`),渲染只读状态(`build_frame`)。
//! 时间确定性(R8):内部 `now_ms` 由注入的 `dt_ms` 逐帧累加,不碰墙钟。

use crate::camera::{Camera2D, Rect};
use crate::content::{parse_markdown_nodes, StyleRole};
use crate::frame::{FrameData, FrameGlyph, FramePanel, FrameRect, FrameWidget};
use crate::fsm::{next_status, FsmInput, SessionStatus, TurnStatus, TurnTracker};
use crate::protocol::{decode, parse_snapshot, Event};
use crate::reveal::{self, RevealScheduler, TableStyleKind};
use crate::seam::{Connection, LayoutEngine, PlacedGlyph, RawEvent, RenderSink};
use crate::smoother::Smoother;
use crate::spatial::HeightIndex;
use crate::store::{AskAnswer, AskKind, Store};
use crate::support::{graphemes, EventQueue};
use crate::theme;

/// catch-up 字形的 spawn_time:置于"远古",着色器淡入早已完成(alpha=1),实现零动画(AR6)。
const CATCHUP_SPAWN: f32 = -1.0e9;

/// 显示数学(`$$…$$`)相对行内的字号倍率:= H3(`roleScale` 1.3),更舒展 + 居中。
const DISPLAY_MATH_SCALE: f32 = 1.3;
/// 数学字形/规则线颜色(RGBA;暗色主题中性亮);后续可走 theme/可配。
const MATH_COLOR: [f32; 4] = [0.86, 0.88, 0.92, 1.0];
/// 数学 glyph 的 `glyph_idx` 基址:远离正文 placed 下标,morph 身份(block_seq,glyph_idx)不撞。
const MATH_IDX_BASE: u32 = 1_000_000;

/// 锚底阈值:**用户主动向下**滚到离底 ≤ 此值才重新跟随新内容(0002 §6 + 0038)。
const ANCHOR_THRESHOLD: f32 = 48.0;

/// 滚动跟随三态(0038,Plan 34 S1):显式 FSM 取代旧 `stick_to_bottom` 布尔。
/// Following == 旧 true 行为(golden 基线);Released 下流式增高零位移;
/// Anchoring = `scroll_to_latest` 的平滑过渡态,到底即 Following。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FollowState {
    /// 锚底跟随「已揭示底」(smooth-damp;落后超一屏直接到位)。
    Following,
    /// 用户接管:相机不动,内容增长不推视口(布局稳定不变量)。
    Released,
    /// 显式跳最新:平滑滚向底部,|pan−底|<0.5 → Following;释放信号可打断。
    Anchoring,
}

/// 锚底**平滑跟随**:临界阻尼 smooth-damp 的接近时间(秒,fps 无关;小=更跟手,大=更顺滑)。
/// 落后 > **一屏**(初次加载 / 历史瞬显 / 大段倾泻)才直接到位,否则平滑跟——流式不再 snap。
const ANCHOR_SMOOTH_TIME: f32 = 0.12;

/// 把累积的行内码 chip(`[x0,x1,y0,y1]`)推成一个带内边距的圆角底。
/// Plan 23 R3:flush 一条 diff 行底色带(整宽 `w`,左缘 `x`)。
/// 0037 §3:装饰指令 → 世界矩形。区间经 `cache.placed` 解析(逐行分段);slot 色 emit 时查
/// theme(0021 颜色不进缓存)。未知种类/越界区间兜底跳过(AR12)。
#[allow(clippy::too_many_arguments)] // reason: 摆放需缓存/几何/主题/命中收集多源;plan33 拆 crate 时收束
fn place_decorations(
    cache: &BlockCache,
    block_seq: u32,
    origin: [f32; 2],
    box_w: f32,
    inset: f32,
    th: &theme::Theme,
    out: &mut Vec<FrameRect>,
    fold_hits: &mut Vec<(u64, Rect)>,
) {
    use crate::partrender::{DecorKind, DecorSlot};
    let slot_color = |slot: DecorSlot| -> [f32; 4] {
        match slot {
            DecorSlot::DiffAddBand => th.diff_add_bg,
            DecorSlot::DiffDelBand => th.diff_del_bg,
            DecorSlot::DiffAddGutter => th.diff_gutter_add,
            DecorSlot::DiffDelGutter => th.diff_gutter_del,
            DecorSlot::DiffModGutter => th.diff_gutter_mod,
            DecorSlot::DiffAddWord => th.diff_word_add_bg,
            DecorSlot::DiffDelWord => th.diff_word_del_bg,
        }
    };
    // 区间 → 行段:按 y 分组连续 glyph(跳零宽换行占位)。
    let rows_of = |s0: usize, e0: usize| -> Vec<(f32, f32, f32, f32)> {
        // (y0, y1, x0, x1)
        let mut rows: Vec<(f32, f32, f32, f32)> = Vec::new();
        for j in s0..e0.min(cache.placed.len()) {
            let p = &cache.placed[j];
            if p.size[0] <= 0.0 && cache.clusters.get(j).is_some_and(|c| c == "\n") {
                continue;
            }
            let (y0, y1) = (p.pos[1], p.pos[1] + p.size[1]);
            let (x0, x1) = (p.pos[0], p.pos[0] + p.size[0]);
            match rows.last_mut() {
                Some(r) if (r.0 - y0).abs() < 0.5 => {
                    r.2 = r.2.min(x0);
                    r.3 = r.3.max(x1);
                    r.1 = r.1.max(y1);
                }
                _ => rows.push((y0, y1, x0, x1)),
            }
        }
        rows
    };
    // Band 两遍:先跨 op 收集行段(每条 diff 行一个 Band op),再按邻接发射 ——
    // 同 group 同 slot 且行贴邻的带经 gloop 融成连续圆角块(D3),颜色不同不融。
    let mut bands: Vec<(u32, DecorSlot, f32, f32)> = Vec::new(); // (group, slot, y0, y1)
                                                                 // gutter/词底后置暂存:带先画(垫底),条/词底压在带上(绘制序 = 数组序)。
    let mut overlays: Vec<FrameRect> = Vec::new();
    for op in &cache.decorations {
        let (s0, e0) = (op.span.0 as usize, op.span.1 as usize);
        if s0 >= e0 {
            continue;
        }
        let color = slot_color(op.slot);
        let rows = rows_of(s0, e0);
        if rows.is_empty() {
            continue;
        }
        match op.kind {
            DecorKind::Band => {
                for &(y0, y1, _, _) in &rows {
                    bands.push((op.group, op.slot, y0, y1));
                }
            }
            DecorKind::Gutter => {
                let line_h = rows[0].1 - rows[0].0;
                let w = (0.275 * line_h).floor().max(2.0); // Zed 语义值(research §2.2)
                let (y0, y1) = (rows[0].0, rows.last().map_or(rows[0].1, |r| r.1));
                overlays.push(FrameRect {
                    pos: [origin[0] + inset, y0 + origin[1]],
                    size: [w, y1 - y0],
                    color,
                    radius: 0.0,
                    stroke: 0.0,
                    fx: [0.0; 4],
                    fx_color: [0.0; 4],
                    gloop: [0.0; 4],
                });
            }
            DecorKind::CharBg => {
                for &(y0, y1, x0, x1) in &rows {
                    overlays.push(FrameRect {
                        pos: [x0 - 1.0 + origin[0], y0 - 1.0 + origin[1]],
                        size: [(x1 - x0) + 2.0, (y1 - y0) + 2.0],
                        color,
                        radius: 3.0,
                        stroke: 0.0,
                        fx: [0.0; 4],
                        fx_color: [0.0; 4],
                        gloop: [0.0; 4],
                    });
                }
            }
            // D4 折叠汇总行:Zed 式强圆角 gutter marker(宽 floor(0.35×行高)、圆角 = 行高,
            // shader 半尺寸夹紧 → 胶囊;research §2.2 deleted-fold marker 语义值)+ tap 命中盒。
            DecorKind::FoldSummary => {
                let (y0, y1) = (rows[0].0, rows.last().map_or(rows[0].1, |r| r.1));
                let line_h = y1 - y0;
                overlays.push(FrameRect {
                    pos: [origin[0] + inset, y0 + origin[1]],
                    size: [(0.35 * line_h).floor().max(3.0), line_h],
                    color,
                    radius: line_h,
                    stroke: 0.0,
                    fx: [0.0; 4],
                    fx_color: [0.0; 4],
                    gloop: [0.0; 4],
                });
                fold_hits.push((
                    (u64::from(block_seq) << 32) | u64::from(op.group),
                    Rect::new(
                        origin[0] + inset,
                        y0 + origin[1],
                        box_w - 2.0 * inset,
                        line_h,
                    ),
                ));
            }
            // D4 展开区:不画(glyph scale 包络在 build_frame 发射处施加)。
            DecorKind::FoldRegion => {}
        }
    }
    // Band 发射(D3 gloop):贴邻同色带携邻行几何,fragment smooth-union 融合;
    // 单行(两侧无邻)= 普通圆角(退化正确)。带整宽同 x → dx 恒 0。
    bands.sort_by(|a, b| a.2.total_cmp(&b.2));
    // 行底应满行盒:glyph 盒之间有行距(leading)→ 相邻带(缝 < 半行高)把 y1 拉到
    // 下一条 y0 闭合(跨色也闭合,同 opencode 满行连续;gloop 仍只融同色)。
    for i in 0..bands.len().saturating_sub(1) {
        let gap = bands[i + 1].2 - bands[i].3;
        let h = bands[i].3 - bands[i].2;
        if gap > 0.0 && gap < h * 0.5 {
            bands[i].3 = bands[i + 1].2;
        }
    }
    let bw = box_w - 2.0 * inset;
    for i in 0..bands.len() {
        let (_, slot, y0, y1) = bands[i];
        let (prev, next) = band_adjacent(&bands, i);
        out.push(FrameRect {
            pos: [origin[0] + inset, y0 + origin[1]],
            size: [bw, y1 - y0],
            color: slot_color(slot),
            radius: DIFF_BAND_RADIUS,
            stroke: 0.0,
            fx: [0.0; 4],
            fx_color: [0.0; 4],
            gloop: [
                0.0,
                if prev { bw } else { 0.0 },
                0.0,
                if next { bw } else { 0.0 },
            ],
        });
    }
    out.append(&mut overlays);
}

/// diff 行底圆角(px)。gloop 融合核 k = 2×radius(rect.wgsl):k=r 时接缝角残
/// +0.16r 的缺口,2r 才闭合(smin(a,a,k) = a − k/4,角点场值 ≈ 0.414r)。
const DIFF_BAND_RADIUS: f32 = 4.0;

/// D3:已按 y0 排序的带列表中第 `i` 条的(上邻接, 下邻接)——「同 group 同 slot 且行贴邻
/// (缝 < 0.5px)」才融合(相邻**同色**才 gloop)。纯函数,native 直测。
pub(crate) fn band_adjacent(
    bands: &[(u32, crate::partrender::DecorSlot, f32, f32)],
    i: usize,
) -> (bool, bool) {
    let (g, s, y0, y1) = bands[i];
    let prev = i > 0 && {
        let p = bands[i - 1];
        p.0 == g && p.1 == s && (y0 - p.3).abs() < 0.5
    };
    let next = i + 1 < bands.len() && {
        let n = bands[i + 1];
        n.0 == g && n.1 == s && (n.2 - y1).abs() < 0.5
    };
    (prev, next)
}

/// D4:展开动画起始 scale(makepad session.rs 范式的入端)。
const FOLD_SCALE_START: f32 = 0.9;

/// D4:折叠展开 scale 指数逼近一步(makepad **范式**:每帧 ×0.9 → 阈值 snap;实现自写)。
/// 归一到注入 dt:`factor = 0.9^(dt/16.667)`(60fps 一帧恰 ×0.9,帧率无关确定性 R8);
/// 距 1 不足 0.01 → snap 到 **恰好 1.0**(收敛恒等 AR3)。
/// F1(0040):反馈收敛自停时长 —— decay^n < 1/255 的 n 帧(60fps 折 ms)。
/// 纯函数(R8);decay≤0 → 0(即停)。
pub(crate) fn feedback_settle_ms(decay: f32) -> f64 {
    if decay <= 0.0 {
        return 0.0;
    }
    let n = (255.0f64.ln() / (1.0 / f64::from(decay.min(0.98))).ln()).ceil();
    n * (1000.0 / 60.0)
}

pub(crate) fn fold_scale_step(s: f32, dt_ms: f32) -> f32 {
    let f = 0.9f32.powf(dt_ms / (1000.0 / 60.0));
    let ns = 1.0 - (1.0 - s) * f;
    if 1.0 - ns < 0.01 {
        1.0
    } else {
        ns
    }
}

fn flush_chip(chip: Option<[f32; 4]>, color: [f32; 4], out: &mut Vec<FrameRect>) {
    if let Some([x0, x1, y0, y1]) = chip {
        out.push(FrameRect {
            pos: [x0 - 2.0, y0 - 1.0],
            size: [(x1 - x0) + 4.0, (y1 - y0) + 2.0],
            color,
            radius: 3.0,
            stroke: 0.0,
            fx: [0.0; 4],
            fx_color: [0.0; 4],
            gloop: [0.0; 4],
        });
    }
}

/// 把累积的删除线段(`[x0,x1,y0,y1]`)推成字中线一条细线(A:`~~…~~`,表格内/正文通用)。
fn flush_strike(seg: Option<[f32; 4]>, color: [f32; 4], out: &mut Vec<FrameRect>) {
    if let Some([x0, x1, y0, y1]) = seg {
        out.push(FrameRect {
            pos: [x0, (y0 + y1) * 0.5 - 0.75], // 字形垂直中点
            size: [x1 - x0, 1.5],
            color,
            radius: 0.0,
            stroke: 0.0,
            fx: [0.0; 4],
            fx_color: [0.0; 4],
            gloop: [0.0; 4],
        });
    }
}

/// Plan 23:给 registry-rendered part(StyledSpan 直出,无 markdown 结构)建**扁平节点树**:
/// `Doc(0..total)` + 单 **`Run`** 叶(0..total)。reveal 调度器只在 `Run`/`Glyph` 叶上标揭示
/// tier(容器靠递归到叶),故必须是叶——全字 tier 0,逐帧按 quota 揭入(打字感)。
/// 空内容(total=0)返回空树(调用方回退 markdown 路径)。
fn flat_node_tree(block_seq: u32, total: u32) -> crate::nodes::NodeTree {
    use crate::nodes::{glyph_key, Node, NodeKind};
    if total == 0 {
        return crate::nodes::NodeTree::default();
    }
    let nodes = vec![
        Node {
            kind: NodeKind::Doc,
            parent: 0,
            range: (0, total),
            key: glyph_key(block_seq, 0),
        },
        Node {
            kind: NodeKind::Run,
            parent: 0,
            range: (0, total),
            key: glyph_key(block_seq, 1),
        },
    ];
    crate::nodes::NodeTree::from_nodes(nodes)
}

/// 节点树调试叠加(Plan 7E / 0020):逐**容器**节点描其 glyph range 的 AABB(按 kind 上色),
/// 肉眼验"树是否套对每个结构块"。复用 4C3 几何叠加,随 `debug_geometry` 开关。
fn node_debug_rects(
    tree: &crate::nodes::NodeTree,
    placed: &[PlacedGlyph],
    origin: [f32; 2],
    out: &mut Vec<FrameRect>,
) {
    use crate::nodes::NodeKind;
    let nodes = tree.nodes();
    // 嵌套深度(build 保证 parent 下标 < 自身下标 → 前向一遍即得)。用于按层**内缩**框,
    // 让 Table>Row>Cell、List>ListItem、Quote>inner 各层不重叠、肉眼可分(否则全 +1px 糊一起)。
    let mut depth = vec![0u32; nodes.len()];
    for (i, n) in nodes.iter().enumerate() {
        if i > 0 {
            depth[i] = depth[n.parent as usize].saturating_add(1);
        }
    }
    for (i, n) in nodes.iter().enumerate() {
        let color = match n.kind {
            NodeKind::Heading => [0.40, 0.65, 1.0, 0.9],
            NodeKind::Paragraph => [0.55, 0.58, 0.66, 0.7],
            NodeKind::List => [0.40, 0.85, 0.50, 0.85],
            NodeKind::ListItem => [0.45, 0.75, 0.62, 0.7],
            NodeKind::Quote => [0.70, 0.55, 1.0, 0.85],
            NodeKind::CodeBlock => [0.95, 0.70, 0.35, 0.85],
            NodeKind::Table => [0.95, 0.45, 0.45, 0.9],
            NodeKind::TableRow => [0.95, 0.55, 0.55, 0.6],
            NodeKind::TableCell => [0.95, 0.70, 0.70, 0.5],
            // Doc(= 块全幅,与块 AABB 重复)/ Run / Glyph / Embed 不画(过密)。
            _ => continue,
        };
        let s = (n.range.0 as usize).min(placed.len());
        let e = (n.range.1 as usize).min(placed.len());
        let (mut x0, mut y0, mut x1, mut y1) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
        for p in &placed[s..e] {
            if p.size[0] <= 0.0 {
                continue; // 跳零墨(换行)占位
            }
            x0 = x0.min(p.pos[0]);
            y0 = y0.min(p.pos[1]);
            x1 = x1.max(p.pos[0] + p.size[0]);
            y1 = y1.max(p.pos[1] + p.size[1]);
        }
        // 外扩 2px 基线 - 按深度内缩 → 浅层在外、深层在内,层层可见。
        let pad = (2.0 - depth[i] as f32 * 1.5).max(-6.0);
        let (bx0, by0, bx1, by1) = (x0 - pad, y0 - pad, x1 + pad, y1 + pad);
        if bx1 > bx0 && by1 > by0 && x1 > x0 {
            out.push(FrameRect {
                pos: [bx0 + origin[0], by0 + origin[1]],
                size: [bx1 - bx0, by1 - by0],
                color,
                radius: 0.0,
                stroke: 1.0,
                fx: [0.0; 4],
                fx_color: [0.0; 4],
                gloop: [0.0; 4],
            });
        }
    }
}

/// 从块的字形角色派生装饰矩形(代码块底 / 行内码 chip / 引用·Alert 左条 / H1·H2 细线 /
/// 分隔线,Plan 4B1)。颜色令牌见 [`crate::theme`]。
#[allow(clippy::too_many_arguments)] // reason: 装饰需缓存/几何/样式/揭示进度多源;Plan 9C 再收束
/// 进场动画 profile id(0025 / Plan 10 §3b):**core 据 角色 + reveal 风格 决策**,shader 据 id 查 profile 表
/// (id 与 `glyph.wgsl::enter_profile_by_id` 对齐)。0=正文 / 1=表头·标题 pop / 2=整表风格的表头(更大更慢)。
/// 这是 3b 的"数据驱动"价值:比 3a(shader 按 style 派生)多了 reveal 上下文(此处用 table_style),
/// 且策略改动只动这一处、不碰 GPU 布局。
fn enter_profile_id(role: u32, table: TableStyleKind) -> u32 {
    let th = StyleRole::TableHeader.as_u32();
    let h1 = StyleRole::Heading.as_u32();
    let h2 = StyleRole::Heading2.as_u32();
    let is_heading = role == h1 || (role >= h2 && role <= h2 + 4); // H1 + H2..H6
    if role == th {
        return if matches!(table, TableStyleKind::Full) {
            2
        } else {
            1
        };
    }
    u32::from(is_heading) // 1 标题 / 0 正文
}

#[allow(clippy::too_many_arguments)] // reason: 装饰需缓存/几何/样式/揭示进度多源;后续再收束为 struct
fn block_decorations(
    cache: &BlockCache,
    block_seq: u32,
    origin: [f32; 2],  // Plan 13:盒左上角 world 坐标(装饰随 view 盒平移)
    box_w: f32,        // 盒宽(全宽装饰:代码底/引用条/分隔线/表头线锚它,非整窗宽)
    th: &theme::Theme, // Plan 26①:运行时令牌(Engine 持,`set_theme` 下一帧生效)
    dpr: f32,          // Plan 28:内衬/卡 pad = 参考 CSS px × dpr
    // Plan 27 A 路:按压中的 ask 按钮 run 下标(0=允许/1=拒绝;None=无按压)→ 该面板画深色。
    ask_pressed_run: Option<usize>,
    ts: &TableStyle,
    spawn: &[Option<f32>],
    reveal_kind: TableStyleKind,
    // Plan 25 M2b:注入时钟 now(卡片 wipe 入场)+ motion 令牌 + 慢时钟秒(流光相位)+ 动效组计数。
    now: f32,
    m: &crate::motion::MotionTokens,
    glow_s: f32,
    anim_groups: &mut usize,
    // Plan 25 M2f:指针世界坐标 + 按下态(卡片 hover 提亮 / press 微缩,0032 ① 视觉=SDF 参数)。
    pointer_world: Option<[f32; 2]>,
    pointer_down: bool,
    // Plan 25 M4:徽章 morph(from_status, changed_at);None = 稳态。
    badge_morph: Option<(u8, f32)>,
    out: &mut Vec<FrameRect>,
    panels: &mut Vec<FramePanel>,
    widgets: &mut Vec<FrameWidget>,
    // Plan 32 D4:折叠汇总行命中盒(build_frame 收集 → tap 展开)。
    fold_hits: &mut Vec<(u64, Rect)>,
) {
    let inline = StyleRole::Code.as_u32();
    let alert = StyleRole::AlertLabel.as_u32();
    let rule = StyleRole::Rule.as_u32();
    let h1 = StyleRole::Heading.as_u32();
    let h2 = StyleRole::Heading2.as_u32();
    let task_off = StyleRole::TaskUnchecked.as_u32();
    let task_on = StyleRole::TaskChecked.as_u32();
    // Plan 23 角色:tool 卡 / reasoning / diff(装饰在主循环前发 → 居字与其它 rect 之下)。
    let tool_title = StyleRole::ToolTitle.as_u32();
    let reasoning = StyleRole::Reasoning.as_u32();
    let diff_add = StyleRole::DiffAdded.as_u32();
    let diff_del = StyleRole::DiffRemoved.as_u32();
    // 是否"卡块"(tool/reasoning):有 ToolTitle 或 Reasoning 角色;且至少一字已揭(避免空卡先现)。
    let is_card = cache
        .roles
        .iter()
        .any(|&r| r == tool_title || r == reasoning);
    let any_revealed = (0..cache.placed.len()).any(|j| spawn.get(j).copied().flatten().is_some());
    // Plan 28 R3:整卡面板 + 状态徽章**退役** —— 参考(opencode)tool trigger 行无底、
    // reasoning 裸文;状态表达:pending/running = 标题 shimmer(R4),error = 红字。
    // (0018 面板/WIDGET_BADGE 机制保留在管线,数据驱动不再发;card_status 采集留给 R4。)
    let _ = (
        is_card,
        any_revealed,
        pointer_world,
        pointer_down,
        badge_morph,
        now,
        glow_s,
        m,
        &anim_groups,
    );
    // Plan 27:ask 按钮/chip 面板 —— AskButton 角色 run(pending 按钮 + 落定所选 chip)背后
    // 一块圆角实底;与 `build_frame` 的命中盒同源(`ask_button_runs`)→ 视觉与命中永远一致。
    for (k, b) in ask_button_runs(cache, 6.0).iter().enumerate() {
        let pressed = ask_pressed_run == Some(k);
        out.push(FrameRect {
            pos: [b[0] + origin[0], b[1] + origin[1]],
            size: [b[2] - b[0], b[3] - b[1]],
            color: if pressed {
                th.ask_button_bg_pressed
            } else {
                th.ask_button_bg
            },
            radius: 7.0,
            stroke: 0.0,
            fx: [0.0; 4],
            fx_color: [0.0; 4],
            gloop: [0.0; 4],
        });
    }
    // Plan 32(0037 首租):diff 行底/gutter/词级高亮**全部**来自 renderer 的声明式装饰指令
    // (cache.decorations),旧「扫 StyleRole 逐行反推」路径删净(flush_diff_band 已除役)。
    // 文件卡 bbox 仍按角色量程(plan28 遗留-2 结构;属卡容器非 diff 装饰)。
    let band_inset = 12.0 * dpr; // 卡内衬(参考 .shiki/diff 内衬 12px CSS)
    let mut diff_bbox: Option<(f32, f32)> = None; // (y0, y1) 世界
    for (j, p) in cache.placed.iter().enumerate() {
        let r = cache.roles[j];
        if r == diff_add || r == diff_del || r == StyleRole::DiffCtx.as_u32() {
            let y0 = p.pos[1] + origin[1];
            diff_bbox = Some(match diff_bbox {
                Some((a, b)) => (a.min(y0), b.max(y0 + p.size[1])),
                None => (y0, y0 + p.size[1]),
            });
        }
    }
    place_decorations(
        cache, block_seq, origin, box_w, band_inset, th, out, fold_hits,
    );
    if let Some((y0, y1)) = diff_bbox {
        let pad = 8.0 * dpr;
        panels.push(FramePanel {
            id: (u64::from(block_seq) << 32) | 0xFFFF_FFFC, // diff 文件卡槽
            pos: [origin[0], y0 - pad],
            size: [box_w, (y1 - y0) + 2.0 * pad],
            radius: 6.0,
            fill: th.code_bg, // --surface-inset-base(参考 diff-view 底)
            line_color: th.card_border,
            header_fill: [0.0; 4],
            line_w: 1.0,
            ao: 0.0,
            ao_color: [0.0; 3],
            ao_width: 0.0,
            header_ratio: 0.0,
            col_ratios: Vec::new(),
            row_ratios: Vec::new(),
            reveal: 1.0,
            edge_glow: 0.0,
            glow_phase: 0.0,
            flags: 0,
        });
    }
    let (mut qy0, mut qy1) = (f32::MAX, f32::MIN);
    let mut has_quote = false;
    // Plan 30 M5:引用左条 = 骨架 —— 按 cache 全量 Quote 角色量程(不看 spawn),
    // 结构确认即整条先现,字随节奏填充(同 code 框/表格网格)。
    for (j, p) in cache.placed.iter().enumerate() {
        let rr = cache.roles[j];
        if (rr == StyleRole::Quote.as_u32() || rr == StyleRole::AlertLabel.as_u32())
            && cache.clusters[j] != "\n"
        {
            let y0 = p.pos[1] + origin[1];
            qy0 = qy0.min(y0);
            qy1 = qy1.max(y0 + p.size[1]);
            has_quote = true;
        }
    }
    // H1/H2 底部细线(GitHub 风):**画在标题行自身底下**,不是块底(text part 含标题+正文时块底
    // 错位,Plan 25 r2 修)。按行聚:同一标题行取 y1 最大值,换行(y0 跳变)另起一条。
    let mut head_rules: Vec<f32> = Vec::new(); // 每条 = 标题行底 y
    let mut head_cur: Option<(f32, f32)> = None; // (行 y0, 行底 y1)
    let mut alert_label = String::new(); // 非空 = 该块是 Alert
                                         // 行内码 chip:同一行连续 Code 角色聚成一个圆角底,逐行 flush。
    let mut chip: Option<[f32; 4]> = None; // [x0, x1, y0, y1]
    let mut strike_seg: Option<[f32; 4]> = None; // 删除线段(同行连续 struck glyph),逐行 flush
    for (j, p) in cache.placed.iter().enumerate() {
        if cache.clusters[j] == "\n" {
            continue;
        }
        // 装饰与字**同一揭示门**(Plan 9):未释放的字不参与任何装饰累积(chip/strike/代码底/
        // 引用条)——否则行框/逐项揭示下,未揭 cell 的内联装饰会先于字显形(孤立色块/横线)。
        // 块级底/条因此只含已揭字 → 随揭示逐步长大(block 也 reveal)。未释放字打断连续段。
        if spawn.get(j).copied().flatten().is_none() {
            flush_chip(chip.take(), th.code_chip, out);
            flush_strike(strike_seg.take(), th.strike, out);
            continue;
        }
        let (x0, y0) = (p.pos[0] + origin[0], p.pos[1] + origin[1]);
        let (x1, y1) = (x0 + p.size[0], y0 + p.size[1]);
        let r = cache.roles[j];
        // 代码块底 / 框 / gutter 在循环后**逐块**从 `cache.code_blocks` 几何发(见下),不在此累加
        // (多代码块合并成一个大框是 boundary bug)。
        // 引用/Alert 左条量程已由块首**全量预扫**承担(Plan 30 M5 骨架先行);此处只拼
        // Alert 标签字形(取色用;标签在块首,首帧即齐)。
        if r == alert {
            alert_label.push_str(&cache.clusters[j]);
        }
        if r == h1 || r == h2 {
            match head_cur {
                Some((cy0, cy1)) if (cy0 - y0).abs() < 0.5 => {
                    head_cur = Some((cy0, cy1.max(y1)));
                }
                Some((_, cy1)) => {
                    head_rules.push(cy1);
                    head_cur = Some((y0, y1));
                }
                None => head_cur = Some((y0, y1)),
            }
        }
        // 分隔线:零墨 Rule 锚点 → 整宽细线(居其行垂直中点)。已释放才到此(循环顶部已门控)→
        // 随揭示节点出现(NodeSpawn,Plan 9 §2.6:ThematicBreak 标其 Rule 锚字 → 释放即画)。
        if r == rule {
            // 分隔线 `---`(Plan 28 R2):参考(opencode)为素细线 → WIDGET_RULE,bright=0.95
            // 近乎全宽实线(≤1px 容差)。喵喵分隔线(WIDGET_RULE_CAT)退役为可选彩蛋,不再默认。
            let mid = (y0 + y1) * 0.5;
            let qh = 6.0; // 细线 quad(留 AA 余量)
            widgets.push(FrameWidget {
                pos: [origin[0], mid - qh * 0.5],
                size: [box_w, qh],
                color: th.hr_rule,
                params: [0.5, 0.95, 0.0, 0.0], // 半厚 0.5(≈1px 线)/ 两端 5% 淡出
                component: crate::frame::WIDGET_RULE,
            });
        }
        // 任务复选框(0026/Plan 11):零墨锚点 cell → SDF 方框(已勾叠对勾);不借通用 FrameRect。
        // 方框为正方,边长 ≈ 行高 0.78×,左对齐锚点 cell、垂直居中(后随 Normal 间隔 cell 给出宽度)。
        if r == task_off || r == task_on {
            let lh = y1 - y0;
            let side = (lh * 0.78).max(6.0);
            let by = y0 + (lh - side) * 0.5;
            let checked = r == task_on;
            widgets.push(FrameWidget {
                pos: [x0, by],
                size: [side, side],
                color: if checked { th.task_done } else { th.task_box },
                params: [side * 0.22, 1.6, if checked { 1.0 } else { 0.0 }, 0.0],
                component: crate::frame::WIDGET_BOX,
            });
        }
        // 行内码:连续且同行则延展,否则 flush 旧的、起新的。
        if r == inline {
            match chip {
                Some(c) if (c[2] - y0).abs() < 0.5 => {
                    chip = Some([c[0], x1, c[2].min(y0), c[3].max(y1)]);
                }
                _ => {
                    flush_chip(chip, th.code_chip, out);
                    chip = Some([x0, x1, y0, y1]);
                }
            }
        } else if chip.is_some() {
            flush_chip(chip, th.code_chip, out);
            chip = None;
        }
        // 删除线(A):连续 struck glyph 同行聚成一段,逐行/逐段 flush → 字中线一条细线。
        if cache.strike[j] {
            match strike_seg {
                Some(c) if (c[2] - y0).abs() < 0.5 => {
                    strike_seg = Some([c[0], x1, c[2].min(y0), c[3].max(y1)]);
                }
                _ => {
                    flush_strike(strike_seg, th.strike, out);
                    strike_seg = Some([x0, x1, y0, y1]);
                }
            }
        } else if strike_seg.is_some() {
            flush_strike(strike_seg, th.strike, out);
            strike_seg = None;
        }
    }
    flush_chip(chip, th.code_chip, out);
    flush_strike(strike_seg, th.strike, out);
    if let Some((_, cy1)) = head_cur {
        head_rules.push(cy1);
    }
    // GitHub:H1/H2 底部细线,跨整块宽,贴标题行底(+2px 呼吸)。
    for ry in head_rules {
        out.push(FrameRect {
            pos: [origin[0], ry + 2.0],
            size: [box_w, 1.5],
            color: th.head_rule,
            radius: 0.0,
            stroke: 0.0,
            fx: [0.0; 4],
            fx_color: [0.0; 4],
            gloop: [0.0; 4],
        });
    }
    // 代码块底 / 框 / gutter:**逐块**从 `cache.code_blocks` 几何发(Plan 15 ①②⑥)。盒 = 全宽 × 行窗高
    // (`min(N,6)·lineH`),top = 块顶(块内相对 + origin),不会合并多块或盖住块间内容(修 box 边界 bug)。
    // 揭示门:块内有已释放字才发(避免流式空框先现)。
    for cb in &cache.code_blocks {
        // Plan 30 M5(0019 §162 骨架先行):结构确认(块已入 cache)即发**整框**,字随节奏
        // 后到 —— 框先于首字 spawn(表格网格先行同款;旧「有已揭字才发」的空框顾虑被骨架
        // 先行设计取代)。settled 块行为不变(框恒在)。
        let _ = &spawn;
        let win_h = crate::codeblock::window_height(cb.n_lines, cb.line_h);
        let bg_pos = [origin[0], origin[1] + cb.top_y - 4.0];
        let bg_size = [box_w, win_h + 8.0];
        out.push(FrameRect {
            pos: bg_pos,
            size: bg_size,
            color: th.code_bg,
            radius: 6.0,
            stroke: 0.0,
            fx: [0.0; 4],
            fx_color: [0.0; 4],
            gloop: [0.0; 4],
        });
        // 外框描边(Plan 15 ⑥:可见 box 框)。stroke>0 → 仅边框(rect.wgsl)。
        out.push(FrameRect {
            pos: bg_pos,
            size: bg_size,
            color: th.code_border,
            radius: 6.0,
            stroke: 1.5,
            fx: [0.0; 4],
            fx_color: [0.0; 4],
            gloop: [0.0; 4],
        });
        // gutter 分隔线(②⑥):行号列与代码区之间一条细竖线,跨行窗高。
        if cb.code_x0 > 0.0 {
            out.push(FrameRect {
                pos: [origin[0] + cb.code_x0 - 4.0, origin[1] + cb.top_y - 2.0],
                size: [1.0, win_h + 4.0],
                color: th.code_gutter_line,
                radius: 0.0,
                stroke: 0.0,
                fx: [0.0; 4],
                fx_color: [0.0; 4],
                gloop: [0.0; 4],
            });
        }
    }
    // 表格(0018 #5):layout 已按表给出精确网格几何(box/cols/rows/header_bottom,块内相对 px)。
    // **逐表**收敛成一个 SDF 面板(圆角外框 + 表头底 + 横线/竖线网格 + AO),不再从 glyph AABB
    // 反推或把同块多表合并成一个巨框。比例 = 网格线相对(加内边距的)框的占比,`top` 在 x/y 比例里
    // 抵消,只用于面板世界 pos.y。
    for (ti, t) in cache.table_panels.iter().enumerate() {
        let pad = 4.0; // 内容到边框的留白
        let gw = (t.w + 2.0 * pad).max(1.0);
        let gh = (t.h + 2.0 * pad).max(1.0);
        let col_ratios: Vec<f32> = t
            .cols
            .iter()
            .map(|&x| ((x - t.x + pad) / gw).clamp(0.0, 1.0))
            .collect();
        let row_ratios: Vec<f32> = t
            .rows
            .iter()
            .map(|&y| ((y - t.y + pad) / gh).clamp(0.0, 1.0))
            .collect();
        let header_ratio = if t.header_bottom > t.y {
            ((t.header_bottom - t.y + pad) / gh).clamp(0.0, 1.0)
        } else {
            0.0
        };
        // 揭示比例(0019 §2 风格化骨架):原始=恒 1;整表骨架=释放即整框(空框先现,字按
        // header→cell tier 后填);行框=框随"已揭行"逐步长大(行框先于该行字)。比例相对**框**
        // (含 pad,与 panel.wgsl 的 uv.y 同基:框顶 = t.y - pad、框高 gh)。
        let reveal = if reveal_kind == TableStyleKind::Raw {
            1.0
        } else {
            let (mut any, mut max_bottom) = (false, f32::MIN);
            for (j, pl) in cache.placed.iter().enumerate() {
                if cache.clusters[j] == "\n" || spawn.get(j).copied().flatten().is_none() {
                    continue; // 仅数已释放(spawn 有值)的字
                }
                let (gx, gy) = (pl.pos[0], pl.pos[1]);
                if gx >= t.x && gx <= t.x + t.w && gy >= t.y && gy <= t.y + t.h {
                    any = true;
                    max_bottom = max_bottom.max(gy + pl.size[1]);
                }
            }
            if !any {
                0.0 // 整表/行框:未释放任何字 → 框尚不画
            } else if reveal_kind == TableStyleKind::Full {
                1.0
            } else {
                // 行框:长到 ≥ 当前已揭字底的最近行线(t.rows ∪ 表底),含该行框线。
                let mut edge = t.y + t.h;
                for &ry in &t.rows {
                    if ry >= max_bottom {
                        edge = edge.min(ry);
                    }
                }
                if edge >= t.y + t.h - 0.5 {
                    1.0
                } else {
                    ((edge - t.y + pad) / gh).clamp(0.0, 1.0)
                }
            }
        };
        panels.push(FramePanel {
            id: (u64::from(block_seq) << 32) | ti as u64, // 稳定身份 → 0016 panel 补间(6D)
            pos: [t.x + origin[0] - pad, t.y + origin[1] - pad],
            size: [gw, gh],
            radius: ts.radius,
            fill: [0.0, 0.0, 0.0, 0.0],
            line_color: ts.line_color,
            header_fill: ts.header_fill,
            line_w: ts.line_w,
            ao: ts.ao,
            ao_color: ts.ao_color,
            ao_width: ts.ao_width,
            header_ratio,
            col_ratios,
            row_ratios,
            reveal,
            edge_glow: 0.0,
            glow_phase: 0.0,
            // Plan 28 遗留-4:参考表格仅横线(th/td 底线,无竖线无外框)→ ROWS_ONLY。
            flags: crate::frame::PANEL_GRID
                | crate::frame::PANEL_AO
                | crate::frame::PANEL_ROWS_ONLY,
        });
    }
    if has_quote {
        let is_alert = !alert_label.is_empty();
        // Alert:整块淡底(GitHub 风)+ 类型色左条;普通引用:中性左条。
        if is_alert {
            out.push(FrameRect {
                pos: [origin[0], qy0 - 3.0],
                size: [box_w, (qy1 - qy0) + 6.0],
                color: th.alert_bg(&alert_label),
                radius: 5.0,
                stroke: 0.0,
                fx: [0.0; 4],
                fx_color: [0.0; 4],
                gloop: [0.0; 4],
            });
        }
        out.push(FrameRect {
            pos: [origin[0], qy0],
            size: [3.0, qy1 - qy0],
            color: if is_alert {
                th.alert_bar(&alert_label)
            } else {
                th.quote_bar
            },
            radius: 0.0,
            stroke: 0.0,
            fx: [0.0; 4],
            fx_color: [0.0; 4],
            gloop: [0.0; 4],
        });
    }
}

/// 已排版块的缓存(Phase G 块冻结 + Phase H markdown):内容/宽度不变则不重排。
///
/// markdown 渲染隐藏语法标记,故**显示字形序列**(`clusters`/`roles`/`placed`)与源文本
/// `revealed` 不再 1:1;三者长度一致(每个显示 grapheme 一组),spawn_time 在 build 时由
/// `revealed` 近似映射。
struct BlockCache {
    /// 排版时的源 grapheme 数(变了即脏)。
    revealed_len: usize,
    /// 排版时的宽度(变了即脏)。
    width: f32,
    /// 显示 grapheme 文本(markdown 渲染后)。
    clusters: Vec<String>,
    /// 每个显示 grapheme 的样式角色数值。
    roles: Vec<u32>,
    /// 每个显示 grapheme 是否删除线(与 `clusters` 1:1;A:render 在字中线画线)。
    strike: Vec<bool>,
    /// 块内相对位置(与 `clusters` 顺序 1:1)。
    placed: Vec<PlacedGlyph>,
    /// 块内容最右墨边(= max over placed of `pos.x+size.x`;Plan 19 P1)。随 `height` 在
    /// `ensure_layouts` fold 一次,`build_frame::sizes` 读它 → 免每帧 fold 全部 placed
    /// (O(总glyph)→O(views),fps 主救)。**聚合维**:P2 释放几何后仍保留(与 `height` 同守布局稳定)。
    content_width: f32,
    /// 块高度。
    height: f32,
    /// 块内每个表格的面板几何(box + 竖/横网格 + 表头底,块内相对 px;0018 #5);非表格块为空。
    table_panels: Vec<crate::TablePanel>,
    /// 内容节点树(0020 / Plan 7):该块结构 + 稳定身份;下游 reveal/embed/morph 的查询地基。
    nodes: crate::nodes::NodeTree,
    /// 数学块(Plan 12 ②/⑤):每个公式的 (glyph 区间, RaTeX 排版结果, 是否显示数学 `$$`)。在
    /// ensure_layouts 算一次(随块冻结缓存,不每帧重排 RaTeX);build_frame 据此出数学 SDF 字形,跳过
    /// 区间内 raw TeX。`display=true`(`$$…$$`)= H3 字号 + 居中;`false`(行内 `$…$`)= 正文字号、贴行。
    math: Vec<((u32, u32), crate::math::MathLayout, bool)>,
    /// 图片嵌入(Plan 14 ①):每个 `![alt](url)` 的 (glyph 占位区间, url, alt)。alt 已作占位文本上屏
    /// (Failed 兜底);下游(②④)据 url 解码 → Ready 时改发纹理 quad。随块冻结缓存。
    embeds: Vec<crate::EmbedRegion>,
    /// 行内链接 sidecar(Plan 34 S2):(glyph 区间, url);URL 旁传不进 glyph 流(AR10)。
    links: Vec<crate::LinkRegion>,
    /// 代码块行窗视口(Plan 15 ①):每个 fenced code block 的 (glyph 区间, 块顶 y, 行数, 行高)。超
    /// `MAX_LINES` 行时 ensure_layouts 已把其**后**内容上移钉死窗高;build_frame 据此 scroll/cull/fade。
    code_blocks: Vec<CodeView>,
    /// tool 卡状态(Plan 25 M2b:0=pending 1=running 2=done 3=error;非 tool 块 None)。
    /// 自 `RenderPart.kind_tag`("tool:<name> · <status>")在 ensure_layouts 提取,随块缓存;
    /// 驱动 running 边缘流光 + 状态徽章。
    /// 0037(Plan 32):声明式装饰指令(随块冻结;emit 时解析 slot→theme 色)。
    decorations: Vec<crate::partrender::DecorationOp>,
    #[allow(dead_code)] // Plan 28 R4 shimmer 将复用(pending/running 标题微光)
    card_status: Option<u8>,
}

/// 一个代码块的行窗视口(Plan 15 ①):`range` = glyph 区间;`top_y` = 块顶 y(块内相对 px);`n_lines`
/// 总行数、`line_h` 行高 → 行窗 `min(N,6)·lineH` + tail/cull/fade。
#[derive(Clone, Copy, Debug)]
struct CodeView {
    range: (u32, u32),
    top_y: f32,
    n_lines: usize,
    line_h: f32,
    /// 代码内容起始 x(块内相对 px,= 首个 CodeBlock 字左缘,行号 gutter 之右)。横滚硬裁左界(Plan 15 ⑤)。
    code_x0: f32,
}

/// build_frame 内每个代码块的行窗解算结果(Plan 15 ①④):区间 + 几何 + 当前 scroll。
#[derive(Clone, Copy)]
struct CodeWindow {
    range: (u32, u32),
    top_y: f32,
    view_h: f32,
    line_h: f32,
    scroll_y: i32,
    max_scroll: i32,
    /// 横向滚动 px(④;仅 CodeBlock 字偏移,行号 gutter 不动)。
    scroll_x: f32,
    /// 代码内容左界世界 x(⑤ 横裁:CodeBlock 字横滚到此左则裁,别压行号 gutter)。
    code_left: f32,
    /// 代码区右界世界 x(⑤ 横裁:= 盒右沿)。
    code_right: f32,
}

/// 本块一个**已就绪**图片嵌入的绘制信息(Plan 14 ③④):
/// `(embed 下标, alt 占位 glyph 区间, 动图?, 解码自然尺寸, tex_id, alpha 淡入)`。
type ReadyEmbed = (usize, (u32, u32), bool, (f32, f32), u32, f32);

/// 图片就绪淡入时长(ms,0025 / Plan 14 ④)。
const IMAGE_FADE_MS: f32 = 200.0;

/// 复制图标边长 / 内边距(world px,Plan 15 ③)。
const COPY_ICON_PX: f32 = 18.0;
const COPY_ICON_PAD: f32 = 6.0;
/// 代码块上/下外边距(world px,Plan 15 ⑥):代码框与上下内容之间的留白。
const CODE_BLOCK_MARGIN: f32 = 10.0;

/// 每个可见 part 的上屏进度 + 排版缓存。
/// 工作集档位(0029 §2,Plan 29 V2 落全四级)。驻留矩阵见 0029;每级重建源都是纯函数链
/// (R8 确定性):`Store text → display_source 重切 → ensure_layouts`。
/// - `Hot`:几何就绪(可绘制)。
/// - `Warm`(>3 屏):释放 `cache`(placed/nodes);留 `agg`(height)/`revealed`/`spawn`。
/// - `Cold`(>8 屏):再丢 `revealed`+`spawn`(重建 = display_source 重切 + 全 `Some(-1e9)`
///   catch-up 瞬显,AR6;与 0029 矩阵差异:nodes 随 cache 在 Warm 已释,记 progress)。
/// - `FrozenFar`(>24 屏,**仅 resync 源可用时**):再释 Store text(0003 catch-up 重取);
///   无 server(重放/bench)封顶 Cold → 重放确定性不破。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tier {
    Hot,
    Warm,
    Cold,
    FrozenFar,
}

/// 块的**聚合维**(Plan 19 P2):释放重几何后仍保留 → boxlayout 占位稳定(零跳变,0029 §3)。
/// = 内容宽 + 高 + 排版时源 grapheme 数(重入 dirty 判据)。
#[derive(Debug, Clone, Copy)]
struct BlockAgg {
    content_width: f32,
    height: f32,
}

struct PartView {
    /// 退场 dissolve 起点(Plan 36 N3;None=正常)。孤儿检测置起点,dur 后真清除。
    exiting: Option<f64>,
    part_id: String,
    /// 已 push 进 smoother 的 grapheme 数(对账后从尾部续推)。
    pushed: usize,
    /// 已入队文本的**字节长度**(Plan 19 P1 真修:`enqueue_new_text` 据此 O(1) 跳过未增长的 part,
    /// 免每帧重切 grapheme + 堆分配整段(原 O(总历史)/帧 = fps 真凶,非 plan19 §0 猜的 sizes fold)。
    /// 对账(updated 全量覆盖)可能缩短文本 → 用「!=」而非「>」判定,缩短也触发重切。
    pushed_bytes: usize,
    /// 已**到达**的 (grapheme, 到达时刻) —— 内容真值(smoother 整流后的源 grapheme 序列)。
    /// 注:到达时刻不再直接作 spawn_time(0019:呈现时刻由调度器定);仅 `< 0`(catch-up)
    /// 用作"瞬显"信号。display 字形序列由其重解析得到(markdown 渲染后与之非 1:1)。
    revealed: Vec<(String, f32)>,
    /// 排版缓存(冻结);None 或脏时重排。
    cache: Option<BlockCache>,
    /// 调度器(0019 §4.3)逐 display 字形的 `spawn_time`:`Some` = 已释放上屏(带其 spawn),
    /// `None` = 未释放(hold,本帧不绘制)。下标 = display 字形序(与 `cache.placed` 1:1)。
    spawn: Vec<Option<f32>>,
    /// 瞬显(catch-up / resync 灌入的历史块):整段一帧上屏,零淡入(AR6),绕过揭示时钟。
    instant: bool,
    /// 已结算(所有**非换行**字已释放):置位后 `schedule` 每帧 O(1) 跳过——不扫 spawn、不重 resolve
    /// (性能命脉,0025 §4)。内容增长(spawn resize)/ `restart_reveal` 清零;换行永不 spawn 故不计入。
    settled: bool,
    /// 角色(Plan 13 §2):user 右 / assistant 左。`view_mut` 创建时按 store 填;未知默认 Assistant。
    role: crate::store::Role,
    /// 工作集档位(Plan 19 P2)。Hot=几何就绪;Warm=已释放(`cache=None`,凭 `agg` 占位)。
    tier: Tier,
    /// 聚合维(Plan 19 P2):随 cache 重建时更新;**释放后仍保留** → boxlayout 占位不塌(0029 §3)。
    /// `None` = 从未排版过(空/未到达)。
    agg: Option<BlockAgg>,
    /// Plan 31 R2:显示字形的**稳定身份**(morph 键;下标对齐 cache.clusters)。前后缀 diff
    /// 分配 → 中段插入不打乱两端身份。id 溢出计数器持续递增(u32 足够:单块字形 ≪ 2^32)。
    glyph_ids: Vec<u32>,
    id_clusters: Vec<String>,
    next_glyph_id: u32,
    /// Plan 31 R2:数学区的稳定身份(region 序对齐 cache.math;同 helper 前后缀 diff,
    /// `$x$`→`$x^2$` 时 x 保身份平滑移动;键空间经 MATH_IDX_BASE 与正文分离)。
    math_ids: Vec<Vec<u32>>,
    math_id_clusters: Vec<Vec<String>>,
    next_math_id: u32,
    /// tool 卡徽章 morph(Plan 25 M4 / Plan 10 相位 4 最小件):`(from_status, changed_at_ms)`——
    /// 状态迁移时记录,`dur_element` 内做 SDF `mix` 形变(pending→running→✓);None = 无迁移中。
    badge_morph: Option<(u8, f32)>,
    /// 上次观察到的 tool 卡状态(**view 级**,跨 cache 重置存活——part.updated 全量重写会重建
    /// cache,旧 cache 的 status 读不到;迁移检测靠本字段)。
    last_card_status: Option<u8>,
}

/// Plan 31 R4(0036):`data:image/svg+xml[,;]…` → SVG 源文本(percent-decode 简版;
/// base64 变体 v1 不解,返回 None 走纹理路)。纯函数(CR1/R8)。
fn svg_source_from_data_uri(url: &str) -> Option<String> {
    let rest = url.strip_prefix("data:image/svg+xml")?;
    let body = rest.strip_prefix(',').or_else(|| {
        rest.strip_prefix(";utf8,")
            .or_else(|| rest.strip_prefix(";charset=utf-8,"))
    })?;
    // percent-decode(%XX;+ 不转空格 —— data URI 语义)。
    let bytes = body.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(v) = u8::from_str_radix(&body[i + 1..i + 3], 16) {
                out.push(v);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8(out).ok()
}

/// Plan 31 R2(magic-move):**内容身份分配** —— 对新旧 cluster 序列做公共前缀 + 公共后缀
/// 匹配:前缀沿用旧 id、后缀沿用旧 id(随内容整体平移)、中段新分配。纯函数(R8):
/// 同输入同输出;append-only 时新 id 恰为 0..n 连续(与旧位置键等价,零回归)。
/// 中段插入(对账重写)时前后文字形保持身份 → morph 平滑移动而非重建闪烁(thinking §2)。
fn assign_stable_ids(
    old_clusters: &[String],
    old_ids: &[u32],
    new_clusters: &[String],
    mut next_id: u32,
) -> (Vec<u32>, u32) {
    debug_assert_eq!(old_clusters.len(), old_ids.len());
    let mut prefix = 0usize;
    while prefix < old_clusters.len()
        && prefix < new_clusters.len()
        && old_clusters[prefix] == new_clusters[prefix]
    {
        prefix += 1;
    }
    let mut suffix = 0usize;
    while suffix < old_clusters.len() - prefix
        && suffix < new_clusters.len() - prefix
        && old_clusters[old_clusters.len() - 1 - suffix]
            == new_clusters[new_clusters.len() - 1 - suffix]
    {
        suffix += 1;
    }
    let mut ids = Vec::with_capacity(new_clusters.len());
    ids.extend_from_slice(&old_ids[..prefix]);
    for _ in 0..(new_clusters.len() - prefix - suffix) {
        ids.push(next_id);
        next_id += 1;
    }
    ids.extend_from_slice(&old_ids[old_ids.len() - suffix..]);
    (ids, next_id)
}

/// 渲染后纯文本(Plan 21 P1):显示字形序列 join。`clusters` 已是 markdown 渲染后字形(语法标记已去),
/// 含 `"\n"` 占位 → concat 即得带换行的纯文本。
fn rendered_text(clusters: &[String]) -> String {
    clusters.concat()
}

/// 从 asked 事件 payload 抽 (prompt, options)(shape 未冻结 → 多键名容忍,缺则用缺省;AR12)。
fn ask_fields(payload: &serde_json::Value, default_prompt: &str) -> (String, Vec<String>) {
    let prompt = ["prompt", "question", "title", "text", "description"]
        .iter()
        .find_map(|k| payload.get(*k).and_then(serde_json::Value::as_str))
        .unwrap_or(default_prompt)
        .to_owned();
    let options = payload
        .get("options")
        .and_then(serde_json::Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(|v| {
                    v.as_str().map(str::to_owned).or_else(|| {
                        v.get("label")
                            .and_then(serde_json::Value::as_str)
                            .map(str::to_owned)
                    })
                })
                .collect()
        })
        .unwrap_or_default();
    (prompt, options)
}

/// 扫描块内 [`StyleRole::AskButton`] 的连续 run(同行)→ 每 run 一个**块内相对** AABB(外扩 pad)。
/// `block_decorations`(画按钮面板)与 `build_frame`(产命中盒)共用 → 视觉与命中永远一致。
fn ask_button_runs(cache: &BlockCache, pad: f32) -> Vec<[f32; 4]> {
    let role = StyleRole::AskButton.as_u32();
    let n = cache.placed.len().min(cache.roles.len());
    let mut out: Vec<[f32; 4]> = Vec::new();
    let mut cur: Option<[f32; 4]> = None; // [x0,y0,x1,y1]
    for i in 0..n {
        let is_btn = cache.roles[i] == role && cache.clusters[i] != "\n";
        if !is_btn {
            if let Some(b) = cur.take() {
                out.push([b[0] - pad, b[1] - pad, b[2] + pad, b[3] + pad]);
            }
            continue;
        }
        let p = &cache.placed[i];
        let (x0, y0) = (p.pos[0], p.pos[1]);
        let (x1, y1) = (x0 + p.size[0], y0 + p.size[1]);
        match &mut cur {
            Some(b) if (y0 - b[1]).abs() < 0.5 => {
                b[0] = b[0].min(x0);
                b[2] = b[2].max(x1);
                b[3] = b[3].max(y1);
            }
            _ => {
                if let Some(b) = cur.take() {
                    out.push([b[0] - pad, b[1] - pad, b[2] + pad, b[3] + pad]);
                }
                cur = Some([x0, y0, x1, y1]);
            }
        }
    }
    if let Some(b) = cur.take() {
        out.push([b[0] - pad, b[1] - pad, b[2] + pad, b[3] + pad]);
    }
    out
}

/// 取 part 的 messageID(Plan 22 P4 / F11 冻结判定用;`Other` → 空)。
fn part_message_id(part: &crate::protocol::Part) -> String {
    use crate::protocol::Part;
    match part {
        Part::Text { message_id, .. }
        | Part::Reasoning { message_id, .. }
        | Part::Tool { message_id, .. }
        | Part::File { message_id, .. }
        | Part::Compaction { message_id, .. } => message_id.clone(),
        Part::Other => String::new(),
    }
}

/// 把一个块的显示字形按 `"\n"` 切行,产逐行 world 盒 + 文本(Plan 21 P2 文本层)。
/// 行盒 = 该行字形的并集 AABB(world = `placed` 相对 + `origin`);纯换行/空行跳过。`char0` = 行首
/// 字形在块内下标(`clusters`/`placed` 1:1)→ host DOM 选区映回字符区间用。
fn line_runs(cache: &BlockCache, block: u32, origin: [f32; 2], out: &mut Vec<VisibleTextRun>) {
    let n = cache.clusters.len().min(cache.placed.len());
    let mut i = 0usize;
    while i < n {
        if cache.clusters[i] == "\n" {
            i += 1;
            continue;
        }
        let start = i;
        let mut text = String::new();
        let (mut x0, mut y0) = (f32::MAX, f32::MAX);
        let (mut x1, mut y1) = (f32::MIN, f32::MIN);
        while i < n && cache.clusters[i] != "\n" {
            let p = &cache.placed[i];
            x0 = x0.min(p.pos[0]);
            y0 = y0.min(p.pos[1]);
            x1 = x1.max(p.pos[0] + p.size[0]);
            y1 = y1.max(p.pos[1] + p.size[1]);
            text.push_str(&cache.clusters[i]);
            i += 1;
        }
        if i < n && cache.clusters[i] == "\n" {
            i += 1; // 吞掉行尾换行
        }
        if x1 > x0 && y1 > y0 {
            out.push(VisibleTextRun {
                block,
                char0: start as u32,
                origin: [x0 + origin[0], y0 + origin[1]],
                width: x1 - x0,
                height: y1 - y0,
                text,
            });
        }
    }
}

/// 选区高亮(Plan 21 P2→P3):为 `view` 块上落在选区内的**非零墨**字形发高亮 `FrameRect`(world =
/// `placed` 相对 + `origin`)。**逐行合并成圆角连续墨团**(P3:同行内被选字形并成一条圆角条,
/// 比逐字形 rect 更像 macOS 选区;跨行各自一条 → 0025 §4 的 smin 跨行连体作进一步视觉细化)。
/// 合并保"不漏选"(每个被选非零墨字形盒 ⊆ 其行墨团,N7)且不波及行内未选字(行内被选字形连续)。
/// 颜色 [`Theme::selection`](theme::Theme);进 `rects`(glyph 前绘制)→ 文字永在其上。
fn push_selection_rects(
    cache: &BlockCache,
    origin: [f32; 2],
    th: &theme::Theme,
    sel: &[(usize, usize, usize)],
    view: usize,
    out: &mut Vec<FrameRect>,
) -> usize {
    let n = cache.placed.len().min(cache.clusters.len());
    let mut pushed = 0usize;
    // 累积当前行墨团的 AABB(world);行变(pos.y 变)或遇换行 → flush。
    let mut cur: Option<[f32; 4]> = None; // [x0,y0,x1,y1]
    let flush = |cur: &mut Option<[f32; 4]>, out: &mut Vec<FrameRect>, pushed: &mut usize| {
        if let Some([x0, y0, x1, y1]) = cur.take() {
            let h = y1 - y0;
            out.push(FrameRect {
                pos: [x0, y0],
                size: [x1 - x0, h],
                color: th.selection,
                radius: (h * 0.28).min(6.0), // 圆角墨团(P3)
                stroke: 0.0,
                fx: [0.0; 4],
                fx_color: [0.0; 4],
                gloop: [0.0; 4],
            });
            *pushed += 1;
        }
    };
    for &(v, s, e) in sel {
        if v != view {
            continue;
        }
        flush(&mut cur, out, &mut pushed); // 不同区间不跨并
        for i in s.min(n)..e.min(n) {
            if cache.clusters[i] == "\n" {
                flush(&mut cur, out, &mut pushed); // 换行 → 收束本行墨团
                continue;
            }
            let p = &cache.placed[i];
            if p.size[0] <= 0.0 {
                continue; // 零墨占位不画
            }
            let (gx0, gy0) = (p.pos[0] + origin[0], p.pos[1] + origin[1]);
            let (gx1, gy1) = (gx0 + p.size[0], gy0 + p.size[1]);
            match &mut cur {
                // 同行(顶 y 接近)→ 并入;换行(y 跳)→ flush 再起新条。
                Some(b) if (gy0 - b[1]).abs() < 0.5 => {
                    b[0] = b[0].min(gx0);
                    b[2] = b[2].max(gx1);
                    b[3] = b[3].max(gy1);
                }
                _ => {
                    flush(&mut cur, out, &mut pushed);
                    cur = Some([gx0, gy0, gx1, gy1]);
                }
            }
        }
        flush(&mut cur, out, &mut pushed);
    }
    flush(&mut cur, out, &mut pushed);
    pushed
}

/// 把 views(到达序)分组成回合(Plan 13 §4.3,纯投影):遇 User part 开新回合;连续 Assistant
/// part 归当前回合的同一 AsstBox。无前导 user 的 assistant 自成一回合(user=None)。`TurnGroup`
/// 定义在 [`crate::boxlayout`](布局消费方)。
fn group_turns(views: &[PartView]) -> Vec<crate::boxlayout::TurnGroup> {
    use crate::boxlayout::TurnGroup;
    use crate::store::Role;
    let mut turns: Vec<TurnGroup> = Vec::new();
    for (vi, v) in views.iter().enumerate() {
        match v.role {
            Role::User => turns.push(TurnGroup {
                user: Some(vi),
                assistant: Vec::new(),
            }),
            Role::Assistant => match turns.last_mut() {
                // 当前回合已有 user 或已有 assistant → 续入同一 AsstBox。
                Some(t) => t.assistant.push(vi),
                // 开头就是 assistant(无 user 锚)→ 自成一回合。
                None => turns.push(TurnGroup {
                    user: None,
                    assistant: vec![vi],
                }),
            },
        }
    }
    turns
}

/// 每帧渲染统计(可观测;`?debug` 时 wasm 侧节流打日志)。emit/total 比值暴露"是否每帧发整篇"。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FrameStats {
    /// 反馈通道活跃(0040;off/收敛自停 = false)。F0 恒 false(pass 未落,直通)。
    pub feedback_active: bool,
    /// 反馈 RT 显存字节(2×w×h×4×dpr²;off = 0,FrameStats 断言口径)。
    pub rt_bytes: usize,
    /// 本帧实际发射的 glyph 数(经 glyph 级裁剪后)。
    pub frame_glyphs: usize,
    /// 可绘制块的 glyph 总数(裁剪前;emit≈total 说明没裁到/单巨块)。
    pub total_glyphs: usize,
    /// 本帧实际出 glyph 的块数。
    pub visible_blocks: usize,
    /// 可绘制块总数。
    pub total_blocks: usize,
    /// 本帧屏上活跃 ShaderBox 数(经 cull 后;离屏不计)。护栏度量(Plan 16 §2.4)。
    pub shaderbox_active: usize,
    /// 屏上 ShaderBox 像素和(Σ box∩viewport 面积;离屏/裁剪外不计)。
    pub shaderbox_pixels: u64,
    /// 本帧活跃动效组数(Plan 25 M2a:dynamic ShaderBox + 呼吸 widget + wipe 面板;设计原则 4「同屏 ≤4」的可观测面,`?debug` 断言)。
    pub anim_groups: usize,
    /// 真相源文本总量(`Σ Store.parts[*].text.len()` 字节;历史规模代理)。Plan 18 §2.1 度量。
    pub store_chars: usize, // (Plan 19 per-phase 计时见 [`PhaseMs`],单列因 f32 不能 Eq)
    /// 驻留 PartView 数(`engine.views.len()`)。Plan 18 §2.1。
    pub retained_views: usize,
    /// **驻留逐字几何总量**(`Σ view.cache.placed.len()`,含离屏/已冻块)= 0029 主攻对象。Plan 18 §2.1。
    pub retained_glyphs: usize,
    /// 驻留节点树规模(`Σ view.cache.nodes.len()`)。Plan 18 §2.1。
    pub retained_nodes: usize,
    /// 工作集档位分布 `[Hot, Warm]`(Plan 19 P2)。Warm = 已释放几何的屏外块。
    pub tier_counts: [usize; 4],
    /// 本帧 `ensure_layouts` 重建块数(Plan 19 §2 thrash 监控;稳态应为 0)。
    pub rebuilds_this_frame: usize,
    /// 本帧发射的选区高亮 `FrameRect` 数(Plan 21 P2;host `stats().selRects` 验选区已上屏)。
    pub selection_rects: usize,
}

/// 一条**可见消息**(Plan 21 P1):每个屏上 Hot 块的复制单元 —— world 盒 + 渲染后纯文本。
/// host(wasm/web)据此每帧在每条消息角摆"复制"按钮(world→screen),点击写剪贴板。
/// **仅可见块**(虚拟化,0029 / Plan 19)→ 数量 ∝ 可见,不随历史涨。
#[derive(Clone, Debug, PartialEq)]
pub struct VisibleMessage {
    /// view 下标(= `block_seq`,append-only 稳定 → host 复用按钮,不每帧重建)。
    pub id: u32,
    /// 所属回合序(`group_turns` 下标)。
    pub turn: u32,
    /// 角色:true = user(右),false = assistant(左)。
    pub user: bool,
    /// 盒左上角 world 坐标。
    pub origin: [f32; 2],
    /// 盒宽 / 高(world px)。
    pub width: f32,
    pub height: f32,
    /// 该块**渲染后纯文本**(显示字形序列 join;markdown 噪声已在排版时去除)。
    pub text: String,
    /// 揭示已结算(全部字形上屏且内容冻结;Plan 25:host 据此做 meta「Complete 态才现」)。
    pub settled: bool,
}

/// 一行**可见文本 run**(Plan 21 P2):每个屏上 Hot 块逐行一个透明 span 的 world 盒 + 文本 +
/// `block`(view 下标)+ 起始字符偏移(块内显示字形序)。host 据此建虚拟透明文本层(原生选区/Cmd+F),
/// 并把 DOM 选区映回字符区间灌 `set_selection`。**仅可见块** → DOM 节点 ∝ 可见(0030 §7.1 硬约束)。
#[derive(Clone, Debug, PartialEq)]
pub struct VisibleTextRun {
    /// 所属块(view 下标 = `block_seq`)。
    pub block: u32,
    /// 行内首个显示字形在块内的下标(`cache.clusters`/`placed` 序)。
    pub char0: u32,
    /// 行左上角 world 坐标。
    pub origin: [f32; 2],
    /// 行宽 / 高(world px;高 = 行高)。
    pub width: f32,
    pub height: f32,
    /// 该行文本(显示字形 join,不含行尾换行)。
    pub text: String,
}

/// 每帧 per-phase 计时(Plan 19 §2:把「fps 归因」从断言变实测;ms)。单列因含 f32 不能进 `FrameStats`
/// 的 `Eq`。`build_frame` 内分三段(layout=group_turns+sizes+Taffy、grid 重建、emit 循环)+ advance。
#[derive(Clone, Copy, Debug, Default)]
pub struct PhaseMs {
    /// `advance`(ingest/reveal/ensure_layouts/schedule)总耗时。
    pub advance: f32,
    /// advance 子段(Plan 19 §2 二级归因)。
    pub adv_ingest: f32,
    pub adv_roles: f32,
    pub adv_reveal: f32,
    pub adv_ensure: f32,
    pub adv_schedule: f32,
    /// `build_frame` 内布局段:group_turns + sizes + `boxlayout::layout_chat`(Taffy)。
    pub bf_layout: f32,
    /// `build_frame` 内空间索引段:grid 清空 + 重建 + 视口查。
    pub bf_grid: f32,
    /// `build_frame` 内 emit 段:可见块 narrow-phase + 出 glyph/shaderbox。
    pub bf_emit: f32,
    /// `build_frame` 总耗时。
    pub bf_total: f32,
}

/// 每帧编排引擎。`C` 事件源、`L` 排版、`R` 渲染汇均经 seam 注入(CR2)。
/// 表格面板的可调渲染样式(0018 / Plan 6;web 层 style 面板实时改)。默认 = theme 常量。
/// `block_decorations` **每帧**读它产 `FramePanel` → setter 改完下一帧即生效(无需重排/reload)。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TableStyle {
    /// 网格线 / 外框色 RGBA。
    pub line_color: [f32; 4],
    /// 表头底色 RGBA。
    pub header_fill: [f32; 4],
    /// 网格线宽(px)。
    pub line_w: f32,
    /// AO 强度(0=无)。
    pub ao: f32,
    /// AO 颜色 RGB(暗色主题取白 → 向内辉光)。
    pub ao_color: [f32; 3],
    /// AO 向内淡出宽度(px)。
    pub ao_width: f32,
    /// 圆角半径(px)。
    pub radius: f32,
}

impl Default for TableStyle {
    fn default() -> Self {
        Self {
            line_color: theme::Theme::default().table_rule,
            header_fill: theme::Theme::default().table_header_bg,
            line_w: 1.0,
            // Plan 28 R3:参考表格无 AO 辉光、无圆角(markdown.css:极简横线结构)。
            ao: 0.0,
            ao_color: [1.0, 1.0, 1.0],
            ao_width: 10.0,
            radius: 0.0,
        }
    }
}

#[allow(clippy::struct_excessive_bools)] // reason: 编排引擎含若干独立调试/状态开关,非配置位域
pub struct Engine<C: Connection, L: LayoutEngine, R: RenderSink> {
    conn: C,
    layout: L,
    sink: R,
    store: Store,
    smoother: Smoother,
    views: Vec<PartView>,
    now_ms: f64,
    /// 本帧注入的 `dt_ms`(advance 时记;build_frame 的锚底平滑跟随用,fps 无关)。
    frame_dt: f64,
    max_width: f32,
    /// 只渲染该 session 的 part(`?session=`);None = 全渲染(Plan1 行为)。
    target_session: Option<String>,
    /// 2D 相机(Plan 3 L):平移 + 缩放。Plan2 的 1D scroll 收敛进 pan.y。
    camera: Camera2D,
    /// 滚动跟随 FSM(0038):Following/Released/Anchoring。
    follow: FollowState,
    /// 本帧「用户向下滚动」意图(0038 重进入:仅主动向下抵底才吸附;内容增长不算)。
    scroll_down_intent: bool,
    /// 锚底平滑跟随的垂直速度态(smooth-damp;0016 风格速度连续,消除换行 scroll 顿挫)。
    pan_vel_y: f32,
    /// CPU 空间索引(Plan 3 L):逐帧由块 AABB 重建,视口查可见块。
    grid: HeightIndex, // Plan 29 V1:一维高度区间索引(旧 SpatialGrid 每帧 O(N) HashMap 重建已删)
    /// 调试几何叠加(Plan 4C3):块 AABB / 视口框。
    debug_geometry: bool,
    /// 回合收尾跟踪(Phase I):多信号 + 看门狗,解决"忘了 idle 卡死"。
    turn: TurnTracker,
    /// 上一帧渲染统计(可观测)。
    last_stats: FrameStats,
    /// 表格面板可调渲染样式(web 层实时改;每帧读,见 [`TableStyle`])。
    table_style: TableStyle,
    /// 运行时视觉令牌(Plan 26①):装饰/选区/调试色。`set_theme` 换主题,下一帧生效
    /// (颜色不进排版缓存 → 不重排,与 0029 虚拟化正交)。默认 = 旧常量观感。
    theme: theme::Theme,
    /// 节奏/间距令牌(Plan 25 P0,design §3.1):时长喂 `FrameData.fade_ms`、间距喂
    /// `layout_chat`;`set_motion` 运行时改,下一帧生效。
    motion: crate::motion::MotionTokens,
    /// 设备像素比(HiDPI;world unit = 设备 px → 内容列上限 ×dpr;native 默认 1)。
    dpr: f32,
    /// 指针位置(屏幕 px;Plan 25 M2f hover/press:视觉反馈走 SDF 参数,0032 ①)。None=指针离开。
    pointer: Option<[f32; 2]>,
    /// 指针按下(press 视觉:卡片 scale 0.99,≤50ms 即时)。
    pointer_down: bool,
    /// 揭示调度器(0019 §4.3):**唯一**揭示路径,定每个 display 字形的 `spawn_time`(限速 /
    /// 放慢 / 骨架先行),与 token 到达解耦。
    scheduler: RevealScheduler,
    /// 数学每 em 的 world px(Plan 12):行内数学用它(贴正文字号),显示数学用 `× DISPLAY_MATH_SCALE`
    /// (H3 字号)。web 启动按正文字号(`FONT_SIZE`,含 DPR)`set_math_em` 注入,默认 32(retina 16px)。
    math_em: f32,
    /// 图片嵌入注册表(Plan 14 ③):key = `(block_seq<<32)|embed_idx`(append-only 稳定)→ [`Embed`]
    /// FSM。build_frame 据 `BlockCache.embeds` 补登(Placeholder);`take_pending_images` 交 JS 解码
    /// (转 Loading);`image_ready`/`image_failed` 回调推进;Ready 时该 key 出纹理 quad。
    image_registry: std::collections::HashMap<u64, crate::embed::Embed>,
    /// 代码块手动滚动态(Plan 15 ④):key=`(view<<32)|cb_idx` → `(scrollX px, scrollY 行, following)`。
    /// `following=true` = 跟随 tail(流式自动);用户滚 → false 脱离看历史;滚回底 → 复跟随。
    code_scroll: std::collections::HashMap<u64, (f32, i32, bool)>,
    /// 各代码块行窗的**世界命中矩形**(Plan 15 ④):build_frame 每帧重建;`code_block_at` 据此把指针
    /// 命中路由到块内滚动(命中则滚块、不滚画布)。
    code_hit_rects: Vec<(u64, Rect)>,
    /// diff 上下文折叠区展开态(Plan 32 D4):key=`(view<<32)|区序号`,存在即已展开。
    /// view 级持久;Cold 回收**不清**(GOAL:重建后保持 —— 不同于 code_scroll 的滚动位置语义)。
    diff_unfold: std::collections::HashSet<u64>,
    /// 展开中的 scale 包络(D4):区 key → 当前 scale;每帧注入 dt 指数逼近 1,snap 后移除
    /// (settled 恒等 AR3;map 空 = 零成本)。
    fold_anim: std::collections::HashMap<u64, f32>,
    /// 折叠汇总行的世界命中盒(D4):build_frame 每帧重建;tap 命中 → 展开。
    fold_targets: Vec<(u64, Rect)>,
    /// 退场 dissolve 时长 ms(Plan 36 N3):0 = off(默认,孤儿即时清除 == 旧行为恒等);
    /// >0 = 孤儿 view 先按 noise 阈值裁剪溶解 dur 毫秒再真清除(0016 exit 首租)。
    exit_dissolve_ms: f64,
    /// Spring 进场曲线开关(Plan 36 N4):off(默认)= 既有曲线恒等;on = 正文默认
    /// profile(id 0)换 spring profile(id 5),标题/表头 pop 不变。
    spring_enter: bool,
    /// 反馈通道衰减(0040/F1):0 = off(默认);用户设定值,发射前经收敛自停调制。
    feedback_decay: f32,
    /// 最近一次「画面活动」时刻(揭示释放/相机移动/退场中;收敛自停判据,R8 注入时钟)。
    last_activity_ms: f64,
    /// 上帧相机 pan(活动追踪探针)。
    last_pan_probe: [f32; 2],
    /// 后处理参数(0040/F2):[vignette, grain, chroma];全零 = off 恒等。
    post_params: [f32; 3],
    /// 注入帧计数(grain seed;R8 非墙钟)。
    frame_counter: u32,
    /// 行内链接的世界命中盒(Plan 34 S2):build_frame 每帧重建,**只收 settled 块**(AR3)。
    link_targets: Vec<(String, Rect)>,
    /// 待派发的链接打开请求(S2):core 不执行导航(CR5/0000 §2.2),tap 命中链接置此,
    /// 宿主层(wasm)取走后回调 onOpenUrl。
    pending_open_url: Option<String>,
    /// URL 白名单策略(S3,shadcn R12):链接派发/图片加载前的守门员;宿主可配置。
    url_policy: crate::UrlPolicy,
    /// 流式播报队列(S4,shadcn R13 / 0030 步骤3):**settled 的 part** 才入队(AR5 投影,
    /// 绝不逐 token);catch-up/瞬显不入队(AR6 零播报)。宿主(镜像层)每帧取走追加进
    /// aria-live 区。元素 = (角色 tag, 纯文本)。
    settle_announcements: Vec<(String, String)>,
    /// ShaderBox 动效节流时钟(Plan 16 护栏4):dynamic box 的 `time` 源,30fps 步进(与主 rAF 解耦)。
    shaderbox_clock: crate::shaderbox::ShaderboxClock,
    /// ShaderBox 画廊调试开关(Plan 16):开 → build_frame 在视口左上钉一格栅,逐格一个内置
    /// shader(50 icon + glow_orb + raymarch),供肉眼验全盘上屏。web `?gallery` 触发。
    shaderbox_gallery: bool,
    /// Plan 19 P1 A/B 开关(调试):true → `sizes` 退回每帧 fold `placed`(P1 前行为),用于同一
    /// 构建里对照 P1 缓存的 fps 收益(`?sizefold`)。默认 false(用缓存)。
    bench_fold_width: bool,
    /// 上帧 per-phase 计时(Plan 19 §2;`?bench` 读出归因)。
    last_phase_ms: PhaseMs,
    /// 本帧 `ensure_layouts` 重建块数(Plan 19 §2 thrash 监控)。
    last_rebuilds: usize,
    /// Plan 19 P2 虚拟化总开关(`?novirt` 关 = 全程 Hot,P2 前行为/兜底)。默认开。
    virtualize: bool,
    /// 上帧实际可见(narrow-phase 通过)的块:`(view 下标, turn 序, world origin, 盒宽, 高)`。
    /// Plan 21:`visible_messages()`/`visible_text_runs()` 据此只吐 **Hot 可见块**(虚拟化,DOM ∝ 可见,
    /// 不随历史涨)。每帧 `build_frame` 末重填;不参与渲染/录像 → 不破确定性(presentation 派生)。
    last_visible: Vec<(usize, u32, [f32; 2], f32, f32)>,
    /// 选区(Plan 21 P2):字符区间 `[(view 下标, start_char, end_char)]`(end 不含,块内显示字形序)。
    /// presentation 输入(同相机 pan),**不进 reveal / 不入录像** → 不破 R8 重放(0030 §7.6)。
    /// `build_frame` 据此查 `cache.placed` 发选区高亮 `FrameRect`(glyph 前),文字永在其上。
    selection: Vec<(usize, usize, usize)>,
    /// 会话生命周期态(Plan 22 P2/P4/P5):由 `ingest_events` 据事件 + `next_status` 驱动;派生
    /// 活跃指示 / 可发送 / 错误卡。**不进录像内容**(presentation/控制态)。
    session_status: crate::fsm::SessionStatus,
    /// 已停止冻结的消息 id 集(Plan 22 P4 / F11):丢弃针对冻结消息的后续 part 事件。
    frozen_messages: std::collections::HashSet<String>,
    /// 重连/对账 epoch(Plan 22 P5 / F12):stop/切会话 +1 → 陈旧异步回包据此丢弃(wasm 侧用)。
    epoch: u64,
    /// host 注入事件队列(Plan 22 P0 / 0031 §3):TS transport `push_event` 塞这里,`ingest_events`
    /// 与 `conn.poll()` 一并消费。**事件入口统一 = 录像入口**(transport 移 TS 不破重放)。
    inject: EventQueue,
    /// Plan 29 V2:是否存在 resync 源(真 server)。true 才允许 FrozenFar(释 Store text 后可经
    /// 0003 catch-up 重取);重放/bench 无源 → 封顶 Cold,保重放确定性。host(wasm)按连接态设。
    resync_available: bool,
    /// Plan 29 V3:FrozenFar 重入待重取标志(host 每帧轮询 `wants_resync` → TS 拉快照灌
    /// `resync_from_snapshot`;core 不做网络,0031 §3)。
    resync_requested: bool,
    /// (Plan 27 A 路)pending permission 按钮命中盒(世界坐标;每帧 build_frame 重建)。
    ask_targets: Vec<(String, Rect)>,
    /// 按压中的 ask 按钮 id(视觉深色;presentation,不入录像)。
    ask_pressed: Option<String>,
    /// Plan 23:part 渲染分派表(0033 契约)。reasoning/tool/compaction 走 specific 漂亮渲染器
    /// → StyledSpan;其余 kind 无 specific → 走 Plan 22 `display_source` markdown 兜底。纯函数(R8)。
    registry: crate::partrender::RenderRegistry,
}

impl<C: Connection, L: LayoutEngine, R: RenderSink> Engine<C, L, R> {
    /// `base_cps` 吐字基线(~200),`max_width` 排版宽度(px)。
    pub fn new(conn: C, layout: L, sink: R, base_cps: f64, max_width: f32) -> Self {
        Self {
            conn,
            layout,
            sink,
            store: Store::new(),
            smoother: Smoother::new(base_cps),
            views: Vec::new(),
            now_ms: 0.0,
            frame_dt: 0.0,
            max_width,
            target_session: None,
            camera: Camera2D::new(max_width, 600.0),
            follow: FollowState::Following,
            scroll_down_intent: false,
            pan_vel_y: 0.0,
            grid: HeightIndex::new(),
            debug_geometry: false,
            turn: TurnTracker::new(),
            last_stats: FrameStats::default(),
            table_style: TableStyle::default(),
            theme: theme::Theme::default(),
            motion: crate::motion::MotionTokens::default(),
            dpr: 1.0,
            pointer: None,
            pointer_down: false,
            scheduler: RevealScheduler::new(),
            math_em: 32.0,
            image_registry: std::collections::HashMap::new(),
            code_scroll: std::collections::HashMap::new(),
            code_hit_rects: Vec::new(),
            diff_unfold: std::collections::HashSet::new(),
            fold_anim: std::collections::HashMap::new(),
            fold_targets: Vec::new(),
            exit_dissolve_ms: 0.0,
            spring_enter: false,
            feedback_decay: 0.0,
            last_activity_ms: 0.0,
            last_pan_probe: [0.0, 0.0],
            post_params: [0.0; 3],
            frame_counter: 0,
            link_targets: Vec::new(),
            pending_open_url: None,
            url_policy: crate::UrlPolicy::default(),
            settle_announcements: Vec::new(),
            shaderbox_clock: crate::shaderbox::ShaderboxClock::new(),
            shaderbox_gallery: false,
            bench_fold_width: false,
            last_phase_ms: PhaseMs::default(),
            last_rebuilds: 0,
            virtualize: true,
            last_visible: Vec::new(),
            selection: Vec::new(),
            session_status: crate::fsm::SessionStatus::Idle,
            frozen_messages: std::collections::HashSet::new(),
            epoch: 0,
            inject: EventQueue::default(),
            registry: crate::partspecific::default_registry(),
            resync_available: false,
            resync_requested: false,
            ask_targets: Vec::new(),
            ask_pressed: None,
        }
    }

    /// 上帧 per-phase 计时(Plan 19 §2 归因)。
    pub fn phase_ms(&self) -> PhaseMs {
        self.last_phase_ms
    }

    /// Plan 19 P2 虚拟化开关(`?novirt` → false = 全程 Hot,不释放;对照/兜底)。
    pub fn set_virtualize(&mut self, on: bool) {
        self.virtualize = on;
    }

    /// Plan 21 P1:**可见消息**(每个屏上 Hot 块)→ world 盒 + 渲染纯文本。host 据此摆复制按钮。
    /// 读上帧 `last_visible`(build_frame 末填)+ 现 cache → 仅可见块,数量 ∝ 可见(虚拟化)。
    pub fn visible_messages(&self) -> Vec<VisibleMessage> {
        self.last_visible
            .iter()
            .filter_map(|&(id, turn, origin, w, h)| {
                let view = self.views.get(id)?;
                let cache = view.cache.as_ref()?; // Warm/已释放 → 跳过(只覆盖 Hot)
                Some(VisibleMessage {
                    id: id as u32,
                    turn,
                    user: view.role == crate::store::Role::User,
                    origin,
                    width: w,
                    height: h,
                    text: rendered_text(&cache.clusters),
                    settled: view.settled,
                })
            })
            .collect()
    }

    /// Plan 21 P2:**可见文本 run**(每个屏上 Hot 块逐行)→ world 盒 + 文本 + 块/字符偏移。
    /// host 据此建虚拟透明文本层(原生选区)。仅可见块 → DOM ∝ 可见(0030 §7.1 硬约束)。
    /// 行切分:按 `cache.clusters` 的 `"\n"` 分段;每段非空 run 取其字形并集盒(world = 块内相对 + origin)。
    pub fn visible_text_runs(&self) -> Vec<VisibleTextRun> {
        let mut runs = Vec::new();
        for &(id, _turn, origin, _w, _h) in &self.last_visible {
            let Some(view) = self.views.get(id) else {
                continue;
            };
            let Some(cache) = view.cache.as_ref() else {
                continue;
            };
            line_runs(cache, id as u32, origin, &mut runs);
        }
        runs
    }

    /// Plan 21 P2:设选区(host 把 DOM 选区映成字符区间灌入)。`ranges` = `[(view 下标, start, end)]`
    /// (end 不含,块内显示字形序)。**presentation 输入**(同相机 pan):只影响下帧选区高亮,
    /// **不进 reveal、不入录像** → 不破 R8 确定性重放(0030 §7.6)。空 vec = 清选区。
    pub fn set_selection(&mut self, ranges: Vec<(usize, usize, usize)>) {
        if !ranges.is_empty() {
            self.follow = FollowState::Released; // 0038:选区拖拽 = 释放信号(新增)
        }
        self.selection = ranges;
    }

    /// 当前选区(只读;测试/host 查询)。
    pub fn selection(&self) -> &[(usize, usize, usize)] {
        &self.selection
    }

    /// 每个 view 的布局叶子尺寸 `(内容宽, 高)`(Plan 13 §4 / Plan 19):Hot 非空读缓存、Warm 凭 agg 占位、
    /// 其余 `(0,0)`。`build_frame` 与 `scroll_to`(查任意块世界 y)共用,避免逻辑漂移。
    fn layout_sizes(&self) -> Vec<(f32, f32)> {
        self.views
            .iter()
            .map(|v| {
                if self.is_filtered(v) {
                    return (0.0, 0.0);
                }
                match (&v.cache, v.tier) {
                    (Some(c), _) if !c.placed.is_empty() => {
                        let w = if self.bench_fold_width {
                            c.placed
                                .iter()
                                .filter(|p| p.size[0] > 0.0)
                                .map(|p| p.pos[0] + p.size[0])
                                .fold(0.0f32, f32::max)
                        } else {
                            c.content_width
                        };
                        (w, c.height)
                    }
                    (None, Tier::Warm | Tier::Cold | Tier::FrozenFar) => match v.agg {
                        Some(a) => (a.content_width, a.height),
                        None => (0.0, 0.0),
                    },
                    _ => (0.0, 0.0),
                }
            })
            .collect()
    }

    /// 某 view 盒的世界 y(顶);跨**全历史**(含屏外 Warm 块,凭 agg 占位)→ Cmd+F 跳转用。
    fn view_world_y(&self, view: usize) -> Option<f32> {
        if view >= self.views.len() {
            return None;
        }
        let sizes = self.layout_sizes();
        let turns = group_turns(&self.views);
        let boxpos =
            crate::boxlayout::layout_chat(&turns, &sizes, self.max_width, &self.motion, self.dpr);
        boxpos.get(view).map(|b| b.origin[1])
    }

    /// Plan 21 P3:跳到某 view(Cmd+F 命中后):相机平移到该块顶(脱离锚底)。下帧虚拟化把它 promote
    /// 回 Hot → 可见 + 可逐字选。presentation(同相机),不破 R8。
    pub fn scroll_to(&mut self, view: usize) {
        if let Some(y) = self.view_world_y(view) {
            let x = self.camera.pan()[0];
            self.camera.set_pan(x, y.max(0.0));
            self.follow = FollowState::Released; // 0038:显式 jump = 释放
        }
    }

    /// Plan 21 P3:跨**全历史**全文查找(Store 源文本,含屏外/Warm 块)。返回每处命中
    /// `[(view 下标, 源文本 char 偏移)]`(文档序、块内升序)。大小写敏感、子串匹配;空 query → 空。
    /// host:`find` 定位 → `scroll_to` 跳转 → 块 promote 后据 `visible_text_runs` 精确选中(0030 §7.7)。
    pub fn find(&self, query: &str) -> Vec<(u32, u32)> {
        if query.is_empty() {
            return Vec::new();
        }
        let mut hits = Vec::new();
        for (vi, v) in self.views.iter().enumerate() {
            let Some(text) = self.store.part_text(&v.part_id) else {
                continue;
            };
            let mut from = 0usize; // byte 游标
            while let Some(rel) = text[from..].find(query) {
                let byte = from + rel;
                let char_off = text[..byte].chars().count() as u32;
                hits.push((vi as u32, char_off));
                from = byte + query.len().max(1); // 不重叠扫描
            }
        }
        hits
    }

    /// 开/关 ShaderBox 画廊调试视图(Plan 16):开后每帧在视口钉一格栅,逐格出一个内置 shader
    /// (50 icon + glow_orb + raymarch),不依赖任何会话内容 → 肉眼一屏验全盘 shader 上屏。
    pub fn set_shaderbox_gallery(&mut self, on: bool) {
        self.shaderbox_gallery = on;
    }

    /// Plan 19 P1 A/B(调试):true → `sizes` 退回每帧 fold(P1 前)。同构建对照 P1 fps 收益。
    pub fn set_bench_fold_width(&mut self, on: bool) {
        self.bench_fold_width = on;
    }

    /// 命中某代码块行窗的 world 点 → 该块 key(Plan 15 ④);未命中 None。web 输入层据此路由滚动。
    pub fn code_block_at(&self, world_x: f32, world_y: f32) -> Option<u64> {
        self.code_hit_rects
            .iter()
            .find(|(_, r)| r.contains(world_x, world_y))
            .map(|(k, _)| *k)
    }

    /// 块内滚动(Plan 15 ④):`dx` px 横滚、`dy_lines` 行纵滚(正=向下/看更新)。脱离 tail
    /// (`following=false`),clamp 留 build_frame(那里有行数/行宽)。滚回底由 build_frame 复跟随。
    pub fn scroll_code_block(&mut self, key: u64, dx: f32, dy_lines: i32) {
        let e = self.code_scroll.entry(key).or_insert((0.0, 0, true));
        e.0 += dx;
        e.1 += dy_lines;
        e.2 = false; // 用户滚 → 脱离 tail
    }

    /// 代码块滚动稳定 key(Plan 15 ④):`(view<<32)|cb_idx`。
    fn code_scroll_key(view: usize, cb_idx: usize) -> u64 {
        ((view as u64) << 32) | cb_idx as u64
    }

    /// 上一帧渲染统计(可观测;`?debug` 节流打印)。
    pub fn frame_stats(&self) -> FrameStats {
        self.last_stats
    }

    /// 第 `block_seq` 个 part(view)的内容节点树(0020 / Plan 7):下游 reveal(0019)/ embed
    /// (0022)/ 节点级 morph(0016)按 kind/区间/祖先查询的地基。块未排版时 None。
    pub fn block_nodes(&self, block_seq: usize) -> Option<&crate::nodes::NodeTree> {
        self.views
            .get(block_seq)
            .and_then(|v| v.cache.as_ref())
            .map(|c| &c.nodes)
    }

    /// 第 `block_seq` 个 part 的图片嵌入(Plan 14 ①):每个 `![alt](url)` 的 (占位区间, url, alt)。
    /// 下游(②④/JS)据此发起解码、Ready 时在该区间出纹理 quad。块未排版 → 空切片。
    pub fn block_embeds(&self, block_seq: usize) -> &[crate::EmbedRegion] {
        self.views
            .get(block_seq)
            .and_then(|v| v.cache.as_ref())
            .map_or(&[], |c| &c.embeds)
    }

    /// 嵌入稳定 key(Plan 14 ③):`(block_seq<<32)|embed_idx`(append-only ⇒ 跨帧稳定)。
    fn embed_key(block_seq: usize, embed_idx: usize) -> u64 {
        ((block_seq as u64) << 32) | embed_idx as u64
    }

    /// 把各块的图片嵌入补登进注册表(Placeholder;Plan 14 ③)。已登记的保留其 FSM 态(幂等)。
    /// build_frame 前调,保证新到的图有占位态、JS 可领取解码。
    fn sync_image_registry(&mut self) {
        for (vi, view) in self.views.iter().enumerate() {
            let Some(cache) = &view.cache else { continue };
            for (ei, region) in cache.embeds.iter().enumerate() {
                let key = Self::embed_key(vi, ei);
                // S3(shadcn R12):白名单外图片**不发起加载**,登记即 Failed(alt 兜底,
                // plan14 既有降级路;loader 永远拿不到该 url)。
                let allowed = self.url_policy.image_allowed(&region.url);
                self.image_registry.entry(key).or_insert_with(|| {
                    let mut e = crate::embed::Embed::new(&region.url, &region.alt);
                    if !allowed {
                        e.on_failed();
                    }
                    e
                });
            }
        }
    }

    /// 领取待解码图片(Plan 14 ③):Placeholder → Loading,返回 `(key, url)` 交 JS 解码上传。
    /// JS 完成后调 [`image_ready`](Self::image_ready) / [`image_failed`](Self::image_failed)。
    pub fn take_pending_images(&mut self) -> Vec<(u64, String)> {
        let mut out = Vec::new();
        for (&key, e) in &mut self.image_registry {
            if e.state == crate::embed::EmbedState::Placeholder {
                e.begin_loading();
                out.push((key, e.url.clone()));
            }
        }
        out
    }

    /// JS 解码 + 纹理上传完成(Plan 14 ③):推进 `key` 的嵌入到 Ready(记 tex_id/自然尺寸/动图标志)。
    pub fn image_ready(&mut self, key: u64, tex_id: u32, w: f32, h: f32, animated: bool) {
        let now = self.now_ms as f32;
        if let Some(e) = self.image_registry.get_mut(&key) {
            e.on_ready(tex_id, w, h, animated, now);
        }
    }

    /// JS 解码/网络失败(Plan 14 ③):`key` 的嵌入 → Failed(显 alt 兜底)。
    pub fn image_failed(&mut self, key: u64) {
        if let Some(e) = self.image_registry.get_mut(&key) {
            e.on_failed();
        }
    }

    /// 开关调试几何叠加(块 AABB / 视口框,Plan 4C3)。
    pub fn set_debug_geometry(&mut self, on: bool) {
        self.debug_geometry = on;
    }

    /// 当前回合收尾状态(供宿主显示 loading / 收尾,Phase I)。
    pub fn turn_status(&self) -> TurnStatus {
        self.turn.status()
    }

    /// 设视口高度(画布尺寸变化时);视口裁剪与锚底据此。
    pub fn set_viewport_height(&mut self, height: f32) {
        let w = self.camera.viewport()[0];
        self.camera.set_viewport(w, height);
    }

    /// 滚动 `dy` 屏幕像素(正 = 向下/看更新内容)= 相机平移。向上滚脱离锚底,滚回底部自动跟随。
    pub fn scroll_by(&mut self, dy: f32) {
        self.camera.pan_by_screen(0.0, dy);
        if dy < 0.0 {
            self.follow = FollowState::Released; // 0038:上滚 = 释放信号
        } else if dy > 0.0 {
            self.scroll_down_intent = true; // 主动向下:build_frame 抵底才吸附
        }
    }

    /// 二维平移 `dx,dy` 屏幕像素(触摸板两指滚动 / 拖拽;web 层输入统一入口)。横向自由平移(宽表
    /// 溢出可拖看),纵向同 `scroll_by`。任意横移或上移即脱离锚底,滚回底部时 `build_frame` 复跟随。
    pub fn pan_by(&mut self, dx: f32, dy: f32) {
        self.camera.pan_by_screen(dx, dy);
        if dx != 0.0 || dy < 0.0 {
            self.follow = FollowState::Released; // 0038:横移/上滚 = 释放(触摸/滚动条同入口)
        } else if dy > 0.0 {
            self.scroll_down_intent = true;
        }
    }

    /// 围绕屏幕点缩放(Plan 3 L:ctrl+滚轮 / 双指)。缩放即脱离锚底。
    pub fn zoom_by(&mut self, factor: f32, screen_x: f32, screen_y: f32) {
        self.camera.zoom_at(factor, screen_x, screen_y);
        self.follow = FollowState::Released; // 0038:缩放 = 释放
    }

    /// 跳到最新(0038:jump-to-latest 正路)—— Anchoring 平滑滚向底部,到底转 Following。
    pub fn scroll_to_latest(&mut self) {
        self.follow = FollowState::Anchoring;
    }

    /// 只读跟随态(宿主画 jump 按钮 / e2e 断言)。
    #[must_use]
    pub fn follow_state(&self) -> FollowState {
        self.follow
    }

    /// 只读相机(供宿主/测试)。
    pub fn camera(&self) -> &Camera2D {
        &self.camera
    }

    /// 设表格面板渲染样式(web 层 style 面板调用)。**无需重排**:`block_decorations` 每帧读它,
    /// 下一帧即生效。
    pub fn set_table_style(&mut self, s: TableStyle) {
        self.table_style = s;
    }

    /// 设运行时主题(Plan 26①:装饰/选区/调试全部视觉令牌)。**无需重排**:颜色不进排版缓存,
    /// `build_frame` emit 时才解析 → 下一帧生效。默认主题 = 旧常量观感(零回归)。
    pub fn set_theme(&mut self, t: theme::Theme) {
        self.theme = t;
    }

    /// 当前主题(只读;测试/面板)。
    pub fn theme(&self) -> &theme::Theme {
        &self.theme
    }

    /// 设 MotionTokens(Plan 25 P0:节奏/间距令牌)。**无需 refresh_fonts**:时长走
    /// `FrameData.fade_ms`(下一帧生效),间距由 `layout_chat` 每帧读(下帧盒位变,
    /// 0016 morph 平滑收口)。
    pub fn set_motion(&mut self, m: crate::motion::MotionTokens) {
        self.motion = m;
    }

    /// 当前 MotionTokens(只读;测试/面板)。
    pub fn motion(&self) -> &crate::motion::MotionTokens {
        &self.motion
    }

    /// 设设备像素比(web 启动/resize 时注入;内容列上限按其缩放)。≤0 忽略。
    pub fn set_dpr(&mut self, d: f32) {
        if d > 0.0 {
            self.dpr = d;
        }
    }

    /// 设指针位置(屏幕 px;M2f hover 反馈)。None = 指针离开画布。presentation-only,不写 store。
    pub fn set_pointer(&mut self, p: Option<[f32; 2]>) {
        self.pointer = p;
    }

    /// 设指针按下态(M2f press 反馈)。
    pub fn set_pointer_down(&mut self, down: bool) {
        self.pointer_down = down;
    }

    /// 设过滤目标 session(`?session=`);None 全渲染。
    pub fn set_target_session(&mut self, session: Option<String>) {
        self.target_session = session;
    }

    /// 用快照历史预热(Phase F catch-up):灌入 store + 直接整段上屏(零淡入,AR6)。
    /// `raw` 为 `GET /session/{id}/message` 的响应原文。
    pub fn prime_from_snapshot(&mut self, raw: &str) {
        let messages = match parse_snapshot(raw) {
            Ok(m) => m,
            Err(e) => {
                tracing::warn!(target: "M2", error = %e, "快照解析失败,跳过 catch-up");
                return;
            }
        };
        self.store.apply_snapshot(&messages);
        for msg in &messages {
            for tp in &msg.text_parts {
                let revealed: Vec<(String, f32)> = graphemes(&tp.text)
                    .into_iter()
                    .map(|g| (g.to_owned(), CATCHUP_SPAWN))
                    .collect();
                let view = self.view_mut(&tp.part_id);
                view.pushed = revealed.len();
                view.revealed = revealed;
                view.instant = true; // 历史块整段瞬显(零淡入,AR6),绕过揭示时钟。
            }
        }
        tracing::info!(target: "M3", n = messages.len(), "快照 catch-up 灌入");
    }

    /// 重连/周期性对账(Phase J):只补 store 里**还没有的 part**(恢复连接间隙错过的历史),
    /// 不动正在 live 的块,避免闪烁/回退(0003 §3.4)。已知 part 的差异交由 `part.updated`
    /// 对账(AR4)。
    /// Plan 29:声明 resync 源可用(真 server 连接)→ 允许 FrozenFar 档(释 Store text)。
    pub fn set_resync_available(&mut self, on: bool) {
        self.resync_available = on;
    }

    /// Plan 29 V3:FrozenFar 块滚回 → 请求重取(host 轮询;取走即清)。
    fn request_resync(&mut self) {
        self.resync_requested = true;
    }

    /// host 每帧轮询:是否需要拉快照重灌(FrozenFar 重入)。读后清(边沿触发)。
    pub fn wants_resync(&mut self) -> bool {
        std::mem::take(&mut self.resync_requested)
    }

    pub fn resync_from_snapshot(&mut self, raw: &str) {
        let messages = match parse_snapshot(raw) {
            Ok(m) => m,
            Err(e) => {
                tracing::warn!(target: "M2", error = %e, "resync 快照解析失败");
                return;
            }
        };
        // 过滤出 store 未知的 part。
        let fresh: Vec<crate::protocol::SnapshotMessage> = messages
            .into_iter()
            .filter_map(|m| {
                let new_parts: Vec<_> = m
                    .text_parts
                    .into_iter()
                    .filter(|tp| self.store.part_text(&tp.part_id).is_none())
                    .collect();
                if new_parts.is_empty() {
                    None
                } else {
                    Some(crate::protocol::SnapshotMessage {
                        text_parts: new_parts,
                        ..m
                    })
                }
            })
            .collect();
        if fresh.is_empty() {
            return;
        }
        self.store.apply_snapshot(&fresh);
        for msg in &fresh {
            for tp in &msg.text_parts {
                let revealed: Vec<(String, f32)> = graphemes(&tp.text)
                    .into_iter()
                    .map(|g| (g.to_owned(), CATCHUP_SPAWN))
                    .collect();
                let view = self.view_mut(&tp.part_id);
                view.pushed = revealed.len();
                view.revealed = revealed;
                view.instant = true; // 错过的历史块同样瞬显,不参与揭示时钟。
            }
        }
        tracing::info!(target: "M3", n = fresh.len(), "resync 补入错过的历史");
    }

    /// 只读访问 store(供对账/断言,R4)。
    pub fn store(&self) -> &Store {
        &self.store
    }

    /// 只读访问渲染汇(供测试断言末帧内容,R4)。
    pub fn sink(&self) -> &R {
        &self.sink
    }

    /// 可变访问渲染汇(供宿主在 resize 时直驱后端重配 surface)。
    pub fn sink_mut(&mut self) -> &mut R {
        &mut self.sink
    }

    /// 更新排版宽度 + 相机视口宽(画布尺寸变化时)。
    pub fn set_max_width(&mut self, max_width: f32) {
        self.max_width = max_width;
        let h = self.camera.viewport()[1];
        self.camera.set_viewport(max_width, h);
    }

    /// 作废所有块的排版缓存,强制下一帧全量重排。
    ///
    /// 字体切换等改变字形宽度但宽度/字数不变的场景:块冻结的脏判据(`revealed_len`/`width`)
    /// 不会自动触发,故显式作废(Plan 4C 调试器换字体用)。
    pub fn mark_layout_dirty(&mut self) {
        for v in &mut self.views {
            v.cache = None;
        }
    }

    /// 推进一帧。串:收事件→落 store→整流到达(smoother)→排版→**揭示调度**(0019)→组帧。
    pub fn frame(&mut self, dt_ms: f64) {
        let t0 = web_time::Instant::now();
        self.advance(dt_ms);
        self.last_phase_ms.advance = t0.elapsed().as_secs_f32() * 1000.0; // Plan 19 §2 归因
        self.render_now();
    }

    /// 推进**模拟**一帧(不出图):时钟 + 事件摄入 + 到达整流 + 排版 + 揭示调度。与 [`render_now`]
    /// 拆分,使 [`seek_reveal`] 可低成本快进(多步只推模拟、末尾出一帧),避免每微步都提交 GPU。
    fn advance(&mut self, dt_ms: f64) {
        self.now_ms += dt_ms;
        self.frame_dt = dt_ms; // 锚底平滑跟随用(build_frame)
        self.frame_counter = self.frame_counter.wrapping_add(1); // F2 grain seed(R8)
        self.shaderbox_clock.tick(dt_ms as f32); // Plan 16 护栏4:动效时钟 30fps 步进
        self.turn.tick(self.now_ms);
        let mk = |t: web_time::Instant| t.elapsed().as_secs_f32() * 1000.0;
        let t = web_time::Instant::now();
        self.ingest_events();
        self.enqueue_new_text();
        self.last_phase_ms.adv_ingest = mk(t);
        let t = web_time::Instant::now();
        self.refresh_roles(); // Plan 13:角色可能 snapshot/resync 后才知 → 每帧从 store 校正(便宜)
        self.last_phase_ms.adv_roles = mk(t);
        let t = web_time::Instant::now();
        // F1(0040):画面活动追踪(收敛自停判据)——有未 settled 视图/退场中/折叠动画
        // 即视为活动;相机移动在 scroll/pan 入口已隐含(下帧内容位移 → 未 settled 不必然,
        // 故 pan 变化单独记)。R8:全部经注入时钟。
        {
            let busy = self
                .views
                .iter()
                .any(|v| (!v.settled && v.cache.is_some()) || v.exiting.is_some())
                || !self.fold_anim.is_empty();
            let pan = self.camera.pan();
            let moved = (pan[0] - self.last_pan_probe[0]).abs() > 0.01
                || (pan[1] - self.last_pan_probe[1]).abs() > 0.01;
            self.last_pan_probe = pan;
            if busy || moved {
                self.last_activity_ms = self.now_ms;
            }
        }
        // D4:折叠展开 scale 包络推进(注入 dt,R8);收敛 snap 即移除(恒等 AR3,map 空零成本)。
        self.fold_anim.retain(|_, sc| {
            *sc = fold_scale_step(*sc, dt_ms as f32);
            *sc < 1.0
        });
        self.reveal(dt_ms); // smoother:token 突发 → 匀速到达(内容真值)
        self.last_phase_ms.adv_reveal = mk(t);
        let t = web_time::Instant::now();
        self.ensure_layouts(); // 块冻结排版 → display 字形 + 节点树就绪
        self.last_phase_ms.adv_ensure = mk(t);
        let t = web_time::Instant::now();
        self.schedule(dt_ms); // 调度器:按风格/门/时钟释放 display 字形,定 spawn_time(唯一揭示路径)
        self.last_phase_ms.adv_schedule = mk(t);
    }

    /// 用当前状态出一帧并提交(不推进时钟)。
    fn render_now(&mut self) {
        let frame = self.build_frame();
        self.sink.submit(&frame);
    }

    /// 重放**揭示**动画到时间轴 `target_ms`(调试播放器拖拽用):清空 spawn 后按固定步长把揭示
    /// 模拟从头推进到 `target_ms`,末尾出一帧。内容已加载(冻结块)时只重跑揭示(0019),确定性
    /// 可重复(同 `target_ms` → 同画面);揭示节奏由当前 `reveal_cps`/`slow` 决定(播放器设固定基速)。
    pub fn seek_reveal(&mut self, target_ms: f64) {
        self.restart_reveal();
        let step: f64 = 16.0;
        let mut t = 0.0;
        while t < target_ms {
            self.advance(step.min(target_ms - t));
            t += step;
        }
        self.render_now();
    }

    /// 设揭示速率上限(glyph/秒);≤0 / 非有限 = 不限速(跟内容到达,默认)。web 调试面板调。
    pub fn set_reveal_cps(&mut self, cps: f32) {
        self.scheduler.set_reveal_cps(cps);
    }

    /// 设揭示放慢因子(`[0.01,1.0]`,越小越慢;0019 北极星"刻意放慢")。web 调试面板调。
    pub fn set_reveal_slow(&mut self, slow: f32) {
        self.scheduler.set_slow(slow);
    }

    /// 切节奏预设(Plan 25 M1:typewriter/reader/flow,design §3.2)。整表覆盖六旋钮。
    pub fn set_reveal_preset(&mut self, p: crate::rhythm::RhythmPreset) {
        self.scheduler.set_preset(p);
    }

    /// 设节奏参数(六旋钮整表;wasm `set_rhythm` 局部覆盖后传入)。
    pub fn set_rhythm(&mut self, p: crate::rhythm::RhythmParams) {
        self.scheduler.set_params(p);
    }

    /// 当前节奏参数(面板/测试读)。
    pub fn rhythm(&self) -> &crate::rhythm::RhythmParams {
        self.scheduler.params()
    }

    /// 设到达整流基线吐字速率(Plan 18 `?bench`:调极大值让长会话即时载满,测稳态规模/内存)。
    pub fn set_stream_rate(&mut self, cps: f64) {
        self.smoother.set_base_cps(cps);
    }

    /// 设数学每 em 的 world px(Plan 12):= 正文字号(含 DPR)。行内数学贴此字号,显示数学 ×1.3(H3)。
    /// web 启动按 `FONT_SIZE` 注入,使公式与正文同尺度(根治"公式太小")。
    pub fn set_math_em(&mut self, px: f32) {
        if px > 0.0 {
            self.math_em = px;
        }
    }

    /// 设表格揭示风格(0=Raw / 1=RowFrame / 2=Full;0019 §2 三风格)。web 下拉调。
    pub fn set_table_reveal_style(&mut self, style: u32) {
        self.scheduler
            .set_table_style(TableStyleKind::from_u32(style));
    }

    /// 重放揭示动画(调试):清空各非瞬显视图的 `spawn` → 下一帧起调度器按**当前**风格/速度
    /// 从头再揭示一遍。用于"改了风格/速度想立刻看到效果":内容已全部上屏(冻结)时,改设置
    /// 本身没有待揭的字,故需主动重启;`set_table_reveal_style`/`set_reveal_*` 后调此即可见效。
    pub fn restart_reveal(&mut self) {
        for view in &mut self.views {
            if view.instant {
                continue; // 历史瞬显块不参与揭示动画
            }
            for s in &mut view.spawn {
                *s = None;
            }
            view.settled = false; // 解冻 → schedule 重新从头揭示
        }
        self.scheduler.idle_reset();
    }

    /// 当前表格揭示风格的数值(0/1/2)。
    pub fn table_reveal_style(&self) -> u32 {
        self.scheduler.table_style() as u32
    }

    /// 揭示调度(0019 §4.3 / Plan 9):**唯一**揭示路径。在 0020 嵌套集上**递归**排程
    /// ([`reveal::resolve_tree`]):tier = 顶层块文档序(块间自上而下、不抢位),delay_ms =
    /// 每容器 ordering 累加的编排时序;用调度器时钟(限速/放慢/可重放)按 (tier, 序) 释放尚未
    /// 上屏的 display 字形,定其 `spawn_time = 释放时刻 + delay`(骨架先行:结构块字带 delay,
    /// 晚于即时入场的容器底/框)。瞬显块(catch-up)整段以 catch-up spawn 释放,绕过时钟。
    fn schedule(&mut self, dt_ms: f64) {
        self.scheduler.advance_clock(dt_ms);
        // Plan 25 M1:毫秒赤字预算 + 节奏成本表(rhythm::cost_table)。释放判据 budget≥0、扣整段
        // 成本(可透支 → 长停顿等足、不死锁);不限速走短路(全部释放,零成本计算)。
        let limited = self.scheduler.is_rate_limited();
        let mut budget = self.scheduler.budget_ms();
        let rhythm_params = *self.scheduler.params();
        let tempo = self.scheduler.effective_tempo_ms();
        let stagger_glyph = self.motion.stagger_glyph;
        let now = self.now_ms as f32;
        let table_style = self.scheduler.table_style();
        let mut had_candidates = false; // 有待揭字但被限速挡住 → 不清预算(攒着下帧揭)
                                        // 9F 内容门:末块在 turn 未收尾前视为"仍在流入(未闭合)";整表风格据此 hold 开放的 Full 表。
        let turn_open = self.turn.status() != TurnStatus::Settled;
        let last_view = self.views.len().saturating_sub(1);
        for vi in 0..self.views.len() {
            let view = &mut self.views[vi];
            let Some(cache) = &view.cache else { continue };
            let gcount = cache.clusters.len();
            // spawn 表与 display 字形对齐(reparse 增长则补 None,收缩则截断);已释放的保留(append 稳定)。
            if view.spawn.len() != gcount {
                view.spawn.resize(gcount, None);
                view.settled = false; // 内容增长 → 解冻,重新揭示/重算
            }
            if gcount == 0 {
                continue;
            }
            // 瞬显:历史块整段一帧上屏(catch-up spawn,零淡入),不走时钟。
            if view.instant {
                for s in &mut view.spawn {
                    if s.is_none() {
                        *s = Some(CATCHUP_SPAWN);
                    }
                }
                view.settled = true; // 整段一帧结算 → 此后 O(1) 跳过
                continue;
            }
            // 已结算 → O(1) 跳过:不扫 spawn、不重 resolve(性能命脉,0025 §4;修 Plan 9 #1)。
            if view.settled {
                continue;
            }
            // 风格 → 逐 glyph tier/offset(0019 §4.2 在 0020 节点树上落地)。**逐顶层块**各用自身
            // 风格(标题/段落逐字、表格走风格、代码/列表/引用骨架),避免含表格的消息把整条当单块
            // → 表格后的标题/段落被连坐永久 hold(c06-all 段间空白根因)。
            // 末块(该视图最后一个顶层块)若属仍在流入的活动视图 → 开放未闭合(9F 内容门)。
            let open_block = if vi == last_view && turn_open {
                cache.nodes.children(0).last()
            } else {
                None
            };
            let plan = reveal::resolve_tree(&cache.nodes, table_style, open_block);
            // 候选 = 已揭示(非 hold)且尚未上屏且非零墨换行;按 (tier, 序) 排 → 骨架/表头先于 body。
            let mut cand: Vec<usize> = (0..gcount)
                .filter(|&g| {
                    plan.revealed(g) && view.spawn[g].is_none() && cache.clusters[g] != "\n"
                })
                .collect();
            cand.sort_by_key(|&g| (plan.tier[g], g));
            had_candidates |= !cand.is_empty();
            if limited {
                // 节奏成本表(M1):词首/CJK 字非零成本,词内/标点/空白 0(整词一拍、停顿吸附)。
                // seed = view 下标(块级种子,R8)。
                let costs = crate::rhythm::cost_table(
                    &cache.clusters,
                    &cache.roles,
                    &cache.nodes,
                    &rhythm_params,
                    tempo,
                    vi as u64,
                );
                // M2e 词内 stagger(design §3.3 装饰层):同拍释放的词内 glyph,spawn 依次
                // +k×stagger.glyph(词内 20–35ms 波浪)——整词同帧释放(内容层),视觉上
                // 字符相继淡入(装饰层);token 置 0 即关(旋钮)。
                let stagger = stagger_glyph;
                let title_role = crate::StyleRole::ToolTitle.as_u32();
                let mut word_k = 0f32;
                for g in cand {
                    if budget < 0.0 {
                        break;
                    }
                    let c = costs.get(g).copied().unwrap_or(0.0);
                    // 卡片标题整行同帧(一个单元一次 enter,零词内波浪——逐字闪即用户所诉);
                    // 其余照词拍:词首归零、词内 +k×stagger。
                    if c > 0.0 || cache.roles.get(g).copied() == Some(title_role) {
                        word_k = 0.0;
                    } else {
                        word_k += 1.0;
                    }
                    budget -= c;
                    view.spawn[g] = Some(now + plan.delay_ms[g] + word_k * stagger);
                }
            } else {
                for g in cand {
                    view.spawn[g] = Some(now + plan.delay_ms[g]);
                    // F1(0040):字形释放 = 画面活动(收敛自停判据;1e9 cps 瞬释也命中)。
                    self.last_activity_ms = self.now_ms;
                }
            }
            // 结算判定:所有**非换行**字已释放(换行永不 spawn,故不阻塞冻结 —— 修 Plan 9 #1:否则
            // 含换行/NodeSpawn 的活动 view 永不冻结、每帧重 resolve)。settled 后下帧起 O(1) 跳过。
            if (0..gcount).all(|g| view.spawn[g].is_some() || cache.clusters[g] == "\n") {
                view.settled = true;
                // S4:流式完成的 part 入播报队列(瞬显/catch-up 不经此路 → AR6 零播报)。
                let role = format!("{:?}", self.store.part_role(&view.part_id)).to_lowercase();
                let text: String = cache.clusters.concat();
                self.settle_announcements.push((role, text));
            }
        }
        self.scheduler.set_budget_ms(budget);
        // 真正空闲(无任何待揭字)才没收正余额,避免空转后突发;限速挡住时保留预算攒到下帧。
        if !had_candidates {
            self.scheduler.idle_reset();
        }
    }

    /// 收事件 → 解码 → 落 store(含 updated 对账,AR4)。
    ///
    /// 事件源 = `conn.poll()`(Player/SSE/synthetic)＋ host 注入队列(Plan 22 P0:TS transport
    /// `push_event` / 测试注入)→ 统一一条解码路径。
    fn ingest_events(&mut self) {
        let mut raws = self.conn.poll();
        raws.extend(self.inject.borrow_mut().drain(..));
        for raw in raws {
            match decode(raw.raw()) {
                Ok(Event::PartDelta {
                    part_id,
                    message_id,
                    field,
                    delta,
                    ..
                }) => {
                    if self.frozen_messages.contains(&message_id) {
                        continue; // F11:停止后丢弃针对冻结消息的事件
                    }
                    self.store
                        .apply_delta(&part_id, &message_id, &field, &delta);
                    self.on_part_activity(&message_id);
                }
                Ok(Event::PartUpdated { part, .. }) => {
                    let mid = part_message_id(&part);
                    if self.frozen_messages.contains(&mid) {
                        continue;
                    }
                    self.store.apply_part_updated(&part);
                    self.on_part_activity(&mid);
                }
                // 会话状态:idle/完成 → 收尾信号 + FSM Idle;busy → 活跃;retry → Retrying(F8)。
                Ok(Event::SessionStatus { status }) => match status.as_str() {
                    "idle" => {
                        // F8:已发送(awaiting)却末条无任何回复/错误 → 注兜底错误卡(别静默卡死)。
                        let awaiting =
                            matches!(self.session_status, SessionStatus::AwaitingAck { .. });
                        if crate::resilience::should_bottom_out(awaiting, false, false) {
                            self.store
                                .upsert_error_card("", "请求结束但未收到回复(超时或服务端错误)");
                            self.drive_fsm(&FsmInput::Error {
                                error: "no response".to_owned(),
                            });
                        } else {
                            self.turn.on_settle_signal();
                            self.store.clear_thinking_row();
                            self.drive_fsm(&FsmInput::Idle);
                        }
                    }
                    "retry" => {
                        self.turn.on_busy();
                        self.drive_fsm(&FsmInput::Retry { attempt: 0 });
                    }
                    "busy" | "working" => {
                        self.turn.on_busy();
                        self.drive_fsm(&FsmInput::Busy);
                    }
                    _ => {}
                },
                Ok(Event::MessageUpdated {
                    message_id,
                    role,
                    session_id,
                    error,
                }) => {
                    self.store.set_message_role(&message_id, &role, &session_id);
                    if let Some(msg) = error {
                        // F4:错误终止 → 合成错误卡(恒一张)+ FSM Errored。
                        self.store.upsert_error_card(&session_id, &msg);
                        self.drive_fsm(&FsmInput::Error { error: msg });
                    } else {
                        self.on_part_activity(&message_id);
                    }
                }
                // 删 part(P1 承载):从三表移除。
                Ok(Event::PartRemoved { part_id, .. }) => {
                    self.store.apply_part_removed(&part_id);
                    self.turn.on_activity(self.now_ms);
                }
                // 会话级错误(F3 ghost-abort / F9 配额 / 错误卡)。
                Ok(Event::SessionError {
                    session_id, name, ..
                }) => {
                    if name.contains("Abort") {
                        // F3 ghost-abort:不弹错误卡,回 Idle(temp 删除属 TS/P5)。
                        self.store.clear_error_card();
                        self.store.clear_thinking_row();
                        self.drive_fsm(&FsmInput::Idle);
                    } else {
                        // F9:配额/限流类错误 → 标注(host 据 status 弹配额 Dock / 发送前预检)。
                        let msg = if crate::resilience::is_quota_error(&name) {
                            format!("额度/配额受限:{name}")
                        } else if name.is_empty() {
                            "会话错误".to_owned()
                        } else {
                            name
                        };
                        self.store.upsert_error_card(&session_id, &msg);
                        self.drive_fsm(&FsmInput::Error { error: msg });
                    }
                }
                // 权限 / 提问(Plan 27):追加**流内 ask 块**(占位/滚动/虚拟化,应答后原位落定)
                // + FSM 阻塞(0031 语义不变,变的只是呈现位置)。
                Ok(Event::PermissionAsked {
                    session_id,
                    payload,
                }) => {
                    let (prompt, _) = ask_fields(&payload, "工具请求权限");
                    self.store.clear_thinking_row(); // ask 卡顶上,等待行撤
                    self.store
                        .push_ask(AskKind::Permission, &prompt, Vec::new(), &session_id);
                    self.drive_fsm(&FsmInput::PermissionAsked);
                }
                Ok(Event::QuestionAsked {
                    session_id,
                    payload,
                }) => {
                    let (prompt, options) = ask_fields(&payload, "助手有一个问题");
                    self.store.clear_thinking_row();
                    self.store
                        .push_ask(AskKind::Question, &prompt, options, &session_id);
                    self.drive_fsm(&FsmInput::QuestionAsked);
                }
                // 服务端回执(他端应答):尽力落定本地 pending 块 + 解阻。
                Ok(Event::PermissionReplied { .. }) => {
                    let _ = self
                        .store
                        .answer_pending_ask(AskAnswer::Permission { allow: true });
                    self.drive_fsm(&FsmInput::Replied);
                }
                Ok(Event::QuestionReplied { .. }) => {
                    let _ = self.store.answer_pending_ask(AskAnswer::Question {
                        options: Vec::new(),
                        text: String::new(),
                    });
                    self.drive_fsm(&FsmInput::Replied);
                }
                Ok(Event::QuestionRejected { .. }) => {
                    let _ = self.store.answer_pending_ask(AskAnswer::Question {
                        options: Vec::new(),
                        text: "(已拒答)".to_owned(),
                    });
                    self.store.clear_thinking_row();
                    self.drive_fsm(&FsmInput::Idle);
                }
                // 子会话 / 元信息 / 压缩 / 握手 / 心跳 / 实例销毁 / 未知:暂不改文档(TS/P5 resync 处理)。
                Ok(_) => {}
                Err(e) => {
                    tracing::warn!(target: "M2", error = %e, "丢弃无法解码的事件");
                }
            }
        }
    }

    /// 驱动会话 FSM(Plan 22 P2/P4):`next_status` 纯转移,集中一处。
    fn drive_fsm(&mut self, input: &FsmInput) {
        self.session_status = next_status(&self.session_status, input);
    }

    /// part 活动(delta/updated 真内容到达):刷新收尾时钟 + 清陈旧合成错误卡(F4)+ 推进 FSM
    /// (等待态首包 → Streaming,否则 Activity 复活)。
    fn on_part_activity(&mut self, message_id: &str) {
        self.turn.on_activity(self.now_ms);
        if self.store.has_error_card() {
            self.store.clear_error_card(); // F4:真回复到 → 清陈旧合成卡(恒一张)
        }
        self.store.clear_thinking_row(); // Plan 28:首包即撤等待指示行
        let input = if matches!(self.session_status, SessionStatus::AwaitingAck { .. }) {
            FsmInput::FirstPart {
                message_id: message_id.to_owned(),
            }
        } else {
            FsmInput::Activity
        };
        self.drive_fsm(&input);
    }

    /// 用户发送(Plan 22 P2:host 发消息前调)→ FSM AwaitingAck(起 no-reply 计时,F1)。
    pub fn note_send(&mut self) {
        // Plan 28 遗留-1:发送后、首包前 → Thinking 状态行(参考 session-turn-thinking;
        // 事件驱动 upsert/clear,无墙钟,R8/录像安全)。
        let sid = self.target_session.clone().unwrap_or_default();
        self.store.upsert_thinking_row(&sid);
        self.drive_fsm(&FsmInput::Send {
            now_ms: self.now_ms,
        });
    }

    /// 用户停止(Plan 22 P4 / F11):FSM Stopped + 冻结当前流式消息(丢弃其后续事件)+ epoch+1。
    pub fn stop(&mut self) {
        self.store.clear_thinking_row(); // Plan 28:停止 → 等待指示行撤

        if let Some(mid) = self.session_status.streaming_id() {
            if !mid.is_empty() {
                self.frozen_messages.insert(mid.to_owned());
            }
        }
        self.epoch += 1;
        self.drive_fsm(&FsmInput::Stop);
    }

    /// 当前会话生命周期态(可观测;host 据此画活跃指示/禁发送/弹 Dock)。
    #[must_use]
    pub fn session_status(&self) -> &SessionStatus {
        &self.session_status
    }

    /// 重连/对账 epoch(Plan 22 P5 / F12:陈旧异步回包据此丢弃)。
    #[must_use]
    pub fn epoch(&self) -> u64 {
        self.epoch
    }

    /// host 注入事件队列把手(Plan 22 P0):TS transport 持一份 clone,经 [`push_event`](crate::push_event)
    /// 塞原始事件;下帧 `ingest_events` 消费。
    #[must_use]
    pub fn event_queue(&self) -> EventQueue {
        self.inject.clone()
    }

    /// 注入一条原始事件(Plan 22 P0:host/测试便捷入口;等价 push 到 `event_queue`)。
    pub fn inject_raw(&self, raw: &str) {
        self.inject.borrow_mut().push_back(RawEvent::new(raw));
    }

    /// 应答 pending **permission**(Plan 27:tap 命中 / 影子 a11y 按钮 / 剧本)。幂等:无 pending
    /// permission → false。落定块原位替换 + FSM 解阻;host 另行 POST 真实 reply。
    pub fn reply_permission_with(&mut self, allow: bool) -> bool {
        if self
            .store
            .answer_pending_ask(AskAnswer::Permission { allow })
            .is_some()
        {
            self.ask_pressed = None;
            self.drive_fsm(&FsmInput::Replied);
            return true;
        }
        false
    }

    /// 应答 pending **question**(Plan 27:DOM 表单提交 / 剧本)。幂等同上。
    pub fn reply_question_with(&mut self, options: Vec<usize>, text: String) -> bool {
        if self
            .store
            .answer_pending_ask(AskAnswer::Question { options, text })
            .is_some()
        {
            self.drive_fsm(&FsmInput::Replied);
            return true;
        }
        false
    }

    /// 当前 pending ask(host/剧本/e2e 真相源;Plan 27 §1.2)。
    #[must_use]
    pub fn pending_ask(&self) -> Option<crate::store::PendingAsk> {
        self.store.pending_ask()
    }

    /// pending ask 块的 view 下标(→ `visible_messages` 里查它的屏幕矩形;无/未上屏 → None)。
    #[must_use]
    pub fn pending_ask_block(&self) -> Option<usize> {
        let ask = self.store.pending_ask()?;
        self.views.iter().position(|v| v.part_id == ask.part_id)
    }

    /// DOM 表单实测高回报(Plan 27 §3,单向一次):高差 > 1 行 → 给 pending ask 块补空行占位。
    pub fn set_ask_height(&mut self, px: f32) {
        let Some(ask) = self.store.pending_ask() else {
            return;
        };
        let Some(view) = self.views.iter().find(|v| v.part_id == ask.part_id) else {
            return;
        };
        let Some(cache) = &view.cache else { return };
        let line_h = cache.placed.first().map_or(18.0, |p| p.size[1].max(1.0));
        let extra = px - cache.height;
        if extra > line_h {
            let n = (extra / line_h).ceil() as u32;
            self.store.set_ask_pad_lines(&ask.part_id, n);
        }
    }

    /// (A 路)pending permission 按钮命中盒(**世界坐标**;本帧 build_frame 重建)。
    #[must_use]
    pub fn ask_targets(&self) -> &[(String, Rect)] {
        &self.ask_targets
    }

    /// D4:本帧折叠汇总行命中盒(世界坐标;key = `(view<<32)|区序号`)。e2e/宿主查询用。
    #[must_use]
    pub fn fold_targets(&self) -> &[(u64, Rect)] {
        &self.fold_targets
    }

    /// 屏幕点(设备像素)命中哪个 ask 按钮(hover cursor 用;不触发应答)。
    #[must_use]
    pub fn ask_hit_at(&self, sx: f32, sy: f32) -> Option<&str> {
        let cam = &self.camera;
        let pan = cam.pan();
        let zoom = cam.zoom().max(f32::EPSILON);
        let (wx, wy) = (sx / zoom + pan[0], sy / zoom + pan[1]);
        self.ask_targets
            .iter()
            .find(|(_, r)| r.contains(wx, wy))
            .map(|(id, _)| id.as_str())
    }

    /// 画布 tap 路由(Plan 27 / 0032 hit-test 层第一个口):命中 ask 按钮 → 应答;
    /// 命中折叠汇总行 → 展开(Plan 32 D4);未命中 → false。
    pub fn tap(&mut self, sx: f32, sy: f32) -> bool {
        match self.ask_hit_at(sx, sy) {
            Some("allow") => return self.reply_permission_with(true),
            Some("deny") => return self.reply_permission_with(false),
            _ => {}
        }
        if let Some(key) = self.fold_hit_at(sx, sy) {
            self.diff_unfold.insert(key);
            self.fold_anim.insert(key, FOLD_SCALE_START);
            let vi = (key >> 32) as usize;
            if let Some(v) = self.views.get_mut(vi) {
                v.cache = None; // 重排版:unfolded 位图进 RenderCtx → 展开行入流
            }
            return true;
        }
        // S2(Plan 34):链接命中 → 置待派发 URL(core 不导航,CR5/0000 §2.2;宿主取走回调)。
        // S3:命中盒只收白名单内链接(拒绝者在 ensure_layouts 已降级不进盒),此处 belt-and-
        // suspenders 再验一次(策略热切换窗口期)。
        if let Some(url) = self.link_hit_at(sx, sy).map(str::to_owned) {
            if self.url_policy.link_allowed(&url) {
                self.pending_open_url = Some(url);
            }
            return true;
        }
        false
    }

    /// S2:链接命中查询(屏幕 px;hover cursor / tap 共用,同 ask_hit_at 变换)。
    #[must_use]
    pub fn link_hit_at(&self, sx: f32, sy: f32) -> Option<&str> {
        let cam = &self.camera;
        let pan = cam.pan();
        let zoom = cam.zoom().max(f32::EPSILON);
        let (wx, wy) = (sx / zoom + pan[0], sy / zoom + pan[1]);
        self.link_targets
            .iter()
            .find(|(_, r)| r.contains(wx, wy))
            .map(|(url, _)| url.as_str())
    }

    /// S2:取走待派发的链接打开请求(宿主层 tap 后轮询;取即清)。
    pub fn take_pending_open_url(&mut self) -> Option<String> {
        self.pending_open_url.take()
    }

    /// S4:取走本帧新 settled 的 part 播报(取即清;镜像层追加进 aria-live 区)。
    pub fn take_settle_announcements(&mut self) -> Vec<(String, String)> {
        std::mem::take(&mut self.settle_announcements)
    }

    /// S2:本帧链接命中盒(测试/宿主)。
    #[must_use]
    pub fn link_targets(&self) -> &[(String, Rect)] {
        &self.link_targets
    }

    /// S3:配置 URL 白名单(宿主 setter;下一次排版/派发生效)。改表后既有块缓存失效
    /// (链接角色降级在 ensure_layouts 做,须重排)。
    pub fn set_url_policy(&mut self, policy: crate::UrlPolicy) {
        if self.url_policy != policy {
            self.url_policy = policy;
            for v in &mut self.views {
                v.cache = None; // 重排:白名单裁决影响 Link 角色降级与命中盒
            }
        }
    }

    /// D4:view i 的折叠区展开位图(RenderCtx 输入;≥64 恒折叠,见 RenderCtx.unfolded)。
    fn unfold_mask(&self, view: usize) -> u64 {
        let hi = (view as u64) << 32;
        self.diff_unfold
            .iter()
            .filter(|k| *k & 0xFFFF_FFFF_0000_0000 == hi)
            .fold(0u64, |m, k| {
                let r = k & 0xFFFF_FFFF;
                if r < 64 {
                    m | (1u64 << r)
                } else {
                    m
                }
            })
    }

    /// D4:折叠汇总行命中(屏幕 px → 世界,同 ask_hit_at 变换)。
    fn fold_hit_at(&self, sx: f32, sy: f32) -> Option<u64> {
        let cam = &self.camera;
        let pan = cam.pan();
        let zoom = cam.zoom().max(f32::EPSILON);
        let (wx, wy) = (sx / zoom + pan[0], sy / zoom + pan[1]);
        self.fold_targets
            .iter()
            .find(|(_, r)| r.contains(wx, wy))
            .map(|(k, _)| *k)
    }

    /// 按压视觉态(Plan 27 A 路手感):pointerdown 命中 → 记 id(装饰画深色);up/应答清除。
    pub fn ask_press_at(&mut self, sx: f32, sy: f32) -> bool {
        self.ask_pressed = self.ask_hit_at(sx, sy).map(str::to_owned);
        self.ask_pressed.is_some()
    }

    /// 清按压态(pointerup / 取消)。
    pub fn ask_release(&mut self) {
        self.ask_pressed = None;
    }

    /// 会话态的稳定字符串标签(Plan 22:host 据此画活跃指示/禁发送/弹 Dock;wasm `stats` 用)。
    #[must_use]
    pub fn session_status_tag(&self) -> &'static str {
        use crate::fsm::Blocker;
        match &self.session_status {
            SessionStatus::Idle => "idle",
            SessionStatus::AwaitingAck { .. } => "awaiting",
            SessionStatus::Streaming { .. } => "streaming",
            SessionStatus::Retrying { .. } => "retrying",
            SessionStatus::Blocked {
                on: Blocker::Permission,
                ..
            } => "blocked:permission",
            SessionStatus::Blocked {
                on: Blocker::Question,
                ..
            } => "blocked:question",
            SessionStatus::Stalled => "stalled",
            SessionStatus::Stopped => "stopped",
            SessionStatus::Errored { .. } => "errored",
        }
    }

    /// 清掉无对应 store part 的孤儿 view(Plan 22:part.removed / 错误卡清除后,视图不再渲染残留)。
    fn clear_orphan_views(&mut self) {
        let live: std::collections::HashSet<String> = self
            .store
            .parts_in_order()
            .map(|(id, _)| id.to_owned())
            .collect();
        let orphans: Vec<String> = self
            .views
            .iter()
            .filter(|v| !live.contains(&v.part_id) && (!v.revealed.is_empty() || v.cache.is_some()))
            .map(|v| v.part_id.clone())
            .collect();
        let (now, dur) = (self.now_ms, self.exit_dissolve_ms);
        for pid in orphans {
            self.smoother.reset_part(&pid);
            let v = self.view_mut(&pid);
            // N3(0016 exit 首租):dissolve 开启时,孤儿先进退场态 —— cache 原样保留
            // (几何不动,AR3),glyph 携 exit_time 交 shader 溶解;到期才真清除。
            // off(dur==0,默认)= 立即清除 == 旧行为(恒等)。
            if dur > 0.0 {
                match v.exiting {
                    None => {
                        v.exiting = Some(now);
                        continue; // 本帧起溶解,保留全部渲染态
                    }
                    Some(start) if now < start + dur => continue, // 溶解中
                    Some(_) => {}                                 // 到期 → 落到真清除
                }
            }
            v.exiting = None;
            v.revealed.clear();
            v.pushed = 0;
            v.pushed_bytes = 0;
            v.spawn.clear();
            v.cache = None;
            v.settled = false;
            v.agg = None;
        }
    }

    /// N3:设退场 dissolve 时长(0=off 即时清除;>0 溶解 ms)。宿主/试衣间调用。
    pub fn set_exit_dissolve_ms(&mut self, ms: f64) {
        self.exit_dissolve_ms = ms.max(0.0);
    }

    /// N3:当前退场 dissolve 时长(wasm 每帧喂 shader globals)。
    #[must_use]
    pub fn exit_dissolve_ms(&self) -> f64 {
        self.exit_dissolve_ms
    }

    /// N4:Spring 进场开关(off 默认恒等;on 正文换 spring 曲线,试衣间调)。
    pub fn set_spring_enter(&mut self, on: bool) {
        self.spring_enter = on;
    }

    /// F1(0040):反馈通道衰减(0=off;(0,1) 拖尾)。宿主/试衣间调。
    pub fn set_feedback_decay(&mut self, decay: f32) {
        self.feedback_decay = decay.clamp(0.0, 0.98);
    }

    /// F2(0040):后处理三小件参数(全零 = 恒等直通)。
    pub fn set_post_params(&mut self, vignette: f32, grain: f32, chroma_px: f32) {
        self.post_params = [
            vignette.clamp(0.0, 1.0),
            grain.clamp(0.0, 0.5),
            chroma_px.clamp(0.0, 6.0),
        ];
    }

    /// 2) 把 store 里新增的文本尾部切 grapheme 入 smoother。
    fn enqueue_new_text(&mut self) {
        self.clear_orphan_views(); // 先清孤儿(part.removed / 错误卡清除)→ 不渲染残留
        let part_ids: Vec<String> = self
            .store
            .parts_in_order()
            .map(|(id, _)| id.to_owned())
            .collect();
        for part_id in part_ids {
            // Plan 22 P3:喂渲染管线的是**显示源**(display_source:文本=正文;tool/file/compaction=
            // 标签+内容的兜底 markdown)→ 每 part 都看得见。Plan 19 P1 优化照旧:先比字节长度 O(1) 跳过未变。
            let cur_bytes = match self.store.display_source(&part_id) {
                Some(text) => text.len(),
                None => continue,
            };
            if cur_bytes == self.view_mut(&part_id).pushed_bytes {
                continue;
            }
            // F1(0040):内容到达即画面活动(与揭示速度无关,收敛自停判据)。
            self.last_activity_ms = self.now_ms;
            // Plan 22 P3:非文本 part(tool/file/…)的显示源可**整体重写**(如 tool pending→completed),
            // 非 append → 重置该 part(清 smoother 队列 + view 流式态)后整段重推,避免拼接旧尾。
            // 文本 part 仍走 append(流式逐字),保 Plan 19 优化。
            if !self.store.part_is_text(&part_id) {
                self.smoother.reset_part(&part_id);
                let v = self.view_mut(&part_id);
                v.revealed.clear();
                v.pushed = 0;
                v.spawn.clear();
                v.cache = None;
                v.settled = false;
            }
            // 变了(增长或对账缩短/载荷更新)→ 重切。克隆成 owned grapheme,先释放 store 借用再改 view/smoother。
            let gs: Vec<String> = match self.store.display_source(&part_id) {
                Some(text) => graphemes(&text).into_iter().map(str::to_owned).collect(),
                None => continue,
            };
            let pushed = self.view_mut(&part_id).pushed;
            if gs.len() > pushed {
                let new: Vec<&str> = gs[pushed..].iter().map(String::as_str).collect();
                self.smoother.push(&part_id, &new);
                self.view_mut(&part_id).pushed = gs.len();
            }
            self.view_mut(&part_id).pushed_bytes = cur_bytes;
        }
    }

    /// 3) 整流吐字 → 记入各 part 的已上屏序列。
    fn reveal(&mut self, dt_ms: f64) {
        for r in self.smoother.update(dt_ms, self.now_ms) {
            self.view_mut(&r.part_id)
                .revealed
                .push((r.cluster, r.spawn_time_ms));
        }
    }

    /// session 过滤(Phase F):目标已设且该 part 归属已知且不匹配 → 过滤掉。
    /// 归属未知时乐观保留(本地单会话常态),待 updated/snapshot 解析后收敛。
    fn is_filtered(&self, view: &PartView) -> bool {
        if let Some(target) = &self.target_session {
            if let Some(sid) = self.store.part_session(&view.part_id) {
                return sid != target;
            }
        }
        false
    }

    /// 排版各块(Phase G 块冻结):内容长度/宽度不变的块跳过 layout 直接用缓存,只有正在
    /// 生长的尾部块(或宽度变化)才重排——根治每帧全量重排。
    fn ensure_layouts(&mut self) {
        self.last_rebuilds = 0;
        for i in 0..self.views.len() {
            // Plan 19 P2:Warm = 已释放几何的屏外 settled 块,**不重排**(留 `agg` 占位);进可见
            // 滞回带由 `reclaim` 翻回 Hot 后,下帧此处重建(cache=None → dirty)。
            if matches!(
                self.views[i].tier,
                Tier::Warm | Tier::Cold | Tier::FrozenFar
            ) {
                continue; // Plan 29:非 Hot 一律不重排(Cold 若进此路会以空 revealed 排出空 cache 塌高)
            }
            let len = self.views[i].revealed.len();
            // Plan 25:折行宽 = 列宽(与 boxlayout 盒宽同源),不再按整文档宽折行。
            let wrap = crate::boxlayout::wrap_width(
                self.max_width,
                self.views[i].role == crate::store::Role::User,
                self.dpr,
            );
            let dirty = match &self.views[i].cache {
                Some(c) => c.revealed_len != len || (c.width - wrap).abs() > f32::EPSILON,
                None => true,
            };
            if !dirty {
                continue;
            }
            self.last_rebuilds += 1; // Plan 19 §2:本帧重建数(thrash 监控)
            let text: String = self.views[i]
                .revealed
                .iter()
                .map(|(c, _)| c.as_str())
                .collect();
            // Plan 23 接缝:reasoning/tool/compaction 有 specific 渲染器 → 直出漂亮 StyledSpan 卡 +
            // 扁平节点树(供 reveal 逐字揭示);无 specific 的 kind(text/file/error)走既有 markdown
            // 路径(0014 B 表格 + 0020 节点树)。registry 是纯函数(R8)→ 不破录像/虚拟化重建等价。
            let part_id = self.views[i].part_id.clone();
            let spec = self
                .store
                .render_part(&part_id)
                .filter(|(k, _)| self.registry.has_specific(*k));
            // Plan 25 M2b:tool 卡状态(kind_tag = "tool:<name> · <status>")→ 驱动流光/徽章。
            let card_status: Option<u8> = spec.as_ref().and_then(|(k, rp)| {
                if *k != crate::partrender::PartKind::Tool {
                    return None;
                }
                rp.kind_tag.rsplit('·').next().map(|s| match s.trim() {
                    "pending" => 0u8,
                    "running" => 1,
                    "error" => 3,
                    _ => 2, // completed/done 及未知态归 done(AR12 精神:未知不炸)
                })
            });
            // Plan 25 M4(Plan 10 相位 4 最小件):状态迁移 → 记 (from, at) 供徽章 SDF mix 形变。
            // prev 用 **view 级** last_card_status(cache 会被全量重写重建,读旧 cache 不可靠)。
            if let (Some(prev), Some(cur)) = (self.views[i].last_card_status, card_status) {
                if prev != cur {
                    self.views[i].badge_morph = Some((prev, self.now_ms as f32));
                }
            }
            self.views[i].last_card_status = card_status;
            let mut decorations: Vec<crate::partrender::DecorationOp> = Vec::new();
            let (spans, tables, nodes, embeds) = if let Some((kind, rp)) = spec {
                let ctx = crate::partrender::RenderCtx {
                    width: wrap,
                    folded: false,
                    unfolded: self.unfold_mask(i), // D4:折叠区展开位图(view 级持久态)
                    parse: crate::content::parse_markdown, // 0039:真 parse 注入(组件零依赖 core)
                };
                // 0037(Plan 32):full 渲染 = spans + 声明式装饰指令(随块缓存;emit 时解析色)。
                let full = self.registry.render_full(kind, &rp, &ctx);
                decorations = full.decorations;
                let spans = full.spans;
                let total: u32 = spans.iter().map(|s| graphemes(s.text()).len() as u32).sum();
                if total == 0 {
                    decorations = Vec::new();
                    parse_markdown_nodes(&text, i as u32)
                } else {
                    (
                        spans,
                        Vec::new(),
                        flat_node_tree(i as u32, total),
                        Vec::new(),
                    )
                }
            } else {
                // 0014 B:带表格结构;0020:同时建内容节点树(块序号 = view 下标,打进 key 高 32)。
                parse_markdown_nodes(&text, i as u32)
            };
            // 行内链接 sidecar(Plan 34 S2):从最终 spans 提取(spec 与 markdown 路皆可;
            // spec 路 tool 卡通常无 Link 角色 → 空)。spans 不变 → 视觉零变化。
            // S3:白名单外链接**不进 sidecar**(无命中盒/无 hover)且角色降级为 Normal
            // (视觉 = 普通文本色,shadcn R12 拒绝路径);拒绝集在 roles 展开后应用。
            let (links, rejected): (Vec<_>, Vec<_>) = crate::content::extract_links(&spans)
                .into_iter()
                .partition(|l| self.url_policy.link_allowed(&l.url));
            // 显示字形序列(markdown 渲染后):与 layout 的 grapheme 切分同源,保证 1:1。
            let mut clusters = Vec::new();
            let mut roles = Vec::new();
            let mut strike = Vec::new();
            for span in &spans {
                let role = span.role().as_u32();
                let struck = span.is_struck();
                for g in graphemes(span.text()) {
                    clusters.push(g.to_owned());
                    roles.push(role);
                    strike.push(struck);
                }
            }
            // S3:白名单外链接角色降级(Link → Normal;文字仍在,只是不像链接、不可点)。
            for l in &rejected {
                for r in roles
                    .iter_mut()
                    .take(l.range.1 as usize)
                    .skip(l.range.0 as usize)
                {
                    *r = StyleRole::Normal.as_u32();
                }
            }
            // Plan 31 R2:内容身份分配(前后缀 diff;append-only 等价旧位置键)。
            {
                let v = &mut self.views[i];
                let (ids, next) =
                    assign_stable_ids(&v.id_clusters, &v.glyph_ids, &clusters, v.next_glyph_id);
                v.glyph_ids = ids;
                v.next_glyph_id = next;
                v.id_clusters.clone_from(&clusters);
            }
            let result = self.layout.layout(&spans, &tables, wrap);
            // 数学(Plan 12 ②③):RaTeX 排版 → 缓存(随块冻结,不每帧重排)。失败者不入(退原文渲染,
            // 兜底相位⑦)。① 显示数学 `$$…$$` = MathDisplay 节点(TeX = 区间字符,无 `$$`,display=true);
            // ② 行内数学 `$…$` = 连续 MathTeX 角色 run(TeX = 去首尾 `$`,display=false)。
            let mut math: Vec<((u32, u32), crate::math::MathLayout, bool)> = nodes
                .nodes_of_kind(crate::nodes::NodeKind::MathDisplay)
                .filter_map(|(_, n)| {
                    let tex: String = clusters
                        .get(n.range.0 as usize..n.range.1 as usize)?
                        .concat();
                    let m = crate::math::layout_math(&tex, true);
                    m.ok.then_some((n.range, m, true)) // 显示数学
                })
                .collect();
            let mathrole = StyleRole::MathTeX.as_u32();
            let mut k = 0usize;
            while k < roles.len() {
                if roles[k] != mathrole {
                    k += 1;
                    continue;
                }
                let s = k;
                while k < roles.len() && roles[k] == mathrole {
                    k += 1;
                }
                let inner: String = clusters[s..k].concat().trim_matches('$').to_string();
                let m = crate::math::layout_math(&inner, false);
                if m.ok {
                    math.push(((s as u32, k as u32), m, false)); // 行内数学
                }
            }
            // 显示数学块高度修正(Plan 12,根治"公式重叠"):JS 只给该块一行 raw TeX 的高,但公式视觉
            // 高(`(height+depth)×显示px`)常远大于一行 → 上下溢出、与邻块重叠。这里**给每个显示公式
            // 预留竖直空间**:按块序累加下移 —— `extra_above`(公式高出基线部分超过该行 ascent 的差)下移
            // 本块及之后,`extra_below`(公式深出基线部分超过行剩余 + 块距的差)再下移其后,块总高同增。
            let dpx = self.math_em * DISPLAY_MATH_SCALE;
            let mut placed = result.glyphs;
            let mut height = result.block_height;
            let mut displays: Vec<(usize, usize, f32, f32)> = math
                .iter()
                .filter(|(_, _, d)| *d)
                .map(|((s, e), m, _)| (*s as usize, *e as usize, m.height * dpx, m.depth * dpx))
                .collect();
            displays.sort_by_key(|&(s, _, _, _)| s);
            for (s, e, ah, dh) in displays {
                if s >= placed.len() {
                    continue;
                }
                let line = placed[s].size[1].max(1.0); // 该块 raw TeX 行高
                let extra_above = (ah - 0.8 * line).max(0.0); // 公式上溢:高出基线超过 ascent 的部分
                if extra_above > 0.0 {
                    for p in &mut placed[s..] {
                        p.pos[1] += extra_above;
                    }
                    height += extra_above;
                }
                let extra_below = (dh - (0.2 * line + self.motion.space_block_gap)).max(0.0); // 公式下溢
                let e2 = e.min(placed.len());
                if extra_below > 0.0 && e2 < placed.len() {
                    for p in &mut placed[e2..] {
                        p.pos[1] += extra_below;
                    }
                    height += extra_below;
                }
            }
            // 代码块行窗(Plan 15 ①):超 MAX_LINES 行 → 钉死窗高,把**块后**内容上移 (N-6)·lineH(不顶
            // 下文,plan13 锚底友好);窗内 N 行原位(build_frame 据 scroll 偏移/cull/fade)。按块序累加。
            let mut code_blocks: Vec<CodeView> = Vec::new();
            let mut cb_ranges: Vec<(u32, u32)> = nodes
                .nodes_of_kind(crate::nodes::NodeKind::CodeBlock)
                .map(|(_, n)| n.range)
                .collect();
            cb_ranges.sort_by_key(|r| r.0);
            for (s, e) in cb_ranges {
                let (s, e) = (s as usize, (e as usize).min(placed.len()));
                if s >= e {
                    continue;
                }
                // 块上边距(Plan 15 ⑥):本块及其后整体下移 → 代码框与上方内容留白(不贴脸)。
                for p in &mut placed[s..] {
                    p.pos[1] += CODE_BLOCK_MARGIN;
                }
                height += CODE_BLOCK_MARGIN;
                let (mut top_y, mut bot_y, mut line_h) = (f32::MAX, f32::MIN, 0.0f32);
                // 代码内容起始 x = 首个**代码内容**字左缘(含高亮各角色;行号 gutter 之右)。横裁左界(⑤)。
                let mut code_x0 = f32::MAX;
                for (k, p) in placed[s..e].iter().enumerate() {
                    top_y = top_y.min(p.pos[1]);
                    bot_y = bot_y.max(p.pos[1]);
                    line_h = line_h.max(p.size[1]);
                    let is_code = roles
                        .get(s + k)
                        .copied()
                        .is_some_and(StyleRole::is_code_text_u32);
                    if is_code && p.size[0] > 0.0 {
                        code_x0 = code_x0.min(p.pos[0]);
                    }
                }
                if line_h <= 0.0 {
                    continue;
                }
                if code_x0 > 1.0e30 {
                    code_x0 = 0.0; // 无可见代码字(纯空行,仍是 f32::MAX 初值)→ 不裁
                }
                let n_lines = ((bot_y - top_y) / line_h).round() as usize + 1;
                if n_lines > crate::codeblock::MAX_LINES {
                    let excess = (n_lines - crate::codeblock::MAX_LINES) as f32 * line_h;
                    for p in &mut placed[e..] {
                        p.pos[1] -= excess;
                    }
                    height -= excess;
                }
                // 块下边距(Plan 15 ⑥):块后内容下移 → 代码框与下方内容留白。
                for p in &mut placed[e..] {
                    p.pos[1] += CODE_BLOCK_MARGIN;
                }
                height += CODE_BLOCK_MARGIN;
                code_blocks.push(CodeView {
                    range: (s as u32, e as u32),
                    top_y,
                    n_lines,
                    line_h,
                    code_x0,
                });
            }
            // Plan 19 P1:块内容宽随 height 算一次(免 build_frame 每帧 fold)。x 不受上面代码块
            // y 调整影响 → 此处 fold 即终值;空块/纯换行 → 0。
            let content_width = placed
                .iter()
                .filter(|p| p.size[0] > 0.0)
                .map(|p| p.pos[0] + p.size[0])
                .fold(0.0f32, f32::max);
            self.views[i].cache = Some(BlockCache {
                revealed_len: len,
                width: wrap,
                decorations,
                card_status,
                clusters,
                roles,
                strike,
                placed,
                content_width,
                height,
                // 各表格面板几何(同源 colX/rowY,0018 #5):layout 回传,逐表收敛成一个 SDF 面板。
                table_panels: result.table_panels,
                nodes,
                math: {
                    // Plan 31 R2:数学区稳定身份(region 按序配对;ch 序列前后缀 diff)。
                    let seqs: Vec<Vec<String>> = math
                        .iter()
                        .map(|(_, m, _)| m.glyphs.iter().map(|g| g.ch.to_string()).collect())
                        .collect();
                    let v = &mut self.views[i];
                    let mut ids: Vec<Vec<u32>> = Vec::with_capacity(seqs.len());
                    for (ri, seq) in seqs.iter().enumerate() {
                        let empty_c: Vec<String> = Vec::new();
                        let empty_i: Vec<u32> = Vec::new();
                        let (oc, oi) = match (v.math_id_clusters.get(ri), v.math_ids.get(ri)) {
                            (Some(c), Some(idv)) => (c, idv),
                            _ => (&empty_c, &empty_i),
                        };
                        let (rid, next) = assign_stable_ids(oc, oi, seq, v.next_math_id);
                        v.next_math_id = next;
                        ids.push(rid);
                    }
                    v.math_ids = ids;
                    v.math_id_clusters = seqs;
                    math
                },
                embeds,
                links,
                code_blocks,
            });
            // Plan 19 P2:聚合维同步(释放几何后凭它占位 → 布局稳定,0029 §3)。
            self.views[i].agg = Some(BlockAgg {
                content_width,
                height,
            });
        }
    }

    /// 组 FrameData(Plan 3 L):块 AABB 入空间索引 → 相机视口查可见 → 出世界坐标 glyph。
    /// 相机变换在着色器里做;锚底 = 相机 pan.y 跟随底部;块冻结仍在(ensure_layouts)。
    fn build_frame(&mut self) -> FrameData {
        let bf_t0 = web_time::Instant::now(); // Plan 19 §2 per-phase 计时
                                              // 排版 + 揭示调度已在 `frame()` 内先行(ensure_layouts → schedule);此处只读状态组帧。
        self.sync_image_registry(); // Plan 14 ③:新到的图补登占位态,JS 可领取解码

        // 1) chat 级盒子布局(Plan 13 §4):角色分组 → Taffy 盒树 → 每 view 盒 origin/width。**收编
        //    手搓 `top += height`**:user 右、assistant 左、一回合一盒(0005)。view 内 glyph 相对位
        //    不变,整体按 box origin 平移(0016 morph 身份稳定)。叶子尺寸 = (内容宽, 块高)。
        let sizes = self.layout_sizes();
        let turns = group_turns(&self.views);
        // Plan 21:view→turn 反查(`visible_messages` 标 turn 序;O(views),与本就 O(views) 的布局同阶)。
        let mut view_turn = vec![0u32; self.views.len()];
        for (ti, t) in turns.iter().enumerate() {
            if let Some(u) = t.user {
                if u < view_turn.len() {
                    view_turn[u] = ti as u32;
                }
            }
            for &a in &t.assistant {
                if a < view_turn.len() {
                    view_turn[a] = ti as u32;
                }
            }
        }
        let boxpos =
            crate::boxlayout::layout_chat(&turns, &sizes, self.max_width, &self.motion, self.dpr);
        let e_layout = bf_t0.elapsed(); // ← layout 段(含 Taffy)止

        // 可绘制块(过滤非目标 session / 空块)+ 盒 (origin, 盒宽, 高)。
        let mut drawable: Vec<(usize, [f32; 2], f32, f32)> = Vec::new(); // (view, origin, 盒宽, 高)
        let mut total_glyphs = 0usize; // 可观测:裁剪前总量
        for (i, view) in self.views.iter().enumerate() {
            if self.is_filtered(view) {
                continue;
            }
            // Plan 19 P2:Warm 块(释放几何)仍进 drawable(凭 agg 高占位 → 上方块不塌,0029 §3),
            // 但 0 glyph → 不 emit。Hot 非空照常;Hot 空块/无 agg 跳过(保 P1 前行为)。
            let h = match (&view.cache, view.tier) {
                (Some(c), _) if !c.placed.is_empty() => {
                    total_glyphs += c.placed.len();
                    c.height
                }
                (None, Tier::Warm | Tier::Cold | Tier::FrozenFar) => match view.agg {
                    Some(a) => a.height,
                    None => continue,
                },
                _ => continue,
            };
            let bp = boxpos.get(i).copied().unwrap_or_default();
            drawable.push((i, bp.origin, bp.width.max(1.0), h));
        }
        // 1.5) 已揭示底(严格 bottom-line):锚底跟「已上屏」的字底,**不是**「已排版」全高——否则
        //      相机先滚到解析全高、文字再慢慢揭(rate-limit 下表现为"预知一段、相机先动文字后出")。
        //      释放按文档序 → 倒序找首个有已释放字的块,其已释放字最低底 = 揭示前沿(更后块未揭、忽略;
        //      更前块已全揭、底 ≤ 此值)。无任何已释放字 → 0(不预滚)。
        let mut revealed_height = 0.0f32;
        for &(i, origin, _w, _h) in drawable.iter().rev() {
            let Some(c) = &self.views[i].cache else {
                continue;
            };
            let spawn = &self.views[i].spawn;
            let mut bmax = -1.0f32;
            for (j, p) in c.placed.iter().enumerate() {
                if spawn.get(j).copied().flatten().is_some() {
                    bmax = bmax.max(p.pos[1] + p.size[1]);
                }
            }
            if bmax >= 0.0 {
                revealed_height = origin[1] + bmax;
                break;
            }
        }

        // 2) 高度区间索引(Plan 29 V1):drawable 按文档序 y 单调 → O(N) 填充零排序,
        //    查询 O(log N + K);无 HashMap 翻搅(旧 grid 段 1.8ms/帧 → 见 plan29 progress)。
        self.grid
            .build(drawable.iter().map(|&(i, o, _w, h)| (i, o[1], h)));

        // 3) 锚底:相机 pan.y **平滑**跟随底部并夹取。直接 set 到底会在每次换行(content 高 +一行)
        //    整屏一次性上移一行 = "换行跳一下";改为指数趋近底部(fps 无关),小跳平滑、大跳(初次/
        //    历史瞬显)直接到位避免慢 scroll 穿过整篇。字本身的重排已由 0016 morph 补间。
        let visible_h = self.camera.viewport()[1] / self.camera.zoom();
        // 锚底跟「已揭示底」(严格 bottom-line);未揭示的解析尾不预滚。
        let max_pan_y = (revealed_height - visible_h).max(0.0);
        let mut pan = self.camera.pan();
        // 0038:Following/Anchoring 共用 smooth-damp 跟底通道;Released 相机纹丝不动
        //(内容增长零位移 —— 只 clamp 上界,pan 不变)。
        if self.follow == FollowState::Released {
            self.pan_vel_y = 0.0; // 用户接管(滚动/缩放)→ 清速度,免重新跟随时残留
        } else if (max_pan_y - pan[1]).abs() > visible_h {
            // 落后超过一屏(初次/历史瞬显/大段倾泻)→ 直接到位,不慢 scroll 穿整篇。
            pan[1] = max_pan_y;
            self.pan_vel_y = 0.0;
        } else {
            // 临界阻尼 smooth-damp(速度连续 → 比指数更顺、无过冲;fps 无关)。
            let dt = (self.frame_dt as f32 / 1000.0).max(1e-4);
            let omega = 2.0 / ANCHOR_SMOOTH_TIME;
            let x = omega * dt;
            let expf = 1.0 / (1.0 + x + 0.48 * x * x + 0.235 * x * x * x);
            let change = pan[1] - max_pan_y;
            let temp = (self.pan_vel_y + omega * change) * dt;
            self.pan_vel_y = (self.pan_vel_y - omega * temp) * expf;
            pan[1] = max_pan_y + (change + temp) * expf;
            if (max_pan_y - pan[1]).abs() < 0.5 {
                pan[1] = max_pan_y; // 收敛即贴底,免长尾抖
                self.pan_vel_y = 0.0;
            }
        }
        pan[1] = pan[1].clamp(0.0, max_pan_y);
        self.camera.set_pan(pan[0], pan[1]);
        // 0038 迁移:Anchoring 到底 → Following;Released 仅在**用户主动向下**抵底时吸附
        //(scroll_down_intent 一帧有效;内容增长永不触发重进入 —— 修旧被动复跟随缺陷)。
        match self.follow {
            FollowState::Anchoring => {
                if (max_pan_y - pan[1]).abs() < 0.5 {
                    self.follow = FollowState::Following;
                }
            }
            FollowState::Released => {
                if self.scroll_down_intent && pan[1] >= max_pan_y - ANCHOR_THRESHOLD {
                    self.follow = FollowState::Following;
                }
            }
            FollowState::Following => {}
        }
        self.scroll_down_intent = false;

        // 4) 视口查可见块(grid 是 broad phase)→ 实际 AABB narrow phase → 出世界坐标 glyph。
        let boxes: std::collections::HashMap<usize, ([f32; 2], f32, f32)> = drawable
            .iter()
            .map(|&(i, o, w, h)| (i, (o, w, h)))
            .collect();
        let visible = self.camera.visible_world_rect();
        let ids = self.grid.query(&visible);
        let e_grid = bf_t0.elapsed(); // ← grid(drawable+索引+查)段止
        let mut glyphs = Vec::new();
        let mut rects: Vec<FrameRect> = Vec::new();
        let mut panels: Vec<FramePanel> = Vec::new();
        let mut widgets: Vec<FrameWidget> = Vec::new();
        // 图片(Plan 14 ③):Ready 静态图 → 纹理 quad;动图 → DOM overlay 世界矩形(下方按嵌入态填)。
        let mut images: Vec<crate::FrameImage> = Vec::new();
        let mut frame_embeds: Vec<crate::FrameEmbed> = Vec::new();
        // shader 画板(Plan 16):代码块 copy icon(§2.7 程序化)等;护栏 = 仅可见块(cull)+ 节流时钟。
        let mut shaderboxes: Vec<crate::FrameShaderBox> = Vec::new();
        let mut shaderbox_pixels: u64 = 0; // 护栏度量:屏上 box∩viewport 面积和(Plan 16 §2.4)
        let mut anim_groups = 0usize; // Plan 25 M2a:本帧活跃动效组(dynamic orb/光标/指示条/wipe;≤4 预算)
        let mut visible_blocks = 0usize; // 可观测:实际出 glyph 的块数
        let mut hit_rects: Vec<(u64, Rect)> = Vec::new(); // Plan 15 ④:代码块行窗世界命中矩形
        let mut visible_recs: Vec<(usize, u32, [f32; 2], f32, f32)> = Vec::new(); // Plan 21:可见块世界盒
        let mut selection_rect_count = 0usize; // Plan 21 P2:本帧选区高亮数(可观测)
        let selection = self.selection.clone(); // Plan 21 P2:本帧选区(快照,避免借用冲突;小)
                                                // Plan 27 A 路:pending ask 块(命中盒仅 permission;卡底两类都发,Plan 28 R3.4)。
        let pending_ask_all: Option<(String, bool)> = self
            .store
            .pending_ask()
            .map(|a| (a.part_id.clone(), a.kind == AskKind::Permission));
        let pending_perm: Option<String> = pending_ask_all
            .as_ref()
            .filter(|(_, is_perm)| *is_perm)
            .map(|(id, _)| id.clone());
        let ask_pressed = self.ask_pressed.clone();
        let mut new_ask_targets: Vec<(String, Rect)> = Vec::new();
        let mut new_fold_targets: Vec<(u64, Rect)> = Vec::new(); // D4
        let mut new_link_targets: Vec<(String, Rect)> = Vec::new(); // S2
        let reveal_kind = self.scheduler.table_style(); // 表格揭示风格(驱动面板骨架揭示)
                                                        // M2f:指针世界坐标(hover/press 视觉;presentation-only)。
        let pointer_world = self.pointer.map(|p| {
            let pan = self.camera.pan();
            let z = self.camera.zoom().max(1e-6);
            [pan[0] + p[0] / z, pan[1] + p[1] / z]
        });
        let pointer_down = self.pointer_down;
        for id in ids {
            let view = &self.views[id];
            let Some(cache) = &view.cache else { continue };
            let (origin, box_w, block_h) =
                boxes
                    .get(&id)
                    .copied()
                    .unwrap_or(([0.0, 0.0], self.max_width, 0.0));
            if !Rect::new(origin[0], origin[1], box_w, block_h).intersects(&visible) {
                continue; // narrow phase:实际矩形不相交 → 裁掉
            }
            // Plan 21:记此可见块世界盒(复制按钮 / 文本层据此只覆盖 Hot 可见块,虚拟化 DOM ∝ 可见)。
            visible_recs.push((id, view_turn[id], origin, box_w, block_h));
            // Plan 28 R1:头像 orb 列**移除**——参考(opencode)无头像结构(NOTES.md §0/§1);
            // 等待/流式指示改由 thinking 行 + 行内光标承担(Plan 25 M2a 机制保留)。
            // Plan 25 M2a(design §4.4 ①③):揭示进行中的 assistant view —— 行内光标(文字流末尾
            // 呼吸小方块,0.15Hz,非 blink)+ 回合左缘 2px 指示条。只在未 settled 时发射 →
            // settled/idle 全退场(页面回全冻结);挂 globals.time_ms(注入时钟,seek 确定)。
            if view.role == crate::store::Role::Assistant && !view.settled {
                let frontier = (0..cache.placed.len().min(view.spawn.len()))
                    .rev()
                    .find(|&g| view.spawn[g].is_some() && cache.clusters[g] != "\n");
                if let Some(g) = frontier {
                    let p = &cache.placed[g];
                    let lh = p.size[1].max(8.0);
                    widgets.push(crate::FrameWidget {
                        pos: [
                            origin[0] + p.pos[0] + p.size[0] + 3.0,
                            origin[1] + p.pos[1] + lh * 0.08,
                        ],
                        size: [(lh * 0.42).clamp(3.0, 12.0), lh * 0.84],
                        color: [0.82, 0.86, 0.95, 0.9],
                        params: [2.0, 0.85, 0.15, 0.05],
                        component: crate::frame::WIDGET_PULSE,
                    });
                    widgets.push(crate::FrameWidget {
                        pos: [origin[0] - 12.0, origin[1]],
                        size: [2.5, block_h.max(lh)],
                        color: [0.45, 0.60, 0.95, 0.55],
                        params: [1.2, 0.8, 0.15, 0.08],
                        component: crate::frame::WIDGET_PULSE,
                    });
                    anim_groups += 2;
                }
            }
            // Plan 25 P0(design §4.1):user 盒弱底面板(0018 退化参数:无网格/无 AO/低对比 fill,
            // 圆角 10)。几何 = 盒 rect 向外 outset 视觉内边距(不动盒位/文字 → 不破锚定,0016);
            // id 低 32 位取 0xFFFF_FFFF,与表格面板 `(block_seq<<32)|ti` 永不相撞。
            if view.role == crate::store::Role::User {
                // Plan 28:内衬 token = CSS px → ×dpr(参考 8×12,retina 不减半)。
                let px = self.motion.space_inset * 0.75 * self.dpr.max(0.5);
                let py = self.motion.space_inset * 0.5 * self.dpr.max(0.5);
                panels.push(crate::FramePanel {
                    id: ((id as u64) << 32) | 0xFFFF_FFFF,
                    pos: [origin[0] - px, origin[1] - py],
                    size: [box_w + 2.0 * px, block_h + 2.0 * py],
                    // Plan 28 R2:参考 user 气泡 = surface-base 底 + 1px border-weak 框 + 圆角 6
                    // (message-part.css:128-137;padding 8×12 由上方 px/py 提供)。
                    radius: 6.0,
                    fill: self.theme.user_bg,
                    line_color: self.theme.card_border,
                    header_fill: [0.0; 4],
                    line_w: 1.0,
                    ao: 0.0,
                    ao_color: [0.0; 3],
                    ao_width: 0.0,
                    header_ratio: 0.0,
                    col_ratios: Vec::new(),
                    row_ratios: Vec::new(),
                    reveal: 1.0,
                    edge_glow: 0.0,
                    glow_phase: 0.0,
                    flags: 0,
                });
            }
            // Plan 27:本块为 pending permission 且有按压 → 传按压 run 下标(0允许/1拒绝)。
            let ask_pressed_run = if pending_perm.as_deref() == Some(view.part_id.as_str()) {
                ask_pressed.as_deref().map(|p| usize::from(p == "deny"))
            } else {
                None
            };
            block_decorations(
                cache,
                id as u32, // block_seq:面板稳定身份高位(6D)
                origin,    // Plan 13:盒 origin(x,y),装饰整体平移到盒位
                box_w,     // 盒宽(全宽装饰:代码底/引用条/分隔线锚它,非整窗宽)
                &self.theme,
                self.dpr.max(0.5),
                ask_pressed_run,
                &self.table_style,
                &view.spawn,
                reveal_kind,
                self.now_ms as f32, // M2b:卡片 wipe 入场(注入时钟)
                &self.motion,
                self.shaderbox_clock.time_s(), // M2b:流光相位(慢时钟)
                &mut anim_groups,
                pointer_world, // M2f hover/press
                pointer_down,
                view.badge_morph, // M4 徽章 morph
                &mut rects,
                &mut panels,
                &mut widgets,
                &mut new_fold_targets, // D4:折叠汇总行命中盒
            ); // 4B/6 装饰 + Plan 11 复选框 + M2b 卡片面板/徽章
               // Plan 28 R4:本块是否 shimmer 标题(pending/running 工具)。计 1 个活跃动效组。
            let shimmer_title = matches!(cache.card_status, Some(0 | 1));
            if shimmer_title {
                anim_groups += 1;
            }
            // Plan 21 P2:选区高亮(在装饰之后、glyph 之前入 rects → 压装饰底之上、文字之下)。
            selection_rect_count +=
                push_selection_rects(cache, origin, &self.theme, &selection, id, &mut rects);
            // Plan 28 R3.4:pending ask 块 = 参考 dock-prompt 卡体(surface-raised 底 + 边框 + 圆角 8,
            // message-part.css:717/829;落定后卡撤 → 答案行融入对话流)。
            if pending_ask_all
                .as_ref()
                .is_some_and(|(pid, _)| pid == view.part_id.as_str())
            {
                let pad = 10.0 * self.dpr.max(0.5);
                panels.push(crate::FramePanel {
                    id: ((id as u64) << 32) | 0xFFFF_FFFD, // ask 卡槽(不撞表格/user/旧卡槽)
                    pos: [origin[0] - pad, origin[1] - pad],
                    size: [box_w + 2.0 * pad, block_h + 2.0 * pad],
                    radius: 8.0,
                    fill: [0.1373, 0.1373, 0.1373, 1.0], // --surface-raised-base #232323(live-dark)
                    line_color: self.theme.card_border,
                    header_fill: [0.0; 4],
                    line_w: 1.0,
                    ao: 0.0,
                    ao_color: [0.0; 3],
                    ao_width: 0.0,
                    header_ratio: 0.0,
                    col_ratios: Vec::new(),
                    row_ratios: Vec::new(),
                    reveal: 1.0,
                    edge_glow: 0.0,
                    glow_phase: 0.0,
                    flags: 0,
                });
            }
            // Plan 27:pending permission 块 → 按钮命中盒(世界)= 面板同源几何(run 序:0允许/1拒绝)。
            if pending_perm.as_deref() == Some(view.part_id.as_str()) {
                for (k, b) in ask_button_runs(cache, 6.0).iter().enumerate() {
                    let ident = if k == 0 { "allow" } else { "deny" };
                    new_ask_targets.push((
                        ident.to_owned(),
                        Rect::new(b[0] + origin[0], b[1] + origin[1], b[2] - b[0], b[3] - b[1]),
                    ));
                }
            }
            let glyphs_before = glyphs.len();
            // 图片(Plan 14 ③):本块**已就绪**(Ready+纹理)嵌入 → (ei, 占位区间, 动图?, 自然尺寸, tex_id)。
            // 就绪即隐藏其 alt 占位字(图替之);未就绪(占位/加载/失败)则 alt 照常上屏(兜底)。
            let ready_embeds: Vec<ReadyEmbed> = cache
                .embeds
                .iter()
                .enumerate()
                .filter_map(|(ei, region)| {
                    let e = self.image_registry.get(&Self::embed_key(id, ei))?;
                    e.is_drawable().then(|| {
                        (
                            ei,
                            region.range,
                            e.animated,
                            e.natural_size.unwrap_or((0.0, 0.0)),
                            e.tex_id,
                            e.alpha(self.now_ms as f32, IMAGE_FADE_MS),
                        )
                    })
                })
                .collect();
            // 代码块行窗(Plan 15 ①④):活动块(未 settled)默认 tail 跟最新窗;手动滚动(④,following=
            // false)→ 用存的 scrollY/X(clamp)。命中矩形写入 hit_rects 供 code_block_at 路由。
            let code_windows: Vec<CodeWindow> = cache
                .code_blocks
                .iter()
                .enumerate()
                .map(|(cb_idx, cb)| {
                    let max_scroll = crate::codeblock::max_scroll_lines(cb.n_lines);
                    let view_h = crate::codeblock::window_height(cb.n_lines, cb.line_h);
                    let key = Self::code_scroll_key(id, cb_idx);
                    let (scroll_y, scroll_x) = match self.code_scroll.get(&key) {
                        Some(&(sx, sy, following)) if !following => (
                            crate::codeblock::clamp_scroll_y(sy, cb.n_lines),
                            sx.max(0.0),
                        ),
                        // 跟随 tail(默认 / following):流式跟最新窗,settled 则顶对齐。
                        _ => {
                            let sy = if view.settled {
                                0
                            } else {
                                crate::codeblock::tail_scroll(cb.n_lines)
                            };
                            (sy, 0.0)
                        }
                    };
                    hit_rects.push((
                        key,
                        Rect::new(origin[0], origin[1] + cb.top_y, box_w, view_h),
                    ));
                    CodeWindow {
                        range: cb.range,
                        top_y: cb.top_y,
                        view_h,
                        line_h: cb.line_h,
                        scroll_y,
                        max_scroll,
                        scroll_x,
                        code_left: origin[0] + cb.code_x0,
                        code_right: origin[0] + box_w,
                    }
                })
                .collect();
            // S2(Plan 34):行内链接命中盒 —— 只收 settled 块(AR3:命中只读 settled 几何);
            // 流式半截链接无 sidecar(content 层剥除)天然不可点。区间按行分段(跨行链接多矩形)。
            // hover:指针落在链接行段上 → 下划线加强(FrameRect,link 色;0032 ① 视觉=参数)。
            if view.settled {
                for l in &cache.links {
                    let (s0, e0) = (l.range.0 as usize, l.range.1 as usize);
                    let mut row: Option<(f32, f32, f32, f32)> = None; // (y0,y1,x0,x1)
                    let mut flush = |r: Option<(f32, f32, f32, f32)>| {
                        if let Some((y0, y1, x0, x1)) = r {
                            let rect = Rect::new(x0 + origin[0], y0 + origin[1], x1 - x0, y1 - y0);
                            let hovered = pointer_world.is_some_and(|p| rect.contains(p[0], p[1]));
                            if hovered {
                                rects.push(FrameRect {
                                    pos: [rect.x, rect.y + rect.h - 1.5],
                                    size: [rect.w, 1.5],
                                    color: self.theme.link_underline,
                                    radius: 0.0,
                                    stroke: 0.0,
                                    fx: [0.0; 4],
                                    fx_color: [0.0; 4],
                                    gloop: [0.0; 4],
                                });
                            }
                            new_link_targets.push((l.url.clone(), rect));
                        }
                    };
                    for j in s0..e0.min(cache.placed.len()) {
                        let p = &cache.placed[j];
                        if p.size[0] <= 0.0 {
                            continue; // 零宽(\n 占位)不进命中盒
                        }
                        let (y0, y1) = (p.pos[1], p.pos[1] + p.size[1]);
                        let (x0, x1) = (p.pos[0], p.pos[0] + p.size[0]);
                        match &mut row {
                            Some(r) if (r.0 - y0).abs() < 0.5 => {
                                r.2 = r.2.min(x0);
                                r.3 = r.3.max(x1);
                                r.1 = r.1.max(y1);
                            }
                            _ => {
                                let prev = row.take();
                                flush(prev);
                                row = Some((y0, y1, x0, x1));
                            }
                        }
                    }
                    flush(row.take());
                }
            }
            // N3:退场 dissolve **进度**(0=无;(0,1]=进行中)。core 按注入时钟折算
            // (R8;glyph 本就逐帧发射,折算成本同量级),shader 直接消费免时长布线。
            #[allow(clippy::cast_possible_truncation)] // reason: 进度 ∈(0,1],f32 足够
            let view_exit = view.exiting.map_or(0.0f32, |t| {
                (((self.now_ms - t) / self.exit_dissolve_ms.max(1.0)).clamp(0.0, 1.0) as f32)
                    .max(0.001)
            });
            // D4:展开中的折叠区 scale 包络(区 glyph span → 当前 scale + 区左上世界锚)。
            // 收敛后 fold_anim 空 → 本 vec 空,逐字零成本(AR3)。
            let fold_env: Vec<(u32, u32, f32, f32, f32)> = if self.fold_anim.is_empty() {
                Vec::new()
            } else {
                cache
                    .decorations
                    .iter()
                    .filter(|d| d.kind == crate::partrender::DecorKind::FoldRegion)
                    .filter_map(|d| {
                        let sc = *self
                            .fold_anim
                            .get(&(((id as u64) << 32) | u64::from(d.group)))?;
                        let p0 = cache.placed.get(d.span.0 as usize)?;
                        Some((
                            d.span.0,
                            d.span.1,
                            sc,
                            p0.pos[0] + origin[0],
                            p0.pos[1] + origin[1],
                        ))
                    })
                    .collect()
            };
            for (j, placed) in cache.placed.iter().enumerate() {
                if cache.clusters[j] == "\n" {
                    continue;
                }
                // 数学块(Plan 12 ②):该字属某 MathDisplay 区间 → 由 RaTeX 重排(下方),跳过 raw TeX 字形。
                if cache
                    .math
                    .iter()
                    .any(|&((s, e), _, _)| (j as u32) >= s && (j as u32) < e)
                {
                    continue;
                }
                // 图片就绪(Plan 14 ③):该字属某 Ready 嵌入的 alt 占位区间 → 隐藏(纹理 quad 替之)。
                if ready_embeds
                    .iter()
                    .any(|&(_, (s, e), ..)| (j as u32) >= s && (j as u32) < e)
                {
                    continue;
                }
                // 揭示门(0019):调度器尚未释放该 display 字形(`None`)→ 本帧不绘制(hold)。
                // 收编即时揭示:spawn_time 一律取调度器所定(唯一来源),不再从 revealed 反推。
                let Some(Some(spawn)) = view.spawn.get(j).copied() else {
                    continue;
                };
                // 代码块行窗(Plan 15 ①④):字属某代码块 → scrollY 偏移、窗外 cull、边缘 fade;CodeBlock 字
                // 再按 scrollX 横移(行号 gutter 不横移,固定左)。
                let mut up_shift = 0.0f32; // 渲染 y 上移量(纵滚)
                let mut left_shift = 0.0f32; // 渲染 x 左移量(横滚;行号免)
                let mut code_alpha = 1.0f32;
                let mut code_culled = false;
                let mut x_clip: Option<(f32, f32)> = None; // CodeBlock 字的横裁区间(world x;⑤)
                for w in &code_windows {
                    if (j as u32) >= w.range.0 && (j as u32) < w.range.1 {
                        let scroll_px = w.scroll_y as f32 * w.line_h;
                        let y_in_view = (placed.pos[1] - w.top_y) - scroll_px;
                        if crate::codeblock::culled(y_in_view, w.view_h) {
                            code_culled = true;
                        } else {
                            up_shift = scroll_px;
                            // 行号(gutter)横滚不动;代码字按 scrollX 横移 + 受横裁(⑤)。
                            if cache.roles[j] != StyleRole::CodeLineNum.as_u32() {
                                left_shift = w.scroll_x;
                                x_clip = Some((w.code_left, w.code_right));
                            }
                            // fade 取字**竖直中心**采样:整行 scroll 下,边缘行恰落淡入淡出带内(行顶对齐
                            // 窗沿会差一截带宽而漏淡)。
                            let y_center = y_in_view + 0.5 * placed.size[1];
                            code_alpha = crate::codeblock::edge_fade(
                                y_center,
                                w.view_h,
                                w.line_h,
                                w.scroll_y,
                                w.max_scroll,
                            );
                        }
                        break;
                    }
                }
                if code_culled {
                    continue;
                }
                let mut eff_y = placed.pos[1] + origin[1] - up_shift; // 行窗 scroll 后的世界 y
                let mut eff_x = placed.pos[0] + origin[0] - left_shift; // 行窗横滚后的世界 x
                let mut eff_size = placed.size;
                // D4:展开动画 —— 区内字绕区左上锚等比缩放(y 向区顶压缩;SDF 字任意 scale 锐利)。
                for &(f0, f1, sc, ax, ay) in &fold_env {
                    if (j as u32) >= f0 && (j as u32) < f1 {
                        eff_x = ax + (eff_x - ax) * sc;
                        eff_y = ay + (eff_y - ay) * sc;
                        eff_size = [placed.size[0] * sc, placed.size[1] * sc];
                        break;
                    }
                }
                // 横裁(Plan 15 ⑤):横滚后整字落在代码区外(左压 gutter / 右溢盒)→ 硬裁不发(CPU 整字
                // 粒度;部分裁切的丝滑边缘留 GPU scissor / shader x-clip 后续)。
                if let Some((cl, cr)) = x_clip {
                    if eff_x + placed.size[0] <= cl || eff_x >= cr {
                        continue;
                    }
                }
                // glyph 级 y 裁剪:单条长消息是一个巨块,块级裁剪不够 —— 块内只发与视口相交
                // 的字,把每帧发射量从"整篇"降到"约一屏",根治长消息的每帧分配风暴。
                let gworld = Rect::new(eff_x, eff_y, eff_size[0], eff_size[1]);
                if !gworld.intersects(&visible) {
                    continue;
                }
                glyphs.push(FrameGlyph {
                    cluster: cache.clusters[j].clone(),
                    pos: [eff_x, eff_y], // 世界(盒 origin 平移 + 行窗 纵/横 scroll + D4 折叠包络)
                    size: eff_size,
                    spawn_time: spawn,
                    // Plan 28 R4:pending/running 工具标题 → shimmer 位(1<<31;GPU 相位扫描,
                    // 参考 TextShimmer 语汇);终态重写自然摘位。
                    style: if shimmer_title && cache.roles[j] == StyleRole::ToolTitle.as_u32() {
                        cache.roles[j] | 0x8000_0000
                    } else {
                        cache.roles[j]
                    },
                    // 身份(0016/0017 → Plan 31 R2):块下标 + **内容稳定身份**(前后缀 diff;
                    // append-only 时 == 位置下标,中段插入时两端身份不变 → morph 平滑移动)。
                    block_seq: id as u32,
                    glyph_idx: view.glyph_ids.get(j).copied().unwrap_or(j as u32),
                    // 进场 profile(0025/Plan 10 §3b):按角色 + reveal 风格选,shader 据 id 查表。
                    anim: {
                        let a = enter_profile_id(cache.roles[j], reveal_kind);
                        // N4:spring 开关只换正文默认 profile(0→5);off 恒等。
                        if self.spring_enter && a == 0 {
                            5
                        } else {
                            a
                        }
                    },
                    alpha: code_alpha, // 行窗边缘淡入淡出(Plan 15 ①;非代码块恒 1)
                    exit_time: view_exit, // N3:退场 dissolve 起点(0=无,恒等)
                });
            }
            // 数学块(Plan 12 ②③):RaTeX 排版 → 数学 SDF 字形(em×px → world)+ 规则线。字号 = 正文
            // `math_em`(行内贴正文)或 ×1.3 = H3(显示数学,更舒展);显示数学**整式水平居中**。
            // spawn = 该块已揭字最晚上屏时刻(随块揭示淡入);未揭则跳过。glyph_idx 用高位基避免 morph 撞。
            for (mi, &((s, e), ref m, display)) in cache.math.iter().enumerate() {
                let mut spawn = 0.0f32;
                let mut revealed = false;
                for j in s..e {
                    if let Some(Some(t)) = view.spawn.get(j as usize).copied() {
                        spawn = spawn.max(t);
                        revealed = true;
                    }
                }
                if !revealed {
                    continue; // 该数学块尚未揭示
                }
                let px = if display {
                    self.math_em * DISPLAY_MATH_SCALE
                } else {
                    self.math_em
                };
                // y:数学基线对齐文本基线(文本基线 ≈ 字顶 + 0.8×字高;数学盒基线在盒顶下 height×px)。
                // x:行内取 run 首字左缘;显示数学**在盒内水平居中**((盒宽 - 整式宽)/2,夹 ≥0)。
                // 均叠加盒 origin(Plan 13:数学随 view 盒平移)。
                let pos = cache
                    .placed
                    .get(s as usize)
                    .map_or([origin[0], origin[1]], |p| {
                        [p.pos[0] + origin[0], p.pos[1] + origin[1] + p.size[1] * 0.8]
                    });
                let ox = if display {
                    origin[0] + ((box_w - m.width * px) * 0.5).max(0.0)
                } else {
                    pos[0]
                };
                let math_origin = [ox, pos[1] - m.height * px];
                let (mg, mr) =
                    crate::math::math_to_frame(m, math_origin, px, id as u32, spawn, MATH_COLOR);
                for (k, mut g) in mg.into_iter().enumerate() {
                    // Plan 31 R2:数学字稳定身份(非区间偏移/顺序位 —— 上文插入/公式扩展时
                    // 既有字保身份 → morph 平滑移动)。
                    g.glyph_idx = MATH_IDX_BASE
                        + view
                            .math_ids
                            .get(mi)
                            .and_then(|v| v.get(k))
                            .copied()
                            .unwrap_or(s + k as u32);
                    glyphs.push(g);
                }
                rects.extend(mr);
            }
            // 图片纹理 quad / 动图 overlay(Plan 14 ③):占位盒 = alt 字形 AABB(origin 偏移),尺寸优先
            // 用解码自然尺寸(④ reportSize 会让排版预留更准)。动图 → FrameEmbed(DOM 自播);否则纹理。
            for &(ei, (s, e), animated, (nw, nh), tex, alpha) in &ready_embeds {
                let slice = cache
                    .placed
                    .get(s as usize..(e as usize).min(cache.placed.len()))
                    .unwrap_or(&[]);
                let (mut x0, mut y0, mut x1, mut y1) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
                for p in slice {
                    x0 = x0.min(p.pos[0]);
                    y0 = y0.min(p.pos[1]);
                    x1 = x1.max(p.pos[0] + p.size[0]);
                    y1 = y1.max(p.pos[1] + p.size[1]);
                }
                if x1 < x0 {
                    continue; // 区间无墨(异常)
                }
                let pos = [x0 + origin[0], y0 + origin[1]];
                let size = if nw > 0.0 && nh > 0.0 {
                    [nw, nh]
                } else {
                    [x1 - x0, y1 - y0]
                };
                // Plan 31 R4(0036):静态 `data:image/svg+xml` 且白名单命中 → **矢量原生**
                // (每图元 FrameRect,任意缩放锐利,分流不走纹理);降级/动画照旧。
                let vector_svg = (!animated)
                    .then(|| cache.embeds.get(ei).map(|r| r.url.as_str()))
                    .flatten()
                    .and_then(svg_source_from_data_uri)
                    .map(|src| crate::svg::parse_svg(&src))
                    .filter(|l| l.degraded == 0 && !l.rects.is_empty());
                if let Some(layout) = vector_svg {
                    let sx = size[0] / layout.view_w.max(1.0);
                    let sy = size[1] / layout.view_h.max(1.0);
                    for r in &layout.rects {
                        rects.push(FrameRect {
                            pos: [pos[0] + r.x * sx, pos[1] + r.y * sy],
                            size: [r.w * sx, r.h * sy],
                            color: r.color,
                            radius: r.radius * sx.min(sy),
                            stroke: 0.0,
                            fx: [0.0; 4],
                            fx_color: [0.0; 4],
                            gloop: [0.0; 4],
                        });
                    }
                    // v1:<text> 图元记降级遗留(glyph 链接入随首个真实需求);此处白名单已过滤。
                } else if animated {
                    frame_embeds.push(crate::FrameEmbed {
                        key: Self::embed_key(id, ei),
                        url: cache
                            .embeds
                            .get(ei)
                            .map(|r| r.url.clone())
                            .unwrap_or_default(),
                        pos,
                        size,
                    });
                } else if alpha < 1.0 {
                    // 溶解淡入(Plan 16 ④):淡入窗内用 `ShaderId::Channel` 把纹理(channel0=tex)按噪声
                    // 阈值「溶解」显出(`params[0]`=进度=alpha);窗满(alpha=1)切回静态 image quad(下分支),
                    // 边界两者皆全显 → 无缝交接。护栏:离屏 cull + 计像素;溶解由进度驱动故 dynamic=false。
                    let irect = Rect::new(pos[0], pos[1], size[0], size[1]);
                    if irect.intersects(&visible) {
                        let mut params = [0.0f32; 8];
                        params[0] = alpha; // 溶解进度(0→1)
                        shaderboxes.push(crate::FrameShaderBox {
                            pos,
                            size,
                            shader_id: crate::ShaderId::Channel.as_u32(),
                            params,
                            bg: [0.0, 0.0, 0.0, 0.0],
                            time: 0.0,
                            dynamic: false,
                            channel0: tex,
                        });
                        shaderbox_pixels += irect.overlap_area(&visible).round().max(0.0) as u64;
                    }
                } else {
                    images.push(crate::FrameImage {
                        pos,
                        size,
                        tex_id: tex,
                        alpha, // 全显(淡入完成);窗内由上分支的溶解 ShaderBox 接管
                        radius: 6.0,
                    });
                }
            }
            // 复制图标(Plan 15 ③ → Plan 16 §2.7:程序化 ShaderBox icon,不再用纹理):每个代码块右上角
            // 钉一个 `ShaderId::Icons` 画板(`params[0]`=copy icon),**不随 scroll**(块相对固定)。dynamic
            // 随 icon 标志(呼吸);time 取节流时钟(护栏4)。
            for cb in &cache.code_blocks {
                let icon = crate::IconId::copy();
                let pos = [
                    origin[0] + box_w - COPY_ICON_PX - COPY_ICON_PAD,
                    origin[1] + cb.top_y + COPY_ICON_PAD,
                ];
                // 护栏1 cull:box 世界 rect ∉ 视口 → 不发(离屏零耗,不计度量)。
                let box_rect = Rect::new(pos[0], pos[1], COPY_ICON_PX, COPY_ICON_PX);
                if !box_rect.intersects(&visible) {
                    continue;
                }
                let mut params = [0.0f32; 8];
                params[0] = icon.as_u32() as f32;
                let dynamic = icon.is_dynamic();
                shaderboxes.push(crate::FrameShaderBox {
                    pos,
                    size: [COPY_ICON_PX, COPY_ICON_PX],
                    shader_id: crate::ShaderId::Icons.as_u32(),
                    params,
                    bg: [0.0, 0.0, 0.0, 0.0],
                    // 护栏2 静态即冻:静态 icon 不喂 time(常量 → GPU 结果逐帧不变,可冻复用);
                    // 仅 dynamic 走节流时钟(护栏4,30fps)。
                    time: if dynamic {
                        self.shaderbox_clock.time_s()
                    } else {
                        0.0
                    },
                    dynamic,
                    channel0: 0,
                });
                // 度量:屏上像素 = box∩viewport 面积(护栏度量,离屏已 cull 不计)。
                shaderbox_pixels += box_rect.overlap_area(&visible).round().max(0.0) as u64;
            }
            if glyphs.len() > glyphs_before {
                visible_blocks += 1;
            }
        }
        let e_emit = bf_t0.elapsed(); // ← emit 段止
        self.code_hit_rects = hit_rects; // Plan 15 ④:本帧代码块命中矩形(供 code_block_at 路由)
                                         // 调试几何叠加(Plan 4C3):块 AABB(描边)+ 视口框 + **内容节点框(Plan 7E / 0020)**。
        if self.debug_geometry {
            for &(id, origin, box_w, h) in &drawable {
                if !Rect::new(origin[0], origin[1], box_w, h).intersects(&visible) {
                    continue;
                }
                rects.push(FrameRect {
                    pos: origin,
                    size: [box_w, h],
                    color: self.theme.dbg_block,
                    radius: 0.0,
                    stroke: 1.5,
                    fx: [0.0; 4],
                    fx_color: [0.0; 4],
                    gloop: [0.0; 4],
                });
                // 节点树:逐容器节点描其 glyph range 的 AABB(肉眼验树,复用 4C3 叠加,7E)。
                if let Some(cache) = self.views[id].cache.as_ref() {
                    node_debug_rects(&cache.nodes, &cache.placed, origin, &mut rects);
                }
            }
            rects.push(FrameRect {
                pos: [visible.x, visible.y],
                size: [visible.w, visible.h],
                color: self.theme.dbg_view,
                radius: 0.0,
                stroke: 2.0,
                fx: [0.0; 4],
                fx_color: [0.0; 4],
                gloop: [0.0; 4],
            });
        }

        // ShaderBox 画廊(Plan 16 调试):视口左上钉一格栅,逐格一个内置 shader → 一屏验全盘上屏。
        // 屏锚(锚 `visible` 左上 → 随相机平移固定屏上)。50 icon + glow_orb + raymarch。
        if self.shaderbox_gallery {
            self.push_shaderbox_gallery(&visible, &mut shaderboxes, &mut shaderbox_pixels);
            // Plan 36 N2:距离带三件演示条(glow / 解析 shadow / 条纹带),钉画廊格栅下方。
            // 仅 ?gallery 下发射(默认视图零影响);e2e golden 据此锁三效果。
            let fy = visible.y + 480.0;
            let demo = |x: f32, fx: [f32; 4], color: [f32; 4], fx_color: [f32; 4]| FrameRect {
                pos: [visible.x + x, fy],
                size: [120.0, 72.0],
                color,
                radius: 10.0,
                stroke: 0.0,
                gloop: [0.0; 4],
                fx,
                fx_color,
            };
            // glow:k=0.18,外扩 R=24(quad 已含 halo 区);青色晕。
            rects.push(demo(
                16.0,
                [1.0, 0.18, 24.0, 0.0],
                [0.12, 0.12, 0.16, 1.0],
                [0.0, 0.81, 0.73, 0.9],
            ));
            // 解析 shadow:σ=8,外扩 3σ=24;黑影(单独一块,无叠底)。
            rects.push(demo(
                156.0,
                [2.0, 8.0, 24.0, 0.0],
                [0.0; 4],
                [0.0, 0.0, 0.0, 0.8],
            ));
            // 条纹带:L=8,duty=0.5;暗底 + 亮带。
            rects.push(demo(
                296.0,
                [3.0, 8.0, 0.5, 0.0],
                [0.12, 0.12, 0.16, 1.0],
                [0.75, 0.83, 0.98, 0.5],
            ));
        }

        // Plan 18 §2.1 规模度量:驻留量(含离屏/已冻块,不止可见)= 0029 before/after 主指标。
        // `Vec::len()` O(1) → 整体 O(views) 每帧,廉价(不扫文本/几何内容)。
        let (mut retained_glyphs, mut retained_nodes) = (0usize, 0usize);
        let mut tier_counts = [0usize; 4]; // [Hot, Warm, Cold, FrozenFar](Plan 29 V2)
        for v in &self.views {
            match v.tier {
                Tier::Hot => tier_counts[0] += 1,
                Tier::Warm => tier_counts[1] += 1,
                Tier::Cold => tier_counts[2] += 1,
                Tier::FrozenFar => tier_counts[3] += 1,
            }
            if let Some(c) = &v.cache {
                retained_glyphs += c.placed.len();
                retained_nodes += c.nodes.len();
            }
        }
        // F1:反馈记账(与 FrameData.feedback 同判据)。
        let fb_active = self.feedback_decay > 0.0
            && self.now_ms - self.last_activity_ms <= feedback_settle_ms(self.feedback_decay);
        let fb_rt_bytes = if fb_active {
            let vp = self.camera.viewport();
            2 * (vp[0] as usize) * (vp[1] as usize) * 4
        } else {
            0
        };
        self.last_stats = FrameStats {
            feedback_active: fb_active, // F1:发射后的有效 decay > 0
            rt_bytes: fb_rt_bytes,      // 2×w×h×4(device px;off = 0,0040 口径)
            frame_glyphs: glyphs.len(),
            total_glyphs,
            visible_blocks,
            total_blocks: drawable.len(),
            shaderbox_active: shaderboxes.len(),
            anim_groups,
            shaderbox_pixels,
            store_chars: self.store.char_count(),
            retained_views: self.views.len(),
            retained_glyphs,
            retained_nodes,
            tier_counts,
            rebuilds_this_frame: self.last_rebuilds,
            selection_rects: selection_rect_count,
        };
        self.last_visible = visible_recs; // Plan 21:本帧可见块世界盒(供 visible_messages/text_runs)
        self.ask_targets = new_ask_targets; // Plan 27:本帧 ask 按钮命中盒(tap/hover 路由)
        self.fold_targets = new_fold_targets; // D4:折叠汇总行命中盒
        self.link_targets = new_link_targets; // S2:链接命中盒(settled 块)
                                              // Plan 19 P2:工作集回收(用本帧 drawable 位置 + 视口)。滞回带:进 promote 带→Hot(下帧
                                              // ensure_layouts 重建);Hot 且 settled 且超 release 带→Warm(释放重几何,留 agg 占位)。
                                              // 带宽以视口高为单位,release > promote → 不 thrash。屏锚/不可逆操作前已落定。
        if self.virtualize {
            self.reclaim(&visible, &drawable);
        }
        // Plan 19 §2:三段差分写入(advance 在 frame() 写)。bf_total = 全 build_frame。
        let ms = |d: web_time::Duration| d.as_secs_f32() * 1000.0;
        self.last_phase_ms.bf_layout = ms(e_layout);
        self.last_phase_ms.bf_grid = ms(e_grid.saturating_sub(e_layout));
        self.last_phase_ms.bf_emit = ms(e_emit.saturating_sub(e_grid));
        self.last_phase_ms.bf_total = ms(bf_t0.elapsed());
        FrameData {
            feedback: {
                // F1:收敛自停 —— 静止超 settle 窗(decay^n<1/255)→ 本帧发 0(pass 退出);
                // 用户设定保留,活动恢复即重开(idle 全退场断言扩展到 pass 级)。
                let settled_long =
                    self.now_ms - self.last_activity_ms > feedback_settle_ms(self.feedback_decay);
                crate::frame::FeedbackParams {
                    decay: if settled_long {
                        0.0
                    } else {
                        self.feedback_decay
                    },
                    mix_mode: 0,
                }
            },
            post: crate::frame::PostParams {
                vignette: self.post_params[0],
                grain: self.post_params[1],
                chroma: self.post_params[2],
                // seed 只在 grain 开启时携带 —— 全零参数 = 真零面(F0 恒等断言口径)。
                grain_seed: if self.post_params[1] > 0.0 {
                    self.frame_counter
                } else {
                    0
                },
            },
            rects,
            panels,
            images,
            embeds: frame_embeds,
            widgets,
            shaderboxes,
            glyphs,
            time_ms: self.now_ms as f32,
            fade_ms: self.motion.dur_glyph,
            arrive_boost: self.motion.arrive_boost,
            cam_pan: self.camera.pan(),
            cam_zoom: self.camera.zoom(),
        }
    }

    /// Plan 19 P2 工作集回收(Hot⇄Warm)。用本帧 `drawable`(每块世界 y 区间)+ `visible` 视口,
    /// 按到视口的距离(以视口高为单位)滞回调档:
    /// - **进 promote 带**(视口 ±[`PROMOTE_MARGIN`] 屏)的 Warm → Hot(`cache=None` → 下帧
    ///   `ensure_layouts` 重建,源 = `revealed`,R8 确定 → 与释放前逐字节等价)。
    /// - **超 release 带**(视口 ±[`RELEASE_MARGIN`] 屏,> promote 带)的 Hot **且 `settled`**(0025 §4)
    ///   → Warm:`cache=None` 丢 placed/clusters/roles/strike/nodes/math/embeds,只留 `agg` 占位。
    ///
    /// `release > promote` 形成滞回带 → 带内不翻档,**防 thrash**。promote 带宽于视口 → 块在真正
    /// 滚入前已重建,无空白帧。`agg`/`revealed` 永不释放 → 布局稳定(0029 §3,零跳变之根)。
    fn reclaim(&mut self, visible: &Rect, drawable: &[(usize, [f32; 2], f32, f32)]) {
        // 分级距离(0029 §4;带宽 Plan 29 实测定):promote < warm < cold < frozen,逐级滞回。
        const PROMOTE_MARGIN: f32 = 1.5; // 视口外 1.5 屏内 → Hot(提前重建,滚入无空白帧)
        const WARM_MARGIN: f32 = 3.0; // >3 屏 → Warm(释几何;>promote 滞回带防 thrash)
        const COLD_MARGIN: f32 = 8.0; // >8 屏 → Cold(再丢 revealed/spawn)
        const FROZEN_MARGIN: f32 = 24.0; // >24 屏 → FrozenFar(释 Store text;仅 resync 源可用)
        let vh = visible.h.max(1.0);
        let (vtop, vbot) = (visible.y, visible.y + visible.h);
        let band = |m: f32| (vtop - vh * m, vbot + vh * m);
        let (promote_lo, promote_hi) = band(PROMOTE_MARGIN);
        let (warm_lo, warm_hi) = band(WARM_MARGIN);
        let (cold_lo, cold_hi) = band(COLD_MARGIN);
        let (frozen_lo, frozen_hi) = band(FROZEN_MARGIN);
        let can_freeze = self.resync_available;
        let mut rehydrate: Vec<usize> = Vec::new();
        for &(i, origin, _w, h) in drawable {
            let (top, bot) = (origin[1], origin[1] + h);
            let within_promote = bot >= promote_lo && top <= promote_hi;
            let beyond = |lo: f32, hi: f32| bot < lo || top > hi;
            let v = &mut self.views[i];
            match v.tier {
                Tier::Warm if within_promote => v.tier = Tier::Hot, // 下帧 ensure_layouts 重建
                Tier::Cold | Tier::FrozenFar if within_promote => rehydrate.push(i),
                Tier::Hot if beyond(warm_lo, warm_hi) && v.settled => {
                    v.tier = Tier::Warm;
                    v.cache = None; // 释重几何;agg/revealed/spawn 保留(0029 §3 布局不动)
                }
                Tier::Warm if beyond(cold_lo, cold_hi) => {
                    v.tier = Tier::Cold;
                    // 丢 grapheme 级驻留(长会话内存大头);重建 = display_source 重切(确定性)。
                    v.revealed = Vec::new();
                    v.spawn = Vec::new();
                    // 0029 §88:随块回收行窗滚动态 / 嵌入注册(键高 32 位 = 块下标);
                    // 重建时同键重登(确定性),丢的只是「滚动位置」等 presentation 态。
                    let hi = (i as u64) << 32;
                    self.code_scroll
                        .retain(|k, _| k & 0xFFFF_FFFF_0000_0000 != hi);
                    self.image_registry
                        .retain(|k, _| k & 0xFFFF_FFFF_0000_0000 != hi);
                }
                Tier::Cold if can_freeze && beyond(frozen_lo, frozen_hi) => {
                    v.tier = Tier::FrozenFar;
                    let pid = v.part_id.clone();
                    self.store.release_text(&pid); // V3:重入走 0003 resync catch-up
                }
                _ => {}
            }
        }
        for i in rehydrate {
            self.rehydrate(i);
        }
    }

    /// Cold/FrozenFar → Hot 重建(0029 §87):display_source 重切 grapheme,spawn 全
    /// `Some(-1e9)`(catch-up 恒等收敛 → 零动画瞬显,AR6);cache 交下帧 `ensure_layouts`
    /// (纯函数,同输入同产物 → 高度/几何与释放前逐字节一致,零跳变)。FrozenFar 且 text 已释
    /// → 触发 resync 重取(V3),本帧凭 agg 占位。
    fn rehydrate(&mut self, i: usize) {
        let part_id = self.views[i].part_id.clone();
        let Some(ds) = self.store.display_source(&part_id) else {
            return;
        };
        if ds.is_empty() && self.store.text_released(&part_id) {
            self.request_resync(); // V3:FrozenFar 重入 → server 快照重取;等待期 agg 占位
            return;
        }
        let gs = graphemes(&ds);
        let v = &mut self.views[i];
        v.revealed = gs.iter().map(|g| ((*g).to_owned(), 1.0)).collect();
        v.spawn = vec![Some(-1e9); v.revealed.len()];
        v.settled = true;
        v.cache = None;
        v.tier = Tier::Hot;
    }

    /// ShaderBox 画廊格栅(Plan 16 调试,`set_shaderbox_gallery(true)`)。在 `visible` 视口左上铺
    /// 一格栅:50 个 icon(`ShaderId::Icons`,`params[0]`=icon_id)+ glow_orb + raymarch 各一格,
    /// 屏锚(锚视口左上 → 内容滚动时格栅固定屏上)。每格计入 shaderbox 度量。GPU 上屏验全盘 shader。
    fn push_shaderbox_gallery(
        &self,
        visible: &Rect,
        shaderboxes: &mut Vec<crate::FrameShaderBox>,
        shaderbox_pixels: &mut u64,
    ) {
        const TILE: f32 = 40.0;
        const GAP: f32 = 8.0;
        const MARGIN: f32 = 16.0;
        let pitch = TILE + GAP;
        let cols = (((visible.w - 2.0 * MARGIN) / pitch).floor() as usize).max(1);
        let t = self.shaderbox_clock.time_s();
        // 50 个 icon(id 0..=49)+ glow_orb + raymarch;按行列铺格。
        let mut emit = |k: usize, shader_id: u32, params: [f32; 8], dynamic: bool| {
            let (col, row) = (k % cols, k / cols);
            let pos = [
                visible.x + MARGIN + col as f32 * pitch,
                visible.y + MARGIN + row as f32 * pitch,
            ];
            let rect = Rect::new(pos[0], pos[1], TILE, TILE);
            if !rect.intersects(visible) {
                return; // 护栏1:滚出视口的格不发
            }
            shaderboxes.push(crate::FrameShaderBox {
                pos,
                size: [TILE, TILE],
                shader_id,
                params,
                bg: [0.09, 0.09, 0.12, 1.0], // 暗底 → icon 覆盖率清晰可辨
                time: if dynamic { t } else { 0.0 }, // 护栏2:静态 icon 冻
                dynamic,
                channel0: 0,
            });
            *shaderbox_pixels += rect.overlap_area(visible).round().max(0.0) as u64;
        };
        // 0..49 = PixelSpiritDeck 整盘;50..65 = opencode tool icon(plan16-tool-icons,全 dynamic)。
        for icon in 0u32..crate::ICON_COUNT {
            let mut params = [0.0f32; 8];
            params[0] = icon as f32; // p0.x = icon_id(无 morph:p0.z=0)
            let dynamic = !matches!(icon, 0 | 15 | 18 | 48); // 4 个静态(护栏2),tool icon 全动
            emit(
                icon as usize,
                crate::ShaderId::Icons.as_u32(),
                params,
                dynamic,
            );
        }
        let base = crate::ICON_COUNT as usize;
        // glow_orb(默认蓝环,常态脉冲)。
        let mut orb = [0.0f32; 8];
        orb[3] = 1.0; // p0.w = 脉冲速度
        emit(base, crate::ShaderId::GlowOrb.as_u32(), orb, true);
        // raymarch(留位 3D SDF)。
        emit(
            base + 1,
            crate::ShaderId::Raymarch.as_u32(),
            [0.0f32; 8],
            true,
        );
        // noise 四象限(Plan 36 N1 debug:value/gradient/fbm/voronoi;p0.x=seed;静态帧恒等)。
        let mut nz = [0.0f32; 8];
        nz[0] = 7.0; // seed
        emit(base + 2, crate::ShaderId::NoiseDebug.as_u32(), nz, false);
    }

    /// 取或建某 part 的视图(保持 store 顺序)。
    /// 每帧从 store 校正各 view 角色(Plan 13):live delta 创建时角色未知(默认 Assistant),待
    /// snapshot/resync 写入 `message_role` 后校正为真实角色(如 user)。仅校正、不新建。
    fn refresh_roles(&mut self) {
        for i in 0..self.views.len() {
            let role = self.store.part_role(&self.views[i].part_id);
            self.views[i].role = role;
        }
    }

    fn view_mut(&mut self, part_id: &str) -> &mut PartView {
        if let Some(idx) = self.views.iter().position(|v| v.part_id == part_id) {
            return &mut self.views[idx];
        }
        let role = self.store.part_role(part_id); // Plan 13:角色定左右分栏(未知默认 Assistant)
        self.views.push(PartView {
            exiting: None,
            part_id: part_id.to_owned(),
            pushed: 0,
            pushed_bytes: 0,
            revealed: Vec::new(),
            cache: None,
            spawn: Vec::new(),
            instant: false,
            settled: false,
            role,
            tier: Tier::Hot,
            agg: None,
            glyph_ids: Vec::new(),
            id_clusters: Vec::new(),
            next_glyph_id: 0,
            math_ids: Vec::new(),
            math_id_clusters: Vec::new(),
            next_math_id: 0,
            badge_morph: None,
            last_card_status: None,
        });
        self.views.last_mut().expect("just pushed") // reason: 上面刚 push
    }
}

#[cfg(test)]
mod tests {
    use super::{assign_stable_ids, band_adjacent, group_turns, PartView, Tier};
    use crate::content::StyleRole;
    use crate::record::Player;
    use crate::support::{CollectSink, MonospaceLayout};
    use crate::{Engine, FrameData, FrameRect};
    use proptest::{prop_assert, prop_assert_eq};

    fn delta(part: &str, delta: &str) -> String {
        format!(
            r#"{{"type":"message.part.delta","properties":{{"sessionID":"s","messageID":"m","partID":"{part}","field":"text","delta":{delta:?}}}}}"#
        )
    }

    // ───────────── Plan 34 S2 · 可点链接(0032 ② + onOpenUrl)─────────────

    /// S2:settled 块的链接进命中盒;tap 中心 → 待派发 URL(core 不导航,宿主取走);
    /// 取即清(幂等);未命中处 tap 不产 URL。
    #[test]
    fn link_tap_sets_pending_open_url() {
        let recs = vec![(
            0.0,
            delta("p1", "文档见 [官网](https://example.com/docs) 结尾。"),
        )];
        let mut eng = Engine::new(
            Player::from_pairs(recs, 16.0),
            MonospaceLayout::default(),
            CollectSink::default(),
            1_000_000.0,
            800.0,
        );
        eng.set_reveal_cps(1.0e9);
        for _ in 0..80 {
            eng.frame(16.0); // 揭示完 + settle(链接命中盒只收 settled 块,AR3)
        }
        let targets: Vec<(String, crate::Rect)> = eng.link_targets().to_vec();
        assert_eq!(targets.len(), 1, "settled 后链接命中盒登记");
        assert_eq!(targets[0].0, "https://example.com/docs");
        let r = &targets[0].1;
        let (pan, zoom) = (eng.camera().pan(), eng.camera().zoom());
        let (sx, sy) = (
            (r.x + r.w * 0.5 - pan[0]) * zoom,
            (r.y + r.h * 0.5 - pan[1]) * zoom,
        );
        assert!(eng.tap(sx, sy), "tap 命中链接");
        assert_eq!(
            eng.take_pending_open_url().as_deref(),
            Some("https://example.com/docs")
        );
        assert_eq!(eng.take_pending_open_url(), None, "取即清");
        assert!(!eng.tap(sx + 5000.0, sy), "空白处不命中");
        assert_eq!(eng.take_pending_open_url(), None);
    }

    /// S4:播报单元 = settled part(整段一次,绝不逐 token);瞬显/catch-up 零播报(AR6);
    /// 取即清。
    #[test]
    fn settle_announcements_per_part_not_per_token() {
        let recs = vec![
            (0.0, delta("p1", "第一段完整回复。")),
            (500.0, delta("p2", "第二段完整回复。")),
        ];
        let mut eng = Engine::new(
            Player::from_pairs(recs, 16.0),
            MonospaceLayout::default(),
            CollectSink::default(),
            1_000_000.0,
            800.0,
        );
        eng.set_reveal_cps(1.0e9);
        let mut all: Vec<(String, String)> = Vec::new();
        for _ in 0..120 {
            eng.frame(16.0);
            all.extend(eng.take_settle_announcements());
        }
        // p1 增长会两次 settle?不:p1 一次到齐 → settle 一次;p2 同。逐 token 会是几十条。
        let texts: Vec<&str> = all.iter().map(|(_, t)| t.as_str()).collect();
        assert!(
            texts.contains(&"第一段完整回复。") && texts.contains(&"第二段完整回复。"),
            "整段文本各播一次: {texts:?}"
        );
        assert!(
            all.len() <= 3,
            "播报单元 = settled part,非逐 token: {}",
            all.len()
        );
        assert!(all.iter().all(|(r, _)| r == "assistant"), "角色标注");
        assert!(eng.take_settle_announcements().is_empty(), "取即清");

        // catch-up(快照瞬显)零播报(AR6)。
        let snap = r#"[{"info":{"id":"m1","sessionID":"s","role":"a"},
            "parts":[{"type":"text","id":"px","messageID":"m1","text":"历史内容"}]}]"#;
        let mut eng2 = Engine::new(
            Player::from_pairs(vec![], 16.0),
            MonospaceLayout::default(),
            CollectSink::default(),
            200.0,
            800.0,
        );
        eng2.prime_from_snapshot(snap);
        for _ in 0..30 {
            eng2.frame(16.0);
        }
        assert!(
            eng2.take_settle_announcements().is_empty(),
            "瞬显不播报(AR6)"
        );
    }

    /// S3:白名单外链接 —— 无命中盒、角色降级 Normal(视觉普通文本)、tap 不产派发;
    /// 白名单外图片 —— 不发起加载,登记即 Failed(alt 兜底)。
    #[test]
    fn url_policy_rejects_hostile_link_and_image() {
        let recs = vec![(
            0.0,
            delta(
                "p1",
                "点 [恶意](javascript:alert(1)) 和 [好链](https://ok.example/x)\n\n![图](file:///etc/passwd)",
            ),
        )];
        let mut eng = Engine::new(
            Player::from_pairs(recs, 16.0),
            MonospaceLayout::default(),
            CollectSink::default(),
            1_000_000.0,
            800.0,
        );
        eng.set_reveal_cps(1.0e9);
        for _ in 0..80 {
            eng.frame(16.0);
        }
        // 命中盒只有白名单内的一条。
        let urls: Vec<&str> = eng.link_targets().iter().map(|(u, _)| u.as_str()).collect();
        assert_eq!(urls, vec!["https://ok.example/x"], "恶意链接不进命中盒");
        // 恶意链接文字角色已降级(块内无 Link 角色残留在其区间;好链仍 Link)。
        let cache = eng.views[0].cache.as_ref().expect("cache");
        let link_role = crate::StyleRole::Link.as_u32();
        let link_glyphs = cache.roles.iter().filter(|&&r| r == link_role).count();
        assert_eq!(link_glyphs, 2, "只剩「好链」两个 glyph 是 Link 角色");
        // 恶意图片:登记即 Failed,不进待解码队列。
        let pending = eng.take_pending_images();
        assert!(pending.is_empty(), "file:// 图片不发起加载: {pending:?}");
    }

    // ───────────── Plan 34 S1 · 0038 滚动跟随 FSM(shadcn 9 场景)─────────────

    /// 场景用具:80 行长文 t=0 到齐,40 行增量 t=2000ms 到 —— 前者滚出视口(600px/18px 行),
    /// 后者测「脱离后增高」。base_cps 大 → 揭示即到即出。
    fn scroll_engine() -> Engine<Player, MonospaceLayout, CollectSink> {
        // 段落分隔(\n\n):markdown 会把单换行并进同段 → 用独立段落保证高度 ≫ 600 视口。
        let mut long = String::new();
        for i in 0..80 {
            use std::fmt::Write as _;
            let _ = write!(long, "line {i}\n\n");
        }
        let mut extra = String::new();
        for i in 80..120 {
            use std::fmt::Write as _;
            let _ = write!(extra, "line {i}\n\n");
        }
        let recs = vec![(0.0, delta("p1", &long)), (2000.0, delta("p1", &extra))];
        let mut eng = Engine::new(
            Player::from_pairs(recs, 16.0),
            MonospaceLayout::default(),
            CollectSink::default(),
            1_000_000.0,
            800.0,
        );
        eng.set_reveal_cps(1.0e9); // 到即出:场景测相机不测节奏
        eng
    }

    fn run_frames(eng: &mut Engine<Player, MonospaceLayout, CollectSink>, n: usize) {
        for _ in 0..n {
            eng.frame(16.0);
        }
    }

    /// ① 流式贴底跟随:默认 Following,揭示中相机贴「已揭示底」。
    #[test]
    fn follow_fsm_streaming_sticks_to_bottom() {
        use super::FollowState;
        let mut eng = scroll_engine();
        run_frames(&mut eng, 60); // 首块揭完(远超视口高)
        assert_eq!(eng.follow_state(), FollowState::Following);
        assert!(eng.camera().pan()[1] > 0.0, "内容超视口 → 已随滚");
    }

    /// ②⑥⑦⑧ + 缩放 + 选区 + jump:释放信号全集(0038 表)逐一 → Released。
    /// 滚轮/键盘滚动键 = scroll_by(dy<0);拖滚动条/触摸纵滚 = pan_by(0,dy<0);
    /// 触摸横滚 = pan_by(dx,0);缩放 = zoom_by;选区 = set_selection;jump = scroll_to。
    #[test]
    fn follow_fsm_release_signal_set() {
        use super::FollowState;
        type Sig = (
            &'static str,
            fn(&mut Engine<Player, MonospaceLayout, CollectSink>),
        );
        let signals: Vec<Sig> = vec![
            ("wheel/keyboard up", |e| e.scroll_by(-40.0)),
            ("scrollbar/touch up", |e| e.pan_by(0.0, -40.0)),
            ("touch horizontal", |e| e.pan_by(30.0, 0.0)),
            ("zoom", |e| e.zoom_by(1.2, 100.0, 100.0)),
            ("selection drag", |e| e.set_selection(vec![(0, 0, 5)])),
            ("explicit jump", |e| e.scroll_to(0)),
        ];
        for (name, sig) in signals {
            let mut eng = scroll_engine();
            run_frames(&mut eng, 60);
            assert_eq!(eng.follow_state(), FollowState::Following, "{name}: 前置");
            sig(&mut eng);
            eng.frame(16.0);
            assert_eq!(eng.follow_state(), FollowState::Released, "{name}: 应释放");
        }
    }

    /// ③ 脱离后增高不推视口:Released 下新内容到达,pan 恒定(布局稳定硬不变量)。
    #[test]
    #[allow(clippy::float_cmp)] // reason: 零位移就是**逐位相等**(0038 不变量),精确比较是断言本体
    fn follow_fsm_released_growth_zero_viewport_shift() {
        use super::FollowState;
        let mut eng = scroll_engine();
        run_frames(&mut eng, 60);
        eng.scroll_by(-200.0); // 上滚脱离(远离阈值)
        run_frames(&mut eng, 5);
        assert_eq!(eng.follow_state(), FollowState::Released);
        let pan_before = eng.camera().pan();
        run_frames(&mut eng, 200); // 越过 t=2000ms:40 行增量到达并揭示
        let pan_after = eng.camera().pan();
        assert_eq!(pan_before, pan_after, "增高零位移(0038 Released 不变量)");
        assert_eq!(
            eng.follow_state(),
            FollowState::Released,
            "增长不得触发重进入"
        );
    }

    /// ④ jump 回底:scroll_to_latest → Anchoring 平滑滚向底,到底转 Following。
    #[test]
    fn follow_fsm_jump_to_latest_anchors_then_follows() {
        use super::FollowState;
        let mut eng = scroll_engine();
        run_frames(&mut eng, 60);
        eng.scroll_by(-300.0);
        eng.frame(16.0);
        assert_eq!(eng.follow_state(), FollowState::Released);
        eng.scroll_to_latest();
        assert_eq!(eng.follow_state(), FollowState::Anchoring);
        run_frames(&mut eng, 60); // smooth-damp 收敛
        assert_eq!(eng.follow_state(), FollowState::Following, "到底即恢复跟随");
    }

    /// ⑤ 选区期间不跟随:选区释放后,流式增长不动相机(读者正在选的内容不被拽走)。
    #[test]
    #[allow(clippy::float_cmp)] // reason: 同上,零位移 = 逐位相等
    fn follow_fsm_selection_stops_following_through_growth() {
        use super::FollowState;
        let mut eng = scroll_engine();
        run_frames(&mut eng, 60);
        eng.set_selection(vec![(0, 2, 30)]);
        eng.frame(16.0);
        let pan = eng.camera().pan();
        run_frames(&mut eng, 200); // 增量到达
        assert_eq!(eng.camera().pan(), pan, "选区期间内容增长零位移");
        assert_eq!(eng.follow_state(), FollowState::Released);
    }

    /// ⑨ 重进入吸附:小幅上滚(阈值内)不得被动拉回(修旧缺陷);用户**主动向下**
    /// 抵底才 Following。
    #[test]
    fn follow_fsm_reenter_only_on_user_downscroll() {
        use super::FollowState;
        let mut eng = scroll_engine();
        run_frames(&mut eng, 60);
        eng.scroll_by(-20.0); // 阈值(48px)内的小幅上滚
        run_frames(&mut eng, 10);
        assert_eq!(
            eng.follow_state(),
            FollowState::Released,
            "阈值内上滚不得被动复跟随(旧缺陷)"
        );
        eng.scroll_by(60.0); // 用户主动向下抵底
        eng.frame(16.0);
        assert_eq!(
            eng.follow_state(),
            FollowState::Following,
            "主动向下抵底 → 吸附"
        );
    }

    /// Plan 37 F1:settle 数学 —— decay^n < 1/255 的帧数折 ms;单调、边界干净(纯函数 R8)。
    #[test]
    #[allow(clippy::float_cmp)] // reason: off=恰 0 / 确定性断言即精确相等语义
    fn feedback_settle_math() {
        use super::feedback_settle_ms;
        assert_eq!(feedback_settle_ms(0.0), 0.0, "off 即停");
        let s5 = feedback_settle_ms(0.5);
        let s9 = feedback_settle_ms(0.9);
        assert!(s5 > 0.0 && s9 > s5, "衰减越慢 settle 越长: {s5} {s9}");
        // decay 0.5:n = ceil(ln255/ln2) = 8 帧 ≈ 133ms。
        assert!((s5 - 8.0 * (1000.0 / 60.0)).abs() < 1.0, "{s5}");
        assert_eq!(feedback_settle_ms(0.5), s5, "确定性");
    }

    /// Plan 37 F1:收敛自停 —— 活动期 decay 透传 + 记账开;静止超窗自动归零(pass 退出),
    /// 活动恢复重开;off 恒零。双跑一致。
    #[test]
    #[allow(clippy::float_cmp)] // reason: off=恰 0 / 确定性断言即精确相等语义
    fn feedback_auto_stop_on_idle_and_resume() {
        let recs = vec![
            (0.0, delta("p1", "拖尾活动源")),
            (3000.0, delta("p1", " 追加再动一次")),
        ];
        let run = || {
            let mut eng = Engine::new(
                Player::from_pairs(recs.clone(), 16.0),
                MonospaceLayout::default(),
                CollectSink::default(),
                1_000_000.0,
                800.0,
            );
            eng.set_reveal_cps(1.0e9);
            eng.set_feedback_decay(0.85);
            let mut probes = Vec::new();
            for i in 0..260 {
                eng.frame(16.0);
                if [10usize, 150, 200].contains(&i) {
                    let f = eng.sink().last().expect("frame");
                    let st = eng.frame_stats();
                    probes.push((f.feedback.decay, st.feedback_active, st.rt_bytes));
                }
            }
            probes
        };
        let p = run();
        assert!(p[0].0 > 0.0 && p[0].1, "活动期透传+记账开: {p:?}");
        assert!(p[0].2 > 0, "RT 记账 = 2×w×h×4: {p:?}");
        assert_eq!(p[1].0, 0.0, "静止超窗自停(idle 退场 pass 级): {p:?}");
        assert!(!p[1].1 && p[1].2 == 0, "自停后记账归零: {p:?}");
        assert!(p[2].0 > 0.0, "3s 追加 → 活动恢复重开: {p:?}");
        assert_eq!(run(), p, "双跑一致(R8)");
    }

    /// Plan 37 F0(0040):默认帧反馈/后处理参数全零(off 直通面)+ 记账恒零。
    #[test]
    fn f0_default_frame_has_zero_feedback_and_post() {
        let recs = vec![(0.0, delta("p1", "pass 直通基线"))];
        let mut eng = Engine::new(
            Player::from_pairs(recs, 16.0),
            MonospaceLayout::default(),
            CollectSink::default(),
            1_000_000.0,
            800.0,
        );
        eng.set_reveal_cps(1.0e9);
        for _ in 0..20 {
            eng.frame(16.0);
        }
        let f = eng.sink().last().expect("frame");
        assert_eq!(
            f.feedback,
            crate::frame::FeedbackParams::default(),
            "off 全零"
        );
        assert_eq!(f.post, crate::frame::PostParams::default(), "post 全零");
        let st = eng.frame_stats();
        assert!(!st.feedback_active, "F0 恒直通");
        assert_eq!(st.rt_bytes, 0, "off 零 RT(0040 记账口径)");
    }

    /// Plan 36 N3:退场 dissolve —— off(默认)孤儿即时清除(旧行为恒等);开启后孤儿
    /// 先保留 cache 溶解(glyph 携 exit_time),到期真清除;双跑一致(R8)。
    #[test]
    fn exit_dissolve_defers_orphan_wipe_then_clears() {
        let removed = r#"{"type":"message.part.removed","properties":{"sessionID":"s","messageID":"m","partID":"p1"}}"#;
        let recs = vec![
            (0.0, delta("p1", "goodbye world")),
            (500.0, removed.to_owned()),
        ];
        let run = |dissolve_ms: f64| -> Vec<(usize, usize)> {
            let mut eng = Engine::new(
                Player::from_pairs(recs.clone(), 16.0),
                MonospaceLayout::default(),
                CollectSink::default(),
                1_000_000.0,
                800.0,
            );
            eng.set_reveal_cps(1.0e9);
            eng.set_exit_dissolve_ms(dissolve_ms);
            let mut probes = Vec::new();
            for i in 0..80 {
                eng.frame(16.0);
                if i == 20 || i == 40 || i == 79 {
                    let f = eng.sink().last().expect("frame");
                    let exiting = f.glyphs.iter().filter(|g| g.exit_time > 0.0).count();
                    probes.push((f.glyphs.len(), exiting));
                }
            }
            probes
        };
        // off:移除即消失,永无 exit_time。
        let off = run(0.0);
        assert!(off[0].0 > 0 && off[0].1 == 0, "移除前正常: {off:?}");
        assert_eq!(off[1].0, 0, "off: 移除即清");
        // on(300ms):40 帧(640ms,移除后 ~140ms)仍在渲染且携 exit_time;结束后清除。
        let on = run(300.0);
        assert!(
            on[1].0 > 0 && on[1].1 == on[1].0,
            "溶解中: 全部 glyph 携 exit_time: {on:?}"
        );
        assert_eq!(on[2].0, 0, "到期真清除(收敛,AR3)");
        assert_eq!(run(300.0), on, "双跑一致(R8)");
    }

    /// Plan 36 N2:距离带三件 —— ?gallery 下三演示条(mode 1/2/3)各在;
    /// 默认(非 gallery)路径全部 rect 的 fx.mode == 0(参数恒等,DoD「置零直通」数据面)。
    #[test]
    #[allow(clippy::float_cmp)] // reason: fx.mode 是判别值(0/1/2/3 整数存 f32),精确比较即语义
    fn fx_demo_rects_only_under_gallery_and_default_is_identity() {
        let recs = vec![(0.0, delta("p1", "hello *world*"))];
        let mut eng = Engine::new(
            Player::from_pairs(recs.clone(), 16.0),
            MonospaceLayout::default(),
            CollectSink::default(),
            1_000_000.0,
            800.0,
        );
        eng.set_reveal_cps(1.0e9);
        for _ in 0..30 {
            eng.frame(16.0);
        }
        let f = eng.sink().last().expect("frame");
        assert!(
            f.rects.iter().all(|r| r.fx[0] == 0.0),
            "默认路径零 fx(恒等直通)"
        );
        // gallery 开:三演示条 mode 1/2/3 各恰一。
        let mut eng2 = Engine::new(
            Player::from_pairs(recs, 16.0),
            MonospaceLayout::default(),
            CollectSink::default(),
            1_000_000.0,
            800.0,
        );
        eng2.set_shaderbox_gallery(true);
        eng2.frame(16.0);
        let f2 = eng2.sink().last().expect("frame");
        for mode in [1.0f32, 2.0, 3.0] {
            assert_eq!(
                f2.rects.iter().filter(|r| r.fx[0] == mode).count(),
                1,
                "gallery 下 fx mode {mode} 演示条恰一"
            );
        }
    }

    /// Plan 32 D4:scale 指数逼近确定性(R8)+ 收敛恒等(AR3)。
    #[test]
    #[allow(clippy::float_cmp)] // reason: snap 语义就是**恰好 1.0**(AR3 恒等),精确比较是断言本体
    fn fold_scale_deterministic_and_converges_to_identity() {
        use super::fold_scale_step;
        // 同 dt 序列双跑逐帧一致(R8)。
        let run = |dts: &[f32]| -> Vec<f32> {
            let mut s = super::FOLD_SCALE_START;
            dts.iter()
                .map(|&dt| {
                    s = fold_scale_step(s, dt);
                    s
                })
                .collect()
        };
        let dts = [16.7f32, 16.7, 33.4, 8.0, 16.7, 16.7, 16.7, 50.0];
        assert_eq!(run(&dts), run(&dts), "注入 dt 双跑一致");
        // 60fps 一帧恰 ×0.9(makepad 语义):距离 1 的残差缩为 0.9 倍。
        let s1 = fold_scale_step(0.9, 1000.0 / 60.0);
        assert!((1.0 - s1 - 0.09).abs() < 1e-4, "s1={s1}");
        // 有限步收敛到**恰好 1.0**(snap;此后恒等 AR3)。
        let mut s = super::FOLD_SCALE_START;
        let mut n = 0;
        while s < 1.0 && n < 120 {
            s = fold_scale_step(s, 16.7);
            n += 1;
        }
        assert_eq!(s, 1.0, "{n} 帧后 snap 到恒等");
        assert_eq!(fold_scale_step(1.0, 16.7), 1.0, "收敛后恒等(AR3)");
    }

    /// Plan 32 D4:tap 折叠汇总行 → 展开(0032 命中路径);展开态在 view 级持久字段。
    #[test]
    fn tap_on_fold_summary_expands_context() {
        let part_updated = |json: &str| -> String {
            format!(
                r#"{{"type":"message.part.updated","properties":{{"part":{json},"time":1.0}}}}"#
            )
        };
        // edit 工具:6 行 context + 1 删 1 增 → 默认折叠出「⋯ 6 unchanged lines」。
        let diff = "@@ -1,8 +1,8 @@\n c0\n c1\n c2\n c3\n c4\n c5\n-x\n+y\n";
        let tool = format!(
            r#"{{"id":"t1","messageID":"m","sessionID":"s","type":"tool","tool":"edit","state":{{"status":"completed","input":{{"path":"a.rs"}},"metadata":{{"filediff":{diff:?}}}}}}}"#
        );
        let recs = vec![(0.0, part_updated(&tool))];
        let mut eng = Engine::new(
            Player::from_pairs(recs, 16.0),
            MonospaceLayout::default(),
            CollectSink::default(),
            100_000.0,
            800.0,
        );
        for _ in 0..60 {
            eng.frame(16.0);
        }
        let text = eng.sink().visible_text();
        assert!(text.contains("6 unchanged lines"), "默认折叠: {text}");
        assert!(!text.contains("c3"), "折叠行不可见: {text}");
        let (_, r) = eng.fold_targets.first().expect("折叠命中盒已登记");
        let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
        // 世界 → 屏幕(tap 期望屏幕 px;默认 pan/zoom 下同值,仍按变换算保健壮)。
        let (pan, zoom) = (eng.camera.pan(), eng.camera.zoom());
        assert!(
            eng.tap((cx - pan[0]) * zoom, (cy - pan[1]) * zoom),
            "tap 命中折叠行"
        );
        for _ in 0..60 {
            eng.frame(16.0);
        }
        let text = eng.sink().visible_text();
        assert!(!text.contains("unchanged lines"), "展开后无汇总行: {text}");
        assert!(
            text.contains("c0") && text.contains("c5"),
            "上下文行已展开: {text}"
        );
        // 展开态入 view 级持久字段(Cold 重建走同一 RenderCtx 位图 → 保持)。
        assert!(!eng.diff_unfold.is_empty());
        // 动画收敛后包络移除(AR3 恒等)。
        assert!(eng.fold_anim.is_empty(), "scale 已 snap 收敛");
    }

    /// Plan 32 D3:gloop 邻接判定 —— 同 group 同 slot 且贴邻才融;换色/跨 hunk/有缝均断链;
    /// 单行两侧皆无(退化独立圆角)。确定性纯函数(R8)。
    #[test]
    fn band_gloop_adjacency_chains_same_color_only() {
        use crate::partrender::DecorSlot::{DiffAddBand, DiffDelBand};
        // hunk0:del,del | add;hunk1(有缝再起):add。行高 16,y 连续。
        let bands = vec![
            (0u32, DiffDelBand, 0.0f32, 16.0f32),
            (0, DiffDelBand, 16.0, 32.0),
            (0, DiffAddBand, 32.0, 48.0), // 贴邻但换色 → 不融
            (1, DiffAddBand, 80.0, 96.0), // 有缝 + 换 hunk → 独立
        ];
        assert_eq!(band_adjacent(&bands, 0), (false, true));
        assert_eq!(band_adjacent(&bands, 1), (true, false), "下邻换色不融");
        assert_eq!(band_adjacent(&bands, 2), (false, false), "上邻换色、下有缝");
        assert_eq!(band_adjacent(&bands, 3), (false, false), "单行退化");
        // 同色同 group 但中间有缝(折叠行隔开)→ 断链。
        let gap = vec![
            (0u32, DiffAddBand, 0.0f32, 16.0f32),
            (0, DiffAddBand, 48.0, 64.0),
        ];
        assert_eq!(band_adjacent(&gap, 0), (false, false));
        assert_eq!(band_adjacent(&gap, 1), (false, false));
    }

    /// Plan 22 P3 + Plan 23:非文本 part(tool/reasoning)渲染出来且内容可见。
    /// Plan 23 起 reasoning/tool 走 registry 的 specific 漂亮渲染器(`▸ bash [done]` / `💭 Thinking`),
    /// 替代 Plan 22 的 `[tool:…]` markdown 兜底;内容(input/正文)仍完整可见。
    #[test]
    fn p3_nontext_parts_render_via_fallback() {
        let part_updated = |json: &str| -> String {
            format!(
                r#"{{"type":"message.part.updated","properties":{{"part":{json},"time":1.0}}}}"#
            )
        };
        let recs = vec![
            (
                0.0,
                part_updated(
                    r#"{"type":"tool","id":"t1","messageID":"m1","tool":"bash","state":{"status":"completed","input":{"cmd":"ls"}}}"#,
                ),
            ),
            (
                0.0,
                part_updated(
                    r#"{"type":"reasoning","id":"r1","messageID":"m1","text":"先看要点"}"#,
                ),
            ),
        ];
        let mut eng = Engine::new(
            Player::from_pairs(recs, 16.0),
            MonospaceLayout::default(),
            CollectSink::default(),
            100_000.0,
            800.0,
        );
        for _ in 0..60 {
            eng.frame(16.0);
        }
        let text = eng.sink().visible_text();
        assert!(text.contains("Shell"), "工具显示名应可见: {text}");
        assert!(
            text.contains("cmd") || text.contains("$ ls"),
            "工具载荷应可见: {text}"
        );
        assert!(
            text.contains("先看要点"),
            "推理正文应可见(裸弱化,无标题): {text}"
        );
    }

    /// Plan 23 R2/R3:tool 卡发卡底面板 rect(CARD_BG),edit diff 发增/删行底色带(DIFF_ADD/DEL_BG)。
    #[test]
    fn r2r3_tool_card_and_diff_emit_decorations() {
        let part_updated = |json: &str| -> String {
            format!(
                r#"{{"type":"message.part.updated","properties":{{"part":{json},"time":1.0}}}}"#
            )
        };
        let recs = vec![(
            0.0,
            part_updated(
                r#"{"type":"tool","id":"e1","messageID":"m1","tool":"edit","state":{"status":"completed","metadata":{"filediff":"@@ -1 +1 @@\n-old\n+new\n"}}}"#,
            ),
        )];
        let mut eng = Engine::new(
            Player::from_pairs(recs, 16.0),
            MonospaceLayout::default(),
            CollectSink::default(),
            100_000.0,
            800.0,
        );
        for _ in 0..60 {
            eng.frame(16.0);
        }
        let f = eng.sink().last().expect("frame");
        let near = |a: [f32; 4], b: [f32; 4]| a.iter().zip(b).all(|(x, y)| (x - y).abs() < 1e-3);
        // Plan 28 R3:整卡面板退役(参考无)→ 只验 diff 装饰仍在。
        assert!(
            f.rects
                .iter()
                .any(|r| near(r.color, crate::theme::Theme::default().diff_add_bg)),
            "diff 应发新增行底色带"
        );
        assert!(
            f.rects
                .iter()
                .any(|r| near(r.color, crate::theme::Theme::default().diff_del_bg)),
            "diff 应发删除行底色带"
        );
        // Plan 28 遗留-2:5 格条退役(参考只有 ±文本);diff 区应有**文件卡面板**(圆角 6 框)。
        let cells = f
            .rects
            .iter()
            .filter(|r| (r.size[0] - 7.0).abs() < 0.01 && (r.size[1] - 7.0).abs() < 0.01)
            .count();
        assert_eq!(cells, 0, "5 格条应退役: {cells}");
        assert!(
            f.panels.iter().any(|p| p.id & 0xFFFF_FFFF == 0xFFFF_FFFC),
            "diff 区应有文件卡面板"
        );
    }

    /// Plan 22 P3:tool 载荷整体重写(pending→completed)→ 重置重渲,不拼接旧尾(无残留)。
    #[test]
    fn p3_tool_rewrite_resets_not_appends() {
        let part_updated = |status: &str| -> String {
            format!(
                r#"{{"type":"message.part.updated","properties":{{"part":{{"type":"tool","id":"t1","messageID":"m1","tool":"bash","state":{{"status":"{status}"}}}},"time":1.0}}}}"#
            )
        };
        let mut eng = Engine::new(
            Player::from_pairs(
                vec![
                    (0.0, part_updated("pending")),
                    (5.0, part_updated("completed")),
                ],
                16.0,
            ),
            MonospaceLayout::default(),
            CollectSink::default(),
            100_000.0,
            800.0,
        );
        for _ in 0..60 {
            eng.frame(16.0);
        }
        let text = eng.sink().visible_text();
        // Plan 28 R3:文本徽章退役 → 以「显示名恰一次」验证全量重写不追加。
        assert!(
            text.matches("Shell").count() == 1,
            "旧状态不应残留(已重置,显示名恰一次): {text}"
        );
    }

    // ───────── Plan 22 P4/P5:错误卡 + 停止冻结(F3/F4/F11 重放) ─────────
    use crate::SessionStatus;

    fn run_events(
        recs: Vec<(f64, String)>,
        frames: usize,
    ) -> Engine<Player, MonospaceLayout, CollectSink> {
        let mut eng = Engine::new(
            Player::from_pairs(recs, 16.0),
            MonospaceLayout::default(),
            CollectSink::default(),
            100_000.0,
            800.0,
        );
        for _ in 0..frames {
            eng.frame(16.0);
        }
        eng
    }

    const ERR_EVENT: &str = r#"{"type":"message.updated","properties":{"info":{"id":"m1","role":"assistant","error":{"name":"APIError"}}}}"#;

    #[test]
    fn p4_error_card_idempotent_one_card() {
        // F4 恒一张:重复错误事件 → 只一张合成错误卡。
        let eng = run_events(vec![(0.0, ERR_EVENT.into()), (1.0, ERR_EVENT.into())], 30);
        assert!(matches!(
            eng.session_status(),
            SessionStatus::Errored { .. }
        ));
        let text = eng.sink().visible_text();
        assert_eq!(text.matches("[error]").count(), 1, "应恒一张错误卡: {text}");
        assert!(text.contains("APIError"), "错误文案可见: {text}");
    }

    #[test]
    fn p4_real_response_clears_error_card() {
        // F4:错误后真回复到达 → 清陈旧合成卡。
        let eng = run_events(vec![(0.0, ERR_EVENT.into()), (1.0, delta("p1", "hi"))], 40);
        let text = eng.sink().visible_text();
        assert!(!text.contains("[error]"), "真回复应清错误卡: {text}");
        assert!(text.contains("hi"), "真回复可见: {text}");
        assert!(matches!(
            eng.session_status(),
            SessionStatus::Streaming { .. }
        ));
    }

    #[test]
    fn p4_ghost_abort_no_card() {
        // F3 ghost-abort:AbortError → 不弹错误卡,回 Idle。
        let abort = r#"{"type":"session.error","properties":{"sessionID":"s","error":{"name":"AbortError"}}}"#;
        let eng = run_events(vec![(0.0, abort.into())], 20);
        assert!(
            !eng.sink().visible_text().contains("[error]"),
            "ghost-abort 不弹卡"
        );
        assert_eq!(eng.session_status(), &SessionStatus::Idle);
    }

    #[test]
    fn p4_stop_freezes_message_events() {
        // F11:停止 → 冻结当前流式消息 → 其后续 part 事件被丢弃。
        let recs = vec![(0.0, delta("p1", "a")), (100.0, delta("p1", "b"))];
        let mut eng = Engine::new(
            Player::from_pairs(recs, 16.0),
            MonospaceLayout::default(),
            CollectSink::default(),
            100_000.0,
            800.0,
        );
        eng.note_send(); // → AwaitingAck,使首个 part 走 FirstPart 捕获 message_id
        for _ in 0..3 {
            eng.frame(16.0);
        }
        assert!(matches!(
            eng.session_status(),
            SessionStatus::Streaming { .. }
        ));
        eng.stop();
        assert_eq!(eng.session_status(), &SessionStatus::Stopped);
        for _ in 0..20 {
            eng.frame(16.0);
        }
        assert_eq!(eng.sink().visible_text(), "a", "冻结消息的后续段应被丢弃");
    }

    #[test]
    fn p0_injected_events_decode_render_and_drive_fsm() {
        // Plan 22 P0:host 注入事件(= TS transport push_event 路径)→ 解码 → 驱动 FSM + 渲染。
        let mut eng = Engine::new(
            Player::from_pairs(vec![], 16.0),
            MonospaceLayout::default(),
            CollectSink::default(),
            100_000.0,
            800.0,
        );
        eng.inject_raw(r#"{"type":"permission.asked","properties":{"sessionID":"s"}}"#);
        eng.frame(16.0);
        assert_eq!(
            eng.session_status_tag(),
            "blocked:permission",
            "权限请求 → 阻塞"
        );
        assert!(eng.reply_permission_with(true), "应答 pending permission");
        assert_ne!(
            eng.session_status_tag(),
            "blocked:permission",
            "应答 → 解阻"
        );
        // 注入工具 part → 兜底渲染可见。
        eng.inject_raw(
            r#"{"type":"message.part.updated","properties":{"part":{"type":"tool","id":"t1","messageID":"m1","tool":"bash","state":{"status":"completed"}},"time":1.0}}"#,
        );
        for _ in 0..40 {
            eng.frame(16.0);
        }
        assert!(
            eng.sink().visible_text().contains("Shell"),
            "注入的工具 part 应渲染(显示名 Shell): {}",
            eng.sink().visible_text()
        );
    }

    // ───────── Plan 27:流内问答卡(PR-A 状态机 / PR-C 命中) ─────────

    const PERM_ASKED: &str = r#"{"type":"permission.asked","properties":{"sessionID":"s","title":"允许写入 src/auth?"}}"#;

    #[test]
    fn p27_permission_lifecycle_inflow() {
        let mut eng = Engine::new(
            Player::from_pairs(vec![], 16.0),
            MonospaceLayout::default(),
            CollectSink::default(),
            100_000.0,
            800.0,
        );
        eng.inject_raw(PERM_ASKED);
        for _ in 0..40 {
            eng.frame(16.0);
        }
        // 流内 pending 卡:标题 + prompt + 两个按钮字都在画布文本里。
        let text = eng.sink().visible_text();
        assert!(text.contains("权限请求"), "pending 卡标题: {text}");
        assert!(text.contains("允许写入 src/auth?"), "prompt 可见: {text}");
        assert!(
            text.contains("允许") && text.contains("拒绝"),
            "按钮字可见: {text}"
        );
        assert_eq!(eng.session_status_tag(), "blocked:permission");
        let ask = eng.pending_ask().expect("有 pending ask");
        assert_eq!(ask.kind, crate::AskKind::Permission);

        // 应答 → 原位落定(身份不变:仍同一 part/块序),FSM 解阻;幂等:再答 false。
        let block_before = eng.pending_ask_block().expect("块已建");
        assert!(eng.reply_permission_with(true));
        assert!(!eng.reply_permission_with(true), "重复应答忽略(幂等)");
        assert_ne!(eng.session_status_tag(), "blocked:permission");
        for _ in 0..40 {
            eng.frame(16.0);
        }
        let text = eng.sink().visible_text();
        assert!(text.contains("已允许"), "落定答案卡: {text}");
        assert!(!text.contains("拒绝"), "选项收起,不留按钮: {text}");
        // 身份不变(0020):落定卡仍是原块(views 数不变,块下标同)。
        assert_eq!(
            eng.visible_messages()
                .iter()
                .filter(|m| m.id as usize == block_before)
                .count(),
            1
        );
    }

    #[test]
    fn p27_permission_hit_targets_and_tap() {
        let mut eng = Engine::new(
            Player::from_pairs(vec![], 16.0),
            MonospaceLayout::default(),
            CollectSink::default(),
            100_000.0,
            800.0,
        );
        eng.inject_raw(PERM_ASKED);
        for _ in 0..40 {
            eng.frame(16.0);
        }
        // 命中盒:恰两个(允许/拒绝),世界坐标;tap 允许中心(世界→屏幕:默认 pan=0/zoom=1)。
        let targets: Vec<(String, crate::Rect)> = eng.ask_targets().to_vec();
        assert_eq!(targets.len(), 2, "两个按钮命中盒");
        assert_eq!(targets[0].0, "allow");
        assert_eq!(targets[1].0, "deny");
        let r = targets[0].1;
        let (cx, cy) = (r.x + r.w / 2.0, r.y + r.h / 2.0);
        assert_eq!(eng.ask_hit_at(cx, cy), Some("allow"), "hover 命中");
        assert!(eng.tap(cx, cy), "tap 命中允许 → 应答");
        assert_eq!(eng.session_status_tag(), "idle"); // Replied:此前未流式(resume 空)→ 回 Idle
                                                      // 落定后下一帧命中盒清空(不再可点)。
        for _ in 0..5 {
            eng.frame(16.0);
        }
        assert!(eng.ask_targets().is_empty(), "落定后无命中盒");
        assert!(!eng.tap(cx, cy), "再 tap 无效(幂等)");
    }

    #[test]
    fn p27_question_flow_and_pad() {
        let mut eng = Engine::new(
            Player::from_pairs(vec![], 16.0),
            MonospaceLayout::default(),
            CollectSink::default(),
            100_000.0,
            800.0,
        );
        eng.inject_raw(
            r#"{"type":"question.asked","properties":{"sessionID":"s","question":"超时阈值要改吗?","options":["保持 1s","提到 2s"]}}"#,
        );
        for _ in 0..40 {
            eng.frame(16.0);
        }
        let text = eng.sink().visible_text();
        assert!(text.contains("超时阈值要改吗?"), "prompt: {text}");
        assert!(
            text.contains("保持 1s") && text.contains("提到 2s"),
            "选项: {text}"
        );
        assert_eq!(eng.session_status_tag(), "blocked:question");
        // DOM 实测高回报 → 补白行(块高增长,一次)。
        let h_before = eng.visible_messages()[0].height;
        eng.set_ask_height(h_before + 100.0);
        for _ in 0..10 {
            eng.frame(16.0);
        }
        let h_after = eng.visible_messages()[0].height;
        assert!(
            h_after > h_before + 30.0,
            "补白后块高增长: {h_before}→{h_after}"
        );
        // 提交(选 1 + 文本)→ 落定答案卡。
        assert!(eng.reply_question_with(vec![0], "维持现状".to_owned()));
        assert_ne!(eng.session_status_tag(), "blocked:question");
        for _ in 0..40 {
            eng.frame(16.0);
        }
        let text = eng.sink().visible_text();
        assert!(text.contains("已回答"), "落定卡: {text}");
        assert!(text.contains("保持 1s"), "所选项 chip: {text}");
        assert!(text.contains("维持现状"), "自由输入原文: {text}");
        assert!(!text.contains("提到 2s"), "未选项收起: {text}");
    }

    #[test]
    fn p5_f8_bottom_out_card_when_no_response() {
        // F8:已发送(AwaitingAck)却收到 idle 收尾而无任何回复 → 注兜底错误卡,不静默卡死。
        let mut eng = Engine::new(
            Player::from_pairs(vec![], 16.0),
            MonospaceLayout::default(),
            CollectSink::default(),
            100_000.0,
            800.0,
        );
        eng.note_send();
        eng.inject_raw(r#"{"type":"session.status","properties":{"status":{"type":"idle"}}}"#);
        for _ in 0..30 {
            eng.frame(16.0);
        }
        assert!(
            eng.sink().visible_text().contains("[error]"),
            "无回复收尾应注兜底错误卡: {}",
            eng.sink().visible_text()
        );
        assert!(matches!(
            eng.session_status(),
            SessionStatus::Errored { .. }
        ));
    }

    #[test]
    fn p5_f9_quota_error_card() {
        // F9:配额/限流类错误 → 错误卡标注额度受限。
        let mut eng = Engine::new(
            Player::from_pairs(vec![], 16.0),
            MonospaceLayout::default(),
            CollectSink::default(),
            100_000.0,
            800.0,
        );
        eng.inject_raw(
            r#"{"type":"session.error","properties":{"sessionID":"s","error":{"name":"RateLimitError"}}}"#,
        );
        for _ in 0..30 {
            eng.frame(16.0);
        }
        let t = eng.sink().visible_text();
        assert!(
            t.contains("额度") || t.contains("配额"),
            "配额错误卡应标注: {t}"
        );
        assert!(matches!(
            eng.session_status(),
            SessionStatus::Errored { .. }
        ));
    }

    #[test]
    fn streams_text_to_visible_glyphs() {
        let records = vec![(0.0, delta("p1", "Hi 你好"))];
        let player = Player::from_pairs(records, 16.0);
        let mut eng = Engine::new(
            player,
            MonospaceLayout::default(),
            CollectSink::default(),
            500.0, // 快一点,几帧就吐完
            800.0,
        );
        for _ in 0..30 {
            eng.frame(16.0);
        }
        assert_eq!(eng.store().part_text("p1"), Some("Hi 你好"));
        // 渲染汇可见文本应等于完整串(顺序无损)。
        assert_eq!(eng.sink().visible_text(), "Hi 你好");
    }

    #[test]
    fn fade_in_spawn_times_increase() {
        let player = Player::from_pairs(vec![(0.0, delta("p", "abcdef"))], 16.0);
        let mut eng = Engine::new(
            player,
            MonospaceLayout::default(),
            CollectSink::default(),
            60.0, // 慢:每帧约吐 1 字,spawn_time 递增
            800.0,
        );
        for _ in 0..60 {
            eng.frame(16.0);
        }
        let frame = eng.sink().last().expect("frame");
        let spawns: Vec<f32> = frame.glyphs.iter().map(|g| g.spawn_time).collect();
        assert_eq!(spawns.len(), 6);
        // 非递减(逐字上屏,后到的 spawn_time >= 先到的)。
        assert!(spawns.windows(2).all(|w| w[1] >= w[0]), "{spawns:?}");
        assert!(spawns[5] > spawns[0], "末字应晚于首字: {spawns:?}");
    }

    #[test]
    fn streaming_emphasis_close_flips_role() {
        // Plan 5C:活动块逐帧重解析(0017 §3);`**bold**` 闭合后该字带 Bold 角色、无字面 `*`。
        let player = Player::from_pairs(vec![(0.0, delta("p", "a **bold** c"))], 16.0);
        let mut eng = Engine::new(
            player,
            MonospaceLayout::default(),
            CollectSink::default(),
            500.0,
            800.0,
        );
        for _ in 0..40 {
            eng.frame(16.0);
        }
        let f = eng.sink().last().expect("frame");
        assert!(
            !f.glyphs.iter().any(|g| g.cluster == "*"),
            "闭合后不应有字面 *"
        );
        let bold = StyleRole::Bold.as_u32();
        assert!(
            f.glyphs.iter().any(|g| g.cluster == "b" && g.style == bold),
            "bold 文本应是 Bold 角色"
        );
    }

    #[test]
    fn node_debug_overlay_emits_kind_boxes() {
        // Plan 7E:debug_geometry 开 → 节点容器框(按 kind 上色,描边)叠加到 rects(肉眼验树)。
        let player = Player::from_pairs(vec![(0.0, delta("p", "# Title\n\nbody"))], 16.0);
        let mut eng = Engine::new(
            player,
            MonospaceLayout::default(),
            CollectSink::default(),
            500.0,
            800.0,
        );
        eng.set_debug_geometry(true);
        for _ in 0..40 {
            eng.frame(16.0);
        }
        let f = eng.sink().last().expect("frame");
        // Heading 节点框(蓝 [0.40,0.65,1.0],描边)应出现。
        let near = |a: [f32; 4], b: [f32; 4]| a.iter().zip(b).all(|(x, y)| (x - y).abs() < 1e-3);
        assert!(
            f.rects
                .iter()
                .any(|r| r.stroke > 0.0 && near(r.color, [0.40, 0.65, 1.0, 0.9])),
            "应有 Heading 节点框"
        );
    }

    #[test]
    fn engine_exposes_block_node_tree() {
        // Plan 7 / 0020:engine 排版后 `block_nodes` 暴露内容节点树(下游查询地基)。
        let player = Player::from_pairs(vec![(0.0, delta("p", "# Title\n\nbody text"))], 16.0);
        let mut eng = Engine::new(
            player,
            MonospaceLayout::default(),
            CollectSink::default(),
            500.0,
            800.0,
        );
        for _ in 0..40 {
            eng.frame(16.0);
        }
        let tree = eng.block_nodes(0).expect("应有节点树");
        assert!(tree.root().is_some(), "应有根");
        assert!(
            tree.nodes_of_kind(crate::nodes::NodeKind::Heading).count() >= 1,
            "应有标题节点"
        );
        assert_eq!(eng.block_nodes(99), None, "越界块返回 None");
    }

    #[test]
    fn streaming_setext_upgrades_to_heading() {
        // Plan 5C:setext —— 下一行 `===` 到达 → 上一行回溯升级为标题(lookahead 重解析)。
        let player = Player::from_pairs(vec![(0.0, delta("p", "Title\n==="))], 16.0);
        let mut eng = Engine::new(
            player,
            MonospaceLayout::default(),
            CollectSink::default(),
            500.0,
            800.0,
        );
        for _ in 0..40 {
            eng.frame(16.0);
        }
        let f = eng.sink().last().expect("frame");
        let h1 = StyleRole::Heading.as_u32(); // setext `===` = H1
        assert!(
            f.glyphs.iter().any(|g| g.cluster == "T" && g.style == h1),
            "setext 下划线到达后标题行应升级为 Heading"
        );
        assert!(
            !f.glyphs.iter().any(|g| g.cluster == "="),
            "setext 下划线不应显形"
        );
    }

    #[test]
    fn glyph_identity_is_append_stable() {
        // Plan 5A/0017 §6:append-only → (block_seq, glyph_idx) 跨重排稳定。首字身份不随追加变。
        let player = Player::from_pairs(vec![(0.0, delta("p", "hello world"))], 16.0);
        let mut eng = Engine::new(
            player,
            MonospaceLayout::default(),
            CollectSink::default(),
            40.0, // 慢吐:逐字揭示
            800.0,
        );
        let mut first_seen: Option<(u32, u32)> = None;
        for _ in 0..120 {
            eng.frame(16.0);
            if let Some(f) = eng.sink().last() {
                if let Some(g) = f.glyphs.iter().find(|g| g.cluster == "h") {
                    let id = (g.block_seq, g.glyph_idx);
                    if let Some(prev) = first_seen {
                        assert_eq!(id, prev, "首字身份应跨帧稳定");
                    }
                    first_seen = Some(id);
                    assert_eq!(g.block_seq, 0, "单块 block_seq=0");
                }
            }
        }
        assert!(first_seen.is_some(), "应揭示出首字");
    }

    #[test]
    fn reveal_scheduler_rate_limits_vs_unlimited() {
        // 8C:限速调度器在等量 token 到达下,单位时间揭示的字应**少于**不限速(节奏与 token 解耦)。
        let build = |cps: f32| {
            let mut eng = Engine::new(
                Player::from_pairs(vec![(0.0, delta("p", "abcdefghijklmnopqrstuvwxyz"))], 16.0),
                MonospaceLayout::default(),
                CollectSink::default(),
                2000.0, // smoother 快:内容很快全部到达,瓶颈在调度器
                800.0,
            );
            eng.set_reveal_cps(cps);
            for _ in 0..6 {
                eng.frame(16.0); // ~96ms
            }
            eng.sink().last().map_or(0, |f| f.glyphs.len())
        };
        let limited = build(50.0); // 50 字/秒 → ~96ms 约 5 字(封顶内)
        let unlimited = build(f32::INFINITY);
        assert!(
            limited < unlimited,
            "限速({limited})应少于不限速({unlimited})"
        );
        assert!(limited >= 1, "限速也应揭示出若干字: {limited}");
    }

    #[test]
    fn reveal_is_deterministic_with_injected_time() {
        // 8C:同 dt 序列 → 同揭示(注入时间,可重放 R8/R9)。
        let run = || {
            let mut eng = Engine::new(
                Player::from_pairs(vec![(0.0, delta("p", "hello 世界 stream"))], 16.0),
                MonospaceLayout::default(),
                CollectSink::default(),
                300.0,
                800.0,
            );
            eng.set_reveal_cps(120.0);
            let mut trace = Vec::new();
            for _ in 0..40 {
                eng.frame(16.0);
                if let Some(f) = eng.sink().last() {
                    trace.push(
                        f.glyphs
                            .iter()
                            .map(|g| (g.glyph_idx, g.spawn_time))
                            .collect::<Vec<_>>(),
                    );
                }
            }
            trace
        };
        assert_eq!(run(), run(), "限速调度应逐帧确定性可重放");
    }

    #[test]
    fn anim_groups_alive_while_streaming_zero_when_settled() {
        // Plan 25 M2a:流式(揭示中)→ 活跃动效组 ∈ 1..=4(orb 呼吸 + 光标 + 指示条);
        // 揭示结算后 → 0(idle 页面回全冻结,rubric 7 / 0028 护栏)+ 无 WIDGET_PULSE 残留。
        let mut eng = Engine::new(
            Player::from_pairs(vec![], 16.0),
            MonospaceLayout::default(),
            CollectSink::default(),
            5000.0,
            800.0,
        );
        eng.set_reveal_preset(crate::rhythm::RhythmPreset::Reader);
        eng.inject_raw(&delta("p", "你好,世界。今天讲节奏与生命感。"));
        for _ in 0..8 {
            eng.frame(16.0); // 内容到达 + 揭示中(reader 步频 180ms/字 → 远未结算)
        }
        let mid = eng.frame_stats().anim_groups;
        assert!((1..=4).contains(&mid), "流式中活跃动效组应 ∈ 1..=4: {mid}");
        let f = eng.sink().last().expect("frame");
        assert!(
            f.widgets
                .iter()
                .any(|w| w.component == crate::frame::WIDGET_PULSE),
            "流式中应有呼吸光标/指示条"
        );
        for _ in 0..900 {
            eng.frame(16.0); // ~14s,揭示 + 停顿全走完 → settled
        }
        assert_eq!(
            eng.frame_stats().anim_groups,
            0,
            "settled/idle 应全退场(页面回全冻结)"
        );
        let f = eng.sink().last().expect("frame");
        assert!(
            f.widgets
                .iter()
                .all(|w| w.component != crate::frame::WIDGET_PULSE),
            "settled 帧不得残留呼吸 widget"
        );
        assert!(
            f.shaderboxes.iter().all(|sb| !sb.dynamic),
            "settled 帧 ShaderBox 全静态(orb 冻结)"
        );
    }

    #[test]
    fn stop_freezes_future_but_pre_stop_content_reveals_under_reader() {
        // TC-F13 native 复现(M1 回归):reader 预设 + cps 1e9(e2e bootVisible 同配)下,
        // note_send → delta AAA → 数帧 → stop:AAA 必须已上屏;stop 后同消息 BBB 被冻结丢弃。
        let mut eng = Engine::new(
            Player::from_pairs(vec![], 16.0),
            MonospaceLayout::default(),
            CollectSink::default(),
            200.0,
            800.0,
        );
        eng.set_reveal_preset(crate::rhythm::RhythmPreset::Reader);
        eng.set_reveal_cps(1e9);
        eng.note_send();
        eng.inject_raw(&delta("pz", "AAA"));
        for _ in 0..10 {
            eng.frame(16.0);
        }
        eng.stop();
        eng.frame(16.0);
        assert!(
            eng.sink().visible_text().contains("AAA"),
            "停止前内容应已揭示(reader+cps1e9 = 即时): {:?}",
            eng.sink().visible_text()
        );
        eng.inject_raw(&delta("pz", "BBB"));
        for _ in 0..5 {
            eng.frame(16.0);
        }
        assert!(
            !eng.sink().visible_text().contains("BBB"),
            "冻结消息的后续段应被丢弃"
        );
    }

    #[test]
    fn rhythm_reader_release_times_deterministic_same_seed() {
        // Plan 25 M1 / DoD #5(R8):reader 预设(含 seeded 微定时)下,同 seed 同输入 →
        // 揭示时刻序列(spawn_time)**逐 ms 相等**。种子 = 块下标,无墙钟无全局态。
        let run = || {
            let mut eng = Engine::new(
                Player::from_pairs(
                    vec![(
                        0.0,
                        delta("p", "你好,世界。hello brave world 结束。\n\n新段落"),
                    )],
                    16.0,
                ),
                MonospaceLayout::default(),
                CollectSink::default(),
                5000.0, // 内容即时到达,瓶颈在节奏
                800.0,
            );
            eng.set_reveal_preset(crate::rhythm::RhythmPreset::Reader);
            for _ in 0..600 {
                eng.frame(16.0); // ~9.6s,足够全部单元 + 停顿走完
            }
            let f = eng.sink().last().expect("frame");
            let mut spawns: Vec<(u32, u32, f32)> = f
                .glyphs
                .iter()
                .map(|g| (g.block_seq, g.glyph_idx, g.spawn_time))
                .collect();
            spawns.sort_by_key(|x| (x.0, x.1));
            spawns
        };
        let a = run();
        let b = run();
        assert!(!a.is_empty(), "应有揭示产物");
        assert_eq!(a, b, "reader 预设含微定时仍必须逐 ms 确定(seed=块 key)");
        // 且 spawn 时刻非匀速(节奏层生效):相邻释放间隔存在显著差异(句读呼吸)。
        let mut times: Vec<f32> = a.iter().map(|x| x.2).collect();
        times.sort_by(f32::total_cmp);
        times.dedup();
        let gaps: Vec<f32> = times.windows(2).map(|w| w[1] - w[0]).collect();
        let max_gap = gaps.iter().copied().fold(0.0f32, f32::max);
        let min_gap = gaps
            .iter()
            .copied()
            .filter(|&g| g > 0.5)
            .fold(f32::MAX, f32::min);
        assert!(
            max_gap > min_gap * 2.0,
            "reader 释放间隔应有节奏差(边界停顿):min={min_gap} max={max_gap}"
        );
    }

    #[test]
    fn table_full_style_skeleton_before_cells() {
        // 8D/8C:整表风格(默认 Full)→ cell 字带骨架/表头 delay(spawn 更晚于"块开揭"),
        // 表头字早于 body 字(tier 有序)。注:网格**面板**几何由 JS 像素两趟回传,native
        // `MonospaceLayout` 不产 → 此处只验**揭示时序**(骨架先行的时间端点),面板视觉在 8E 重放验。
        let md = "| A | B |\n|---|---|\n| 1 | 2 |\n| 3 | 4 |";
        let player = Player::from_pairs(vec![(0.0, delta("p", md))], 16.0);
        let mut eng = Engine::new(
            player,
            MonospaceLayout::default(),
            CollectSink::default(),
            2000.0,
            800.0,
        );
        for _ in 0..60 {
            eng.frame(16.0);
        }
        // 默认整表风格等表闭合(9F):推进看门狗收尾 → 整表(网格→表头→cell)揭示。
        for _ in 0..6 {
            eng.frame(8_000.0);
        }
        let f = eng.sink().last().expect("frame");
        // 表头 'A' 与 body '3' 都应揭示;表头早于 body(tier:表头 < body)→ 骨架/网格先、表头次、body 末。
        let spawn_of = |c: &str| {
            f.glyphs
                .iter()
                .find(|g| g.cluster == c)
                .map(|g| g.spawn_time)
        };
        let a = spawn_of("A").expect("表头 'A' 应揭示");
        let three = spawn_of("3").expect("body '3' 应揭示");
        assert!(a > 0.0, "cell 字 spawn 应带骨架延迟(>0): {a}");
        assert!(a < three, "表头 'A'({a}) 应早于 body '3'({three})");
    }

    #[test]
    fn forming_table_emits_no_raw_glyphs_until_confirmed() {
        // 8D:成形中的表格(表头到、分隔行未到)→ 绝不闪 raw `| a | b |`(suppress);
        // 分隔行到齐确认成 Table 后才揭示表头字。
        let player = Player::from_pairs(vec![(0.0, delta("p", "| a | b |"))], 16.0);
        let mut eng = Engine::new(
            player,
            MonospaceLayout::default(),
            CollectSink::default(),
            2000.0,
            800.0,
        );
        for _ in 0..30 {
            eng.frame(16.0);
        }
        let f = eng.sink().last().expect("frame");
        assert!(
            f.glyphs.is_empty(),
            "成形中的表格不应揭示任何 raw 字: {:?}",
            f.glyphs.iter().map(|g| &g.cluster).collect::<Vec<_>>()
        );
        // 分隔行到齐 → 确认成表;默认整表风格(Full)等表闭合(turn 收尾)才揭(9F 内容门)。
        let player2 = Player::from_pairs(vec![(0.0, delta("p2", "| a | b |\n|---|---|"))], 16.0);
        let mut eng2 = Engine::new(
            player2,
            MonospaceLayout::default(),
            CollectSink::default(),
            2000.0,
            800.0,
        );
        for _ in 0..6 {
            eng2.frame(8_000.0); // 推进 ~48s → 看门狗收尾(表闭合)→ 整表揭示
        }
        assert!(
            eng2.sink().visible_text().contains('a'),
            "表闭合(turn 收尾)后整表应揭示表头字"
        );
    }

    #[test]
    fn code_block_bg_reveals_with_chars() {
        // Plan 9 评审:代码块**非**骨架先行(骨架先行=表格专属)。代码底随**已揭码字**画出
        // (装饰接揭示门);码揭示后底在(随字 reveal),未揭字不提前显形。
        let player = Player::from_pairs(vec![(0.0, delta("p", "```\nlet x = 1;\n```"))], 16.0);
        let mut eng = Engine::new(
            player,
            MonospaceLayout::default(),
            CollectSink::default(),
            2000.0,
            800.0,
        );
        for _ in 0..60 {
            eng.frame(16.0);
        }
        let f = eng.sink().last().expect("frame");
        // 码字已揭示。
        assert!(f.glyphs.iter().any(|g| g.cluster == "x"), "应揭示码字 'x'");
        // 代码底 rect 随已揭码字出现(装饰接揭示门:有已释放的 code 字 → 画底)。
        let close = |a: [f32; 4], b: [f32; 4]| a.iter().zip(b).all(|(x, y)| (x - y).abs() < 1e-6);
        assert!(
            f.rects
                .iter()
                .any(|r| close(r.color, crate::theme::Theme::default().code_bg)),
            "代码块应有底色 rect(随已揭字 reveal)"
        );
    }

    #[test]
    fn inline_decorations_gated_by_reveal() {
        // Plan 9 回归(红框):未释放的字的内联装饰(行内码 chip / 删除线 strike)绝不提前显形——
        // 装饰与字同一揭示门。限速逐字揭示,在"前缀已揭、`Z`/`W` 未揭"的中间态断言无 chip/strike。
        let player = Player::from_pairs(vec![(0.0, delta("p", "xxxx `Z` yyy ~~W~~ end"))], 16.0);
        let mut eng = Engine::new(
            player,
            MonospaceLayout::default(),
            CollectSink::default(),
            2000.0,
            800.0,
        );
        eng.set_reveal_cps(80.0); // 慢:逐字揭示 → 有"前缀已揭、Z/W 未揭"的中间帧
        let close = |a: [f32; 4], b: [f32; 4]| a.iter().zip(b).all(|(x, y)| (x - y).abs() < 1e-6);
        let mut saw_mid = false;
        for _ in 0..400 {
            eng.frame(16.0);
            let vt = eng.sink().visible_text();
            let f = eng.sink().last().expect("frame");
            if vt.contains('x') && !vt.contains('Z') {
                // Z(行内码)未揭 → 不应有 chip;此前缀里也无删除线 → 不应有 strike。
                saw_mid = true;
                assert!(
                    !f.rects
                        .iter()
                        .any(|r| close(r.color, crate::theme::Theme::default().code_chip)),
                    "未揭行内码不应提前画 chip"
                );
            }
            if vt.contains('W') {
                break; // 删除线字已揭,后续装饰本应出现
            }
        }
        assert!(saw_mid, "应出现'前缀已揭、Z 未揭'的中间态");
        // 全部揭示后 → chip + strike 都应出现(装饰随字 reveal)。
        for _ in 0..200 {
            eng.frame(16.0);
        }
        let f = eng.sink().last().expect("frame");
        assert!(
            f.rects
                .iter()
                .any(|r| close(r.color, crate::theme::Theme::default().code_chip)),
            "码全揭示后应有 chip"
        );
        assert!(
            f.rects
                .iter()
                .any(|r| close(r.color, crate::theme::Theme::default().strike)),
            "删除线全揭示后应有 strike"
        );
    }

    #[test]
    fn display_math_emits_sdf_glyphs_not_raw_tex() {
        // Plan 12 ②:`$$E=mc^2$$` → RaTeX 数学 SDF 字形(math 角色),raw TeX 字符不出 Code 字形。
        let mut eng = Engine::new(
            Player::from_pairs(vec![], 16.0),
            MonospaceLayout::default(),
            CollectSink::default(),
            500.0,
            800.0,
        );
        let snap = r#"[{"info":{"id":"m1","sessionID":"s","role":"a"},
            "parts":[{"type":"text","id":"p1","messageID":"m1","text":"$$E=mc^2$$"}]}]"#;
        eng.prime_from_snapshot(snap);
        eng.frame(16.0);
        let f = eng.sink().last().expect("frame");
        // 数学字形(角色 ≥ 26 = Math*)应出现,且含 'E'/'m'/'c'。
        let math = StyleRole::MathMain.as_u32();
        let mathvar = StyleRole::MathVar.as_u32();
        let is_math = |s: u32| s >= math; // 26+ 全是数学角色
        assert!(
            f.glyphs.iter().any(|g| is_math(g.style)),
            "应有数学 SDF 字形(角色 26+)"
        );
        for c in ['E', 'm', 'c'] {
            assert!(
                f.glyphs
                    .iter()
                    .any(|g| g.cluster == c.to_string() && is_math(g.style)),
                "数学字形应含 {c}(math 角色)"
            );
        }
        // raw TeX(Code 角色 = 5)不应出现(被 RaTeX 字形取代)。
        let code = StyleRole::Code.as_u32();
        assert!(
            !f.glyphs.iter().any(|g| g.style == code),
            "数学块不应渲染 raw TeX 的 Code 字形"
        );
        // 变量 m/c 用斜体数学字体(MathVar);= 号用直立(MathMain)—— 验字族映射生效。
        assert!(
            f.glyphs
                .iter()
                .any(|g| g.cluster == "m" && g.style == mathvar),
            "变量 m 应是 MathVar(斜体数学体)"
        );
    }

    #[test]
    fn display_fraction_emits_visible_rule_bar() {
        // Plan 12:`$$\frac{1}{2}$$` 的分数线 = FrameRect(MATH_COLOR),应入帧且**够粗可见**(非亚像素)。
        let mut eng = Engine::new(
            Player::from_pairs(vec![], 16.0),
            MonospaceLayout::default(),
            CollectSink::default(),
            500.0,
            800.0,
        );
        eng.set_math_em(32.0);
        let snap = r#"[{"info":{"id":"m1","sessionID":"s","role":"a"},
            "parts":[{"type":"text","id":"p1","messageID":"m1","text":"$$\\frac{1}{2}$$"}]}]"#;
        eng.prime_from_snapshot(snap);
        eng.frame(16.0);
        let f = eng.sink().last().expect("frame");
        let close = |a: [f32; 4], b: [f32; 4]| a.iter().zip(b).all(|(x, y)| (x - y).abs() < 1e-6);
        let bar = f
            .rects
            .iter()
            .find(|r| close(r.color, super::MATH_COLOR))
            .expect("分数线 rect 应入帧");
        assert!(bar.size[0] > 4.0, "分数线应有宽度: {}", bar.size[0]);
        // 可见下限 = em 的 5%(32×1.3×0.05 ≈ 2.08px),高 DPI 不被 AA 抹没。
        assert!(
            bar.size[1] >= 2.0,
            "分数线应够粗可见(≥2px): {}",
            bar.size[1]
        );
    }

    #[test]
    fn inline_math_renders_between_text() {
        // Plan 12 ③:行内 `$E=mc^2$` → RaTeX 数学字形(math 角色),夹在正文之间;`$` 不显形。
        let mut eng = Engine::new(
            Player::from_pairs(vec![], 16.0),
            MonospaceLayout::default(),
            CollectSink::default(),
            500.0,
            800.0,
        );
        let snap = r#"[{"info":{"id":"m1","sessionID":"s","role":"a"},
            "parts":[{"type":"text","id":"p1","messageID":"m1","text":"before $E=mc^2$ after"}]}]"#;
        eng.prime_from_snapshot(snap);
        eng.frame(16.0);
        let f = eng.sink().last().expect("frame");
        // 正文 before/after 在;行内公式的 'E'(MathMain)/'m'(MathVar)是数学字形。
        assert!(
            f.glyphs.iter().any(|g| g.cluster == "b"),
            "正文 before 应在"
        );
        let mathvar = StyleRole::MathVar.as_u32();
        let is_math = |s: u32| s >= StyleRole::MathMain.as_u32() && s <= StyleRole::MathTt.as_u32();
        assert!(
            f.glyphs
                .iter()
                .any(|g| g.cluster == "E" && is_math(g.style)),
            "行内公式 E 应是数学字形"
        );
        assert!(
            f.glyphs
                .iter()
                .any(|g| g.cluster == "m" && g.style == mathvar),
            "行内公式变量 m 应是 MathVar(斜体)"
        );
        // `$` 定界符不显形(被 RaTeX 字形取代)。
        assert!(!f.glyphs.iter().any(|g| g.cluster == "$"), "$ 不应显形");
    }

    #[test]
    fn group_turns_user_opens_assistant_groups_into_one_box() {
        // Plan 13 §4.3:user part 开新回合;连续 assistant part(跨 message)归同一回合(一个 AsstBox)。
        use crate::store::Role;
        let v = |role: Role| PartView {
            exiting: None,
            part_id: String::new(),
            pushed: 0,
            pushed_bytes: 0,
            revealed: Vec::new(),
            cache: None,
            spawn: Vec::new(),
            instant: false,
            settled: false,
            role,
            tier: Tier::Hot,
            agg: None,
            glyph_ids: Vec::new(),
            id_clusters: Vec::new(),
            next_glyph_id: 0,
            math_ids: Vec::new(),
            math_id_clusters: Vec::new(),
            next_math_id: 0,
            badge_morph: None,
            last_card_status: None,
        };
        // u, a, a(同回合), u, a → 2 回合;回合1 assistant 两 part 一个 AsstBox。
        let views = vec![
            v(Role::User),
            v(Role::Assistant),
            v(Role::Assistant),
            v(Role::User),
            v(Role::Assistant),
        ];
        let turns = group_turns(&views);
        assert_eq!(turns.len(), 2, "两个回合");
        assert_eq!(turns[0].user, Some(0));
        assert_eq!(
            turns[0].assistant,
            vec![1, 2],
            "连续 assistant → 一个 AsstBox"
        );
        assert_eq!(turns[1].user, Some(3));
        assert_eq!(turns[1].assistant, vec![4]);
        // 开头 assistant(无 user 锚)自成回合。
        let lead = group_turns(&[v(Role::Assistant), v(Role::User), v(Role::Assistant)]);
        assert_eq!(lead.len(), 2);
        assert_eq!(lead[0].user, None);
        assert_eq!(lead[0].assistant, vec![0]);
    }

    #[test]
    fn role_from_snapshot_user_vs_assistant() {
        // Plan 13 ①:snapshot 的 info.role → part_role;user/assistant 各对。
        let mut eng = Engine::new(
            Player::from_pairs(vec![], 16.0),
            MonospaceLayout::default(),
            CollectSink::default(),
            200.0,
            800.0,
        );
        let snap = r#"[
            {"info":{"id":"m1","sessionID":"s","role":"user"},"parts":[{"type":"text","id":"pu","messageID":"m1","text":"hi"}]},
            {"info":{"id":"m2","sessionID":"s","role":"assistant"},"parts":[{"type":"text","id":"pa","messageID":"m2","text":"hello"}]}
        ]"#;
        eng.prime_from_snapshot(snap);
        assert_eq!(eng.store().part_role("pu"), crate::store::Role::User);
        assert_eq!(eng.store().part_role("pa"), crate::store::Role::Assistant);
        assert_eq!(
            eng.store().part_role("unknown"),
            crate::store::Role::Assistant,
            "未知默认 Assistant"
        );
    }

    #[test]
    fn snapshot_primes_instantly_without_fade() {
        // Phase F:快照历史一帧即整段上屏,spawn_time 在远古(零淡入,AR6)。
        let mut eng = Engine::new(
            Player::from_pairs(vec![], 16.0),
            MonospaceLayout::default(),
            CollectSink::default(),
            200.0,
            800.0,
        );
        let snap = r#"[{"info":{"id":"m1","sessionID":"sX","role":"assistant"},
            "parts":[{"type":"text","id":"p1","messageID":"m1","text":"历史回复"}]}]"#;
        eng.prime_from_snapshot(snap);
        eng.frame(16.0); // 一帧
        assert_eq!(eng.sink().visible_text(), "历史回复");
        let frame = eng.sink().last().expect("frame");
        assert!(
            frame.glyphs.iter().all(|g| g.spawn_time < 0.0),
            "catch-up 字形 spawn_time 应在远古"
        );
    }

    #[test]
    fn session_filter_excludes_other_session() {
        // Phase F:目标 sX → 只渲染 sX 的 part,sY 被排除。
        let mut eng = Engine::new(
            Player::from_pairs(vec![], 16.0),
            MonospaceLayout::default(),
            CollectSink::default(),
            200.0,
            800.0,
        );
        eng.set_target_session(Some("sX".into()));
        let snap = r#"[
            {"info":{"id":"m1","sessionID":"sX","role":"assistant"},
             "parts":[{"type":"text","id":"p1","messageID":"m1","text":"AAA"}]},
            {"info":{"id":"m2","sessionID":"sY","role":"assistant"},
             "parts":[{"type":"text","id":"p2","messageID":"m2","text":"BBB"}]}
        ]"#;
        eng.prime_from_snapshot(snap);
        eng.frame(16.0);
        assert_eq!(eng.sink().visible_text(), "AAA");
    }

    // 计数排版器:数 layout 被调用多少次,验证块冻结(Phase G)。
    struct CountingLayout {
        inner: MonospaceLayout,
        calls: std::rc::Rc<std::cell::Cell<usize>>,
    }
    impl crate::LayoutEngine for CountingLayout {
        fn layout(
            &mut self,
            spans: &[crate::StyledSpan],
            tables: &[crate::TableRegion],
            w: f32,
        ) -> crate::LayoutResult {
            self.calls.set(self.calls.get() + 1);
            self.inner.layout(spans, tables, w)
        }
    }

    #[test]
    fn block_freeze_skips_settled_relayout() {
        // 流式期间每帧重排尾部;吐完(settled)后再多帧不应再调 layout。
        let calls = std::rc::Rc::new(std::cell::Cell::new(0usize));
        let layout = CountingLayout {
            inner: MonospaceLayout::default(),
            calls: calls.clone(),
        };
        let mut eng = Engine::new(
            Player::from_pairs(vec![(0.0, delta("p", "abcdefghij"))], 16.0),
            layout,
            CollectSink::default(),
            500.0,
            800.0,
        );
        for _ in 0..40 {
            eng.frame(16.0); // 吐完
        }
        assert_eq!(eng.sink().visible_text(), "abcdefghij");
        let settled = calls.get();
        for _ in 0..10 {
            eng.frame(16.0); // 无新增 → 不应再排版
        }
        assert_eq!(calls.get(), settled, "已冻结块被重排了");
    }

    #[test]
    fn viewport_culls_offscreen_blocks() {
        // 多块 + 小视口 + 锚底 → 顶部块裁掉,底部块可见(Phase G)。
        let mut eng = Engine::new(
            Player::from_pairs(vec![], 16.0),
            MonospaceLayout::default(),
            CollectSink::default(),
            200.0,
            800.0,
        );
        eng.set_viewport_height(25.0); // 约一行
        let snap = r#"[
            {"info":{"id":"m1","sessionID":"s","role":"a"},"parts":[{"type":"text","id":"p1","messageID":"m1","text":"AAAA"}]},
            {"info":{"id":"m2","sessionID":"s","role":"a"},"parts":[{"type":"text","id":"p2","messageID":"m2","text":"BBBB"}]},
            {"info":{"id":"m3","sessionID":"s","role":"a"},"parts":[{"type":"text","id":"p3","messageID":"m3","text":"CCCC"}]},
            {"info":{"id":"m4","sessionID":"s","role":"a"},"parts":[{"type":"text","id":"p4","messageID":"m4","text":"DDDD"}]},
            {"info":{"id":"m5","sessionID":"s","role":"a"},"parts":[{"type":"text","id":"p5","messageID":"m5","text":"EEEE"}]}
        ]"#;
        eng.prime_from_snapshot(snap);
        eng.frame(16.0);
        let visible = eng.sink().visible_text();
        assert!(visible.contains("EEEE"), "底部块应可见: {visible}");
        assert!(!visible.contains("AAAA"), "顶部块应被裁剪: {visible}");
    }

    #[test]
    fn turn_settles_via_watchdog_even_without_idle() {
        // Phase I:delta 到达 → Active;之后久无事件(模型忘了 idle)→ 看门狗强制收尾。
        let player = Player::from_pairs(vec![(0.0, delta("p", "hi"))], 16.0);
        let mut eng = Engine::new(
            player,
            MonospaceLayout::default(),
            CollectSink::default(),
            500.0,
            800.0,
        );
        eng.frame(16.0);
        assert_eq!(eng.turn_status(), crate::TurnStatus::Active);
        for _ in 0..6 {
            eng.frame(8_000.0); // 推进 ~48s,无新事件
        }
        assert_eq!(eng.turn_status(), crate::TurnStatus::Settled);
    }

    #[test]
    fn resync_adds_missed_history_without_disturbing_live() {
        // Phase J:p1 正在 live;resync 补入错过的历史 p0,但不重置 live 的 p1。
        let player = Player::from_pairs(vec![(0.0, delta("p1", "live"))], 16.0);
        let mut eng = Engine::new(
            player,
            MonospaceLayout::default(),
            CollectSink::default(),
            500.0,
            800.0,
        );
        for _ in 0..10 {
            eng.frame(16.0);
        }
        assert_eq!(eng.store().part_text("p1"), Some("live"));
        // resync:含错过的 p0 + 已知的 p1(快照里 p1 更长,但应被跳过不动 live)。
        let snap = r#"[
            {"info":{"id":"m0","sessionID":"s","role":"a"},"parts":[{"type":"text","id":"p0","messageID":"m0","text":"missed"}]},
            {"info":{"id":"m1","sessionID":"s","role":"a"},"parts":[{"type":"text","id":"p1","messageID":"m1","text":"liveXXXX"}]}
        ]"#;
        eng.resync_from_snapshot(snap);
        eng.frame(16.0);
        assert_eq!(
            eng.store().part_text("p0"),
            Some("missed"),
            "应补入错过历史"
        );
        assert_eq!(
            eng.store().part_text("p1"),
            Some("live"),
            "live 块不应被 resync 覆盖"
        );
    }

    #[test]
    fn camera_pan_up_shows_earlier_blocks() {
        // Plan 3 L:小视口锚底先看底部块;向上滚(相机平移)→ 看到顶部块,底部裁掉。
        let mut eng = Engine::new(
            Player::from_pairs(vec![], 16.0),
            MonospaceLayout::default(),
            CollectSink::default(),
            200.0,
            800.0,
        );
        eng.set_viewport_height(25.0);
        let snap = r#"[
            {"info":{"id":"m1","sessionID":"s","role":"a"},"parts":[{"type":"text","id":"p1","messageID":"m1","text":"AAAA"}]},
            {"info":{"id":"m2","sessionID":"s","role":"a"},"parts":[{"type":"text","id":"p2","messageID":"m2","text":"BBBB"}]},
            {"info":{"id":"m3","sessionID":"s","role":"a"},"parts":[{"type":"text","id":"p3","messageID":"m3","text":"CCCC"}]},
            {"info":{"id":"m4","sessionID":"s","role":"a"},"parts":[{"type":"text","id":"p4","messageID":"m4","text":"DDDD"}]},
            {"info":{"id":"m5","sessionID":"s","role":"a"},"parts":[{"type":"text","id":"p5","messageID":"m5","text":"EEEE"}]}
        ]"#;
        eng.prime_from_snapshot(snap);
        eng.frame(16.0);
        assert!(
            eng.sink().visible_text().contains("EEEE"),
            "初始锚底应见底部"
        );
        eng.scroll_by(-1000.0); // 滚到顶
        eng.frame(16.0);
        eng.frame(16.0); // Plan 19 P2:跳滚到已释放(Warm)的历史块 → promote 后下帧 ensure_layouts 重建
        let v = eng.sink().visible_text();
        assert!(v.contains("AAAA"), "向上滚应见顶部: {v}");
        assert!(!v.contains("EEEE"), "底部应被裁掉: {v}");
        assert!((eng.camera().zoom() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn code_block_emits_background_rect() {
        // Plan 4B:代码块由角色派生底色 rect。
        let mut eng = Engine::new(
            Player::from_pairs(vec![], 16.0),
            MonospaceLayout::default(),
            CollectSink::default(),
            200.0,
            800.0,
        );
        let snap = r#"[{"info":{"id":"m1","sessionID":"s","role":"a"},
            "parts":[{"type":"text","id":"p1","messageID":"m1","text":"```\nlet x = 1;\n```"}]}]"#;
        eng.prime_from_snapshot(snap);
        eng.frame(16.0);
        let f = eng.sink().last().expect("frame");
        assert!(!f.rects.is_empty(), "代码块应有底色 rect");
        assert!(
            f.rects.iter().any(|r| r.stroke < 0.01),
            "应有填充底色(stroke=0)"
        );
        assert!(
            f.rects.iter().any(|r| r.stroke > 0.5),
            "应有外框描边(Plan 15⑥ box 框)"
        );
    }

    #[test]
    fn inline_code_emits_chip_rect() {
        // Plan 4B1:行内码 `x` 派生一块 chip 底(填充,非整宽)。
        let mut eng = Engine::new(
            Player::from_pairs(vec![], 16.0),
            MonospaceLayout::default(),
            CollectSink::default(),
            200.0,
            800.0,
        );
        let snap = r#"[{"info":{"id":"m1","sessionID":"s","role":"a"},
            "parts":[{"type":"text","id":"p1","messageID":"m1","text":"run `cargo test` now"}]}]"#;
        eng.prime_from_snapshot(snap);
        eng.frame(16.0);
        let f = eng.sink().last().expect("frame");
        assert!(!f.rects.is_empty(), "行内码应有 chip");
        assert!(
            f.rects.iter().all(|r| r.size[0] < 800.0),
            "chip 不应占整块宽"
        );
    }

    #[test]
    fn github_alert_emits_tinted_bar_and_bg() {
        // Plan 4B1:`> [!WARNING]` → 类型色左条(实心)+ 整块淡底。
        let mut eng = Engine::new(
            Player::from_pairs(vec![], 16.0),
            MonospaceLayout::default(),
            CollectSink::default(),
            200.0,
            800.0,
        );
        let snap = r#"[{"info":{"id":"m1","sessionID":"s","role":"a"},
            "parts":[{"type":"text","id":"p1","messageID":"m1","text":"> [!WARNING]\n> be careful"}]}]"#;
        eng.prime_from_snapshot(snap);
        eng.frame(16.0);
        let f = eng.sink().last().expect("frame");
        let warn = crate::theme::Theme::default().alert_bar("WARNING");
        let close = |a: [f32; 4], b: [f32; 4]| a.iter().zip(b).all(|(x, y)| (x - y).abs() < 1e-6);
        assert!(
            f.rects.iter().any(|r| close(r.color, warn)),
            "应有 WARNING 类型色左条"
        );
        // 淡底:整宽、低 alpha。
        assert!(
            f.rects
                .iter()
                .any(|r| r.size[0] > 700.0 && r.color[3] < 0.2 && r.color[3] > 0.0),
            "应有整块淡底"
        );
    }

    #[test]
    fn thematic_break_emits_full_width_rule() {
        let mut eng = Engine::new(
            Player::from_pairs(vec![], 16.0),
            MonospaceLayout::default(),
            CollectSink::default(),
            200.0,
            800.0,
        );
        let snap = r#"[{"info":{"id":"m1","sessionID":"s","role":"a"},
            "parts":[{"type":"text","id":"p1","messageID":"m1","text":"above\n\n---\n\nbelow"}]}]"#;
        eng.prime_from_snapshot(snap);
        eng.frame(16.0);
        let f = eng.sink().last().expect("frame");
        // Plan 28 R2:分隔线 = 整宽素细线 widget(WIDGET_RULE,bright 0.95;参考 opencode hr)。
        assert!(
            f.widgets
                .iter()
                .any(|w| w.component == crate::frame::WIDGET_RULE && w.size[0] > 700.0),
            "分隔线应是整宽素线 rule widget"
        );
    }

    #[test]
    fn debug_geometry_adds_stroked_rects() {
        // Plan 4C3:开调试几何 → 块 AABB / 视口框(描边)。
        let mut eng = Engine::new(
            Player::from_pairs(vec![(0.0, delta("p", "hi"))], 16.0),
            MonospaceLayout::default(),
            CollectSink::default(),
            500.0,
            800.0,
        );
        eng.set_debug_geometry(true);
        for _ in 0..5 {
            eng.frame(16.0);
        }
        let f = eng.sink().last().expect("frame");
        assert!(
            f.rects.iter().any(|r| r.stroke > 0.0),
            "调试几何应有描边 rect"
        );
    }

    #[test]
    fn anchor_bottom_sticks_to_computed_box_bottom() {
        // Plan 13③ 锚底回归:内容高于视口时,相机贴底 → **末行字底恰落在视口下沿**。这证明锚底读的是
        // Taffy 末盒 computed bottom(box 高 = cache.height,= revealed_height 源),收编后语义不变。
        let body: String = (0..100)
            .map(|i| format!("assistant reply line number {i}"))
            .collect::<Vec<_>>()
            .join("\\n"); // JSON 内换行(写成 \n 转义),保证内容远高于 600 视口
        let snap = format!(
            r#"[{{"info":{{"id":"m1","sessionID":"s","role":"a"}},
            "parts":[{{"type":"text","id":"p1","messageID":"m1","text":"{body}"}}]}}]"#
        );
        let mut eng = Engine::new(
            Player::from_pairs(vec![], 16.0),
            MonospaceLayout::default(),
            CollectSink::default(),
            200.0,
            800.0,
        );
        eng.prime_from_snapshot(&snap);
        for _ in 0..200 {
            eng.frame(16.0); // 让贴底平滑收敛
        }
        let f = eng.sink().last().expect("frame");
        let max_g_bottom = f
            .glyphs
            .iter()
            .map(|g| g.pos[1] + g.size[1])
            .fold(f32::MIN, f32::max);
        let vis = eng.camera().visible_world_rect();
        let viewport_bottom = vis.y + vis.h;
        // 内容必须确实高于视口(否则贴底无意义)。
        assert!(vis.y > 1.0, "内容应高于视口、相机已下滚: pan.y={}", vis.y);
        assert!(
            (viewport_bottom - max_g_bottom).abs() < 30.0,
            "末行字底应锚在视口下沿(= 末盒 computed bottom): 视口底 {viewport_bottom} vs 字底 {max_g_bottom}"
        );
    }

    #[test]
    fn image_embed_ready_emits_frameimage_and_hides_alt() {
        // Plan 14 ③:`![cat](url)` 未就绪显 alt;领取解码 → Ready → 出纹理 quad、隐藏 alt 占位字。
        let snap = r#"[{"info":{"id":"m1","sessionID":"s","role":"a"},
            "parts":[{"type":"text","id":"p1","messageID":"m1","text":"![cat](http://x/c.png)"}]}]"#;
        let mut eng = Engine::new(
            Player::from_pairs(vec![], 16.0),
            MonospaceLayout::default(),
            CollectSink::default(),
            200.0,
            800.0,
        );
        eng.prime_from_snapshot(snap);
        eng.frame(16.0);
        let f = eng.sink().last().expect("frame");
        assert!(f.images.is_empty(), "未就绪应无纹理 quad");
        assert!(
            f.glyphs.iter().any(|g| g.cluster == "c"),
            "未就绪显 alt 文本"
        );
        // 领取待解码 → Ready → 再帧。
        let pending = eng.take_pending_images();
        assert_eq!(pending.len(), 1, "一张待解码图");
        assert_eq!(pending[0].1, "http://x/c.png", "url 透传");
        eng.image_ready(pending[0].0, 9, 320.0, 200.0, false);
        eng.frame(16.0);
        let f = eng.sink().last().expect("frame");
        // Plan 16 ④:淡入窗内由 `ShaderId::Channel` 溶解接管(channel0=纹理 id),暂不出静态 quad。
        assert!(f.images.is_empty(), "淡入窗内走溶解 ShaderBox,无静态 quad");
        let diss = f
            .shaderboxes
            .iter()
            .find(|sb| sb.shader_id == crate::ShaderId::Channel.as_u32())
            .expect("淡入应出溶解 ShaderBox");
        assert_eq!(diss.channel0, 9, "channel0 = 图纹理 id");
        assert!(
            !f.glyphs.iter().any(|g| g.cluster == "c"),
            "就绪应隐藏 alt 占位字"
        );
        // 淡入完成(> IMAGE_FADE_MS)→ 切回静态纹理 quad,溶解 ShaderBox 退场。
        for _ in 0..16 {
            eng.frame(16.0);
        }
        let f = eng.sink().last().expect("frame");
        assert_eq!(f.images.len(), 1, "淡入完成出纹理 quad");
        assert_eq!(f.images[0].tex_id, 9);
        assert!(
            (f.images[0].size[0] - 320.0).abs() < 0.5,
            "尺寸 = 解码自然宽"
        );
        assert!(
            f.shaderboxes
                .iter()
                .all(|sb| sb.shader_id != crate::ShaderId::Channel.as_u32()),
            "淡入完成不再溶解"
        );
        // 领取后不再重复待解码(已转 Loading)。
        assert!(eng.take_pending_images().is_empty(), "不重复领取");
    }

    #[test]
    fn animated_image_emits_frameembed_not_frameimage() {
        // Plan 14 ⑤:动图(animated=true)就绪 → 走 FrameEmbed(DOM overlay 自播),不出 canvas 纹理 quad。
        let snap = r#"[{"info":{"id":"m1","sessionID":"s","role":"a"},
            "parts":[{"type":"text","id":"p1","messageID":"m1","text":"![gif](http://x/a.gif)"}]}]"#;
        let mut eng = Engine::new(
            Player::from_pairs(vec![], 16.0),
            MonospaceLayout::default(),
            CollectSink::default(),
            200.0,
            800.0,
        );
        eng.prime_from_snapshot(snap);
        eng.frame(16.0);
        let key = eng.take_pending_images()[0].0;
        eng.image_ready(key, 5, 100.0, 80.0, true); // animated = true
        eng.frame(16.0);
        let f = eng.sink().last().expect("frame");
        assert!(f.images.is_empty(), "动图不进 canvas 纹理 quad");
        assert_eq!(f.embeds.len(), 1, "动图出 FrameEmbed(DOM overlay)");
        assert_eq!(f.embeds[0].key, key);
    }

    fn code_block_glyph_rows(f: &super::FrameData) -> Vec<i64> {
        let code = StyleRole::CodeBlock.as_u32();
        let mut rows: Vec<i64> = f
            .glyphs
            .iter()
            .filter(|g| g.style == code)
            .map(|g| (g.pos[1] * 4.0).round() as i64)
            .collect();
        rows.sort_unstable();
        rows.dedup();
        rows
    }

    fn prime_code(eng: &mut Engine<Player, MonospaceLayout, CollectSink>, n: usize) {
        let body = (0..n)
            .map(|i| format!("codeline{i}"))
            .collect::<Vec<_>>()
            .join("\n");
        let md = format!("```\n{body}\n```");
        let snap = format!(
            r#"[{{"info":{{"id":"m1","sessionID":"s","role":"a"}},
            "parts":[{{"type":"text","id":"p1","messageID":"m1","text":{md:?}}}]}}]"#
        );
        eng.prime_from_snapshot(&snap);
        for _ in 0..6 {
            eng.frame(16.0);
        }
    }

    #[test]
    fn code_block_over_max_lines_windows_to_six_rows_with_edge_fade() {
        // Plan 15 ①:10 行代码块 → 行窗只露 ≤6 行(窗外 cull),且边缘有淡入淡出(alpha<1)。
        let mut eng = Engine::new(
            Player::from_pairs(vec![], 16.0),
            MonospaceLayout::default(),
            CollectSink::default(),
            200.0,
            800.0,
        );
        prime_code(&mut eng, 10);
        let f = eng.sink().last().expect("frame");
        let rows = code_block_glyph_rows(f);
        assert!(!rows.is_empty(), "应有代码字");
        assert!(rows.len() <= 6, "行窗 ≤6 行(超出 cull),实 {}", rows.len());
        let code = StyleRole::CodeBlock.as_u32();
        let faded = f.glyphs.iter().any(|g| g.style == code && g.alpha < 0.99);
        assert!(faded, "超窗代码块边缘应淡入淡出(某字 alpha<1)");
    }

    #[test]
    fn code_block_horizontal_clip_reveals_on_scroll() {
        // Plan 15 ⑤:超宽代码行 → 盒右外的字横裁不发;右滚后露出。max_width 大避免折行(代码不折)。
        let mut eng = Engine::new(
            Player::from_pairs(vec![], 16.0),
            MonospaceLayout::default(),
            CollectSink::default(),
            200.0,
            2000.0,
        );
        let line = format!("{}Z", "a".repeat(80)); // 81 字,'Z' 在 ~x800(盒宽 760 之外)
        let md = format!("```\n{line}\n```");
        let snap = format!(
            r#"[{{"info":{{"id":"m1","sessionID":"s","role":"a"}},
            "parts":[{{"type":"text","id":"p1","messageID":"m1","text":{md:?}}}]}}]"#
        );
        eng.prime_from_snapshot(&snap);
        for _ in 0..6 {
            eng.frame(16.0);
        }
        let code = StyleRole::CodeBlock.as_u32();
        let has_z = |eng: &Engine<Player, MonospaceLayout, CollectSink>| {
            eng.sink()
                .last()
                .expect("frame")
                .glyphs
                .iter()
                .any(|g| g.style == code && g.cluster == "Z")
        };
        assert!(!has_z(&eng), "'Z' 在盒右外,横裁不发");
        let g = {
            let f = eng.sink().last().expect("frame");
            *f.glyphs
                .iter()
                .find(|g| g.style == code)
                .map(|g| &g.pos)
                .expect("应有代码字")
        };
        let key = eng.code_block_at(g[0], g[1]).expect("命中代码块");
        eng.scroll_code_block(key, 120.0, 0); // 右滚 120px
        eng.frame(16.0);
        assert!(has_z(&eng), "右滚后 'Z' 进代码区视口");
    }

    #[test]
    fn code_block_hit_and_manual_scroll_state() {
        // Plan 15 ④:指针命中代码块行窗 → code_block_at 返回 key;scroll_code_block 脱离 tail 记态。
        let mut eng = Engine::new(
            Player::from_pairs(vec![], 16.0),
            MonospaceLayout::default(),
            CollectSink::default(),
            200.0,
            800.0,
        );
        prime_code(&mut eng, 10);
        let f = eng.sink().last().expect("frame");
        let code = StyleRole::CodeBlock.as_u32();
        let g = f
            .glyphs
            .iter()
            .find(|g| g.style == code)
            .expect("应有代码字");
        let key = eng
            .code_block_at(g.pos[0], g.pos[1])
            .expect("代码字所在点应命中代码块");
        assert!(
            eng.code_block_at(g.pos[0], g.pos[1] + 100_000.0).is_none(),
            "远处不命中"
        );
        // 手动滚 → following=false、记 scrollX/Y。
        eng.scroll_code_block(key, 12.0, 2);
        assert_eq!(eng.code_scroll.get(&key), Some(&(12.0, 2, false)));
        eng.frame(16.0); // 不 panic;clamp 在 build_frame(codeblock 单测已覆盖)
    }

    #[test]
    fn copy_icon_is_shaderbox_pinned_top_right() {
        // Plan 16 §2.7:代码块右上角 copy 图标 = `ShaderId::Icons` 画板(程序化,非纹理),不随 scroll。
        let mut eng = Engine::new(
            Player::from_pairs(vec![], 16.0),
            MonospaceLayout::default(),
            CollectSink::default(),
            200.0,
            800.0,
        );
        prime_code(&mut eng, 8);
        let f = eng.sink().last().expect("frame");
        let icon = f
            .shaderboxes
            .iter()
            .find(|sb| sb.shader_id == crate::ShaderId::Icons.as_u32())
            .expect("应有 copy 图标 ShaderBox");
        assert!(icon.pos[0] > 100.0, "图标应靠右: {}", icon.pos[0]);
        assert!((icon.size[0] - 18.0).abs() < 0.01, "图标 18px");
        assert_eq!(
            icon.params[0] as u32,
            crate::IconId::copy().as_u32(),
            "params[0] = copy icon_id"
        );
        assert!(f.images.is_empty(), "不再用纹理 quad 画 copy 图标");
    }

    #[test]
    fn shaderbox_metrics_count_onscreen_pixels() {
        // Plan 16 ②/§2.4:屏上 copy 图标计入 shaderbox_active + shaderbox_pixels(Σ 18² 面积)。
        let mut eng = Engine::new(
            Player::from_pairs(vec![], 16.0),
            MonospaceLayout::default(),
            CollectSink::default(),
            200.0,
            800.0,
        );
        prime_code(&mut eng, 8);
        // Plan 28 R1:头像 orb 已移除 → 无需左移视口;copy 图标在列右缘内,默认视口即全可见。
        eng.frame(16.0);
        let st = eng.frame_stats();
        let f = eng.sink().last().expect("frame");
        assert!(st.shaderbox_active >= 1, "应有活跃 ShaderBox");
        assert_eq!(st.shaderbox_active, f.shaderboxes.len(), "active = 发射数");
        // 屏内 box 全可见(prime_code 内容短)→ 像素 = Σ 各 box 面积。
        let expected: u64 = f
            .shaderboxes
            .iter()
            .map(|sb| (sb.size[0] * sb.size[1]) as u64)
            .sum();
        assert_eq!(st.shaderbox_pixels, expected, "屏上像素 = Σ box 面积");
    }

    #[test]
    fn no_avatar_orb_on_chat_frames() {
        // Plan 28 R1:参考(opencode)无头像列 → 聊天帧不得再发 GlowOrb 头像
        // (shader 画廊页仍可用该 shader,与聊天路径无关)。
        let mut eng = Engine::new(
            Player::from_pairs(vec![], 16.0),
            MonospaceLayout::default(),
            CollectSink::default(),
            200.0,
            800.0,
        );
        prime_code(&mut eng, 3);
        eng.pan_by(-80.0, 0.0);
        eng.frame(16.0);
        let f = eng.sink().last().expect("frame");
        assert!(
            !f.shaderboxes
                .iter()
                .any(|sb| sb.shader_id == crate::ShaderId::GlowOrb.as_u32()),
            "聊天帧不应有头像 orb"
        );
    }

    #[test]
    fn tool_title_shimmers_while_pending_then_clears() {
        // Plan 28 R4:pending/running 工具标题 glyph 带 shimmer 位(1<<31)+ 计 1 动效组;
        // completed 重写自然摘位、动效组归零(idle 全冻结不破)。
        let tool = |status: &str| {
            format!(
                r#"{{"type":"message.part.updated","properties":{{"part":{{"type":"tool","id":"t1","messageID":"m1","tool":"bash","state":{{"status":{status:?},"input":{{"cmd":"ls"}}}}}},"time":1}}}}"#
            )
        };
        let mut eng = Engine::new(
            Player::from_pairs(
                vec![(0.0, tool("running")), (500.0, tool("completed"))],
                16.0,
            ),
            MonospaceLayout::default(),
            CollectSink::default(),
            100_000.0,
            800.0,
        );
        for _ in 0..10 {
            eng.frame(16.0);
        }
        let f = eng.sink().last().expect("frame");
        assert!(
            f.glyphs.iter().any(|g| g.style & 0x8000_0000 != 0),
            "running 期标题应带 shimmer 位"
        );
        assert!(eng.frame_stats().anim_groups >= 1, "shimmer 计动效组");
        for _ in 0..60 {
            eng.frame(16.0);
        }
        let f = eng.sink().last().expect("frame");
        assert!(
            !f.glyphs.iter().any(|g| g.style & 0x8000_0000 != 0),
            "completed 后 shimmer 位应摘除"
        );
        assert_eq!(eng.frame_stats().anim_groups, 0, "终态动效组归零");
    }

    #[test]
    fn stable_ids_prefix_and_suffix_survive_midblock_insert() {
        // Plan 31 R2:前后缀 diff —— 中段插入只有新段新身份,两端沿用(morph 平滑非重建)。
        let cl = |t: &str| -> Vec<String> { t.chars().map(|c| c.to_string()).collect() };
        let (ids1, n1) = assign_stable_ids(&[], &[], &cl("hello world"), 0);
        assert_eq!(ids1, (0..11).collect::<Vec<u32>>(), "首建 = 顺序");
        let (ids2, n2) = assign_stable_ids(&cl("hello world"), &ids1, &cl("hello brave world"), n1);
        assert_eq!(&ids2[..6], &ids1[..6], "前缀身份沿用");
        assert_eq!(
            &ids2[ids2.len() - 5..],
            &ids1[ids1.len() - 5..],
            "后缀身份沿用"
        );
        assert!(ids2[6..12].iter().all(|&x| x >= n1), "中段新身份");
        // 删除:后缀仍稳。
        let (ids3, _n3) =
            assign_stable_ids(&cl("hello brave world"), &ids2, &cl("hello world"), n2);
        assert_eq!(ids3, ids1, "删除回原文 → 身份还原");
        // append-only 等价旧位置键。
        let (ids4, _) = assign_stable_ids(&cl("abc"), &[0, 1, 2], &cl("abcd"), 3);
        assert_eq!(ids4, vec![0, 1, 2, 3]);
    }

    #[test]
    fn stable_ids_deterministic_under_seeded_insert_storm() {
        // Plan 31 R2(GOAL §6「proptest 精神」,确定性版):seeded LCG 生成 200 次随机中段
        // 插入;每步不变式 —— 未触碰的公共前后缀身份保持;双跑逐字节一致(R8)。
        let run = |seed: u64| -> Vec<u32> {
            let mut state = seed;
            let mut rng = move || {
                state = state
                    .wrapping_mul(6_364_136_223_846_793_005)
                    .wrapping_add(1_442_695_040_888_963_407);
                (state >> 33) as usize
            };
            let mut text: Vec<String> = "base".chars().map(|c| c.to_string()).collect();
            let mut ids: Vec<u32> = (0..text.len() as u32).collect();
            let mut next = text.len() as u32;
            for step in 0..200 {
                let at = rng() % (text.len() + 1);
                let ins: Vec<String> = format!("i{step}").chars().map(|c| c.to_string()).collect();
                let mut grown = text[..at].to_vec();
                grown.extend(ins.clone());
                grown.extend_from_slice(&text[at..]);
                let (nids, nn) = assign_stable_ids(&text, &ids, &grown, next);
                // 不变式:公共前缀(插入点前)身份保持。
                let keep = at.min(text.len());
                assert_eq!(&nids[..keep.min(4)], &ids[..keep.min(4)], "前缀身份不乱");
                text = grown;
                ids = nids;
                next = nn;
            }
            ids
        };
        assert_eq!(run(42), run(42), "同 seed 双跑逐字节一致");
        assert_ne!(run(42), run(43), "不同 seed 序列不同(功能性抽样)");
    }

    #[test]
    fn tool_rewrite_midinsert_keeps_tail_identity() {
        // Plan 31 R2 集成:tool 全量重写在中段插入 → 尾部字形 glyph_idx(morph 键)跨重写稳定。
        let tool = |out: &str| {
            format!(
                r#"{{"type":"message.part.updated","properties":{{"part":{{"type":"tool","id":"t1","messageID":"m1","tool":"bash","state":{{"status":"completed","input":{{"cmd":"x"}},"output":{out:?}}}}},"time":1}}}}"#
            )
        };
        let mut eng = Engine::new(
            Player::from_pairs(
                vec![(0.0, tool("alpha TAIL")), (200.0, tool("alpha MID TAIL"))],
                16.0,
            ),
            MonospaceLayout::default(),
            CollectSink::default(),
            100_000.0,
            800.0,
        );
        for _ in 0..8 {
            eng.frame(16.0);
        }
        let tail_ids = |f: &crate::FrameData| -> Vec<u32> {
            // 取 "TAIL" 四字的 glyph_idx(按可见序)。
            let mut ids: Vec<(f32, u32)> = f
                .glyphs
                .iter()
                .filter(|g| "TAIL".contains(g.cluster.as_str()) && g.cluster != " ")
                .map(|g| (g.pos[0], g.glyph_idx))
                .collect();
            ids.sort_by(|a, b| a.0.total_cmp(&b.0));
            ids.into_iter().map(|(_, i)| i).collect()
        };
        let before = tail_ids(eng.sink().last().expect("f"));
        assert!(!before.is_empty());
        for _ in 0..40 {
            eng.frame(16.0);
        }
        let after = tail_ids(eng.sink().last().expect("f"));
        let keep = before.iter().filter(|i| after.contains(i)).count();
        assert!(
            keep >= before.len().saturating_sub(1),
            "尾部身份应跨中段插入保持: before={before:?} after={after:?}"
        );
    }

    #[test]
    fn static_whitelist_svg_renders_vector_not_texture() {
        // Plan 31 R4(0036 §3 分流):静态白名单 SVG data-uri → 每图元 FrameRect(矢量),
        // 不出 FrameImage(纹理);zoom ×4 后矢量 rect 尺寸精确等比(无栅格)。
        let svg = "data:image/svg+xml,%3Csvg%20viewBox='0%200%20100%2050'%3E%3Crect%20x='10'%20y='10'%20width='40'%20height='20'%20fill='%23ff6347'/%3E%3Ccircle%20cx='80'%20cy='25'%20r='15'%20fill='blue'/%3E%3C/svg%3E";
        let snap = format!(
            r#"[{{"info":{{"id":"m1","sessionID":"s","role":"a"}},"parts":[{{"type":"text","id":"p1","messageID":"m1","text":"图 ![v]({svg}) 尾"}}]}}]"#
        );
        let mut eng = Engine::new(
            Player::from_pairs(vec![], 16.0),
            MonospaceLayout::default(),
            CollectSink::default(),
            200.0,
            800.0,
        );
        eng.prime_from_snapshot(&snap);
        eng.frame(16.0);
        let pending = eng.take_pending_images();
        assert_eq!(pending.len(), 1, "待解码一张:{pending:?}");
        eng.image_ready(pending[0].0, 7, 100.0, 50.0, false);
        for _ in 0..10 {
            eng.frame(16.0);
        }
        let f = eng.sink().last().expect("frame");
        assert!(f.images.is_empty(), "白名单 SVG 不应走纹理 quad");
        let tomato = f
            .rects
            .iter()
            .find(|r| (r.color[0] - 1.0).abs() < 0.01 && (r.color[1] - 0.388).abs() < 0.01)
            .expect("应有 tomato 矢量 rect");
        let w1 = tomato.size[0];
        eng.zoom_by(4.0, 0.0, 0.0);
        eng.frame(16.0);
        let f2 = eng.sink().last().expect("frame");
        let tomato2 = f2
            .rects
            .iter()
            .find(|r| (r.color[0] - 1.0).abs() < 0.01 && (r.color[1] - 0.388).abs() < 0.01)
            .expect("zoom 后仍矢量 rect");
        // world 坐标系尺寸不随 zoom 改(zoom 在相机);矢量语义 = 图元恒精确,无栅格重采样。
        assert!((tomato2.size[0] - w1).abs() < 1e-3);
    }

    #[test]
    fn math_glyph_sequence_keeps_identity_on_superscript_growth() {
        // Plan 31 R2 公式场景(机制级):x,+,1 → x,2,+,1(上标插入)—— 既有字保身份。
        // 集成路径(text part 中段重写)受既有 append-only 对账语义限制 → v1 收窄(GOAL §6
        // 预案),真身场景记遗留;此处锁 math_ids 分配正确性(ensure_layouts 用同函数)。
        let seq = |t: &str| -> Vec<String> { t.chars().map(|c| c.to_string()).collect() };
        let (ids1, n1) = assign_stable_ids(&[], &[], &seq("x+1"), 0);
        let (ids2, _) = assign_stable_ids(&seq("x+1"), &ids1, &seq("x2+1"), n1);
        assert_eq!(ids2[0], ids1[0], "x 保身份");
        assert_eq!(&ids2[2..], &ids1[1..], "+1 保身份(平移)");
        assert!(ids2[1] >= n1, "2 新身份");
    }

    #[test]
    fn rehydrate_bypasses_reveal_scheduler_instant_full() {
        // Plan 30 边界③守卫(AR6):Cold→Hot 重建**不得**进节奏调度器 —— promote 后
        // 一帧内该块字形即全量可见(spawn 全 catch-up),不逐字重播。用 reader 预设
        //(节奏最强档)证明:若重建走了调度器,一帧绝不可能吐完整块。
        let mut eng = Engine::new(
            Player::from_pairs(bench_records(40, 8), 1.0),
            MonospaceLayout::default(),
            CollectSink::default(),
            1.0e9,
            800.0,
        );
        eng.set_reveal_preset(crate::rhythm::RhythmPreset::Reader);
        eng.set_reveal_cps(1.0e9); // 首次揭示即时(只测重建路径)
        eng.set_viewport_height(250.0);
        for _ in 0..80 {
            eng.frame(1.0);
        }
        eng.scroll_to(39);
        for _ in 0..30 {
            eng.frame(1.0);
        }
        assert!(eng.frame_stats().tier_counts[2] > 0, "前置:应有 Cold 块");
        // 换慢节奏(tempo 180ms/拍):若 rehydrate 走调度器,一帧只能出 ~0 字。
        eng.set_reveal_cps(5.0);
        eng.scroll_to(0);
        for _ in 0..3 {
            eng.frame(16.0); // 3 帧 × 16ms:调度器路径最多放出 ~0-1 字
        }
        let f = eng.sink().last().expect("frame");
        let visible_of_block0: usize = f
            .glyphs
            .iter()
            .filter(|g| g.block_seq == 0 && g.cluster != "\n")
            .count();
        assert!(
            visible_of_block0 > 50,
            "重建块应一帧全量瞬显(零调度器介入),实际仅 {visible_of_block0} 字"
        );
    }

    #[test]
    fn skeleton_panel_enters_before_first_code_glyph() {
        // Plan 30 M5(§3 判据):结构块容器早于其首字 spawn —— 围栏确认(remend/闭合)即整框,
        // 字按节奏后到。逐帧找 code_bg rect 首现帧 vs 首个代码字可见帧。
        let md = "intro\n\n```rs\nlet a = 1;\nlet b = 2;\n```\n";
        let rec = format!(
            r#"{{"type":"message.part.delta","properties":{{"messageID":"m1","partID":"p1","field":"text","delta":{md:?}}}}}"#
        );
        let mut eng = Engine::new(
            Player::from_pairs(vec![(0.0, rec)], 16.0),
            MonospaceLayout::default(),
            CollectSink::default(),
            100_000.0,
            800.0,
        );
        eng.set_reveal_cps(30.0); // 慢揭示 → 帧间可分辨先后
        let near = |a: [f32; 4], b: [f32; 4]| a.iter().zip(b).all(|(x, y)| (x - y).abs() < 1e-3);
        let mut panel_frame = None;
        let mut glyph_frame = None;
        for fno in 0..600 {
            eng.frame(16.0);
            let f = eng.sink().last().expect("frame");
            if panel_frame.is_none()
                && f.rects
                    .iter()
                    .any(|r| near(r.color, crate::theme::Theme::default().code_bg))
            {
                panel_frame = Some(fno);
            }
            if glyph_frame.is_none()
                && f.glyphs
                    .iter()
                    .any(|g| g.cluster == "l" && g.style & 0x7FFF_FFFF >= 44)
            {
                glyph_frame = Some(fno);
            }
            if panel_frame.is_some() && glyph_frame.is_some() {
                break;
            }
        }
        let (p, g) = (
            panel_frame.expect("面板应出现"),
            glyph_frame.expect("代码字应出现"),
        );
        assert!(p < g, "骨架面板({p})应早于首个代码字({g})");
    }

    #[test]
    fn tier_cold_releases_then_rehydrates_byte_identical() {
        // Plan 29 V2/V3 + DoD-3/4:滚远 → Cold(丢 revealed/spawn);滚回 → 重建;
        // 可见字形(cluster+pos)与释放前**逐字节一致**(零跳变、确定性)。
        let mut eng = Engine::new(
            Player::from_pairs(bench_records(40, 8), 1.0),
            MonospaceLayout::default(),
            CollectSink::default(),
            1.0e9,
            800.0,
        );
        eng.set_viewport_height(250.0);
        for _ in 0..80 {
            eng.frame(1.0);
        }
        // 看顶部(块 0 可见)→ 记快照。
        eng.scroll_to(0);
        for _ in 0..10 {
            eng.frame(1.0);
        }
        let snap = |eng: &Engine<Player, MonospaceLayout, CollectSink>| -> Vec<(String, i32, i32)> {
            eng.sink()
                .last()
                .expect("frame")
                .glyphs
                .iter()
                .map(|g| {
                    (
                        g.cluster.clone(),
                        (g.pos[0] * 8.0).round() as i32,
                        (g.pos[1] * 8.0).round() as i32,
                    )
                })
                .collect()
        };
        let before = snap(&eng);
        assert!(!before.is_empty());
        // 滚到底(顶部块 >8 屏外)→ 反复 frame 让 reclaim 逐级降档。
        eng.scroll_to(39);
        for _ in 0..30 {
            eng.frame(1.0);
        }
        let st = eng.frame_stats();
        assert!(
            st.tier_counts[2] > 0,
            "远端块应降 Cold: {:?}",
            st.tier_counts
        );
        assert_eq!(st.tier_counts[3], 0, "无 resync 源不得 FrozenFar");
        // 滚回顶部 → promote 带内 rehydrate → 下帧重建。
        eng.scroll_to(0);
        for _ in 0..10 {
            eng.frame(1.0);
        }
        let after = snap(&eng);
        assert_eq!(before, after, "Cold→Hot 重建后可见字形应逐字节一致");
    }

    #[test]
    fn frozen_far_only_with_resync_source() {
        // Plan 29 V2:FrozenFar 需 resync 源;开启后远端块释 Store text 并在滚回时请求重取。
        let mut eng = Engine::new(
            Player::from_pairs(bench_records(80, 8), 1.0),
            MonospaceLayout::default(),
            CollectSink::default(),
            1.0e9,
            800.0,
        );
        eng.set_viewport_height(120.0); // 小视口 → 24 屏带更易越过
        eng.set_resync_available(true);
        for _ in 0..120 {
            eng.frame(1.0);
        }
        eng.scroll_to(79);
        for _ in 0..40 {
            eng.frame(1.0);
        }
        let st = eng.frame_stats();
        assert!(
            st.tier_counts[3] > 0,
            "有 resync 源、极远块应 FrozenFar: {:?}",
            st.tier_counts
        );
        // 滚回 → FrozenFar 块 display_source 空 → 请求 resync(host 轮询边沿)。
        eng.scroll_to(0);
        for _ in 0..10 {
            eng.frame(1.0);
        }
        assert!(eng.wants_resync(), "FrozenFar 重入应请求快照重取");
        assert!(!eng.wants_resync(), "读后清(边沿触发)");
    }

    #[test]
    fn thinking_row_on_send_clears_on_first_part() {
        // Plan 28 遗留-1:note_send → Thinking 合成行(tool:thinking·pending → shimmer 标题);
        // 首个真内容到达 → 行撤(不留历史)。事件驱动,无墙钟(R8)。
        let mut eng = Engine::new(
            Player::from_pairs(vec![], 16.0),
            MonospaceLayout::default(),
            CollectSink::default(),
            100_000.0,
            800.0,
        );
        eng.note_send();
        for _ in 0..10 {
            eng.frame(16.0);
        }
        assert!(
            eng.sink().visible_text().contains("Thinking"),
            "发送后应有 Thinking 行: {}",
            eng.sink().visible_text()
        );
        eng.inject_raw(
            r#"{"type":"message.part.delta","properties":{"messageID":"m1","partID":"p1","field":"text","delta":"hi"}}"#,
        );
        for _ in 0..40 {
            eng.frame(16.0);
        }
        assert!(
            !eng.sink().visible_text().contains("Thinking"),
            "首包后 Thinking 行应撤: {}",
            eng.sink().visible_text()
        );
    }

    #[test]
    fn context_tools_group_into_explored_row() {
        // Plan 28 遗留-1:连续 read/glob/grep → 一行 "Explored N read, M searches"(参考 s5/s7);
        // 组员块塌空不占位。
        let tool = |id: &str, name: &str| {
            format!(
                r#"{{"type":"message.part.updated","properties":{{"part":{{"type":"tool","id":{id:?},"messageID":"m1","tool":{name:?},"state":{{"status":"completed","input":{{}},"output":"x"}}}},"time":1}}}}"#
            )
        };
        let mut eng = Engine::new(
            Player::from_pairs(
                vec![
                    (0.0, tool("t1", "read")),
                    (1.0, tool("t2", "glob")),
                    (2.0, tool("t3", "grep")),
                ],
                16.0,
            ),
            MonospaceLayout::default(),
            CollectSink::default(),
            100_000.0,
            800.0,
        );
        for _ in 0..60 {
            eng.frame(16.0);
        }
        let text = eng.sink().visible_text();
        assert!(text.contains("Explored"), "应有聚合行: {text}");
        assert!(
            text.contains("1 read") && text.contains("2 searches"),
            "计数: {text}"
        );
        assert!(
            !text.contains("Glob") && !text.contains("Grep"),
            "组员应折叠: {text}"
        );
    }

    #[test]
    fn tool_block_has_no_card_panel_or_badge() {
        // Plan 28 R3:参考 tool trigger 无整卡面板、无状态徽章(0018/M4 机制保留但默认不发)。
        let tool = r#"{"type":"message.part.updated","properties":{"part":{"type":"tool","id":"t1","messageID":"m1","tool":"bash","state":{"status":"completed","input":{"cmd":"ls"},"output":"ok"}},"time":1}}"#;
        let mut eng = Engine::new(
            Player::from_pairs(vec![(0.0, tool.to_owned())], 16.0),
            MonospaceLayout::default(),
            CollectSink::default(),
            100_000.0,
            800.0,
        );
        for _ in 0..60 {
            eng.frame(16.0);
        }
        let f = eng.sink().last().expect("frame");
        assert!(
            !f.panels.iter().any(|p| p.id & 0xFFFF_FFFF == 0xFFFF_FFFE),
            "不应发整卡面板"
        );
        assert!(
            !f.widgets
                .iter()
                .any(|w| w.component == crate::frame::WIDGET_BADGE),
            "不应发状态徽章"
        );
    }

    #[test]
    fn shaderbox_culled_when_offscreen() {
        // Plan 16 护栏1:copy 图标随代码块滚出视口 → 不发、不计度量(离屏零耗)。
        let mut eng = Engine::new(
            Player::from_pairs(vec![], 16.0),
            MonospaceLayout::default(),
            CollectSink::default(),
            200.0,
            800.0,
        );
        prime_code(&mut eng, 8);
        assert!(eng.frame_stats().shaderbox_active >= 1, "初始应在屏上");
        eng.pan_by(5000.0, 0.0); // 水平移开整篇(锚底只管 y,x 保留)
        eng.frame(16.0);
        let st = eng.frame_stats();
        assert_eq!(st.shaderbox_active, 0, "离屏应 cull");
        assert_eq!(st.shaderbox_pixels, 0, "离屏不计像素");
        assert!(
            eng.sink().last().expect("frame").shaderboxes.is_empty(),
            "离屏不发 ShaderBox"
        );
    }

    #[test]
    fn shaderbox_gallery_emits_all_builtin_shaders() {
        // Plan 16 调试:开 gallery → 视口出 50 icon + glow_orb + raymarch 各一格(不依赖内容)。
        let mut eng = Engine::new(
            Player::from_pairs(vec![], 16.0),
            MonospaceLayout::default(),
            CollectSink::default(),
            200.0,
            800.0,
        );
        eng.set_shaderbox_gallery(true);
        eng.frame(16.0);
        let f = eng.sink().last().expect("frame");
        // 69 格全发(空会话也出 → 屏锚视口,与内容解耦)。66 icon + orb + raymarch + noise(N1)。
        let tiles = crate::ICON_COUNT as usize + 3;
        assert_eq!(
            f.shaderboxes.len(),
            tiles,
            "66 icon + glow_orb + raymarch + noise_debug"
        );
        let icons = f
            .shaderboxes
            .iter()
            .filter(|sb| sb.shader_id == crate::ShaderId::Icons.as_u32())
            .count();
        assert_eq!(icons, 66, "deck 50 + tool 16");
        // 4 个静态 icon(Void/TheTemple/TheHermit/Enlightenment)time 冻为 0。
        let statics = f
            .shaderboxes
            .iter()
            .filter(|sb| sb.shader_id == crate::ShaderId::Icons.as_u32() && !sb.dynamic)
            .count();
        assert_eq!(statics, 4, "4 个静态 icon 冻");
        assert!(
            f.shaderboxes
                .iter()
                .any(|sb| sb.shader_id == crate::ShaderId::GlowOrb.as_u32()),
            "含 glow_orb 格"
        );
        assert!(
            f.shaderboxes
                .iter()
                .any(|sb| sb.shader_id == crate::ShaderId::Raymarch.as_u32()),
            "含 raymarch 格"
        );
        assert_eq!(
            eng.frame_stats().shaderbox_active,
            tiles,
            "度量计全部 gallery 格"
        );
    }

    #[test]
    fn code_block_within_max_lines_no_window_no_fade() {
        // Plan 15 ①:3 行代码块 → 全显、无 cull、无 fade(所有代码字 alpha=1)。
        let mut eng = Engine::new(
            Player::from_pairs(vec![], 16.0),
            MonospaceLayout::default(),
            CollectSink::default(),
            200.0,
            800.0,
        );
        prime_code(&mut eng, 3);
        let f = eng.sink().last().expect("frame");
        let rows = code_block_glyph_rows(f);
        assert_eq!(rows.len(), 3, "3 行全显");
        let code = StyleRole::CodeBlock.as_u32();
        assert!(
            f.glyphs
                .iter()
                .filter(|g| g.style == code)
                .all(|g| g.alpha > 0.99),
            "不足窗无 fade"
        );
    }

    #[test]
    fn image_embed_failed_keeps_alt_fallback() {
        // Plan 14 ③:解码失败 → Failed → 仍显 alt,无纹理 quad。
        // (URL 须在白名单内 —— S3 起白名单外根本不发起加载,预拒路径见 url_policy_rejects_*。)
        let snap = r#"[{"info":{"id":"m1","sessionID":"s","role":"a"},
            "parts":[{"type":"text","id":"p1","messageID":"m1","text":"![dog](https://x.example/bad.png)"}]}]"#;
        let mut eng = Engine::new(
            Player::from_pairs(vec![], 16.0),
            MonospaceLayout::default(),
            CollectSink::default(),
            200.0,
            800.0,
        );
        eng.prime_from_snapshot(snap);
        eng.frame(16.0);
        let key = eng.take_pending_images()[0].0;
        eng.image_failed(key);
        eng.frame(16.0);
        let f = eng.sink().last().expect("frame");
        assert!(f.images.is_empty(), "失败无纹理 quad");
        assert!(f.glyphs.iter().any(|g| g.cluster == "d"), "失败显 alt 兜底");
    }

    #[test]
    fn reflow_shifts_box_origin_into_glyph_endpoints() {
        // Plan 13⑤(0016 接合):宽度变 → 右对齐 user 盒 origin 变 → glyph **世界位**(补间端点)随之变。
        // 渲染侧 Scene 按 (block_seq, glyph_idx) 身份补间该端点;此处只验 core 产的端点确随 reflow 更新。
        let snap = r#"[{"info":{"id":"m1","sessionID":"s","role":"user"},
            "parts":[{"type":"text","id":"p1","messageID":"m1","text":"hi there"}]}]"#;
        let mut eng = Engine::new(
            Player::from_pairs(vec![], 16.0),
            MonospaceLayout::default(),
            CollectSink::default(),
            200.0,
            800.0,
        );
        let right_edge = |f: &super::FrameData| {
            f.glyphs
                .iter()
                .map(|g| g.pos[0] + g.size[0])
                .fold(f32::MIN, f32::max)
        };
        eng.prime_from_snapshot(snap);
        eng.frame(16.0);
        let r800 = right_edge(eng.sink().last().expect("frame"));
        eng.set_max_width(1000.0); // 文档变宽 200 → 居中列右缘右移 100(= 留白增量,Plan 25 居中)
        eng.frame(16.0);
        let r1000 = right_edge(eng.sink().last().expect("frame"));
        assert!(
            (r1000 - r800 - 100.0).abs() < 5.0,
            "user 盒右沿应随文档宽右移 ~100(origin delta 进端点): {r800} → {r1000}"
        );
    }

    // ───────────────────────── Plan 18:规模 / 内存度量(before 基线)─────────────────────────

    /// 一个合成「turn」的混合 markdown(plan18 §3.1:60% 段落 / 15% 列表 / 15% 代码 / 10% 表格),
    /// 约 `lines` 行,确定性(随 turn 序变体,覆盖各 BlockCache 子结构)。
    fn bench_turn_md(turn: usize, lines: usize) -> String {
        use std::fmt::Write as _;
        let mut s = String::new();
        let para = lines * 6 / 10;
        let list = lines * 15 / 100;
        let code = lines * 15 / 100;
        let table_rows = (lines - para - list - code).max(2);
        let _ = write!(s, "# Turn {turn}\n\n");
        for i in 0..para {
            let _ = writeln!(
                s,
                "This is paragraph line {i} of turn {turn}, some **bold** and `code` words."
            );
        }
        s.push('\n');
        for i in 0..list {
            let _ = writeln!(s, "- list item {i} in turn {turn}");
        }
        s.push_str("\n```rust\n");
        for i in 0..code {
            let _ = writeln!(s, "let x{i} = {i} + turn_{turn};");
        }
        s.push_str("```\n\n| col_a | col_b |\n| --- | --- |\n");
        for i in 0..table_rows {
            let _ = writeln!(s, "| r{i}a | r{i}b |");
        }
        s.push('\n');
        s
    }

    /// 把一段文本作为「turn `i`」的 part 整体到达(各 turn 独立 part/message;`t=i+0.5`,
    /// 配 `step_ms=1.0` → 每 `frame()` 恰释放一个 turn)。
    fn bench_records(turns: usize, lines_per_turn: usize) -> Vec<(f64, String)> {
        (0..turns)
            .map(|i| {
                let md = bench_turn_md(i, lines_per_turn);
                let raw = format!(
                    r#"{{"type":"message.part.delta","properties":{{"sessionID":"s","messageID":"m{i}","partID":"p{i}","field":"text","delta":{md:?}}}}}"#
                );
                (i as f64 + 0.5, raw)
            })
            .collect()
    }

    /// Plan 19 P2:载多 turn → 屏外 settled 块释放 → `retained_glyphs` 跟可见窗(远小于满载),
    /// 且出现 Warm 档;`?novirt` 关虚拟化则全保留(对照)。
    #[test]
    fn p2_retained_falls_back_to_visible_window() {
        let make = |virt: bool| {
            let mut eng = Engine::new(
                Player::from_pairs(bench_records(40, 10), 1.0),
                MonospaceLayout::default(),
                CollectSink::default(),
                1.0e9,
                800.0,
            );
            eng.set_viewport_height(300.0); // 小视口 → 多数 turn 屏外
            eng.set_virtualize(virt);
            for _ in 0..80 {
                eng.frame(1.0); // 释放各 turn + 结算 + 回收稳定
            }
            eng.frame_stats()
        };
        let off = make(false);
        let on = make(true);
        assert_eq!(off.tier_counts[1], 0, "novirt:无 Warm,全 Hot");
        assert!(on.tier_counts[1] > 0, "virt:屏外块应释放为 Warm");
        assert!(
            on.retained_glyphs * 3 < off.retained_glyphs,
            "virt 驻留几何应远小于满载(跟可见窗): {} vs {}",
            on.retained_glyphs,
            off.retained_glyphs
        );
        assert_eq!(on.rebuilds_this_frame, 0, "稳态无 thrash(scenario C)");
    }

    /// Plan 19 P2 / R8:释放重几何 → 重入 `ensure_layouts` 重建,`placed` **逐字节等价**(确定性,
    /// 源 = `revealed`+`max_width`)。这是「释放安全」的根不变量。
    #[test]
    fn p2_release_then_rebuild_is_byte_identical() {
        let snap = r##"[
            {"info":{"id":"m1","sessionID":"s","role":"a"},"parts":[{"type":"text","id":"p1","messageID":"m1","text":"# Head one\n\npara one body text here"}]},
            {"info":{"id":"m2","sessionID":"s","role":"a"},"parts":[{"type":"text","id":"p2","messageID":"m2","text":"second block lots of words to fill several lines wrapping around the width"}]},
            {"info":{"id":"m3","sessionID":"s","role":"a"},"parts":[{"type":"text","id":"p3","messageID":"m3","text":"third block bottom anchor"}]}
        ]"##;
        // 先关虚拟化,settle,抓 p1 的 placed(释放前真值)。
        let mut eng = Engine::new(
            Player::from_pairs(vec![], 16.0),
            MonospaceLayout::default(),
            CollectSink::default(),
            1.0e9,
            200.0,
        );
        eng.set_viewport_height(30.0);
        eng.set_virtualize(false);
        eng.prime_from_snapshot(snap);
        for _ in 0..6 {
            eng.frame(16.0);
        }
        let before = eng
            .views
            .iter()
            .find(|v| v.part_id == "p1")
            .and_then(|v| v.cache.as_ref())
            .expect("p1 应有 cache")
            .placed
            .clone();
        assert!(!before.is_empty());
        // 开虚拟化:p1 在顶、锚底 → 屏外释放为 Warm(cache=None,agg 保留)。
        eng.set_virtualize(true);
        eng.frame(16.0);
        {
            let p1 = eng.views.iter().find(|v| v.part_id == "p1").expect("p1");
            assert_eq!(p1.tier, Tier::Warm, "p1 屏外应释放为 Warm");
            assert!(p1.cache.is_none(), "Warm 应丢几何");
            assert!(p1.agg.is_some(), "聚合维须保留(占位)");
        }
        // 滚到顶 → promote + 重建。
        eng.scroll_by(-1000.0);
        eng.frame(16.0);
        eng.frame(16.0);
        let after = eng
            .views
            .iter()
            .find(|v| v.part_id == "p1")
            .and_then(|v| v.cache.as_ref())
            .expect("重建后 p1 应有 cache")
            .placed
            .clone();
        assert_eq!(before, after, "释放→重建后 placed 逐字节等价(R8)");
    }

    /// Plan 19 P2 / 0029 §3:释放屏外块几何**不动**其它块布局(零跳变根)——可见块世界 y 与
    /// `?novirt` 全 Hot 时一致(聚合维 `agg.height` 占位)。
    #[test]
    fn p2_release_keeps_layout_stable() {
        let baseline = |virt: bool| {
            let mut eng = Engine::new(
                Player::from_pairs(bench_records(20, 8), 1.0),
                MonospaceLayout::default(),
                CollectSink::default(),
                1.0e9,
                800.0,
            );
            eng.set_viewport_height(250.0);
            eng.set_virtualize(virt);
            for _ in 0..60 {
                eng.frame(1.0);
            }
            // 底部锚定 → 可见区最低字底(揭示前沿)= 锚不变量。
            eng.sink()
                .last()
                .expect("frame")
                .glyphs
                .iter()
                .map(|g| g.pos[1] + g.size[1])
                .fold(0.0f32, f32::max)
        };
        let off = baseline(false);
        let on = baseline(true);
        assert!(
            (off - on).abs() < 0.5,
            "释放屏外块不应移动可见块 y(零跳变): novirt={off} virt={on}"
        );
    }

    /// Plan 21 N3:`visible_messages()` 渲染纯文本确定(同状态两次逐字节相同)且 = 渲染后字形序列。
    #[test]
    fn visible_turns_text_deterministic() {
        let player = Player::from_pairs(vec![(0.0, delta("p1", "Hello world"))], 16.0);
        let mut eng = Engine::new(
            player,
            MonospaceLayout::default(),
            CollectSink::default(),
            100_000.0,
            800.0,
        );
        for _ in 0..40 {
            eng.frame(16.0);
        }
        let a = eng.visible_messages();
        let b = eng.visible_messages();
        assert_eq!(a, b, "同状态两次调用应逐字节相同(presentation 派生,确定)");
        assert_eq!(a.len(), 1, "一条可见消息");
        let m = &a[0];
        // 单行消息:渲染纯文本 = 可见字形序列(sink 拼帧,排除换行占位)。
        assert_eq!(m.text, "Hello world");
        assert_eq!(m.text, eng.sink().visible_text(), "= 渲染后字形 join");
        assert!(!m.user, "未带 user part → assistant 默认");
        assert!(m.width > 0.0 && m.height > 0.0, "应有几何盒");
    }

    /// 跑满揭示的单行文本引擎(Plan 21 选区测试夹具):单 part、无换行 → 显示字形 1:1 字符,
    /// 等宽盒序列(MonospaceLayout 10×18)。
    fn revealed_engine(text: &str) -> Engine<Player, MonospaceLayout, CollectSink> {
        let player = Player::from_pairs(vec![(0.0, delta("p1", text))], 16.0);
        let mut eng = Engine::new(
            player,
            MonospaceLayout::default(),
            CollectSink::default(),
            100_000.0,
            800.0,
        );
        for _ in 0..40 {
            eng.frame(16.0);
        }
        eng
    }

    fn selection_rects(f: &FrameData) -> Vec<&FrameRect> {
        let sel = crate::theme::Theme::default().selection;
        let near = |a: [f32; 4], b: [f32; 4]| a.iter().zip(b).all(|(x, y)| (x - y).abs() < 1e-4);
        f.rects
            .iter()
            .filter(|r| near(r.color, sel) && r.stroke < 0.5)
            .collect()
    }

    fn rect_contains(r: &FrameRect, cx: f32, cy: f32) -> bool {
        cx >= r.pos[0] && cx <= r.pos[0] + r.size[0] && cy >= r.pos[1] && cy <= r.pos[1] + r.size[1]
    }

    proptest::proptest! {
        /// Plan 21 N1:选区 `FrameRect` 覆盖区间内每个非零墨字形盒、且不覆盖区间外字形。
        #[test]
        fn sel_highlight_rects_cover_selected_glyphs(x in 0usize..=10, y in 0usize..=10) {
            let (a, b) = (x.min(y), x.max(y));
            let mut eng = revealed_engine("abcdefghij");
            eng.set_selection(vec![(0, a, b)]);
            eng.frame(16.0);
            let f = eng.sink().last().expect("frame");
            // 单行明文:frame.glyphs 顺序 = 字符序;glyph i 即字符 i。
            prop_assert_eq!(f.glyphs.len(), 10);
            let sel = selection_rects(f);
            for (i, g) in f.glyphs.iter().enumerate() {
                let cx = g.pos[0] + g.size[0] / 2.0;
                let cy = g.pos[1] + g.size[1] / 2.0;
                let covered = sel.iter().any(|r| rect_contains(r, cx, cy));
                if (a..b).contains(&i) {
                    prop_assert!(covered, "选中字 {} 应被高亮覆盖", i);
                } else {
                    prop_assert!(!covered, "区间外字 {} 不应被覆盖", i);
                }
            }
        }
    }

    /// Plan 21 N2:空 / 越界区间 → 0 个选区高亮。
    #[test]
    fn sel_empty_range_no_highlight() {
        let mut eng = revealed_engine("abcdefghij");
        // 空区间(start==end)。
        eng.set_selection(vec![(0, 4, 4)]);
        eng.frame(16.0);
        assert_eq!(
            selection_rects(eng.sink().last().expect("f")).len(),
            0,
            "空区间无高亮"
        );
        // 越界区间(全在文本之外)。
        eng.set_selection(vec![(0, 100, 200)]);
        eng.frame(16.0);
        assert_eq!(
            selection_rects(eng.sink().last().expect("f")).len(),
            0,
            "越界区间无高亮"
        );
        // 越界 view。
        eng.set_selection(vec![(99, 0, 5)]);
        eng.frame(16.0);
        assert_eq!(
            selection_rects(eng.sink().last().expect("f")).len(),
            0,
            "越界 view 无高亮"
        );
    }

    /// Plan 26①:`set_theme` 不重排、下一帧生效 —— 自定义选区色即刻反映在选区 FrameRect;
    /// 默认主题逐值 = 旧观感(theme.rs default_matches_legacy_palette + 视觉黄金帧共同守护)。
    #[test]
    fn theme_swap_changes_emitted_colors_next_frame() {
        let near = |a: [f32; 4], b: [f32; 4]| a.iter().zip(b).all(|(x, y)| (x - y).abs() < 1e-4);
        let mut eng = revealed_engine("abcdef");
        eng.set_selection(vec![(0, 0, 4)]);
        eng.frame(16.0);
        let sel_default = crate::theme::Theme::default().selection;
        let f = eng.sink().last().expect("frame");
        assert!(
            f.rects.iter().any(|r| near(r.color, sel_default)),
            "默认主题选区色应上屏"
        );
        // 局部覆盖 selection(serde 缺字段默认)→ 下一帧新色生效、旧色不残留。
        let t: crate::theme::Theme =
            serde_json::from_str(r#"{"selection":[1.0,0.0,0.0,0.5]}"#).expect("theme json");
        eng.set_theme(t);
        eng.frame(16.0);
        let f = eng.sink().last().expect("frame");
        assert!(
            f.rects.iter().any(|r| near(r.color, [1.0, 0.0, 0.0, 0.5])),
            "新主题选区色下一帧生效"
        );
        assert!(
            !f.rects.iter().any(|r| near(r.color, sel_default)),
            "旧选区色不残留"
        );
    }

    /// Plan 21 N4:`visible_text_runs()` 不含屏外 / Warm 块(虚拟化:DOM ∝ 可见)。
    #[test]
    fn visible_text_runs_excludes_offscreen() {
        let mut eng = Engine::new(
            Player::from_pairs(bench_records(30, 6), 1.0),
            MonospaceLayout::default(),
            CollectSink::default(),
            1.0e9,
            800.0,
        );
        eng.set_viewport_height(300.0); // 小视口 → 多数块屏外(锚底 → 顶部块离屏)
        for _ in 0..80 {
            eng.frame(1.0);
        }
        let runs = eng.visible_text_runs();
        let vis: std::collections::HashSet<u32> =
            eng.visible_messages().iter().map(|m| m.id).collect();
        assert!(!runs.is_empty(), "应有可见行 run");
        // 每个 run 的块必属本帧可见集(仅 Hot 可见块)。
        for r in &runs {
            assert!(vis.contains(&r.block), "run 块 {} 必在可见集", r.block);
        }
        let blocks: std::collections::HashSet<u32> = runs.iter().map(|r| r.block).collect();
        assert!(
            blocks.len() < 30,
            "可见块数应远小于总块数(虚拟化): {}",
            blocks.len()
        );
        assert!(!blocks.contains(&0), "顶部块 0 已滚出视口,不应在文本层");
    }

    /// Plan 21 N5(R8):`set_selection` 是 presentation,不影响 reveal —— 有/无选区,末帧字形逐字段相同。
    #[test]
    fn selection_does_not_affect_reveal() {
        let run = |with_sel: bool| {
            let player =
                Player::from_pairs(vec![(0.0, delta("p1", "# Title\n\nhello world"))], 16.0);
            let mut eng = Engine::new(
                player,
                MonospaceLayout::default(),
                CollectSink::default(),
                500.0,
                800.0,
            );
            for _ in 0..30 {
                eng.frame(16.0);
            }
            if with_sel {
                eng.set_selection(vec![(0, 2, 6)]);
            }
            eng.frame(16.0);
            eng.sink().last().expect("frame").glyphs.clone()
        };
        let plain = run(false);
        let selected = run(true);
        assert_eq!(plain, selected, "选区不得扰动字形(reveal/spawn/位置确定)");
    }

    proptest::proptest! {
        /// Plan 21 N6(P3):`find` 命中序列 = 朴素子串扫描(同 query 同源 → 确定)。
        #[test]
        fn find_hits_match_naive(s in "[a-c ]{0,40}", q in "[a-c]{1,3}") {
            let eng = revealed_engine(&s);
            let hits = eng.find(&q);
            // 朴素:对同一源文本(store 全量摄入后 == s)非重叠扫描。
            let src = eng.store().part_text("p1").unwrap_or("");
            let mut naive: Vec<(u32, u32)> = Vec::new();
            let mut from = 0usize;
            while let Some(rel) = src[from..].find(q.as_str()) {
                let byte = from + rel;
                naive.push((0, src[..byte].chars().count() as u32));
                from = byte + q.len();
            }
            prop_assert_eq!(hits, naive);
        }
    }

    /// Plan 21 N7(P3):选区墨团(逐行合并圆角条)**包含**每个被选非零墨字形盒(升级不漏选)。
    #[test]
    fn selection_ink_blob_contains_line_rects() {
        // 多段 → 多行显示字形(行间 "\n" 占位 → 逐行 flush 出多条墨团)。
        let mut eng = revealed_engine("alpha\n\nbravo\n\ncharlie delta");
        let n = eng.sink().last().map_or(0, |f| f.glyphs.len());
        eng.set_selection(vec![(0, 0, n + 50)]); // 覆盖全部(end 越界 → clamp)
        eng.frame(16.0);
        let f = eng.sink().last().expect("frame");
        let blobs = selection_rects(f);
        assert!(!blobs.is_empty(), "应有墨团");
        // 每个可见非零墨字形盒(四角)必被某条墨团包含。
        let inside = |r: &FrameRect, x: f32, y: f32| {
            x >= r.pos[0] - 0.01
                && x <= r.pos[0] + r.size[0] + 0.01
                && y >= r.pos[1] - 0.01
                && y <= r.pos[1] + r.size[1] + 0.01
        };
        for g in &f.glyphs {
            if g.size[0] <= 0.0 {
                continue;
            }
            let covered = blobs.iter().any(|r| {
                inside(r, g.pos[0], g.pos[1])
                    && inside(r, g.pos[0] + g.size[0], g.pos[1] + g.size[1])
            });
            assert!(covered, "字形 {:?}@{:?} 未被墨团包含", g.cluster, g.pos);
        }
        // 墨团数 ≤ 行数(逐行合并,非逐字)→ 证"合并"确实发生(远少于字形数)。
        assert!(blobs.len() <= f.glyphs.len(), "墨团应合并(条数 ≤ 字形数)");
    }

    /// Plan 21 P3:`scroll_to` 把屏外块移入可见(锚底脱离 + 虚拟化 promote)。
    #[test]
    fn scroll_to_brings_block_into_view() {
        let mut eng = Engine::new(
            Player::from_pairs(bench_records(30, 6), 1.0),
            MonospaceLayout::default(),
            CollectSink::default(),
            1.0e9,
            800.0,
        );
        eng.set_viewport_height(300.0);
        for _ in 0..80 {
            eng.frame(1.0);
        }
        // 顶部块 0 初始屏外(锚底)。
        assert!(
            !eng.visible_messages().iter().any(|m| m.id == 0),
            "块 0 初始应屏外"
        );
        eng.scroll_to(0);
        for _ in 0..10 {
            eng.frame(1.0); // 数帧:promote → 重排 → 可见
        }
        assert!(
            eng.visible_messages().iter().any(|m| m.id == 0),
            "scroll_to 后块 0 应可见"
        );
    }

    /// 度量字段随内容增长(非 ignored 回归:retained_* 被填且单调随历史增长)。
    #[test]
    fn scale_stats_grow_with_history() {
        let recs = bench_records(6, 20);
        let mut eng = Engine::new(
            Player::from_pairs(recs, 1.0),
            MonospaceLayout::default(),
            CollectSink::default(),
            1.0e9, // 巨 cps → 一帧内整流+排版到达内容
            800.0,
        );
        eng.set_viewport_height(800.0);
        eng.frame(1.0); // 释放 turn 0
        let s1 = eng.frame_stats();
        assert!(s1.retained_glyphs > 0, "首 turn 应有驻留几何");
        assert!(s1.retained_views >= 1 && s1.retained_nodes > 0);
        assert!(s1.store_chars > 0);
        for _ in 0..6 {
            eng.frame(1.0); // 释放其余 turn
        }
        let s2 = eng.frame_stats();
        assert!(
            s2.retained_glyphs > s1.retained_glyphs,
            "驻留几何应随历史增长(before:无虚拟化,全保留): {} → {}",
            s1.retained_glyphs,
            s2.retained_glyphs
        );
        assert!(s2.retained_views > s1.retained_views, "驻留 view 数应增长");
    }

    /// Plan 18 §4 三场景 before 基线采集器(GPU 无关:retained_* / store_chars 纯 CPU 数据结构)。
    /// fps / wasm 线性内存须浏览器 `?bench`(本测不覆盖)。运行:
    ///   `cargo test -p infinite-chat-core --release bench_scale_before -- --ignored --nocapture`
    #[test]
    #[ignore = "plan29 after 采集器(release 手跑):cargo test -p infinite-chat-core --release bench_scale_after -- --ignored --nocapture"]
    fn bench_scale_after() {
        // Plan 29 V4:同 plan18 场景、虚拟化**开**(默认)→ after 曲线。B 滚动来回后
        // retained 回落到 Hot 工作集(vs before 恒 == 满载 463850)。
        const LINES_PER_TURN: usize = 50;
        const TURNS: usize = 200;
        let recs = bench_records(TURNS, LINES_PER_TURN);
        let mut eng = Engine::new(
            Player::from_pairs(recs, 1.0),
            MonospaceLayout::default(),
            CollectSink::default(),
            1.0e9,
            900.0,
        );
        eng.set_viewport_height(800.0);
        for _ in 0..(TURNS + 60) {
            eng.frame(1.0);
        }
        let full = eng.frame_stats();
        eprintln!("# Plan 29 after(虚拟化开;tier=[hot,warm,cold,frozen])");
        eprintln!(
            "full_bottom retained_glyphs={} tiers={:?}",
            full.retained_glyphs, full.tier_counts
        );
        // 滚到顶 → 底部远端块降档。
        eng.scroll_to(0);
        for _ in 0..60 {
            eng.frame(1.0);
        }
        let top = eng.frame_stats();
        eprintln!(
            "at_top      retained_glyphs={} tiers={:?}",
            top.retained_glyphs, top.tier_counts
        );
        // 回底 → 顶部远端块降档;Hot 工作集 = 视口带内。
        eng.scroll_to(TURNS - 1);
        for _ in 0..60 {
            eng.frame(1.0);
        }
        let back = eng.frame_stats();
        eprintln!(
            "back_bottom retained_glyphs={} tiers={:?}",
            back.retained_glyphs, back.tier_counts
        );
        // DoD-1(native 版):驻留 ≤ before 满载(novirt 463850,plan18/29 progress)量级的 20%
        // ——「内存跟可见屏走」。full 此处已虚拟化(≈Hot 窗口),back 与其同量级即稳定。
        assert!(
            back.retained_glyphs < 50_000,
            "after 驻留应为 Hot 窗口量级(before 满载 463850): back={}",
            back.retained_glyphs
        );
        assert!(
            back.retained_glyphs <= full.retained_glyphs * 2,
            "来回滚动不应使驻留攀升: back={} full={}",
            back.retained_glyphs,
            full.retained_glyphs
        );
        assert!(
            back.tier_counts[2] > 0,
            "应有 Cold 块: {:?}",
            back.tier_counts
        );
    }

    #[test]
    #[ignore = "plan18 before 采集器(release 手跑)"]
    fn bench_scale_before() {
        const LINES_PER_TURN: usize = 50;
        const TURNS: usize = 200; // 200 × 50 = 10k 行
        const VIEWPORT_H: f32 = 800.0;
        let recs = bench_records(TURNS, LINES_PER_TURN);
        let mut eng = Engine::new(
            Player::from_pairs(recs, 1.0),
            MonospaceLayout::default(),
            CollectSink::default(),
            1.0e9,
            900.0,
        );
        eng.set_viewport_height(VIEWPORT_H);
        eng.set_virtualize(false); // 这是 plan18 **before**(无虚拟化)基线采集器;P2 后用 ?bench 测 after

        // ── 场景 A:增长曲线(每释放一个 turn 一帧;每 1k 行采样)──
        eprintln!("# Plan 18 before 基线(native;retained_* / store_chars 为主指标)");
        eprintln!("scenario,lines,turns,store_chars,retained_views,retained_glyphs,retained_nodes,frame_glyphs");
        let sample =
            |eng: &Engine<Player, MonospaceLayout, CollectSink>, tag: &str, turns_done: usize| {
                let s = eng.frame_stats();
                eprintln!(
                    "{tag},{},{turns_done},{},{},{},{},{}",
                    turns_done * LINES_PER_TURN,
                    s.store_chars,
                    s.retained_views,
                    s.retained_glyphs,
                    s.retained_nodes,
                    s.frame_glyphs,
                );
                s
            };
        let lines_per_1k = 1000 / LINES_PER_TURN; // = 每多少 turn 满 1k 行
        let mut a_samples: Vec<(usize, usize)> = Vec::new(); // (lines, retained_glyphs)
        for t in 1..=TURNS {
            eng.frame(1.0);
            if t % lines_per_1k == 0 {
                let s = sample(&eng, "A_growth", t);
                a_samples.push((t * LINES_PER_TURN, s.retained_glyphs));
            }
        }
        let full = eng.frame_stats();

        // ── 场景 B:滚到顶 → 回底(before:屏外不释放 → retained 不回落)──
        let total_h = 1.0e7; // 远超内容总高 → pan 夹到顶
        eng.pan_by(0.0, -total_h); // 滚到顶
        for _ in 0..4 {
            eng.frame(1.0);
        }
        let at_top = sample(&eng, "B_top", TURNS);
        eng.pan_by(0.0, total_h); // 回底
        for _ in 0..4 {
            eng.frame(1.0);
        }
        let back_bottom = sample(&eng, "B_back_bottom", TURNS);

        // ── 场景 C:静止结算(无新内容,多帧;retained 应恒定)──
        for _ in 0..30 {
            eng.frame(16.0);
        }
        let settled = sample(&eng, "C_settled", TURNS);

        // ── 斜率 + before 断言 ──
        let slope = match (a_samples.first(), a_samples.last()) {
            (Some(&(l0, g0)), Some(&(l1, g1))) if l1 > l0 => (g1 - g0) as f64 / (l1 - l0) as f64,
            _ => 0.0,
        };
        eprintln!("# A 斜率(retained_glyphs / 行)= {slope:.2}  → ∝ 历史(线性增长)");
        eprintln!(
            "# B 回底/满载 retained_glyphs = {}/{}(before:屏外不释放,应≈相等)",
            back_bottom.retained_glyphs, full.retained_glyphs
        );
        eprintln!(
            "# C settled retained_glyphs = {}(静止恒定)",
            settled.retained_glyphs
        );

        // before 北极星:A 线性正增长;B 回底不回落(== 满载,屏外不释放);C 恒定。
        assert!(slope > 0.0, "A:retained_glyphs 应随行数线性正增长");
        assert!(
            full.retained_glyphs > VIEWPORT_H as usize,
            "10k 行驻留几何应远超一屏可容(before:无虚拟化)"
        );
        assert_eq!(
            back_bottom.retained_glyphs, full.retained_glyphs,
            "B before:屏外不释放 → 回底 retained == 满载"
        );
        assert_eq!(
            at_top.retained_glyphs, full.retained_glyphs,
            "B before:滚到顶 retained 也不变"
        );
        assert_eq!(
            settled.retained_glyphs, full.retained_glyphs,
            "C:静止 retained 恒定(无 thrash)"
        );
    }
}
