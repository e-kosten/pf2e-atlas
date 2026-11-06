//! Real command behavior over checked reference fixtures; vectors are structural, not model output.
use atlas_search::test_support::{add_remaster_relationship, input, open_input_fixture, record};
use serde_json::{Value, json};
use std::process::Command;

#[test]
fn all_versions_controls_real_browse_text_and_similar_requests_and_results() {
    let seed = "items:seed000000000001";
    let old = "items:legacy0000000001";
    let modern = "items:modern0000000001";
    let mut data = input(
        vec![
            record(
                "items",
                "Item",
                json!({"_id":"seed000000000001","name":"Signal seed","type":"action","system":{"description":{"value":"<p>Signal information.</p>"}}}),
            ),
            record(
                "items",
                "Item",
                json!({"_id":"legacy0000000001","name":"Signal old","type":"action","system":{"publication":{"remaster":false},"description":{"value":"<p>Signal information.</p>"}}}),
            ),
            record(
                "items",
                "Item",
                json!({"_id":"modern0000000001","name":"Signal modern","type":"action","system":{"publication":{"remaster":true},"description":{"value":"<p>Signal information.</p>"}}}),
            ),
        ],
        true,
    );
    add_remaster_relationship(&mut data, &old.parse().unwrap(), &modern.parse().unwrap()).unwrap();
    for unit in &mut data.semantic_units {
        unit.vector.fill(0.0);
        unit.vector[0] = 1.0;
    }
    let (retrieval, artifact) = open_input_fixture(data).unwrap();
    drop(retrieval);
    for arguments in [
        vec!["search"],
        vec!["search", "Signal", "--retrieval", "fts"],
        vec!["similar", seed, "--method", "baseline"],
    ] {
        for all_versions in [false, true] {
            let mut command = Command::new(env!("CARGO_BIN_EXE_atlas"));
            command
                .args(&arguments)
                .arg("--index")
                .arg(artifact.path())
                .args(["--limit", "100", "--json"]);
            if all_versions {
                command.arg("--all-versions");
            }
            for name in [
                "GIT_DIR",
                "GIT_WORK_TREE",
                "GIT_INDEX_FILE",
                "GIT_PREFIX",
                "ATLAS_LOG",
            ] {
                command.env_remove(name);
            }
            let output = command.output().unwrap();
            assert!(
                output.status.success(),
                "{arguments:?} all={all_versions}: {} {}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            let envelope: Value = serde_json::from_slice(&output.stdout).unwrap();
            let result = &envelope["data"];
            assert_eq!(
                result["prefer_remaster"], !all_versions,
                "{arguments:?}: {result}"
            );
            let keys = result["rows"]
                .as_array()
                .unwrap()
                .iter()
                .filter_map(|row| row["record"]["record_key"].as_str())
                .collect::<Vec<_>>();
            assert!(keys.contains(&modern), "{arguments:?}: {result}");
            assert_eq!(keys.contains(&old), all_versions, "{arguments:?}: {result}");
        }
    }
}
