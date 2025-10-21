use cot_4210_final_proj::{Dfa, DfaBuilder, DfaError};

fn even_ones_dfa() -> Dfa {
    DfaBuilder::new(['0', '1'], 2)
        .unwrap()
        .start_state(0)
        .unwrap()
        .accept_state(0)
        .unwrap()
        .transition(0, '0', 0)
        .unwrap()
        .transition(0, '1', 1)
        .unwrap()
        .transition(1, '0', 1)
        .unwrap()
        .transition(1, '1', 0)
        .unwrap()
        .build()
        .unwrap()
}

#[test]
fn accepts_and_complement_behave_as_expected() {
    let dfa = even_ones_dfa();
    assert!(dfa.accepts_str("0101").unwrap());
    assert!(!dfa.accepts_str("001").unwrap());

    let complement = dfa.complement();
    assert!(!complement.accepts_str("0101").unwrap());
    assert!(complement.accepts_str("001").unwrap());
}

#[test]
fn intersection_requires_identical_alphabet() {
    let dfa_a = even_ones_dfa();
    let dfa_b = DfaBuilder::new(['a', 'b'], 1)
        .unwrap()
        .start_state(0)
        .unwrap()
        .accept_state(0)
        .unwrap()
        .transition(0, 'a', 0)
        .unwrap()
        .transition(0, 'b', 0)
        .unwrap()
        .build()
        .unwrap();

    let err = dfa_a.intersection(&dfa_b).unwrap_err();
    assert!(matches!(err, DfaError::AlphabetMismatch));
}

#[test]
fn audit_detects_unreachable_and_dead_states() {
    // DFA with three states: 0 start, 1 accepting, 2 unreachable and dead.
    let dfa = DfaBuilder::new(['a'], 3)
        .unwrap()
        .start_state(0)
        .unwrap()
        .accept_state(1)
        .unwrap()
        .transition(0, 'a', 1)
        .unwrap()
        .transition(1, 'a', 1)
        .unwrap()
        .transition(2, 'a', 2)
        .unwrap()
        .build()
        .unwrap();

    let audit = dfa.audit();
    assert_eq!(audit.total_states, 3);
    assert_eq!(audit.unreachable_states, vec![2]);
    assert_eq!(audit.dead_states, vec![]);
    assert!(audit.is_complete);
}

#[test]
fn minimize_collapses_equivalent_states() {
    // DFA that tracks the last two bits of input but only cares about parity of ones.
    let transitions = vec![
        vec![0, 1], // 0 -> even parity states
        vec![1, 0], // 1 -> odd parity states
        vec![2, 3], // 2 mirrors state 0
        vec![3, 2], // 3 mirrors state 1
    ];
    let accept_states = [0, 2];
    let dfa = Dfa::new(['0', '1'], transitions, 0, accept_states).unwrap();

    let minimized = dfa.minimize();
    assert_eq!(minimized.state_count(), 2);
    assert!(minimized.accepts_str("0101").unwrap());
    assert!(!minimized.accepts_str("111").unwrap());
}

#[test]
fn accepts_reports_unknown_symbol() {
    let dfa = even_ones_dfa();
    let err = dfa.accepts_str("02").unwrap_err();
    assert!(matches!(err, DfaError::UnknownSymbol('2')));
}

#[test]
fn builder_rejects_invalid_states() {
    let err = DfaBuilder::new(['a'], 2)
        .unwrap()
        .start_state(5)
        .unwrap_err();
    assert!(matches!(err, DfaError::InvalidState { state: 5, .. }));
}

#[test]
fn new_validates_transition_matrix_dimensions() {
    let transitions = vec![vec![0], vec![]];
    let err = Dfa::new(['a'], transitions, 0, [1]).unwrap_err();
    assert!(matches!(err, DfaError::MissingTransition { state: 1, .. }));
}

#[test]
fn intersection_language_is_correct() {
    // Accepts binary strings with an even number of ones AND ending with 0.
    let even = even_ones_dfa();
    let ending_zero = DfaBuilder::new(['0', '1'], 2)
        .unwrap()
        .start_state(0)
        .unwrap()
        .accept_state(0)
        .unwrap()
        .transition(0, '0', 0)
        .unwrap()
        .transition(0, '1', 1)
        .unwrap()
        .transition(1, '0', 0)
        .unwrap()
        .transition(1, '1', 1)
        .unwrap()
        .build()
        .unwrap();

    let intersection = even.intersection(&ending_zero).unwrap();
    assert!(intersection.accepts_str("110").unwrap());
    assert!(!intersection.accepts_str("10").unwrap());
}
