use iced::{
    Alignment, Element,
    widget::{button, row, text},
};

use crate::app::message::{Message, ViewMessage, ViewMode};
use crate::app::state::App;

/// Renders toggle buttons to switch between AST and NFA views.
pub fn render(app: &App) -> Element<'_, Message> {
    let ast_button = view_mode_button(ViewMode::Ast, app.view_mode == ViewMode::Ast);
    let nfa_button = view_mode_button(ViewMode::Nfa, app.view_mode == ViewMode::Nfa);

    row![text("View:").size(16), ast_button, nfa_button]
        .spacing(12)
        .align_y(Alignment::Center)
        .into()
}

fn view_mode_button(mode: ViewMode, is_active: bool) -> Element<'static, Message> {
    let mut label = mode.label().to_string();
    if is_active {
        label.push_str(" ✓");
    }

    button(text(label).size(16))
        .padding([6, 16])
        .on_press(Message::View(ViewMessage::ViewModeChanged(mode)))
        .into()
}
