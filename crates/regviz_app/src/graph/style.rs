use iced::Color;
use regviz_core::core::automaton::BoxId;

/// Deterministically generates a pseudo-random color for a bounding box.
#[must_use]
pub fn color_for_box(id: BoxId) -> Color {
    let mut value = (id as u64 + 1).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    value ^= value >> 33;
    value = value.wrapping_mul(0xC2B2_AE35_0A97_0A4D);
    value ^= value >> 29;
    value = value.wrapping_mul(0x1656_67B1_9E37_9B97);

    let r = ((value >> 16) & 0xFF) as f32 / 255.0;
    let g = ((value >> 24) & 0xFF) as f32 / 255.0;
    let b = ((value >> 32) & 0xFF) as f32 / 255.0;

    Color::from_rgba(r, g, b, 0.25)
}
