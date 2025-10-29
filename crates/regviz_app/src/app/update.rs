use super::constants::{MAX_ZOOM_FACTOR, MIN_ZOOM_FACTOR};
use super::message::Message;
use super::state::App;

impl App {
    /// Handles incoming messages and updates application state accordingly.
    ///
    /// This is the main state transition function. It processes user actions
    /// and updates the app state in response.
    pub fn update(&mut self, message: Message) {
        match message {
            Message::InputChanged(input) => {
                self.handle_input_changed(input);
            }
            Message::ToggleBox(kind) => {
                self.handle_toggle_box(kind);
            }
            Message::ZoomChanged(value) => {
                self.handle_zoom_changed(value);
            }
            Message::ViewModeChanged(mode) => {
                self.handle_view_mode_changed(mode);
            }
        }
    }

    /// Updates the input text and re-parses the regex.
    fn handle_input_changed(&mut self, input: String) {
        self.input = input;
        self.lex_and_parse();
    }

    /// Toggles visibility of a specific bounding box type in the NFA view.
    fn handle_toggle_box(&mut self, kind: regviz_core::core::automaton::BoxKind) {
        self.box_visibility.toggle(kind);
    }

    /// Updates the zoom factor, clamping it to valid range.
    fn handle_zoom_changed(&mut self, value: f32) {
        self.zoom_factor = value.clamp(MIN_ZOOM_FACTOR, MAX_ZOOM_FACTOR);
    }

    /// Switches between AST and NFA visualization modes.
    fn handle_view_mode_changed(&mut self, mode: super::message::ViewMode) {
        self.view_mode = mode;
    }
}
