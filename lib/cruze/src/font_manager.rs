extern crate rusttype;

use rusttype::{
    gpu_cache::Cache,
    Font,
    Scale,
    point,
    Rect,
    PositionedGlyph,
};

pub struct FontManager {
}

impl FontManager {
    pub fn new(font_name: String) {
        FontManager {
        }
    }

    pub fn position_glyphs<'a>(&mut self, primitive: &canvas::Primitive)
        -> (Vec<PositionedGlyph<'a>>, rusttype::Rect<i32>) {
            let scale = Scale::uniform(50.0);

            let font = &self.font_caches
                .get(&primitive.font)
                .unwrap()
                .font;

            let v_metrics = font.v_metrics(scale);
            let advance_height =
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

                let mut glyph = base_glyph
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

            let bbox = Rect {
                min: point(min_x, min_y),
                max: point(max_x, max_y),
            };

            (result, bbox)
        }
}
