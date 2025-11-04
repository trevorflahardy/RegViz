use super::AppTheme;
use iced::{Background, Color, Vector, widget::button};

const BUTTON_RADIUS: f32 = 8.0;

fn lighten(color: Color, amount: f32) -> Color {
    let amount = amount.clamp(0.0, 1.0);
    Color::from_rgba(
        color.r + (1.0 - color.r) * amount,
        color.g + (1.0 - color.g) * amount,
        color.b + (1.0 - color.b) * amount,
        color.a,
    )
}

fn darken(color: Color, amount: f32) -> Color {
    let amount = amount.clamp(0.0, 1.0);
    Color::from_rgba(
        color.r * (1.0 - amount),
        color.g * (1.0 - amount),
        color.b * (1.0 - amount),
        color.a,
    )
}

fn base_style(background: Color, text: Color, border: Color) -> button::Style {
    button::Style {
        background: Some(Background::Color(background)),
        text_color: text,
        border: iced::Border {
            color: border,
            width: 1.0,
            radius: iced::border::Radius::from(BUTTON_RADIUS),
        },
        shadow: iced::Shadow {
            color: Color::TRANSPARENT,
            offset: Vector::new(0.0, 0.0),
            blur_radius: 0.0,
        },
        snap: false,
    }
}

fn primary_style(theme: &AppTheme) -> button::Style {
    let accent = theme.accent();
    base_style(accent, theme.text_primary(), darken(accent, 0.3))
}

fn secondary_style(theme: &AppTheme) -> button::Style {
    let bg = theme.bg_high();
    base_style(bg, theme.text_primary(), darken(bg, 0.25))
}

fn danger_style(theme: &AppTheme) -> button::Style {
    let danger = theme.error();
    base_style(danger, theme.text_primary(), darken(danger, 0.2))
}

#[derive(Debug, Default, Clone)]
pub enum ButtonClass {
    #[default]
    Primary,
    Secondary,
    Danger,
}

// Allow closures to be used when calling `.style()` on a button.
// Without this, you must pass a ButtonClass variant directly.
impl<'a> From<button::StyleFn<'a, AppTheme>> for ButtonClass {
    fn from(_fn: button::StyleFn<'a, AppTheme>) -> Self {
        ButtonClass::Primary
    }
}

impl button::Catalog for AppTheme {
    type Class<'a> = ButtonClass;

    fn default<'a>() -> Self::Class<'a> {
        ButtonClass::Primary
    }

    fn style(&self, class: &Self::Class<'_>, status: button::Status) -> button::Style {
        let mut style: button::Style = match class {
            ButtonClass::Primary => primary_style(self),
            ButtonClass::Secondary => secondary_style(self),
            ButtonClass::Danger => danger_style(self),
        };

        match status {
            button::Status::Hovered => {
                if let Some(Background::Color(color)) = style.background {
                    style.background = Some(Background::Color(lighten(color, 0.08)));
                }
            }
            button::Status::Pressed => {
                if let Some(Background::Color(color)) = style.background {
                    style.background = Some(Background::Color(darken(color, 0.1)));
                }
            }
            button::Status::Disabled => {
                if let Some(Background::Color(color)) = style.background {
                    style.background = Some(Background::Color(lighten(color, 0.35)));
                }
                style.text_color = self.text_dim();
                style.border.color = lighten(style.border.color, 0.35);
            }
            button::Status::Active => {}
        };

        style
    }
}
