use atlas_domain::{RecordKey, RecordKind, SearchFilterNode};
use atlas_record::{
    ActorMechanics, AtlasRecord, AtlasRecordSet, FoundryDocumentMechanics, FoundryRecordType,
    ItemMechanics, RetrievedRecord, SpellRecord,
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
        self.with_diesel_connection(|connection| {
            let bodies = canonical::read_canonical_record_bodies(connection)?;
            let by_key = canonical::bodies_by_key(bodies.clone())?;
            canonical::reconcile_spell_query_projections(connection, &by_key, None)?;
            canonical::reconcile_consumable_query_projections(connection, &by_key, None)?;
            Ok(bodies)
        })
    }

    pub fn load_canonical_record_bodies_by_key(
        &self,
        keys: &[RecordKey],
    ) -> Result<Vec<atlas_record::RecordBody>, RecordLoadError> {
        self.with_diesel_connection(|connection| {
            let bodies = canonical::read_canonical_record_bodies_by_key(connection, keys)?;
            let by_key = canonical::bodies_by_key(bodies.clone())?;
            canonical::reconcile_spell_query_projections(connection, &by_key, Some(keys))?;
            canonical::reconcile_consumable_query_projections(connection, &by_key, Some(keys))?;
            Ok(bodies)
        })
    }

    pub fn load_hydrated_records(&self) -> Result<Vec<RetrievedRecord>, RecordLoadError> {
        self.with_diesel_connection(|connection| {
            let records = load_persisted_records_from_diesel_connection(connection)?;
            let bodies =
                canonical::bodies_by_key(canonical::read_canonical_record_bodies(connection)?)?;
            canonical::reconcile_spell_query_projections(connection, &bodies, None)?;
            canonical::reconcile_consumable_query_projections(connection, &bodies, None)?;
            let children =
                canonical::spell_children_by_parent(canonical::read_spell_children(connection)?)?;
            let occurrences = canonical::read_consumable_occurrences(connection, None)?;
            canonical::reconcile_spell_owned_projections(connection, &bodies, &children)?;
            hydrate_record_parts(records, bodies, children, occurrences)
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
            )?)?;
            canonical::reconcile_spell_query_projections(connection, &bodies, Some(keys))?;
            canonical::reconcile_consumable_query_projections(connection, &bodies, Some(keys))?;
            let children = canonical::spell_children_by_parent(
                canonical::read_spell_children_by_parent_key(connection, keys)?,
            )?;
            let occurrences = canonical::read_consumable_occurrences(connection, Some(keys))?;
            canonical::reconcile_spell_owned_projections(connection, &bodies, &children)?;
            hydrate_record_parts(records, bodies, children, occurrences)
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
    mut spell_children: std::collections::BTreeMap<
        RecordKey,
        Vec<atlas_record::ConsumableSpellChild>,
    >,
    mut consumable_occurrences: std::collections::BTreeMap<
        RecordKey,
        atlas_record::ConsumableOccurrenceSet,
    >,
) -> Result<Vec<RetrievedRecord>, RecordLoadError> {
    let mut hydrated = Vec::with_capacity(records.len());
    for record in records {
        let body = bodies.remove(&record.identity.key);
        let expected_family = expected_canonical_body_family(&record)?;
        match (expected_family, body.as_ref()) {
            (Some(CanonicalBodyFamily::Creature), Some(atlas_record::RecordBody::Creature(_)))
            | (Some(CanonicalBodyFamily::Hazard), Some(atlas_record::RecordBody::Hazard(_)))
            | (None, None) => {}
            (Some(CanonicalBodyFamily::Spell), Some(atlas_record::RecordBody::Spell(spell))) => {
                validate_spell_body_owner(&record, spell)?;
            }
            (
                Some(CanonicalBodyFamily::Consumable),
                Some(atlas_record::RecordBody::Consumable(consumable)),
            ) => {
                validate_consumable_body_owner(&record, consumable)?;
            }
            (Some(expected), None) => {
                return Err(RecordLoadError::InvalidData(format!(
                    "canonical {} `{}` is missing its required body",
                    expected.as_str(),
                    record.identity.key
                )));
            }
            (expected, Some(actual)) => {
                let expected = expected.map_or("no", CanonicalBodyFamily::as_str);
                let actual = match actual {
                    atlas_record::RecordBody::Creature(_) => "creature",
                    atlas_record::RecordBody::Hazard(_) => "hazard",
                    atlas_record::RecordBody::Spell(_) => "spell",
                    atlas_record::RecordBody::Consumable(_) => "consumable",
                };
                return Err(RecordLoadError::InvalidData(format!(
                    "record `{}` expects {expected} canonical body but has an unexpected canonical {actual} body",
                    record.identity.key,
                )));
            }
        }
        let children = spell_children
            .remove(&record.identity.key)
            .unwrap_or_default();
        let occurrences = consumable_occurrences
            .remove(&record.identity.key)
            .unwrap_or_default();
        if !children.is_empty() && record.foundry.record_type != FoundryRecordType::Consumable {
            return Err(RecordLoadError::InvalidData(format!(
                "non-consumable record `{}` has consumable spell children",
                record.identity.key
            )));
        }
        if (!occurrences.entities.is_empty() || !occurrences.occurrences.is_empty())
            && !matches!(
                record.foundry.record_type,
                FoundryRecordType::Npc | FoundryRecordType::Character | FoundryRecordType::Hazard
            )
        {
            return Err(RecordLoadError::InvalidData(format!(
                "record `{}` cannot own consumable occurrences",
                record.identity.key
            )));
        }
        hydrated.push(RetrievedRecord {
            record,
            body,
            spell_children: children,
            consumable_occurrences: occurrences,
        });
    }
    if let Some(extra) = bodies.keys().next() {
        return Err(RecordLoadError::InvalidData(format!(
            "canonical body `{extra}` has no matching persisted record"
        )));
    }
    if let Some(extra) = spell_children.keys().next() {
        return Err(RecordLoadError::InvalidData(format!(
            "consumable spell children for `{extra}` have no matching persisted record"
        )));
    }
    if let Some(extra) = consumable_occurrences.keys().next() {
        return Err(RecordLoadError::InvalidData(format!(
            "consumable occurrences for `{extra}` have no matching persisted record"
        )));
    }
    Ok(hydrated)
}

#[derive(Debug, Clone, Copy)]
enum CanonicalBodyFamily {
    Creature,
    Hazard,
    Spell,
    Consumable,
}

impl CanonicalBodyFamily {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Creature => "creature",
            Self::Hazard => "hazard",
            Self::Spell => "spell",
            Self::Consumable => "consumable",
        }
    }
}

fn expected_canonical_body_family(
    record: &AtlasRecord,
) -> Result<Option<CanonicalBodyFamily>, RecordLoadError> {
    match (record.classification.kind, &record.foundry.record_type) {
        (RecordKind::Creature, FoundryRecordType::Npc) => Ok(Some(CanonicalBodyFamily::Creature)),
        (RecordKind::Hazard, FoundryRecordType::Hazard) => Ok(Some(CanonicalBodyFamily::Hazard)),
        (RecordKind::Spell, FoundryRecordType::Spell) => Ok(Some(CanonicalBodyFamily::Spell)),
        (RecordKind::Equipment, FoundryRecordType::Consumable) => {
            Ok(Some(CanonicalBodyFamily::Consumable))
        }
        (RecordKind::Creature, actual) => Err(RecordLoadError::InvalidData(format!(
            "creature record `{}` has Foundry type `{}` instead of `npc`",
            record.identity.key,
            actual.as_str(),
        ))),
        (RecordKind::Hazard, actual) => Err(RecordLoadError::InvalidData(format!(
            "hazard record `{}` has Foundry type `{}` instead of `hazard`",
            record.identity.key,
            actual.as_str(),
        ))),
        (RecordKind::Spell, actual) => Err(RecordLoadError::InvalidData(format!(
            "spell record `{}` has Foundry type `{}` instead of `spell`",
            record.identity.key,
            actual.as_str(),
        ))),
        (actual, FoundryRecordType::Npc) => Err(RecordLoadError::InvalidData(format!(
            "Foundry NPC `{}` has record kind `{}` instead of `creature`",
            record.identity.key,
            actual.as_str(),
        ))),
        (actual, FoundryRecordType::Hazard) => Err(RecordLoadError::InvalidData(format!(
            "Foundry hazard `{}` has record kind `{}` instead of `hazard`",
            record.identity.key,
            actual.as_str(),
        ))),
        (actual, FoundryRecordType::Spell) => Err(RecordLoadError::InvalidData(format!(
            "Foundry spell `{}` has record kind `{}` instead of `spell`",
            record.identity.key,
            actual.as_str(),
        ))),
        (actual, FoundryRecordType::Consumable) => Err(RecordLoadError::InvalidData(format!(
            "Foundry consumable `{}` has record kind `{}` instead of `equipment`",
            record.identity.key,
            actual.as_str(),
        ))),
        _ => Ok(None),
    }
}

fn validate_consumable_body_owner(
    record: &AtlasRecord,
    consumable: &atlas_record::ConsumableRecord,
) -> Result<(), RecordLoadError> {
    if consumable.identity.record_key != record.identity.key
        || consumable.identity.name != record.identity.name
        || consumable.identity.source_id.as_str() != record.identity.id().as_str()
    {
        return Err(RecordLoadError::InvalidData(format!(
            "consumable body identity for `{}` does not match its generic record owner",
            record.identity.key
        )));
    }
    Ok(())
}
pub(super) fn validate_spell_body_owner(
    record: &AtlasRecord,
    spell: &SpellRecord,
) -> Result<(), RecordLoadError> {
    if spell.identity.record_key != record.identity.key
        || spell.identity.name != record.identity.name
        || spell.identity.source_id.as_str() != record.identity.id().as_str()
    {
        return Err(RecordLoadError::InvalidData(format!(
            "canonical spell body identity for `{}` does not match its generic record owner",
            record.identity.key
        )));
    }
    Ok(())
}

fn attach_record_details(
    connection: &mut SqliteConnection,
    records: &mut [AtlasRecord],
) -> Result<(), RecordLoadError> {
    let metrics = metrics::read_metrics(connection)?;
    let actor_data = mechanics::read_actor_mechanics(connection)?;
    let item_data = mechanics::read_item_mechanics(connection)?;
    let supplemental_content = content::read_record_content(connection)?;

    for record in records {
        let key = record.identity.key.to_string();
        if !matches!(
            record.classification.kind,
            RecordKind::Creature | RecordKind::Hazard | RecordKind::Spell
        ) {
            if let Some(rows) = metrics.get(&key) {
                record.mechanics.metrics.clone_from(rows);
            }
            record.mechanics.document = document_mechanics_for_key(&key, &actor_data, &item_data);
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
    let supplemental_content = content::read_record_content_by_keys(connection, keys)?;

    for record in records {
        let key = record.identity.key.to_string();
        if !matches!(
            record.classification.kind,
            RecordKind::Creature | RecordKind::Hazard | RecordKind::Spell
        ) {
            if let Some(rows) = metrics.get(&key) {
                record.mechanics.metrics.clone_from(rows);
            }
            record.mechanics.document = document_mechanics_for_key(&key, &actor_data, &item_data);
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
) -> FoundryDocumentMechanics {
    if let Some(actor) = actor_data.get(key).cloned() {
        return FoundryDocumentMechanics::Actor(actor);
    }
    if let Some(item) = item_data.get(key).cloned() {
        return FoundryDocumentMechanics::Item(item);
    }
    FoundryDocumentMechanics::None
}
