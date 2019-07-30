use gl;
use canvas;

use std;
use std::ffi::{CString, CStr};

pub struct Renderer {
    gl: gl::Gl,
    vertices: Vec<f64>,
    indices: Vec<u32>,
    program: Program,
}

impl Renderer {
    pub fn new(gl: gl::Gl) -> Renderer {
        let (vertices, indices) = (vec![], vec![]);
            canvas::rectangle(0.5, 0.5 * ((4/3) as f32), 0.5);

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
            gl: gl,
            vertices: vertices,
            indices: indices,
            program: program
        }
    }

    fn generate_geometry(&self) -> (Vec<f32>, Vec<u32>) {
        // The geometry has to be generated from
        // the Widget Tree
        println!("Generating geometry");
        (Vec::new(), Vec::new())
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
