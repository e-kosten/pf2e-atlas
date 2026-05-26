use std::collections::BTreeSet;

use atlas_domain::SearchFilterNode;
use atlas_index::{
    FilteredRecordSort, RecordLoadOptions, RecordResolutionMatchKind, RecordResolutionResult,
};
use atlas_record::{PersistedRecord, RecordAlias};

use crate::{AtlasRetrievalService, SearchError, normalize_record_query};

impl AtlasRetrievalService {
    pub fn resolve_record(
        &self,
        query: &str,
        filter: Option<&SearchFilterNode>,
    ) -> Result<Vec<RecordResolutionResult>, SearchError> {
        self.resolve_record_with_options(query, filter, RecordLoadOptions::include_raw_json())
    }

    pub fn resolve_record_with_options(
        &self,
        query: &str,
        filter: Option<&SearchFilterNode>,
        options: RecordLoadOptions,
    ) -> Result<Vec<RecordResolutionResult>, SearchError> {
        let resolved_filter = self.index.resolve_metric_filters(filter)?;
        let filter = resolved_filter.as_ref().or(filter);
        let normalized_query = normalize_record_query(query);
        if let Some(mut matches) = self.index.resolve_record_matches_with_options(
            query,
            &normalized_query,
            filter,
            options,
        )? {
            dedupe_resolution_matches_by_record_key(&mut matches);
            self.enrich_resolution_reference_labels(&mut matches)?;
            return Ok(matches);
        }
        let mut record_set = self.index.load_record_set()?;
        record_set
            .records
            .retain(|record| record.is_default_visible);
        let default_visible_keys = record_set
            .records
            .iter()
            .map(|record| record.key.clone())
            .collect::<BTreeSet<_>>();
        record_set
            .aliases
            .retain(|alias| default_visible_keys.contains(&alias.canonical_record_key));
        if let Some(filter) = filter {
            let allowed = self
                .index
                .list_filtered_record_keys(
                    Some(filter),
                    FilteredRecordSort::RecordKey,
                    u32::MAX,
                    0,
                )?
                .record_keys
                .into_iter()
                .collect::<BTreeSet<_>>();
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

        dedupe_resolution_matches_by_record_key(&mut matches);
        self.enrich_resolution_reference_labels(&mut matches)?;
        Ok(matches)
    }
}

fn dedupe_resolution_matches_by_record_key(matches: &mut Vec<RecordResolutionResult>) {
    let mut seen = BTreeSet::new();
    matches.retain(|resolution| seen.insert(resolution.record.key.clone()));
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

#[cfg(test)]
mod tests {
    use super::*;
    use atlas_domain::{PackName, PublicationFamily, RecordFamily, RecordId, RecordKey};

    #[test]
    fn dedupe_resolution_matches_collapses_duplicate_record_keys() {
        let mut matches = vec![
            resolution_for_test("actions:reactive", "Reactive Strike"),
            resolution_for_test("actions:reactive", "Reactive Strike"),
            resolution_for_test("actions:other", "Other"),
        ];

        dedupe_resolution_matches_by_record_key(&mut matches);

        assert_eq!(
            matches
                .iter()
                .map(|resolution| resolution.record.key.to_string())
                .collect::<Vec<_>>(),
            vec!["actions:reactive", "actions:other"]
        );
    }

    fn resolution_for_test(key: &str, name: &str) -> RecordResolutionResult {
        RecordResolutionResult {
            query: "Attack of Opportunity".to_string(),
            normalized_query: "attack of opportunity".to_string(),
            match_kind: RecordResolutionMatchKind::Alias,
            matched_text: "Attack of Opportunity".to_string(),
            alias_source: Some("test".to_string()),
            alias_source_ref: Some("test".to_string()),
            record: record(key, name),
        }
    }

    fn record(key: &str, name: &str) -> PersistedRecord {
        PersistedRecord {
            key: RecordKey::parse(key).expect("record key should parse"),
            id: RecordId::new("test").expect("record id should parse"),
            name: name.to_string(),
            normalized_name: normalize_record_query(name),
            record_family: RecordFamily::Rule,
            pack_name: PackName::new("actions").expect("pack name should parse"),
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
            source_path: "test.json".to_string(),
            is_default_visible: true,
            raw_json: "{}".to_string(),
        }
    }
}
