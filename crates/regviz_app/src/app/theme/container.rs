use super::AppTheme;
use iced::widget::container;
use iced::{Background, Border};

#[derive(Debug, Default, Clone, Copy)]
pub enum ContainerClass {
    #[default]
    Default,
    Rounded,
    RoundedLarge,
    FilledWith(iced::Color),
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
            ContainerClass::FilledWith(color) => (Background::Color(*color), Border::default()),
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
