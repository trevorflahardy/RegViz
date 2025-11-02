use std::collections::{HashSet, VecDeque};

use crate::core::automaton::StateId;
use crate::core::dfa::Dfa;

/// Minimizes a DFA using Hopcroft's partition refinement algorithm.
pub fn minimize(dfa: &Dfa) -> Dfa {
    if dfa.trans.len() <= 1 {
        return dfa.clone();
    }

    PartitionRefinement::new(dfa).run()
}

struct PartitionRefinement<'a> {
    dfa: &'a Dfa,
    partitions: Vec<Vec<usize>>,
    state_class: Vec<usize>,
    worklist: VecDeque<(usize, usize)>,
    accepting: HashSet<StateId>,
}

impl<'a> PartitionRefinement<'a> {
    fn new(dfa: &'a Dfa) -> Self {
        let accepting: HashSet<StateId> = dfa.accepts.iter().copied().collect();
        let mut partitions = Vec::new();
        let mut accepting_block = Vec::new();
        let mut rejecting_block = Vec::new();
        for state in 0..dfa.trans.len() {
            if accepting.contains(&(state as StateId)) {
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

        let mut state_class = vec![0; dfa.trans.len()];
        for (class, block) in partitions.iter().enumerate() {
            for &state in block {
                state_class[state] = class;
            }
        }

        let mut worklist = VecDeque::new();
        for (class_idx, block) in partitions.iter().enumerate() {
            if block.is_empty() {
                continue;
            }
            for symbol_idx in 0..dfa.alphabet.len() {
                worklist.push_back((class_idx, symbol_idx));
            }
        }

        Self {
            dfa,
            partitions,
            state_class,
            worklist,
            accepting,
        }
    }

    fn run(mut self) -> Dfa {
        while let Some((class_idx, symbol_idx)) = self.worklist.pop_front() {
            let involved = self.collect_involved(class_idx, symbol_idx);
            if involved.is_empty() {
                continue;
            }
            let splits = self.split_partitions(&involved);
            self.enqueue_splits(splits);
        }
        self.build_minimized()
    }

    fn collect_involved(&self, class_idx: usize, symbol_idx: usize) -> HashSet<usize> {
        let mut involved = HashSet::new();
        for state in 0..self.dfa.trans.len() {
            let Some(dst) = self.dfa.trans[state][symbol_idx] else {
                continue;
            };
            if self.state_class[dst as usize] == class_idx {
                involved.insert(state);
            }
        }
        involved
    }

    fn split_partitions(&mut self, involved: &HashSet<usize>) -> Vec<usize> {
        let mut split_targets = Vec::new();
        let mut idx = 0;
        while idx < self.partitions.len() {
            let block = self.partitions[idx].clone();
            let (in_part, out_part) = self.partition_block(&block, involved);
            if in_part.is_empty() || out_part.is_empty() {
                idx += 1;
                continue;
            }

            self.partitions[idx] = in_part;
            let new_idx = self.partitions.len();
            self.partitions.push(out_part);
            self.relabel_block(idx);
            self.relabel_block(new_idx);

            let push_idx = if self.partitions[idx].len() < self.partitions[new_idx].len() {
                idx
            } else {
                new_idx
            };
            split_targets.push(push_idx);
            idx += 1;
        }
        split_targets
    }

    fn partition_block(
        &self,
        block: &[usize],
        involved: &HashSet<usize>,
    ) -> (Vec<usize>, Vec<usize>) {
        let mut in_part = Vec::new();
        let mut out_part = Vec::new();
        for &state in block {
            if involved.contains(&state) {
                in_part.push(state);
            } else {
                out_part.push(state);
            }
        }
        (in_part, out_part)
    }

    fn relabel_block(&mut self, block_idx: usize) {
        for &state in &self.partitions[block_idx] {
            self.state_class[state] = block_idx;
        }
    }

    fn enqueue_splits(&mut self, splits: Vec<usize>) {
        for idx in splits {
            for symbol_idx in 0..self.dfa.alphabet.len() {
                self.worklist.push_back((idx, symbol_idx));
            }
        }
    }

    fn build_minimized(self) -> Dfa {
        let mut new_trans = vec![vec![None; self.dfa.alphabet.len()]; self.partitions.len()];
        for (class_idx, block) in self.partitions.iter().enumerate() {
            if block.is_empty() {
                continue;
            }
            let repr = block[0];
            for (symbol_idx, dest) in self.dfa.trans[repr].iter().enumerate() {
                if let Some(dst) = dest {
                    new_trans[class_idx][symbol_idx] =
                        Some(self.state_class[*dst as usize] as StateId);
                }
            }
        }

        let mut new_accepts = Vec::new();
        for (idx, block) in self.partitions.iter().enumerate() {
            if block
                .iter()
                .any(|state| self.accepting.contains(&(*state as StateId)))
            {
                new_accepts.push(idx as StateId);
            }
        }

        let new_states: Vec<StateId> = (0..self.partitions.len()).map(|i| i as StateId).collect();
        let start = self.state_class[self.dfa.start as usize] as StateId;

        Dfa {
            states: new_states,
            start,
            accepts: new_accepts,
            trans: new_trans,
            alphabet: self.dfa.alphabet.to_vec(),
        }
    }
}
