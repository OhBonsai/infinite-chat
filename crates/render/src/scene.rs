//! scene(M8)— GPU 实例(Plan 3 K/L:SDF tile + 多页层 + 统一 quad)。
//!
//! 平铺 `#[repr(C)]` instance,`bytemuck::Pod` 零拷贝上传(CR4)。文字/矩形/图片共用同一
//! 实例管线(L5);本期实现文字 quad,矩形/图片留占位。每帧由可见集重建(块冻结仍在)。

use bytemuck::{Pod, Zeroable};

/// 一个字形/quad 的 GPU 实例(对应 glyph.wgsl 的 `InstanceIn`)。坐标为**世界坐标**,
/// 相机变换在着色器里做(L1)。
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct GpuInstance {
    /// 左上角世界坐标(px)。
    pub pos: [f32; 2],
    /// 宽高(px)。
    pub size: [f32; 2],
    /// SDF tile 在所属页内的 UV `[u0,v0,u1,v1]`。
    pub uv: [f32; 4],
    /// 上屏时刻(ms),着色器淡入。
    pub spawn_time: f32,
    /// 样式角色(着色器上色)。
    pub style: u32,
    /// atlas 页(纹理数组层)。
    pub layer: u32,
}

impl GpuInstance {
    /// 顶点缓冲布局(step mode = Instance)。
    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        const ATTRS: [wgpu::VertexAttribute; 6] = wgpu::vertex_attr_array![
            0 => Float32x2, // pos
            1 => Float32x2, // size
            2 => Float32x4, // uv
            3 => Float32,   // spawn_time
            4 => Uint32,    // style
            5 => Uint32,    // layer
        ];
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<GpuInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &ATTRS,
        }
    }
}

/// atlas 字形 key:同一 grapheme 在不同样式角色下是不同 SDF tile,需分桶。
/// render 与上传方(wasm GpuSink)必须用同一 key。
pub fn glyph_key(style: u32, cluster: &str) -> String {
    format!("{style}\u{1}{cluster}")
}
