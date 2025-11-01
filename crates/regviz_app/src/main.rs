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

    application(|| (App::default(), Task::none()), App::update, App::view)
        .theme(|state: &App| Some(state.theme))
        .antialiasing(true)
        // .centered() // Commented out: causes macOS objc2-foundation crash
        .transparent(true)
        .decorations(false)
        .title(|_: &App| String::from("RegViz - Regular Expression Visualizer"))
        .run()
}
