use std::process::Command;

mod support;

use support::json::parse_ok_data;
use support::path::temp_source_root;
use support::source::write_price_sort_source;

#[test]
fn filter_search_supports_price_sorting() -> Result<(), Box<dyn std::error::Error>> {
    let root = temp_source_root("cli-search-price-sort");
    write_price_sort_source(&root)?;
    let index_path = root.join("artifact.sqlite");
    let build_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["index", "build", "--source"])
        .arg(&root)
        .args(["--output"])
        .arg(&index_path)
        .arg("--no-embeddings")
        .arg("--json")
        .output()?;
    assert!(build_output.status.success());

    let ascending_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args([
            "search",
            "--family",
            "equipment",
            "--sort",
            "price_asc",
            "--index",
        ])
        .arg(&index_path)
        .arg("--json")
        .output()?;
    assert!(ascending_output.status.success());
    let ascending_data = parse_ok_data(&ascending_output)?;
    assert_eq!(ascending_data["sort"]["kind"], "price_asc");
    assert_eq!(
        ascending_data["results"][0]["record"]["key"],
        "equipment:cheapItem0001"
    );
    assert_eq!(
        ascending_data["results"][1]["record"]["key"],
        "equipment:expensiveItem"
    );

    let descending_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args([
            "search",
            "--family",
            "equipment",
            "--sort",
            "price_desc",
            "--index",
        ])
        .arg(&index_path)
        .arg("--json")
        .output()?;
    assert!(descending_output.status.success());
    let descending_data = parse_ok_data(&descending_output)?;
    assert_eq!(descending_data["sort"]["kind"], "price_desc");
    assert_eq!(
        descending_data["results"][0]["record"]["key"],
        "equipment:expensiveItem"
    );
    assert_eq!(
        descending_data["results"][1]["record"]["key"],
        "equipment:cheapItem0001"
    );

    std::fs::remove_dir_all(root)?;
    Ok(())
}
