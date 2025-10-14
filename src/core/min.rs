use std::collections::{HashSet, VecDeque};

use crate::core::dfa::Dfa;

pub fn minimize(dfa: &Dfa, alphabet: &[char]) -> Dfa {
    let num_states = dfa.trans.len();
    if num_states <= 1 {
        return dfa.clone();
    }

    let accepts: HashSet<u32> = dfa.accepts.iter().copied().collect();
    let mut partitions: Vec<Vec<usize>> = Vec::new();
    let mut accepting_block = Vec::new();
    let mut rejecting_block = Vec::new();
    for state in 0..num_states {
        if accepts.contains(&(state as u32)) {
            accepting_block.push(state);
        } else {
            rejecting_block.push(state);
        }
    }
    if !accepting_block.is_empty() {
        partitions.push(accepting_block);
    }
    if !rejecting_block.is_empty() {
        partitions.push(rejecting_block);
    }

    let mut state_class: Vec<usize> = vec![0; num_states];
    for (class, block) in partitions.iter().enumerate() {
        for &state in block {
            state_class[state] = class;
        }
    }

    let mut worklist: VecDeque<(usize, usize)> = VecDeque::new();
    for (class_idx, block) in partitions.iter().enumerate() {
        if block.is_empty() {
            continue;
        }
        for symbol_idx in 0..alphabet.len() {
            worklist.push_back((class_idx, symbol_idx));
        }
    }

    while let Some((class_idx, symbol_idx)) = worklist.pop_front() {
        let mut involved = HashSet::new();
        for state in 0..num_states {
            if let Some(dst) = dfa.trans[state][symbol_idx] {
                if state_class[dst as usize] == class_idx {
                    involved.insert(state);
                }
            }
        }
        if involved.is_empty() {
            continue;
        }
        let mut split_targets = Vec::new();
        let mut idx = 0;
        while idx < partitions.len() {
            let block = &partitions[idx];
            let mut in_part = Vec::new();
            let mut out_part = Vec::new();
            for &state in block {
                if involved.contains(&state) {
                    in_part.push(state);
                } else {
                    out_part.push(state);
                }
            }
            if in_part.is_empty() || out_part.is_empty() {
                idx += 1;
                continue;
            }
            partitions[idx] = in_part;
            let new_idx = partitions.len();
            partitions.push(out_part);
            for &state in &partitions[idx] {
                state_class[state] = idx;
            }
            for &state in &partitions[new_idx] {
                state_class[state] = new_idx;
            }
            let push_idx = if partitions[idx].len() < partitions[new_idx].len() {
                idx
            } else {
                new_idx
            };
            split_targets.push(push_idx);
            idx += 1;
        }
        for target in split_targets {
            for symbol_idx in 0..alphabet.len() {
                worklist.push_back((target, symbol_idx));
            }
        }
    }

    let mut new_trans = vec![vec![None; alphabet.len()]; partitions.len()];
    for (class_idx, block) in partitions.iter().enumerate() {
        if block.is_empty() {
            continue;
        }
        let repr = block[0];
        for (symbol_idx, dest) in dfa.trans[repr].iter().enumerate() {
            if let Some(dst) = dest {
                new_trans[class_idx][symbol_idx] = Some(state_class[*dst as usize] as u32);
            }
        }
    }

    let mut new_accepts = Vec::new();
    for (idx, block) in partitions.iter().enumerate() {
        if block.iter().any(|state| accepts.contains(&(*state as u32))) {
            new_accepts.push(idx as u32);
        }
    }

    let new_states: Vec<u32> = (0..partitions.len()).map(|i| i as u32).collect();
    let start = state_class[dfa.start as usize] as u32;

    Dfa {
        states: new_states,
        start,
        accepts: new_accepts,
        trans: new_trans,
    }
}
