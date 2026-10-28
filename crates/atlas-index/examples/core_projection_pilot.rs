//! Explicit, rollback-only operational diagnostic; never a product query path.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args_os().skip(1);
    let database = args.next().ok_or("expected task-copy database path")?;
    let output = args.next().ok_or("expected JSON receipt path")?;
    if args.next().is_some() {
        return Err("expected exactly database and output paths".into());
    }
    let report = atlas_index::test_support::core_projection_pilot(std::path::Path::new(&database))?;
    std::fs::write(output, serde_json::to_vec_pretty(&report)?)?;
    println!(
        "{}",
        serde_json::json!({"records":report["records"],"duration_ms":report["duration_ms"],"rolled_back":true})
    );
    Ok(())
}
