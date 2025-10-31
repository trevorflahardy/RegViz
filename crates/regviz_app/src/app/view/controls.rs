use iced::{
    Alignment, Element, Length,
    widget::{button, column, row, slider, text},
};
use regviz_core::core::automaton::BoxKind;

use crate::app::constants::{MAX_ZOOM_FACTOR, MIN_ZOOM_FACTOR};
use crate::app::message::{Message, ViewMessage};
use crate::app::state::App;

/// Renders buttons for toggling bounding box visibility (NFA only).
pub fn bounding_boxes(app: &App) -> Element<'_, Message> {
    let toggles = row![
        box_toggle_button(app, BoxKind::Literal, "Literal"),
        box_toggle_button(app, BoxKind::Concat, "Concat"),
        box_toggle_button(app, BoxKind::Alternation, "Alt"),
        box_toggle_button(app, BoxKind::KleeneStar, "Star"),
        box_toggle_button(app, BoxKind::KleenePlus, "Plus"),
        box_toggle_button(app, BoxKind::Optional, "Optional"),
    ]
    .spacing(8);

    column![text("Bounding Boxes:").size(14), toggles]
        .spacing(4)
        .into()
}

/// Renders zoom controls with slider and percentage display.
pub fn zoom(app: &App) -> Element<'_, Message> {
    let zoom_percentage = (app.zoom_factor * 100.0).round() as i32;
    let zoom_display = text(format!("Zoom: {zoom_percentage}%")).size(14);

    let zoom_slider = slider(
        MIN_ZOOM_FACTOR..=MAX_ZOOM_FACTOR,
        app.zoom_factor,
        |value| Message::View(ViewMessage::ZoomChanged(value)),
    )
    .step(0.05)
    .width(Length::Fixed(200.0));

    row![zoom_display, zoom_slider]
        .spacing(12)
        .align_y(Alignment::Center)
        .into()
}

fn box_toggle_button(app: &App, kind: BoxKind, label: &'static str) -> Element<'static, Message> {
    let is_visible = app.box_visibility.is_visible(kind);
    let button_text = format!("{}: {}", label, if is_visible { "✓" } else { "✗" });

    button(text(button_text).size(14))
        .padding([4, 10])
        .on_press(Message::View(ViewMessage::ToggleBox(kind)))
        .into()
}
