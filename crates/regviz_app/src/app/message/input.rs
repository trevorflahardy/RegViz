/// Messages originating from the regex input controls.
#[derive(Debug, Clone)]
pub enum InputMessage {
    /// User changed the regex input text.
    Changed(String),
}
