use atlas_domain::SearchFilterNode;
use atlas_index::{
    FilterCompileError, FilteredRecordSort, RecordIdentityMatch, RecordIdentityMatchKind,
    RecordLoadError,
};
use atlas_record::{PersistedRecord, RecordAlias};

use crate::query::normalize_record_query;
use crate::{AtlasRetrievalService, SearchError};

use super::{RecordResolutionMatchKind, RecordResolutionResult};

pub(crate) fn resolve_record(
    service: &AtlasRetrievalService,
    query: &str,
    filter: Option<&SearchFilterNode>,
) -> Result<Vec<RecordResolutionResult>, SearchError> {
    let resolved_filter = service
        .index
        .resolve_metric_filters(filter)
        .map_err(SearchError::from_filter)?;
    let filter = resolved_filter.as_ref().or(filter);
    let normalized_query = normalize_record_query(query);
    if let Some(mut matches) =
        service.resolve_record_with_index(query, &normalized_query, filter)?
    {
        service.enrich_resolution_reference_labels(&mut matches)?;
        return Ok(matches);
    }

    let mut record_set = service
        .index
        .load_record_set()
        .map_err(SearchError::from_record_load)?;
    record_set
        .records
        .retain(|record| record.is_default_visible);
    let default_visible_keys = record_set
        .records
        .iter()
        .map(|record| record.key.clone())
        .collect::<std::collections::BTreeSet<_>>();
    record_set
        .aliases
        .retain(|alias| default_visible_keys.contains(&alias.canonical_record_key));
    if let Some(filter) = filter {
        let allowed = service
            .index
            .list_filtered_record_keys(Some(filter), FilteredRecordSort::RecordKey, u32::MAX, 0)
            .map_err(SearchError::from_filter)?
            .record_keys
            .into_iter()
            .collect::<std::collections::BTreeSet<_>>();
        record_set
            .records
            .retain(|record| allowed.contains(&record.key));
        record_set
            .aliases
            .retain(|alias| allowed.contains(&alias.canonical_record_key));
    }

    let mut matches = resolution_matches_for_kind(
        query,
        &normalized_query,
        RecordResolutionMatchKind::Name,
        &record_set.records,
        &record_set.aliases,
    );
    if matches.is_empty() {
        matches = resolution_matches_for_kind(
            query,
            &normalized_query,
            RecordResolutionMatchKind::NormalizedName,
            &record_set.records,
            &record_set.aliases,
        );
    }
    if matches.is_empty() {
        matches = resolution_matches_for_kind(
            query,
            &normalized_query,
            RecordResolutionMatchKind::Alias,
            &record_set.records,
            &record_set.aliases,
        );
    }
    if matches.is_empty() {
        matches = resolution_matches_for_kind(
            query,
            &normalized_query,
            RecordResolutionMatchKind::VariantName,
            &record_set.records,
            &record_set.aliases,
        );
    }

    service.enrich_resolution_reference_labels(&mut matches)?;
    Ok(matches)
}

impl AtlasRetrievalService {
    fn resolve_record_with_index(
        &self,
        query: &str,
        normalized_query: &str,
        filter: Option<&SearchFilterNode>,
    ) -> Result<Option<Vec<RecordResolutionResult>>, SearchError> {
        let identity_matches =
            match self
                .index
                .resolve_record_identity_matches(query, normalized_query, filter)
            {
                Ok(Some(matches)) => matches,
                Ok(None) => return Ok(None),
                Err(FilterCompileError::QueryFailed(message)) => {
                    return Err(SearchError::from_record_load(RecordLoadError::QueryFailed(
                        message,
                    )));
                }
                Err(FilterCompileError::InvalidValue(message)) => {
                    return Err(SearchError::from_record_load(RecordLoadError::InvalidData(
                        message,
                    )));
                }
                Err(error) => return Err(SearchError::from_filter(error)),
            };
        if identity_matches.is_empty() {
            return Ok(Some(Vec::new()));
        }

        let record_keys = identity_matches
            .iter()
            .map(|identity| identity.record_key.clone())
            .collect::<Vec<_>>();
        let records = self
            .index
            .load_records_by_key(&record_keys)
            .map_err(SearchError::from_record_load)?
            .into_iter()
            .map(|record| (record.key.clone(), record))
            .collect::<std::collections::BTreeMap<_, _>>();
        let mut matches = identity_matches
            .into_iter()
            .filter_map(|identity| {
                let record = records.get(&identity.record_key)?;
                Some(resolution_result_from_identity(
                    query,
                    normalized_query,
                    identity,
                    record,
                ))
            })
            .collect::<Vec<_>>();
        matches.sort_by(|left, right| left.record.key.cmp(&right.record.key));
        matches.dedup_by(|left, right| left.record.key == right.record.key);
        Ok(Some(matches))
    }
}

fn resolution_matches_for_kind(
    query: &str,
    normalized_query: &str,
    kind: RecordResolutionMatchKind,
    records: &[PersistedRecord],
    aliases: &[RecordAlias],
) -> Vec<RecordResolutionResult> {
    let mut matches = Vec::new();
    match kind {
        RecordResolutionMatchKind::Name => {
            matches.extend(
                records
                    .iter()
                    .filter(|record| record.name == query)
                    .map(|record| {
                        resolution_result(
                            query,
                            normalized_query,
                            kind,
                            record.name.clone(),
                            None,
                            record,
                        )
                    }),
            );
        }
        RecordResolutionMatchKind::NormalizedName => {
            matches.extend(
                records
                    .iter()
                    .filter(|record| {
                        record.variant_label.is_none() && record.normalized_name == normalized_query
                    })
                    .map(|record| {
                        resolution_result(
                            query,
                            normalized_query,
                            kind,
                            record.normalized_name.clone(),
                            None,
                            record,
                        )
                    }),
            );
        }
        RecordResolutionMatchKind::Alias => {
            for alias in aliases
                .iter()
                .filter(|alias| alias.normalized_alias == normalized_query)
            {
                matches.extend(
                    records
                        .iter()
                        .filter(|record| record.key == alias.canonical_record_key)
                        .map(|record| {
                            resolution_result(
                                query,
                                normalized_query,
                                kind,
                                alias.alias_text.clone(),
                                Some(alias),
                                record,
                            )
                        }),
                );
            }
        }
        RecordResolutionMatchKind::VariantName => {
            matches.extend(
                records
                    .iter()
                    .filter(|record| {
                        record.variant_label.is_some() && record.normalized_name == normalized_query
                    })
                    .map(|record| {
                        resolution_result(
                            query,
                            normalized_query,
                            kind,
                            record.normalized_name.clone(),
                            None,
                            record,
                        )
                    }),
            );
        }
    }
    matches.sort_by(|left, right| left.record.key.cmp(&right.record.key));
    matches.dedup_by(|left, right| left.record.key == right.record.key);
    matches
}

fn resolution_result(
    query: &str,
    normalized_query: &str,
    match_kind: RecordResolutionMatchKind,
    matched_text: String,
    alias: Option<&RecordAlias>,
    record: &PersistedRecord,
) -> RecordResolutionResult {
    RecordResolutionResult {
        query: query.to_string(),
        normalized_query: normalized_query.to_string(),
        match_kind,
        matched_text,
        alias_source: alias.map(|alias| alias.source.as_str().to_string()),
        alias_source_ref: alias.map(|alias| alias.source_ref.clone()),
        record: record.clone(),
    }
}

fn resolution_result_from_identity(
    query: &str,
    normalized_query: &str,
    identity: RecordIdentityMatch,
    record: &PersistedRecord,
) -> RecordResolutionResult {
    RecordResolutionResult {
        query: query.to_string(),
        normalized_query: normalized_query.to_string(),
        match_kind: match identity.match_kind {
            RecordIdentityMatchKind::Name => RecordResolutionMatchKind::Name,
            RecordIdentityMatchKind::NormalizedName => RecordResolutionMatchKind::NormalizedName,
            RecordIdentityMatchKind::Alias => RecordResolutionMatchKind::Alias,
            RecordIdentityMatchKind::VariantName => RecordResolutionMatchKind::VariantName,
        },
        matched_text: identity.matched_text,
        alias_source: identity.alias_source,
        alias_source_ref: identity.alias_source_ref,
        record: record.clone(),
    }
}

#[cfg(test)]
mod tests {
    use atlas_domain::{PackName, PublicationFamily, RecordFamily, RecordKey};
    use atlas_record::{AliasSource, PersistedRecord, RecordAlias};

    use super::*;

    #[test]
    fn alias_resolution_returns_one_match_per_record_key() {
        let record = fake_record("actions:KAVf7AmRnbCAHrkT", "Reactive Strike");
        let aliases = vec![
            fake_alias(&record.key, "Attack of Opportunity", "legacy"),
            fake_alias(&record.key, "Attack of Opportunity", "alternate"),
        ];

        let matches = resolution_matches_for_kind(
            "Attack of Opportunity",
            "attack of opportunity",
            RecordResolutionMatchKind::Alias,
            std::slice::from_ref(&record),
            &aliases,
        );

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].record.key, record.key);
    }

    fn fake_alias(record_key: &RecordKey, alias_text: &str, source_ref: &str) -> RecordAlias {
        RecordAlias {
            canonical_record_key: record_key.clone(),
            alias_text: alias_text.to_string(),
            normalized_alias: alias_text.to_lowercase(),
            source: AliasSource::CompendiumSource,
            source_ref: source_ref.to_string(),
        }
    }

    fn fake_record(key: &str, name: &str) -> PersistedRecord {
        let key = RecordKey::parse(key).expect("fixture key should parse");
        PersistedRecord {
            id: key.id().clone(),
            key,
            name: name.to_string(),
            normalized_name: name.to_lowercase(),
            record_family: RecordFamily::Rule,
            pack_name: PackName::new("actions").expect("fixture pack should parse"),
            pack_label: "Actions".to_string(),
            foundry_document_type: "Item".to_string(),
            foundry_record_type: "action".to_string(),
            level: None,
            rarity: None,
            traits: Vec::new(),
            prerequisites: Vec::new(),
            system_category: None,
            system_group: None,
            system_base_item: None,
            system_usage: None,
            system_price_json: None,
            system_actions_value: None,
            system_time_value: None,
            system_duration_value: None,
            price_cp: None,
            activation_time: None,
            duration: None,
            metrics: Vec::new(),
            actor_data: None,
            item_data: None,
            spell_data: None,
            publication_title: None,
            publication_remaster: false,
            description: None,
            blurb: None,
            supplemental_content: Vec::new(),
            publication_family: PublicationFamily::Unknown,
            folder_id: None,
            taxonomy_families: Vec::new(),
            variant_group_key: None,
            variant_base_name: None,
            variant_label: None,
            variant_axes: Vec::new(),
            variant_confidence: None,
            variant_source: "none".to_string(),
            source_path: format!("packs/actions/{name}.json"),
            is_default_visible: true,
            raw_json: "{}".to_string(),
        }
    }
}
