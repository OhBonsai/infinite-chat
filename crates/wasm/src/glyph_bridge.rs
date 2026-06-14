//! glyph_bridge(M8)— 调 JS 侧 TinySDF 生成单个 grapheme 的 SDF tile(Plan 3 K)。
//!
//! JS 契约:`(cluster: string, style: number) => Uint8Array`(长度 = `TILE_PX²` 的 R8
//! 单通道距离场,0.5≈字形边缘)。固定 tile 尺寸(SDF 缩放无关)。

use js_sys::Uint8Array;
use wasm_bindgen::{JsCast, JsValue};

/// 调 JS 生成 SDF tile;失败/类型不符 → `None`(调用方跳过该字形)。
pub(crate) fn rasterize_sdf(
    raster_fn: &js_sys::Function,
    cluster: &str,
    style: u32,
) -> Option<Vec<u8>> {
    let ret = raster_fn
        .call2(
            &JsValue::NULL,
            &JsValue::from_str(cluster),
            &JsValue::from_f64(f64::from(style)),
        )
        .ok()?;
    let arr = ret.dyn_into::<Uint8Array>().ok()?;
    Some(arr.to_vec())
}
