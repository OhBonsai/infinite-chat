//! msdf(0015 §2.3)— 离线烘焙 MSDF 字体元数据 + 源解析几何。
//!
//! 持有 BMFont `chars[]` 解析出的逐字 metrics(cell + offset/page)与 coverage(codepoint 集),
//! 供 `GpuSink` 在 atlas alloc 处判命中(O(1))并算 MSDF quad 的世界几何。像素页由 lib.rs 解码
//! 上传到 render 的静态图集;本模块不碰 wgpu。

use std::collections::HashMap;

use js_sys::{Reflect, Uint32Array};
use wasm_bindgen::{JsCast, JsValue};

/// 一个 MSDF 字形在烘焙图集里的 cell(px)+ 相对 pen 的偏移(BMFont 单位,字号 = `MsdfFont::size`)。
#[derive(Clone, Copy)]
pub(crate) struct MsdfGlyph {
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) w: f32,
    pub(crate) h: f32,
    pub(crate) xoff: f32,
    pub(crate) yoff: f32,
    pub(crate) page: u32,
}

/// 烘焙 MSDF 字体:图集尺寸 + 字号(em)+ 逐字 metrics。
pub(crate) struct MsdfFont {
    pub(crate) atlas_w: f32,
    pub(crate) atlas_h: f32,
    /// BMFont 字号(metrics/offset 的单位 em)。
    pub(crate) size: f32,
    glyphs: HashMap<u32, MsdfGlyph>,
}

impl MsdfFont {
    /// 命中查询(coverage 判定 + 取 metrics)。
    pub(crate) fn glyph(&self, codepoint: u32) -> Option<&MsdfGlyph> {
        self.glyphs.get(&codepoint)
    }

    pub(crate) fn len(&self) -> usize {
        self.glyphs.len()
    }

    /// 从 JS 传入的元数据构建(`atlasW/atlasH/fontSize` 标量 + `ids` Uint32Array(codepoint)
    /// + `cells` Float32Array(每字 7 个:x,y,w,h,xoff,yoff,page))。
    pub(crate) fn from_js(meta: &JsValue) -> Result<MsdfFont, String> {
        let atlas_w = num(meta, "atlasW")?;
        let atlas_h = num(meta, "atlasH")?;
        let size = num(meta, "fontSize")?;
        let ids: Uint32Array = field(meta, "ids")?
            .dyn_into()
            .map_err(|_| "ids 非 Uint32Array".to_string())?;
        let cells = js_sys::Float32Array::from(field(meta, "cells")?);
        let n = ids.length() as usize;
        if cells.length() as usize != n * 7 {
            return Err(format!("cells 长度 {} ≠ ids {} ×7", cells.length(), n));
        }
        let ids_v = ids.to_vec();
        let cells_v = cells.to_vec();
        let mut glyphs = HashMap::with_capacity(n);
        for i in 0..n {
            let c = &cells_v[i * 7..i * 7 + 7];
            glyphs.insert(
                ids_v[i],
                MsdfGlyph {
                    x: c[0],
                    y: c[1],
                    w: c[2],
                    h: c[3],
                    xoff: c[4],
                    yoff: c[5],
                    page: c[6] as u32,
                },
            );
        }
        Ok(MsdfFont {
            atlas_w,
            atlas_h,
            size,
            glyphs,
        })
    }
}

fn field(obj: &JsValue, key: &str) -> Result<JsValue, String> {
    Reflect::get(obj, &JsValue::from_str(key)).map_err(|_| format!("缺字段 {key}"))
}

fn num(obj: &JsValue, key: &str) -> Result<f32, String> {
    field(obj, key)?
        .as_f64()
        .map(|v| v as f32)
        .ok_or_else(|| format!("{key} 非数字"))
}
