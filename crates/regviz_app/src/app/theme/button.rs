use super::AppTheme;
use iced::widget::button;

const BUTTON_DARK_PRIMARY: button::Style = button::Style {
    background: Some(iced::Background::Color(iced::Color::from_rgb(
        44.0 / 255.0,
        46.0 / 255.0,
        44.0 / 255.0,
    ))),
    text_color: AppTheme::TEXT_PRIMARY_DARK,
    border: iced::Border {
        color: iced::Color::from_rgb(0_f32, 0_f32, 0_f32),
        width: 0_f32,
        radius: iced::border::Radius {
            top_left: 15.0,
            top_right: 15.0,
            bottom_left: 15.0,
            bottom_right: 15.0,
        },
    },
    shadow: iced::Shadow {
        color: iced::Color::from_rgba(0_f32, 0_f32, 0_f32, 0.2),
        offset: iced::Vector::new(0.0, 2.0),
        blur_radius: 4.0,
    },
    snap: false,
};

const BUTTON_DARK_SECONDARY: button::Style = button::Style {
    background: Some(iced::Background::Color(iced::Color::from_rgb(
        64.0 / 255.0,
        66.0 / 255.0,
        64.0 / 255.0,
    ))),
    ..BUTTON_DARK_PRIMARY
};

const BUTTON_DARK_DANGER: button::Style = button::Style {
    background: Some(iced::Background::Color(iced::Color::from_rgb(
        150.0 / 255.0,
        30.0 / 255.0,
        30.0 / 255.0,
    ))),
    ..BUTTON_DARK_PRIMARY
};

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
            ButtonClass::Primary => BUTTON_DARK_PRIMARY,
            ButtonClass::Secondary => BUTTON_DARK_SECONDARY,
            ButtonClass::Danger => BUTTON_DARK_DANGER,
        };

        // adjust opacity on hover/press
        if status == button::Status::Hovered {
            style.background = style.background.map(|bg| bg.scale_alpha(0.9_f32));
        }
        if status == button::Status::Pressed {
            style.background = style.background.map(|bg| bg.scale_alpha(0.8_f32));
        }

        style
    }
}
