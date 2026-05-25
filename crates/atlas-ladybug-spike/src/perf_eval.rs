use std::path::Path;

use serde_json::Value as JsonValue;

use crate::atlas_cli::run_atlas_json;

struct PerfCase {
    category: &'static str,
    name: &'static str,
    args: &'static [&'static str],
    note: &'static str,
    backend_support: BackendSupport,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BackendSupport {
    Both,
    SqliteOnly,
}

#[derive(Debug)]
struct BackendRunStats {
    samples_ms: Vec<u128>,
    median_ms: u128,
    min_ms: u128,
    max_ms: u128,
    summary: String,
    error: Option<String>,
}

impl BackendRunStats {
    fn not_applicable() -> Self {
        Self {
            samples_ms: Vec::new(),
            median_ms: 0,
            min_ms: 0,
            max_ms: 0,
            summary: "n/a".to_string(),
            error: None,
        }
    }
}

pub(crate) fn run_cli_performance_eval(
    sqlite_path: &Path,
    ladybug_path: &Path,
    atlas_bin: &Path,
    repeats: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let repeats = repeats.max(1);
    println!("Atlas CLI read-path performance evaluation");
    println!("==========================================");
    println!("sqlite:  {}", sqlite_path.display());
    println!("ladybug: {}", ladybug_path.display());
    println!("atlas:   {}", atlas_bin.display());
    println!("repeats: {repeats}");
    println!();
    println!(
        "Each case runs the real CLI with --json --progress never. Timings include CLI process startup, index open, query execution, hydration, and JSON rendering; build tasks are excluded."
    );

    for case in perf_cases() {
        let sqlite = run_case(
            case,
            "sqlite",
            sqlite_path,
            ladybug_path,
            atlas_bin,
            repeats,
        );
        let ladybug = if case.backend_support == BackendSupport::Both {
            run_case(
                case,
                "ladybug",
                sqlite_path,
                ladybug_path,
                atlas_bin,
                repeats,
            )
        } else {
            BackendRunStats::not_applicable()
        };
        print_case(case, &sqlite, &ladybug);
    }

    Ok(())
}

fn perf_cases() -> &'static [PerfCase] {
    &[
        PerfCase {
            category: "record",
            name: "get spell summary",
            args: &[
                "record",
                "get",
                "spells-srd:4koZzrnMXhhosn0D",
                "--detail",
                "summary",
            ],
            note: "Direct key lookup and summary hydration.",
            backend_support: BackendSupport::Both,
        },
        PerfCase {
            category: "record",
            name: "get spell full",
            args: &[
                "record",
                "get",
                "spells-srd:4koZzrnMXhhosn0D",
                "--detail",
                "full",
            ],
            note: "Direct key lookup with full presentation payload.",
            backend_support: BackendSupport::Both,
        },
        PerfCase {
            category: "record",
            name: "resolve spell",
            args: &["record", "resolve", "Fear", "--family", "spell"],
            note: "Strict name resolution plus record hydration.",
            backend_support: BackendSupport::Both,
        },
        PerfCase {
            category: "search-hybrid",
            name: "direct title",
            args: &[
                "search",
                "battle medicine",
                "--retrieval",
                "hybrid",
                "--limit",
                "10",
            ],
            note: "Precision title/alias plus vector hybrid search.",
            backend_support: BackendSupport::Both,
        },
        PerfCase {
            category: "search-hybrid",
            name: "semantic spell filtered",
            args: &[
                "search",
                "low level spell that makes enemies afraid",
                "--retrieval",
                "hybrid",
                "--family",
                "spell",
                "--max-level",
                "3",
                "--limit",
                "10",
            ],
            note: "Natural-language hybrid search with scalar filters.",
            backend_support: BackendSupport::Both,
        },
        PerfCase {
            category: "search-hybrid",
            name: "metric filtered creature",
            args: &[
                "search",
                "high armor creature",
                "--retrieval",
                "hybrid",
                "--family",
                "creature",
                "--metric",
                "ac.value>=25",
                "--limit",
                "10",
            ],
            note: "Hybrid search with relationship-backed numeric metric filter.",
            backend_support: BackendSupport::Both,
        },
        PerfCase {
            category: "search-hybrid",
            name: "reference filtered spell",
            args: &[
                "search",
                "frightened",
                "--retrieval",
                "hybrid",
                "--family",
                "spell",
                "--references",
                "conditionitems:TBSHQspnbcqxsmjL",
                "--limit",
                "10",
            ],
            note: "Hybrid search with graph/reference-derived filter.",
            backend_support: BackendSupport::Both,
        },
        PerfCase {
            category: "filter-list",
            name: "spell level page",
            args: &[
                "search",
                "--family",
                "spell",
                "--max-level",
                "3",
                "--sort",
                "record_key",
                "--limit",
                "20",
            ],
            note: "Filter-only listing with total remaining count.",
            backend_support: BackendSupport::Both,
        },
        PerfCase {
            category: "filter-list",
            name: "creature metric page",
            args: &[
                "search",
                "--family",
                "creature",
                "--metric",
                "ac.value>=25",
                "--sort",
                "record_key",
                "--limit",
                "20",
            ],
            note: "Filter-only listing with metric predicate.",
            backend_support: BackendSupport::Both,
        },
        PerfCase {
            category: "filter-list",
            name: "equipment price sort",
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
                "20",
            ],
            note: "Filter-only listing with range filter and sort.",
            backend_support: BackendSupport::Both,
        },
        PerfCase {
            category: "filter-fields",
            name: "available spell filter keys",
            args: &["filters", "fields", "--family", "spell"],
            note: "Available filter fields for a family.",
            backend_support: BackendSupport::Both,
        },
        PerfCase {
            category: "filter-fields",
            name: "remaining filter keys after spell filter",
            args: &[
                "filters",
                "fields",
                "--family",
                "spell",
                "--max-level",
                "3",
                "--trait",
                "healing",
            ],
            note: "Available fields after an active filter set.",
            backend_support: BackendSupport::Both,
        },
        PerfCase {
            category: "filter-values",
            name: "spell trait counts",
            args: &[
                "filters", "values", "--field", "traits", "--family", "spell", "--sort", "count",
            ],
            note: "Enumerable facet counts.",
            backend_support: BackendSupport::Both,
        },
        PerfCase {
            category: "filter-values",
            name: "remaining spell trait counts",
            args: &[
                "filters",
                "values",
                "--field",
                "traits",
                "--family",
                "spell",
                "--max-level",
                "3",
                "--sort",
                "count",
            ],
            note: "Remaining enumerable facet counts under active filters.",
            backend_support: BackendSupport::Both,
        },
        PerfCase {
            category: "filter-values-dynamic",
            name: "broad multi-family trait counts",
            args: &[
                "filters",
                "values",
                "--field",
                "traits",
                "--family",
                "creature",
                "--family",
                "equipment",
                "--family",
                "feat",
                "--sort",
                "count",
                "--disable-discovery-catalog",
            ],
            note: "Dynamic trait facet counts over several broad record families; forces SQLite off persisted discovery catalogs.",
            backend_support: BackendSupport::Both,
        },
        PerfCase {
            category: "filter-values",
            name: "equipment rarity counts",
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
            note: "Small enum-like value counts.",
            backend_support: BackendSupport::Both,
        },
        PerfCase {
            category: "filter-stats",
            name: "spell level stats",
            args: &["filters", "values", "--field", "level", "--family", "spell"],
            note: "Numeric stats for a scalar field.",
            backend_support: BackendSupport::Both,
        },
        PerfCase {
            category: "filter-stats",
            name: "remaining spell level stats",
            args: &[
                "filters", "values", "--field", "level", "--family", "spell", "--trait", "healing",
            ],
            note: "Numeric stats after an active filter set.",
            backend_support: BackendSupport::Both,
        },
        PerfCase {
            category: "filter-stats",
            name: "equipment price stats",
            args: &[
                "filters",
                "values",
                "--field",
                "price_cp",
                "--family",
                "equipment",
            ],
            note: "Numeric stats with nulls for equipment price.",
            backend_support: BackendSupport::Both,
        },
        PerfCase {
            category: "filter-stats",
            name: "spell sustained boolean counts",
            args: &[
                "filters",
                "values",
                "--field",
                "sustained",
                "--family",
                "spell",
            ],
            note: "Boolean value counts.",
            backend_support: BackendSupport::Both,
        },
        PerfCase {
            category: "metric-discovery",
            name: "creature armor metric keys",
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
            note: "Metric key discovery and matching counts.",
            backend_support: BackendSupport::Both,
        },
        PerfCase {
            category: "metric-discovery",
            name: "creature ac metric stats",
            args: &[
                "filters", "values", "--field", "metric", "--family", "creature", "--metric",
                "ac.value",
            ],
            note: "Selected metric numeric stats.",
            backend_support: BackendSupport::Both,
        },
        PerfCase {
            category: "graph",
            name: "spell graph context",
            args: &["graph", "get", "spells-srd:4koZzrnMXhhosn0D"],
            note: "One-hop graph context retrieval.",
            backend_support: BackendSupport::SqliteOnly,
        },
    ]
}

fn run_case(
    case: &PerfCase,
    backend: &str,
    sqlite_path: &Path,
    ladybug_path: &Path,
    atlas_bin: &Path,
    repeats: usize,
) -> BackendRunStats {
    let mut samples = Vec::new();
    let mut summary = String::new();
    let mut error = None;
    for _ in 0..repeats {
        let mut args = case
            .args
            .iter()
            .map(|arg| (*arg).to_string())
            .collect::<Vec<_>>();
        args.extend([
            "--index".to_string(),
            sqlite_path.display().to_string(),
            "--json".to_string(),
            "--progress".to_string(),
            "never".to_string(),
        ]);
        if case.backend_support == BackendSupport::Both {
            args.push("--index-backend".to_string());
            args.push(backend.to_string());
        }
        if backend == "ladybug" && case.backend_support == BackendSupport::Both {
            args.push("--ladybug-index".to_string());
            args.push(ladybug_path.display().to_string());
        }
        let run = run_atlas_json(atlas_bin, &args);
        samples.push(run.elapsed_ms);
        if let Some(run_error) = run.error {
            error = Some(run_error);
            summary = "error".to_string();
            break;
        }
        if let Some(json) = run.json.as_ref() {
            summary = summarize_result(json);
        }
    }
    samples.sort_unstable();
    let median_ms = samples
        .get(samples.len().saturating_sub(1) / 2)
        .copied()
        .unwrap_or(0);
    let min_ms = samples.first().copied().unwrap_or(0);
    let max_ms = samples.last().copied().unwrap_or(0);
    BackendRunStats {
        samples_ms: samples,
        median_ms,
        min_ms,
        max_ms,
        summary,
        error,
    }
}

fn print_case(case: &PerfCase, sqlite: &BackendRunStats, ladybug: &BackendRunStats) {
    println!();
    println!("## [{}] {}", case.category, case.name);
    println!("args: {}", case.args.join(" "));
    println!("note: {}", case.note);
    println!(
        "sqlite:  median={}ms min={}ms max={}ms samples={:?} {}",
        sqlite.median_ms, sqlite.min_ms, sqlite.max_ms, sqlite.samples_ms, sqlite.summary
    );
    println!(
        "ladybug: median={}ms min={}ms max={}ms samples={:?} {}",
        ladybug.median_ms, ladybug.min_ms, ladybug.max_ms, ladybug.samples_ms, ladybug.summary
    );
    if case.backend_support == BackendSupport::SqliteOnly {
        println!("backend note: this CLI surface does not currently expose --index-backend");
    }
    if let Some(error) = &sqlite.error {
        println!("sqlite error: {error}");
    }
    if let Some(error) = &ladybug.error {
        println!("ladybug error: {error}");
    }
    if case.backend_support == BackendSupport::Both
        && sqlite.error.is_none()
        && ladybug.error.is_none()
        && sqlite.median_ms > 0
    {
        println!(
            "ratio: ladybug/sqlite={:.2}x",
            ladybug.median_ms as f64 / sqlite.median_ms as f64
        );
    }
}

fn summarize_result(json: &JsonValue) -> String {
    if json.pointer("/status").and_then(JsonValue::as_str) != Some("ok") {
        return format!(
            "status={:?}",
            json.pointer("/status").and_then(JsonValue::as_str)
        );
    }
    if let Some(total) = json
        .pointer("/data/pagination/total")
        .and_then(JsonValue::as_u64)
    {
        let count = json
            .pointer("/data/pagination/count")
            .and_then(JsonValue::as_u64)
            .unwrap_or(0);
        return format!("ok total={total} count={count}");
    }
    if let Some(count) = json
        .pointer("/data/matching_record_count")
        .and_then(JsonValue::as_u64)
    {
        let fields = json
            .pointer("/data/fields")
            .and_then(JsonValue::as_array)
            .map(Vec::len);
        let values = json
            .pointer("/data/values/items")
            .or_else(|| json.pointer("/data/values"))
            .and_then(JsonValue::as_array)
            .map(Vec::len);
        let metric_count = json
            .pointer("/data/metric/count")
            .and_then(JsonValue::as_u64);
        return format!(
            "ok matching={count} fields={fields:?} values={values:?} metric_count={metric_count:?}"
        );
    }
    if let Some(record_key) = json.pointer("/data/record/key").and_then(JsonValue::as_str) {
        return format!("ok record={record_key}");
    }
    if let Some(record_key) = json
        .pointer("/data/result/record/key")
        .and_then(JsonValue::as_str)
    {
        return format!("ok resolved={record_key}");
    }
    if let Some(seed) = json.pointer("/data/seed/key").and_then(JsonValue::as_str) {
        let sections = json
            .pointer("/data/sections")
            .and_then(JsonValue::as_array)
            .map(Vec::len)
            .unwrap_or(0);
        return format!("ok graph_seed={seed} sections={sections}");
    }
    "ok".to_string()
}
