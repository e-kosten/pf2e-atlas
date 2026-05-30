use atlas_domain::SearchFilterNode;
use atlas_index::FilteredRecordSort;
use atlas_record::{PersistedRecord, RecordAlias};
use serde::{Deserialize, Serialize};

use crate::query::normalize_record_query;
use crate::{AtlasRetrievalService, SearchError};

#[derive(Debug, Clone, PartialEq)]
pub struct RecordResolutionResult {
    pub query: String,
    pub normalized_query: String,
    pub match_kind: RecordResolutionMatchKind,
    pub matched_text: String,
    pub alias_source: Option<String>,
    pub alias_source_ref: Option<String>,
    pub record: PersistedRecord,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecordResolutionMatchKind {
    Name,
    NormalizedName,
    Alias,
    VariantName,
}

impl RecordResolutionMatchKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Name => "name",
            Self::NormalizedName => "normalized_name",
            Self::Alias => "alias",
            Self::VariantName => "variant_name",
        }
    }
}

impl AtlasRetrievalService {
    pub fn resolve_record(
        &self,
        query: &str,
        filter: Option<&SearchFilterNode>,
    ) -> Result<Vec<RecordResolutionResult>, SearchError> {
        let resolved_filter = self.index.resolve_metric_filters(filter)?;
        let filter = resolved_filter.as_ref().or(filter);
        let normalized_query = normalize_record_query(query);
        let mut record_set = self.index.load_record_set()?;
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
