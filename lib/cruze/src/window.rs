use super::app;
use super::render_gl;
use super::layout_manager;
use super::font_manager;

use super::canvas::{
    Size,
    Color,
};

use super::widgets::{
    Orientation,
    Alignment,
    Rect,
    Label,
    Widget,
    WidgetOptions
};

use glutin::window::{WindowBuilder, WindowId};
use glutin::event::{
    VirtualKeyCode,
    ElementState,
    MouseButton
};

use glutin::{
    ContextBuilder,
    ContextWrapper,
    PossiblyCurrent
};

pub struct Window {
    pub id: WindowId,
    pub context: ContextWrapper<PossiblyCurrent, glutin::window::Window>,
    pub children: Vec<Box<dyn Widget>>,
    mouse_x: f64,
    mouse_y: f64,
    size: glutin::dpi::LogicalSize,
    renderer: render_gl::Renderer,
    layout: layout_manager::LayoutBuilder,
    font_manager: font_manager::FontManager,
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

        let renderer = render_gl::Renderer::new(&gl);
        let layout = layout_manager::LayoutBuilder::new();
        let font_manager = font_manager::FontManager::new();

        let mut window = Window {
            children: vec![],
            size: window_size,
            renderer: renderer,
            font_manager: font_manager,
            layout: layout,
            context: context,
            id: window_id,
            mouse_x: 0.0,
            mouse_y: 0.0,
        };

        window.generate_content();

        window
    }

    pub fn generate_content(&mut self) {
        self.children.push(
            Rect::new(
                WidgetOptions {
                    id: "main_cont".to_string(),
                    orientation: Orientation::Column,
                    ..Default::default()
                },
                vec![
                    Rect::new(
                        WidgetOptions {
                            id: "top_bar".to_string(),
                            height: stretch::style::Dimension::Points(60.0),
                            horizontal_align: Alignment::Center,
                            vertical_align: Alignment::Center,
                            color: Color::from_rgb(0.1, 0.1, 0.1),
                            ..Default::default()
                        },
                        vec![
                            Label::new(
                                WidgetOptions {
                                    font_size: 16.0,
                                    ..Default::default()
                                },
                                "Top Bar".to_string()
                            )
                        ]
                    ),
                    Rect::new(
                        WidgetOptions {
                            id: "main_content".to_string(),
                            flex: 1.0,
                            color: Color::from_rgb(0.9, 0.9, 0.9),
                            ..Default::default()
                        },
                        vec![
                            Rect::new(
                                WidgetOptions {
                                    padding: WidgetOptions::uniform_padding(5.0),
                                    id: "left_side".to_string(),
                                    color: Color::from_rgb(0.7, 0.7, 0.7),
                                    width: stretch::style::Dimension::Points(250.0),
                                    vertical_align: Alignment::Center,
                                    horizontal_align: Alignment::Center,
                                    ..Default::default()
                                },
                                vec![
                                    Label::new(
                                        WidgetOptions {
                                            font_size: 21.0,
                                            ..Default::default()
                                        },
                                        "Left Side".to_string(),
                                    ),
                                ]
                            ),
                            Rect::new(
                                WidgetOptions {
                                    id: "canvas".to_string(),
                                    flex: 1.0,
                                    vertical_align: Alignment::Center,
                                    horizontal_align: Alignment::Center,
                                    ..Default::default()
                                },
                                vec![
                                    Label::new(
                                        WidgetOptions {
                                            font_size: 21.0,
                                            color: Color::from_rgb(0.0, 0.0, 0.0),
                                            ..Default::default()
                                        },
                                        "Canvas".to_string(),
                                    )
                                ]
                            ),
                            Rect::new(
                                WidgetOptions {
                                    padding: WidgetOptions::uniform_padding(5.0),
                                    id: "right_side".to_string(),
                                    color: Color::from_rgb(0.7, 0.7, 0.7),
                                    width: stretch::style::Dimension::Points(250.0),
                                    vertical_align: Alignment::Center,
                                    horizontal_align: Alignment::Center,
                                    ..Default::default()
                                },
                                vec![
                                    Label::new(
                                        WidgetOptions {
                                            font_size: 21.0,
                                            vertical_align: Alignment::Center,
                                            horizontal_align: Alignment::Center,
                                            ..Default::default()
                                        },
                                        "Right Side".to_string(),
                                    )
                                ]
                            ),
                        ]
                    ),
                    Rect::new(
                        WidgetOptions {
                            id: "bottom_bar".to_string(),
                            height: stretch::style::Dimension::Points(60.0),
                            horizontal_align: Alignment::Center,
                            vertical_align: Alignment::Center,
                            color: Color::from_rgb(0.1, 0.1, 0.1),
                            ..Default::default()
                        },
                        vec![
                            Label::new(
                                WidgetOptions {
                                    font_size: 16.0,
                                    ..Default::default()
                                },
                                "Bottom Bar".to_string()
                            )
                        ]
                    )
                ]
            )
        );

        self.renderer.resize(self.size, &self.children, &mut self.font_manager);
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

        self.layout.build(self.size, &mut self.children, &mut self.font_manager);

        self.renderer.resize(size, &self.children, &mut self.font_manager);
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

    pub fn send_mouse_input(&mut self, state: ElementState, button: MouseButton) {
        println!("Button: {:?} has been {:?} at [{}, {}]", button, state, self.mouse_x, self.mouse_y);
    }

    pub fn set_cursor_position(&mut self,x: f64, y: f64) {
        self.mouse_x = x;
        self.mouse_y = y;
    }

    pub fn get_size(&self) -> glutin::dpi::LogicalSize {
        self.size
    }
}
