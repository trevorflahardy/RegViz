use std::fmt;

/// Identifier type for automaton states.
pub type StateId = u32;

/// Identifier type for bounding boxes surrounding states during visualization.
pub type BoxId = u32;

/// Labels describing the kind of transition between states.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EdgeLabel {
    /// Epsilon transition that consumes no input.
    Eps,
    /// Consumes a specific symbol.
    Sym(char),
}

impl From<EdgeLabel> for String {
    fn from(label: EdgeLabel) -> Self {
        match label {
            EdgeLabel::Eps => "ε".to_string(),
            EdgeLabel::Sym(c) => c.to_string(),
        }
    }
}

impl fmt::Display for EdgeLabel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EdgeLabel::Eps => write!(f, "ε"),
            EdgeLabel::Sym(c) => write!(f, "{c}"),
        }
    }
}

/// A flattened representation of a transition, useful for visualization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Edge {
    /// Origin state.
    pub from: StateId,
    /// Destination state.
    pub to: StateId,
    /// Transition label.
    pub label: EdgeLabel,
}

/// Transition stored in adjacency lists.
#[derive(Debug, Clone, PartialEq)]
pub struct Transition {
    /// Destination state.
    pub to: StateId,
    /// Transition label.
    pub label: EdgeLabel,
}

/// Describes the kind of AST operation represented by a bounding box.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BoxKind {
    Literal,
    Concat,
    Alternation,
    KleeneStar,
    KleenePlus,
    Optional,
}

/// Metadata describing a bounding box and the states it contains.
#[derive(Debug, Clone)]
pub struct BoundingBox {
    /// Unique identifier for the bounding box.
    pub id: BoxId,
    /// The AST construct associated with the box.
    pub kind: BoxKind,
    /// Identifier of the parent box, if any.
    pub parent: Option<BoxId>,
    /// States created while this box was active.
    pub states: Vec<StateId>,
}

/// Metadata associated with a concrete automaton state.
#[derive(Debug, Clone)]
pub struct State {
    /// Identifier of the state.
    pub id: StateId,
    /// Bounding box this state belongs to.
    pub box_id: Option<BoxId>,
}
