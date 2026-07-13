//! svg(Plan 31 R4 / 0036)— 流式 SVG **矢量原生** v1:白名单图元解析 → 引擎 SDF/quad 图元。
//!
//! 与 0035(math display list)同范式:每个图元都是与正文同管线的实例 → 任意缩放锐利、
//! 逐元素流式揭示(plan30 骨架门)、可进 0016 补间。白名单外(path/斜线/渐变/transform)
//! **整图纹理降级**(plan14 位图链),不 panic(AR12),降级数入 FrameStats。
//!
//! 解析器为**极简手写**(v1 白名单本就窄):不引 XML 依赖(CR1 纯逻辑,确定性 R8);
//! 属性乱序/自闭合/引号单双均容;数值解析失败 → 该元素计降级。

/// 一个矢量矩形图元(含"圆"——radius = w/2 即圆,rect.wgsl 圆角全开)。
#[derive(Debug, Clone, PartialEq)]
pub struct SvgRect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub radius: f32,
    pub color: [f32; 4],
}

/// 一段文本图元(走正文 glyph 链)。
#[derive(Debug, Clone, PartialEq)]
pub struct SvgText {
    pub x: f32,
    pub y: f32, // 基线 y(SVG 语义)
    pub text: String,
    pub size: f32,
    pub color: [f32; 4],
}

/// 解析产物:viewBox 归一后的图元列表(单位 = viewBox 坐标;上屏时按目标宽等比缩放)。
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SvgLayout {
    pub view_w: f32,
    pub view_h: f32,
    pub rects: Vec<SvgRect>,
    pub texts: Vec<SvgText>,
    /// 白名单外元素数(降级计数;>0 且 rects/texts 为空 → 调用方走整图纹理)。
    pub degraded: usize,
}

/// named/hex 颜色 → RGBA(未知 → 中性灰,AR12 不丢元素)。
fn parse_color(v: &str) -> [f32; 4] {
    let named = |r: u8, g: u8, b: u8| {
        [
            f32::from(r) / 255.0,
            f32::from(g) / 255.0,
            f32::from(b) / 255.0,
            1.0,
        ]
    };
    let v = v.trim();
    if let Some(hex) = v.strip_prefix('#') {
        let h = |i: usize| u8::from_str_radix(hex.get(i..i + 2).unwrap_or("00"), 16).unwrap_or(0);
        if hex.len() >= 6 {
            return named(h(0), h(2), h(4));
        }
        if hex.len() == 3 {
            let d = |i: usize| {
                let c = u8::from_str_radix(hex.get(i..=i).unwrap_or("0"), 16).unwrap_or(0);
                c * 17
            };
            return named(d(0), d(1), d(2));
        }
    }
    match v {
        "black" => named(0, 0, 0),
        "white" => named(255, 255, 255),
        "red" => named(255, 0, 0),
        "green" => named(0, 128, 0),
        "blue" => named(0, 0, 255),
        "tomato" => named(255, 99, 71),
        "orange" => named(255, 165, 0),
        "gray" | "grey" => named(128, 128, 128),
        "none" | "transparent" => [0.0, 0.0, 0.0, 0.0],
        _ => [0.6, 0.6, 0.6, 1.0], // 未知色兜中性(0036 §2)
    }
}

/// 从元素标签文本抽属性值(`name="v"` / `name='v'`)。
fn attr<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let pat = format!("{name}=");
    let mut from = 0;
    while let Some(rel) = tag[from..].find(&pat) {
        let i = from + rel;
        // 词边界:前一字符须为空白(防 stroke-width 命中 width)。
        if i > 0 && !tag.as_bytes()[i - 1].is_ascii_whitespace() {
            from = i + pat.len();
            continue;
        }
        let rest = &tag[i + pat.len()..];
        let quote = rest.chars().next()?;
        if quote != '"' && quote != '\'' {
            return None;
        }
        let rest = &rest[1..];
        let end = rest.find(quote)?;
        return Some(&rest[..end]);
    }
    None
}

fn num(tag: &str, name: &str) -> Option<f32> {
    attr(tag, name).and_then(|v| v.trim().parse::<f32>().ok())
}

/// 解析 SVG 源(0036 白名单)。纯函数、确定性;任何异常形态 → 计降级,不 panic。
#[must_use]
pub fn parse_svg(src: &str) -> SvgLayout {
    let mut out = SvgLayout::default();
    // viewBox / width/height(缺省 100×100)。
    if let Some(svg_start) = src.find("<svg") {
        let tag_end = src[svg_start..]
            .find('>')
            .map_or(src.len(), |e| svg_start + e);
        let tag = &src[svg_start..tag_end];
        if let Some(vb) = attr(tag, "viewBox") {
            let p: Vec<f32> = vb
                .split_whitespace()
                .filter_map(|x| x.parse::<f32>().ok())
                .collect();
            if p.len() == 4 {
                out.view_w = p[2];
                out.view_h = p[3];
            }
        }
        if out.view_w <= 0.0 {
            out.view_w = num(tag, "width").unwrap_or(100.0);
            out.view_h = num(tag, "height").unwrap_or(100.0);
        }
    } else {
        out.view_w = 100.0;
        out.view_h = 100.0;
    }

    // 逐元素扫描(白名单;不解析嵌套结构 —— v1 无 group transform,遇 <g transform> 记降级)。
    let mut i = 0;
    while let Some(rel) = src[i..].find('<') {
        let start = i + rel;
        let end = src[start..].find('>').map_or(src.len(), |e| start + e + 1);
        let tag = &src[start..end];
        i = end;
        let name = tag
            .trim_start_matches('<')
            .split(|c: char| c.is_ascii_whitespace() || c == '>' || c == '/')
            .next()
            .unwrap_or("");
        let fill = attr(tag, "fill").map_or([0.6, 0.6, 0.6, 1.0], parse_color);
        match name {
            "svg" | "/svg" | "" => {}
            "rect" => {
                let (Some(w), Some(h)) = (num(tag, "width"), num(tag, "height")) else {
                    out.degraded += 1;
                    continue;
                };
                out.rects.push(SvgRect {
                    x: num(tag, "x").unwrap_or(0.0),
                    y: num(tag, "y").unwrap_or(0.0),
                    w,
                    h,
                    radius: num(tag, "rx").unwrap_or(0.0),
                    color: fill,
                });
            }
            "circle" => {
                let (Some(cx), Some(cy), Some(r)) = (num(tag, "cx"), num(tag, "cy"), num(tag, "r"))
                else {
                    out.degraded += 1;
                    continue;
                };
                out.rects.push(SvgRect {
                    x: cx - r,
                    y: cy - r,
                    w: r * 2.0,
                    h: r * 2.0,
                    radius: r, // 圆角全开 = 圆(0036 §2)
                    color: fill,
                });
            }
            "line" => {
                let (x1, y1, x2, y2) = (
                    num(tag, "x1").unwrap_or(0.0),
                    num(tag, "y1").unwrap_or(0.0),
                    num(tag, "x2").unwrap_or(0.0),
                    num(tag, "y2").unwrap_or(0.0),
                );
                let w = num(tag, "stroke-width").unwrap_or(1.0);
                let color = attr(tag, "stroke").map_or([0.6, 0.6, 0.6, 1.0], parse_color);
                if (y1 - y2).abs() < f32::EPSILON {
                    out.rects.push(SvgRect {
                        x: x1.min(x2),
                        y: y1 - w / 2.0,
                        w: (x2 - x1).abs(),
                        h: w,
                        radius: 0.0,
                        color,
                    });
                } else if (x1 - x2).abs() < f32::EPSILON {
                    out.rects.push(SvgRect {
                        x: x1 - w / 2.0,
                        y: y1.min(y2),
                        w,
                        h: (y2 - y1).abs(),
                        radius: 0.0,
                        color,
                    });
                } else {
                    out.degraded += 1; // 斜线:v1 无旋转通道(0036 §2)
                }
            }
            "text" => {
                // 文本内容 = 标签闭合到 </text> 之间的字面。
                let body_end = src[i..].find("</text>").map_or(src.len(), |e| i + e);
                let body = src[i..body_end].trim().to_owned();
                i = body_end;
                out.texts.push(SvgText {
                    x: num(tag, "x").unwrap_or(0.0),
                    y: num(tag, "y").unwrap_or(0.0),
                    text: body,
                    size: num(tag, "font-size").unwrap_or(16.0),
                    color: fill,
                });
            }
            _ if name.starts_with('/') || name.starts_with('!') || name.starts_with('?') => {}
            _ => out.degraded += 1, // path/polyline/g/渐变…:白名单外(0036 §2 降级)
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r##"<svg viewBox="0 0 200 100" xmlns="http://www.w3.org/2000/svg">
      <rect x="10" y="10" width="60" height="40" rx="6" fill="#ff6347"/>
      <circle cx="120" cy="30" r="20" fill="blue"/>
      <line x1="10" y1="70" x2="190" y2="70" stroke="#282828" stroke-width="2"/>
      <text x="10" y="95" font-size="12" fill="white">hello svg</text>
    </svg>"##;

    #[test]
    fn whitelist_parse_snapshot() {
        // Plan 31 R4:解析快照(0036 白名单;回归即红)。
        let l = parse_svg(SAMPLE);
        insta::assert_debug_snapshot!("svg_whitelist_parse", l);
    }

    #[test]
    fn circle_is_full_radius_rect() {
        let l =
            parse_svg(r#"<svg viewBox="0 0 10 10"><circle cx="5" cy="5" r="4" fill="red"/></svg>"#);
        assert_eq!(l.rects.len(), 1);
        let c = &l.rects[0];
        assert_eq!((c.x, c.y, c.w, c.h, c.radius), (1.0, 1.0, 8.0, 8.0, 4.0));
    }

    #[test]
    fn out_of_whitelist_degrades_not_panics() {
        // path/斜线/渐变 → 计降级(AR12);已解析元素不丢。
        let l = parse_svg(
            r#"<svg viewBox="0 0 10 10"><path d="M0 0L10 10"/><line x1="0" y1="0" x2="10" y2="10" stroke="red"/><rect width="4" height="4"/><linearGradient id="g"/></svg>"#,
        );
        assert_eq!(l.rects.len(), 1, "rect 仍解析");
        assert_eq!(l.degraded, 3, "path + 斜线 + 渐变 各计一次");
    }

    #[test]
    fn deterministic_same_input_same_output() {
        assert_eq!(parse_svg(SAMPLE), parse_svg(SAMPLE)); // R8
    }

    #[test]
    fn malformed_attrs_dont_panic() {
        let l =
            parse_svg(r#"<svg><rect width="oops" height="4"/><circle cx="1"/><text x="1" y="2">t"#);
        assert_eq!(l.degraded, 2);
        assert_eq!(l.texts.len(), 1, "未闭合 text 容错取尾");
    }
}
