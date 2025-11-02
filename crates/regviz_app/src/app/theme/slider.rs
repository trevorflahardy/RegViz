use super::AppTheme;
use iced::widget::slider;
use iced::{Border, Color};

impl slider::Catalog for AppTheme {
    type Class<'a> = ();

    fn default<'a>() -> Self::Class<'a> {}

    fn style(&self, _class: &Self::Class<'_>, status: slider::Status) -> slider::Style {
        let base = slider::Style {
            rail: slider::Rail {
                backgrounds: (
                    self.accent().into(),
                    Color {
                        a: 0.3,
                        ..self.accent()
                    }
                    .into(),
                ),
                width: 4.0,
                border: Border::default(),
            },
            handle: slider::Handle {
                shape: slider::HandleShape::Circle { radius: 8.0 },
                background: self.accent().into(),
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
            },
        };

        match status {
            slider::Status::Active => base,
            slider::Status::Hovered => slider::Style {
                handle: slider::Handle {
                    shape: slider::HandleShape::Circle { radius: 10.0 },
                    ..base.handle
                },
                ..base
            },
            slider::Status::Dragged => slider::Style {
                handle: slider::Handle {
                    background: Color {
                        a: 0.8,
                        ..self.accent()
                    }
                    .into(),
                    ..base.handle
                },
                ..base
            },
        }
    }
}
