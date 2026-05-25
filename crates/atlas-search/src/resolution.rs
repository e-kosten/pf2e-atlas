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

        self.enrich_resolution_reference_labels(&mut matches)?;
        Ok(matches)
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
