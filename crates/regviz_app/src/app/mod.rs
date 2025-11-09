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

macro_rules! define_font {
    // Grab the byte path to the font and load it only if the embed-fonts
    // field is passed. Otherwise, load an empty list
    ($name:ident, $file:expr) => {
        #[cfg(feature = "embed-fonts")]
        pub const $name: &[u8] = include_bytes!(concat!("./../../public/fonts/", $file, ".ttf"));
    };
}

#[cfg(feature = "embed-fonts")]
pub const APP_FONT: Font = Font::with_name("Inter 28pt");

#[cfg(not(feature = "embed-fonts"))]
pub const APP_FONT: Font = Font::DEFAULT;

define_font!(INTER_REGULAR, "Inter-Regular");
define_font!(INTER_MEDIUM, "Inter-Medium");
define_font!(INTER_SEMIBOLD, "Inter-Semibold");
