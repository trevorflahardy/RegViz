use std::collections::{HashMap, HashSet};

use regviz_core::core::automaton::{EdgeLabel, StateId};
use regviz_core::core::dfa::Dfa;
use regviz_core::core::nfa::Nfa;
use regviz_core::core::sim;

use crate::graph::{EdgeHighlight, Highlights, StateHighlight};

/// Specifies which automaton should drive the simulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SimulationTarget {
    /// Drive the simulation using the Thompson NFA.
    #[default]
    Nfa,
    /// Drive the simulation using the determinised DFA.
    Dfa,
}

/// Snapshot describing the automaton after consuming a prefix of the input.
#[derive(Debug, Clone)]
pub struct SimulationStep {
    /// Index of the step (0 = before consuming any input).
    pub index: usize,
    /// Character consumed to reach this step (None for the initial state).
    pub consumed: Option<char>,
    /// Set of states that are currently active.
    pub active_states: HashSet<StateId>,
    /// Edges that were taken while advancing to this step.
    pub traversed_edges: HashSet<EdgeHighlight>,
    /// Whether this step represents an accepting frontier.
    pub accepted: bool,
}

impl SimulationStep {
    /// Creates a new simulation step snapshot.
    #[must_use]
    pub fn new(
        index: usize,
        consumed: Option<char>,
        active_states: HashSet<StateId>,
        traversed_edges: HashSet<EdgeHighlight>,
        accepted: bool,
    ) -> Self {
        Self {
            index,
            consumed,
            active_states,
            traversed_edges,
            accepted,
        }
    }
}

/// Ordered collection of simulation steps from start to finish.
#[derive(Debug, Clone)]
pub struct SimulationTrace {
    steps: Vec<SimulationStep>,
}

impl SimulationTrace {
    /// Creates a new trace from a list of steps.
    #[must_use]
    pub fn new(steps: Vec<SimulationStep>) -> Self {
        Self { steps }
    }

    /// Returns the number of stored steps.
    #[must_use]
    pub fn len(&self) -> usize {
        self.steps.len()
    }

    /// Retrieves a step by index.
    #[must_use]
    pub fn step(&self, index: usize) -> Option<&SimulationStep> {
        self.steps.get(index)
    }
}

/// Reactive state used by the UI to drive step-by-step simulation.
#[derive(Debug, Clone, Default)]
pub struct SimulationState {
    /// Input string provided by the user.
    pub input: String,
    /// Index of the currently displayed step.
    pub cursor: usize,
    /// Selected automaton (NFA or DFA).
    pub target: SimulationTarget,
    trace: Option<SimulationTrace>,
}

impl SimulationState {
    /// Replaces the current trace and clamps the cursor to the new bounds.
    pub fn set_trace(&mut self, trace: Option<SimulationTrace>) {
        self.trace = trace;
        let len = self.trace.as_ref().map(|t| t.len()).unwrap_or(0);
        if len == 0 {
            self.cursor = 0;
        } else if self.cursor >= len {
            self.cursor = len.saturating_sub(1);
        }
    }

    /// Clears the current trace (used when automata cannot be simulated).
    pub fn clear_trace(&mut self) {
        self.trace = None;
        self.cursor = 0;
    }

    /// Returns the number of available steps.
    #[must_use]
    pub fn step_count(&self) -> Option<usize> {
        self.trace.as_ref().map(|trace| trace.len())
    }

    /// Returns the currently selected snapshot.
    #[must_use]
    pub fn current_step(&self) -> Option<&SimulationStep> {
        self.trace
            .as_ref()
            .and_then(|trace| trace.step(self.cursor))
    }

    /// Returns highlights describing the active states and edges.
    #[must_use]
    pub fn current_highlights(&self) -> Option<Highlights> {
        let step = self.current_step()?;
        let total_steps = self.step_count()?;
        let is_terminal_step = self.cursor + 1 == total_steps;
        let highlight_style = if is_terminal_step && !step.accepted {
            StateHighlight::Rejected
        } else {
            StateHighlight::Active
        };

        let mut states = HashMap::new();
        for state in &step.active_states {
            states.insert(*state, highlight_style);
        }

        Some(Highlights::new(states, step.traversed_edges.clone()))
    }

    /// Returns whether stepping backward is possible.
    #[must_use]
    pub fn can_step_backward(&self) -> bool {
        self.cursor > 0
    }

    /// Returns whether stepping forward is possible.
    #[must_use]
    pub fn can_step_forward(&self) -> bool {
        match self.step_count() {
            Some(len) => self.cursor + 1 < len,
            None => false,
        }
    }

    /// Moves to the previous snapshot if possible.
    pub fn step_backward(&mut self) {
        if self.can_step_backward() {
            self.cursor -= 1;
        }
    }

    /// Advances to the next snapshot if possible.
    pub fn step_forward(&mut self) {
        if self.can_step_forward() {
            self.cursor += 1;
        }
    }

    /// Resets the cursor to the initial step.
    pub fn reset_cursor(&mut self) {
        self.cursor = 0;
    }

    /// Returns whether the cursor is positioned at the terminal step without acceptance.
    #[must_use]
    pub fn is_current_rejection(&self) -> bool {
        let Some(step) = self.current_step() else {
            return false;
        };
        let Some(total_steps) = self.step_count() else {
            return false;
        };
        self.cursor + 1 == total_steps && !step.accepted
    }
}

/// Builds a simulation trace for an NFA by computing epsilon closures between steps.
#[must_use]
pub fn build_nfa_trace(nfa: &Nfa, input: &str) -> SimulationTrace {
    let symbols: Vec<char> = input.chars().collect();
    let mut steps = Vec::with_capacity(symbols.len() + 1);

    let mut current: HashSet<StateId> = HashSet::new();
    current.insert(nfa.start);
    current = sim::epsilon_closure(&current, nfa);
    let initial_accepting = current.iter().any(|state| nfa.accepts.contains(state));
    steps.push(SimulationStep::new(
        0,
        None,
        current.clone(),
        HashSet::new(),
        initial_accepting,
    ));

    for (idx, symbol) in symbols.iter().enumerate() {
        let mut traversed = HashSet::new();

        for state in &current {
            for transition in nfa.transitions(*state) {
                if transition.label == EdgeLabel::Sym(*symbol) {
                    traversed.insert(EdgeHighlight::new(
                        *state,
                        transition.to,
                        EdgeLabel::Sym(*symbol),
                    ));
                }
            }
        }

        let moved = sim::move_on(&current, *symbol, nfa);
        let next = sim::epsilon_closure(&moved, nfa);
        let accepting = next.iter().any(|state| nfa.accepts.contains(state));

        steps.push(SimulationStep::new(
            idx + 1,
            Some(*symbol),
            next.clone(),
            traversed,
            accepting,
        ));
        current = next;

        if current.is_empty() && idx + 1 < symbols.len() {
            break;
        }
    }

    SimulationTrace::new(steps)
}

/// Builds a simulation trace for a DFA using the deterministic transition table.
#[must_use]
pub fn build_dfa_trace(dfa: &Dfa, alphabet: &[char], input: &str) -> SimulationTrace {
    let symbols: Vec<char> = input.chars().collect();
    let mut steps = Vec::with_capacity(symbols.len() + 1);
    let mut current = Some(dfa.start);

    let mut initial = HashSet::new();
    initial.insert(dfa.start);
    let initial_accepting = initial.iter().any(|state| dfa.accepts.contains(state));
    steps.push(SimulationStep::new(
        0,
        None,
        initial,
        HashSet::new(),
        initial_accepting,
    ));

    for (idx, symbol) in symbols.iter().enumerate() {
        let mut traversed = HashSet::new();

        if let Some(state) = current {
            if let Some(symbol_idx) = alphabet.iter().position(|&candidate| candidate == *symbol) {
                if let Some(next) = dfa.trans[state as usize][symbol_idx] {
                    traversed.insert(EdgeHighlight::new(state, next, EdgeLabel::Sym(*symbol)));
                    current = Some(next);
                } else {
                    current = None;
                }
            } else {
                current = None;
            }
        }

        let mut active = HashSet::new();
        if let Some(state) = current {
            active.insert(state);
        }
        let accepting = active.iter().any(|state| dfa.accepts.contains(state));

        steps.push(SimulationStep::new(
            idx + 1,
            Some(*symbol),
            active,
            traversed,
            accepting,
        ));

        if current.is_none() && idx + 1 < symbols.len() {
            break;
        }
    }

    SimulationTrace::new(steps)
}
