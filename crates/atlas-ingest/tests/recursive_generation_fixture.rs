// Test-only graph: same generated emitter, same Rust source primitives.
#[path = "fixtures/source_model/mod.rs"]
#[allow(dead_code, unused_imports)] // The harness imports whole production primitive modules.
mod source_model;
