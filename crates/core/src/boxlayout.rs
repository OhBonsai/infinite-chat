//! boxlayout(M13 / Plan 13 / 0023)— chat 级 **Taffy 盒子布局**:角色左右分栏 + 内容竖直堆叠。
//!
//! 把 `build_frame` 的「手搓 `top += height`」升级为 **Flexbox 盒树**(0023):每 view 一个叶子盒,
//! 按 **0005 角色**放进 UserBox(`align_self:FlexEnd` 右)/ AsstBox(`FlexStart` 左);assistant **一回合
//! 一个 AsstBox**(回合内多 part 作盒内块,守 §2.1)。输出 = 每 view 的**世界绝对 origin**(top-left),
//! build_frame 据此整体平移该 view 的字/框(view 内相对位不变 → 0016 morph 身份稳定)。
//!
//! 纯 Rust(taffy,无 wasm-bindgen/web-sys/wgpu),native 可测(CR1)。Tier A(本文件)只做 chat 级
//! 分栏 + per-part 叶子;Tier B/C(块内嵌套、measure 回调)= Plan 13 ③④。

use crate::motion::MotionTokens;
use taffy::prelude::*;
use taffy::{NodeId, Style, TaffyTree};

/// assistant 内容列最大宽(px;design §2.1 内容列 ≈768,两侧留白靠居中产生)。
const CONTENT_MAX: f32 = 800.0; // Plan 28 R1:参考 md:max-w-200 = 800px(message-timeline.tsx:1096)
/// user 气泡最大宽 = 内容列宽 × 此比例(design §4.1 max-width ~85% 列宽)。
const BUBBLE_RATIO: f32 = 0.82; // Plan 28 R1:参考 user 气泡 max-width:min(82%,64ch)

/// 排版折行宽(Plan 25):文字必须按**列宽**折行(assistant = 内容列,user = 气泡宽),与
/// [`layout_chat`] 的盒宽同源——否则排版按整文档宽折行,长段落溢出居中列(r1 自评硬伤)。
/// `dpr`:world unit = **设备像素**(HiDPI 排版按设备像素,layout-bridge)→ 列上限须 ×dpr,
/// 否则 retina 上 760 设备 px ≈ 380 CSS px,列宽被锁死(用户实测);native/无头 dpr=1 不变。
pub(crate) fn wrap_width(viewport_w: f32, user: bool, dpr: f32) -> f32 {
    let col = (CONTENT_MAX * dpr.max(0.5)).min(viewport_w.max(1.0));
    if user {
        col * BUBBLE_RATIO
    } else {
        col
    }
}

/// 一个回合的角色分组(0005 投影,纯计算;Plan 13 §4.3)。`user` = 该回合 user part 的 view 下标
/// (可空);`assistant` = 该回合**连续** assistant part 的 view 下标(跨 message,守「一回合一盒」)。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct TurnGroup {
    pub(crate) user: Option<usize>,
    pub(crate) assistant: Vec<usize>,
}

/// 列容器样式(flex column + 行距 + 可选左右对齐 + 可选最大宽)。
fn col(gap: f32, align: Option<AlignItems>, max_w: Option<f32>) -> Style {
    Style {
        display: Display::Flex,
        flex_direction: FlexDirection::Column,
        gap: Size {
            width: length(0.0),
            height: length(gap),
        },
        align_self: align,
        max_size: Size {
            width: max_w.map_or_else(auto, length),
            height: auto(),
        },
        ..Default::default()
    }
}

/// 一个 view 盒在世界里的位置(Plan 13 §4):`origin` = 左上角 world 坐标;`width` = 盒宽(taffy
/// 算出,= 内容宽夹到 max)。build_frame 据此整体平移该 view 的字 + 据 `width` 摆全宽装饰/裁剪。
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct BoxPos {
    pub(crate) origin: [f32; 2],
    pub(crate) width: f32,
}

/// **Tier A 盒树布局**(Plan 13 §4):turns(角色分组)+ 各 view 叶子尺寸 `sizes[vi]=(w,h)` →
/// ChatRoot ▸ Turn ▸ UserBox(右)/AsstBox(左)▸ per-part 叶子 → 算出**每 view 的 [`BoxPos`]**。
/// `viewport_w` = 文档宽(左右对齐锚它,§4.5)。任意失败/越界 → 该 view 退 `origin[0,0]`(防御)。
/// 间距/留白走 [`MotionTokens`](Plan 25 P0):turn 间 `space_turn`、part 间 `space_part`、
/// 顶部留白 `space_turn/2`;内容列夹 [`CONTENT_MAX`] 后**水平居中**(design §2.1)。
pub(crate) fn layout_chat(
    turns: &[TurnGroup],
    sizes: &[(f32, f32)],
    viewport_w: f32,
    m: &MotionTokens,
    dpr: f32,
    // Plan 46 DoD-6:tui 动态兄弟间距。`tui=false`(rich/默认)→ 走原**均匀 gap** 路径,逐字节恒等
    // (rich 恒等铁律)。`tui=true` → assistant 盒 gap=0,逐子 margin-top:连续 inline tool 贴合(0),
    // 否则 `space_part`(= 原间距)。`inline_tool[view_i]` = 该 view 是否 tui InlineTool 形态(core 据
    // partspecific::tui_tool_is_inline 预分类);空/越界按 false。跨类型/非工具 → 保持 space_part。
    tui: bool,
    inline_tool: &[bool],
) -> Vec<BoxPos> {
    let n = sizes.len();
    let mut tree: TaffyTree<()> = TaffyTree::new();
    // 关掉 taffy 的整数像素取整:保子像素位,**逐字等于**旧 `top += height` 的小数前缀和(Plan 13③
    // ±0 回归)+ 不引入换行抖动;字形最终走 SDF 子像素渲染,无需 layout 期取整。
    tree.disable_rounding();
    let mut leaf_of: Vec<Option<NodeId>> = vec![None; n];
    let vw = viewport_w.max(1.0);

    // assistant 内容列宽(固定):内容块(分隔线/代码底/表格/Alert 底)铺满此列,而非各自收缩
    // 到本行文字宽——否则 `---` 这种零墨块会塌成 0 宽(§2.2 盒内异构块共用一列)。夹到视口。
    let content_col = (CONTENT_MAX * dpr.max(0.5)).min(vw); // 列上限 ×dpr(见 wrap_width 注)
    let bubble_max = content_col * BUBBLE_RATIO;
    // Plan 28 R4+:间距 token 同样 ×dpr —— token 值 = 参考的 CSS px(session-turn.css gap),
    // 画布是设备 px;不乘则 retina 上目视间距减半(作者实测「全黏死」的根因)。
    let d = dpr.max(0.5);
    let m = &crate::motion::MotionTokens {
        space_turn: m.space_turn * d,
        space_part: m.space_part * d,
        space_inset: m.space_inset * d,
        ..m.clone()
    };

    // 建一个 view 叶子。`fixed_w=Some(w)` → 显式宽(user 气泡,收缩夹 max);`None` → auto 宽
    // (assistant,靠父 `align_items:Stretch` 铺满内容列)。高恒为内容高。
    let mut make_leaf =
        |tree: &mut TaffyTree<()>, vi: usize, fixed_w: Option<f32>| -> Option<NodeId> {
            let (_, h) = *sizes.get(vi)?;
            let node = tree
                .new_leaf(Style {
                    size: Size {
                        width: fixed_w.map_or_else(auto, length),
                        height: length(h.max(0.0)),
                    },
                    ..Default::default()
                })
                .ok()?;
            if let Some(slot) = leaf_of.get_mut(vi) {
                *slot = Some(node);
            }
            Some(node)
        };

    let mut turn_nodes: Vec<NodeId> = Vec::new();
    for t in turns {
        let mut turn_children: Vec<NodeId> = Vec::new();
        // UserBox(右):一个 user part 叶子,收缩到文字宽(夹 bubble_max),气泡右对齐。
        if let Some(ui) = t.user {
            let (w, _) = sizes.get(ui).copied().unwrap_or((0.0, 0.0));
            if let Some(l) = make_leaf(&mut tree, ui, Some(w.clamp(0.0, bubble_max))) {
                if let Ok(b) = tree
                    .new_with_children(col(0.0, Some(AlignItems::FlexEnd), Some(bubble_max)), &[l])
                {
                    turn_children.push(b);
                }
            }
        }
        // AsstBox(左):该回合所有 assistant part 叶子(一个盒,守「一回合一盒」),固定内容列宽,
        // 子块 `align_items:Stretch`(taffy 默认)铺满该列。
        let mut ach: Vec<NodeId> = Vec::new();
        let mut prev_ai: Option<usize> = None;
        for &ai in &t.assistant {
            if let Some(l) = make_leaf(&mut tree, ai, None) {
                if tui {
                    // 逐子 margin-top:首子 0(盒顶齐);连续 inline tool 贴合 0;否则 space_part。
                    let both_inline = prev_ai.is_some_and(|p| {
                        inline_tool.get(ai).copied().unwrap_or(false)
                            && inline_tool.get(p).copied().unwrap_or(false)
                    });
                    let mt = if prev_ai.is_none() || both_inline {
                        0.0
                    } else {
                        m.space_part
                    };
                    if let Ok(cur) = tree.style(l).cloned() {
                        let styled = Style {
                            margin: taffy::geometry::Rect {
                                left: zero(),
                                right: zero(),
                                top: length(mt),
                                bottom: zero(),
                            },
                            ..cur
                        };
                        let _ = tree.set_style(l, styled);
                    }
                }
                ach.push(l);
                prev_ai = Some(ai);
            }
        }
        if !ach.is_empty() {
            // tui:盒 gap=0(间距全由逐子 margin 表达);rich:原均匀 space_part gap(恒等)。
            let asst_gap = if tui { 0.0 } else { m.space_part };
            let asst_style = Style {
                size: Size {
                    width: length(content_col),
                    height: auto(),
                },
                ..col(asst_gap, Some(AlignItems::FlexStart), Some(content_col))
            };
            if let Ok(b) = tree.new_with_children(asst_style, &ach) {
                turn_children.push(b);
            }
        }
        // Turn 盒固定 = 内容列宽 → 根 align_items:Center 把整列水平居中(user-only 回合也不漂)。
        let turn_style = Style {
            size: Size {
                width: length(content_col),
                height: auto(),
            },
            ..col(m.space_part, None, None)
        };
        if let Ok(turn) = tree.new_with_children(turn_style, &turn_children) {
            turn_nodes.push(turn);
        }
    }

    // 根:内容列居中(两侧留白 = (vw-列宽)/2,design §2.1)+ 顶/底留白 space_turn/2
    // (首 user 弱底面板的 outset 不被 y=0 裁掉)。
    let pad = length(m.space_turn * 0.5);
    let Ok(root) = tree.new_with_children(
        Style {
            size: Size {
                width: length(vw),
                height: auto(),
            },
            align_items: Some(AlignItems::Center),
            padding: taffy::geometry::Rect {
                left: zero(),
                right: zero(),
                top: pad,
                bottom: pad,
            },
            ..col(m.space_turn, None, None)
        },
        &turn_nodes,
    ) else {
        return vec![BoxPos::default(); n];
    };
    if tree
        .compute_layout(
            root,
            Size {
                width: AvailableSpace::Definite(vw),
                height: AvailableSpace::MaxContent,
            },
        )
        .is_err()
    {
        return vec![BoxPos::default(); n];
    }

    // DFS 累加绝对 origin(taffy `Layout.location` 相对父);叶子节点 → 对应 view 的 origin + width。
    let mut out = vec![BoxPos::default(); n];
    let mut stack: Vec<(NodeId, [f32; 2])> = vec![(root, [0.0, 0.0])];
    while let Some((node, base)) = stack.pop() {
        let (lx, ly, w) = tree.layout(node).map_or((0.0, 0.0, 0.0), |l| {
            (l.location.x, l.location.y, l.size.width)
        });
        let abs = [base[0] + lx, base[1] + ly];
        for (vi, slot) in leaf_of.iter().enumerate() {
            if *slot == Some(node) {
                out[vi] = BoxPos {
                    origin: abs,
                    width: w,
                };
            }
        }
        if let Ok(children) = tree.children(node) {
            for c in children {
                stack.push((c, abs));
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 默认令牌(design §3.1)。viewport 1000 → 内容列 760 居中,左缘 120、右缘 880。
    fn m() -> MotionTokens {
        MotionTokens::default()
    }

    #[test]
    fn user_right_assistant_left() {
        // 一回合:user(右)+ assistant(左)。viewport 1000;user/asst 各宽 200/300。
        let turns = vec![TurnGroup {
            user: Some(0),
            assistant: vec![1],
        }];
        let sizes = vec![(200.0, 30.0), (300.0, 50.0)];
        let o = layout_chat(&turns, &sizes, 1000.0, &m(), 1.0, false, &[]);
        let col_left = (1000.0 - CONTENT_MAX) / 2.0; // 120
        let col_right = col_left + CONTENT_MAX; // 880
                                                // user 右对齐列右缘:x ≈ 880 - 200 = 680。assistant 左对齐列左缘:x ≈ 120。
        assert!(
            (o[0].origin[0] - (col_right - 200.0)).abs() < 2.0,
            "user 应右对齐列右缘: {}",
            o[0].origin[0]
        );
        assert!(
            (o[1].origin[0] - col_left).abs() < 2.0,
            "assistant 应左对齐列左缘: {}",
            o[1].origin[0]
        );
        // assistant 在 user 下方(回合内 user 先 / asst 后,竖直堆叠)。
        assert!(
            o[1].origin[1] > o[0].origin[1],
            "asst 在 user 下: {} vs {}",
            o[1].origin[1],
            o[0].origin[1]
        );
    }

    #[test]
    fn user_bubble_clamped_to_max_width() {
        // 超长 user 文本(宽 2000)→ 盒宽夹到 85% 列宽(646),右对齐列右缘。
        let turns = vec![TurnGroup {
            user: Some(0),
            assistant: vec![],
        }];
        let sizes = vec![(2000.0, 40.0)];
        let o = layout_chat(&turns, &sizes, 1000.0, &m(), 1.0, false, &[]);
        let bubble_max = CONTENT_MAX * BUBBLE_RATIO;
        let col_right = (1000.0 - CONTENT_MAX) / 2.0 + CONTENT_MAX;
        assert!(
            (o[0].origin[0] - (col_right - bubble_max)).abs() < 2.0,
            "user 盒宽应夹到 {bubble_max},右对齐 x≈{}: 实 {}",
            col_right - bubble_max,
            o[0].origin[0]
        );
    }

    // Plan 46 DoD-6:tui 动态兄弟间距。一回合三 assistant part(tool, tool, text):tui 下前两个
    // (连续 inline tool)贴合 0 gap,第三个(跨类型)留 space_part;rich 全 space_part(恒等)。
    #[test]
    fn tui_consecutive_inline_tools_stack_tight() {
        let turns = vec![TurnGroup {
            user: None,
            assistant: vec![0, 1, 2],
        }];
        let sizes = vec![(200.0, 30.0), (200.0, 30.0), (300.0, 40.0)];
        let inline = [true, true, false];
        let sp = m().space_part;
        // tui:连续 inline tool 贴合。
        let o = layout_chat(&turns, &sizes, 1000.0, &m(), 1.0, true, &inline);
        assert!(
            (o[1].origin[1] - o[0].origin[1] - 30.0).abs() < 0.01,
            "连续 inline tool 应贴合(0 gap):Δ={}",
            o[1].origin[1] - o[0].origin[1]
        );
        assert!(
            ((o[2].origin[1] - (o[1].origin[1] + 30.0)) - sp).abs() < 0.01,
            "跨类型应留 space_part:Δ={}",
            o[2].origin[1] - (o[1].origin[1] + 30.0)
        );
        // rich 恒等:全 space_part(空 inline slice + tui=false)。
        let r = layout_chat(&turns, &sizes, 1000.0, &m(), 1.0, false, &[]);
        assert!(
            ((r[1].origin[1] - r[0].origin[1]) - (30.0 + sp)).abs() < 0.01,
            "rich 应均匀 space_part:Δ={}",
            r[1].origin[1] - r[0].origin[1]
        );
    }

    #[test]
    fn turns_stack_vertically_with_gap() {
        // 两回合竖直堆叠:回合2 在回合1 下方,间距 = space_turn(分组靠间距,design §2.1)。
        let turns = vec![
            TurnGroup {
                user: Some(0),
                assistant: vec![1],
            },
            TurnGroup {
                user: Some(2),
                assistant: vec![3],
            },
        ];
        let sizes = vec![(200.0, 30.0), (300.0, 50.0), (200.0, 30.0), (300.0, 50.0)];
        let o = layout_chat(&turns, &sizes, 1000.0, &m(), 1.0, false, &[]);
        // 回合2 的 user(view 2)与回合1 的 assistant(view 1)底的间距 ≥ space_turn。
        let turn1_bottom = o[1].origin[1] + 50.0;
        assert!(
            o[2].origin[1] - turn1_bottom >= m().space_turn - 0.01,
            "回合间距应 ≥ space_turn: {} vs {}",
            o[2].origin[1],
            turn1_bottom
        );
    }

    #[test]
    fn assistant_parts_share_one_box_stacked() {
        // 一回合多 assistant part(一个 AsstBox):竖直堆叠、都左对齐列左缘。
        let turns = vec![TurnGroup {
            user: None,
            assistant: vec![0, 1],
        }];
        let sizes = vec![(300.0, 30.0), (300.0, 40.0)];
        let o = layout_chat(&turns, &sizes, 1000.0, &m(), 1.0, false, &[]);
        let col_left = (1000.0 - CONTENT_MAX) / 2.0;
        assert!(
            (o[0].origin[0] - col_left).abs() < 2.0 && (o[1].origin[0] - col_left).abs() < 2.0,
            "都左对齐列左缘"
        );
        assert!(
            o[1].origin[1] > o[0].origin[1],
            "part2 在 part1 下(同盒堆叠)"
        );
    }

    #[test]
    fn legacy_stacking_equivalence_pm0() {
        // Plan 13③ 回归(±0):纯 assistant 单列(关掉角色分栏的退化形)下,各 part 的 y 位必须**逐字
        // 等于**旧 `top += height + BLOCK_GAP` 的前缀和——证 Tier A 收编手搓堆叠**无几何漂移**。
        // 令牌置 space_part=8(旧 BLOCK_GAP)、space_turn=0(顶部留白 0)→ 纯等价退化形。
        const LEGACY_GAP: f32 = 8.0;
        let toks = MotionTokens {
            space_part: LEGACY_GAP,
            space_turn: 0.0,
            ..MotionTokens::default()
        };
        let heights = [30.0f32, 52.5, 18.0, 44.0];
        let turns = vec![TurnGroup {
            user: None,
            assistant: (0..heights.len()).collect(),
        }];
        let sizes: Vec<(f32, f32)> = heights.iter().map(|&h| (300.0, h)).collect();
        let o = layout_chat(&turns, &sizes, 1000.0, &toks, 1.0, false, &[]);
        let mut legacy_top = 0.0f32;
        for (i, &h) in heights.iter().enumerate() {
            assert!(
                (o[i].origin[1] - legacy_top).abs() < 0.01,
                "part {i} y 位漂移: taffy {} vs legacy {}",
                o[i].origin[1],
                legacy_top
            );
            legacy_top += h + LEGACY_GAP;
        }
        // 锚底回归:末盒 computed bottom == 末 part origin.y + 末 part 高(= ChatRoot 内容底)。
        let last = heights.len() - 1;
        let computed_bottom = o[last].origin[1] + heights[last];
        let legacy_bottom = legacy_top - LEGACY_GAP; // 减去末尾多加的一截 gap
        assert!(
            (computed_bottom - legacy_bottom).abs() < 0.01,
            "末盒 bottom 漂移: {computed_bottom} vs {legacy_bottom}"
        );
    }

    #[test]
    fn content_column_centered_with_margins() {
        // design §2.1:宽视口下内容列居中、两侧留白 ≥48;顶部留白 = space_turn/2。
        let turns = vec![TurnGroup {
            user: None,
            assistant: vec![0],
        }];
        let sizes = vec![(300.0, 30.0)];
        let o = layout_chat(&turns, &sizes, 1000.0, &m(), 1.0, false, &[]);
        let margin = (1000.0 - CONTENT_MAX) / 2.0;
        assert!(margin >= 48.0, "留白应 ≥48: {margin}");
        assert!(
            (o[0].origin[0] - margin).abs() < 2.0,
            "内容列应居中: x={} margin={margin}",
            o[0].origin[0]
        );
        assert!(
            (o[0].origin[1] - m().space_turn * 0.5).abs() < 0.01,
            "顶部留白应 = space_turn/2: {}",
            o[0].origin[1]
        );
    }
}
