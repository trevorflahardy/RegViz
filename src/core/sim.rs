use std::collections::HashSet;

use crate::core::dfa::Dfa;
use crate::core::nfa::{EdgeLabel, Nfa, StateId};

/// Simulates a DFA and reports whether it accepts the provided input.
pub fn simulate_dfa(dfa: &Dfa, alphabet: &[char], input: &str) -> bool {
    let mut state = dfa.start;
    for ch in input.chars() {
        let idx = match alphabet.iter().position(|&c| c == ch) {
            Some(i) => i,
            None => return false,
        };
        match dfa.trans[state as usize][idx] {
            Some(next) => state = next,
            None => return false,
        }
    }
    dfa.accepts.contains(&state)
}

/// Computes the epsilon-closure of a state set in an NFA using DFS.
pub fn epsilon_closure(seed: &HashSet<StateId>, nfa: &Nfa) -> HashSet<StateId> {
    let mut closure = seed.clone();
    let mut stack: Vec<StateId> = seed.iter().copied().collect();
    while let Some(state) = stack.pop() {
        for tr in nfa.transitions(state) {
            if tr.label == EdgeLabel::Eps && closure.insert(tr.to) {
                stack.push(tr.to);
            }
        }
    }
    closure
}

/// Advances the frontier one step on a symbol, without taking epsilon-closures.
pub fn move_on(states: &HashSet<StateId>, symbol: char, nfa: &Nfa) -> HashSet<StateId> {
    let mut frontier = HashSet::new();
    for state in states {
        for tr in nfa.transitions(*state) {
            if tr.label == EdgeLabel::Sym(symbol) {
                frontier.insert(tr.to);
            }
        }
    }
    frontier
}

/// Simulates an NFA using the standard powerset traversal.
pub fn nfa_accepts(nfa: &Nfa, input: &str) -> bool {
    let mut current = HashSet::new();
    current.insert(nfa.start);
    current = epsilon_closure(&current, nfa);
    for ch in input.chars() {
        let moved = move_on(&current, ch, nfa);
        current = epsilon_closure(&moved, nfa);
        if current.is_empty() {
            return false;
        }
    }
    current.iter().any(|state| nfa.accepts.contains(state))
}
