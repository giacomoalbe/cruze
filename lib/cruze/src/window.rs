use super::app;
use super::render_gl;
use super::canvas;

use glutin::window::{WindowBuilder, WindowId};
use glutin::{
    ContextBuilder,
    ContextWrapper,
    PossiblyCurrent
};

use gl::Gl;

pub struct Window {
    pub id: WindowId,
    pub context: ContextWrapper<PossiblyCurrent, glutin::window::Window>,
    renderer: render_gl::Renderer,
    gl: gl::Gl,
    width: u32,
    height: u32,
    title: &'static str,
}

impl Window {
    pub fn new(app: &mut app::App, width: u32, height: u32, title: &'static str) -> Window {
        let wb = WindowBuilder::new()
            .with_title(title)
            .with_inner_size(glutin::dpi::LogicalSize {
                width: width as f64,
                height: height as f64
            })
            .with_min_inner_size(glutin::dpi::LogicalSize {
                width: 300 as f64,
                height: 300 as f64
            });

        let context = ContextBuilder::new()
            .build_windowed(wb, &app.el)
            .unwrap();

        let context = unsafe {
            context
                .make_current()
                .unwrap()
        };

        let window_id = context.window().id();

        let gl = gl::Gl::load(&context.context());

        let mut renderer = render_gl::Renderer::new(&gl);

        renderer.generate_geometry();

        Window {
            renderer: renderer,
            gl: gl,
            context: context,
            id: window_id,
            width,
            height,
            title
        }
    }

    pub fn draw(&self) {
        self.renderer.draw();
        self.context
            .swap_buffers()
            .unwrap();
    }

    pub fn resize(&self, size: glutin::dpi::LogicalSize) {
        self.renderer.resize(size);
    }
}