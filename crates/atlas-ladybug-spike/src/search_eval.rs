use std::path::Path;

use serde_json::Value as JsonValue;

use crate::atlas_cli::{run_atlas_json, summarize_json};

struct SearchEvalCase {
    name: &'static str,
    query: &'static str,
    retrieval: &'static str,
    extra_args: &'static [&'static str],
    expected_keys: &'static [&'static str],
    note: &'static str,
}

#[derive(Debug)]
struct SearchEvalRun {
    elapsed_ms: u128,
    total: Option<u64>,
    keys: Vec<String>,
    names: Vec<String>,
    expected_positions: Vec<Option<usize>>,
    error: Option<String>,
}

pub(crate) fn run_search_eval(
    sqlite_path: &Path,
    ladybug_path: &Path,
    atlas_bin: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let cases = search_eval_cases();
    println!("LadybugDB search parity and quality evaluation");
    println!("==============================================");
    println!("sqlite:  {}", sqlite_path.display());
    println!("ladybug: {}", ladybug_path.display());
    println!("atlas:   {}", atlas_bin.display());
    println!();
    println!(
        "Timing is end-to-end CLI wall time, including process startup and embedding model load for vector/hybrid queries."
    );
    println!("Overlap is top-10 key overlap between SQLite and Ladybug for the same query.");

    for case in cases {
        let sqlite = run_search_eval_case(case, "sqlite", sqlite_path, ladybug_path, atlas_bin);
        let ladybug = run_search_eval_case(case, "ladybug", sqlite_path, ladybug_path, atlas_bin);
        print_search_eval_case(case, &sqlite, &ladybug);
    }
    Ok(())
}

fn search_eval_cases() -> &'static [SearchEvalCase] {
    &[
        SearchEvalCase {
            name: "direct lexical title",
            query: "battle medicine",
            retrieval: "fts",
            extra_args: &[],
            expected_keys: &["feats-srd:wYerMk6F1RZb0Fwt"],
            note: "Direct title hit should stay first; tests identity resolution plus FTS fallback ordering.",
        },
        SearchEvalCase {
            name: "filtered lexical spell",
            query: "fear",
            retrieval: "fts",
            extra_args: &["--family", "spell", "--max-level", "3"],
            expected_keys: &["spells-srd:4koZzrnMXhhosn0D"],
            note: "FTS with family + numeric filter; expected direct spell remains first.",
        },
        SearchEvalCase {
            name: "semantic concept spell",
            query: "low level spell that makes enemies afraid",
            retrieval: "vector",
            extra_args: &["--family", "spell", "--max-level", "3"],
            expected_keys: &["spells-srd:1xLVcA8Y1onw7toT", "spells-srd:4koZzrnMXhhosn0D"],
            note: "Semantic-only concept query with structural filtering; checks prefiltered vector behavior.",
        },
        SearchEvalCase {
            name: "hybrid concept spell",
            query: "low level spell that makes enemies afraid",
            retrieval: "hybrid",
            extra_args: &["--family", "spell", "--max-level", "3"],
            expected_keys: &["spells-srd:1xLVcA8Y1onw7toT", "spells-srd:4koZzrnMXhhosn0D"],
            note: "Hybrid query where weak OR-style FTS can add noise; compare ranking against vector lane.",
        },
        SearchEvalCase {
            name: "trait structured hybrid",
            query: "healing spell",
            retrieval: "hybrid",
            extra_args: &[
                "--family",
                "spell",
                "--max-level",
                "3",
                "--trait",
                "healing",
            ],
            expected_keys: &["spells-srd:qvwIwJ9QBihy8N7L"],
            note: "Family + level + trait filter under hybrid search.",
        },
        SearchEvalCase {
            name: "trait structured vector",
            query: "healing spell",
            retrieval: "vector",
            extra_args: &[
                "--family",
                "spell",
                "--max-level",
                "3",
                "--trait",
                "healing",
            ],
            expected_keys: &["spells-srd:qvwIwJ9QBihy8N7L"],
            note: "Family + level + trait filter under vector-only search; isolates semantic/filter parity before fusion.",
        },
        SearchEvalCase {
            name: "metric structured hybrid",
            query: "high armor creature",
            retrieval: "hybrid",
            extra_args: &["--family", "creature", "--metric", "ac.value>=25"],
            expected_keys: &[],
            note: "Metric filter under hybrid search; quality is judged by plausible creature-only high-AC results and backend agreement.",
        },
        SearchEvalCase {
            name: "metric structured vector",
            query: "high armor creature",
            retrieval: "vector",
            extra_args: &["--family", "creature", "--metric", "ac.value>=25"],
            expected_keys: &[],
            note: "Metric filter under vector-only search; isolates semantic/filter parity before fusion.",
        },
        SearchEvalCase {
            name: "publication remaster lexical",
            query: "fear",
            retrieval: "fts",
            extra_args: &[
                "--family",
                "spell",
                "--publication-title",
                "Pathfinder Player Core",
            ],
            expected_keys: &["spells-srd:4koZzrnMXhhosn0D"],
            note: "Publication filter plus lexical query.",
        },
        SearchEvalCase {
            name: "publication remaster vector",
            query: "fear",
            retrieval: "vector",
            extra_args: &[
                "--family",
                "spell",
                "--publication-title",
                "Pathfinder Player Core",
            ],
            expected_keys: &["spells-srd:4koZzrnMXhhosn0D"],
            note: "Publication filter plus vector-only search; isolates semantic/filter parity before FTS or fusion.",
        },
        SearchEvalCase {
            name: "reference-derived filter",
            query: "frightened",
            retrieval: "fts",
            extra_args: &[
                "--family",
                "spell",
                "--references",
                "conditionitems:TBSHQspnbcqxsmjL",
            ],
            expected_keys: &[],
            note: "Graph/reference-derived filter applied to FTS; useful for checking feature parity and query shape.",
        },
        SearchEvalCase {
            name: "reference-derived vector",
            query: "frightened",
            retrieval: "vector",
            extra_args: &[
                "--family",
                "spell",
                "--references",
                "conditionitems:TBSHQspnbcqxsmjL",
            ],
            expected_keys: &[],
            note: "Graph/reference-derived filter under vector-only search; isolates semantic/filter parity before FTS or fusion.",
        },
        SearchEvalCase {
            name: "long action semantic vector",
            query: "use a skill to earn money during downtime",
            retrieval: "vector",
            extra_args: &[],
            expected_keys: &["actionspf2e:QyzlsLrqM0EEwd7j"],
            note: "Known over-limit parent document; checks that long action text still retrieves Earn Income before chunking changes.",
        },
        SearchEvalCase {
            name: "long class feature semantic vector",
            query: "bard spell repertoire occult cantrips",
            retrieval: "vector",
            extra_args: &[],
            expected_keys: &["classfeatures:1RfnAiyQ5FR7vnuH"],
            note: "Worst over-limit parent from chunk diagnostics; checks long class feature semantics survive truncation.",
        },
        SearchEvalCase {
            name: "long inventor feature semantic vector",
            query: "revolutionary armor modification armor innovation",
            retrieval: "vector",
            extra_args: &[],
            expected_keys: &["classfeatures:tXbadIT3LzwuSR19"],
            note: "Long class feature with detailed mechanical text; expected record should remain a direct semantic hit.",
        },
        SearchEvalCase {
            name: "long feat semantic vector",
            query: "soulforged armament essence powers",
            retrieval: "vector",
            extra_args: &["--family", "feat"],
            expected_keys: &["feats-srd:SoocjFrWNOpchTVb"],
            note: "Parent and heading-section units are both over-limit; checks long feat and child-section embedding behavior.",
        },
        SearchEvalCase {
            name: "long spell semantic vector",
            query: "transform into an avatar of your deity huge battle form",
            retrieval: "vector",
            extra_args: &["--family", "spell"],
            expected_keys: &["spells-srd:ckUOoqOM7Kg7VqxB"],
            note: "Known over-limit spell parent; checks that long spell mechanics still retrieve Avatar.",
        },
        SearchEvalCase {
            name: "long creature semantic vector",
            query: "frost drake frozen reaches hunt caribou wolves",
            retrieval: "vector",
            extra_args: &["--family", "creature"],
            expected_keys: &["pathfinder-monster-core:2SixuEUfKpEyfOEY"],
            note: "Known over-limit creature parent; checks that long creature description/lore still retrieves Frost Drake.",
        },
    ]
}

fn run_search_eval_case(
    case: &SearchEvalCase,
    backend: &str,
    sqlite_path: &Path,
    ladybug_path: &Path,
    atlas_bin: &Path,
) -> SearchEvalRun {
    let mut args = vec![
        "search".to_string(),
        case.query.to_string(),
        "--index".to_string(),
        sqlite_path.display().to_string(),
        "--index-backend".to_string(),
        backend.to_string(),
        "--retrieval".to_string(),
        case.retrieval.to_string(),
        "--limit".to_string(),
        "10".to_string(),
        "--detail".to_string(),
        "summary".to_string(),
        "--json".to_string(),
        "--progress".to_string(),
        "never".to_string(),
        "--explain".to_string(),
    ];
    if backend == "ladybug" {
        args.push("--ladybug-index".to_string());
        args.push(ladybug_path.display().to_string());
    }
    args.extend(case.extra_args.iter().map(|arg| (*arg).to_string()));

    let run = run_atlas_json(atlas_bin, &args);
    if let Some(error) = run.error {
        return SearchEvalRun {
            elapsed_ms: run.elapsed_ms,
            total: None,
            keys: Vec::new(),
            names: Vec::new(),
            expected_positions: Vec::new(),
            error: Some(error),
        };
    }
    let Some(json) = run.json else {
        return SearchEvalRun {
            elapsed_ms: run.elapsed_ms,
            total: None,
            keys: Vec::new(),
            names: Vec::new(),
            expected_positions: Vec::new(),
            error: Some("atlas JSON runner returned neither JSON nor error".to_string()),
        };
    };
    parse_search_eval_json(case, run.elapsed_ms, &json)
}

fn parse_search_eval_json(
    case: &SearchEvalCase,
    elapsed_ms: u128,
    json: &JsonValue,
) -> SearchEvalRun {
    if json.pointer("/status").and_then(JsonValue::as_str) != Some("ok") {
        return SearchEvalRun {
            elapsed_ms,
            total: None,
            keys: Vec::new(),
            names: Vec::new(),
            expected_positions: Vec::new(),
            error: Some(format!("non-ok JSON envelope: {}", summarize_json(json))),
        };
    }
    let results = json
        .pointer("/data/results")
        .and_then(JsonValue::as_array)
        .cloned()
        .unwrap_or_default();
    let keys = results
        .iter()
        .filter_map(|item| {
            item.pointer("/record/key")
                .and_then(JsonValue::as_str)
                .map(ToString::to_string)
        })
        .collect::<Vec<_>>();
    let names = results
        .iter()
        .filter_map(|item| {
            item.pointer("/record/name")
                .and_then(JsonValue::as_str)
                .map(ToString::to_string)
        })
        .collect::<Vec<_>>();
    let expected_positions = case
        .expected_keys
        .iter()
        .map(|expected| {
            keys.iter()
                .position(|actual| actual == expected)
                .map(|index| index + 1)
        })
        .collect::<Vec<_>>();
    SearchEvalRun {
        elapsed_ms,
        total: json
            .pointer("/data/pagination/total")
            .and_then(JsonValue::as_u64),
        keys,
        names,
        expected_positions,
        error: None,
    }
}

fn print_search_eval_case(case: &SearchEvalCase, sqlite: &SearchEvalRun, ladybug: &SearchEvalRun) {
    println!();
    println!("## {}", case.name);
    println!("query: {:?}", case.query);
    println!("retrieval: {}", case.retrieval);
    if !case.extra_args.is_empty() {
        println!("filters/options: {}", case.extra_args.join(" "));
    }
    println!("note: {}", case.note);
    println!(
        "sqlite:  {}ms total={} top={}",
        sqlite.elapsed_ms,
        optional_u64(sqlite.total),
        summarize_search_run(sqlite)
    );
    println!(
        "ladybug: {}ms total={} top={}",
        ladybug.elapsed_ms,
        optional_u64(ladybug.total),
        summarize_search_run(ladybug)
    );
    if let Some(error) = &sqlite.error {
        println!("sqlite error: {error}");
    }
    if let Some(error) = &ladybug.error {
        println!("ladybug error: {error}");
    }
    if sqlite.error.is_none() && ladybug.error.is_none() {
        println!(
            "top10 overlap: {}",
            top_overlap(&sqlite.keys, &ladybug.keys)
        );
        if !case.expected_keys.is_empty() {
            println!(
                "expected positions: sqlite={} ladybug={}",
                format_expected_positions(case, sqlite),
                format_expected_positions(case, ladybug)
            );
        }
    }
}

fn summarize_search_run(run: &SearchEvalRun) -> String {
    run.keys
        .iter()
        .zip(run.names.iter())
        .take(5)
        .enumerate()
        .map(|(index, (key, name))| format!("{}. {} ({})", index + 1, name, key))
        .collect::<Vec<_>>()
        .join("; ")
}

fn optional_u64(value: Option<u64>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

fn top_overlap(left: &[String], right: &[String]) -> usize {
    left.iter()
        .filter(|key| right.iter().any(|right_key| right_key == *key))
        .count()
}

fn format_expected_positions(case: &SearchEvalCase, run: &SearchEvalRun) -> String {
    case.expected_keys
        .iter()
        .zip(run.expected_positions.iter())
        .map(|(key, position)| match position {
            Some(position) => format!("{key}@{position}"),
            None => format!("{key}@missing"),
        })
        .collect::<Vec<_>>()
        .join(", ")
}
