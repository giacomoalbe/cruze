extern crate cruze;

use cruze::{window, app};

fn main() {
    let width: u32 = 800;
    let height: u32 = 600;

    let mut app = app::App::new();

    let window = window::Window::new(
        &mut app,
        width,
        height,
        "Come è bello, come da gioia"
    );

    app.add_window(window);

    app.run();
}
