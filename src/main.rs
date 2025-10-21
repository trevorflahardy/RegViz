use cot_4210_final_proj::DfaBuilder;

fn main() {
    // Example DFA: accepts binary strings with an even number of 1s.
    let dfa = DfaBuilder::new(['0', '1'], 2)
        .expect("alphabet should be valid")
        .start_state(0)
        .and_then(|builder| builder.accept_state(0))
        .and_then(|builder| builder.transition(0, '0', 0))
        .and_then(|builder| builder.transition(0, '1', 1))
        .and_then(|builder| builder.transition(1, '0', 1))
        .and_then(|builder| builder.transition(1, '1', 0))
        .and_then(|builder| builder.build())
        .expect("builder should be complete");

    let audit = dfa.audit();
    println!(
        "DFA states: {} (dead states: {:?})",
        audit.total_states, audit.dead_states
    );
}
