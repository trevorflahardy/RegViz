use super::AppTheme;
use iced::widget::text;

impl text::Catalog for AppTheme {
    type Class<'a> = ();

    fn default<'a>() -> Self::Class<'a> {
        ()
    }

    fn style(&self, _class: &Self::Class<'_>) -> text::Style {
        text::Style {
            color: Some(AppTheme::TEXT_DARK),
        }
    }
}
