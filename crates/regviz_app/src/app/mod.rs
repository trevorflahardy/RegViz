mod constants;
pub mod message;
mod parser;
mod simulation;
mod state;
pub mod theme;
mod update;
mod view;

// Re-export main types for convenience
pub use state::App;

use iced::Font;

/// Font used for all text rendered on the graph canvas and other parts of the app.
///
/// Web builds do not have access to system fonts, so we explicitly request the
/// bundled Fira Sans family provided by the `iced` crate and include it
pub const APP_FONT: Font = Font::with_name("Fira Sans");
