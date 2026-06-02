use serde_json::json;

mod support;

use support::command::{atlas_json, build_index};
use support::json::{parse_json, parse_ok_data};
use support::path::temp_source_root;
use support::source::write_metric_source;

#[test]
fn metric_query_and_short_label_filters_use_catalog_resolution()
-> Result<(), Box<dyn std::error::Error>> {
    let root = temp_source_root("cli-search-metric-resolution");
    write_metric_source(&root)?;
    let index_path = build_index(&root)?;

    let metric_query = atlas_json(&[
        "filters",
        "values",
        "--field",
        "metric",
        "--family",
        "creature",
        "--metric-query",
        "defense ac",
        "--index",
        index_path.to_str().unwrap(),
    ])?;
    assert!(metric_query.status.success());
    let metric_query_data = parse_ok_data(&metric_query)?;
    let armor_class = metric_query_data["metrics"]
        .as_array()
        .unwrap()
        .iter()
        .find(|metric| metric["metric_key"] == "ac.value")
        .expect("armor class metric should be discoverable by query");
    assert_eq!(armor_class["label"], "Armor Class");
    assert_eq!(armor_class["short_label"], "AC");
    assert_eq!(armor_class["group"], "defense");

    let metric_short_label_values = atlas_json(&[
        "filters",
        "values",
        "--field",
        "metric",
        "--family",
        "creature",
        "--metric",
        "AC",
        "--index",
        index_path.to_str().unwrap(),
    ])?;
    assert!(metric_short_label_values.status.success());
    let metric_short_label_data = parse_ok_data(&metric_short_label_values)?;
    assert_eq!(metric_short_label_data["metric"]["metric_key"], "ac.value");
    assert_eq!(
        metric_short_label_data["values"]["stats"]["p50"],
        json!(17.0)
    );

    let metric_filter_search = atlas_json(&[
        "search",
        "--family",
        "creature",
        "--metric",
        "AC>=17",
        "--index",
        index_path.to_str().unwrap(),
    ])?;
    assert!(metric_filter_search.status.success());
    let metric_filter_data = parse_ok_data(&metric_filter_search)?;
    assert_eq!(metric_filter_data["pagination"]["total"], 1);
    assert_eq!(
        metric_filter_data["results"][0]["record"]["key"],
        "bestiary:testActor0001"
    );

    let unknown_metric_filter = atlas_json(&[
        "search",
        "--family",
        "creature",
        "--metric",
        "armor>=17",
        "--index",
        index_path.to_str().unwrap(),
    ])?;
    assert_eq!(unknown_metric_filter.status.code(), Some(3));
    let unknown_metric_json = parse_json(&unknown_metric_filter)?;
    assert_eq!(unknown_metric_json["error"]["code"], "invalid_filter");
    let unknown_metric_message = unknown_metric_json["error"]["message"].as_str().unwrap();
    assert!(unknown_metric_message.contains("unknown metric `armor`"));
    assert!(unknown_metric_message.contains("ac.value"));
    assert!(unknown_metric_message.contains("--metric-query armor"));

    std::fs::remove_dir_all(root)?;
    Ok(())
}
