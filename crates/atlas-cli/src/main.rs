#![deny(unsafe_code)]

mod agent_skills;
mod cli;
mod client;
mod commands;
mod output;
mod progress;
mod terminal;

fn main() -> std::process::ExitCode {
    cli::main()
}
