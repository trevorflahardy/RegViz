use iced::Point;

use crate::core::nfa::{Nfa, StateId};

pub fn layout_states(nfa: &Nfa, width: f32, height: f32) -> Vec<(StateId, Point)> {
    let count = nfa.states.len();
    if count == 0 {
        return Vec::new();
    }
    let center = Point::new(width / 2.0, height / 2.0);
    if count == 1 {
        return vec![(nfa.start, center)];
    }
    let radius = 0.35 * width.min(height);
    nfa.states
        .iter()
        .enumerate()
        .map(|(idx, state)| {
            let angle = (idx as f32 / count as f32) * std::f32::consts::TAU;
            let x = center.x + radius * angle.cos();
            let y = center.y + radius * angle.sin();
            (*state, Point::new(x, y))
        })
        .collect()
}
