use gl;
use super::program::{
    Shader,
    Program
};

use super::canvas;
use super::widgets::{
    Widget,
    Rect
};

use std;
use std::ffi::{
    CString,
};

use cgmath::{
    Matrix4,
};

use cgmath::prelude::*;

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

pub struct Renderer {
    gl: gl::Gl,
    canvas_data: canvas::CanvasData,
    program: Program,
    projection: Matrix4<f32>,
    model: Matrix4<f32>,
    texture: GlGlyphTexture,
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
            canvas_data: canvas::CanvasData::new(),
            program: program,
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

        renderer.create_vertex_arrays();

        renderer
    }

    fn generate_geometry_primitives(&mut self, children: &Vec<Box<dyn Widget>>) {
        self.canvas_data = canvas::generate_mesh_from_widget(&children);
        //self.canvas_data = canvas::generate_mesh();

        self.bind_vertex_arrays();
    }

    fn create_vertex_arrays(&mut self) {
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

            // Unbind VAO
            gl.BindVertexArray(0);

            (vao, pbo, tbo, ebo)
        };

        self.vao = vao;
        self.pbo = pbo;
        self.tbo = tbo;
        self.ebo = ebo;
    }

    fn bind_vertex_arrays(&mut self) {
        unsafe {
            let gl = self.gl.clone();

            gl.BindBuffer(gl::ARRAY_BUFFER, self.pbo);
            gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ebo);

            gl.BufferData(
                gl::ARRAY_BUFFER,
                (self.canvas_data.vertices.len() * std::mem::size_of::<gl::types::GLuint>()) as gl::types::GLsizeiptr,
                self.canvas_data.vertices.as_ptr() as *const gl::types::GLvoid,
                gl::STATIC_DRAW
            );

            gl.BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (self.canvas_data.indices.len() * std::mem::size_of::<gl::types::GLfloat>()) as gl::types::GLsizeiptr,
                self.canvas_data.indices.as_ptr() as *const gl::types::GLvoid,
                gl::STATIC_DRAW
            );

            gl.BindBuffer(gl::ARRAY_BUFFER, self.tbo);

            gl.BufferData(
                gl::ARRAY_BUFFER,
                (self.canvas_data.glyph_vertices.len() * std::mem::size_of::<gl::types::GLfloat>()) as gl::types::GLsizeiptr,
                self.canvas_data.glyph_vertices.as_ptr() as *const gl::types::GLvoid,
                gl::STATIC_DRAW
            );

            // Send texture to GPU
            for texture_data in self.canvas_data.glyph_tex_data.iter() {
                self.gl.BindTexture(gl::TEXTURE_2D, self.texture.name);
                self.gl.TexSubImage2D(
                    gl::TEXTURE_2D,
                    0,
                    texture_data.rect.min.x as _,
                    texture_data.rect.min.y as _,
                    texture_data.rect.width() as _,
                    texture_data.rect.height() as _,
                    gl::RED,
                    gl::UNSIGNED_BYTE,
                    texture_data.data.as_ptr() as _,
                );
            }

            // Unbind both VBO and VAO
            gl.BindBuffer(gl::ARRAY_BUFFER, 0);
            gl.BindVertexArray(0);
        }
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

    pub fn draw_primitives(&mut self) {

        let gl = self.gl.clone();

        let mut tris_offset = 0;
        let mut glyph_offset = 0;

        for primitive in self.canvas_data.primitives.iter() {
            unsafe {
                self.program.set_vec4("bbox", &primitive.bbox);
                self.program.set_gradient(&primitive.gradient);
                self.program.get_bool("is_textured");

                match primitive.kind {
                    canvas::PrimitiveType::Path => {
                        self.program.set_bool("is_textured", false);

                        gl.BindBuffer(gl::ARRAY_BUFFER, self.pbo);
                        gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ebo);

                        // Enable location "Position (0)" in Vertex Shader
                        // This must be done AFTER buffers have been
                        // filled with data
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
                        self.program.set_texture("font_tex", self.texture.name);
                        self.program.set_mat4("model", &primitive.model);

                        gl.BindBuffer(gl::ARRAY_BUFFER, self.tbo);

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

                        for _i in 0..primitive.num_vertices {
                            gl.DrawArrays(gl::TRIANGLE_STRIP, glyph_offset, 4);
                            glyph_offset += 4;
                        }
                    }
                }
            }
        }
    }

    pub fn resize(&mut self, size: glutin::dpi::LogicalSize, children: &Vec<Box<dyn Widget>>) {
        self.generate_geometry_primitives(&children);

        unsafe {
            self.projection = cgmath::ortho(
                0.0,
                size.width as f32,
                0.0,
                size.height as f32,
                -1.0,
                1.0
            );

            self.gl.Viewport(0, 0, size.width as i32, size.height as i32);
        }
    }
}
