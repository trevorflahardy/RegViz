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
pub const APP_FONT: Font = Font::with_name("Inter 28pt");

/// Monospace font used for code, errors, and technical text.
pub const MONOSPACE: Font = Font::with_name("JetBrains Mono");

// Embed fonts for native builds (not needed for WASM)
pub const INTER_REGULAR: &[u8] = include_bytes!("../../public/fonts/Inter-Regular.ttf");

pub const INTER_MEDIUM: &[u8] = include_bytes!("../../public/fonts/Inter-Medium.ttf");

pub const INTER_SEMIBOLD: &[u8] = include_bytes!("../../public/fonts/Inter-SemiBold.ttf");

pub const JETBRAINS_MONO_REGULAR: &[u8] =
    include_bytes!("../../public/fonts/JetBrainsMono-Regular.ttf");

pub const JETBRAINS_MONO_MEDIUM: &[u8] =
    include_bytes!("../../public/fonts/JetBrainsMono-Medium.ttf");
