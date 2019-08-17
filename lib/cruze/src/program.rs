use gl;

use std;
use std::ffi::{
    CString,
    CStr
};

use super::canvas;

use cgmath::{
    Matrix,
    Matrix4,
};

use cgmath::prelude::*;

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

    pub unsafe fn set_texture(&self, name: &str, texId: u32) {
        let name = CString::new(name).unwrap();
        let uniform_location = self.gl.GetUniformLocation(self.id, name.as_ptr());

        self.gl.Uniform1i(uniform_location, 0);
        self.gl.ActiveTexture(gl::TEXTURE0);
        self.gl.BindTexture(gl::TEXTURE_2D, texId as u32);
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

        //println!("Value: {} => {}", field, boolean);
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
