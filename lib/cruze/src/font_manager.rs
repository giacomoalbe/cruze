extern crate rusttype;

use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::iter::FromIterator;

use rusttype::{
    gpu_cache::Cache,
    Font,
    Scale,
    point,
    Rect,
    PositionedGlyph,
};

use super::canvas;

pub struct FontManager {
    font_caches: HashMap<String, Font<'static>>,
    glyphs: Vec<PositionedGlyph<'static>>,
    glyph_cache: Cache<'static>,
}

#[derive(Debug)]
pub struct GlyphTexData {
    pub rect: Rect<u32>,
    pub data: Vec<u8>,
}

impl FontManager {
    pub fn new() -> FontManager {
        FontManager {
            font_caches: HashMap::new(),
            glyphs: Vec::new(),
            glyph_cache: Cache::builder().build(),
        }
    }

    pub fn position_glyphs<'a>(&mut self, primitive: &mut canvas::Primitive) {
        let scale = Scale::uniform(primitive.stroke_width);

        let font = match self.font_caches.entry(primitive.font.clone()) {
            Entry::Vacant(entry) => {
                // TODO: pick the correct font with font-kit
                let font_data = include_bytes!("../fonts/DejaVuSans.ttf");
                let font = Font::from_bytes(font_data as &[u8]).unwrap();

                entry.insert(font)
            },
            Entry::Occupied(entry) => entry.into_mut()
        };

        let v_metrics = font.v_metrics(scale);
        let _advance_height =
            v_metrics.ascent -
            v_metrics.descent +
            v_metrics.line_gap;

        let mut caret = point(0.0, v_metrics.ascent);
        let mut last_glyph_id = None;

        let mut result = vec![];

        let mut min_x: i32 = 10000;
        let mut max_x: i32 = 0;
        let mut min_y: i32 = 10000;
        let mut max_y: i32 = 0;

        for c in primitive.text.chars() {
            let base_glyph = font.glyph(c);

            if let Some(id) = last_glyph_id.take() {
                caret.x += font.pair_kerning(scale, id, base_glyph.id());
            }

            last_glyph_id = Some(base_glyph.id());

            let glyph = base_glyph
                .scaled(scale)
                .positioned(caret);

            let advance = glyph.unpositioned().h_metrics().advance_width;

            if let Some(bb) = glyph.pixel_bounding_box() {
                min_x = min_x.min(bb.min.x);
                max_x = max_x.max(bb.max.x);
                min_y = min_y.min(bb.min.y);
                max_y = max_y.max(bb.max.y);
            }

            caret.x += advance;

            result.push(glyph);
        }

        let bbox = cgmath::Vector4::new(
            max_y as f32,
            max_x as f32,
            min_y as f32,
            min_x as f32
        );

        primitive.bbox = bbox;
        primitive.num_vertices = primitive.text.split_whitespace().collect::<String>().len() as u32;

        self.glyphs.append(&mut result);
    }

    pub fn cache_glyphs(&mut self) -> Vec<GlyphTexData> {
        // Get the cached instance on self
        // not create a new one each time
        // TODO: get dpi factor from gl context
        let dpi_factor = 1.0;

        let (cache_width, cache_height) = (
            (512.0 * dpi_factor) as u32,
            (512.0 * dpi_factor) as u32
        );

        let mut glyph_tex_data = Vec::new();

        self.glyph_cache
            .to_builder()
            .dimensions(cache_width, cache_height)
            .rebuild(&mut self.glyph_cache);

        for glyph in self.glyphs.iter() {
            self.glyph_cache.queue_glyph(0, glyph.clone());
        }

        self.glyph_cache.cache_queued(|rect, data| {
            glyph_tex_data.push(GlyphTexData {
                rect,
                data: Vec::from_iter(data.iter().cloned())
            });
        }).expect("FontCache is too big");

        glyph_tex_data
    }

    pub fn generate_glyph_vertices(&mut self) -> (Vec<f32>, Vec<GlyphTexData>) {
        let glyph_tex_data = self.cache_glyphs();

        let mut vertices: Vec<f32> = Vec::new();

        for glyph in self.glyphs.iter() {
            if let Ok(Some((uv_rect, s_rect))) = self.glyph_cache.rect_for(0, glyph) {
                // TL
                vertices.push(s_rect.min.x as f32);
                vertices.push(s_rect.max.y as f32);
                vertices.push(1.0);
                vertices.push(uv_rect.min.x as f32);
                vertices.push(uv_rect.max.y as f32);

                // TR
                vertices.push(s_rect.max.x as f32);
                vertices.push(s_rect.max.y as f32);
                vertices.push(1.0);
                vertices.push(uv_rect.max.x as f32);
                vertices.push(uv_rect.max.y as f32);

                // BL
                vertices.push(s_rect.min.x as f32);
                vertices.push(s_rect.min.y as f32);
                vertices.push(1.0);
                vertices.push(uv_rect.min.x as f32);
                vertices.push(uv_rect.min.y as f32);

                // BR
                vertices.push(s_rect.max.x as f32);
                vertices.push(s_rect.min.y as f32);
                vertices.push(1.0);
                vertices.push(uv_rect.max.x as f32);
                vertices.push(uv_rect.min.y as f32);
            }
        }

        (vertices, glyph_tex_data)
    }
}
