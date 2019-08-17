extern crate rusttype;
extern crate image;

use gl;
use super::program::{
    Shader,
    Program
};

use super::canvas;

use std;
use std::ffi::{
    CString,
    CStr
};

use std::collections::HashMap;

use cgmath::{
    Matrix,
    Matrix4,
};

use cgmath::prelude::*;

use rusttype::{
    gpu_cache::Cache,
    Font,
    Scale,
    point,
    Rect,
    PositionedGlyph,
};

struct GlGlyphTexture {
    name: gl::types::GLuint,
    gl: gl::Gl,
}

impl GlGlyphTexture {
    fn new(gl: gl::Gl, (width, height): (u32, u32)) -> Self {
        let mut name = 0;
        unsafe {
            // Create a texture for the glyphs
            // The texture holds 1 byte per pixel as alpha data
            let gl = gl.clone();

            gl.ActiveTexture(gl::TEXTURE0);
            gl.PixelStorei(gl::UNPACK_ALIGNMENT, 1);
            gl.GenTextures(1, &mut name);
            gl.BindTexture(gl::TEXTURE_2D, name);
            gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as _);
            gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as _);
            gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as _);
            gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as _);
            gl.TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RED as _,
                width as _,
                height as _,
                0,
                gl::RED,
                gl::UNSIGNED_BYTE,
                std::ptr::null(),
            );

            Self {
                name,
                gl
            }
        }
    }
}

impl Drop for GlGlyphTexture {
    fn drop(&mut self) {
        unsafe {
            self.gl.DeleteTextures(1, &self.name);
        }
    }
}

pub struct FontCache {
    pub cache: Cache<'static>,
    pub font: Font<'static>,
}

pub struct Renderer {
    gl: gl::Gl,
    vertices: Vec<f32>,
    indices: Vec<u32>,
    primitives: Vec<canvas::Primitive>,
    program: Program,
    projection: Matrix4<f32>,
    model: Matrix4<f32>,
    texture: GlGlyphTexture,
    font_caches: HashMap<String, FontCache>,
    vao: gl::types::GLuint,
    pbo: gl::types::GLuint,
    tbo: gl::types::GLuint,
    ebo: gl::types::GLuint,
}

impl Renderer {
    pub fn new(gl: &gl::Gl) -> Renderer {
        // TODO Remove CString boilerplate, put
        // in shader
        let vert_shader = Shader::from_vert_source(
            &gl,
            &CString::new(include_str!("triangle.vert")).unwrap()
        ).unwrap();

        let frag_shader = Shader::from_frag_source(
            &gl,
            &CString::new(include_str!("triangle.frag")).unwrap()
        ).unwrap();

        let program = Program
            ::from_shaders(
                &gl,
                &[vert_shader, frag_shader]
            )
            .unwrap();

        let mut renderer = Renderer {
            texture: GlGlyphTexture::new(gl.clone(), (512, 512)),
            projection: Matrix4::identity(),
            model: Matrix4::identity(),
            gl: gl.clone(),
            vertices: vec![],
            indices: vec![],
            primitives: vec![],
            program: program,
            font_caches: HashMap::new(),
            vao: 0,
            pbo: 0,
            tbo: 0,
            ebo: 0,
        };

        // Enable blending
        unsafe {
            gl.Enable(gl::BLEND);
            gl.BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl.Enable(gl::MULTISAMPLE);
        }

        renderer.generate_geometry_primitives();
        renderer.bind_vertex_arrays();

        renderer
    }

    fn generate_geometry_primitives(&mut self) {
        let (vertices, indices, primitives, fonts) = canvas::generate_mesh();

        self.indices = indices;
        self.vertices = vertices;
        self.primitives = primitives;

        // TODO: get dpi factor from gl context
        let dpi_factor = 1.0;

        let (cache_width, cache_height) = (
            (512.0 * dpi_factor) as u32,
            (512.0 * dpi_factor) as u32
        );

        for font in &fonts {
            let font_data = include_bytes!("../fonts/DejaVuSans.ttf");

            self.font_caches.insert(font.to_string(), FontCache {
                cache: Cache::builder()
                    .dimensions(cache_width, cache_height)
                    .build(),
                font: Font::from_bytes(font_data as &[u8]).unwrap()
            });
        }
    }

    fn bind_vertex_arrays(&mut self) {
        // pbo = PRIMITIVE Buffer Object, keeps everything related to primitive
        // tbo = TEXT Buffer Object, keeps all data relative to font rendering
        // ebo = ELEMENT Buffer Object, data relative to Indices

        let (vao, pbo, tbo, ebo) = unsafe {
            let gl = self.gl.clone();

            let (mut vao, mut pbo, mut tbo, mut ebo) = (0, 0, 0, 0);

            gl.GenVertexArrays(1, &mut vao);
            gl.BindVertexArray(vao);

            gl.GenBuffers(1, &mut pbo);
            gl.GenBuffers(1, &mut ebo);
            gl.GenBuffers(1, &mut tbo);

            gl.BindBuffer(gl::ARRAY_BUFFER, pbo);
            gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);

            gl.BufferData(
                gl::ARRAY_BUFFER,
                (self.vertices.len() * std::mem::size_of::<gl::types::GLuint>()) as gl::types::GLsizeiptr,
                self.vertices.as_ptr() as *const gl::types::GLvoid,
                gl::STATIC_DRAW
            );

            gl.BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (self.indices.len() * std::mem::size_of::<gl::types::GLfloat>()) as gl::types::GLsizeiptr,
                self.indices.as_ptr() as *const gl::types::GLvoid,
                gl::STATIC_DRAW
            );

            // Enable location "Position (0)" in Vertex Shader
            // This must be done AFTER buffers have been
            // filled with data
            gl.EnableVertexAttribArray(0);
            gl.VertexAttribPointer(
                0,
                3,
                gl::FLOAT,
                gl::FALSE,
                (3 * std::mem::size_of::<f32>()) as gl::types::GLint,
                std::ptr::null()
            );

            // Unbind both VBO and VAO
            gl.BindBuffer(gl::ARRAY_BUFFER, 0);
            gl.BindVertexArray(0);

            (vao, pbo, tbo, ebo)
        };

        self.vao = vao;
        self.pbo = pbo;
        self.tbo = tbo;
        self.ebo = ebo;
    }

    pub fn draw(&mut self) {
        unsafe {
            let gl = self.gl.clone();

            gl.ClearColor(0.3, 0.3, 0.5, 0.1);
            gl.Clear(gl::COLOR_BUFFER_BIT);

            self.program.set_used();

            self.program.set_mat4("projection", &self.projection);
            self.program.set_mat4("model", &self.model);

            gl.BindVertexArray(self.vao);

            self.draw_primitives();
        }
    }

    fn layout_text<'a>(&self, primitive: &canvas::Primitive)
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

    fn queue_glyphs(
        &self,
        primitive: &canvas::Primitive,
        glyphs: &Vec<PositionedGlyph<'static>>,
        bbox: rusttype::Rect<i32>,
    ) {
        // Get the cached instance on self
        // not create a new one each time
        // TODO: get dpi factor from gl context
        let dpi_factor = 1.0;

        let (cache_width, cache_height) = (
            (512.0 * dpi_factor) as u32,
            (512.0 * dpi_factor) as u32
        );

        let mut glyph_cache = Cache::builder()
            .dimensions(cache_width, cache_height)
            .build();

        for glyph in glyphs {
            glyph_cache.queue_glyph(0, glyph.clone());
        }

        glyph_cache.cache_queued(|rect, data| {
            unsafe {
                self.gl.BindTexture(gl::TEXTURE_2D, self.texture.name);
                self.gl.TexSubImage2D(
                    gl::TEXTURE_2D,
                    0,
                    rect.min.x as _,
                    rect.min.y as _,
                    rect.width() as _,
                    rect.height() as _,
                    gl::RED,
                    gl::UNSIGNED_BYTE,
                    data.as_ptr() as _,
                );
            }
        }).expect("FontCache is too big");

        let mut vertices: Vec<f32> = Vec::new();

        for glyph in glyphs {
            if let Ok(Some((uv_rect, s_rect))) = glyph_cache.rect_for(0, glyph) {
                let bbox_height = bbox.height() + bbox.min.y;

                let glyph_bbox = Rect {
                    min: point(
                        s_rect.min.x,
                        bbox_height - s_rect.max.y
                    ),
                    max: point(
                        s_rect.max.x,
                        bbox_height - s_rect.min.y
                    ),
                };

                // TL
                vertices.push(glyph_bbox.min.x as f32);
                vertices.push(glyph_bbox.max.y as f32);
                vertices.push(1.0);
                vertices.push(uv_rect.min.x as f32);
                vertices.push(uv_rect.min.y as f32);

                // TR
                vertices.push(glyph_bbox.max.x as f32);
                vertices.push(glyph_bbox.max.y as f32);
                vertices.push(1.0);
                vertices.push(uv_rect.max.x as f32);
                vertices.push(uv_rect.min.y as f32);

                // BL
                vertices.push(glyph_bbox.min.x as f32);
                vertices.push(glyph_bbox.min.y as f32);
                vertices.push(1.0);
                vertices.push(uv_rect.min.x as f32);
                vertices.push(uv_rect.max.y as f32);

                // BR
                vertices.push(glyph_bbox.max.x as f32);
                vertices.push(glyph_bbox.min.y as f32);
                vertices.push(1.0);
                vertices.push(uv_rect.max.x as f32);
                vertices.push(uv_rect.max.y as f32);
            }
        }

        let gl = self.gl.clone();

        unsafe {
            gl.BindVertexArray(self.vao);
            gl.BindBuffer(gl::ARRAY_BUFFER, self.tbo);
            gl.BufferData(
                gl::ARRAY_BUFFER,
                (vertices.len() * std::mem::size_of::<gl::types::GLfloat>()) as gl::types::GLsizeiptr,
                vertices.as_ptr() as *const gl::types::GLvoid,
                gl::STATIC_DRAW
            );

            // Enable location "Position (0)" in Vertex Shader
            // This must be done AFTER buffers have been
            // filled with data
            gl.EnableVertexAttribArray(0);
            gl.EnableVertexAttribArray(1);

            gl.VertexAttribPointer(
                0,
                3,
                gl::FLOAT,
                gl::FALSE,
                (5 * std::mem::size_of::<f32>()) as gl::types::GLint,
                std::ptr::null()
            );

            gl.VertexAttribPointer(
                1,
                2,
                gl::FLOAT,
                gl::FALSE,
                (5 * std::mem::size_of::<f32>()) as gl::types::GLint,
                (3 * std::mem::size_of::<gl::types::GLfloat>()) as _,
            );

            let vec4_bbox = cgmath::Vector4::new(
                bbox.max.y as f32,
                bbox.max.x as f32,
                bbox.min.y as f32,
                bbox.min.x as f32,
            );

            self.program.set_texture("font_tex", self.texture.name);
            self.program.set_vec4("bbox", &vec4_bbox);

            for (index, _glyph) in glyphs.iter().enumerate() {
                let offset = (index*4) as i32;

                gl.DrawArrays(gl::TRIANGLE_STRIP, offset, 4);
            }
        }
    }

    pub fn draw_primitives(&mut self) {
        let _start_time = std::time::Instant::now();

        let gl = self.gl.clone();

        let mut tris_offset = 0;

        for primitive in self.primitives.iter() {
            unsafe {
                self.program.set_vec4("bbox", &primitive.bbox);
                self.program.set_gradient(&primitive.gradient);
                self.program.get_bool("is_textured");

                match primitive.kind {
                    canvas::PrimitiveType::Path => {
                        self.program.set_bool("is_textured", false);

                        gl.BindVertexArray(self.vao);
                        gl.BindBuffer(gl::ARRAY_BUFFER, self.pbo);
                        gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ebo);

                        // Disable only TexCoord;
                        gl.DisableVertexAttribArray(1);
                        gl.EnableVertexAttribArray(0);

                        gl.VertexAttribPointer(
                            0,
                            3,
                            gl::FLOAT,
                            gl::FALSE,
                            (3 * std::mem::size_of::<f32>()) as gl::types::GLint,
                            std::ptr::null()
                        );

                        gl.DrawElements(
                            gl::TRIANGLES,
                            primitive.num_vertices as i32,
                            gl::UNSIGNED_INT,
                            (tris_offset * std::mem::size_of::<gl::types::GLuint>())
                            as *const std::ffi::c_void
                        );

                        tris_offset += primitive.num_vertices as usize;
                    },
                    canvas::PrimitiveType::Text => {
                        self.program.set_bool("is_textured", true);

                        // Layout text with info from primitive
                        let (glyphs, metrics) = self.layout_text(&primitive);

                        // Update texture position
                        self.queue_glyphs(&primitive, &glyphs, metrics);

                        // Load glyph position to TBO

                        // Enable TBO
                        // Draw Triangle Strip for each glyph
                    }
                }
            }
        }

        println!("Frame render: {}", _start_time.elapsed().as_millis());
    }

    pub fn resize(&mut self, size: glutin::dpi::LogicalSize) {
        unsafe {
            self.projection = cgmath::ortho(
                0.0,
                size.width as f32,
                0.0,
                size.height as f32,
                -1.0,
                1.0
            );

            /*
               self.rectangle.translate(
               cgmath::Vector2::new(
               size.width as f32 / 2.0,
               size.height as f32 / 2.0
               )
               );

               self.model = self.rectangle.model();
               */

            self.gl.Viewport(0, 0, size.width as i32, size.height as i32);
        }
    }

    /*
       pub fn send_key(&mut self, key: VirtualKeyCode) {
       match key {
       VirtualKeyCode::R => {
       let current_rotation = self.rectangle.current_rotation + 15.0;

       self.rectangle.rotate(current_rotation);
       },
       VirtualKeyCode::D => {
       let current_rotation = self.rectangle.current_rotation - 15.0;

       self.rectangle.rotate(current_rotation);
       },
       VirtualKeyCode::Left => {
       let mut current_translate = self.rectangle.current_translate;

       current_translate.x -= 10.0;

       self.rectangle.translate(current_translate);
       },
       VirtualKeyCode::Right => {
       let mut current_translate = self.rectangle.current_translate;

       current_translate.x += 10.0;

       self.rectangle.translate(current_translate);
       },
       VirtualKeyCode::Up => {
       let mut current_translate = self.rectangle.current_translate;

       current_translate.y += 10.0;

       self.rectangle.translate(current_translate);
       },
       VirtualKeyCode::Down => {
       let mut current_translate = self.rectangle.current_translate;

       current_translate.y -= 10.0;

       self.rectangle.translate(current_translate);
       },
       VirtualKeyCode::S => {
       let current_scale = self.rectangle.current_scale + 0.1;

       self.rectangle.scale(current_scale);
       },
       VirtualKeyCode::X => {
       let current_scale = self.rectangle.current_scale - 0.1;

       self.rectangle.scale(current_scale);
       },
       _ => ()
       };

       self.model = self.rectangle.model()
       }
       */
}
