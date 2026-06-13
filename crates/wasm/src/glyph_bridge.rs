//! glyph_bridge(M8)— 调 JS 侧 OffscreenCanvas 光栅化单个 grapheme → RGBA 位图。
//!
//! JS 契约:`(cluster: string) => { data: Uint8Array, width: number, height: number }`
//! (`data` 为 RGBA8,长度 = w*h*4)。

use js_sys::{Object, Reflect, Uint8Array};
use wasm_bindgen::{JsCast, JsValue};

pub(crate) struct Raster {
    pub(crate) rgba: Vec<u8>,
    pub(crate) w: u32,
    pub(crate) h: u32,
}

/// 调 JS 光栅化;`style` 为 StyleRole 数值(JS 据此选粗/斜/等宽字体)。
/// 任何字段缺失/类型不符 → `None`(由调用方跳过该字形)。
pub(crate) fn rasterize(raster_fn: &js_sys::Function, cluster: &str, style: u32) -> Option<Raster> {
    let ret = raster_fn
        .call2(
            &JsValue::NULL,
            &JsValue::from_str(cluster),
            &JsValue::from_f64(f64::from(style)),
        )
        .ok()?;
    let obj: &Object = ret.dyn_ref()?;
    let w = Reflect::get(obj, &JsValue::from_str("width"))
        .ok()?
        .as_f64()? as u32;
    let h = Reflect::get(obj, &JsValue::from_str("height"))
        .ok()?
        .as_f64()? as u32;
    let data = Reflect::get(obj, &JsValue::from_str("data")).ok()?;
    let bytes = data.dyn_into::<Uint8Array>().ok()?.to_vec();
    if bytes.len() < (w * h * 4) as usize {
        return None;
    }
    Some(Raster { rgba: bytes, w, h })
}
