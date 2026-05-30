use std::fs;

use rusqlite::Connection;

use super::{
    create_contract_database, record_key_strings, replace_fts_rows, replace_single_column_rows,
    temp_db_path, weights_for_column,
};
use crate::{FtsColumnWeights, FtsQuery, SqliteIndexReader};

#[test]
fn weighted_ranking_prefers_title_matches_over_body_matches()
-> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("fts-weight-order");
    create_contract_database(&path)?;
    let connection = Connection::open(&path)?;
    replace_fts_rows(
        &connection,
        &[
            ("actions:testAction1", "needle", "", "", "", "", "", "", ""),
            (
                "actions:testAction2",
                "other",
                "",
                "",
                "",
                "",
                "",
                "",
                "needle needle needle",
            ),
            (
                "actions:testAction3",
                "other",
                "",
                "",
                "",
                "",
                "",
                "",
                "needle",
            ),
        ],
    )?;
    drop(connection);

    let query = FtsQuery::from_tokens(vec!["needle".to_string()]).expect("query");
    let hits = SqliteIndexReader::open_read_only(&path)?.query_weighted_fts_index(
        &query,
        None,
        10,
        FtsColumnWeights::default(),
    )?;

    assert_eq!(hits[0].record_key.to_string(), "actions:testAction1");
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn weighted_ranking_covers_structured_term_columns() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("fts-structured-weight-order");
    create_contract_database(&path)?;
    let connection = Connection::open(&path)?;
    replace_fts_rows(
        &connection,
        &[
            (
                "actions:testAction1",
                "one",
                "",
                "needle",
                "",
                "",
                "",
                "",
                "",
            ),
            (
                "actions:testAction2",
                "two",
                "",
                "",
                "",
                "",
                "needle",
                "",
                "",
            ),
            (
                "actions:testAction3",
                "three",
                "",
                "",
                "",
                "",
                "",
                "needle",
                "",
            ),
        ],
    )?;
    drop(connection);

    let query = FtsQuery::from_tokens(vec!["needle".to_string()]).expect("query");
    let index = SqliteIndexReader::open_read_only(&path)?;
    let taxonomy_weighted = index.query_weighted_fts_index(
        &query,
        None,
        10,
        FtsColumnWeights {
            taxonomy_terms: 8.0,
            source_terms: 1.0,
            metric_terms: 1.0,
            ..FtsColumnWeights::default()
        },
    )?;
    assert_eq!(
        record_key_strings(&taxonomy_weighted),
        vec![
            "actions:testAction1",
            "actions:testAction2",
            "actions:testAction3"
        ]
    );

    let source_weighted = index.query_weighted_fts_index(
        &query,
        None,
        10,
        FtsColumnWeights {
            taxonomy_terms: 1.0,
            source_terms: 8.0,
            metric_terms: 1.0,
            ..FtsColumnWeights::default()
        },
    )?;
    assert_eq!(
        record_key_strings(&source_weighted),
        vec![
            "actions:testAction2",
            "actions:testAction1",
            "actions:testAction3"
        ]
    );

    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn weighted_ranking_covers_all_fts_columns() -> Result<(), Box<dyn std::error::Error>> {
    let cases = [
        ("title", "actions:testAction1"),
        ("aliases", "actions:testAction1"),
        ("traits", "actions:testAction1"),
        ("taxonomy_terms", "actions:testAction1"),
        ("constraint_terms", "actions:testAction1"),
        ("mechanic_terms", "actions:testAction1"),
        ("source_terms", "actions:testAction1"),
        ("metric_terms", "actions:testAction1"),
        ("headings", "actions:testAction1"),
        ("body", "actions:testAction1"),
        ("facts", "actions:testAction1"),
        ("reference_terms", "actions:testAction1"),
        ("embedded_content", "actions:testAction1"),
    ];

    for (column, expected_key) in cases {
        let path = temp_db_path(&format!("fts-column-{column}"));
        create_contract_database(&path)?;
        let connection = Connection::open(&path)?;
        replace_single_column_rows(&connection, column)?;
        drop(connection);

        let query = FtsQuery::from_tokens(vec!["needle".to_string()]).expect("query");
        let hits = SqliteIndexReader::open_read_only(&path)?.query_weighted_fts_index(
            &query,
            None,
            1,
            weights_for_column(column),
        )?;

        assert_eq!(record_key_strings(&hits), vec![expected_key], "{column}");
        fs::remove_file(path)?;
    }
    Ok(())
}

#[test]
fn strict_conjunction_tier_ranks_before_or_fallback() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("fts-strict-before-fallback");
    create_contract_database(&path)?;
    let connection = Connection::open(&path)?;
    replace_fts_rows(
        &connection,
        &[
            (
                "actions:testAction1",
                "Generic Body Match",
                "",
                "",
                "",
                "",
                "",
                "",
                "shield block",
            ),
            ("actions:testAction2", "Shield", "", "", "", "", "", "", ""),
            ("actions:testAction3", "Block", "", "", "", "", "", "", ""),
        ],
    )?;
    drop(connection);

    let query =
        FtsQuery::from_tokens(vec!["shield".to_string(), "block".to_string()]).expect("query");
    let hits = SqliteIndexReader::open_read_only(&path)?.query_weighted_fts_index(
        &query,
        None,
        3,
        FtsColumnWeights::default(),
    )?;

    assert_eq!(
        record_key_strings(&hits),
        vec![
            "actions:testAction1",
            "actions:testAction2",
            "actions:testAction3"
        ]
    );
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn fallback_rows_are_deduped_and_truncated_after_strict_rows()
-> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("fts-fallback-dedupe-limit");
    create_contract_database(&path)?;
    let connection = Connection::open(&path)?;
    replace_fts_rows(
        &connection,
        &[
            (
                "actions:testAction1",
                "Body Match",
                "",
                "",
                "",
                "",
                "",
                "",
                "alpha beta",
            ),
            ("actions:testAction2", "Alpha", "", "", "", "", "", "", ""),
            ("actions:testAction3", "Beta", "", "", "", "", "", "", ""),
        ],
    )?;
    drop(connection);

    let query =
        FtsQuery::from_tokens(vec!["alpha".to_string(), "beta".to_string()]).expect("query");
    let hits = SqliteIndexReader::open_read_only(&path)?.query_weighted_fts_index(
        &query,
        None,
        2,
        FtsColumnWeights::default(),
    )?;

    assert_eq!(
        record_key_strings(&hits),
        vec!["actions:testAction1", "actions:testAction2"]
    );
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn title_phrase_signals_rank_above_body_matches() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("fts-title-phrase-rank");
    create_contract_database(&path)?;
    let connection = Connection::open(&path)?;
    replace_fts_rows(
        &connection,
        &[
            (
                "actions:testAction1",
                "Other",
                "",
                "",
                "",
                "",
                "",
                "",
                "concealed flat check concealed flat check concealed flat check",
            ),
            (
                "actions:testAction2",
                "Concealed Flat Check",
                "",
                "",
                "",
                "",
                "",
                "",
                "",
            ),
            (
                "actions:testAction3",
                "Other",
                "",
                "",
                "",
                "",
                "",
                "",
                "concealed flat check",
            ),
        ],
    )?;
    drop(connection);

    let query = FtsQuery::from_tokens(vec![
        "concealed".to_string(),
        "flat".to_string(),
        "check".to_string(),
    ])
    .expect("query");
    let hits = SqliteIndexReader::open_read_only(&path)?.query_weighted_fts_index(
        &query,
        None,
        1,
        FtsColumnWeights::default(),
    )?;

    assert_eq!(record_key_strings(&hits), vec!["actions:testAction2"]);
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn token_coverage_signals_rank_high_value_matches_above_body_only_matches()
-> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("fts-token-coverage-rank");
    create_contract_database(&path)?;
    let connection = Connection::open(&path)?;
    replace_fts_rows(
        &connection,
        &[
            (
                "actions:testAction1",
                "Other",
                "",
                "",
                "",
                "",
                "",
                "",
                "alpha filler beta alpha filler beta alpha filler beta",
            ),
            (
                "actions:testAction2",
                "Alpha",
                "",
                "",
                "",
                "",
                "",
                "beta",
                "filler",
            ),
            (
                "actions:testAction3",
                "Beta",
                "",
                "",
                "",
                "",
                "",
                "",
                "alpha",
            ),
        ],
    )?;
    drop(connection);

    let query =
        FtsQuery::from_tokens(vec!["alpha".to_string(), "beta".to_string()]).expect("query");
    let hits = SqliteIndexReader::open_read_only(&path)?.query_weighted_fts_index(
        &query,
        None,
        1,
        FtsColumnWeights::default(),
    )?;

    assert_eq!(record_key_strings(&hits), vec!["actions:testAction2"]);
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn partial_title_match_ranks_above_body_only_phrase_match() -> Result<(), Box<dyn std::error::Error>>
{
    let path = temp_db_path("fts-partial-title-rank");
    create_contract_database(&path)?;
    let connection = Connection::open(&path)?;
    replace_fts_rows(
        &connection,
        &[
            (
                "actions:testAction1",
                "Reactive Strike",
                "",
                "",
                "",
                "",
                "",
                "",
                "prevention",
            ),
            (
                "actions:testAction2",
                "Other",
                "",
                "",
                "",
                "",
                "",
                "",
                "reactive strike prevention reactive strike prevention",
            ),
            (
                "actions:testAction3",
                "Prevention",
                "",
                "",
                "",
                "",
                "",
                "",
                "reactive strike",
            ),
        ],
    )?;
    drop(connection);

    let query = FtsQuery::from_tokens(vec![
        "reactive".to_string(),
        "strike".to_string(),
        "prevention".to_string(),
    ])
    .expect("query");
    let hits = SqliteIndexReader::open_read_only(&path)?.query_weighted_fts_index(
        &query,
        None,
        1,
        FtsColumnWeights::default(),
    )?;

    assert_eq!(record_key_strings(&hits), vec!["actions:testAction1"]);
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn effect_records_are_downranked_below_canonical_records() -> Result<(), Box<dyn std::error::Error>>
{
    let path = temp_db_path("fts-effect-downrank");
    create_contract_database(&path)?;
    let connection = Connection::open(&path)?;
    connection.execute(
        "UPDATE records
         SET foundry_record_type = CASE record_key
             WHEN 'actions:testAction1' THEN 'effect'
             WHEN 'actions:testAction2' THEN 'action'
             ELSE foundry_record_type
         END",
        [],
    )?;
    replace_fts_rows(
        &connection,
        &[
            (
                "actions:testAction1",
                "Fly Speed",
                "",
                "",
                "",
                "",
                "fly speed",
                "",
                "fly speed",
            ),
            (
                "actions:testAction2",
                "Fly Speed",
                "",
                "",
                "",
                "",
                "fly speed",
                "",
                "fly speed",
            ),
            (
                "actions:testAction3",
                "Other Fly",
                "",
                "",
                "",
                "",
                "",
                "",
                "fly speed",
            ),
        ],
    )?;
    drop(connection);

    let query = FtsQuery::from_tokens(vec!["fly".to_string(), "speed".to_string()]).expect("query");
    let hits = SqliteIndexReader::open_read_only(&path)?.query_weighted_fts_index(
        &query,
        None,
        2,
        FtsColumnWeights::default(),
    )?;

    assert_eq!(
        record_key_strings(&hits),
        vec!["actions:testAction2", "actions:testAction1"]
    );
    fs::remove_file(path)?;
    Ok(())
}
