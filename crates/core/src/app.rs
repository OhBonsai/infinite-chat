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
use crate::spatial::SpatialGrid;
use crate::store::Store;
use crate::support::{graphemes, EventQueue};
use crate::theme;

/// catch-up 字形的 spawn_time:置于"远古",着色器淡入早已完成(alpha=1),实现零动画(AR6)。
const CATCHUP_SPAWN: f32 = -1.0e9;

/// 块间纵向间距(px)。
const BLOCK_GAP: f32 = 8.0;

/// 显示数学(`$$…$$`)相对行内的字号倍率:= H3(`roleScale` 1.3),更舒展 + 居中。
const DISPLAY_MATH_SCALE: f32 = 1.3;
/// 数学字形/规则线颜色(RGBA;暗色主题中性亮);后续可走 theme/可配。
const MATH_COLOR: [f32; 4] = [0.86, 0.88, 0.92, 1.0];
/// 数学 glyph 的 `glyph_idx` 基址:远离正文 placed 下标,morph 身份(block_seq,glyph_idx)不撞。
const MATH_IDX_BASE: u32 = 1_000_000;

/// 锚底阈值:滚到离底 ≤ 此值即重新跟随新内容(0002 §6)。
const ANCHOR_THRESHOLD: f32 = 48.0;

/// 锚底**平滑跟随**:临界阻尼 smooth-damp 的接近时间(秒,fps 无关;小=更跟手,大=更顺滑)。
/// 落后 > **一屏**(初次加载 / 历史瞬显 / 大段倾泻)才直接到位,否则平滑跟——流式不再 snap。
const ANCHOR_SMOOTH_TIME: f32 = 0.12;

/// 把累积的行内码 chip(`[x0,x1,y0,y1]`)推成一个带内边距的圆角底。
fn flush_chip(chip: Option<[f32; 4]>, out: &mut Vec<FrameRect>) {
    if let Some([x0, x1, y0, y1]) = chip {
        out.push(FrameRect {
            pos: [x0 - 2.0, y0 - 1.0],
            size: [(x1 - x0) + 4.0, (y1 - y0) + 2.0],
            color: theme::CODE_CHIP,
            radius: 3.0,
            stroke: 0.0,
        });
    }
}

/// 把累积的删除线段(`[x0,x1,y0,y1]`)推成字中线一条细线(A:`~~…~~`,表格内/正文通用)。
fn flush_strike(seg: Option<[f32; 4]>, out: &mut Vec<FrameRect>) {
    if let Some([x0, x1, y0, y1]) = seg {
        out.push(FrameRect {
            pos: [x0, (y0 + y1) * 0.5 - 0.75], // 字形垂直中点
            size: [x1 - x0, 1.5],
            color: theme::STRIKE,
            radius: 0.0,
            stroke: 0.0,
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
    origin: [f32; 2], // Plan 13:盒左上角 world 坐标(装饰随 view 盒平移)
    box_w: f32,       // 盒宽(全宽装饰:代码底/引用条/分隔线/表头线锚它,非整窗宽)
    ts: &TableStyle,
    spawn: &[Option<f32>],
    reveal_kind: TableStyleKind,
    out: &mut Vec<FrameRect>,
    panels: &mut Vec<FramePanel>,
    widgets: &mut Vec<FrameWidget>,
) {
    let inline = StyleRole::Code.as_u32();
    let quote = StyleRole::Quote.as_u32();
    let alert = StyleRole::AlertLabel.as_u32();
    let rule = StyleRole::Rule.as_u32();
    let h1 = StyleRole::Heading.as_u32();
    let h2 = StyleRole::Heading2.as_u32();
    let task_off = StyleRole::TaskUnchecked.as_u32();
    let task_on = StyleRole::TaskChecked.as_u32();
    let (mut qy0, mut qy1) = (f32::MAX, f32::MIN);
    let (mut has_quote, mut has_head_rule) = (false, false);
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
            flush_chip(chip.take(), out);
            flush_strike(strike_seg.take(), out);
            continue;
        }
        let (x0, y0) = (p.pos[0] + origin[0], p.pos[1] + origin[1]);
        let (x1, y1) = (x0 + p.size[0], y0 + p.size[1]);
        let r = cache.roles[j];
        // 代码块底 / 框 / gutter 在循环后**逐块**从 `cache.code_blocks` 几何发(见下),不在此累加
        // (多代码块合并成一个大框是 boundary bug)。
        // 引用与 Alert 共用左条范围;Alert 标签字形拼出类型用于取色。
        if r == quote || r == alert {
            has_quote = true;
            qy0 = qy0.min(y0);
            qy1 = qy1.max(y1);
            if r == alert {
                alert_label.push_str(&cache.clusters[j]);
            }
        }
        if r == h1 || r == h2 {
            has_head_rule = true;
        }
        // 分隔线:零墨 Rule 锚点 → 整宽细线(居其行垂直中点)。已释放才到此(循环顶部已门控)→
        // 随揭示节点出现(NodeSpawn,Plan 9 §2.6:ThematicBreak 标其 Rule 锚字 → 释放即画)。
        if r == rule {
            // 分隔线 `---` → 喵喵分隔线 widget(默认,Plan 11):线条画的猫坐在分割线上。quad 需较高
            // 容纳猫(40px),分割线居 quad 偏下、猫在其上;猫几何在 rule_cat.wgsl 按 quad 高自适应。
            // 渐变线版仍可用(WIDGET_RULE);如需朴素线把 component 改回去即可。
            let mid = (y0 + y1) * 0.5;
            let qh = 72.0; // 容纳较大的猫(升起 + 身体);线在 quad 偏下(LINE_FRAC),猫在其上
            widgets.push(FrameWidget {
                pos: [origin[0], mid - qh + 14.0], // 线接近 rule 行中线;猫向上延展
                size: [box_w, qh],
                color: theme::HR_RULE,
                params: [0.0, 0.0, 0.0, 0.0],
                component: crate::frame::WIDGET_RULE_CAT,
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
                color: if checked {
                    theme::TASK_DONE
                } else {
                    theme::TASK_BOX
                },
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
                    flush_chip(chip, out);
                    chip = Some([x0, x1, y0, y1]);
                }
            }
        } else if chip.is_some() {
            flush_chip(chip, out);
            chip = None;
        }
        // 删除线(A):连续 struck glyph 同行聚成一段,逐行/逐段 flush → 字中线一条细线。
        if cache.strike[j] {
            match strike_seg {
                Some(c) if (c[2] - y0).abs() < 0.5 => {
                    strike_seg = Some([c[0], x1, c[2].min(y0), c[3].max(y1)]);
                }
                _ => {
                    flush_strike(strike_seg, out);
                    strike_seg = Some([x0, x1, y0, y1]);
                }
            }
        } else if strike_seg.is_some() {
            flush_strike(strike_seg, out);
            strike_seg = None;
        }
    }
    flush_chip(chip, out);
    flush_strike(strike_seg, out);
    if has_head_rule {
        // GitHub:H1/H2 底部细线,跨整块宽。
        let ry = origin[1] + cache.height - 2.0;
        out.push(FrameRect {
            pos: [origin[0], ry],
            size: [box_w, 1.5],
            color: theme::HEAD_RULE,
            radius: 0.0,
            stroke: 0.0,
        });
    }
    // 代码块底 / 框 / gutter:**逐块**从 `cache.code_blocks` 几何发(Plan 15 ①②⑥)。盒 = 全宽 × 行窗高
    // (`min(N,6)·lineH`),top = 块顶(块内相对 + origin),不会合并多块或盖住块间内容(修 box 边界 bug)。
    // 揭示门:块内有已释放字才发(避免流式空框先现)。
    for cb in &cache.code_blocks {
        let revealed =
            (cb.range.0..cb.range.1).any(|j| spawn.get(j as usize).copied().flatten().is_some());
        if !revealed {
            continue;
        }
        let win_h = crate::codeblock::window_height(cb.n_lines, cb.line_h);
        let bg_pos = [origin[0], origin[1] + cb.top_y - 4.0];
        let bg_size = [box_w, win_h + 8.0];
        out.push(FrameRect {
            pos: bg_pos,
            size: bg_size,
            color: theme::CODE_BG,
            radius: 6.0,
            stroke: 0.0,
        });
        // 外框描边(Plan 15 ⑥:可见 box 框)。stroke>0 → 仅边框(rect.wgsl)。
        out.push(FrameRect {
            pos: bg_pos,
            size: bg_size,
            color: theme::CODE_BORDER,
            radius: 6.0,
            stroke: 1.5,
        });
        // gutter 分隔线(②⑥):行号列与代码区之间一条细竖线,跨行窗高。
        if cb.code_x0 > 0.0 {
            out.push(FrameRect {
                pos: [origin[0] + cb.code_x0 - 4.0, origin[1] + cb.top_y - 2.0],
                size: [1.0, win_h + 4.0],
                color: theme::CODE_GUTTER_LINE,
                radius: 0.0,
                stroke: 0.0,
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
            flags: crate::frame::PANEL_GRID | crate::frame::PANEL_AO,
        });
    }
    if has_quote {
        let is_alert = !alert_label.is_empty();
        // Alert:整块淡底(GitHub 风)+ 类型色左条;普通引用:中性左条。
        if is_alert {
            out.push(FrameRect {
                pos: [origin[0], qy0 - 3.0],
                size: [box_w, (qy1 - qy0) + 6.0],
                color: theme::alert_bg(&alert_label),
                radius: 5.0,
                stroke: 0.0,
            });
        }
        out.push(FrameRect {
            pos: [origin[0], qy0],
            size: [3.0, qy1 - qy0],
            color: if is_alert {
                theme::alert_bar(&alert_label)
            } else {
                theme::QUOTE_BAR
            },
            radius: 0.0,
            stroke: 0.0,
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
    /// 代码块行窗视口(Plan 15 ①):每个 fenced code block 的 (glyph 区间, 块顶 y, 行数, 行高)。超
    /// `MAX_LINES` 行时 ensure_layouts 已把其**后**内容上移钉死窗高;build_frame 据此 scroll/cull/fade。
    code_blocks: Vec<CodeView>,
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
/// Agent 回复 logo(glow-orb)头像尺寸 / 与盒左缘间距(world px,Plan 16 §2.6)。
const AVATAR_PX: f32 = 32.0;
const AVATAR_GAP: f32 = 8.0;

/// 每个可见 part 的上屏进度 + 排版缓存。
/// 工作集档位(Plan 19 P2 / 0029):本期两档。`Hot` = 几何就绪(可绘制);`Warm` = 屏外 settled
/// 已**释放重几何**(`cache=None`),只留 [`BlockAgg`] 占位,进可见滞回带再 `ensure_layouts` 重建。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tier {
    Hot,
    Warm,
}

/// 块的**聚合维**(Plan 19 P2):释放重几何后仍保留 → boxlayout 占位稳定(零跳变,0029 §3)。
/// = 内容宽 + 高 + 排版时源 grapheme 数(重入 dirty 判据)。
#[derive(Debug, Clone, Copy)]
struct BlockAgg {
    content_width: f32,
    height: f32,
}

struct PartView {
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
}

/// 渲染后纯文本(Plan 21 P1):显示字形序列 join。`clusters` 已是 markdown 渲染后字形(语法标记已去),
/// 含 `"\n"` 占位 → concat 即得带换行的纯文本。
fn rendered_text(clusters: &[String]) -> String {
    clusters.concat()
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
/// 颜色 [`theme::SELECTION`];进 `rects`(glyph 前绘制)→ 文字永在其上。
fn push_selection_rects(
    cache: &BlockCache,
    origin: [f32; 2],
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
                color: crate::theme::SELECTION,
                radius: (h * 0.28).min(6.0), // 圆角墨团(P3)
                stroke: 0.0,
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
    /// 真相源文本总量(`Σ Store.parts[*].text.len()` 字节;历史规模代理)。Plan 18 §2.1 度量。
    pub store_chars: usize, // (Plan 19 per-phase 计时见 [`PhaseMs`],单列因 f32 不能 Eq)
    /// 驻留 PartView 数(`engine.views.len()`)。Plan 18 §2.1。
    pub retained_views: usize,
    /// **驻留逐字几何总量**(`Σ view.cache.placed.len()`,含离屏/已冻块)= 0029 主攻对象。Plan 18 §2.1。
    pub retained_glyphs: usize,
    /// 驻留节点树规模(`Σ view.cache.nodes.len()`)。Plan 18 §2.1。
    pub retained_nodes: usize,
    /// 工作集档位分布 `[Hot, Warm]`(Plan 19 P2)。Warm = 已释放几何的屏外块。
    pub tier_counts: [usize; 2],
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
            line_color: theme::TABLE_RULE,
            header_fill: theme::TABLE_HEADER_BG,
            line_w: 1.0,
            ao: 0.12,
            ao_color: [1.0, 1.0, 1.0],
            ao_width: 10.0,
            radius: 4.0,
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
    /// 锚底:在底部时新内容跟随滚动(0002 §6)。
    stick_to_bottom: bool,
    /// 锚底平滑跟随的垂直速度态(smooth-damp;0016 风格速度连续,消除换行 scroll 顿挫)。
    pan_vel_y: f32,
    /// CPU 空间索引(Plan 3 L):逐帧由块 AABB 重建,视口查可见块。
    grid: SpatialGrid,
    /// 调试几何叠加(Plan 4C3):块 AABB / 视口框。
    debug_geometry: bool,
    /// 回合收尾跟踪(Phase I):多信号 + 看门狗,解决"忘了 idle 卡死"。
    turn: TurnTracker,
    /// 上一帧渲染统计(可观测)。
    last_stats: FrameStats,
    /// 表格面板可调渲染样式(web 层实时改;每帧读,见 [`TableStyle`])。
    table_style: TableStyle,
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
            stick_to_bottom: true,
            pan_vel_y: 0.0,
            grid: SpatialGrid::new(),
            debug_geometry: false,
            turn: TurnTracker::new(),
            last_stats: FrameStats::default(),
            table_style: TableStyle::default(),
            scheduler: RevealScheduler::new(),
            math_em: 32.0,
            image_registry: std::collections::HashMap::new(),
            code_scroll: std::collections::HashMap::new(),
            code_hit_rects: Vec::new(),
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
                    (None, Tier::Warm) => match v.agg {
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
        let boxpos = crate::boxlayout::layout_chat(&turns, &sizes, self.max_width);
        boxpos.get(view).map(|b| b.origin[1])
    }

    /// Plan 21 P3:跳到某 view(Cmd+F 命中后):相机平移到该块顶(脱离锚底)。下帧虚拟化把它 promote
    /// 回 Hot → 可见 + 可逐字选。presentation(同相机),不破 R8。
    pub fn scroll_to(&mut self, view: usize) {
        if let Some(y) = self.view_world_y(view) {
            let x = self.camera.pan()[0];
            self.camera.set_pan(x, y.max(0.0));
            self.stick_to_bottom = false;
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
                self.image_registry
                    .entry(key)
                    .or_insert_with(|| crate::embed::Embed::new(&region.url, &region.alt));
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
            self.stick_to_bottom = false;
        }
    }

    /// 二维平移 `dx,dy` 屏幕像素(触摸板两指滚动 / 拖拽;web 层输入统一入口)。横向自由平移(宽表
    /// 溢出可拖看),纵向同 `scroll_by`。任意横移或上移即脱离锚底,滚回底部时 `build_frame` 复跟随。
    pub fn pan_by(&mut self, dx: f32, dy: f32) {
        self.camera.pan_by_screen(dx, dy);
        if dx != 0.0 || dy < 0.0 {
            self.stick_to_bottom = false;
        }
    }

    /// 围绕屏幕点缩放(Plan 3 L:ctrl+滚轮 / 双指)。缩放即脱离锚底。
    pub fn zoom_by(&mut self, factor: f32, screen_x: f32, screen_y: f32) {
        self.camera.zoom_at(factor, screen_x, screen_y);
        self.stick_to_bottom = false;
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
        let mut quota = self.scheduler.quota();
        let now = self.now_ms as f32;
        let table_style = self.scheduler.table_style();
        let mut released = 0usize;
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
            for g in cand {
                if quota == 0 {
                    break;
                }
                view.spawn[g] = Some(now + plan.delay_ms[g]);
                quota = quota.saturating_sub(1);
                released += 1;
            }
            // 结算判定:所有**非换行**字已释放(换行永不 spawn,故不阻塞冻结 —— 修 Plan 9 #1:否则
            // 含换行/NodeSpawn 的活动 view 永不冻结、每帧重 resolve)。settled 后下帧起 O(1) 跳过。
            if (0..gcount).all(|g| view.spawn[g].is_some() || cache.clusters[g] == "\n") {
                view.settled = true;
            }
        }
        self.scheduler.consume(released);
        // 真正空闲(无任何待揭字)才清预算,避免空转后突发;限速挡住时保留预算攒到下帧。
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
                // 权限 / 提问 → 阻塞;应答 → 解阻(Dock 在 TS,P4)。
                Ok(Event::PermissionAsked { .. }) => self.drive_fsm(&FsmInput::PermissionAsked),
                Ok(Event::QuestionAsked { .. }) => self.drive_fsm(&FsmInput::QuestionAsked),
                Ok(Event::PermissionReplied { .. } | Event::QuestionReplied { .. }) => {
                    self.drive_fsm(&FsmInput::Replied);
                }
                Ok(Event::QuestionRejected { .. }) => self.drive_fsm(&FsmInput::Idle),
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
        self.drive_fsm(&FsmInput::Send {
            now_ms: self.now_ms,
        });
    }

    /// 用户停止(Plan 22 P4 / F11):FSM Stopped + 冻结当前流式消息(丢弃其后续事件)+ epoch+1。
    pub fn stop(&mut self) {
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

    /// 用户应答权限/反问(Plan 22 P4:Dock 点击 → 解阻)。host 同时 POST 真实 reply。
    pub fn note_reply(&mut self) {
        self.drive_fsm(&FsmInput::Replied);
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
        for pid in orphans {
            self.smoother.reset_part(&pid);
            let v = self.view_mut(&pid);
            v.revealed.clear();
            v.pushed = 0;
            v.pushed_bytes = 0;
            v.spawn.clear();
            v.cache = None;
            v.settled = false;
            v.agg = None;
        }
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
            if self.views[i].tier == Tier::Warm {
                continue;
            }
            let len = self.views[i].revealed.len();
            let dirty = match &self.views[i].cache {
                Some(c) => c.revealed_len != len || (c.width - self.max_width).abs() > f32::EPSILON,
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
            let (spans, tables, nodes, embeds) = if let Some((kind, rp)) = spec {
                let ctx = crate::partrender::RenderCtx {
                    width: self.max_width,
                    folded: false,
                };
                let spans = self.registry.render(kind, &rp, &ctx);
                let total: u32 = spans.iter().map(|s| graphemes(s.text()).len() as u32).sum();
                if total == 0 {
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
            let result = self.layout.layout(&spans, &tables, self.max_width);
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
                let extra_below = (dh - (0.2 * line + BLOCK_GAP)).max(0.0); // 公式下溢
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
                width: self.max_width,
                clusters,
                roles,
                strike,
                placed,
                content_width,
                height,
                // 各表格面板几何(同源 colX/rowY,0018 #5):layout 回传,逐表收敛成一个 SDF 面板。
                table_panels: result.table_panels,
                nodes,
                math,
                embeds,
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
        let boxpos = crate::boxlayout::layout_chat(&turns, &sizes, self.max_width);
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
                (None, Tier::Warm) => match view.agg {
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

        // 2) 重建空间索引(块 AABB)。
        self.grid.clear();
        for &(i, origin, box_w, h) in &drawable {
            self.grid
                .insert(i, &Rect::new(origin[0], origin[1], box_w, h));
        }

        // 3) 锚底:相机 pan.y **平滑**跟随底部并夹取。直接 set 到底会在每次换行(content 高 +一行)
        //    整屏一次性上移一行 = "换行跳一下";改为指数趋近底部(fps 无关),小跳平滑、大跳(初次/
        //    历史瞬显)直接到位避免慢 scroll 穿过整篇。字本身的重排已由 0016 morph 补间。
        let visible_h = self.camera.viewport()[1] / self.camera.zoom();
        // 锚底跟「已揭示底」(严格 bottom-line);未揭示的解析尾不预滚。
        let max_pan_y = (revealed_height - visible_h).max(0.0);
        let mut pan = self.camera.pan();
        if self.stick_to_bottom {
            if (max_pan_y - pan[1]).abs() > visible_h {
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
        } else {
            self.pan_vel_y = 0.0; // 用户接管(滚动/缩放)→ 清速度,免重新跟随时残留
        }
        pan[1] = pan[1].clamp(0.0, max_pan_y);
        self.camera.set_pan(pan[0], pan[1]);
        if pan[1] >= max_pan_y - ANCHOR_THRESHOLD {
            self.stick_to_bottom = true;
        }

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
        let mut visible_blocks = 0usize; // 可观测:实际出 glyph 的块数
        let mut hit_rects: Vec<(u64, Rect)> = Vec::new(); // Plan 15 ④:代码块行窗世界命中矩形
        let mut visible_recs: Vec<(usize, u32, [f32; 2], f32, f32)> = Vec::new(); // Plan 21:可见块世界盒
        let mut selection_rect_count = 0usize; // Plan 21 P2:本帧选区高亮数(可观测)
        let selection = self.selection.clone(); // Plan 21 P2:本帧选区(快照,避免借用冲突;小)
        let reveal_kind = self.scheduler.table_style(); // 表格揭示风格(驱动面板骨架揭示)
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
            // Agent 回复 logo(Plan 16 §2.6):assistant 盒左侧钉一个 dynamic glow-orb 头像;流式
            // (未 settled)= 加速脉冲作 busy 指示。护栏:离屏 cull(下方 box_rect.intersects)+ 节流时钟。
            if view.role == crate::store::Role::Assistant {
                let apos = [origin[0] - AVATAR_PX - AVATAR_GAP, origin[1]];
                let arect = Rect::new(apos[0], apos[1], AVATAR_PX, AVATAR_PX);
                if arect.intersects(&visible) {
                    let mut params = [0.0f32; 8];
                    params[3] = if view.settled { 1.0 } else { 2.6 }; // p0.w = 脉冲速度(流式更快)
                    params[4] = 0.62; // p1.x = 内半径
                    shaderboxes.push(crate::FrameShaderBox {
                        pos: apos,
                        size: [AVATAR_PX, AVATAR_PX],
                        shader_id: crate::ShaderId::GlowOrb.as_u32(),
                        params,
                        bg: [0.0, 0.0, 0.0, 0.0],
                        time: self.shaderbox_clock.time_s(),
                        dynamic: true,
                        channel0: 0,
                    });
                    shaderbox_pixels += arect.overlap_area(&visible).round().max(0.0) as u64;
                }
            }
            block_decorations(
                cache,
                id as u32, // block_seq:面板稳定身份高位(6D)
                origin,    // Plan 13:盒 origin(x,y),装饰整体平移到盒位
                box_w,     // 盒宽(全宽装饰:代码底/引用条/分隔线锚它,非整窗宽)
                &self.table_style,
                &view.spawn,
                reveal_kind,
                &mut rects,
                &mut panels,
                &mut widgets,
            ); // 4B/6 装饰 + Plan 11 复选框
               // Plan 21 P2:选区高亮(在装饰之后、glyph 之前入 rects → 压装饰底之上、文字之下)。
            selection_rect_count += push_selection_rects(cache, origin, &selection, id, &mut rects);
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
                let eff_y = placed.pos[1] + origin[1] - up_shift; // 行窗 scroll 后的世界 y
                let eff_x = placed.pos[0] + origin[0] - left_shift; // 行窗横滚后的世界 x
                                                                    // 横裁(Plan 15 ⑤):横滚后整字落在代码区外(左压 gutter / 右溢盒)→ 硬裁不发(CPU 整字
                                                                    // 粒度;部分裁切的丝滑边缘留 GPU scissor / shader x-clip 后续)。
                if let Some((cl, cr)) = x_clip {
                    if eff_x + placed.size[0] <= cl || eff_x >= cr {
                        continue;
                    }
                }
                // glyph 级 y 裁剪:单条长消息是一个巨块,块级裁剪不够 —— 块内只发与视口相交
                // 的字,把每帧发射量从"整篇"降到"约一屏",根治长消息的每帧分配风暴。
                let gworld = Rect::new(eff_x, eff_y, placed.size[0], placed.size[1]);
                if !gworld.intersects(&visible) {
                    continue;
                }
                glyphs.push(FrameGlyph {
                    cluster: cache.clusters[j].clone(),
                    pos: [eff_x, eff_y], // 世界(盒 origin 平移 + 行窗 纵/横 scroll)
                    size: placed.size,
                    spawn_time: spawn,
                    style: cache.roles[j],
                    // 身份(0016/0017):块在 views 里的下标(append-only 稳定)+ 块内 placed 下标。
                    block_seq: id as u32,
                    glyph_idx: j as u32,
                    // 进场 profile(0025/Plan 10 §3b):按角色 + reveal 风格选,shader 据 id 查表。
                    anim: enter_profile_id(cache.roles[j], reveal_kind),
                    alpha: code_alpha, // 行窗边缘淡入淡出(Plan 15 ①;非代码块恒 1)
                });
            }
            // 数学块(Plan 12 ②③):RaTeX 排版 → 数学 SDF 字形(em×px → world)+ 规则线。字号 = 正文
            // `math_em`(行内贴正文)或 ×1.3 = H3(显示数学,更舒展);显示数学**整式水平居中**。
            // spawn = 该块已揭字最晚上屏时刻(随块揭示淡入);未揭则跳过。glyph_idx 用高位基避免 morph 撞。
            for &((s, e), ref m, display) in &cache.math {
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
                    g.glyph_idx = MATH_IDX_BASE + s + k as u32;
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
                if animated {
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
                    color: theme::DBG_BLOCK,
                    radius: 0.0,
                    stroke: 1.5,
                });
                // 节点树:逐容器节点描其 glyph range 的 AABB(肉眼验树,复用 4C3 叠加,7E)。
                if let Some(cache) = self.views[id].cache.as_ref() {
                    node_debug_rects(&cache.nodes, &cache.placed, origin, &mut rects);
                }
            }
            rects.push(FrameRect {
                pos: [visible.x, visible.y],
                size: [visible.w, visible.h],
                color: theme::DBG_VIEW,
                radius: 0.0,
                stroke: 2.0,
            });
        }

        // ShaderBox 画廊(Plan 16 调试):视口左上钉一格栅,逐格一个内置 shader → 一屏验全盘上屏。
        // 屏锚(锚 `visible` 左上 → 随相机平移固定屏上)。50 icon + glow_orb + raymarch。
        if self.shaderbox_gallery {
            self.push_shaderbox_gallery(&visible, &mut shaderboxes, &mut shaderbox_pixels);
        }

        // Plan 18 §2.1 规模度量:驻留量(含离屏/已冻块,不止可见)= 0029 before/after 主指标。
        // `Vec::len()` O(1) → 整体 O(views) 每帧,廉价(不扫文本/几何内容)。
        let (mut retained_glyphs, mut retained_nodes) = (0usize, 0usize);
        let mut tier_counts = [0usize; 2]; // [Hot, Warm]
        for v in &self.views {
            match v.tier {
                Tier::Hot => tier_counts[0] += 1,
                Tier::Warm => tier_counts[1] += 1,
            }
            if let Some(c) = &v.cache {
                retained_glyphs += c.placed.len();
                retained_nodes += c.nodes.len();
            }
        }
        self.last_stats = FrameStats {
            frame_glyphs: glyphs.len(),
            total_glyphs,
            visible_blocks,
            total_blocks: drawable.len(),
            shaderbox_active: shaderboxes.len(),
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
            rects,
            panels,
            images,
            embeds: frame_embeds,
            widgets,
            shaderboxes,
            glyphs,
            time_ms: self.now_ms as f32,
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
        const PROMOTE_MARGIN: f32 = 1.5; // 视口外 1.5 屏内 → 保 Hot / 提前重建(留滚动余量)
        const RELEASE_MARGIN: f32 = 3.0; // 视口外 3 屏外 → 释放(> promote 1.5 屏滞回带,防 thrash)
        let vh = visible.h.max(1.0);
        let (vtop, vbot) = (visible.y, visible.y + visible.h);
        let promote_lo = vtop - vh * PROMOTE_MARGIN;
        let promote_hi = vbot + vh * PROMOTE_MARGIN;
        let release_lo = vtop - vh * RELEASE_MARGIN;
        let release_hi = vbot + vh * RELEASE_MARGIN;
        for &(i, origin, _w, h) in drawable {
            let (top, bot) = (origin[1], origin[1] + h);
            let within_promote = bot >= promote_lo && top <= promote_hi;
            let beyond_release = bot < release_lo || top > release_hi;
            let v = &mut self.views[i];
            match v.tier {
                Tier::Warm if within_promote => v.tier = Tier::Hot, // 下帧重建
                Tier::Hot if beyond_release && v.settled => {
                    v.tier = Tier::Warm;
                    v.cache = None; // 释放重几何;agg/revealed/spawn 保留 → 重入瞬显、布局不动
                }
                _ => {}
            }
        }
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
        });
        self.views.last_mut().expect("just pushed") // reason: 上面刚 push
    }
}

#[cfg(test)]
mod tests {
    use super::{group_turns, PartView, Tier};
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
        assert!(text.contains("bash"), "工具名应可见: {text}");
        assert!(text.contains("cmd"), "工具载荷(input)应可见: {text}");
        assert!(text.contains("Thinking"), "推理卡标题应可见: {text}");
        assert!(text.contains("先看要点"), "推理正文应可见: {text}");
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
        assert!(text.contains("[done]"), "应显示最终状态徽章: {text}");
        assert!(
            !text.contains("[pending]"),
            "旧状态不应残留(已重置): {text}"
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
        eng.note_reply();
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
            eng.sink().visible_text().contains("bash"),
            "注入的工具 part 应渲染: {}",
            eng.sink().visible_text()
        );
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
                .any(|r| close(r.color, crate::theme::CODE_BG)),
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
                        .any(|r| close(r.color, crate::theme::CODE_CHIP)),
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
                .any(|r| close(r.color, crate::theme::CODE_CHIP)),
            "码全揭示后应有 chip"
        );
        assert!(
            f.rects.iter().any(|r| close(r.color, crate::theme::STRIKE)),
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
        let warn = crate::theme::alert_bar("WARNING");
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
        // Plan 11:分隔线迁为 markdown widget(中间亮两端淡出渐变线),整宽 quad。
        assert!(
            f.widgets
                .iter()
                .any(|w| w.component == crate::frame::WIDGET_RULE_CAT && w.size[0] > 700.0),
            "分隔线应是整宽喵喵 rule widget"
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
    fn agent_glow_orb_logo_on_assistant_box() {
        // Plan 16 §2.6:assistant 盒左侧出一个 GlowOrb dynamic 头像(身份 logo)。
        let mut eng = Engine::new(
            Player::from_pairs(vec![], 16.0),
            MonospaceLayout::default(),
            CollectSink::default(),
            200.0,
            800.0,
        );
        // 给 assistant 盒留出左侧画布(pan 到负 x → 头像位 origin-40 进可见区)。
        prime_code(&mut eng, 3);
        eng.pan_by(-80.0, 0.0);
        eng.frame(16.0);
        let f = eng.sink().last().expect("frame");
        let orb = f
            .shaderboxes
            .iter()
            .find(|sb| sb.shader_id == crate::ShaderId::GlowOrb.as_u32())
            .expect("应有 agent glow-orb 头像");
        assert!(orb.dynamic, "glow-orb 永远 dynamic(呼吸)");
        assert!((orb.size[0] - 32.0).abs() < 0.01, "头像 32px");
        assert!(orb.params[3] > 0.0, "p0.w 脉冲速度 > 0");
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
        // 68 格全发(空会话也出 → 屏锚视口,与内容解耦)。66 icon + glow_orb + raymarch。
        let tiles = crate::ICON_COUNT as usize + 2;
        assert_eq!(f.shaderboxes.len(), tiles, "66 icon + glow_orb + raymarch");
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
        let snap = r#"[{"info":{"id":"m1","sessionID":"s","role":"a"},
            "parts":[{"type":"text","id":"p1","messageID":"m1","text":"![dog](bad)"}]}]"#;
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
        eng.set_max_width(1000.0); // 文档变宽 200 → user 盒右对齐右移 200
        eng.frame(16.0);
        let r1000 = right_edge(eng.sink().last().expect("frame"));
        assert!(
            (r1000 - r800 - 200.0).abs() < 5.0,
            "user 盒右沿应随文档宽右移 ~200(origin delta 进端点): {r800} → {r1000}"
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
        let sel = crate::theme::SELECTION;
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
    #[ignore = "规模基线采集器(显式跑;打印 CSV)"]
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
