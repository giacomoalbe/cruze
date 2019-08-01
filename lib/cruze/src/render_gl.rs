use gl;
use super::canvas;

use std;
use std::ffi::{CString, CStr};

use cgmath::{Matrix, Matrix4, Vector4};
use cgmath::prelude::*;

use glutin::event::VirtualKeyCode;

pub struct Renderer {
    gl: gl::Gl,
    vertices: Vec<f32>,
    indices: Vec<u32>,
    primitives: Vec<canvas::Primitive>,
    program: Program,
    projection: Matrix4<f32>,
    model: Matrix4<f32>,
    vao: gl::types::GLuint,
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
            projection: Matrix4::identity(),
            model: Matrix4::identity(),
            gl: gl.clone(),
            vertices: vec![],
            indices: vec![],
            primitives: vec![],
            program: program,
            vao: 0,
        };

        renderer.bind_vertex_arrays();

        renderer
    }

    /*
    fn generate_geometry() -> canvas::Rectangle {
        // The geometry has to be generated from
        // the Widget Tree

        let mut rectangle = canvas::Rectangle::new(
            canvas::Point2::new(0.0, 0.0),
            canvas::Size::new(200.0, 200.0),
            5.0
        );

        rectangle.translate(cgmath::Vector2::new(0.0, 0.0));
        rectangle.rotate(30.0);
        rectangle.scale(1.7);

        rectangle
    }
    */

    fn generate_geometry_primitives(&mut self) {
        let (vertices, indices, primitives) = canvas::generate_mesh();

        self.indices = indices;
        self.vertices = vertices;
        self.primitives = primitives;
    }

    fn bind_vertex_arrays(&mut self) {
        self.generate_geometry_primitives();

        self.vao = unsafe {
            let gl = self.gl.clone();

            let (mut vao, mut vbo, mut ebo) = (0, 0, 0);

            gl.GenVertexArrays(1, &mut vao);
            gl.GenBuffers(1, &mut vbo);
            gl.GenBuffers(1, &mut ebo);
            gl.BindVertexArray(vao);

            gl.BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl.BufferData(
                gl::ARRAY_BUFFER,
                (self.vertices.len() * std::mem::size_of::<gl::types::GLuint>()) as gl::types::GLsizeiptr,
                self.vertices.as_ptr() as *const gl::types::GLvoid,
                gl::STATIC_DRAW
            );

            gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
            gl.BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (self.indices.len() * std::mem::size_of::<gl::types::GLfloat>()) as gl::types::GLsizeiptr,
                self.indices.as_ptr() as *const gl::types::GLvoid,
                gl::STATIC_DRAW
            );

            // Enable location "Position (0)" in Vertex Shader
            gl.EnableVertexAttribArray(0);
            gl.VertexAttribPointer(
                0,
                3,
                gl::FLOAT,
                gl::FALSE,
                (3 * std::mem::size_of::<f32>()) as gl::types::GLint,
                std::ptr::null()
            );

            /*
            // Enable location "Color(1)" in Vertex Shader
            gl.EnableVertexAttribArray(1);
            gl.VertexAttribPointer(
                1,
                3,
                gl::FLOAT,
                gl::FALSE,
                (6 * std::mem::size_of::<f32>()) as gl::types::GLint,
                (3 * std::mem::size_of::<f32>()) as *const gl::types::GLvoid,
            );
            */

            // Unbind both VBO and VAO
            gl.BindBuffer(gl::ARRAY_BUFFER, 0);
            gl.BindVertexArray(0);

            vao
        };
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

            canvas::draw_primitives(
                gl,
                &mut self.program,
                &self.primitives
            );
        }
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
        self.gl.Uniform3fv(
            uniform_location,
            1,
            gradient.first_color.to_vec3().as_ptr()
        );

        let name = CString::new("last_color").unwrap();
        let uniform_location = self.gl.GetUniformLocation(self.id, name.as_ptr());
        self.gl.Uniform3fv(
            uniform_location,
            1,
            gradient.last_color.to_vec3().as_ptr()
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
