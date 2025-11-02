#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn start() {
    // Better panic messages in the browser console
    console_error_panic_hook::set_once();

    // Route Rust logs to the browser console (if `log` is used elsewhere)
    let _ = console_log::init_with_level(log::Level::Info);

    // Run the Iced application
    let _ = iced::run(
        "RegViz - Regular Expression Visualizer",
        app::App::update,
        app::App::view,
    );
}
