use std::process::Command;

mod support;

use support::command::help_output;

#[test]
fn help_text_includes_setup_validate_and_record_examples() -> Result<(), Box<dyn std::error::Error>>
{
    let root_help = help_output(&[])?;
    assert!(root_help.contains("atlas setup"));
    assert!(root_help.contains("atlas record get actionspf2e:1kGNdIIhuglAjIp9"));

    let setup_help = help_output(&["setup"])?;
    assert!(setup_help.contains("atlas setup --no-embeddings"));
    assert!(setup_help.contains("--offline"));

    let validate_help = help_output(&["index", "validate"])?;
    assert!(validate_help.contains("atlas index validate --embeddings-only"));
    assert!(validate_help.contains("--no-embeddings"));

    let check_help = help_output(&["index", "check"])?;
    assert!(check_help.contains("fast artifact readiness check"));
    assert!(check_help.contains("--no-embeddings"));

    let build_help = help_output(&["index", "build"])?;
    assert!(build_help.contains("atlas index build --no-embeddings"));
    assert!(build_help.contains(
        "atlas index build --source vendor/pf2e --output .cache/pf2e-index.sqlite --json"
    ));
    assert!(build_help.contains("Standard users should run `atlas setup` instead."));

    let analyze_help = help_output(&["index", "analyze"])?;
    assert!(analyze_help.contains(
        "atlas index analyze --source vendor/pf2e --manifest scratch/ingest-manifest.json --json"
    ));

    let inspect_help = help_output(&["index", "inspect"])?;
    assert!(inspect_help.contains("atlas index inspect --json"));

    let record_get_help = help_output(&["record", "get"])?;
    assert!(record_get_help.contains("equipment-srd:s1vB3HdXjMigYAnY"));
    assert!(record_get_help.contains("Canonical record keys"));

    let record_resolve_help = help_output(&["record", "resolve"])?;
    assert!(record_resolve_help.contains("atlas record resolve \"Treat Wounds\""));
    assert!(record_resolve_help.contains("--filter-json"));
    assert!(record_resolve_help.contains("atlas filters fields"));
    assert!(record_resolve_help.contains("atlas filters values --field traits"));

    let graph_links_help = help_output(&["graph", "links"])?;
    assert!(graph_links_help.contains("atlas graph links"));
    assert!(graph_links_help.contains("--backlinks"));

    let similar_help = help_output(&["similar"])?;
    assert!(similar_help.contains("atlas similar \"Dirge of Doom\""));
    assert!(similar_help.contains("--semantic-weight"));
    assert!(similar_help.contains("--reference-weight"));
    assert!(similar_help.contains("--trait-weight"));

    let search_help = help_output(&["search"])?;
    assert!(search_help.contains("atlas search \"low level healing spell\""));
    assert!(search_help.contains("atlas search --kind creature --metric 'ac.value>=25'"));
    assert!(
        search_help
            .contains("atlas search --kind creature --metric 'hp.value:40' --print-filter --json")
    );
    assert!(search_help.contains("atlas filters fields"));
    assert!(search_help.contains("atlas filters values --field traits"));
    assert!(search_help.contains("atlas filters values --field metric"));
    assert!(search_help.contains("--retrieval selects fts, vector, or hybrid retrieval"));
    assert!(search_help.contains("--pack-name"));
    assert!(search_help.contains("--publication-title"));
    assert!(search_help.contains("--price"));
    assert!(search_help.contains("--min-price"));
    assert!(search_help.contains("--max-price"));
    assert!(search_help.contains("--references"));
    assert!(search_help.contains("--referenced-by"));
    assert!(search_help.contains("--metric"));
    assert!(search_help.contains("ac.value>=18"));
    assert!(search_help.contains("price_asc"));
    assert!(search_help.contains("price_desc"));
    assert!(search_help.contains("--print-filter"));

    let filters_help = help_output(&["filters"])?;
    assert!(filters_help.contains("fields"));
    assert!(filters_help.contains("values"));

    let filter_fields_help = help_output(&["filters", "fields"])?;
    assert!(filter_fields_help.contains("atlas filters fields --kind spell"));
    assert!(filter_fields_help.contains("atlas filters fields --kind creature --json"));
    assert!(filter_fields_help.contains("--json"));

    let filter_values_help = help_output(&["filters", "values"])?;
    assert!(filter_values_help.contains("--field"));
    assert!(filter_values_help.contains("--metric-query"));
    assert!(filter_values_help.contains("--metric-label"));
    assert!(filter_values_help.contains("--sample-limit"));
    assert!(filter_values_help.contains("--limit"));
    assert!(filter_values_help.contains("--json"));
    assert!(
        filter_values_help
            .contains("atlas filters values --field metric --kind creature --metric-query armor")
    );

    let agent_skills_help = help_output(&["agent", "skills"])?;
    assert!(
        agent_skills_help
            .contains("atlas agent skills install --target codex --scope global --yes")
    );
    assert!(agent_skills_help.contains("atlas agent skills doctor --json"));

    let agent_install_help = help_output(&["agent", "skills", "install"])?;
    assert!(agent_install_help.contains(
        "atlas agent skills install --target agents --scope workspace --force --yes --json"
    ));
    assert!(agent_install_help.contains("--skill"));
    assert!(agent_install_help.contains("--target"));

    let agent_doctor_help = help_output(&["agent", "skills", "doctor"])?;
    assert!(agent_doctor_help.contains("atlas agent skills doctor --target codex --scope global"));
    assert!(agent_doctor_help.contains("--scope"));

    Ok(())
}

#[test]
fn legacy_top_level_index_commands_are_not_supported() -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .arg("validate-index")
        .output()?;

    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8(output.stderr)?;
    assert!(stderr.contains("unrecognized subcommand 'validate-index'"));
    Ok(())
}

#[test]
fn validate_vectors_subcommand_is_removed() -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["index", "validate-vectors"])
        .output()?;

    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8(output.stderr)?;
    assert!(stderr.contains("unrecognized subcommand 'validate-vectors'"));
    Ok(())
}
