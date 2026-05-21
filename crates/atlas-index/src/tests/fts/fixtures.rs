use rusqlite::Connection;

use crate::FtsColumnWeights;

type FtsFixtureRow<'a> = (
    &'a str,
    &'a str,
    &'a str,
    &'a str,
    &'a str,
    &'a str,
    &'a str,
    &'a str,
    &'a str,
);

pub(crate) fn replace_fts_rows(
    connection: &Connection,
    rows: &[FtsFixtureRow<'_>],
) -> Result<(), Box<dyn std::error::Error>> {
    connection.execute("DELETE FROM records_fts", [])?;
    for (
        record_key,
        title,
        aliases,
        taxonomy_terms,
        constraint_terms,
        mechanic_terms,
        source_terms,
        metric_terms,
        body,
    ) in rows
    {
        connection.execute(
            "INSERT INTO records_fts (
              record_key, title, aliases, traits, taxonomy_terms, constraint_terms, mechanic_terms,
              source_terms, metric_terms, headings, body, facts, reference_terms, embedded_content
             ) VALUES (?1, ?2, ?3, '', ?4, ?5, ?6, ?7, ?8, '', ?9, '', '', '')",
            (
                record_key,
                title,
                aliases,
                taxonomy_terms,
                constraint_terms,
                mechanic_terms,
                source_terms,
                metric_terms,
                body,
            ),
        )?;
    }
    Ok(())
}

pub(crate) fn replace_single_column_rows(
    connection: &Connection,
    target_column: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    connection.execute("DELETE FROM records_fts", [])?;
    insert_single_column_row(connection, "actions:testAction1", target_column, "needle")?;
    let alternate_column = if target_column == "body" {
        "embedded_content"
    } else {
        "body"
    };
    insert_single_column_row(
        connection,
        "actions:testAction2",
        alternate_column,
        "needle needle",
    )?;
    Ok(())
}

fn insert_single_column_row(
    connection: &Connection,
    record_key: &str,
    target_column: &str,
    value: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let column_value = |column| {
        if column == target_column { value } else { "" }
    };
    connection.execute(
        "INSERT INTO records_fts (
          record_key, title, aliases, traits, taxonomy_terms, constraint_terms, mechanic_terms,
          source_terms, metric_terms, headings, body, facts, reference_terms, embedded_content
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
        (
            record_key,
            column_value("title"),
            column_value("aliases"),
            column_value("traits"),
            column_value("taxonomy_terms"),
            column_value("constraint_terms"),
            column_value("mechanic_terms"),
            column_value("source_terms"),
            column_value("metric_terms"),
            column_value("headings"),
            column_value("body"),
            column_value("facts"),
            column_value("reference_terms"),
            column_value("embedded_content"),
        ),
    )?;
    Ok(())
}

pub(crate) fn weights_for_column(column: &str) -> FtsColumnWeights {
    let mut weights = FtsColumnWeights {
        title: 0.1,
        aliases: 0.1,
        traits: 0.1,
        taxonomy_terms: 0.1,
        constraint_terms: 0.1,
        mechanic_terms: 0.1,
        source_terms: 0.1,
        metric_terms: 0.1,
        headings: 0.1,
        body: 0.1,
        facts: 0.1,
        reference_terms: 0.1,
        embedded_content: 0.1,
    };
    match column {
        "title" => weights.title = 12.0,
        "aliases" => weights.aliases = 12.0,
        "traits" => weights.traits = 12.0,
        "taxonomy_terms" => weights.taxonomy_terms = 12.0,
        "constraint_terms" => weights.constraint_terms = 12.0,
        "mechanic_terms" => weights.mechanic_terms = 12.0,
        "source_terms" => weights.source_terms = 12.0,
        "metric_terms" => weights.metric_terms = 12.0,
        "headings" => weights.headings = 12.0,
        "body" => weights.body = 12.0,
        "facts" => weights.facts = 12.0,
        "reference_terms" => weights.reference_terms = 12.0,
        "embedded_content" => weights.embedded_content = 12.0,
        unknown => panic!("unknown FTS column fixture: {unknown}"),
    }
    weights
}

pub(crate) fn insert_fixture_record(
    connection: &Connection,
    record_key: &str,
    record_family: &str,
    foundry_record_type: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let id = record_key
        .split(':')
        .nth(1)
        .ok_or_else(|| format!("invalid fixture key: {record_key}"))?;
    let name = id.to_string();
    let source_path = format!("packs/actions/{id}.json");
    connection.execute(
        "INSERT INTO records (
          record_key, id, name, normalized_name, record_family, pack_name, pack_label,
          foundry_document_type, foundry_record_type, traits_json, prerequisites_json,
          publication_remaster, publication_family, taxonomy_families_json, variant_axes_json,
          variant_source, source_path, is_default_visible, raw_json
        ) VALUES (?1, ?2, ?3, ?4, ?5, 'actions', 'Actions', 'Item', ?6,
          '[]', '[]', 0, 'unknown', '[]', '[]', 'none', ?7, 1, '{}')",
        (
            record_key,
            id,
            name.as_str(),
            name.to_lowercase(),
            record_family,
            foundry_record_type,
            source_path,
        ),
    )?;
    Ok(())
}

pub(crate) fn record_key_strings(hits: &[crate::FtsSearchHit]) -> Vec<String> {
    hits.iter().map(|hit| hit.record_key.to_string()).collect()
}
