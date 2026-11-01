//! Explicit-path controlled ingest observations; never part of product bundles.
#[path = "ingest_experiment/mod.rs"]
mod experiment;
fn main() -> std::process::ExitCode {
    match experiment::run() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            std::process::ExitCode::from(2)
        }
    }
}
