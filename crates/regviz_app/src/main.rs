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
use iced::{Program, application};

/// Initializes debug tracing for development builds.
///
/// This function sets up structured logging using the `tracing` crate,
/// which helps with debugging and understanding program flow. The trace
/// level can be controlled via the `RUST_LOG` environment variable.
///
/// Only compiled in debug builds for performance reasons.
#[cfg(debug_assertions)]
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
    #[cfg(debug_assertions)]
    init_tracing();

    // Iced does not expose a way to create an Application from
    // a Program directly, but we need this for our custom Theme implementation.
    // We'll create a mock Application instance using the builtin, manually
    // overwrite the 'raw' attribute to use our App instead.
    let app = App::default();
    let settings = app.settings().clone();
    let window_settings = app.window().clone().unwrap_or_default();

    application(move || app.boot(), App::update, App::view)
        .antialiasing(settings.antialiasing)
        .window(window_settings)
        .centered()
        .transparent(true)
        .run()
}
