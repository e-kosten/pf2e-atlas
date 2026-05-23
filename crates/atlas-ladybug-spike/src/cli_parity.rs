use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use serde_json::Value as JsonValue;

use crate::atlas_cli::{run_atlas_json, summarize_json};

struct CliParityCase {
    name: &'static str,
    args: &'static [&'static str],
    comparison: CliParityComparison,
    note: &'static str,
}

#[derive(Clone, Copy)]
enum CliParityComparison {
    SearchPage,
    FilterFields,
    FilterValues,
    NumericStats,
    BooleanCounts,
    MetricValues,
    SelectedPointers(&'static [&'static str]),
}

#[derive(Debug)]
struct CliParityRun {
    elapsed_ms: u128,
    json: Option<JsonValue>,
    error: Option<String>,
}

pub(crate) fn run_cli_parity(
    sqlite_path: &Path,
    ladybug_path: &Path,
    atlas_bin: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("LadybugDB non-FTS CLI parity evaluation");
    println!("=======================================");
    println!("sqlite:  {}", sqlite_path.display());
    println!("ladybug: {}", ladybug_path.display());
    println!("atlas:   {}", atlas_bin.display());
    println!();
    println!(
        "This mode focuses on non-FTS surfaces: record get, filter-only search, filter discovery, and vector search."
    );

    for case in cli_parity_cases() {
        let sqlite = run_cli_parity_case(case, "sqlite", sqlite_path, ladybug_path, atlas_bin);
        let ladybug = run_cli_parity_case(case, "ladybug", sqlite_path, ladybug_path, atlas_bin);
        print_cli_parity_case(case, &sqlite, &ladybug);
    }

    Ok(())
}

fn cli_parity_cases() -> &'static [CliParityCase] {
    &[
        CliParityCase {
            name: "record get full spell",
            args: &[
                "record",
                "get",
                "spells-srd:4koZzrnMXhhosn0D",
                "--detail",
                "full",
            ],
            comparison: CliParityComparison::SelectedPointers(&[
                "/data/detail",
                "/data/record/key",
                "/data/record/name",
                "/data/record/record_family",
                "/data/record/level",
                "/data/record/rarity",
                "/data/record/traits",
                "/data/record/source/publication_title",
                "/data/record/source/publication_family",
                "/data/record/source/publication_remaster",
                "/data/record/sections",
            ]),
            note: "Checks full direct record hydration for a spell, including sections and source metadata.",
        },
        CliParityCase {
            name: "record get preview equipment",
            args: &[
                "record",
                "get",
                "equipment-srd:00gDg8WcPv3TKC9N",
                "--detail",
                "preview",
            ],
            comparison: CliParityComparison::SelectedPointers(&[
                "/data/detail",
                "/data/record/key",
                "/data/record/name",
                "/data/record/record_family",
                "/data/record/level",
                "/data/record/rarity",
                "/data/record/source/publication_title",
                "/data/record/sections",
            ]),
            note: "Checks direct record hydration for equipment and preview-level presentation.",
        },
        CliParityCase {
            name: "record get standard creature",
            args: &[
                "record",
                "get",
                "abomination-vaults-bestiary:00s3MhFQ4yOp2rTf",
                "--detail",
                "standard",
            ],
            comparison: CliParityComparison::SelectedPointers(&[
                "/data/detail",
                "/data/record/key",
                "/data/record/name",
                "/data/record/record_family",
                "/data/record/level",
                "/data/record/traits",
                "/data/record/source/pack",
                "/data/record/sections",
            ]),
            note: "Checks direct record hydration for actor-style side data and standard sections.",
        },
        CliParityCase {
            name: "record get summary hazard",
            args: &[
                "record",
                "get",
                "abomination-vaults-bestiary:6NijRSpkIuQpSxUp",
                "--detail",
                "summary",
            ],
            comparison: CliParityComparison::SelectedPointers(&[
                "/data/detail",
                "/data/record/key",
                "/data/record/name",
                "/data/record/record_family",
                "/data/record/level",
                "/data/record/traits",
                "/data/record/sections",
            ]),
            note: "Checks direct record hydration for hazard records.",
        },
        CliParityCase {
            name: "record resolve exact spell",
            args: &[
                "record", "resolve", "Fear", "--family", "spell", "--detail", "summary",
            ],
            comparison: CliParityComparison::SelectedPointers(&[
                "/data/detail",
                "/data/result/query",
                "/data/result/record/key",
                "/data/result/record/name",
                "/data/result/record/record_family",
                "/data/result/resolution/match_kind",
                "/data/result/resolution/matched_text",
            ]),
            note: "Checks strict name resolution through the read backend.",
        },
        CliParityCase {
            name: "filter-only spell level page",
            args: &[
                "search",
                "--family",
                "spell",
                "--max-level",
                "3",
                "--sort",
                "record_key",
                "--limit",
                "10",
            ],
            comparison: CliParityComparison::SearchPage,
            note: "Filter-only list with scalar family and numeric level filters.",
        },
        CliParityCase {
            name: "filter-only spell pagination",
            args: &[
                "search",
                "--family",
                "spell",
                "--max-level",
                "3",
                "--sort",
                "record_key",
                "--offset",
                "20",
                "--limit",
                "10",
            ],
            comparison: CliParityComparison::SearchPage,
            note: "Filter-only pagination should preserve total count and page keys.",
        },
        CliParityCase {
            name: "filter-only healing spells",
            args: &[
                "search",
                "--family",
                "spell",
                "--max-level",
                "3",
                "--trait",
                "healing",
                "--sort",
                "record_key",
                "--limit",
                "10",
            ],
            comparison: CliParityComparison::SearchPage,
            note: "Filter-only list with relationship-backed trait filter.",
        },
        CliParityCase {
            name: "filter-only any-trait spell",
            args: &[
                "search",
                "--family",
                "spell",
                "--any-trait",
                "fire",
                "--any-trait",
                "cold",
                "--sort",
                "record_key",
                "--limit",
                "10",
            ],
            comparison: CliParityComparison::SearchPage,
            note: "Filter-only list with OR-style any-trait filtering.",
        },
        CliParityCase {
            name: "filter-only creature AC metric",
            args: &[
                "search",
                "--family",
                "creature",
                "--metric",
                "ac.value>=25",
                "--sort",
                "record_key",
                "--limit",
                "10",
            ],
            comparison: CliParityComparison::SearchPage,
            note: "Filter-only list with relationship-backed numeric metric filter.",
        },
        CliParityCase {
            name: "filter-only equipment price sort",
            args: &[
                "search",
                "--family",
                "equipment",
                "--min-price",
                "100",
                "--max-price",
                "1000",
                "--sort",
                "price_asc",
                "--limit",
                "10",
            ],
            comparison: CliParityComparison::SearchPage,
            note: "Filter-only list with price range and price sort.",
        },
        CliParityCase {
            name: "filter-only reference filter",
            args: &[
                "search",
                "--family",
                "spell",
                "--references",
                "conditionitems:TBSHQspnbcqxsmjL",
                "--sort",
                "record_key",
                "--limit",
                "10",
            ],
            comparison: CliParityComparison::SearchPage,
            note: "Filter-only list over graph/reference-derived filters.",
        },
        CliParityCase {
            name: "filter-only reverse reference filter",
            args: &[
                "search",
                "--family",
                "rule",
                "--referenced-by",
                "spells-srd:4koZzrnMXhhosn0D",
                "--sort",
                "record_key",
                "--limit",
                "10",
            ],
            comparison: CliParityComparison::SearchPage,
            note: "Filter-only list over reverse reference filters.",
        },
        CliParityCase {
            name: "filter fields for spells",
            args: &["filters", "fields", "--family", "spell"],
            comparison: CliParityComparison::FilterFields,
            note: "Compares available filter field IDs and matching record count.",
        },
        CliParityCase {
            name: "filter fields for creatures",
            args: &["filters", "fields", "--family", "creature"],
            comparison: CliParityComparison::FilterFields,
            note: "Compares available filter field IDs for a metric-heavy family.",
        },
        CliParityCase {
            name: "trait values for spells",
            args: &[
                "filters", "values", "--field", "traits", "--family", "spell", "--sort", "count",
            ],
            comparison: CliParityComparison::FilterValues,
            note: "Compares enumerable trait facet counts under a family filter.",
        },
        CliParityCase {
            name: "rarity values for equipment",
            args: &[
                "filters",
                "values",
                "--field",
                "rarity",
                "--family",
                "equipment",
                "--sort",
                "canonical",
            ],
            comparison: CliParityComparison::FilterValues,
            note: "Compares canonical-order enumerable values.",
        },
        CliParityCase {
            name: "publication values for spells",
            args: &[
                "filters",
                "values",
                "--field",
                "publication_title",
                "--family",
                "spell",
                "--sort",
                "alpha",
            ],
            comparison: CliParityComparison::FilterValues,
            note: "Compares high-cardinality enumerable publication values.",
        },
        CliParityCase {
            name: "level stats for spells",
            args: &["filters", "values", "--field", "level", "--family", "spell"],
            comparison: CliParityComparison::NumericStats,
            note: "Compares numeric field stats for spell level.",
        },
        CliParityCase {
            name: "price stats for equipment",
            args: &[
                "filters",
                "values",
                "--field",
                "price_cp",
                "--family",
                "equipment",
            ],
            comparison: CliParityComparison::NumericStats,
            note: "Compares numeric field stats with nulls for equipment price.",
        },
        CliParityCase {
            name: "sustained counts for spells",
            args: &[
                "filters",
                "values",
                "--field",
                "sustained",
                "--family",
                "spell",
            ],
            comparison: CliParityComparison::BooleanCounts,
            note: "Compares boolean filter counts.",
        },
        CliParityCase {
            name: "metric discovery armor creatures",
            args: &[
                "filters",
                "values",
                "--field",
                "metric",
                "--family",
                "creature",
                "--metric-query",
                "armor",
            ],
            comparison: CliParityComparison::MetricValues,
            note: "Compares metric-key discovery metadata and numeric stats.",
        },
        CliParityCase {
            name: "metric numeric stats armor creatures",
            args: &[
                "filters", "values", "--field", "metric", "--family", "creature", "--metric",
                "ac.value",
            ],
            comparison: CliParityComparison::SelectedPointers(&[
                "/data/matching_record_count",
                "/data/metric/namespace_prefix",
                "/data/metric/count",
                "/data/metric/numeric_stats/count",
                "/data/metric/numeric_stats/null_count",
                "/data/metric/numeric_stats/min",
                "/data/metric/numeric_stats/p50",
                "/data/metric/numeric_stats/max",
                "/data/values/stats/count",
                "/data/values/stats/null_count",
                "/data/values/stats/min",
                "/data/values/stats/p50",
                "/data/values/stats/max",
            ]),
            note: "Compares metric-value numeric stats for a concrete metric.",
        },
        CliParityCase {
            name: "vector spell concept with filters",
            args: &[
                "search",
                "low level spell that makes enemies afraid",
                "--retrieval",
                "vector",
                "--family",
                "spell",
                "--max-level",
                "3",
                "--limit",
                "10",
            ],
            comparison: CliParityComparison::SearchPage,
            note: "Vector-only semantic search with scalar structured filters.",
        },
        CliParityCase {
            name: "vector reference-derived filter",
            args: &[
                "search",
                "frightened",
                "--retrieval",
                "vector",
                "--family",
                "spell",
                "--references",
                "conditionitems:TBSHQspnbcqxsmjL",
                "--limit",
                "10",
            ],
            comparison: CliParityComparison::SearchPage,
            note: "Vector-only semantic search with graph/reference-derived filter.",
        },
    ]
}

fn run_cli_parity_case(
    case: &CliParityCase,
    backend: &str,
    sqlite_path: &Path,
    ladybug_path: &Path,
    atlas_bin: &Path,
) -> CliParityRun {
    let mut args = case
        .args
        .iter()
        .map(|arg| (*arg).to_string())
        .collect::<Vec<_>>();
    args.extend([
        "--index".to_string(),
        sqlite_path.display().to_string(),
        "--index-backend".to_string(),
        backend.to_string(),
        "--json".to_string(),
        "--progress".to_string(),
        "never".to_string(),
    ]);
    if backend == "ladybug" {
        args.push("--ladybug-index".to_string());
        args.push(ladybug_path.display().to_string());
    }

    let run = run_atlas_json(atlas_bin, &args);
    CliParityRun {
        elapsed_ms: run.elapsed_ms,
        json: run.json,
        error: run.error,
    }
}

fn print_cli_parity_case(case: &CliParityCase, sqlite: &CliParityRun, ladybug: &CliParityRun) {
    println!();
    println!("## {}", case.name);
    println!("args: {}", case.args.join(" "));
    println!("note: {}", case.note);
    println!("sqlite:  {}ms {}", sqlite.elapsed_ms, run_summary(sqlite));
    println!("ladybug: {}ms {}", ladybug.elapsed_ms, run_summary(ladybug));
    if let Some(error) = &sqlite.error {
        println!("sqlite error: {error}");
    }
    if let Some(error) = &ladybug.error {
        println!("ladybug error: {error}");
    }

    let result = compare_case(case.comparison, sqlite.json.as_ref(), ladybug.json.as_ref());
    println!("status: {}", result.status);
    for detail in result.details {
        println!("- {detail}");
    }
}

fn run_summary(run: &CliParityRun) -> String {
    if run.error.is_some() {
        return "error".to_string();
    }
    let Some(json) = &run.json else {
        return "missing-json".to_string();
    };
    match json.pointer("/status").and_then(JsonValue::as_str) {
        Some("ok") => "ok".to_string(),
        Some(status) => format!("status={status}"),
        None => "unknown-status".to_string(),
    }
}

struct CompareResult {
    status: &'static str,
    details: Vec<String>,
}

fn compare_case(
    comparison: CliParityComparison,
    sqlite: Option<&JsonValue>,
    ladybug: Option<&JsonValue>,
) -> CompareResult {
    let (Some(sqlite), Some(ladybug)) = (sqlite, ladybug) else {
        return CompareResult {
            status: "blocked",
            details: vec!["one or both backends did not return JSON".to_string()],
        };
    };
    if json_status(sqlite) != Some("ok") || json_status(ladybug) != Some("ok") {
        return CompareResult {
            status: "blocked",
            details: vec![format!(
                "non-ok status: sqlite={:?} ladybug={:?}",
                json_status(sqlite),
                json_status(ladybug)
            )],
        };
    }

    match comparison {
        CliParityComparison::SearchPage => compare_search_page(sqlite, ladybug),
        CliParityComparison::FilterFields => compare_filter_fields(sqlite, ladybug),
        CliParityComparison::FilterValues => compare_filter_values(sqlite, ladybug),
        CliParityComparison::NumericStats => compare_numeric_stats(sqlite, ladybug),
        CliParityComparison::BooleanCounts => compare_boolean_counts(sqlite, ladybug),
        CliParityComparison::MetricValues => compare_metric_values(sqlite, ladybug),
        CliParityComparison::SelectedPointers(pointers) => {
            compare_selected_pointers(sqlite, ladybug, pointers)
        }
    }
}

fn compare_search_page(sqlite: &JsonValue, ladybug: &JsonValue) -> CompareResult {
    let sqlite_total = u64_at(sqlite, "/data/pagination/total");
    let ladybug_total = u64_at(ladybug, "/data/pagination/total");
    let sqlite_keys = result_keys(sqlite);
    let ladybug_keys = result_keys(ladybug);
    let mut details = vec![
        format!("total: sqlite={sqlite_total:?} ladybug={ladybug_total:?}"),
        format!("keys sqlite={}", sqlite_keys.join(", ")),
        format!("keys ladybug={}", ladybug_keys.join(", ")),
    ];
    let status = if sqlite_total == ladybug_total && sqlite_keys == ladybug_keys {
        "parity-clean"
    } else {
        details.push(format!(
            "top overlap: {}/{}",
            ordered_overlap(&sqlite_keys, &ladybug_keys),
            sqlite_keys.len().max(ladybug_keys.len())
        ));
        "parity-gap"
    };
    CompareResult { status, details }
}

fn compare_filter_fields(sqlite: &JsonValue, ladybug: &JsonValue) -> CompareResult {
    let sqlite_count = u64_at(sqlite, "/data/matching_record_count");
    let ladybug_count = u64_at(ladybug, "/data/matching_record_count");
    let sqlite_fields = field_ids(sqlite);
    let ladybug_fields = field_ids(ladybug);
    set_compare_result(
        "fields",
        sqlite_count,
        ladybug_count,
        sqlite_fields,
        ladybug_fields,
    )
}

fn compare_filter_values(sqlite: &JsonValue, ladybug: &JsonValue) -> CompareResult {
    let sqlite_count = u64_at(sqlite, "/data/matching_record_count");
    let ladybug_count = u64_at(ladybug, "/data/matching_record_count");
    let sqlite_values = value_counts(sqlite);
    let ladybug_values = value_counts(ladybug);
    map_compare_result(
        "values",
        sqlite_count,
        ladybug_count,
        sqlite_values,
        ladybug_values,
    )
}

fn compare_metric_values(sqlite: &JsonValue, ladybug: &JsonValue) -> CompareResult {
    let sqlite_count = u64_at(sqlite, "/data/matching_record_count");
    let ladybug_count = u64_at(ladybug, "/data/matching_record_count");
    let sqlite_metrics = metric_counts(sqlite);
    let ladybug_metrics = metric_counts(ladybug);
    map_compare_result(
        "metrics",
        sqlite_count,
        ladybug_count,
        sqlite_metrics,
        ladybug_metrics,
    )
}

fn compare_numeric_stats(sqlite: &JsonValue, ladybug: &JsonValue) -> CompareResult {
    compare_selected_pointers(
        sqlite,
        ladybug,
        &[
            "/data/matching_record_count",
            "/data/value_policy",
            "/data/stats/count",
            "/data/stats/null_count",
            "/data/stats/min",
            "/data/stats/p05",
            "/data/stats/p25",
            "/data/stats/p50",
            "/data/stats/mean",
            "/data/stats/p75",
            "/data/stats/p95",
            "/data/stats/max",
        ],
    )
}

fn compare_boolean_counts(sqlite: &JsonValue, ladybug: &JsonValue) -> CompareResult {
    compare_selected_pointers(
        sqlite,
        ladybug,
        &[
            "/data/matching_record_count",
            "/data/value_policy",
            "/data/counts/true",
            "/data/counts/false",
            "/data/counts/null",
        ],
    )
}

fn compare_selected_pointers(
    sqlite: &JsonValue,
    ladybug: &JsonValue,
    pointers: &[&str],
) -> CompareResult {
    let mut details = Vec::new();
    let mut clean = true;
    for pointer in pointers {
        let sqlite_value = sqlite.pointer(pointer);
        let ladybug_value = ladybug.pointer(pointer);
        if sqlite_value == ladybug_value {
            details.push(format!("{pointer}: match"));
        } else {
            clean = false;
            details.push(format!(
                "{pointer}: sqlite={} ladybug={}",
                sqlite_value
                    .map(summarize_json)
                    .unwrap_or_else(|| "<missing>".to_string()),
                ladybug_value
                    .map(summarize_json)
                    .unwrap_or_else(|| "<missing>".to_string())
            ));
        }
    }
    CompareResult {
        status: if clean { "parity-clean" } else { "parity-gap" },
        details,
    }
}

fn set_compare_result(
    label: &str,
    sqlite_count: Option<u64>,
    ladybug_count: Option<u64>,
    sqlite_values: BTreeSet<String>,
    ladybug_values: BTreeSet<String>,
) -> CompareResult {
    let missing_in_ladybug = sqlite_values
        .difference(&ladybug_values)
        .cloned()
        .collect::<Vec<_>>();
    let extra_in_ladybug = ladybug_values
        .difference(&sqlite_values)
        .cloned()
        .collect::<Vec<_>>();
    let clean = sqlite_count == ladybug_count
        && missing_in_ladybug.is_empty()
        && extra_in_ladybug.is_empty();
    CompareResult {
        status: if clean { "parity-clean" } else { "parity-gap" },
        details: vec![
            format!("matching_record_count: sqlite={sqlite_count:?} ladybug={ladybug_count:?}"),
            format!(
                "{label}: sqlite={} ladybug={}",
                sqlite_values.len(),
                ladybug_values.len()
            ),
            format!(
                "missing_in_ladybug={}",
                summarize_strings(&missing_in_ladybug)
            ),
            format!("extra_in_ladybug={}", summarize_strings(&extra_in_ladybug)),
        ],
    }
}

fn map_compare_result(
    label: &str,
    sqlite_count: Option<u64>,
    ladybug_count: Option<u64>,
    sqlite_values: BTreeMap<String, String>,
    ladybug_values: BTreeMap<String, String>,
) -> CompareResult {
    let sqlite_keys = sqlite_values.keys().cloned().collect::<BTreeSet<_>>();
    let ladybug_keys = ladybug_values.keys().cloned().collect::<BTreeSet<_>>();
    let missing_in_ladybug = sqlite_keys
        .difference(&ladybug_keys)
        .cloned()
        .collect::<Vec<_>>();
    let extra_in_ladybug = ladybug_keys
        .difference(&sqlite_keys)
        .cloned()
        .collect::<Vec<_>>();
    let differing = sqlite_keys
        .intersection(&ladybug_keys)
        .filter(|key| sqlite_values.get(*key) != ladybug_values.get(*key))
        .cloned()
        .collect::<Vec<_>>();
    let clean = sqlite_count == ladybug_count
        && missing_in_ladybug.is_empty()
        && extra_in_ladybug.is_empty()
        && differing.is_empty();
    CompareResult {
        status: if clean { "parity-clean" } else { "parity-gap" },
        details: vec![
            format!("matching_record_count: sqlite={sqlite_count:?} ladybug={ladybug_count:?}"),
            format!(
                "{label}: sqlite={} ladybug={}",
                sqlite_values.len(),
                ladybug_values.len()
            ),
            format!(
                "missing_in_ladybug={}",
                summarize_strings(&missing_in_ladybug)
            ),
            format!("extra_in_ladybug={}", summarize_strings(&extra_in_ladybug)),
            format!("differing_counts={}", summarize_strings(&differing)),
        ],
    }
}

fn json_status(json: &JsonValue) -> Option<&str> {
    json.pointer("/status").and_then(JsonValue::as_str)
}

fn u64_at(json: &JsonValue, pointer: &str) -> Option<u64> {
    json.pointer(pointer).and_then(JsonValue::as_u64)
}

fn result_keys(json: &JsonValue) -> Vec<String> {
    json.pointer("/data/results")
        .and_then(JsonValue::as_array)
        .into_iter()
        .flatten()
        .filter_map(|item| {
            item.pointer("/record/key")
                .and_then(JsonValue::as_str)
                .map(ToString::to_string)
        })
        .collect()
}

fn field_ids(json: &JsonValue) -> BTreeSet<String> {
    json.pointer("/data/fields")
        .and_then(JsonValue::as_array)
        .into_iter()
        .flatten()
        .filter_map(|item| {
            item.pointer("/field")
                .and_then(JsonValue::as_str)
                .map(ToString::to_string)
        })
        .collect()
}

fn value_counts(json: &JsonValue) -> BTreeMap<String, String> {
    json.pointer("/data/values")
        .and_then(JsonValue::as_array)
        .into_iter()
        .flatten()
        .filter_map(|item| {
            let value = item.pointer("/value").and_then(JsonValue::as_str)?;
            let count = item.pointer("/count").and_then(JsonValue::as_u64)?;
            Some((value.to_string(), count.to_string()))
        })
        .collect()
}

fn metric_counts(json: &JsonValue) -> BTreeMap<String, String> {
    json.pointer("/data/metrics")
        .and_then(JsonValue::as_array)
        .into_iter()
        .flatten()
        .filter_map(|item| {
            let key = item.pointer("/metric_key").and_then(JsonValue::as_str)?;
            let count = item.pointer("/count").and_then(JsonValue::as_u64)?;
            let min = item
                .pointer("/numeric_stats/min")
                .and_then(JsonValue::as_f64)
                .map(|value| value.to_string())
                .unwrap_or_else(|| "none".to_string());
            let max = item
                .pointer("/numeric_stats/max")
                .and_then(JsonValue::as_f64)
                .map(|value| value.to_string())
                .unwrap_or_else(|| "none".to_string());
            Some((
                key.to_string(),
                format!("count={count};min={min};max={max}"),
            ))
        })
        .collect()
}

fn ordered_overlap(left: &[String], right: &[String]) -> usize {
    left.iter()
        .filter(|key| right.iter().any(|right_key| right_key == *key))
        .count()
}

fn summarize_strings(values: &[String]) -> String {
    if values.is_empty() {
        return "none".to_string();
    }
    let mut summary = values
        .iter()
        .take(8)
        .cloned()
        .collect::<Vec<_>>()
        .join(", ");
    if values.len() > 8 {
        summary.push_str(&format!(" ... +{}", values.len() - 8));
    }
    summary
}
