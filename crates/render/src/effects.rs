//! effects(M9)— Plan1 仅一个着色器淡入 + profile 占位。
//!
//! 效果靠 `time - spawn_time` 在 WGSL 算(0002 §5),CPU 不参与逐字动画。profile 决定
//! 淡入时长;`Off` = 参数置零(`fade_ms=0`),非分支,满足恒等收敛(AR3)。

use bytemuck::{Pod, Zeroable};

/// 效果档位(0002 §5.1)。GPU 降级/省电/用户设置统一汇入这里。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EffectProfile {
    /// 全效:~180ms 淡入。
    #[default]
    Full,
    /// 减弱:~80ms。
    Reduced,
    /// 关闭:无淡入。
    Off,
}

impl EffectProfile {
    /// 淡入时长(ms)。
    pub fn fade_ms(self) -> f32 {
        match self {
            EffectProfile::Full => 180.0,
            EffectProfile::Reduced => 80.0,
            EffectProfile::Off => 0.0,
        }
    }
}

/// 着色器全局 uniform(对应 glyph.wgsl 的 `Globals`)。含 2D 相机(Plan 3 L)。
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct Globals {
    pub viewport: [f32; 2],
    pub time_ms: f32,
    pub fade_ms: f32,
    /// 相机:屏幕左上角对应的世界坐标。
    pub cam_pan: [f32; 2],
    /// 相机缩放。
    pub cam_zoom: f32,
    /// 到达高亮强度(Plan 25 M2e / design §3.3 karaoke 读头):enter 期 tint ×(1+boost×(1-e)),
    /// 0 = 关(兼当 uniform 16 字节对齐位)。
    pub arrive_boost: f32,
    /// Plan 42 编队进度(0=恒等直通,不进 formation 支;1=完全成形)。注入时钟推进,可反向散场。
    pub form_progress: f32,
    /// 16 字节对齐填充(下方 wind 是 vec4,须 16 对齐)。
    pub form_pad: [f32; 3],
    /// Plan 42 指针风场 `[pos_x, pos_y(世界), radius, strength]`;默认全 0=无风恒等。
    pub wind: [f32; 4],
}
