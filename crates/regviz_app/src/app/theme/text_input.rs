use super::AppTheme;
use iced::widget::text_input;
use iced::{Background, Border};

impl text_input::Catalog for AppTheme {
    type Class<'a> = ();

    fn default<'a>() -> Self::Class<'a> {
        ()
    }

    fn style(&self, _class: &Self::Class<'_>, status: text_input::Status) -> text_input::Style {
        let base = text_input::Style {
            background: Background::Color(self.bg_mid()),
            border: Border {
                color: self.bg_high(),
                width: 1.0,
                radius: 4.0.into(),
            },
            icon: self.text_secondary(),
            placeholder: self.text_secondary(),
            value: self.text_primary(),
            selection: self.accent(),
        };

        match status {
            text_input::Status::Active => base,
            text_input::Status::Hovered => text_input::Style {
                border: Border {
                    color: self.accent_dim(),
                    ..base.border
                },
                ..base
            },
            text_input::Status::Focused { .. } => text_input::Style {
                border: Border {
                    color: self.accent(),
                    width: 2.0,
                    ..base.border
                },
                ..base
            },
            text_input::Status::Disabled => text_input::Style {
                background: Background::Color(self.bg_low()),
                value: self.text_dim(),
                ..base
            },
        }
    }
}
