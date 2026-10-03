//! mighty-wasm-dec as a library: parse → lift → passes → emit.
//!
//! The binary (`src/main.rs`) is a thin CLI over this API. External pass
//! authors start here: see the `passes::FuncPass` trait and
//! `examples/const_on_right.rs`.

pub mod callgraph;
pub mod cfg;
pub mod emit;
pub mod ir;
pub mod lift;
pub mod parse;
pub mod passes;
pub mod types;
