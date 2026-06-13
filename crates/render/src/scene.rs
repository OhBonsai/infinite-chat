//! scene(M8)— FrameGlyph → GPU instance buffer(Plan1 无裁剪)。
//!
//! 平铺 `#[repr(C)]` instance,`bytemuck::Pod` 零拷贝上传(CR4)。Plan1 全量上传,
//! 视口裁剪/块缓存留 Plan2。

use bytemuck::{Pod, Zeroable};
use opencode_chat_core::FrameData;

use crate::atlas::GlyphAtlas;

/// 一个字形的 GPU 实例(对应 glyph.wgsl 的 `InstanceIn`)。
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct GpuInstance {
    pub pos: [f32; 2],
    pub size: [f32; 2],
    pub uv: [f32; 4],
    pub spawn_time: f32,
    /// 样式角色(着色器据此上色)。
    pub style: u32,
}

impl GpuInstance {
    /// 顶点缓冲布局(step mode = Instance)。
    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        const ATTRS: [wgpu::VertexAttribute; 5] = wgpu::vertex_attr_array![
            0 => Float32x2, // pos
            1 => Float32x2, // size
            2 => Float32x4, // uv
            3 => Float32,   // spawn_time
            4 => Uint32,    // style
        ];
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<GpuInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &ATTRS,
        }
    }
}

/// atlas 字形 key:同一 grapheme 在不同样式角色(粗/斜/code)下是不同位图,需分桶。
/// render 与上传方(wasm GpuSink)必须用同一 key。
pub fn glyph_key(style: u32, cluster: &str) -> String {
    format!("{style}\u{1}{cluster}")
}

/// 从一帧 + atlas 组装可绘制 instance。atlas 里没有的字形(尚未光栅化)跳过。
pub fn build_instances(frame: &FrameData, atlas: &GlyphAtlas) -> Vec<GpuInstance> {
    let mut out = Vec::with_capacity(frame.glyphs.len());
    for g in &frame.glyphs {
        let Some(uv) = atlas.uv(&glyph_key(g.style, &g.cluster)) else {
            continue;
        };
        out.push(GpuInstance {
            pos: g.pos,
            size: g.size,
            uv,
            spawn_time: g.spawn_time,
            style: g.style,
        });
    }
    out
}
