use super::{
    args::{Phase, SimilarityArgs, SimilarityCommand},
    execution::recall,
    performance::percentile,
};
use atlas_domain::RecordKey;
use clap::Parser;
use std::collections::BTreeSet;
#[derive(Parser)]
struct Command {
    #[command(flatten)]
    args: SimilarityArgs,
}
#[test]
fn requires_explicit_scenarios_and_preserves_subset_flags() {
    let base = [
        "atlas-dev",
        "evaluate",
        "--manifest",
        "manifest.json",
        "--corpus",
        "corpus.json",
        "--output-dir",
        "out",
        "--build-revision",
        "0000000000000000000000000000000000000000",
    ];
    assert!(Command::try_parse_from(base).is_err());
    let mut input = base.to_vec();
    input.extend([
        "--scenario",
        "dev-h01",
        "--profile",
        "bge-base",
        "--variant",
        "baseline",
        "--phase",
        "oracle",
        "--json",
    ]);
    let parsed = Command::try_parse_from(input).unwrap();
    let SimilarityCommand::Evaluate(options) = parsed.args.command;
    assert_eq!(options.scenarios, ["dev-h01"]);
    assert_eq!(options.profiles, ["bge-base"]);
    assert_eq!(options.phase, Phase::Oracle);
    assert!(options.json);
}
#[test]
fn separates_admission_recall_from_returned_overlap_and_unknown_denominators() {
    let a = RecordKey::parse("test:a").unwrap();
    let b = RecordKey::parse("test:b").unwrap();
    let admitted = BTreeSet::from([a.clone(), b.clone()]);
    let value = recall(
        &[a.clone(), b.clone()],
        Some(&admitted),
        std::slice::from_ref(&a),
        true,
    );
    assert_eq!(value["candidate_admission_recall_at10"], 1.0);
    assert_eq!(value["returned_top10_agreement"], 0.5);
    assert!(recall(&[a], None, &[b], true)["candidate_admission_recall_at10"].is_null());
    assert!(recall(&[], Some(&admitted), &[], true)["returned_top10_agreement"].is_null());
    assert!(recall(&admitted.into_iter().collect::<Vec<_>>(),None,&[],false)["returned_top10_agreement"].is_null());
}
#[test]
fn reports_nearest_rank_percentiles_and_no_empty_latency() {
    let samples = (1..=20).map(f64::from).collect::<Vec<_>>();
    assert_eq!(percentile(&samples, 0.95), Some(19.0));
    assert_eq!(percentile(&samples, 0.5), Some(10.0));
    assert_eq!(percentile(&[], 0.95), None);
}

#[test]
fn rejects_changed_frozen_input_before_opening_a_workspace() {
    let source = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/research/evaluations/2026-10-similarity");
    let temporary = tempfile::tempdir().unwrap();
    let freeze: serde_json::Value =
        serde_json::from_slice(&std::fs::read(source.join("freeze.json")).unwrap()).unwrap();
    for name in freeze["files"]
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .chain(std::iter::once("freeze.json"))
    {
        std::fs::copy(source.join(name), temporary.path().join(name)).unwrap();
    }
    let command = Command::try_parse_from([
        "atlas-dev",
        "evaluate",
        "--manifest",
        temporary.path().join("manifest.json").to_str().unwrap(),
        "--corpus",
        temporary.path().join("corpus.json").to_str().unwrap(),
        "--output-dir",
        temporary.path().join("out").to_str().unwrap(),
        "--build-revision",
        "0000000000000000000000000000000000000000",
        "--scenario",
        "dev-h01",
    ])
    .unwrap();
    let SimilarityCommand::Evaluate(options) = command.args.command;
    assert!(super::model::load(&options).is_ok());
    let mut manifest = std::fs::OpenOptions::new()
        .append(true)
        .open(&options.manifest)
        .unwrap();
    use std::io::Write;
    manifest.write_all(b"\n").unwrap();
    assert!(
        super::model::load(&options)
            .err()
            .unwrap()
            .contains("frozen input changed")
    );
}
