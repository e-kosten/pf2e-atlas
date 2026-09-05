use atlas_domain::{RecordKey, RecordKind, SearchFilterNode};
use atlas_record::{
    ActorMechanics, AtlasRecord, AtlasRecordSet, FoundryDocumentMechanics, FoundryRecordType,
    ItemMechanics, ItemTypeMechanics, RetrievedRecord, SpellMechanics,
};
use diesel::SqliteConnection;
use thiserror::Error;

use crate::sqlite::SqliteIndexReader;
use crate::{FilterCompileError, RecordIdentityMatch, SearchCandidateRecord};

mod candidates;
mod canonical;
pub(crate) mod children;
mod content;
mod identity;
mod mechanics;
mod metrics;
mod parse;
mod relationships;
mod rows;

#[cfg(test)]
thread_local! {
    static CANONICAL_COHERENCE_SCAN_COUNT: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

#[derive(Debug, Error)]
pub enum RecordLoadError {
    #[error("index is unavailable: {0}")]
    Unavailable(String),
    #[error("record query failed: {0}")]
    QueryFailed(String),
    #[error("record data is invalid: {0}")]
    InvalidData(String),
}

pub fn load_persisted_record_set_from_diesel_connection(
    connection: &mut SqliteConnection,
) -> Result<AtlasRecordSet, RecordLoadError> {
    Ok(AtlasRecordSet {
        records: load_persisted_records_from_diesel_connection(connection)?,
        reference_edges: relationships::read_reference_edges(connection)?,
        aliases: relationships::read_aliases(connection)?,
        remaster_links: relationships::read_remaster_links(connection)?,
    })
}

pub fn load_persisted_records_from_diesel_connection(
    connection: &mut SqliteConnection,
) -> Result<Vec<AtlasRecord>, RecordLoadError> {
    let mut records = rows::read_record_rows(connection)?;
    attach_record_details(connection, &mut records)?;
    Ok(records)
}

pub fn load_persisted_records_by_key_from_diesel_connection(
    connection: &mut SqliteConnection,
    keys: &[RecordKey],
) -> Result<Vec<AtlasRecord>, RecordLoadError> {
    let mut records = rows::read_record_rows_by_keys(connection, keys)?;
    attach_record_details_by_key(connection, &mut records, keys)?;
    Ok(records)
}

pub fn load_search_candidate_records_from_diesel_connection(
    connection: &mut SqliteConnection,
    keys: &[RecordKey],
) -> Result<Vec<SearchCandidateRecord>, RecordLoadError> {
    candidates::read_search_candidate_records_by_keys(connection, keys)
}

pub fn resolve_record_identity_matches_from_diesel_connection(
    connection: &mut SqliteConnection,
    query: &str,
    normalized_query: &str,
    filter: Option<&SearchFilterNode>,
) -> Result<Vec<RecordIdentityMatch>, FilterCompileError> {
    identity::resolve_record_identity_matches(connection, query, normalized_query, filter)
}

impl SqliteIndexReader {
    /// Runs the optional broad canonical hydration and relational projection
    /// reconciliation diagnostic. Normal validation targets intentionally use
    /// structural/global checks plus strict decoding of records when requested.
    pub fn validate_canonical_coherence(
        &self,
    ) -> Result<Vec<crate::ArtifactValidationDiagnostic>, crate::IndexValidationError> {
        #[cfg(test)]
        CANONICAL_COHERENCE_SCAN_COUNT.set(CANONICAL_COHERENCE_SCAN_COUNT.get() + 1);
        let connection = self.validation_connection()?;
        let mut diagnostics = Vec::new();
        crate::artifact::validation::canonical::validate_canonical_records(
            &connection,
            &mut diagnostics,
        )?;
        Ok(diagnostics)
    }

    pub fn load_canonical_record_bodies(
        &self,
    ) -> Result<Vec<atlas_record::RecordBody>, RecordLoadError> {
        self.with_diesel_connection(canonical::read_canonical_record_bodies)
    }

    pub fn load_canonical_record_bodies_by_key(
        &self,
        keys: &[RecordKey],
    ) -> Result<Vec<atlas_record::RecordBody>, RecordLoadError> {
        self.with_diesel_connection(|connection| {
            canonical::read_canonical_record_bodies_by_key(connection, keys)
        })
    }

    pub fn load_hydrated_records(&self) -> Result<Vec<RetrievedRecord>, RecordLoadError> {
        self.with_diesel_connection(|connection| {
            let records = load_persisted_records_from_diesel_connection(connection)?;
            let bodies =
                canonical::bodies_by_key(canonical::read_canonical_record_bodies(connection)?);
            hydrate_record_parts(records, bodies)
        })
    }

    pub fn load_hydrated_records_by_key(
        &self,
        keys: &[RecordKey],
    ) -> Result<Vec<RetrievedRecord>, RecordLoadError> {
        self.with_diesel_connection(|connection| {
            let records = load_persisted_records_by_key_from_diesel_connection(connection, keys)?;
            let bodies = canonical::bodies_by_key(canonical::read_canonical_record_bodies_by_key(
                connection, keys,
            )?);
            hydrate_record_parts(records, bodies)
        })
    }

    pub fn load_records(&self) -> Result<Vec<AtlasRecord>, RecordLoadError> {
        self.with_diesel_connection(load_persisted_records_from_diesel_connection)
    }

    pub fn load_record_set(&self) -> Result<AtlasRecordSet, RecordLoadError> {
        self.with_diesel_connection(load_persisted_record_set_from_diesel_connection)
    }

    pub fn load_records_by_key(
        &self,
        keys: &[RecordKey],
    ) -> Result<Vec<AtlasRecord>, RecordLoadError> {
        self.with_diesel_connection(|connection| {
            load_persisted_records_by_key_from_diesel_connection(connection, keys)
        })
    }

    pub fn load_search_candidate_records(
        &self,
        keys: &[RecordKey],
    ) -> Result<Vec<SearchCandidateRecord>, RecordLoadError> {
        self.with_diesel_connection(|connection| {
            load_search_candidate_records_from_diesel_connection(connection, keys)
        })
    }

    pub fn resolve_record_identity_matches(
        &self,
        query: &str,
        normalized_query: &str,
        filter: Option<&SearchFilterNode>,
    ) -> Result<Vec<RecordIdentityMatch>, FilterCompileError> {
        self.with_diesel_connection(|connection| {
            resolve_record_identity_matches_from_diesel_connection(
                connection,
                query,
                normalized_query,
                filter,
            )
        })
    }
}

#[cfg(test)]
pub(crate) fn reset_canonical_coherence_scan_count() {
    CANONICAL_COHERENCE_SCAN_COUNT.set(0);
}

#[cfg(test)]
pub(crate) fn canonical_coherence_scan_count() -> usize {
    CANONICAL_COHERENCE_SCAN_COUNT.get()
}

/// Applies the same canonical-body ownership checks used by artifact reads to
/// already decoded record and canonical-body values.
pub fn hydrate_record_parts(
    records: Vec<AtlasRecord>,
    mut bodies: std::collections::BTreeMap<RecordKey, atlas_record::RecordBody>,
) -> Result<Vec<RetrievedRecord>, RecordLoadError> {
    let mut hydrated = Vec::with_capacity(records.len());
    for record in records {
        let body = bodies.remove(&record.identity.key);
        match (&record.foundry.record_type, body.is_some()) {
            (FoundryRecordType::Npc, false) => {
                return Err(RecordLoadError::InvalidData(format!(
                    "v2 canonical NPC `{}` is missing its required creature body",
                    record.identity.key
                )));
            }
            (FoundryRecordType::Npc, true) | (_, false) => {}
            (_, true) => {
                return Err(RecordLoadError::InvalidData(format!(
                    "non-NPC record `{}` has an unexpected canonical creature body",
                    record.identity.key
                )));
            }
        }
        hydrated.push(RetrievedRecord { record, body });
    }
    if let Some(extra) = bodies.keys().next() {
        return Err(RecordLoadError::InvalidData(format!(
            "canonical creature body `{extra}` has no matching persisted record"
        )));
    }
    Ok(hydrated)
}

fn attach_record_details(
    connection: &mut SqliteConnection,
    records: &mut [AtlasRecord],
) -> Result<(), RecordLoadError> {
    let metrics = metrics::read_metrics(connection)?;
    let actor_data = mechanics::read_actor_mechanics(connection)?;
    let item_data = mechanics::read_item_mechanics(connection)?;
    let spell_data = mechanics::read_spell_mechanics(connection)?;
    let supplemental_content = content::read_record_content(connection)?;

    for record in records {
        let key = record.identity.key.to_string();
        if record.classification.kind != RecordKind::Creature {
            if let Some(rows) = metrics.get(&key) {
                record.mechanics.metrics.clone_from(rows);
            }
            record.mechanics.document =
                document_mechanics_for_key(&key, &actor_data, &item_data, &spell_data);
        }
        if let Some(documents) = supplemental_content.get(&key) {
            record.content.documents.extend(documents.iter().cloned());
        }
    }

    Ok(())
}

fn attach_record_details_by_key(
    connection: &mut SqliteConnection,
    records: &mut [AtlasRecord],
    keys: &[RecordKey],
) -> Result<(), RecordLoadError> {
    if records.is_empty() {
        return Ok(());
    }

    let metrics = metrics::read_metrics_by_keys(connection, keys)?;
    let actor_data = mechanics::read_actor_mechanics_by_keys(connection, keys)?;
    let item_data = mechanics::read_item_mechanics_by_keys(connection, keys)?;
    let spell_data = mechanics::read_spell_mechanics_by_keys(connection, keys)?;
    let supplemental_content = content::read_record_content_by_keys(connection, keys)?;

    for record in records {
        let key = record.identity.key.to_string();
        if record.classification.kind != RecordKind::Creature {
            if let Some(rows) = metrics.get(&key) {
                record.mechanics.metrics.clone_from(rows);
            }
            record.mechanics.document =
                document_mechanics_for_key(&key, &actor_data, &item_data, &spell_data);
        }
        if let Some(documents) = supplemental_content.get(&key) {
            record.content.documents.extend(documents.iter().cloned());
        }
    }

    Ok(())
}

fn document_mechanics_for_key(
    key: &str,
    actor_data: &std::collections::BTreeMap<String, ActorMechanics>,
    item_data: &std::collections::BTreeMap<String, ItemMechanics>,
    spell_data: &std::collections::BTreeMap<String, SpellMechanics>,
) -> FoundryDocumentMechanics {
    if let Some(actor) = actor_data.get(key).cloned() {
        return FoundryDocumentMechanics::Actor(actor);
    }
    if let Some(mut item) = item_data.get(key).cloned() {
        item.foundry_type = spell_data.get(key).cloned().map(ItemTypeMechanics::Spell);
        return FoundryDocumentMechanics::Item(item);
    }
    FoundryDocumentMechanics::None
}
