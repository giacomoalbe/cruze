use gl;
use super::canvas;

use std;
use std::ffi::{
    CString,
    CStr
};

use cgmath::{
    Matrix,
    Matrix4,
    Vector4
};

use cgmath::prelude::*;

use glutin::event::VirtualKeyCode;

use glyph_brush::{
    *,
    rusttype::*,
    BrushAction,
    BrushError,
    GlyphBrushBuilder,
    Section,
};

type Vertex = [gl::types::GLfloat; 9];

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
    vertices: Vec<f32>,
    indices: Vec<u32>,
    primitives: Vec<canvas::Primitive>,
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
            texture: GlGlyphTexture::new(gl.clone(), (256, 256)),
            projection: Matrix4::identity(),
            model: Matrix4::identity(),
            gl: gl.clone(),
            vertices: vec![],
            indices: vec![],
            primitives: vec![],
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

        renderer.generate_geometry_primitives();
        renderer.bind_vertex_arrays();
        renderer.bind_glyph_renderer();

        renderer
    }

    fn bind_glyph_renderer(&self) {
        let gl = self.gl.clone();

        unsafe {
            gl.BindVertexArray(self.vao);
            gl.BindBuffer(gl::ARRAY_BUFFER, self.tbo);

            let mut offset = 0;

            for (v_field, float_count) in &[
                ("left_top", 3),
                ("right_bottom", 2),
                ("tex_left_top", 2),
                ("tex_right_bottom", 2)
            ] {
                let attr = self.program.getAttribLocation(v_field);

                println!("{} GetAttribLocation -> {}", v_field, attr);

                gl.EnableVertexAttribArray(attr as _);
                gl.VertexAttribPointer(
                    attr as _,
                    *float_count,
                    gl::FLOAT,
                    gl::FALSE as _,
                    std::mem::size_of::<Vertex>() as _,
                    offset as _,
                );

                gl.VertexAttribDivisor(attr as _, 1);

                offset += float_count * 4;
            }

            // Unbind text primitive array buffer
            gl.BindBuffer(gl::ARRAY_BUFFER, 0);
            gl.BindVertexArray(0);
        };
    }

    fn generate_geometry_primitives(&mut self) {
        let (vertices, indices, primitives) = canvas::generate_mesh();

        self.indices = indices;
        self.vertices = vertices;
        self.primitives = primitives;
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

    pub fn draw_primitives(&mut self) {
        let start_time = std::time::Instant::now();

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

                        gl.BindBuffer(gl::ARRAY_BUFFER, self.pbo);
                        gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ebo);

                        gl.EnableVertexAttribArray(0);

                        gl.DisableVertexAttribArray(1);
                        gl.DisableVertexAttribArray(2);
                        gl.DisableVertexAttribArray(3);
                        gl.DisableVertexAttribArray(4);

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

                        println!("Gradient:  {:?}", primitive.gradient);

                        let dejavu: &[u8] = include_bytes!("../fonts/DejaVuSans.ttf");

                        let mut glyph_brush = GlyphBrushBuilder
                            ::using_font_bytes(dejavu)
                            .build();

                        glyph_brush.queue(Section {
                            text: &primitive.text,
                            screen_position: (400.0, 400.0),
                            scale: Scale::uniform(70.0),
                            layout: Layout::default()
                                .h_align(HorizontalAlign::Center)
                                .v_align(VerticalAlign::Center),
                            ..Section::default()
                        });

                        let mut vertex_count = 0;
                        let mut vertex_max = vertex_count;
                        let mut min_x: f32 = 1e6;

                        match glyph_brush.process_queued(
                            |rect, tex_data| unsafe {
                                // Update part of gpu texture with new glyph alpha values
                                // Run this for every character in the text
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
                                    tex_data.as_ptr() as _,
                                );
                            },
                            |vertex_data| {
                                // Window coordinates
                                let glyph_brush::GlyphVertex {
                                    mut tex_coords,
                                    pixel_coords,
                                    bounds,
                                    color,
                                    z,
                                } = vertex_data;

                                let gl_bounds = bounds;

                                let mut gl_rect = glyph_brush::rusttype::Rect {
                                    min: glyph_brush::rusttype::point(pixel_coords.min.x as f32, pixel_coords.min.y as f32),
                                    max: glyph_brush::rusttype::point(pixel_coords.max.x as f32, pixel_coords.max.y as f32),
                                };

                                // handle overlapping bounds, modify uv_rect to preserve texture aspect
                                if gl_rect.max.x > gl_bounds.max.x {
                                    let old_width = gl_rect.width();
                                    gl_rect.max.x = gl_bounds.max.x;
                                    tex_coords.max.x = tex_coords.min.x + tex_coords.width() * gl_rect.width() / old_width;
                                }
                                if gl_rect.min.x < gl_bounds.min.x {
                                    let old_width = gl_rect.width();
                                    gl_rect.min.x = gl_bounds.min.x;
                                    tex_coords.min.x = tex_coords.max.x - tex_coords.width() * gl_rect.width() / old_width;
                                }
                                if gl_rect.max.y > gl_bounds.max.y {
                                    let old_height = gl_rect.height();
                                    gl_rect.max.y = gl_bounds.max.y;
                                    tex_coords.max.y = tex_coords.min.y + tex_coords.height() * gl_rect.height() / old_height;
                                }
                                if gl_rect.min.y < gl_bounds.min.y {
                                    let old_height = gl_rect.height();
                                    gl_rect.min.y = gl_bounds.min.y;
                                    tex_coords.min.y = tex_coords.max.y - tex_coords.height() * gl_rect.height() / old_height;
                                }

                                println!("Pixel Coord: {:?}", pixel_coords);
                                println!("GL_RECT w: {} h: {}", gl_rect.width(), gl_rect.height());
                                println!("Tex coords: {:?}", tex_coords);

                                println!("Min x: {}, gl_rect.min.x: {}", min_x, gl_rect.min.x);
                                if gl_rect.min.x < min_x {
                                    min_x = gl_rect.min.x;
                                }
                                println!("Min x: {} after", min_x);

                                [
                                    gl_rect.min.x,
                                    gl_rect.max.y,
                                    z,
                                    gl_rect.max.x,
                                    gl_rect.min.y,
                                    tex_coords.min.x,
                                    tex_coords.max.y,
                                    tex_coords.max.x,
                                    tex_coords.min.y,
                                ]
                            }
                        ) {
                            Ok(BrushAction::Draw(vertices)) => {
                                // Draw new vertices.
                                let vertex_count = vertices.len();
                                let gl = self.gl.clone();
                                gl.BindBuffer(gl::ARRAY_BUFFER, self.tbo);

                                // Disable Position attribute,
                                // since it's not used
                                gl.DisableVertexAttribArray(0);

                                gl.EnableVertexAttribArray(1);
                                gl.EnableVertexAttribArray(2);
                                gl.EnableVertexAttribArray(3);
                                gl.EnableVertexAttribArray(4);

                                println!("Min x: {}", min_x);

                                unsafe {
                                    if vertex_max < vertex_count {
                                        println!("BufferData");
                                        gl.BufferData(
                                            gl::ARRAY_BUFFER,
                                            (vertex_count * std::mem::size_of::<Vertex>()) as gl::types::GLsizeiptr,
                                            vertices.as_ptr() as _,
                                            gl::STATIC_DRAW,
                                        );
                                    } else {
                                        println!("BufferSubData");
                                        gl.BufferSubData(
                                            gl::ARRAY_BUFFER,
                                            0,
                                            (vertex_count * std::mem::size_of::<Vertex>()) as gl::types::GLsizeiptr,
                                            vertices.as_ptr() as _,
                                        );
                                    }
                                }

                                vertex_max = vertex_max.max(vertex_count);

                                gl.DrawArraysInstanced(gl::TRIANGLE_STRIP, 0, 4, vertex_count as _);
                            }

                            Ok(BrushAction::ReDraw) => {
                                // Re-draw last frame's vertices unmodified.
                            }
                            Err(BrushError::TextureTooSmall { suggested }) => {
                                // Enlarge texture + glyph_brush texture cache and retry.
                            }
                        }
                    }
                    _ => ()
                }
            }
        }

        //println!("Frame render: {}", start_time.elapsed().as_micros());
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

pub struct Program {
    gl: gl::Gl,
    id: gl::types::GLuint,
}

impl Program {
    pub fn from_shaders(gl: &gl::Gl, shaders: &[Shader])
        -> Result<Program, String> {
            let program_id = unsafe { gl.CreateProgram() };

            for shader in shaders {
                unsafe { gl.AttachShader(program_id, shader.id()); }
            }

            unsafe { gl.LinkProgram(program_id); }

            // Error handling of Program Link
            let mut success: gl::types::GLint = 1;

            unsafe {
                gl.GetProgramiv(program_id, gl::LINK_STATUS, &mut success);
            }

            if success == 0 {
                // ERROR in Program Link
                // get error buffer length
                let mut len: gl::types::GLint = 0;

                unsafe {
                    gl.GetProgramiv(program_id, gl::INFO_LOG_LENGTH, &mut len);
                }

                let error = create_whitespace_cstring_with_len(len as usize);

                unsafe {
                    gl.GetProgramInfoLog(
                        program_id,
                        len,
                        std::ptr::null_mut(),
                        error.as_ptr() as *mut gl::types::GLchar
                    );
                }

                return Err(error.to_string_lossy().into_owned());
            }

            // Once compiled I can detach shaders
            for shader in shaders {
                unsafe { gl.DetachShader(program_id, shader.id()); }
            }

            Ok (Program {
                gl: gl.clone(),
                id: program_id
            })
        }

    pub fn id(&self) -> gl::types::GLuint {
        self.id
    }

    pub fn set_used(&self) {
        unsafe {
            self.gl.UseProgram(self.id);
        }
    }

    pub unsafe fn set_bool(&self, name: &str, value: bool) {
        let name = CString::new(name).unwrap();
        let uniform_location = self.gl.GetUniformLocation(self.id, name.as_ptr());

        self.gl.Uniform1i(uniform_location, value as _);
    }

    pub unsafe fn set_mat4(&self, name: &str, mat: &Matrix4<f32>) {
        let name = CString::new(name).unwrap();
        let uniform_location = self.gl.GetUniformLocation(self.id, name.as_ptr());

        self.gl.UniformMatrix4fv(uniform_location, 1, gl::FALSE, mat.as_ptr());
    }

    pub unsafe fn set_vec2(&self, name: &str, vec: &cgmath::Vector2<f32>) {
        let name = CString::new(name).unwrap();
        let uniform_location = self.gl.GetUniformLocation(self.id, name.as_ptr());
        self.gl.Uniform2fv(uniform_location, 1, vec.as_ptr());
    }

    pub unsafe fn set_vec3(&self, name: &str, mat: &cgmath::Vector3<f32>) {
        let name = CString::new(name).unwrap();
        let uniform_location = self.gl.GetUniformLocation(self.id, name.as_ptr());
        self.gl.Uniform3fv(uniform_location, 1, mat.as_ptr());
    }

    pub unsafe fn set_vec4(&self, name: &str, vec: &cgmath::Vector4<f32>) {
        let name = CString::new(name).unwrap();
        let uniform_location = self.gl.GetUniformLocation(self.id, name.as_ptr());
        self.gl.Uniform4fv(uniform_location, 1, vec.as_ptr());
    }

    pub unsafe fn set_gradient(&self, gradient: &canvas::Gradient) {
        let name = CString::new("first_color").unwrap();
        let uniform_location = self.gl.GetUniformLocation(self.id, name.as_ptr());
        self.gl.Uniform4fv(
            uniform_location,
            1,
            gradient.first_color.to_vec().as_ptr()
        );

        let name = CString::new("last_color").unwrap();
        let uniform_location = self.gl.GetUniformLocation(self.id, name.as_ptr());
        self.gl.Uniform4fv(
            uniform_location,
            1,
            gradient.last_color.to_vec().as_ptr()
        );

        let name = CString::new("start_pos").unwrap();
        let uniform_location = self.gl.GetUniformLocation(self.id, name.as_ptr());
        let start_pos = cgmath::Vector2 {
            x: gradient.start_pos.x,
            y: gradient.start_pos.y
        };
        self.gl.Uniform2fv(
            uniform_location,
            1,
            start_pos.as_ptr()
        );

        let name = CString::new("end_pos").unwrap();
        let uniform_location = self.gl.GetUniformLocation(self.id, name.as_ptr());
        let end_pos = cgmath::Vector2 {
            x: gradient.end_pos.x,
            y: gradient.end_pos.y
        };
        self.gl.Uniform2fv(
            uniform_location,
            1,
            end_pos.as_ptr()
        );

        let name = CString::new("gradient_type").unwrap();
        let uniform_location = self.gl.GetUniformLocation(self.id, name.as_ptr());
        self.gl.Uniform1ui(
            uniform_location,
            gradient.gradient_type as gl::types::GLuint
        );

        let name = CString::new("radius").unwrap();
        let uniform_location = self.gl.GetUniformLocation(self.id, name.as_ptr());
        self.gl.Uniform1f(
            uniform_location,
            gradient.radius as gl::types::GLfloat
        );
    }

    pub unsafe fn getAttribLocation(&self, field: &str) -> gl::types::GLint {
        self.gl.GetAttribLocation(self.id, CString::new(field).unwrap().as_ptr())
    }

    pub unsafe fn get_bool(&self, field: &str) {
        let name = CString::new(field).unwrap();
        let uniform_location = self.gl.GetUniformLocation(self.id, name.as_ptr());

        let mut boolean: i32 = 0;

        self.gl.GetUniformiv(self.id, uniform_location, &mut boolean as _);

        println!("Value: {} => {}", field, boolean);
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        unsafe {
            self.gl.DeleteProgram(self.id);
        }
    }
}

pub struct Shader {
    gl: gl::Gl,
    id: gl::types::GLuint,
}

impl Shader {
    pub fn from_source(
        gl: &gl::Gl,
        source: &CStr,
        kind: gl::types::GLenum
    ) -> Result<Shader, String> {
        let id = shader_from_source(gl, source, kind)?;

        Ok(Shader {
            gl: gl.clone(),
            id
        })
    }

    pub fn id(&self) -> gl::types::GLuint {
        self.id
    }

    pub fn from_vert_source(gl: &gl::Gl, source: &CStr) -> Result<Shader, String> {
        Shader::from_source(gl, source, gl::VERTEX_SHADER)
    }

    pub fn from_frag_source(gl: &gl::Gl, source: &CStr) -> Result<Shader, String> {
        Shader::from_source(gl, source, gl::FRAGMENT_SHADER)
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            self.gl.DeleteShader(self.id);
        }
    }
}

fn shader_from_source(
    gl: &gl::Gl,
    source: &CStr,
    kind: gl::types::GLuint
) -> Result<gl::types::GLuint, String> {
    let id = unsafe {
        gl.CreateShader(kind)
    };

    unsafe {
        gl.ShaderSource(id, 1, &source.as_ptr(), std::ptr::null());
        gl.CompileShader(id);
    }

    let mut success: gl::types::GLint = 1;

    unsafe {
        gl.GetShaderiv(id, gl::COMPILE_STATUS, &mut success);
    }

    if success == 0 {
        // Error in Shader compilation

        // Length of the error string in OpenGL
        let mut len: gl::types::GLint = 0;

        unsafe {
            gl.GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut len);
        }

        let error = create_whitespace_cstring_with_len(len as usize);


        unsafe {
            gl.GetShaderInfoLog(
                id,
                len,
                std::ptr::null_mut(),
                error.as_ptr() as *mut gl::types::GLchar
            );
        }

        return Err(error.to_string_lossy().into_owned());
    }

    Ok(id)
}

fn create_whitespace_cstring_with_len(len: usize) -> CString {
    let mut buffer: Vec<u8> = Vec::with_capacity(len + 1);
    buffer.extend(([b' ']).iter().cycle().take(len));

    unsafe {
        CString::from_vec_unchecked(buffer)
    }
}
