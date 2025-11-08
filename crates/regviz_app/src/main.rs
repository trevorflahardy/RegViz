/// RegViz - Regular Expression Visualizer
///
/// This application provides interactive visualizations of regular expressions,
/// allowing users to see both the parse tree (AST) and the non-deterministic
/// finite automaton (NFA) representation.
///
/// # Features
///
/// - **Parse Tree**: Hierarchical tree layout showing operator precedence
/// - **NFA**: State machine visualization with configurable bounding boxes
/// - **Interactive**: Zoom in/out, toggle elements, switch between views
/// - **Real-time**: Immediate feedback as you type regex patterns
mod app;
mod graph;

use app::App;
use iced::{Task, application};

/// Initializes debug tracing for development builds.
///
/// This function sets up structured logging using the `tracing` crate,
/// which helps with debugging and understanding program flow. The trace
/// level can be controlled via the `RUST_LOG` environment variable.
///
/// Only compiled in debug builds for performance reasons.
#[cfg(all(debug_assertions, not(target_arch = "wasm32")))]
fn init_tracing() {
    use tracing_subscriber::EnvFilter;

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .pretty()
        .try_init()
        .ok();
}

/// Application entry point.
///
/// Initializes tracing (in debug mode) and starts the Iced event loop
/// with the RegViz application.
fn main() -> iced::Result {
    // Debug assertions and not wasm32
    #[cfg(all(debug_assertions, not(target_arch = "wasm32")))]
    init_tracing();

    #[cfg(target_arch = "wasm32")]
    {
        console_error_panic_hook::set_once();
        console_log::init_with_level(log::Level::Debug).expect("could not initialize logger");
    }

    let app = application(|| (App::default(), Task::none()), App::update, App::view)
        .theme(|state: &App| Some(state.theme))
        .antialiasing(true)
        .decorations(true)
        .title(|_: &App| String::from("RegViz - Regular Expression Visualizer"))
        // Load embedded fonts for native builds (WASM loads via CSS)
        .font(app::INTER_REGULAR)
        .font(app::INTER_MEDIUM)
        .font(app::INTER_SEMIBOLD)
        .font(app::JETBRAINS_MONO_REGULAR)
        .font(app::JETBRAINS_MONO_MEDIUM)
        .default_font(app::APP_FONT);

    app.run()
}
