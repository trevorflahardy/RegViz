use std::collections::{HashSet, VecDeque};

use indexmap::IndexMap;

use crate::core::nfa::{Nfa, StateId};
use crate::core::sim;

#[derive(Debug, Clone)]
pub struct Dfa {
    pub states: Vec<u32>,
    pub start: u32,
    pub accepts: Vec<u32>,
    pub trans: Vec<Vec<Option<u32>>>,
}

pub fn determinize(nfa: &Nfa) -> (Dfa, Vec<char>) {
    let alphabet = nfa.alphabet();
    let mut map: IndexMap<Vec<StateId>, u32> = IndexMap::new();
    let mut queue: VecDeque<Vec<StateId>> = VecDeque::new();
    let mut transitions: Vec<Vec<Option<u32>>> = Vec::new();

    let mut seed = HashSet::new();
    seed.insert(nfa.start);
    let seed = sim::epsilon_closure(&seed, nfa);
    let start_key = set_to_key(seed);
    map.insert(start_key.clone(), 0);
    queue.push_back(start_key.clone());

    while let Some(key) = queue.pop_front() {
        let state_id = map[&key];
        ensure_capacity(&mut transitions, state_id as usize + 1, alphabet.len());
        let subset: HashSet<StateId> = key.iter().copied().collect();
        for (i, symbol) in alphabet.iter().enumerate() {
            let moved = sim::move_on(&subset, *symbol, nfa);
            if moved.is_empty() {
                transitions[state_id as usize][i] = None;
                continue;
            }
            let closure = sim::epsilon_closure(&moved, nfa);
            let key_next = set_to_key(closure);
            let entry = if let Some(id) = map.get(&key_next) {
                *id
            } else {
                let new_id = map.len() as u32;
                map.insert(key_next.clone(), new_id);
                queue.push_back(key_next);
                new_id
            };
            transitions[state_id as usize][i] = Some(entry);
        }
    }

    let mut dfa_states = Vec::new();
    let mut accepts = Vec::new();
    for (key, id) in map.iter() {
        dfa_states.push(*id);
        if key.iter().any(|s| nfa.accepts.contains(s)) {
            accepts.push(*id);
        }
    }

    let dfa = Dfa {
        states: dfa_states,
        start: 0,
        accepts,
        trans: transitions,
    };
    (dfa, alphabet)
}

fn set_to_key(set: HashSet<StateId>) -> Vec<StateId> {
    let mut vec: Vec<StateId> = set.into_iter().collect();
    vec.sort_unstable();
    vec
}

fn ensure_capacity(trans: &mut Vec<Vec<Option<u32>>>, len: usize, alpha: usize) {
    while trans.len() < len {
        trans.push(vec![None; alpha]);
    }
}
