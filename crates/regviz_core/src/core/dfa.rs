use std::collections::{HashSet, VecDeque};

use indexmap::IndexMap;

use crate::core::automaton::StateId;
use crate::core::nfa::Nfa;
use crate::core::sim;

/// Deterministic finite automaton produced from subset construction.
#[derive(Debug, Clone)]
pub struct Dfa {
    /// All DFA state identifiers.
    pub states: Vec<StateId>,
    /// Start state identifier.
    pub start: StateId,
    /// Accepting state identifiers.
    pub accepts: Vec<StateId>,
    /// Transition table indexed by state then alphabet symbol.
    pub trans: Vec<Vec<Option<StateId>>>,
}

/// A helper function to determinize an NFA into a DFA using subset construction.
///
/// # Arguments
///
/// - `nfa` (`&Nfa`) - The NFA to be determinized.
///
/// # Returns
///
/// - `(Dfa, Vec<char>)` - A tuple containing the resulting DFA and its alphabet.
pub fn determinize(nfa: &Nfa) -> (Dfa, Vec<char>) {
    Determinizer::new(nfa).run()
}

fn set_to_key(set: HashSet<StateId>) -> Vec<StateId> {
    let mut vec: Vec<StateId> = set.into_iter().collect();
    vec.sort_unstable();
    vec
}

/// Represents a Determinizer performing subset construction.
/// When run, transforms the given NFA into an equivalent DFA.
struct Determinizer<'a> {
    /// The underlying NFA being determinized.
    nfa: &'a Nfa,

    /// The alphabet of symbols used in the NFA.
    alphabet: Vec<char>,

    /// Mapping from NFA state subsets to DFA state IDs.
    map: IndexMap<Vec<StateId>, StateId>,

    /// Queue of NFA state subsets to process.
    queue: VecDeque<Vec<StateId>>,

    /// Array of DFA transitions being built.
    transitions: Vec<Vec<Option<StateId>>>,
}

impl<'a> Determinizer<'a> {
    /// Creates a new [`Determinizer`] for the given NFA.
    ///
    /// # Arguments
    ///
    /// - `nfa` (`&'a Nfa`) - The NFA to be determinized.
    ///
    /// # Returns
    ///
    /// - `Self` - A new instance of `Determinizer`.
    fn new(nfa: &'a Nfa) -> Self {
        let alphabet = nfa.alphabet();
        let mut map = IndexMap::new();
        let mut queue = VecDeque::new();

        let mut seed = HashSet::new();
        seed.insert(nfa.start);
        let closure = sim::epsilon_closure(&seed, nfa);
        let start_key = set_to_key(closure);

        let start_id: StateId = 0;
        map.insert(start_key.clone(), start_id);
        queue.push_back(start_key);

        Self {
            nfa,
            alphabet,
            map,
            queue,
            transitions: Vec::new(),
        }
    }

    fn run(mut self) -> (Dfa, Vec<char>) {
        while let Some(key) = self.queue.pop_front() {
            let state_id = self.map[&key];
            self.ensure_capacity(state_id as usize + 1);
            let subset: HashSet<StateId> = key.iter().copied().collect();

            for symbol_idx in 0..self.alphabet.len() {
                let symbol = self.alphabet[symbol_idx];
                let next = self.advance_subset(&subset, symbol);
                self.transitions[state_id as usize][symbol_idx] = next;
            }
        }

        let accepts = self.collect_accepting();
        let states: Vec<StateId> = (0..self.map.len()).map(|i| i as StateId).collect();
        let dfa = Dfa {
            states,
            start: 0,
            accepts,
            trans: self.transitions,
        };
        (dfa, self.alphabet)
    }

    /// Ensures the transitions vector has at least `len` elements.
    ///
    /// # Arguments
    ///
    /// - `len` (`usize`) - The minimum length to ensure for the transitions vector.
    ///
    /// # Returns
    ///
    /// - `()` - This function does not return a value.
    fn ensure_capacity(&mut self, len: usize) -> () {
        while self.transitions.len() < len {
            self.transitions.push(vec![None; self.alphabet.len()]);
        }
    }

    /// Gets the next DFA state for a given NFA state subset and input symbol.
    ///
    /// # Arguments
    ///
    /// - `subset` (`&HashSet<StateId>`) - The current subset of NFA states.
    /// - `symbol` (`char`) - The input symbol to advance on.
    ///
    /// # Returns
    ///
    /// - `Option<StateId>` - The next DFA state ID, or `None` if there is no transition.
    fn advance_subset(&mut self, subset: &HashSet<StateId>, symbol: char) -> Option<StateId> {
        let moved = sim::move_on(subset, symbol, self.nfa);
        if moved.is_empty() {
            return None;
        }

        let closure = sim::epsilon_closure(&moved, self.nfa);
        Some(self.lookup_or_insert(closure))
    }

    /// Looks up or inserts a set of DFA states into the underlying map and queue.
    ///
    /// # Arguments
    ///
    /// - `subset` (`HashSet<StateId>`) - The subset of NFA states to look up or insert.
    ///
    /// # Returns
    ///
    /// - `StateId` - The DFA state ID corresponding to the subset.
    fn lookup_or_insert(&mut self, subset: HashSet<StateId>) -> StateId {
        let key = set_to_key(subset);
        if let Some(id) = self.map.get(&key) {
            *id
        } else {
            let new_id = self.map.len() as StateId;
            self.map.insert(key.clone(), new_id);
            self.queue.push_back(key);
            new_id
        }
    }

    /// Collects the list of underlying accepting state IDS.
    ///
    /// # Returns
    ///
    /// - `Vec<StateId>` - A vector of DFA state IDs that are accepting states.
    fn collect_accepting(&self) -> Vec<StateId> {
        self.map
            .iter()
            .filter_map(|(subset, id)| {
                let accepting = subset.iter().any(|state| self.nfa.accepts.contains(state));
                accepting.then_some(*id)
            })
            .collect()
    }
}
