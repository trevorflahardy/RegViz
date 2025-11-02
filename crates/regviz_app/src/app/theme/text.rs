use super::AppTheme;
use iced::{Pixels, widget::text};

#[allow(dead_code)]
pub enum TextSize {
    H1,
    H2,
    H3,
    Body,
    // Small body, for accents
    Small,
}

impl TextSize {
    pub fn to_size(&self) -> u16 {
        match self {
            TextSize::H1 => 24,
            TextSize::H2 => 20,
            TextSize::H3 => 18,
            TextSize::Body => 14,
            TextSize::Small => 12,
        }
    }
}

impl From<TextSize> for Pixels {
    fn from(val: TextSize) -> Self {
        Pixels::from(val.to_size() as u32)
    }
}

pub enum TextClass {
    Primary,
    Secondary,
    Success,
    Warning,
    Error,
}

impl<'a> From<text::StyleFn<'a, AppTheme>> for TextClass {
    fn from(_fn: text::StyleFn<'a, AppTheme>) -> Self {
        TextClass::Primary
    }
}

impl text::Catalog for AppTheme {
    type Class<'a> = TextClass;

    fn default<'a>() -> Self::Class<'a> {
        TextClass::Primary
    }

    fn style(&self, class: &Self::Class<'_>) -> text::Style {
        match class {
            TextClass::Primary => text::Style {
                color: Some(self.text_primary()),
            },
            TextClass::Secondary => text::Style {
                color: Some(self.text_secondary()),
            },
            TextClass::Success => text::Style {
                color: Some(self.success()),
            },
            TextClass::Warning => text::Style {
                color: Some(self.warning()),
            },
            TextClass::Error => text::Style {
                color: Some(self.error()),
            },
        }
    }
}
