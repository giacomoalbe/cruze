use super::window;

use std::collections::HashMap;

use glutin::window::WindowId;
use glutin::event::{Event, WindowEvent};
use glutin::event_loop::ControlFlow;

pub struct App {
    pub el: glutin::event_loop::EventLoop<()>,
    windows: HashMap<WindowId, window::Window>
}

impl App {
    pub fn new() -> App {
        let el = glutin::event_loop::EventLoop::new();
        let windows = HashMap::new();

        App {
            el: el,
            windows: windows
        }
    }

    pub fn run(self) {
        let windows = self.windows;

        for (id, window) in windows.iter() {
            // Set only if window is visible
            window.draw();
        }

        self.el.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;

            match event {
                Event::LoopDestroyed => return,
                Event::WindowEvent { event, window_id } => {
                    let window = windows.get(&window_id);

                    match window {
                        Some(window) => match event {
                                WindowEvent::Resized(logical_size) => {
                                    window.resize(logical_size);
                                },
                                WindowEvent::RedrawRequested => {
                                    window.draw();
                                },
                                WindowEvent::CloseRequested => {
                                    *control_flow = ControlFlow::Exit
                                },
                                _ => (),
                        },
                        _ => {
                            println!("{:?}", event);
                            println!("Something went wrong...")
                        }
                    };
                }
                _ => (),
            }
        });
    }

    pub fn add_window(&mut self, window: window::Window) {
        self.windows.insert(window.id, window);
    }
}
