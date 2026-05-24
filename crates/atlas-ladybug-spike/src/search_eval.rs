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

struct SearchQualityCase {
    name: &'static str,
    query: &'static str,
    extra_args: &'static [&'static str],
    note: &'static str,
}

struct SearchRunRequest<'a> {
    query: &'a str,
    retrieval: &'a str,
    extra_args: &'a [&'a str],
    backend: &'a str,
    sqlite_path: &'a Path,
    ladybug_path: &'a Path,
    atlas_bin: &'a Path,
    expected_keys: &'a [&'a str],
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

pub(crate) fn run_search_quality_matrix(
    sqlite_path: &Path,
    ladybug_path: &Path,
    atlas_bin: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let cases = search_quality_cases();
    println!("LadybugDB search quality matrix");
    println!("===============================");
    println!("sqlite:  {}", sqlite_path.display());
    println!("ladybug: {}", ladybug_path.display());
    println!("atlas:   {}", atlas_bin.display());
    println!();
    println!(
        "Each case is run through SQLite hybrid, Ladybug hybrid, SQLite vector, and Ladybug vector."
    );
    println!(
        "The matrix reports returned results, overlap, and timing; it intentionally does not encode expected result positions."
    );

    for case in cases {
        let sqlite_hybrid = run_search_case(SearchRunRequest {
            query: case.query,
            retrieval: "hybrid",
            extra_args: case.extra_args,
            backend: "sqlite",
            sqlite_path,
            ladybug_path,
            atlas_bin,
            expected_keys: &[],
        });
        let ladybug_hybrid = run_search_case(SearchRunRequest {
            query: case.query,
            retrieval: "hybrid",
            extra_args: case.extra_args,
            backend: "ladybug",
            sqlite_path,
            ladybug_path,
            atlas_bin,
            expected_keys: &[],
        });
        let sqlite_vector = run_search_case(SearchRunRequest {
            query: case.query,
            retrieval: "vector",
            extra_args: case.extra_args,
            backend: "sqlite",
            sqlite_path,
            ladybug_path,
            atlas_bin,
            expected_keys: &[],
        });
        let ladybug_vector = run_search_case(SearchRunRequest {
            query: case.query,
            retrieval: "vector",
            extra_args: case.extra_args,
            backend: "ladybug",
            sqlite_path,
            ladybug_path,
            atlas_bin,
            expected_keys: &[],
        });
        print_search_quality_case(
            case,
            &sqlite_hybrid,
            &ladybug_hybrid,
            &sqlite_vector,
            &ladybug_vector,
        );
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
            note: "Default hybrid query; weak lexical votes are demoted so semantic ordering should survive broad query text.",
        },
        SearchEvalCase {
            name: "hybrid concept spell all FTS",
            query: "low level spell that makes enemies afraid",
            retrieval: "hybrid",
            extra_args: &[
                "--family",
                "spell",
                "--max-level",
                "3",
                "--fts-fusion-policy",
                "all",
            ],
            expected_keys: &["spells-srd:1xLVcA8Y1onw7toT", "spells-srd:4koZzrnMXhhosn0D"],
            note: "Explicit old-style policy; useful for spotting when weak OR-style FTS noise can overpower semantic matches.",
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
            name: "long title partial hazard",
            query: "house spirits regenerating pies",
            retrieval: "hybrid",
            extra_args: &["--family", "hazard"],
            expected_keys: &["pfs-season-4-bestiary:w0empC43Ni3K4IQN"],
            note: "Long-title partial token match; checks whether title evidence can recover a record when stopwords and adjectives are omitted.",
        },
        SearchEvalCase {
            name: "long title distinctive hazard",
            query: "absurd regenerating pies",
            retrieval: "hybrid",
            extra_args: &["--family", "hazard"],
            expected_keys: &["pfs-season-4-bestiary:ETL4tgrSRxPS5ooV"],
            note: "Long-title distinctive partial match; expected title shares rare tokens with the query.",
        },
        SearchEvalCase {
            name: "long title partial affliction",
            query: "hunting spider venom first magazine",
            retrieval: "hybrid",
            extra_args: &["--family", "affliction"],
            expected_keys: &["derived-affliction-instances:2206b9d6"],
            note: "Long generated affliction title with parenthetical detail; checks omitted filler words and partial parenthetical matching.",
        },
        SearchEvalCase {
            name: "long title partial creature",
            query: "advanced eltha embercall",
            retrieval: "hybrid",
            extra_args: &["--family", "creature"],
            expected_keys: &["pfs-season-3-bestiary:3r3etd3bUOn58TH3"],
            note: "Long creature variant title; checks partial matching across repeated base name and level suffix text.",
        },
        SearchEvalCase {
            name: "long title partial equipment",
            query: "handkerchief disagreeable greater",
            retrieval: "hybrid",
            extra_args: &["--family", "equipment"],
            expected_keys: &["equipment-srd:KR35T5By2iaLKEwH"],
            note: "Long equipment title with omitted connector words and parenthetical variant.",
        },
        SearchEvalCase {
            name: "long title rank variant equipment",
            query: "smoldering fireballs 3rd rank",
            retrieval: "hybrid",
            extra_args: &["--family", "equipment"],
            expected_keys: &["equipment-srd:c2Oa9UbhjwAsZaPp"],
            note: "Equipment variant title; checks whether rank/variant tokens disambiguate otherwise similar names.",
        },
        SearchEvalCase {
            name: "long title spellcaster creature",
            query: "resurrection dragon spellcaster",
            retrieval: "hybrid",
            extra_args: &["--family", "creature"],
            expected_keys: &["pathfinder-monster-core-2:CZc6eVvRcsVJgjBU"],
            note: "Creature title with parenthetical descriptor; checks title coverage when age token is omitted.",
        },
        SearchEvalCase {
            name: "long alias religious symbol",
            query: "religious symbol sarenrae",
            retrieval: "hybrid",
            extra_args: &["--family", "equipment"],
            expected_keys: &["equipment-srd:plplsXJsqrdqNQVI"],
            note: "Long alias partial match; expected record is keyed by a generic item but has a source alias with deity-specific tokens.",
        },
        SearchEvalCase {
            name: "long alias map partial",
            query: "banyan map sea caves",
            retrieval: "hybrid",
            extra_args: &["--family", "equipment"],
            expected_keys: &["equipment-srd:UhcRWtnjU2WLSClx"],
            note: "Long alias partial match with omitted possessive/title token; tests alias lexical coverage.",
        },
        SearchEvalCase {
            name: "partial title abbreviation",
            query: "battle med",
            retrieval: "hybrid",
            extra_args: &[],
            expected_keys: &["feats-srd:wYerMk6F1RZb0Fwt"],
            note: "Abbreviated title query; expected to expose whether prefix-like matching exists beyond exact identity resolution.",
        },
        SearchEvalCase {
            name: "alias missing stopword",
            query: "attack opportunity",
            retrieval: "hybrid",
            extra_args: &[],
            expected_keys: &["actionspf2e:KAVf7AmRnbCAHrkT"],
            note: "Alias query missing a stopword from 'Attack of Opportunity'; expected to expose missing-token title/alias resolver gaps.",
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

fn search_quality_cases() -> &'static [SearchQualityCase] {
    &[
        SearchQualityCase {
            name: "direct title",
            query: "battle medicine",
            extra_args: &[],
            note: "Precision title lookup; hybrid should preserve the obvious direct hit while vector shows semantic behavior.",
        },
        SearchQualityCase {
            name: "partial title abbreviation",
            query: "battle med",
            extra_args: &[],
            note: "Partial/prefix title lookup; useful for judging deterministic lexical assistance in hybrid.",
        },
        SearchQualityCase {
            name: "alias missing stopword",
            query: "attack opportunity",
            extra_args: &[],
            note: "Alias lookup missing a stopword from Attack of Opportunity; tests title/alias candidate handling.",
        },
        SearchQualityCase {
            name: "filtered direct spell",
            query: "fear",
            extra_args: &["--family", "spell", "--max-level", "3"],
            note: "Short direct spell lookup under family and numeric filters.",
        },
        SearchQualityCase {
            name: "semantic concept spell",
            query: "low level spell that makes enemies afraid",
            extra_args: &["--family", "spell", "--max-level", "3"],
            note: "Semantic concept query; vector should carry most of the product value.",
        },
        SearchQualityCase {
            name: "trait structured search",
            query: "healing spell",
            extra_args: &[
                "--family",
                "spell",
                "--max-level",
                "3",
                "--trait",
                "healing",
            ],
            note: "Trait-filtered spell query; compares structured filtering plus retrieval quality.",
        },
        SearchQualityCase {
            name: "metric structured search",
            query: "high armor creature",
            extra_args: &["--family", "creature", "--metric", "ac.value>=25"],
            note: "Metric-filtered creature query; useful for structural parity and ranking plausibility.",
        },
        SearchQualityCase {
            name: "publication structured search",
            query: "fear",
            extra_args: &[
                "--family",
                "spell",
                "--publication-title",
                "Pathfinder Player Core",
            ],
            note: "Publication-filtered direct spell query.",
        },
        SearchQualityCase {
            name: "reference structured search",
            query: "frightened",
            extra_args: &[
                "--family",
                "spell",
                "--references",
                "conditionitems:TBSHQspnbcqxsmjL",
            ],
            note: "Reference-filtered spell query; checks graph-derived filtering in product-shaped search.",
        },
        SearchQualityCase {
            name: "facet disambiguation",
            query: "undead dragon",
            extra_args: &["--family", "creature"],
            note: "Facet-heavy query where lexical traits can usefully disambiguate semantic dragon results.",
        },
        SearchQualityCase {
            name: "broad mechanical concept",
            query: "persistent damage",
            extra_args: &["--family", "spell"],
            note: "Mechanical concept query; weak FTS should not swamp better semantic spell matches.",
        },
        SearchQualityCase {
            name: "long title partial hazard",
            query: "house spirits regenerating pies",
            extra_args: &["--family", "hazard"],
            note: "Long-title partial token match with omitted filler words.",
        },
        SearchQualityCase {
            name: "long title distinctive hazard",
            query: "absurd regenerating pies",
            extra_args: &["--family", "hazard"],
            note: "Long-title rare-token match.",
        },
        SearchQualityCase {
            name: "long title partial affliction",
            query: "hunting spider venom first magazine",
            extra_args: &["--family", "affliction"],
            note: "Generated affliction title with parenthetical detail and omitted filler words.",
        },
        SearchQualityCase {
            name: "long title partial creature",
            query: "advanced eltha embercall",
            extra_args: &["--family", "creature"],
            note: "Creature variant title with partial matching across repeated base and suffix text.",
        },
        SearchQualityCase {
            name: "long title partial equipment",
            query: "handkerchief disagreeable greater",
            extra_args: &["--family", "equipment"],
            note: "Equipment title with omitted connector words and variant text.",
        },
        SearchQualityCase {
            name: "long title rank variant equipment",
            query: "smoldering fireballs 3rd rank",
            extra_args: &["--family", "equipment"],
            note: "Rank/variant title disambiguation.",
        },
        SearchQualityCase {
            name: "long title spellcaster creature",
            query: "resurrection dragon spellcaster",
            extra_args: &["--family", "creature"],
            note: "Creature title with parenthetical descriptor and omitted age token.",
        },
        SearchQualityCase {
            name: "long alias religious symbol",
            query: "religious symbol sarenrae",
            extra_args: &["--family", "equipment"],
            note: "Alias lexical coverage for deity-specific equipment naming.",
        },
        SearchQualityCase {
            name: "long alias map partial",
            query: "banyan map sea caves",
            extra_args: &["--family", "equipment"],
            note: "Alias lexical coverage with omitted possessive/title token.",
        },
        SearchQualityCase {
            name: "long action semantic",
            query: "use a skill to earn money during downtime",
            extra_args: &[],
            note: "Long action text semantic query; checks embedding quality after chunking changes.",
        },
        SearchQualityCase {
            name: "long class feature semantic",
            query: "bard spell repertoire occult cantrips",
            extra_args: &[],
            note: "Long class feature semantic query; useful for truncation/chunking quality.",
        },
        SearchQualityCase {
            name: "long inventor feature semantic",
            query: "revolutionary armor modification armor innovation",
            extra_args: &[],
            note: "Mechanical long class feature query.",
        },
        SearchQualityCase {
            name: "long feat semantic",
            query: "soulforged armament essence powers",
            extra_args: &["--family", "feat"],
            note: "Long feat semantic query over parent and child-section embeddings.",
        },
        SearchQualityCase {
            name: "long spell semantic",
            query: "transform into an avatar of your deity huge battle form",
            extra_args: &["--family", "spell"],
            note: "Long spell mechanics query.",
        },
        SearchQualityCase {
            name: "long creature semantic",
            query: "frost drake frozen reaches hunt caribou wolves",
            extra_args: &["--family", "creature"],
            note: "Creature description/lore semantic query.",
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
    run_search_case(SearchRunRequest {
        query: case.query,
        retrieval: case.retrieval,
        extra_args: case.extra_args,
        backend,
        sqlite_path,
        ladybug_path,
        atlas_bin,
        expected_keys: case.expected_keys,
    })
}

fn run_search_case(request: SearchRunRequest<'_>) -> SearchEvalRun {
    let mut args = vec![
        "search".to_string(),
        request.query.to_string(),
        "--index".to_string(),
        request.sqlite_path.display().to_string(),
        "--index-backend".to_string(),
        request.backend.to_string(),
        "--retrieval".to_string(),
        request.retrieval.to_string(),
        "--limit".to_string(),
        "10".to_string(),
        "--detail".to_string(),
        "summary".to_string(),
        "--json".to_string(),
        "--progress".to_string(),
        "never".to_string(),
        "--explain".to_string(),
    ];
    if request.backend == "ladybug" {
        args.push("--ladybug-index".to_string());
        args.push(request.ladybug_path.display().to_string());
    }
    args.extend(request.extra_args.iter().map(|arg| (*arg).to_string()));

    let run = run_atlas_json(request.atlas_bin, &args);
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
    parse_search_eval_json(request.expected_keys, run.elapsed_ms, &json)
}

fn parse_search_eval_json(
    expected_keys: &[&str],
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
    let expected_positions = expected_keys
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

fn print_search_quality_case(
    case: &SearchQualityCase,
    sqlite_hybrid: &SearchEvalRun,
    ladybug_hybrid: &SearchEvalRun,
    sqlite_vector: &SearchEvalRun,
    ladybug_vector: &SearchEvalRun,
) {
    println!();
    println!("## {}", case.name);
    println!("query: {:?}", case.query);
    if !case.extra_args.is_empty() {
        println!("filters/options: {}", case.extra_args.join(" "));
    }
    println!("note: {}", case.note);
    print_quality_run("sqlite hybrid", sqlite_hybrid);
    print_quality_run("ladybug hybrid", ladybug_hybrid);
    print_quality_run("sqlite vector", sqlite_vector);
    print_quality_run("ladybug vector", ladybug_vector);
    if sqlite_hybrid.error.is_none()
        && ladybug_hybrid.error.is_none()
        && sqlite_vector.error.is_none()
        && ladybug_vector.error.is_none()
    {
        println!(
            "overlap: hybrid sqlite/ladybug={} vector sqlite/ladybug={} sqlite hybrid/vector={} ladybug hybrid/vector={}",
            top_overlap(&sqlite_hybrid.keys, &ladybug_hybrid.keys),
            top_overlap(&sqlite_vector.keys, &ladybug_vector.keys),
            top_overlap(&sqlite_hybrid.keys, &sqlite_vector.keys),
            top_overlap(&ladybug_hybrid.keys, &ladybug_vector.keys)
        );
    }
}

fn print_quality_run(label: &str, run: &SearchEvalRun) {
    println!(
        "{label}: {}ms total={} top={}",
        run.elapsed_ms,
        optional_u64(run.total),
        summarize_search_run(run)
    );
    if let Some(error) = &run.error {
        println!("{label} error: {error}");
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
