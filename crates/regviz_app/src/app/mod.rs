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

pub const APP_FONT: Font = Font::with_name("Fira Sans");
