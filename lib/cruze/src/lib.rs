extern crate sdl2;
extern crate gl;

pub fn main_loop() {
    println!("Main loop :)");
    loop {}
}

pub struct Window {
    width: u32,
    height: u32,
    window: sdl2::video::Window
}

impl Window {
    pub fn new(width: u32, height: u32) -> Window {
        let sdl = sdl2::init().unwrap();

        let video_subsystem = sdl.video().unwrap();

        // Setting OpenGL Attributes
        let gl_attr = video_subsystem.gl_attr();

        gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
        gl_attr.set_context_version(4, 5);

        let window = video_subsystem
            .window("OpenGL Rust", width, height)
            .opengl()           // Add OpenGL flag
            .resizable()
            .build()
            .unwrap();

        let _gl_context = window.gl_create_context().unwrap();

        let gl = gl::Gl::load_with(|s| {
            video_subsystem.gl_get_proc_address(s)
                as *const std::os::raw::c_void
        });

        Window {
            width,
            height,
        }
    }

    pub fn show(&self) {
        println!("Showing the Window");
    }
}
