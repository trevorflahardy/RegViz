use super::AppTheme;
use iced::widget::container;
use iced::{Background, Border, Color};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ContainerClass {
    #[default]
    Default,
    Rounded,
    RoundedLarge,
    /// Semi-transparent background (90% opaque) for glass effect
    GlassEffect,
}

impl<'a> From<container::StyleFn<'a, AppTheme>> for ContainerClass {
    fn from(_fn: container::StyleFn<'a, AppTheme>) -> Self {
        ContainerClass::Default
    }
}

impl container::Catalog for AppTheme {
    type Class<'a> = ContainerClass;

    fn default<'a>() -> Self::Class<'a> {
        ContainerClass::Default
    }

    fn style(&self, class: &Self::Class<'_>) -> container::Style {
        let (background, border) = match class {
            ContainerClass::Default => (Background::Color(self.bg_low()), Border::default()),
            ContainerClass::Rounded => (
                Background::Color(self.bg_low()),
                Border {
                    radius: iced::border::Radius::new(10.0),
                    ..Border::default()
                },
            ),
            ContainerClass::RoundedLarge => (
                Background::Color(self.bg_low()),
                Border {
                    radius: iced::border::Radius::new(20.0),
                    width: 2.0,
                    color: self.text_primary().scale_alpha(0.3),
                },
            ),
            ContainerClass::GlassEffect => {
                // Create a semi-transparent background (90% opaque = 0.9 alpha)
                // For more transparency, decrease this value (e.g., 0.85 = 85% opaque, 15% transparent)
                const GLASS_OPACITY: f32 = 0.5;

                let base_color = self.bg_low();
                let glass_color = Color {
                    a: GLASS_OPACITY,
                    ..base_color
                };

                (
                    Background::Color(glass_color),
                    Border {
                        radius: iced::border::Radius::new(20.0),
                        width: 1.5,
                        color: self.text_primary().scale_alpha(0.2),
                    },
                )
            }
        };

        container::Style {
            background: Some(background),
            text_color: Some(self.text_primary()),
            border,
            shadow: Default::default(),
            snap: false,
        }
    }
}
