//! 效果预设库(0041,Plan 38):场景槽 × 预设的纯数据表(serde;JSON 可加载)。
//!
//! 36 造词汇、37 开通道、本模块编字典:`subtle` = 现观感的显式化(默认观感单一真值源),
//! `off` = 全恒等,`expressive` = 36/37 词汇的保守组合。profile(full/reduced/off)与
//! preset 正交。坏 JSON 整份拒绝(AR12,调用方回退当前档)。

use serde::Deserialize;

/// 进场:曲线选择 + 时长因子(乘 MotionTokens.dur_glyph)。
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(default)]
pub struct EnterSpec {
    /// true = Spring 曲线(N4 profile 5);false = 既定角色 profile。
    pub spring: bool,
    /// 时长因子(1 = token 原值)。
    pub dur_factor: f32,
}

/// 退场:dissolve 时长 ms(0 = 即时清除,旧行为)。
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(default)]
pub struct ExitSpec {
    pub dissolve_ms: f32,
}

/// 强调:hit-flash 混色强度与时长(触发器由调用方;0 = 无)。
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(default)]
pub struct EmphasisSpec {
    pub flash_alpha: f32,
    pub flash_ms: f32,
}

/// 待机:呼吸脉冲(plan25 M2a 光标/指示条)幅度与频率(0 幅度 = 无)。
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(default)]
pub struct IdleSpec {
    pub pulse_amp: f32,
    pub pulse_hz: f32,
}

/// 思考中:工具标题 shimmer 开关 + 微拖尾 decay(0 = 无拖尾)。
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(default)]
pub struct ThinkingSpec {
    pub shimmer: bool,
    pub trail_decay: f32,
}

/// 悬停:链接下划线 alpha + 卡片提亮强度。
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(default)]
pub struct HoverSpec {
    pub underline_alpha: f32,
    pub card_lift: f32,
}

/// 庆祝:v1 无实现(粒子层远期);字段留位。
#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Default)]
#[serde(default)]
pub struct CelebrateSpec {
    pub enabled: bool,
}

/// 一份预设 = 七槽参数集(0041 §2)。
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(default)]
pub struct EffectPreset {
    pub name: String,
    pub enter: EnterSpec,
    pub exit: ExitSpec,
    pub emphasis: EmphasisSpec,
    pub idle: IdleSpec,
    pub thinking: ThinkingSpec,
    pub hover: HoverSpec,
    pub celebrate: CelebrateSpec,
}

// —— 默认值 = subtle(现观感的显式化;E1 用 golden 逐字节验证等价)——

impl Default for EnterSpec {
    fn default() -> Self {
        Self {
            spring: false,
            dur_factor: 1.0,
        }
    }
}
impl Default for ExitSpec {
    fn default() -> Self {
        Self { dissolve_ms: 0.0 }
    }
}
impl Default for EmphasisSpec {
    fn default() -> Self {
        Self {
            flash_alpha: 0.0,
            flash_ms: 300.0,
        }
    }
}
impl Default for IdleSpec {
    fn default() -> Self {
        // plan25 M2a 现值:0.15Hz 慢呼吸、幅度 ≤±8%(frame.rs WIDGET_PULSE 注释)。
        Self {
            pulse_amp: 0.08,
            pulse_hz: 0.15,
        }
    }
}
impl Default for ThinkingSpec {
    fn default() -> Self {
        Self {
            shimmer: true,
            trail_decay: 0.0,
        }
    }
}
impl Default for HoverSpec {
    fn default() -> Self {
        // S2 现值:下划线 alpha 0.9(theme link_underline 第四分量)。
        Self {
            underline_alpha: 0.9,
            card_lift: 1.0,
        }
    }
}
impl Default for EffectPreset {
    fn default() -> Self {
        Self::subtle()
    }
}

impl EffectPreset {
    /// subtle(默认档)= 现观感显式化。
    #[must_use]
    pub fn subtle() -> Self {
        Self {
            name: "subtle".into(),
            enter: EnterSpec::default(),
            exit: ExitSpec::default(),
            emphasis: EmphasisSpec::default(),
            idle: IdleSpec::default(),
            thinking: ThinkingSpec::default(),
            hover: HoverSpec::default(),
            celebrate: CelebrateSpec::default(),
        }
    }

    /// off = 全恒等(一切效果压平)。
    #[must_use]
    pub fn off() -> Self {
        Self {
            name: "off".into(),
            enter: EnterSpec {
                spring: false,
                dur_factor: 0.0,
            },
            exit: ExitSpec { dissolve_ms: 0.0 },
            emphasis: EmphasisSpec {
                flash_alpha: 0.0,
                flash_ms: 0.0,
            },
            idle: IdleSpec {
                pulse_amp: 0.0,
                pulse_hz: 0.0,
            },
            thinking: ThinkingSpec {
                shimmer: false,
                trail_decay: 0.0,
            },
            hover: HoverSpec {
                underline_alpha: 0.0,
                card_lift: 0.0,
            },
            celebrate: CelebrateSpec::default(),
        }
    }

    /// expressive = 36/37 词汇的保守组合(数值可经面板旋钮实调,不自评)。
    #[must_use]
    pub fn expressive() -> Self {
        Self {
            name: "expressive".into(),
            enter: EnterSpec {
                spring: true,
                dur_factor: 1.0,
            },
            exit: ExitSpec { dissolve_ms: 400.0 },
            emphasis: EmphasisSpec {
                flash_alpha: 0.6,
                flash_ms: 300.0,
            },
            idle: IdleSpec {
                pulse_amp: 0.08,
                pulse_hz: 0.15,
            },
            thinking: ThinkingSpec {
                shimmer: true,
                trail_decay: 0.7,
            },
            hover: HoverSpec {
                underline_alpha: 0.9,
                card_lift: 1.0,
            },
            celebrate: CelebrateSpec::default(),
        }
    }

    /// 按名取内置档;未知名 → None(调用方保持当前档,AR12)。
    #[must_use]
    pub fn builtin(name: &str) -> Option<Self> {
        match name {
            "off" => Some(Self::off()),
            "subtle" => Some(Self::subtle()),
            "expressive" => Some(Self::expressive()),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// E0/E2:内置三档数据快照(改数值必显式过快照,同 motion token 惯例)。
    #[test]
    fn builtin_presets_snapshot() {
        insta::assert_debug_snapshot!(
            "effect_presets",
            (
                EffectPreset::off(),
                EffectPreset::subtle(),
                EffectPreset::expressive()
            )
        );
    }

    /// schema 序列化往返:JSON 加载(缺字段用默认)+ 坏数据整份拒绝(AR12)。
    #[test]
    #[allow(clippy::float_cmp)] // reason: JSON 字面量往返即精确相等语义
    fn preset_json_roundtrip_and_reject() {
        let p: EffectPreset =
            serde_json::from_str(r#"{"name":"custom","exit":{"dissolve_ms":250}}"#).expect("json");
        assert_eq!(p.exit.dissolve_ms, 250.0);
        assert_eq!(p.enter, EnterSpec::default(), "缺字段用 subtle 默认");
        assert!(
            serde_json::from_str::<EffectPreset>("{oops").is_err(),
            "坏数据整份拒绝"
        );
        assert!(EffectPreset::builtin("nope").is_none(), "未知名不半应用");
    }
}
