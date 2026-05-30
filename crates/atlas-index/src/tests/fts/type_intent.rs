use std::fs;

use rusqlite::Connection;

use super::{insert_fixture_record, record_key_strings, replace_fts_rows};
use crate::{AtlasIndex, FtsColumnWeights, FtsQuery};

#[test]
fn type_intent_can_promote_matching_family_under_low_final_limit()
-> Result<(), Box<dyn std::error::Error>> {
    let path = super::temp_db_path("fts-type-intent");
    super::create_contract_database(&path)?;
    let connection = Connection::open(&path)?;
    connection.execute(
        "UPDATE records
         SET record_family = CASE record_key
             WHEN 'actions:testAction1' THEN 'creature'
             WHEN 'actions:testAction2' THEN 'spell'
             ELSE record_family
         END,
             foundry_record_type = CASE record_key
             WHEN 'actions:testAction1' THEN 'npc'
             WHEN 'actions:testAction2' THEN 'spell'
             ELSE foundry_record_type
         END",
        [],
    )?;
    replace_fts_rows(
        &connection,
        &[
            (
                "actions:testAction1",
                "Fire Fire Fire",
                "",
                "",
                "",
                "",
                "",
                "",
                "",
            ),
            (
                "actions:testAction2",
                "Other",
                "",
                "spell",
                "",
                "",
                "",
                "",
                "fire",
            ),
            (
                "actions:testAction3",
                "Fire Rule",
                "",
                "",
                "",
                "",
                "",
                "",
                "",
            ),
        ],
    )?;
    drop(connection);

    let query =
        FtsQuery::from_tokens(vec!["spell".to_string(), "fire".to_string()]).expect("query");
    assert_eq!(query.as_conjunction_match_query(), "\"fire\"");
    let hits = AtlasIndex::open_read_only(&path)?.query_weighted_fts_index(
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
fn mixed_case_tokens_are_normalized_before_type_intent_lowering()
-> Result<(), Box<dyn std::error::Error>> {
    let path = super::temp_db_path("fts-mixed-case-type-intent");
    super::create_contract_database(&path)?;
    let connection = Connection::open(&path)?;
    connection.execute(
        "UPDATE records
         SET record_family = CASE record_key
             WHEN 'actions:testAction2' THEN 'spell'
             ELSE 'creature'
         END,
             foundry_record_type = CASE record_key
             WHEN 'actions:testAction2' THEN 'spell'
             ELSE 'npc'
         END",
        [],
    )?;
    replace_fts_rows(
        &connection,
        &[
            (
                "actions:testAction1",
                "Fire Fire Fire",
                "",
                "",
                "",
                "",
                "",
                "",
                "",
            ),
            (
                "actions:testAction2",
                "Other",
                "",
                "spell",
                "",
                "",
                "",
                "",
                "fire",
            ),
            (
                "actions:testAction3",
                "Fire Rule",
                "",
                "",
                "",
                "",
                "",
                "",
                "",
            ),
        ],
    )?;
    drop(connection);

    let query =
        FtsQuery::from_tokens(vec!["Spell".to_string(), "Fire".to_string()]).expect("query");
    assert_eq!(query.as_match_query(), "\"spell\" OR \"fire\"");
    assert_eq!(query.as_conjunction_match_query(), "\"fire\"");
    let hits = AtlasIndex::open_read_only(&path)?.query_weighted_fts_index(
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
fn type_intent_uses_internal_candidate_window_before_final_truncation()
-> Result<(), Box<dyn std::error::Error>> {
    let path = super::temp_db_path("fts-type-intent-candidate-window");
    super::create_contract_database(&path)?;
    let connection = Connection::open(&path)?;
    connection.execute(
        "UPDATE records
         SET record_family = CASE record_key
             WHEN 'actions:testAction2' THEN 'spell'
             ELSE 'creature'
         END,
             foundry_record_type = CASE record_key
             WHEN 'actions:testAction2' THEN 'spell'
             ELSE 'npc'
         END",
        [],
    )?;
    for index in 4..=8 {
        insert_fixture_record(
            &connection,
            &format!("actions:generic{index}"),
            "creature",
            "npc",
        )?;
    }
    replace_fts_rows(
        &connection,
        &[
            (
                "actions:testAction1",
                "Fire Fire Fire",
                "",
                "",
                "",
                "",
                "",
                "",
                "",
            ),
            (
                "actions:testAction2",
                "Other",
                "",
                "spell",
                "",
                "",
                "",
                "",
                "fire",
            ),
            (
                "actions:testAction3",
                "Fire Fire Fire",
                "",
                "",
                "",
                "",
                "",
                "",
                "",
            ),
            (
                "actions:generic4",
                "Fire Fire Fire",
                "",
                "",
                "",
                "",
                "",
                "",
                "",
            ),
            (
                "actions:generic5",
                "Fire Fire Fire",
                "",
                "",
                "",
                "",
                "",
                "",
                "",
            ),
            (
                "actions:generic6",
                "Fire Fire Fire",
                "",
                "",
                "",
                "",
                "",
                "",
                "",
            ),
            (
                "actions:generic7",
                "Fire Fire Fire",
                "",
                "",
                "",
                "",
                "",
                "",
                "",
            ),
            (
                "actions:generic8",
                "Fire Fire Fire",
                "",
                "",
                "",
                "",
                "",
                "",
                "",
            ),
        ],
    )?;
    drop(connection);

    let query =
        FtsQuery::from_tokens(vec!["spell".to_string(), "fire".to_string()]).expect("query");
    let hits = AtlasIndex::open_read_only(&path)?.query_weighted_fts_index(
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
fn secondary_type_intent_promotes_matching_equipment_type_on_database_path()
-> Result<(), Box<dyn std::error::Error>> {
    let path = super::temp_db_path("fts-secondary-type-intent");
    super::create_contract_database(&path)?;
    let connection = Connection::open(&path)?;
    connection.execute(
        "UPDATE records
         SET record_family = CASE record_key
             WHEN 'actions:testAction1' THEN 'creature'
             WHEN 'actions:testAction2' THEN 'equipment'
             ELSE 'creature'
         END,
             foundry_record_type = CASE record_key
             WHEN 'actions:testAction1' THEN 'npc'
             WHEN 'actions:testAction2' THEN 'weapon'
             ELSE 'npc'
         END",
        [],
    )?;
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
                "weapon filler fire weapon filler fire",
            ),
            (
                "actions:testAction2",
                "Other",
                "",
                "weapon",
                "",
                "",
                "",
                "",
                "fire",
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
                "fire weapon rule",
            ),
        ],
    )?;
    drop(connection);

    let query =
        FtsQuery::from_tokens(vec!["weapon".to_string(), "fire".to_string()]).expect("query");
    assert_eq!(query.as_conjunction_match_query(), "\"weapon\" \"fire\"");
    let hits = AtlasIndex::open_read_only(&path)?.query_weighted_fts_index(
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
fn all_type_intent_query_skips_strict_and_uses_or_fallback()
-> Result<(), Box<dyn std::error::Error>> {
    let path = super::temp_db_path("fts-type-intent-only");
    super::create_contract_database(&path)?;
    let connection = Connection::open(&path)?;
    connection.execute(
        "UPDATE records
         SET record_family = CASE record_key
             WHEN 'actions:testAction2' THEN 'spell'
             ELSE record_family
         END,
             foundry_record_type = CASE record_key
             WHEN 'actions:testAction2' THEN 'spell'
             ELSE foundry_record_type
         END",
        [],
    )?;
    replace_fts_rows(
        &connection,
        &[
            (
                "actions:testAction1",
                "Spell-Like Rule",
                "",
                "",
                "",
                "",
                "",
                "",
                "spell",
            ),
            (
                "actions:testAction2",
                "Spell Record",
                "",
                "spell",
                "",
                "",
                "",
                "",
                "",
            ),
            ("actions:testAction3", "Other", "", "", "", "", "", "", ""),
        ],
    )?;
    drop(connection);

    let query = FtsQuery::from_tokens(vec!["spell".to_string()]).expect("query");
    assert_eq!(query.as_conjunction_match_query(), "");
    let hits = AtlasIndex::open_read_only(&path)?.query_weighted_fts_index(
        &query,
        None,
        1,
        FtsColumnWeights::default(),
    )?;

    assert_eq!(record_key_strings(&hits), vec!["actions:testAction2"]);
    fs::remove_file(path)?;
    Ok(())
}
