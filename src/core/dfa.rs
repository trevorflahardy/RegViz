use std::collections::{HashSet, VecDeque};

use indexmap::IndexMap;

use crate::core::nfa::{Nfa, StateId};
use crate::core::sim;

/// Deterministic finite automaton produced from subset construction.
#[derive(Debug, Clone)]
pub struct Dfa {
    /// All DFA state identifiers.
    pub states: Vec<u32>,
    /// Start state identifier.
    pub start: u32,
    /// Accepting state identifiers.
    pub accepts: Vec<u32>,
    /// Transition table indexed by state then alphabet symbol.
    pub trans: Vec<Vec<Option<u32>>>,
}

/// Executes subset construction on an [`Nfa`], returning the DFA and alphabet.
pub fn determinize(nfa: &Nfa) -> (Dfa, Vec<char>) {
    Determinizer::new(nfa).run()
}

struct Determinizer<'a> {
    nfa: &'a Nfa,
    alphabet: Vec<char>,
    map: IndexMap<Vec<StateId>, u32>,
    queue: VecDeque<Vec<StateId>>,
    transitions: Vec<Vec<Option<u32>>>,
}

impl<'a> Determinizer<'a> {
    fn new(nfa: &'a Nfa) -> Self {
        let alphabet = nfa.alphabet();
        let mut map = IndexMap::new();
        let mut queue = VecDeque::new();

        let mut seed = HashSet::new();
        seed.insert(nfa.start);
        let closure = sim::epsilon_closure(&seed, nfa);
        let start_key = set_to_key(closure);

        map.insert(start_key.clone(), 0);
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
        let states: Vec<u32> = (0..self.map.len()).map(|i| i as u32).collect();
        let dfa = Dfa {
            states,
            start: 0,
            accepts,
            trans: self.transitions,
        };
        (dfa, self.alphabet)
    }

    fn ensure_capacity(&mut self, len: usize) {
        while self.transitions.len() < len {
            self.transitions.push(vec![None; self.alphabet.len()]);
        }
    }

    fn advance_subset(&mut self, subset: &HashSet<StateId>, symbol: char) -> Option<u32> {
        let moved = sim::move_on(subset, symbol, self.nfa);
        if moved.is_empty() {
            return None;
        }
        let closure = sim::epsilon_closure(&moved, self.nfa);
        Some(self.lookup_or_insert(closure))
    }

    fn lookup_or_insert(&mut self, subset: HashSet<StateId>) -> u32 {
        let key = set_to_key(subset);
        if let Some(id) = self.map.get(&key) {
            *id
        } else {
            let new_id = self.map.len() as u32;
            self.map.insert(key.clone(), new_id);
            self.queue.push_back(key);
            new_id
        }
    }

    fn collect_accepting(&self) -> Vec<u32> {
        self.map
            .iter()
            .filter_map(|(subset, id)| {
                let accepting = subset.iter().any(|state| self.nfa.accepts.contains(state));
                accepting.then_some(*id)
            })
            .collect()
    }
}

fn set_to_key(set: HashSet<StateId>) -> Vec<StateId> {
    let mut vec: Vec<StateId> = set.into_iter().collect();
    vec.sort_unstable();
    vec
}
