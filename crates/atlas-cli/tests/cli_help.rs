use std::process::Command;
fn help(args: &[&str]) -> String {
    let o = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(args)
        .arg("--help")
        .output()
        .unwrap();
    assert!(o.status.success());
    String::from_utf8(o.stdout).unwrap()
}
#[test]
fn command_surface_uses_catalog_cel_and_retires_normalized_flags() {
    let root = help(&[]);
    for name in [
        "setup",
        "index",
        "record",
        "search",
        "similar",
        "graph",
        "filters",
        "lists",
        "tags",
        "agent",
        "completions",
        "build-info",
    ] {
        assert!(root.contains(name));
    }
    let search = help(&["search"]);
    for flag in [
        "--where",
        "--kind",
        "--trait",
        "--pack-name",
        "--rarity",
        "--retrieval",
        "--search-content",
        "--sort",
        "--seed",
        "--links-to",
        "--linked-from",
        "--links-to-main-entry",
        "--linked-from-main-entry",
    ] {
        assert!(search.contains(flag));
    }
    for old in [
        "--metric",
        "--references",
        "--referenced-by",
        "--filter-json",
        "--include-raw",
        "--fusion",
        "--min-price",
    ] {
        assert!(!search.contains(old), "retired flag is advertised: {old}");
    }
    assert!(help(&["index", "build"]).contains("--locale"));
    assert!(help(&["setup"]).contains("--locale"));
    assert!(help(&["record", "get"]).contains("--passage"));
    assert!(help(&["record", "get"]).contains("record source"));
    assert!(help(&["record", "source"]).contains("embedded documents"));
    assert!(help(&["filters", "fields"]).contains("operators"));
    assert!(!help(&["similar"]).contains("--weight"));
}

#[test]
fn profile_flags_support_both_builtins_and_reject_retired_or_conflicting_options() {
    for command in [
        &["index", "build"][..],
        &["setup"][..],
        &["search"][..],
        &["similar"][..],
        #[cfg(feature = "web")]
        &["web"][..],
    ] {
        let text = help(command);
        assert!(text.contains("--search-profile"));
        assert!(!text.contains("--embedding-model"));
    }
    for args in [
        vec!["setup", "--no-embeddings", "--search-profile", "bge-base"],
        vec![
            "index",
            "build",
            "--no-embeddings",
            "--search-profile",
            "embeddinggemma",
        ],
        vec!["search", "--search-profile", "bge-small"],
        vec!["search", "--embedding-model", "bge-base-en-v1.5"],
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
            .args(args)
            .output()
            .unwrap();
        assert!(!output.status.success());
        assert!(!String::from_utf8_lossy(&output.stderr).contains("panicked"));
    }
    // Check mode avoids setup writes or source fetching while parsing repeated flags.
    let unused = std::env::temp_dir().join(format!("atlas-profile-args-{}", std::process::id()));
    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args([
            "setup",
            "--check",
            "--offline",
            "--search-profile",
            "bge-base",
            "--search-profile",
            "embeddinggemma",
            "--json",
        ])
        .arg("--source")
        .arg(unused.join("source"))
        .arg("--index")
        .arg(unused.join("artifact.sqlite"))
        .arg("--embedding-cache-path")
        .arg(unused.join("models"))
        .output()
        .unwrap();
    let response: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        response["data"]["search_profiles"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
}

#[test]
fn explicit_human_detail_conflicts_with_json_before_reading_an_artifact() {
    for detail in ["summary", "standard", "full"] {
        for options in [
            vec!["--detail", detail, "--json"],
            vec!["--json", "--detail", detail],
        ] {
            let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
                .args(["record", "get", "actions:fixture"])
                .args(options)
                .output()
                .unwrap();
            assert_eq!(output.status.code(), Some(2));
            let error: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
            assert_eq!(error["status"], "error");
            let message = error["error"]["message"].as_str().unwrap();
            assert!(message.contains("--detail"));
            assert!(message.contains("--json"));
            assert!(message.contains("cannot be used"));
        }
    }
}
#[test]
fn retired_flags_fail_instead_of_becoming_a_broader_query() {
    for flag in [
        "--metric",
        "--references",
        "--referenced-by",
        "--filter-json",
        "--include-raw",
        "--fusion",
        "--min-price",
    ] {
        let o = Command::new(env!("CARGO_BIN_EXE_atlas"))
            .args(["search", flag, "x", "--json"])
            .output()
            .unwrap();
        assert!(!o.status.success());
        let json: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
        assert_eq!(json["status"], "error");
    }
}

#[test]
fn relationship_filters_and_occurrence_paging_are_discoverable_in_help() {
    let search = help(&["search"]);
    assert!(search.contains("links_to("));
    assert!(search.contains("linked_from("));
    for command in [
        &["similar"][..],
        &["filters", "values"][..],
        &["record", "resolve"][..],
    ] {
        let text = help(command);
        assert!(text.contains("--links-to"), "{command:?}");
        assert!(text.contains("--linked-from-main-entry"), "{command:?}");
    }
    let occurrences = help(&["graph", "occurrences"]);
    for flag in ["--artifact-generation", "--clause-id", "--page", "--limit"] {
        assert!(occurrences.contains(flag));
    }
    let discovery = help(&["filters", "values"]);
    for flag in ["--text", "--list", "--clause-id", "--artifact-generation"] {
        assert!(discovery.contains(flag));
    }
}

#[test]
fn similar_help_exposes_parts_methods_and_trait_seeds_without_new_ranking_flags() {
    let text = help(&["similar"]);
    for flag in [
        "--method",
        "--decay",
        "--part",
        "--candidate-parts",
        "--sections",
        "--search-content",
        "--list-parts",
        "--artifact-generation",
        "--explain",
    ] {
        assert!(text.contains(flag), "missing Similar control: {flag}");
    }
    assert!(text.contains("trait:fire"));
    assert!(text.contains("content-average"));
    assert!(!text.contains("--ranking"));
}

#[test]
fn invalid_similarity_controls_fail_before_workspace_access_with_clean_json() {
    for (flags, expected_code) in [
        (
            vec!["--method", "baseline", "--decay", "0.5"],
            "invalid_request",
        ),
        (
            vec!["--method", "content-average", "--decay", "0.5"],
            "invalid_request",
        ),
        (
            vec!["--method", "baseline", "--candidate-parts", "any"],
            "invalid_request",
        ),
        (vec!["--part", "opaque-selector"], "invalid_input"),
        (
            vec!["--method", "baseline", "--sections", "core"],
            "invalid_request",
        ),
        (
            vec!["--sections", "core", "--part", "opaque-selector"],
            "invalid_input",
        ),
        (
            vec!["--list-parts", "--search-content", "core"],
            "invalid_input",
        ),
        (vec!["--list-parts", "--kind", "hazard"], "invalid_request"),
        (vec!["--list-parts", "--page", "2"], "invalid_input"),
        (vec!["--decay", "NaN"], "invalid_input"),
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
            .args(["similar", "No workspace should be opened"])
            .args(flags)
            .arg("--json")
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(2));
        let envelope: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(envelope["error"]["code"], expected_code);
        assert!(!String::from_utf8_lossy(&output.stderr).contains("panicked"));
    }
}

#[test]
fn filter_discovery_rejects_record_query_flags_before_opening_a_workspace() {
    for (flag, value) in [
        ("--query", "fire"),
        ("--retrieval", "hybrid"),
        ("--search-profile", "bge-base"),
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
            .args([
                "filters", "values", "--field", "rarity", flag, value, "--json",
            ])
            .output()
            .unwrap();
        assert!(!output.status.success(), "{flag}");
        let body: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(body["status"], "error");
        assert!(body.to_string().contains("unexpected argument"), "{body}");
        assert!(body.to_string().contains(flag), "{body}");
    }
}
