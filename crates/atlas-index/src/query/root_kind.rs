//! Shared classification for result summaries, query eligibility and discovery.
use crate::{IndexError, SqliteIndexReader};
use atlas_domain::QueryLiteral;
use atlas_domain::root_kind::{JOURNAL_PACK_PREFIX, ROOT_KINDS};
use rusqlite::{Connection, params_from_iter, types::Value};
use std::collections::{BTreeMap, BTreeSet};

/// Evaluated with `records r`. NULL means that a public selection could not be
/// established; campaign records with no specific known category retain their group.
pub const CLASSIFICATION_SQL: &str = include_str!("root_kind.sql");

pub(super) fn field_state() -> String {
    format!("CASE WHEN ({CLASSIFICATION_SQL}) IS NOT NULL THEN 'value'
        WHEN r.source_type='feat' THEN coalesce(
            nullif((SELECT category_state FROM ability_projection WHERE root_record_id=r.record_id),'value'),
            'not_applicable')
        WHEN r.source_type_state != 'value' THEN r.source_type_state
        ELSE 'not_applicable' END")
}

/// A selector can name the specific classification or any of its parent's
/// descendants. Keep predicate structure independent of the number of children.
pub(super) fn matches(
    values: &[QueryLiteral],
    parameters: &mut Vec<Value>,
) -> Result<String, IndexError> {
    let mut accepted = BTreeSet::new();
    let mut any_journal = false;
    for value in values {
        let QueryLiteral::String(id) = value else {
            return Err(IndexError::Invalid(
                "validated kind must be a string".into(),
            ));
        };
        any_journal |= id == "journal";
        accepted.insert(id.as_str());
        accepted.extend(
            ROOT_KINDS
                .iter()
                .filter(|child| child.parent == Some(id))
                .map(|child| child.id),
        );
    }
    // A campaign feature whose authored category is unavailable still proves the
    // parent, but negating a possible campaign feature subtype must stay unknown.
    let partial_campaign = !accepted.contains("kingdom_and_warfare")
        && accepted.iter().any(|id| {
            *id != "army" && atlas_domain::root_kind::parent(id) == Some("kingdom_and_warfare")
        });
    let classification = if partial_campaign {
        format!("nullif(({CLASSIFICATION_SQL}),'kingdom_and_warfare')")
    } else {
        format!("({CLASSIFICATION_SQL})")
    };
    let slots = accepted
        .into_iter()
        .map(|id| super::compiler::parameter(parameters, Value::Text(id.into())))
        .collect::<Vec<_>>()
        .join(",");
    let membership = format!("{classification} IN ({slots})");
    Ok(if any_journal {
        format!("(r.document_kind='JournalEntry' OR {membership})")
    } else {
        membership
    })
}

pub(crate) fn counts(
    connection: &Connection,
    eligible: &str,
    parameters: &[Value],
) -> Result<BTreeMap<String, u64>, IndexError> {
    let sql = format!(
        "SELECT {CLASSIFICATION_SQL}, count(*) FROM records r
        WHERE r.record_id IN ({eligible}) GROUP BY 1"
    );
    let mut stmt = connection.prepare(&sql)?;
    let rows = stmt.query_map(params_from_iter(parameters), |row| {
        Ok((row.get::<_, Option<String>>(0)?, row.get::<_, u64>(1)?))
    })?;
    let mut counts = BTreeMap::new();
    for row in rows {
        let (Some(id), count) = row? else { continue };
        if let Some(parent) = atlas_domain::root_kind::parent(&id) {
            *counts.entry(parent.to_owned()).or_default() += count;
        }
        *counts.entry(id).or_default() += count;
    }
    Ok(counts)
}

fn packs(connection: &Connection) -> Result<Vec<(String, String)>, IndexError> {
    let mut stmt = connection.prepare(
        "SELECT p.pack_id,p.label FROM packs p JOIN
         (SELECT DISTINCT pack_id FROM records WHERE document_kind='JournalEntry') journals USING(pack_id)
         ORDER BY p.pack_id"
    )?;
    Ok(stmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
        .collect::<Result<_, _>>()?)
}

pub(crate) fn pack_choices(
    connection: &Connection,
) -> Result<BTreeMap<String, String>, IndexError> {
    Ok(packs(connection)?
        .into_iter()
        .map(|(id, label)| (format!("{JOURNAL_PACK_PREFIX}{id}"), label))
        .collect())
}

impl SqliteIndexReader {
    /// Current source labels keyed by raw pack ID, independent of query filters.
    pub fn journal_kind_packs(&self) -> Result<Vec<(String, String)>, IndexError> {
        self.count_statement();
        packs(self.native()?)
    }
}
