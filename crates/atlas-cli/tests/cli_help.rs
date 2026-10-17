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
        "web",
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
        "--sort",
        "--min-price",
    ] {
        assert!(!search.contains(old));
    }
    assert!(help(&["index", "build"]).contains("--locale"));
    assert!(help(&["setup"]).contains("--locale"));
    assert!(help(&["record", "get"]).contains("--passage"));
    assert!(help(&["record", "get"]).contains("record source"));
    assert!(help(&["record", "source"]).contains("embedded documents"));
    assert!(help(&["filters", "fields"]).contains("operators"));
    assert!(!help(&["similar"]).contains("weight"));
}

#[test]
fn profile_flags_support_both_builtins_and_reject_retired_or_conflicting_options() {
    for command in [
        &["index", "build"][..],
        &["setup"][..],
        &["search"][..],
        &["similar"][..],
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
    for flag in ["--metric", "--filter-json", "--references", "--include-raw"] {
        let o = Command::new(env!("CARGO_BIN_EXE_atlas"))
            .args(["search", flag, "x", "--json"])
            .output()
            .unwrap();
        assert!(!o.status.success());
        let json: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
        assert_eq!(json["status"], "error");
    }
}
