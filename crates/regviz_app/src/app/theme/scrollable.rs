use super::AppTheme;
use iced::{Background, Color, widget::scrollable};

#[derive(Default)]
pub enum ScrollableClass {
    #[default]
    Default,
}

impl<'a> From<scrollable::StyleFn<'a, AppTheme>> for ScrollableClass {
    fn from(_fn: scrollable::StyleFn<'a, AppTheme>) -> Self {
        ScrollableClass::Default
    }
}

impl scrollable::Catalog for AppTheme {
    type Class<'a> = ScrollableClass;

    fn default<'a>() -> Self::Class<'a> {
        ScrollableClass::Default
    }

    fn style(&self, _class: &Self::Class<'_>, status: scrollable::Status) -> scrollable::Style {
        let scroller_color = match status {
            scrollable::Status::Active { .. } => self.bg_mid(),
            scrollable::Status::Hovered { .. } => self.bg_high(),
            scrollable::Status::Dragged { .. } => self.bg_low(),
        };

        scrollable::Style {
            container: Default::default(),
            vertical_rail: scrollable::Rail {
                background: Some(Background::Color(Color::TRANSPARENT)),
                border: Default::default(),
                scroller: scrollable::Scroller {
                    color: scroller_color,
                    border: Default::default(),
                },
            },
            horizontal_rail: scrollable::Rail {
                background: Some(Background::Color(Color::TRANSPARENT)),
                border: Default::default(),
                scroller: scrollable::Scroller {
                    color: scroller_color,
                    border: Default::default(),
                },
            },
            gap: None,
        }
    }
}
