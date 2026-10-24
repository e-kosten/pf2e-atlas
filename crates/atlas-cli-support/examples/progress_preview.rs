//! Simulated progress for terminal presentation review; no workspace is opened.
use atlas_cli_support::{ProgressMode, ProgressOptions, init_tracing};
use std::{thread, time::Duration};

fn main() {
    init_tracing(ProgressOptions {
        mode: ProgressMode::Always,
        json: false,
        setup_timing: true,
    });
    eprintln!("Simulated setup progress; resize the terminal while it runs.");
    for (phase, label, total, unit) in [
        (
            "migrate",
            "Upgrading embedding storage",
            161_731,
            "passages",
        ),
        (
            "passages",
            "Splitting text into search passages",
            29_894,
            "records",
        ),
    ] {
        for step in 0..=20 {
            let current = total * step / 20;
            tracing::info!(target:"atlas_progress",phase,current,total,unit,"{label}");
            thread::sleep(Duration::from_millis(300));
        }
    }
    for (phase, label) in [
        ("reuse", "Reading indexed reference records for reuse"),
        ("snapshot", "Copying a consistent database snapshot"),
        (
            "validate",
            "Validating all reference records and stored vectors",
        ),
        ("bge", "Loading bge-base and testing vector search"),
        ("gemma", "Loading embeddinggemma and testing vector search"),
    ] {
        tracing::info!(target:"atlas_progress",phase,"{label}");
        thread::sleep(Duration::from_secs(2));
    }
    tracing::info!(target:"atlas_progress",complete=true,"Preview finished");
    println!("Preview finished; no database was read or changed.");
}
