//! Core library for constructing and analyzing deterministic finite automata (DFAs).
//!
//! The crate provides a small, well documented toolkit for building DFAs,
//! executing them over input strings, and performing common regular-language
//! transformations such as complementation, intersection, and minimisation.
//! Additionally, a detailed [`DfaAudit`] report is available to inspect the
//! health of an automaton, including unreachable and dead states.

pub mod automata;

pub use automata::dfa::{Dfa, DfaAudit, DfaBuilder, DfaError, StateId};
