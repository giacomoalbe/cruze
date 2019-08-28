use super::window;

use std::collections::HashMap;

use glutin::window::WindowId;

use glutin::event::{
    Event,
    WindowEvent,
    KeyboardInput,
    //VirtualKeyCode,
    ElementState::Pressed
};

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
        let mut windows = self.windows;

        for (_id, window) in windows.iter_mut() {
            // Set only if window is visible
            window.resize(window.get_size());
            window.draw();
        }

        self.el.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;

            match event {
                Event::LoopDestroyed => return,
                Event::WindowEvent { event, window_id } => {
                    let window = windows
                        .get_mut(&window_id);

                    match window {
                        Some(window) => {
                            match event {
                                WindowEvent::Resized(logical_size) => {
                                    window.resize(logical_size);
                                },
                                WindowEvent::RedrawRequested => {
                                    window.draw();
                                },
                                WindowEvent::KeyboardInput { input, .. } => {
                                    match input.state {
                                        Pressed => {
                                            window.send_key(input.virtual_keycode.unwrap());
                                        },
                                        Released => {
                                        }
                                    }
                                },
                                WindowEvent::MouseInput { state, button, .. } => {
                                    window.send_mouse_input(state, button);
                                },
                                WindowEvent::CursorMoved { position, .. } => {
                                    window.set_cursor_position(position.x, position.y);
                                },
                                WindowEvent::CloseRequested => {
                                    *control_flow = ControlFlow::Exit
                                },
                                _ => (),
                            };
                        },
                        None => {
                            println!("Qualcosa è andato storto :(");
                        }
                    }
                },
                _ => (),
            }
        });
    }

    pub fn add_window(&mut self, window: window::Window) {
        self.windows.insert(window.id, window);
    }
}
