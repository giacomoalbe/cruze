use super::app;
use super::render_gl;
use super::layout_manager;
use super::canvas::{
    Size,
    Color,
};

use super::widgets::{
    Col,
    Row,
    Rect,
    Widget,
    WidgetOptions
};

use glutin::window::{WindowBuilder, WindowId};
use glutin::event::VirtualKeyCode;
use glutin::{
    ContextBuilder,
    ContextWrapper,
    PossiblyCurrent
};

pub struct Window {
    pub id: WindowId,
    pub context: ContextWrapper<PossiblyCurrent, glutin::window::Window>,
    pub children: Vec<Box<dyn Widget>>,
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
        self.children.push(
            Col::new(
                WidgetOptions::default(),
                vec![
                    /*
                    Row::new(
                        WidgetOptions {
                            padding: WidgetOptions::uniform_padding(10.0),
                            ..Default::default()
                        },
                        vec![
                            Col::new(
                                WidgetOptions {
                                    margin: WidgetOptions::uniform_padding(5.0),
                                    color: Color::from_rgb(1.0, 0.0, 0.0),
                                    ..Default::default()
                                },
                                vec![]
                            ),
                            Col::new(
                                WidgetOptions {
                                    margin: WidgetOptions::uniform_padding(5.0),
                                    color: Color::from_rgb(0.0, 1.0, 0.0),
                                    ..Default::default()
                                },
                                vec![]
                            ),
                            Col::new(
                                WidgetOptions {
                                    margin: WidgetOptions::uniform_padding(5.0),
                                    color: Color::from_rgb(0.0, 0.0, 1.0),
                                    ..Default::default()
                                },
                                vec![]
                            ),
                        ]
                    ),
                    */
                    Row::new(
                        WidgetOptions {
                            padding: WidgetOptions::uniform_padding(10.0),
                            ..Default::default()
                        },
                        vec![
                            Rect::new(
                                WidgetOptions {
                                    radius: 15.0,
                                    size: WidgetOptions::percent(10.0),
                                    color: Color::from_rgb(0.5, 0.5, 0.5),
                                    ..Default::default()
                                }
                            ),
                            Rect::new(
                                WidgetOptions {
                                    radius: 15.0,
                                    color: Color::from_rgb(0.0, 1.0, 0.0),
                                    ..Default::default()
                                }
                            ),
                            Rect::new(
                                WidgetOptions {
                                    radius: 15.0,
                                    color: Color::from_rgb(0.0, 0.0, 1.0),
                                    ..Default::default()
                                }
                            ),
                        ]
                    )
                ]
            )
        );

        self.renderer.resize(self.size, &self.children);
    }

    pub fn draw(&mut self) {
        let _start_time = std::time::Instant::now();

        self.renderer.draw();

        //println!("Frame render (renderer): {}", _start_time.elapsed().as_micros());

        let _start_time = std::time::Instant::now();

        self.context
            .swap_buffers()
            .unwrap();

        //println!("Frame render (buffer swap): {}", _start_time.elapsed().as_micros());
    }

    pub fn resize(&mut self, size: glutin::dpi::LogicalSize) {
        self.size = size;

        self.layout.build(self.size, &mut self.children);

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

    pub fn get_size(&self) -> glutin::dpi::LogicalSize {
        self.size
    }
}
