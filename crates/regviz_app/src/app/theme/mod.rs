mod button;
mod colors;
mod container;
mod pane_grid;
mod slider;
mod text;
mod text_input;

use crate::app::{
    message::Message,
    theme::colors::{
        AMBER_500, GRAY_50, GRAY_100, GRAY_200, GRAY_300, GRAY_500, GRAY_950, GREEN_400, GREEN_500,
        RED_400, RED_500, SKY_500, SKY_800, SLATE_900,
    },
};
use iced::{Color, Element, theme};

// Re-exports for easier access in renderer modules
#[allow(unused_imports)]
pub use button::ButtonClass;
pub use container::ContainerClass;
#[allow(unused_imports)]
pub use text::{TextClass, TextSize};

pub type ElementType<'a> = Element<'a, Message, AppTheme>;

#[derive(Default, Copy, Clone, PartialEq, Eq, Hash)]
pub enum AppTheme {
    #[default]
    Dark,
    // TODO: Light theme as well
}

impl AppTheme {
    const PRIMARY: Color = SKY_500;

    const DARK_COLOR_PALETTE: theme::Palette = theme::Palette {
        background: SLATE_900,
        text: GRAY_50,
        primary: Self::PRIMARY,
        success: GREEN_500,
        warning: AMBER_500,
        danger: RED_500,
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
            background_color: SLATE_900,
            text_color: GRAY_50,
        }
    }

    fn palette(&self) -> Option<theme::Palette> {
        match self {
            AppTheme::Dark => Some(Self::DARK_COLOR_PALETTE),
        }
    }
}

impl From<AppTheme> for iced::Theme {
    fn from(theme: AppTheme) -> Self {
        match theme {
            AppTheme::Dark => {
                iced::Theme::custom("REGVIZ_DARK".to_string(), AppTheme::DARK_COLOR_PALETTE)
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
            AppTheme::Dark => GRAY_50,
        }
    }

    pub fn text_primary_inverse(&self) -> Color {
        match self {
            AppTheme::Dark => GRAY_950,
        }
    }

    pub fn text_secondary(&self) -> Color {
        match self {
            AppTheme::Dark => GRAY_300,
        }
    }

    pub fn text_dim(&self) -> Color {
        match self {
            AppTheme::Dark => GRAY_200,
        }
    }

    // Accent colors
    pub fn accent(&self) -> Color {
        match self {
            AppTheme::Dark => Self::PRIMARY,
        }
    }

    pub fn accent_dim(&self) -> Color {
        match self {
            AppTheme::Dark => SKY_800,
        }
    }

    // Graph visualization colors
    pub fn graph_node_default(&self) -> Color {
        match self {
            AppTheme::Dark => GRAY_50,
        }
    }

    pub fn graph_node_active(&self) -> Color {
        match self {
            AppTheme::Dark => GREEN_500,
        }
    }

    pub fn graph_node_rejected(&self) -> Color {
        match self {
            AppTheme::Dark => RED_500,
        }
    }

    pub fn graph_node_outline_default(&self) -> Color {
        match self {
            AppTheme::Dark => GRAY_500,
        }
    }

    pub fn graph_node_outline_active(&self) -> Color {
        match self {
            AppTheme::Dark => GREEN_400,
        }
    }

    pub fn graph_node_outline_rejected(&self) -> Color {
        match self {
            AppTheme::Dark => RED_400,
        }
    }

    pub fn graph_edge_default(&self) -> Color {
        match self {
            AppTheme::Dark => GRAY_200,
        }
    }

    pub fn graph_edge_active(&self) -> Color {
        match self {
            AppTheme::Dark => GRAY_100,
        }
    }

    // User feedback colors
    pub fn success(&self) -> Color {
        match self {
            AppTheme::Dark => Self::DARK_COLOR_PALETTE.success,
        }
    }

    pub fn warning(&self) -> Color {
        match self {
            AppTheme::Dark => Self::DARK_COLOR_PALETTE.warning,
        }
    }

    pub fn error(&self) -> Color {
        match self {
            AppTheme::Dark => Self::DARK_COLOR_PALETTE.danger,
        }
    }
}
