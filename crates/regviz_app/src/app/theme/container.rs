use super::AppTheme;
use iced::widget::container;
use iced::{Background, Border};

impl container::Catalog for AppTheme {
    type Class<'a> = ();

    fn default<'a>() -> Self::Class<'a> {
        ()
    }

    fn style(&self, _class: &Self::Class<'_>) -> container::Style {
        container::Style {
            background: Some(Background::Color(self.bg_low())),
            text_color: Some(self.text_primary()),
            border: Border::default(),
            shadow: Default::default(),
            snap: false,
        }
    }
}
