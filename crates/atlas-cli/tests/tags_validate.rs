use std::path::PathBuf;

mod support;

use support::command::atlas_command;
use support::json::parse_ok_data;

#[test]
fn tags_validate_json_reports_valid_corpus_counts() -> Result<(), Box<dyn std::error::Error>> {
    let output = atlas_command()
        .args(["tags", "validate", "--path"])
        .arg(corpus_fixture("corpus"))
        .arg("--json")
        .output()?;

    assert!(output.status.success());
    let actual = parse_ok_data(&output)?;
    assert_eq!(actual["valid"], true);
    assert_eq!(actual["catalog_tags"], 3);
    assert_eq!(actual["assignment_records"], 3);
    assert_eq!(actual["assigned_tags"], 2);
    assert_eq!(actual["reviewed_empty_records"], 1);
    assert_eq!(actual["ontology_suggestions"], 1);
    Ok(())
}

#[test]
fn tags_validate_human_output_reports_valid_corpus_counts() -> Result<(), Box<dyn std::error::Error>>
{
    let output = atlas_command()
        .args(["tags", "validate", "--path"])
        .arg(corpus_fixture("corpus"))
        .output()?;

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout)?;
    assert!(stdout.contains("Tag corpus valid"));
    assert!(stdout.contains("Catalog tags: 3"));
    assert!(stdout.contains("Assignment records: 3"));
    assert!(stdout.contains("Assigned tags: 2"));
    assert!(stdout.contains("Reviewed empty records: 1"));
    assert!(stdout.contains("Ontology suggestions: 1"));
    Ok(())
}

#[test]
fn tags_validate_json_reports_invalid_corpus() -> Result<(), Box<dyn std::error::Error>> {
    let output = atlas_command()
        .args(["tags", "validate", "--path"])
        .arg(corpus_fixture("corpus-unknown-assignment"))
        .arg("--json")
        .output()?;

    assert_eq!(output.status.code(), Some(3));
    let actual = parse_ok_data(&output)?;
    assert_eq!(actual["valid"], false);
    assert!(
        actual["errors"][0]
            .as_str()
            .expect("message is string")
            .contains("unknown tag")
    );
    Ok(())
}

fn corpus_fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("atlas-tags")
        .join("tests")
        .join("fixtures")
        .join(name)
}
