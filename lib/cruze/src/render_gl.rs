use gl;
use super::canvas;

use std;
use std::ffi::{CString, CStr};

pub struct Renderer {
    gl: gl::Gl,
    vertices: Vec<f32>,
    indices: Vec<u32>,
    program: Program,
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

        Renderer {
            gl: gl.clone(),
            vertices: vec![],
            indices: vec![],
            program: program,
            vao: 0,
        }
    }

    pub fn generate_geometry(&mut self) {
        // The geometry has to be generated from
        // the Widget Tree

        let (vertices, indices) =
            //canvas::rectangle(0.5, 0.3, 0.1);
        (
            vec![
                -0.8,  0.8, 1.0, 1.0, 0.0, 0.0,
                 0.8,  0.8, 1.0, 1.0, 0.0, 0.0,
                 0.8, -0.8, 1.0, 1.0, 0.0, 0.0,
                -0.8, -0.8, 1.0, 1.0, 0.0, 0.0,
            ],
            vec![
                0, 1, 2,
                0, 2, 3
            ]
        );

        self.vertices = vertices;
        self.indices = indices;

        self.bind_vertex_arrays();
    }

    fn bind_vertex_arrays(&mut self) {
        let (vao, vbo, ebo) = unsafe {
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
                (6 * std::mem::size_of::<f32>()) as gl::types::GLint,
                std::ptr::null()
            );

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

            // Unbind both VBO and VAO
            gl.BindBuffer(gl::ARRAY_BUFFER, 0);
            gl.BindVertexArray(0);

            (vao, vbo, ebo)
        };

        self.vao = vao;
    }

    pub fn draw(&self) {
        unsafe {
            let gl = self.gl.clone();

            gl.ClearColor(0.3, 0.3, 0.5, 0.1);
            gl.Clear(gl::COLOR_BUFFER_BIT);

            self.program.set_used();

            gl.BindVertexArray(self.vao);

            gl.DrawElements(
                gl::TRIANGLES,
                self.indices.len() as i32,
                gl::UNSIGNED_INT,
                std::ptr::null()
            );
        }
    }

    pub fn resize(&self, size: glutin::dpi::LogicalSize) {
        unsafe {
            self.gl.Viewport(0, 0, size.width as i32, size.height as i32);
        }
    }
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
