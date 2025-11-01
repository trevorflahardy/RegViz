mod button;
mod container;
mod pane_grid;
mod slider;
mod text;
mod text_input;

use crate::app::message::Message;
use iced::{Color, Element, theme};

// Re-exports for easier access in renderer modules
pub use button::ButtonClass;
pub use container::ContainerClass;

pub type ElementType<'a> = Element<'a, Message, AppTheme>;

#[derive(Default, Copy, Clone, PartialEq, Eq, Hash)]
pub enum AppTheme {
    #[default]
    Dark,
    // TODO: Light theme as well
}

impl AppTheme {
    const TEXT_DARK: Color = Color::from_rgb(230.0 / 255.0, 230.0 / 255.0, 230.0 / 255.0);

    pub const THEME_DARK_PALETTE: theme::Palette = theme::Palette {
        background: iced::Color::from_rgb(19.0 / 255.0, 26.0 / 255.0, 32.0 / 255.0),
        text: iced::Color::from_rgb(230.0 / 255.0, 230.0 / 255.0, 230.0 / 255.0),
        ..theme::Palette::DARK
    };
}

impl theme::Base for AppTheme {
    fn default(_preference: theme::Mode) -> Self {
        // you could return Light or Dark depending on preference.
        // we only support Dark for now:
        Self::Dark
    }

    fn mode(&self) -> theme::Mode {
        theme::Mode::Dark
    }

    fn base(&self) -> iced::theme::Style {
        iced::theme::Style {
            background_color: Self::THEME_DARK_PALETTE.background,
            text_color: Self::TEXT_DARK,
        }
    }

    fn palette(&self) -> Option<theme::Palette> {
        match self {
            AppTheme::Dark => Some(Self::THEME_DARK_PALETTE),
        }
    }
}

impl From<AppTheme> for iced::Theme {
    fn from(theme: AppTheme) -> Self {
        match theme {
            AppTheme::Dark => {
                iced::Theme::custom("REGVIZ_DARK".to_string(), AppTheme::THEME_DARK_PALETTE)
            }
        }
    }
}

impl AppTheme {
    // Background colors
    pub fn bg_low(&self) -> Color {
        match self {
            AppTheme::Dark => Color::from_rgb(0.075, 0.102, 0.125),
        }
    }

    pub fn bg_mid(&self) -> Color {
        match self {
            AppTheme::Dark => Color::from_rgb(0.125, 0.152, 0.175),
        }
    }

    pub fn bg_high(&self) -> Color {
        match self {
            AppTheme::Dark => Color::from_rgb(0.175, 0.202, 0.225),
        }
    }

    // Text colors
    pub fn text_primary(&self) -> Color {
        match self {
            AppTheme::Dark => Self::TEXT_DARK,
        }
    }

    pub fn text_secondary(&self) -> Color {
        match self {
            AppTheme::Dark => Color::from_rgb(0.7, 0.7, 0.7),
        }
    }

    pub fn text_dim(&self) -> Color {
        match self {
            AppTheme::Dark => Color::from_rgb(0.5, 0.5, 0.5),
        }
    }

    // Accent colors
    pub fn accent(&self) -> Color {
        match self {
            AppTheme::Dark => Color::from_rgb(0.35, 0.65, 0.95),
        }
    }

    pub fn accent_dim(&self) -> Color {
        match self {
            AppTheme::Dark => Color::from_rgb(0.25, 0.45, 0.75),
        }
    }

    // Graph visualization colors
    pub fn graph_node_default(&self) -> Color {
        match self {
            AppTheme::Dark => Color::from_rgb(0.3, 0.3, 0.35),
        }
    }

    pub fn graph_node_active(&self) -> Color {
        match self {
            AppTheme::Dark => Color::from_rgb(0.35, 0.65, 0.95),
        }
    }

    pub fn graph_node_accept(&self) -> Color {
        match self {
            AppTheme::Dark => Color::from_rgb(0.2, 0.7, 0.4),
        }
    }

    pub fn graph_edge_default(&self) -> Color {
        match self {
            AppTheme::Dark => Color::from_rgb(0.5, 0.5, 0.5),
        }
    }

    pub fn graph_edge_active(&self) -> Color {
        match self {
            AppTheme::Dark => Color::from_rgb(0.95, 0.65, 0.35),
        }
    }
}
