extern crate sdl2;
extern crate gl;

mod render_gl;
mod canvas;

use std;
use std::ffi::{CString, CStr};

fn main() {
    let width: i32 = 800;
    let height: i32 = 800;

    let sdl = sdl2::init().unwrap();

    let video_subsystem = sdl.video().unwrap();

    // Setting OpenGL Attributes
    let gl_attr = video_subsystem.gl_attr();

    gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
    gl_attr.set_context_version(4, 5);

    let window = video_subsystem
        .window("OpenGL Rust", width as u32, height as u32)
        .opengl()           // Add OpenGL flag
        .resizable()
        .build()
        .unwrap();

    let _gl_context = window.gl_create_context().unwrap();

    let gl = gl::Gl::load_with(|s| {
        video_subsystem.gl_get_proc_address(s)
            as *const std::os::raw::c_void
    });

    let vert_shader = render_gl::Shader::from_vert_source(
        &gl,
        &CString::new(include_str!("triangle.vert")).unwrap()
    ).unwrap();

    let frag_shader = render_gl::Shader::from_frag_source(
        &gl,
        &CString::new(include_str!("triangle.frag")).unwrap()
    ).unwrap();

    let shader_program = render_gl::Program::from_shaders(
        &gl,
        &[vert_shader, frag_shader]
    ).unwrap();

    shader_program.set_used();

    unsafe {
        gl.Viewport(0, 0, width, height);
        gl.ClearColor(0.3, 0.3, 0.5, 0.1);
    }

    let (vertices, indices) =
        canvas::rectangle(0.5, 0.5 * ((4/3) as f32), 0.5);

    println!("{:?}", vertices);
    println!("{:?}", indices);

    /*
    let vertices: Vec<f32> = vec![
        -0.5,  0.5, 0.0, 1.0, 0.0, 0.0,
        0.5,  0.5, 0.0, 0.0, 1.0, 0.0,
        0.5, -0.5, 0.0, 0.0, 0.0, 1.0,
        -0.5, -0.5, 0.0, 0.0, 0.0, 0.0,
    ];

    let indices: Vec<u32> = vec![
        0, 1, 3,
        1, 2, 3
    ];
    */

    let (mut vbo, mut vao, mut ebo) = (0, 0, 0);

    unsafe {
        gl.GenVertexArrays(1, &mut vao);
        gl.GenBuffers(1, &mut vbo);
        gl.GenBuffers(1, &mut ebo);
        gl.BindVertexArray(vao);

        gl.BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl.BufferData(
            gl::ARRAY_BUFFER,
            (vertices.len() * std::mem::size_of::<gl::types::GLuint>()) as gl::types::GLsizeiptr,
            vertices.as_ptr() as *const gl::types::GLvoid,
            gl::STATIC_DRAW
        );

        gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
        gl.BufferData(
            gl::ELEMENT_ARRAY_BUFFER,
            (indices.len() * std::mem::size_of::<gl::types::GLfloat>()) as gl::types::GLsizeiptr,
            indices.as_ptr() as *const gl::types::GLvoid,
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
    }

    let mut event_pump = sdl.event_pump().unwrap();

    'main: loop {
        for event in event_pump.poll_iter() {
            // Handle user input here
            match event {
                sdl2::event::Event::Quit {..}  => break 'main,
                _ => {},
            }
        }

        unsafe {
            gl.Clear(gl::COLOR_BUFFER_BIT);
            shader_program.set_used();
            gl.BindVertexArray(vao);

            /*
               gl.DrawArrays(
               gl::TRIANGLES, // mode
               0,             // triangle offset of start vertex
               3              // number of vertices to be rendered
               );
               */
            gl.DrawElements(
                gl::TRIANGLES,
                indices.len() as i32,
                gl::UNSIGNED_INT,
                std::ptr::null()
            );
        }

        // Render window contents here
        window.gl_swap_window();

    }
}
