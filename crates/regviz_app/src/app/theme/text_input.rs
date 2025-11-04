use super::AppTheme;
use iced::widget::text_input;
use iced::{Background, Border, Color};

const INPUT_RADIUS: f32 = 8.0;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum TextInputClass {
    #[default]
    Default,
    Invalid,
}

impl text_input::Catalog for AppTheme {
    type Class<'a> = TextInputClass;

    fn default<'a>() -> Self::Class<'a> {
        TextInputClass::Default
    }

    fn style(&self, class: &Self::Class<'_>, status: text_input::Status) -> text_input::Style {
        let mut base = text_input::Style {
            background: Background::Color(self.bg_mid()),
            border: Border {
                color: self.bg_high(),
                width: 1.0,
                radius: INPUT_RADIUS.into(),
            },
            icon: self.text_secondary(),
            placeholder: self.text_secondary(),
            value: self.text_primary(),
            selection: self.accent(),
        };

        let is_invalid = matches!(class, TextInputClass::Invalid);

        match status {
            text_input::Status::Disabled => {
                base.background = Background::Color(self.bg_low());
                base.value = self.text_dim();
                base.placeholder = self.text_dim();
                base.icon = self.text_dim();
                return base;
            }
            text_input::Status::Focused { .. } if !is_invalid => {
                base.border = Border {
                    color: self.accent(),
                    width: 2.0,
                    radius: INPUT_RADIUS.into(),
                };
                return base;
            }
            text_input::Status::Hovered if !is_invalid => {
                base.border = Border {
                    color: self.accent_dim(),
                    ..base.border
                };
                return base;
            }
            _ => {}
        }

        if is_invalid {
            base.border = Border {
                color: self.error(),
                width: 2.0,
                radius: INPUT_RADIUS.into(),
            };
            base.placeholder = self.text_secondary();
            base.background = Background::Color(shade(self.bg_mid(), 0.08));
        }

        base
    }
}

fn shade(color: Color, amount: f32) -> Color {
    let amount = amount.clamp(0.0, 1.0);
    Color::from_rgba(
        color.r + (1.0 - color.r) * amount,
        color.g + (1.0 - color.g) * amount,
        color.b + (1.0 - color.b) * amount,
        color.a,
    )
}
