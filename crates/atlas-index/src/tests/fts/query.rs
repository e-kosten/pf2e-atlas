use std::fs;

use atlas_domain::{NumericMatch, RecordKey};
use rusqlite::Connection;

use super::record_key_strings;
use crate::{AtlasIndex, FilterCompileError, FtsColumnWeights, FtsQuery};

#[test]
fn ranked_query_respects_structured_filters() -> Result<(), Box<dyn std::error::Error>> {
    let path = super::temp_db_path("fts-filtered");
    super::create_contract_database(&path)?;
    let connection = Connection::open(&path)?;
    connection.execute(
        "UPDATE records
         SET level = CASE record_key
             WHEN 'actions:testAction1' THEN 1
             WHEN 'actions:testAction2' THEN 2
             ELSE 3
         END",
        [],
    )?;
    drop(connection);

    let filter = atlas_domain::SearchFilterNode::level(NumericMatch::Gte { value: 2.0 });
    let query = FtsQuery::from_tokens(vec!["action".to_string()]).expect("query");
    let hits = AtlasIndex::open_read_only(&path)?.query_fts_index(
        &query,
        Some(&filter),
        10,
        FtsColumnWeights::default(),
    )?;

    assert_eq!(
        record_key_strings(&hits),
        vec!["actions:testAction2", "actions:testAction3"]
    );
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn ranked_query_applies_structured_filters_to_or_fallback() -> Result<(), Box<dyn std::error::Error>>
{
    let path = super::temp_db_path("fts-filtered-fallback");
    super::create_contract_database(&path)?;
    let connection = Connection::open(&path)?;
    connection.execute(
        "UPDATE records
         SET level = CASE record_key
             WHEN 'actions:testAction1' THEN 1
             WHEN 'actions:testAction2' THEN 2
             ELSE 3
         END",
        [],
    )?;
    super::replace_fts_rows(
        &connection,
        &[
            (
                "actions:testAction1",
                "Alpha Beta",
                "",
                "",
                "",
                "",
                "",
                "",
                "",
            ),
            ("actions:testAction2", "Alpha", "", "", "", "", "", "", ""),
            ("actions:testAction3", "Other", "", "", "", "", "", "", ""),
        ],
    )?;
    drop(connection);

    let filter = atlas_domain::SearchFilterNode::level(NumericMatch::Gte { value: 2.0 });
    let query =
        FtsQuery::from_tokens(vec!["alpha".to_string(), "beta".to_string()]).expect("query");
    let hits = AtlasIndex::open_read_only(&path)?.query_fts_index(
        &query,
        Some(&filter),
        10,
        FtsColumnWeights::default(),
    )?;

    assert_eq!(record_key_strings(&hits), vec!["actions:testAction2"]);
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn candidate_key_query_is_bounded_to_supplied_candidates() -> Result<(), Box<dyn std::error::Error>>
{
    let path = super::temp_db_path("fts-candidate-keys");
    super::create_contract_database(&path)?;

    let query = FtsQuery::from_tokens(vec!["action".to_string()]).expect("query");
    let hits = AtlasIndex::open_read_only(&path)?.query_fts_candidate_record_keys(
        &query,
        &[
            RecordKey::parse("actions:testAction1")?,
            RecordKey::parse("actions:missing")?,
        ],
    )?;

    assert_eq!(
        hits.iter().map(|hit| hit.to_string()).collect::<Vec<_>>(),
        vec!["actions:testAction1"]
    );
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn candidate_key_query_uses_or_matching_for_multiple_tokens()
-> Result<(), Box<dyn std::error::Error>> {
    let path = super::temp_db_path("fts-candidate-keys-or");
    super::create_contract_database(&path)?;
    let connection = Connection::open(&path)?;
    super::replace_fts_rows(
        &connection,
        &[
            (
                "actions:testAction1",
                "Alpha Beta",
                "",
                "",
                "",
                "",
                "",
                "",
                "",
            ),
            ("actions:testAction2", "Alpha", "", "", "", "", "", "", ""),
            ("actions:testAction3", "Beta", "", "", "", "", "", "", ""),
        ],
    )?;
    drop(connection);

    let query =
        FtsQuery::from_tokens(vec!["alpha".to_string(), "beta".to_string()]).expect("query");
    let hits = AtlasIndex::open_read_only(&path)?.query_fts_candidate_record_keys(
        &query,
        &[
            RecordKey::parse("actions:testAction2")?,
            RecordKey::parse("actions:testAction3")?,
        ],
    )?;

    assert_eq!(
        hits.iter().map(|hit| hit.to_string()).collect::<Vec<_>>(),
        vec!["actions:testAction2", "actions:testAction3"]
    );
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn record_key_query_respects_structured_filters_order_and_limit()
-> Result<(), Box<dyn std::error::Error>> {
    let path = super::temp_db_path("fts-record-key-query");
    super::create_contract_database(&path)?;
    let connection = Connection::open(&path)?;
    connection.execute(
        "UPDATE records
         SET level = CASE record_key
             WHEN 'actions:testAction1' THEN 1
             WHEN 'actions:testAction2' THEN 2
             ELSE 3
         END",
        [],
    )?;
    drop(connection);

    let filter = atlas_domain::SearchFilterNode::level(NumericMatch::Gte { value: 2.0 });
    let query = FtsQuery::from_tokens(vec!["action".to_string()]).expect("query");
    let hits =
        AtlasIndex::open_read_only(&path)?.query_fts_record_keys(&query, Some(&filter), 1)?;

    assert_eq!(
        hits.iter().map(|hit| hit.to_string()).collect::<Vec<_>>(),
        vec!["actions:testAction2"]
    );
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn record_key_query_uses_or_matching_for_multiple_tokens() -> Result<(), Box<dyn std::error::Error>>
{
    let path = super::temp_db_path("fts-record-key-query-or");
    super::create_contract_database(&path)?;
    let connection = Connection::open(&path)?;
    super::replace_fts_rows(
        &connection,
        &[
            (
                "actions:testAction1",
                "Alpha Beta",
                "",
                "",
                "",
                "",
                "",
                "",
                "",
            ),
            ("actions:testAction2", "Alpha", "", "", "", "", "", "", ""),
            ("actions:testAction3", "Beta", "", "", "", "", "", "", ""),
        ],
    )?;
    drop(connection);

    let query =
        FtsQuery::from_tokens(vec!["alpha".to_string(), "beta".to_string()]).expect("query");
    let hits = AtlasIndex::open_read_only(&path)?.query_fts_record_keys(&query, None, 3)?;

    assert_eq!(
        hits.iter().map(|hit| hit.to_string()).collect::<Vec<_>>(),
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
fn query_drops_unsafe_tokens_before_rendering() {
    let query = FtsQuery::from_tokens(vec![
        "safe".to_string(),
        "unsafe\" OR anything".to_string(),
        "token2".to_string(),
    ])
    .expect("query");

    assert_eq!(query.as_match_query(), "\"safe\" OR \"token2\"");
    assert!(FtsQuery::from_tokens(vec!["\"".to_string()]).is_none());
}

#[test]
fn ranked_query_reports_invalid_record_keys_from_matching_fts_rows()
-> Result<(), Box<dyn std::error::Error>> {
    let path = super::temp_db_path("fts-invalid-record-key");
    super::create_contract_database(&path)?;
    let connection = Connection::open(&path)?;
    connection.execute(
        "UPDATE records
         SET record_key = 'invalid-record-key'
         WHERE record_key = 'actions:testAction1'",
        [],
    )?;
    super::replace_fts_rows(
        &connection,
        &[("invalid-record-key", "Needle", "", "", "", "", "", "", "")],
    )?;
    drop(connection);

    let query = FtsQuery::from_tokens(vec!["needle".to_string()]).expect("query");
    let error = AtlasIndex::open_read_only(&path)?
        .query_fts_index(&query, None, 10, FtsColumnWeights::default())
        .expect_err("invalid record key should fail FTS query");

    assert!(matches!(error, FilterCompileError::InvalidValue(_)));
    fs::remove_file(path)?;
    Ok(())
}
