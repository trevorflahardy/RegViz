use super::AppTheme;
use iced::widget::pane_grid;
use iced::{Background, Border};

impl pane_grid::Catalog for AppTheme {
    type Class<'a> = ();

    fn default<'a>() -> <Self as pane_grid::Catalog>::Class<'a> {
        ()
    }

    fn style(&self, _class: &<Self as pane_grid::Catalog>::Class<'_>) -> pane_grid::Style {
        pane_grid::Style {
            hovered_region: pane_grid::Highlight {
                background: Background::Color(self.bg_high()),
                border: Border {
                    color: self.accent(),
                    width: 2.0,
                    ..Default::default()
                },
            },
            picked_split: pane_grid::Line {
                color: self.accent(),
                width: 4.0,
            },
            hovered_split: pane_grid::Line {
                color: self.accent_dim(),
                width: 2.0,
            },
        }
    }
}
