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
    /// The DFA being minimized.
    dfa: &'a Dfa,
    /// Current partitions of states.
    partitions: Vec<Vec<usize>>,
    /// Mapping from state to its partition class (index in `partitions`).
    state_class: Vec<usize>,
    /// Worklist of (partition class, symbol index) pairs to process.
    worklist: VecDeque<(usize, usize)>,
    /// Set of accepting states for quick lookup.
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
            let dst = self.dfa.trans[state][symbol_idx];
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
            let block = self.partitions[idx].as_slice();
            let (in_part, out_part) = self.partition_block(block, involved);
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
        let mut new_trans_table = vec![];
        for block in self.partitions.iter() {
            if block.is_empty() {
                continue;
            }
            let mut new_trans_row = vec![];
            let repr = block[0];
            for dest in self.dfa.trans[repr].iter() {
                new_trans_row.push(self.state_class[*dest as usize] as StateId);
            }
            new_trans_table.push(new_trans_row);
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
            trans: new_trans_table,
            alphabet: self.dfa.alphabet.to_vec(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::dfa;
    use crate::core::nfa::Nfa;
    use crate::core::parser::Ast;
    use crate::errors::BuildError;

    fn build_minimized_dfa(regex: &str) -> Result<Dfa, BuildError> {
        let ast = Ast::build(regex)?;
        let nfa = Nfa::build(&ast);
        let dfa = dfa::determinize(&nfa);
        Ok(minimize(&dfa))
    }

    fn dfa_accepts(dfa: &Dfa, input: &str) -> bool {
        let mut current = dfa.start;

        for ch in input.chars() {
            let symbol_idx = dfa.alphabet.iter().position(|&c| c == ch);
            match symbol_idx {
                Some(idx) => {
                    current = dfa.trans[current as usize][idx];
                }
                None => return false, // Symbol not in alphabet, reject
            }
        }

        dfa.accepts.contains(&current)
    }

    #[test]
    fn test_minimize_a_plus_a_star_equals_a_star() {
        // a+a* should minimize to same as a*
        let min1 = build_minimized_dfa("a+a*").unwrap();
        let min2 = build_minimized_dfa("a*").unwrap();

        // Both should have same number of states
        assert_eq!(
            min1.states.len(),
            min2.states.len(),
            "a+a* and a* should minimize to same number of states"
        );
        assert_eq!(
            min1.accepts.len(),
            min2.accepts.len(),
            "a+a* and a* should have same number of accepting states"
        );

        // Both should accept same strings
        let test_cases = vec!["", "a", "aa", "aaa", "aaaa"];
        for test in test_cases {
            assert_eq!(
                dfa_accepts(&min1, test),
                dfa_accepts(&min2, test),
                "a+a* and a* should accept same strings for input: {}",
                test
            );
        }
    }

    #[test]
    fn test_minimize_aa_star_has_two_states() {
        // aa* should have 2 states: non-accepting start, accepting with self-loop
        let min = build_minimized_dfa("aa*").unwrap();

        assert_eq!(min.states.len(), 2, "aa* should have exactly 2 states");
        assert_eq!(min.accepts.len(), 1, "aa* should have 1 accepting state");
        assert!(
            !min.accepts.contains(&min.start),
            "Start state should not be accepting (empty string not in aa*)"
        );

        // Verify behavior
        assert!(!dfa_accepts(&min, ""), "Should reject empty string");
        assert!(dfa_accepts(&min, "a"), "Should accept 'a'");
        assert!(dfa_accepts(&min, "aaa"), "Should accept 'aaa'");
        assert!(!dfa_accepts(&min, "b"), "Should reject 'b'");
    }

    #[test]
    fn test_minimize_a_star_single_accepting_state() {
        // a* should have 1 state that is both start and accepting
        let min = build_minimized_dfa("a*").unwrap();

        assert_eq!(min.states.len(), 1, "a* should minimize to single state");
        assert!(
            min.accepts.contains(&min.start),
            "Single state should be accepting (empty string matches)"
        );

        // Verify behavior
        assert!(dfa_accepts(&min, ""), "Should accept empty string");
        assert!(dfa_accepts(&min, "a"), "Should accept 'a'");
        assert!(dfa_accepts(&min, "aaaa"), "Should accept 'aaaa'");
    }

    #[test]
    fn test_minimize_epsilon_single_state() {
        // ε (epsilon) should be single accepting state
        let min = build_minimized_dfa("\\e").unwrap();

        assert_eq!(min.states.len(), 1, "epsilon should be single state");
        assert!(min.accepts.contains(&min.start), "Should be accepting");

        assert!(dfa_accepts(&min, ""), "Should accept empty string");
        assert!(!dfa_accepts(&min, "a"), "Should reject non-empty strings");
    }

    #[test]
    fn test_minimize_single_char() {
        // Single character 'a' should have 2-3 states (start, accept, maybe dead)
        let min = build_minimized_dfa("a").unwrap();

        assert!(
            min.states.len() <= 3,
            "Single char should have at most 3 states"
        );
        assert_eq!(min.accepts.len(), 1, "Should have one accepting state");

        assert!(!dfa_accepts(&min, ""), "Should reject empty string");
        assert!(dfa_accepts(&min, "a"), "Should accept 'a'");
        assert!(!dfa_accepts(&min, "aa"), "Should reject 'aa'");
    }

    #[test]
    fn test_minimize_alternation_merges_equivalent() {
        // (a+b)(a+b) - after first char, both branches are equivalent
        let ast = Ast::build("(a+b)(a+b)").unwrap();
        let nfa = Nfa::build(&ast);
        let dfa = dfa::determinize(&nfa);
        let original_size = dfa.states.len();

        let min = minimize(&dfa);

        // Minimized should be smaller or same
        assert!(
            min.states.len() <= original_size,
            "Minimization should not increase states"
        );

        // Verify correctness
        assert!(!dfa_accepts(&min, ""), "Should reject empty");
        assert!(!dfa_accepts(&min, "a"), "Should reject single char");
        assert!(dfa_accepts(&min, "aa"), "Should accept 'aa'");
        assert!(dfa_accepts(&min, "ab"), "Should accept 'ab'");
        assert!(dfa_accepts(&min, "ba"), "Should accept 'ba'");
        assert!(dfa_accepts(&min, "bb"), "Should accept 'bb'");
        assert!(!dfa_accepts(&min, "aaa"), "Should reject 'aaa'");
    }

    #[test]
    fn test_minimize_a_star_b_star_two_branches() {
        // a*b* should have accepting states for both branches
        let min = build_minimized_dfa("a*b*").unwrap();

        assert!(dfa_accepts(&min, ""), "Should accept empty");
        assert!(dfa_accepts(&min, "a"), "Should accept 'a'");
        assert!(dfa_accepts(&min, "aa"), "Should accept 'aa'");
        assert!(dfa_accepts(&min, "b"), "Should accept 'b'");
        assert!(dfa_accepts(&min, "bb"), "Should accept 'bb'");
        assert!(dfa_accepts(&min, "ab"), "Should accept 'ab'");
        assert!(dfa_accepts(&min, "aabb"), "Should accept 'aabb'");
        assert!(!dfa_accepts(&min, "ba"), "Should reject 'ba' (b before a)");
    }

    #[test]
    fn test_minimize_complex_redundant_pattern() {
        // (a+aa)* should minimize to a* (both accept zero or more a's)
        let min1 = build_minimized_dfa("(a+aa)*").unwrap();
        let min2 = build_minimized_dfa("a*").unwrap();

        // Should have same number of states after minimization
        assert_eq!(
            min1.states.len(),
            min2.states.len(),
            "(a+aa)* should minimize to same as a*"
        );

        let test_cases = vec!["", "a", "aa", "aaa", "aaaa"];
        for test in test_cases {
            assert_eq!(
                dfa_accepts(&min1, test),
                dfa_accepts(&min2, test),
                "(a+aa)* and a* should accept same strings for input: {}",
                test
            );
        }
    }

    #[test]
    fn test_minimize_already_minimal() {
        // Simple pattern that should already be minimal
        let ast = Ast::build("ab").unwrap();
        let nfa = Nfa::build(&ast);
        let dfa = dfa::determinize(&nfa);
        let original_size = dfa.states.len();

        let min = minimize(&dfa);

        // Size might be same or smaller, but behavior should match
        assert!(min.states.len() <= original_size);

        assert!(!dfa_accepts(&min, ""), "Should reject empty");
        assert!(!dfa_accepts(&min, "a"), "Should reject 'a'");
        assert!(dfa_accepts(&min, "ab"), "Should accept 'ab'");
        assert!(!dfa_accepts(&min, "aba"), "Should reject 'aba'");
    }

    #[test]
    fn test_minimize_optional_patterns() {
        // a? should have 3 states (2 accepting: empty and 'a', 1 dead state)
        let min = build_minimized_dfa("a?").unwrap();

        assert!(min.states.len() <= 3, "a? should have at most 3 states");
        assert!(
            min.accepts.contains(&min.start),
            "Start should be accepting (empty matches)"
        );

        assert!(dfa_accepts(&min, ""), "Should accept empty");
        assert!(dfa_accepts(&min, "a"), "Should accept 'a'");
        assert!(!dfa_accepts(&min, "aa"), "Should reject 'aa'");
    }

    #[test]
    fn test_minimize_nested_stars() {
        // (a*)* should minimize to a* (nested Kleene stars are redundant)
        let min1 = build_minimized_dfa("(a*)*").unwrap();
        let min2 = build_minimized_dfa("a*").unwrap();

        assert_eq!(
            min1.states.len(),
            min2.states.len(),
            "(a*)* should minimize to same as a*"
        );

        let test_cases = vec!["", "a", "aa", "aaa"];
        for test in test_cases {
            assert_eq!(
                dfa_accepts(&min1, test),
                dfa_accepts(&min2, test),
                "(a*)* and a* should accept same strings for input: {}",
                test
            );
        }
    }

    #[test]
    fn test_minimize_preserves_correctness() {
        // DFA should handle minimization without crashing
        let ast = Ast::build("a").unwrap();
        let nfa = Nfa::build(&ast);
        let dfa = dfa::determinize(&nfa);

        // Should handle small DFAs gracefully
        let min = minimize(&dfa);
        assert!(!min.states.is_empty(), "Should have at least one state");
    }

    #[test]
    fn test_minimize_disjoint_accepting_states() {
        // a+b should have structure: start -> {accept_a, accept_b}
        let min = build_minimized_dfa("a+b").unwrap();

        assert!(!dfa_accepts(&min, ""), "Should reject empty");
        assert!(dfa_accepts(&min, "a"), "Should accept 'a'");
        assert!(dfa_accepts(&min, "b"), "Should accept 'b'");
        assert!(!dfa_accepts(&min, "ab"), "Should reject 'ab'");
        assert!(!dfa_accepts(&min, "c"), "Should reject 'c'");
    }

    #[test]
    fn test_minimize_multiple_equivalent_paths() {
        // (aa+aa) should minimize to aa (duplicate branches)
        let min1 = build_minimized_dfa("(aa+aa)").unwrap();
        let min2 = build_minimized_dfa("aa").unwrap();

        assert_eq!(
            min1.states.len(),
            min2.states.len(),
            "Duplicate branches should minimize to single path"
        );

        let test_cases = vec!["", "a", "aa", "aaa"];
        for test in test_cases {
            assert_eq!(
                dfa_accepts(&min1, test),
                dfa_accepts(&min2, test),
                "(aa+aa) and aa should accept same strings for input: {}",
                test
            );
        }
    }
}
