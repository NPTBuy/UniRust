//! Engine module providing pure-Rust Vietnamese IME processing.

pub mod telex;
pub mod unicode_map;

pub use telex::{EngineAction, TelexEngine, WordSnapshot};

