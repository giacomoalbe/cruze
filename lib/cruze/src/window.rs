use super::app;
use super::render_gl;
use super::layout_manager;
use super::canvas::{
    Size,
    Color,
};

use glutin::window::{WindowBuilder, WindowId};
use glutin::event::VirtualKeyCode;
use glutin::{
    ContextBuilder,
    ContextWrapper,
    PossiblyCurrent
};

pub struct Widget {
    pub size: Size<f32>,
    pub position: lyon::math::Point,
    pub color: Color,
    pub flex: f32,
}

impl Widget {
    pub fn new(color: Color, flex: f32) -> Widget {
        Widget {
            size: Size::new(0.0, 0.0),
            position: lyon::math::point(0.0, 0.0),
            color,
            flex
        }
    }

    pub fn set_size(&mut self, size: Size<f32>) {
        self.size = size;
    }

    pub fn set_position(&mut self, position: lyon::math::Point) {
        self.position = position;
    }
}

pub struct Window {
    pub id: WindowId,
    pub context: ContextWrapper<PossiblyCurrent, glutin::window::Window>,
    pub children: Vec<Widget>,
    size: glutin::dpi::LogicalSize,
    renderer: render_gl::Renderer,
    layout: layout_manager::LayoutBuilder,
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
            .with_multisampling(8)
            .with_vsync(true)
            .build_windowed(wb, &app.el)
            .unwrap();

        let context = unsafe {
            context
                .make_current()
                .unwrap()
        };

        let window_id = context.window().id();

        let gl = gl::Gl::load(&context.context());

        let window_size = glutin::dpi::LogicalSize {
            width: width as f64,
            height: height as f64
        };

        let mut renderer = render_gl::Renderer::new(&gl);

        let layout = layout_manager::LayoutBuilder::new();

        let mut window = Window {
            children: vec![],
            size: window_size,
            renderer: renderer,
            layout: layout,
            context: context,
            id: window_id,
        };

        window.generate_content();

        window
    }

    pub fn generate_content(&mut self) {
        self.children.push(Widget::new(Color::from_rgb(1.0, 0.0, 0.0), 2.0));
        self.children.push(Widget::new(Color::from_rgb(0.0, 1.0, 0.0), 1.0));
        self.children.push(Widget::new(Color::from_rgb(0.0, 0.0, 1.0), 1.0));
        self.children.push(Widget::new(Color::from_rgb(1.0, 1.0, 1.0), 1.0));

        self.renderer.resize(self.size, &self.children);
    }

    pub fn draw(&mut self) {
        self.renderer.draw();

        self.context
            .swap_buffers()
            .unwrap();
    }

    pub fn resize(&mut self, size: glutin::dpi::LogicalSize) {
        self.size = size;

        let children_pos_loc = self.layout.build(self.size, &self.children);

        for (index, child) in self.children.iter_mut().enumerate() {
            let child_pos_loc = children_pos_loc.get(index).unwrap();

            child.set_size(child_pos_loc.size);
            child.set_position(child_pos_loc.position);
        }

        self.renderer.resize(size, &self.children);
    }

    pub fn send_key(&mut self, key: VirtualKeyCode) {
        match key {
                VirtualKeyCode::Left |
                VirtualKeyCode::Right |
                VirtualKeyCode::Up |
                VirtualKeyCode::Down |
                VirtualKeyCode::R |
                VirtualKeyCode::D |
                VirtualKeyCode::S |
                VirtualKeyCode::X
             => (),//self.renderer.send_key(key),
            _ => println!("Do nothing")
        };

        self.draw();
    }
}
