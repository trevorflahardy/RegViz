//! Deterministic finite automaton (DFA) implementation with rich analysis tools.
//!
//! The module is intentionally opinionated: DFAs are required to be complete and
//! operate over a fixed, non-empty alphabet. The implementation focuses on being
//! easy to read and reason about. Each algorithm is broken into small helper
//! functions with extensive documentation.

use std::collections::{BTreeSet, HashMap, HashSet, VecDeque};
use std::fmt;

/// Index of a state inside a DFA transition table.
pub type StateId = usize;

/// Errors that can be produced while constructing or operating on DFAs.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum DfaError {
    /// The provided alphabet was empty.
    #[error("alphabet must contain at least one symbol")]
    EmptyAlphabet,

    /// The alphabet contained duplicate symbols.
    #[error("alphabet contains duplicate symbol '{0}'")]
    DuplicateSymbol(char),

    /// The state index was out of bounds for the automaton.
    #[error("state {state} is outside the valid range 0..{max}")]
    InvalidState { state: StateId, max: StateId },

    /// A transition for the given `(state, symbol)` pair is missing.
    #[error("missing transition from state {state} on symbol '{symbol}'")]
    MissingTransition { state: StateId, symbol: char },

    /// The provided input symbol does not exist in the automaton's alphabet.
    #[error("symbol '{0}' is not part of the DFA alphabet")]
    UnknownSymbol(char),

    /// Attempted to combine DFAs with different alphabets.
    #[error("alphabets must match in order to combine DFAs")]
    AlphabetMismatch,

    /// The builder has no start state registered.
    #[error("builder requires a start state before calling build()")]
    StartStateMissing,
}

/// Alphabet wrapper that validates uniqueness and provides quick lookups.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Alphabet {
    symbols: Vec<char>,
    positions: HashMap<char, usize>,
}

impl Alphabet {
    fn new<I>(symbols: I) -> Result<Self, DfaError>
    where
        I: IntoIterator<Item = char>,
    {
        let mut unique = BTreeSet::new();
        for symbol in symbols {
            if !unique.insert(symbol) {
                return Err(DfaError::DuplicateSymbol(symbol));
            }
        }
        if unique.is_empty() {
            return Err(DfaError::EmptyAlphabet);
        }
        let symbols: Vec<char> = unique.iter().copied().collect();
        let positions = symbols
            .iter()
            .enumerate()
            .map(|(idx, ch)| (*ch, idx))
            .collect();
        Ok(Self { symbols, positions })
    }

    fn index(&self, symbol: char) -> Result<usize, DfaError> {
        self.positions
            .get(&symbol)
            .copied()
            .ok_or(DfaError::UnknownSymbol(symbol))
    }

    fn len(&self) -> usize {
        self.symbols.len()
    }

    fn iter(&self) -> impl Iterator<Item = char> + '_ {
        self.symbols.iter().copied()
    }
}

/// Fully constructed deterministic finite automaton.
#[derive(Clone, PartialEq, Eq)]
pub struct Dfa {
    alphabet: Alphabet,
    transitions: Vec<Vec<StateId>>,
    start_state: StateId,
    accept_states: BTreeSet<StateId>,
}

impl fmt::Debug for Dfa {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Dfa")
            .field("alphabet", &self.alphabet.symbols)
            .field("transitions", &self.transitions)
            .field("start_state", &self.start_state)
            .field("accept_states", &self.accept_states)
            .finish()
    }
}

impl Dfa {
    /// Creates a DFA from a fully populated transition table.
    ///
    /// The matrix must contain one row per state and one column per alphabet
    /// symbol. Each entry describes the destination state for the corresponding
    /// `(state, symbol)` pair.
    pub fn new<I>(
        alphabet: I,
        transitions: Vec<Vec<StateId>>,
        start_state: StateId,
        accept_states: impl IntoIterator<Item = StateId>,
    ) -> Result<Self, DfaError>
    where
        I: IntoIterator<Item = char>,
    {
        let alphabet = Alphabet::new(alphabet)?;
        validate_transition_matrix(&transitions, start_state)?;
        ensure_matrix_matches_alphabet(&alphabet, &transitions)?;
        let accept_states = collect_states(&transitions, accept_states)?;
        Ok(Self {
            alphabet,
            transitions,
            start_state,
            accept_states,
        })
    }

    /// Returns the number of states contained in the DFA.
    pub fn state_count(&self) -> usize {
        self.transitions.len()
    }

    /// Returns an iterator over the alphabet symbols.
    pub fn alphabet(&self) -> impl Iterator<Item = char> + '_ {
        self.alphabet.iter()
    }

    /// Executes the DFA over the provided input string.
    pub fn accepts_str(&self, input: &str) -> Result<bool, DfaError> {
        self.accepts(input.chars())
    }

    /// Executes the DFA over an arbitrary sequence of symbols.
    pub fn accepts<I>(&self, input: I) -> Result<bool, DfaError>
    where
        I: IntoIterator<Item = char>,
    {
        let mut state = self.start_state;
        for symbol in input {
            let column = self.alphabet.index(symbol)?;
            state = self.transitions[state][column];
        }
        Ok(self.accept_states.contains(&state))
    }

    /// Produces the complement DFA where accepting and rejecting states swap.
    pub fn complement(&self) -> Self {
        let accept_states = (0..self.state_count())
            .filter(|state| !self.accept_states.contains(state))
            .collect();
        Self {
            alphabet: self.alphabet.clone(),
            transitions: self.transitions.clone(),
            start_state: self.start_state,
            accept_states,
        }
    }

    /// Computes the product intersection of two DFAs sharing the same alphabet.
    pub fn intersection(&self, other: &Self) -> Result<Self, DfaError> {
        if self.alphabet != other.alphabet {
            return Err(DfaError::AlphabetMismatch);
        }
        let state_map = |a: StateId, b: StateId| a * other.state_count() + b;
        let mut transitions =
            vec![vec![0; self.alphabet.len()]; self.state_count() * other.state_count()];
        for (a_state, a_row) in self.transitions.iter().enumerate() {
            for (b_state, b_row) in other.transitions.iter().enumerate() {
                let combined = state_map(a_state, b_state);
                for (symbol_idx, (&a_next, &b_next)) in a_row.iter().zip(b_row).enumerate() {
                    transitions[combined][symbol_idx] = state_map(a_next, b_next);
                }
            }
        }
        let accept_states = self
            .accept_states
            .iter()
            .flat_map(|&a_state| {
                other
                    .accept_states
                    .iter()
                    .map(move |&b_state| state_map(a_state, b_state))
            })
            .collect::<BTreeSet<_>>();
        Ok(Self {
            alphabet: self.alphabet.clone(),
            transitions,
            start_state: state_map(self.start_state, other.start_state),
            accept_states,
        })
    }

    /// Minimises the DFA using Hopcroft's partition refinement algorithm.
    pub fn minimize(&self) -> Self {
        let reachable = self.reachable_states();
        let partitioner = Partitioner::new(self, &reachable);
        let partitions = partitioner.run();
        partitioner.into_minimized_dfa(partitions)
    }

    /// Generates an [`DfaAudit`] report describing key automaton metrics.
    pub fn audit(&self) -> DfaAudit {
        let reachable = self.reachable_states();
        let dead_states = self.dead_states(&reachable);
        DfaAudit {
            total_states: self.state_count(),
            alphabet: self.alphabet.symbols.clone(),
            unreachable_states: collect_sorted_difference(self.state_count(), &reachable),
            dead_states,
            is_complete: true,
        }
    }

    fn reachable_states(&self) -> BTreeSet<StateId> {
        let mut visited = BTreeSet::new();
        let mut queue = VecDeque::new();
        visited.insert(self.start_state);
        queue.push_back(self.start_state);
        while let Some(state) = queue.pop_front() {
            for &next in &self.transitions[state] {
                if visited.insert(next) {
                    queue.push_back(next);
                }
            }
        }
        visited
    }

    fn dead_states(&self, reachable: &BTreeSet<StateId>) -> Vec<StateId> {
        let mut reverse = vec![Vec::new(); self.state_count()];
        for (from, row) in self.transitions.iter().enumerate() {
            for &to in row {
                reverse[to].push(from);
            }
        }
        let mut alive = HashSet::new();
        let mut queue: VecDeque<_> = self.accept_states.iter().copied().collect();
        while let Some(state) = queue.pop_front() {
            if alive.insert(state) {
                queue.extend(reverse[state].iter().copied());
            }
        }
        reachable
            .iter()
            .filter(|state| !alive.contains(state))
            .copied()
            .collect()
    }
}

/// Builder to incrementally construct DFAs with readable code.
#[derive(Debug, Clone)]
pub struct DfaBuilder {
    alphabet: Alphabet,
    transitions: Vec<Vec<Option<StateId>>>,
    start_state: Option<StateId>,
    accept_states: BTreeSet<StateId>,
}

impl DfaBuilder {
    /// Creates a builder for a DFA with the provided alphabet and state count.
    pub fn new<I>(alphabet: I, state_count: usize) -> Result<Self, DfaError>
    where
        I: IntoIterator<Item = char>,
    {
        let alphabet = Alphabet::new(alphabet)?;
        let transitions = vec![vec![None; alphabet.len()]; state_count];
        Ok(Self {
            alphabet,
            transitions,
            start_state: None,
            accept_states: BTreeSet::new(),
        })
    }

    /// Sets the start state of the DFA.
    pub fn start_state(mut self, start: StateId) -> Result<Self, DfaError> {
        validate_state_index(start, self.transitions.len())?;
        self.start_state = Some(start);
        Ok(self)
    }

    /// Marks a state as accepting.
    pub fn accept_state(mut self, state: StateId) -> Result<Self, DfaError> {
        validate_state_index(state, self.transitions.len())?;
        self.accept_states.insert(state);
        Ok(self)
    }

    /// Adds a transition for the provided `(state, symbol)` pair.
    pub fn transition(
        mut self,
        from: StateId,
        symbol: char,
        to: StateId,
    ) -> Result<Self, DfaError> {
        validate_state_index(from, self.transitions.len())?;
        validate_state_index(to, self.transitions.len())?;
        let column = self.alphabet.index(symbol)?;
        self.transitions[from][column] = Some(to);
        Ok(self)
    }

    /// Finalises the builder into a fully constructed DFA.
    pub fn build(self) -> Result<Dfa, DfaError> {
        let start_state = self.start_state.ok_or(DfaError::StartStateMissing)?;
        let mut transitions = Vec::with_capacity(self.transitions.len());
        for (state, row) in self.transitions.into_iter().enumerate() {
            let filled = row
                .into_iter()
                .enumerate()
                .map(|(idx, cell)| {
                    cell.ok_or_else(|| DfaError::MissingTransition {
                        state,
                        symbol: self.alphabet.symbols[idx],
                    })
                })
                .collect::<Result<Vec<_>, _>>()?;
            transitions.push(filled);
        }
        Ok(Dfa {
            alphabet: self.alphabet,
            transitions,
            start_state,
            accept_states: self.accept_states,
        })
    }
}

/// Detailed report describing structural DFA properties.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DfaAudit {
    /// Total amount of states registered in the automaton.
    pub total_states: usize,
    /// Alphabet symbols in sorted order.
    pub alphabet: Vec<char>,
    /// States that cannot be reached from the start state.
    pub unreachable_states: Vec<StateId>,
    /// States from which no accepting state is reachable.
    pub dead_states: Vec<StateId>,
    /// DFAs constructed by the library are always complete.
    pub is_complete: bool,
}

/// Maintains the intermediate data required by Hopcroft's algorithm.
struct Partitioner<'a> {
    dfa: &'a Dfa,
    reachable: &'a BTreeSet<StateId>,
}

impl<'a> Partitioner<'a> {
    fn new(dfa: &'a Dfa, reachable: &'a BTreeSet<StateId>) -> Self {
        Self { dfa, reachable }
    }

    fn run(&self) -> Vec<BTreeSet<StateId>> {
        let mut partitions = self.initial_partitions();
        loop {
            let mut refined = Vec::new();
            let mut changed = false;
            let index_lookup = build_partition_index(&partitions);
            for part in partitions.iter() {
                let mut groups: HashMap<Vec<usize>, BTreeSet<StateId>> = HashMap::new();
                for &state in part {
                    let signature = self.state_signature(state, &index_lookup);
                    groups.entry(signature).or_default().insert(state);
                }
                if groups.len() == 1 {
                    refined.extend(groups.into_values());
                } else {
                    changed = true;
                    refined.extend(groups.into_values());
                }
            }
            partitions = deduplicate(refined);
            if !changed {
                break partitions;
            }
        }
    }

    fn initial_partitions(&self) -> Vec<BTreeSet<StateId>> {
        let (accepting, rejecting): (BTreeSet<_>, BTreeSet<_>) = self
            .reachable
            .iter()
            .partition(|state| self.dfa.accept_states.contains(state));
        match (accepting.is_empty(), rejecting.is_empty()) {
            (true, true) => vec![BTreeSet::new()],
            (true, false) => vec![rejecting],
            (false, true) => vec![accepting],
            (false, false) => vec![accepting, rejecting],
        }
    }

    fn state_signature(
        &self,
        state: StateId,
        index_lookup: &HashMap<StateId, usize>,
    ) -> Vec<usize> {
        self.dfa.transitions[state]
            .iter()
            .map(|&target| index_lookup[&target])
            .collect()
    }

    fn into_minimized_dfa(self, partitions: Vec<BTreeSet<StateId>>) -> Dfa {
        let mut representative: HashMap<StateId, StateId> = HashMap::new();
        for (idx, part) in partitions.iter().enumerate() {
            for &state in part {
                representative.insert(state, idx);
            }
        }
        let alphabet = self.dfa.alphabet.clone();
        let mut transitions = vec![vec![0; alphabet.len()]; partitions.len()];
        for (idx, part) in partitions.iter().enumerate() {
            let &state = part.iter().next().expect("partition cannot be empty");
            for (symbol_idx, &target) in self.dfa.transitions[state].iter().enumerate() {
                transitions[idx][symbol_idx] = representative[&target];
            }
        }
        let accept_states = partitions
            .iter()
            .enumerate()
            .filter(|(_, part)| part.iter().any(|s| self.dfa.accept_states.contains(s)))
            .map(|(idx, _)| idx)
            .collect();
        let start_state = representative[&self.dfa.start_state];
        Dfa {
            alphabet,
            transitions,
            start_state,
            accept_states,
        }
    }
}

fn collect_states(
    transitions: &[Vec<StateId>],
    accept_states: impl IntoIterator<Item = StateId>,
) -> Result<BTreeSet<StateId>, DfaError> {
    let max_state = transitions.len();
    let mut set = BTreeSet::new();
    for state in accept_states {
        validate_state_index(state, max_state)?;
        set.insert(state);
    }
    Ok(set)
}

fn validate_transition_matrix(
    transitions: &[Vec<StateId>],
    start_state: StateId,
) -> Result<(), DfaError> {
    if transitions.is_empty() {
        return Err(DfaError::InvalidState {
            state: start_state,
            max: 0,
        });
    }
    let len = transitions.len();
    validate_state_index(start_state, len)?;
    for row in transitions.iter() {
        for &target in row {
            validate_state_index(target, len)?;
        }
    }
    Ok(())
}

fn ensure_matrix_matches_alphabet(
    alphabet: &Alphabet,
    transitions: &[Vec<StateId>],
) -> Result<(), DfaError> {
    for (state, row) in transitions.iter().enumerate() {
        if row.len() != alphabet.len() {
            return Err(DfaError::MissingTransition {
                state,
                symbol: alphabet.symbols.first().copied().unwrap_or('?'),
            });
        }
    }
    Ok(())
}

fn validate_state_index(state: StateId, len: usize) -> Result<(), DfaError> {
    if state < len {
        Ok(())
    } else {
        Err(DfaError::InvalidState {
            state,
            max: len.saturating_sub(1),
        })
    }
}

fn collect_sorted_difference(total: usize, reachable: &BTreeSet<StateId>) -> Vec<StateId> {
    (0..total)
        .filter(|state| !reachable.contains(state))
        .collect()
}

fn deduplicate(mut partitions: Vec<BTreeSet<StateId>>) -> Vec<BTreeSet<StateId>> {
    partitions.sort_by_key(|part| part.iter().copied().collect::<Vec<_>>());
    partitions.dedup();
    partitions
}

fn build_partition_index(partitions: &[BTreeSet<StateId>]) -> HashMap<StateId, usize> {
    let mut map = HashMap::new();
    for (idx, part) in partitions.iter().enumerate() {
        for &state in part {
            map.insert(state, idx);
        }
    }
    map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_enforces_complete_transitions() {
        let builder = DfaBuilder::new(['a', 'b'], 2).unwrap();
        let err = builder
            .clone()
            .start_state(0)
            .unwrap()
            .accept_state(1)
            .unwrap()
            .transition(0, 'a', 1)
            .unwrap()
            .build()
            .unwrap_err();
        assert!(matches!(err, DfaError::MissingTransition { state: 0, .. }));
    }
}
