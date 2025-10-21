//! Automata related data structures and utilities.

pub mod dfa;

pub use dfa::{Dfa, DfaAudit, DfaBuilder, DfaError, StateId};
