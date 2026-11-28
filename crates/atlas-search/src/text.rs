use crate::{RecordScope, SearchPage, SearchPageInfo};
use atlas_domain::{QueryPredicate, SourceRecordSummary};
use atlas_index::SourceUnitLocation;
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RetrievalMode {
    Fts,
    Vector,
    Hybrid,
}
impl RetrievalMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Fts => "fts",
            Self::Vector => "vector",
            Self::Hybrid => "hybrid",
        }
    }
    pub(crate) fn uses_fts(self) -> bool {
        matches!(self, Self::Fts | Self::Hybrid)
    }
    /// Report lexical input errors before Hybrid starts query inference.
    pub fn validate_input_before_inference(self, text: &str) -> Result<(), crate::SearchError> {
        if self == Self::Hybrid {
            atlas_index::validate_lexical_text(text).map_err(atlas_index::IndexError::from)?;
        }
        Ok(())
    }
    pub fn uses_vector(self) -> bool {
        matches!(self, Self::Vector | Self::Hybrid)
    }
}
pub struct TextSearchRequest<'a> {
    pub lexical: atlas_domain::LexicalOptions,
    pub result_detail: atlas_domain::SearchResultDetail,
    pub content_scope: atlas_domain::SearchContentScope,
    pub sort: atlas_domain::ResultSort,
    pub ranking: atlas_domain::SearchRanking,
    pub search_profile: Option<&'a atlas_domain::SearchProfileId>,
    pub query: &'a str,
    pub mode: RetrievalMode,
    pub filter: Option<&'a QueryPredicate>,
    pub scope: RecordScope<'a>,
    pub page: SearchPage,
}
impl<'a> TextSearchRequest<'a> {
    pub fn new(query: &'a str, page: SearchPage) -> Self {
        Self {
            lexical: Default::default(),
            result_detail: atlas_domain::SearchResultDetail::Evidence,
            sort: atlas_domain::ResultSort::Relevance {},
            ranking: Default::default(),
            search_profile: None,
            query,
            content_scope: Default::default(),
            mode: RetrievalMode::Hybrid,
            filter: None,
            scope: RecordScope::All,
            page,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SearchLane {
    Lexical,
    Semantic,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SearchWitness {
    pub location: SourceUnitLocation,
    pub lane: SearchLane,
    pub label: Option<String>,
    pub preview: Option<atlas_record::source_record::SourceMatchPreview>,
    #[serde(skip)]
    pub parent_heading: Option<String>,
    pub lexical_rank: Option<f64>,
    pub semantic_similarity: Option<f64>,
}
#[derive(Debug, Clone)]
pub struct TextSearchRecord {
    pub record: SourceRecordSummary,
    pub exact_journal_pages: Option<atlas_index::ExactJournalPages>,
    pub matches: Vec<SearchWitness>,
    pub lexical_rank: Option<usize>,
    pub semantic_similarity: Option<f64>,
    pub score: f64,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SearchCoverage {
    pub exhaustive: bool,
    pub semantic_unit_window: Option<usize>,
    pub candidate_roots: usize,
    pub count_basis: String,
}
#[derive(Debug, Clone)]
pub struct TextSearchResult {
    pub sort: atlas_domain::ResultSort,
    pub search_profile: atlas_domain::SearchProfileId,
    pub artifact_generation: String,
    pub records: Vec<TextSearchRecord>,
    pub page: SearchPageInfo,
    pub coverage: SearchCoverage,
}
use crate::{AtlasRetrievalService, SearchError};
use atlas_domain::{RecordKey, SourcePassageAddress};
use atlas_index::{SourceLexicalHit, SourceVectorHit, ValidatedQuery};
use std::collections::BTreeMap;
const SEMANTIC_WINDOWS: [usize; 3] = [1024, 2048, 4096];
const TARGET_ROOTS: usize = 200;

pub(crate) fn identity_search_text(text: &str) -> &str {
    let text = text.trim();
    text.strip_prefix('"')
        .and_then(|inner| inner.strip_suffix('"'))
        .unwrap_or(text)
}
#[derive(Debug, Clone)]
pub(crate) struct RankedRoot {
    pub key: RecordKey,
    pub lexical_rank: Option<usize>,
    pub similarity: Option<f64>,
    pub score: f64,
    pub tier: u8,
    pub witnesses: Vec<SearchWitness>,
}
pub(crate) struct CandidateRequest<'a> {
    pub result_detail: atlas_domain::SearchResultDetail,
    pub lexical: atlas_domain::LexicalOptions,
    pub content_scope: atlas_domain::SearchContentScope,
    pub ranking: atlas_domain::SearchRanking,
    pub search_profile: &'a atlas_domain::SearchProfileId,
    pub text: &'a str,
    pub mode: RetrievalMode,
    pub query: &'a ValidatedQuery,
    pub scope: RecordScope<'a>,

    pub vector: Option<&'a [f32]>,
    pub exclude: Option<&'a RecordKey>,
}
pub(crate) fn label(location: &SourceUnitLocation) -> Option<String> {
    match &location.address {
        Some(
            SourcePassageAddress::HtmlSection { label, .. }
            | SourcePassageAddress::PlainSection { label, .. },
        ) => label.clone(),
        _ => None,
    }
}
fn lexical_witness(hit: SourceLexicalHit) -> SearchWitness {
    SearchWitness {
        label: hit.label.clone(),
        location: hit.location,
        lane: SearchLane::Lexical,
        preview: None,
        parent_heading: None,
        lexical_rank: Some(hit.rank),
        semantic_similarity: None,
    }
}
fn semantic_witness(hit: SourceVectorHit) -> SearchWitness {
    SearchWitness {
        label: label(&hit.location),
        location: hit.location,
        lane: SearchLane::Semantic,
        preview: None,
        parent_heading: None,
        lexical_rank: None,
        semantic_similarity: Some(1.0 - hit.distance),
    }
}
pub(crate) fn semantic_roots(
    hits: Vec<SourceVectorHit>,
    include_evidence: bool,
) -> Vec<RankedRoot> {
    let mut grouped = BTreeMap::<RecordKey, Vec<SourceVectorHit>>::new();
    for hit in hits {
        grouped
            .entry(hit.location.record.clone())
            .or_default()
            .push(hit);
    }
    let mut roots = grouped
        .into_iter()
        .map(|(key, mut hits)| {
            hits.sort_by(|a, b| {
                a.distance
                    .total_cmp(&b.distance)
                    .then(a.location.unit_id.cmp(&b.location.unit_id))
            });
            let similarity = 1.0 - hits[0].distance;
            RankedRoot {
                key,
                lexical_rank: None,
                similarity: Some(similarity),
                score: similarity,
                tier: 3,
                witnesses: if include_evidence {
                    hits.into_iter().take(3).map(semantic_witness).collect()
                } else {
                    Vec::new()
                },
            }
        })
        .collect::<Vec<_>>();
    roots.sort_by(|a, b| b.score.total_cmp(&a.score).then(a.key.cmp(&b.key)));
    roots
}
impl AtlasRetrievalService {
    pub fn search_text(
        &mut self,
        request: TextSearchRequest<'_>,
    ) -> Result<TextSearchResult, SearchError> {
        Self::validate_search_sort(&request.sort)?;
        if request.query.trim().is_empty() {
            return Err(SearchError::invalid_search_options(
                "text search requires nonempty text; use list_records for browse",
            ));
        }
        let search_profile =
            self.resolve_search_profile(request.search_profile, request.mode.uses_vector())?;
        let predicate = self.predicate(request.filter)?;
        let vector = if request.mode.uses_vector() {
            Some(self.embed_query(&search_profile, request.query)?)
        } else {
            None
        };
        let (roots, window) = self.rank_candidates(CandidateRequest {
            result_detail: request.result_detail,
            lexical: request.lexical,
            content_scope: request.content_scope,
            ranking: request.ranking,
            search_profile: &search_profile,
            text: request.query,
            mode: request.mode,
            query: &predicate,
            scope: request.scope,

            vector: vector.as_deref(),
            exclude: None,
        })?;
        let mut result = self.finish_results(
            &search_profile,
            roots,
            request.page,
            window,
            request.sort,
            request.result_detail,
        )?;
        let journals = result
            .records
            .iter()
            .filter(|row| row.record.document_kind == "JournalEntry")
            .map(|row| &row.record.key)
            .collect::<Vec<_>>();
        let mut pages =
            self.index
                .exact_journal_pages(&journals, request.query, request.content_scope)?;
        for row in &mut result.records {
            row.exact_journal_pages = pages.remove(&row.record.key);
        }
        Ok(result)
    }
    fn native_vector_candidates(
        &self,
        request: atlas_index::SourceVectorCandidateRequest<'_>,
        text: &str,
    ) -> Result<Vec<SourceVectorHit>, SearchError> {
        let retained = self.neighbor_reuse.as_ref();
        let Some((store, epoch)) = retained else {
            return Ok(self.index.vector_candidates(request)?);
        };
        let Some(key) = store.key(*epoch, &self.index, text, &request) else {
            return Ok(self.index.vector_candidates(request)?);
        };
        if let Some(found) = store.get(&key) {
            return self
                .index
                .hydrate_vector_neighbors(request, &found.neighbors)
                .map_err(|error| {
                    store.remove(&key);
                    error.into()
                });
        }
        #[cfg(test)]
        store.miss_checkpoint();
        let (hits, neighbors) = self.index.vector_candidates_with_neighbors(request)?;
        store.insert(key, neighbors);
        Ok(hits)
    }
    pub(crate) fn vector_window(
        &self,
        request: &CandidateRequest<'_>,
        vector: &[f32],
    ) -> Result<(Vec<RankedRoot>, usize), SearchError> {
        for window in SEMANTIC_WINDOWS {
            let input = atlas_index::SourceVectorCandidateRequest {
                profile: request.search_profile,
                vector,
                query: request.query,
                keys: request.scope.keys(),
                exclude: request.exclude,
                content_scope: request.content_scope,
                limit: window,
            };
            let hits = self.native_vector_candidates(input, request.text)?;
            let exhausted = hits.len() < window;
            let roots = semantic_roots(hits, request.result_detail.includes_evidence());
            if exhausted || roots.len() >= TARGET_ROOTS || window == 4096 {
                return Ok((roots, window));
            }
        }
        Err(SearchError::query_failed(
            "semantic candidate policy has no accepted window",
        ))
    }
    pub(crate) fn rank_candidates(
        &self,
        request: CandidateRequest<'_>,
    ) -> Result<(Vec<RankedRoot>, Option<usize>), SearchError> {
        let CandidateRequest {
            lexical: options,
            ranking,
            content_scope,
            text,
            mode,
            query,
            scope,

            vector,
            ..
        } = request;
        let mut roots = BTreeMap::<RecordKey, RankedRoot>::new();
        if mode.uses_fts() {
            let identity_text = identity_search_text(text);
            let mut reviewed_labels = self.reviewed_creature_label_matches(identity_text)?;
            reviewed_labels.retain(|matched| {
                request.exclude != Some(&matched.key)
                    && scope.keys().is_none_or(|keys| keys.contains(&matched.key))
            });
            let mut identities = self.index.lookup_name_or_alias(identity_text)?;
            identities.extend(reviewed_labels.iter().cloned());
            let mut lexical = self.index.lexical_root_candidates(
                text,
                query,
                scope.keys(),
                content_scope,
                options,
                request.result_detail.includes_evidence(),
            )?;
            let tier = |key: &RecordKey, name_starts_with_query: bool| {
                if identities
                    .iter()
                    .any(|m| &m.key == key && m.alias.is_none())
                {
                    0
                } else if identities
                    .iter()
                    .any(|m| &m.key == key && m.alias.is_some())
                {
                    1
                } else if name_starts_with_query {
                    2
                } else {
                    3
                }
            };
            lexical.sort_by(|a, b| {
                tier(&a.record, a.name_starts_with_query)
                    .cmp(&tier(&b.record, b.name_starts_with_query))
                    .then(a.best_rank.total_cmp(&b.best_rank))
                    .then(a.record.cmp(&b.record))
            });
            let scores = crate::fusion::lane_scores(
                &lexical.iter().map(|hit| -hit.best_rank).collect::<Vec<_>>(),
                if mode == RetrievalMode::Hybrid {
                    ranking
                } else {
                    atlas_domain::SearchRanking::Rrf
                },
                0.25,
            );
            for (i, (hit, score)) in lexical.into_iter().zip(scores).enumerate() {
                let t = tier(&hit.record, hit.name_starts_with_query);
                let rank = i + 1;
                roots.insert(
                    hit.record.clone(),
                    RankedRoot {
                        key: hit.record.clone(),
                        lexical_rank: Some(rank),
                        similarity: None,
                        score,
                        tier: t,
                        witnesses: hit.witnesses.into_iter().map(lexical_witness).collect(),
                    },
                );
            }
            let unranked_labels: Vec<_> = reviewed_labels
                .into_iter()
                .map(|matched| matched.key)
                .filter(|key| !roots.contains_key(key))
                .collect();
            if !unranked_labels.is_empty() {
                let eligible = self.index.eligible_keys(
                    query,
                    Some(&unranked_labels),
                    atlas_domain::ResultSort::attribute(
                        atlas_domain::SortField::Name,
                        atlas_domain::SortDirection::Asc,
                    ),
                    0,
                    unranked_labels.len(),
                )?;
                for key in eligible.keys {
                    roots.insert(
                        key.clone(),
                        RankedRoot {
                            tier: tier(&key, false),
                            key,
                            lexical_rank: None,
                            similarity: None,
                            score: 0.0,
                            witnesses: Vec::new(),
                        },
                    );
                }
            }
        }
        let mut window = None;
        if mode.uses_vector() {
            let vector = vector.ok_or_else(|| {
                SearchError::vector_readiness_required(
                    "semantic retrieval requires a compatible vector",
                )
            })?;
            let (semantic, k) = self.vector_window(&request, vector)?;
            window = Some(k);
            let values = semantic.iter().map(|root| root.score).collect::<Vec<_>>();
            let scores = if mode == RetrievalMode::Hybrid {
                crate::fusion::lane_scores(&values, ranking, 0.75)
            } else {
                values
            };
            for (semantic, semantic_score) in semantic.into_iter().zip(scores) {
                roots
                    .entry(semantic.key.clone())
                    .and_modify(|root| {
                        root.similarity = semantic.similarity;
                        root.score += semantic_score;
                        root.witnesses.truncate(2);
                        root.witnesses
                            .extend(semantic.witnesses.iter().take(1).cloned());
                    })
                    .or_insert(RankedRoot {
                        score: semantic_score,
                        ..semantic
                    });
            }
        }
        let mut roots = roots.into_values().collect::<Vec<_>>();
        roots.sort_by(|a, b| {
            a.tier
                .cmp(&b.tier)
                .then(b.score.total_cmp(&a.score))
                .then(a.key.cmp(&b.key))
        });
        Ok((roots, window))
    }
    pub(crate) fn finish_results(
        &self,
        search_profile: &atlas_domain::SearchProfileId,
        roots: Vec<RankedRoot>,
        page: SearchPage,
        window: Option<usize>,
        sort: atlas_domain::ResultSort,
        detail: atlas_domain::SearchResultDetail,
    ) -> Result<TextSearchResult, SearchError> {
        let roots = self.order_ranked_roots(roots, sort.clone())?;
        let total = roots.len();
        let selected = roots
            .into_iter()
            .skip(page.offset()? as usize)
            .take(page.size() as usize)
            .collect::<Vec<_>>();
        let keys = selected.iter().map(|r| r.key.clone()).collect::<Vec<_>>();
        let summaries = self.summaries(&keys)?;
        let mut records = selected
            .into_iter()
            .zip(summaries)
            .map(|(root, record)| TextSearchRecord {
                record,
                exact_journal_pages: None,
                matches: root.witnesses,
                lexical_rank: root.lexical_rank,
                semantic_similarity: root.similarity,
                score: root.score,
            })
            .collect::<Vec<_>>();
        if records.iter().any(|record| !record.matches.is_empty()) {
            self.hydrate_text_matches(&mut records, detail)?;
        }
        let info = SearchPageInfo::from_page(page, records.len(), total as u64)?;
        Ok(TextSearchResult {
            sort,
            search_profile: search_profile.clone(),
            artifact_generation: self.artifact_generation()?,
            records,
            page: info,
            coverage: SearchCoverage {
                exhaustive: window.is_none(),
                semantic_unit_window: window,
                candidate_roots: total,
                count_basis: if window.is_some() {
                    "accepted bounded root candidates"
                } else {
                    "complete eligible lexical roots"
                }
                .to_owned(),
            },
        })
    }
    fn hydrate_text_matches(
        &self,
        records: &mut [TextSearchRecord],
        detail: atlas_domain::SearchResultDetail,
    ) -> Result<(), SearchError> {
        if detail == atlas_domain::SearchResultDetail::Compact {
            self.select_compact_matches(records)?;
        } else {
            let locations = records
                .iter()
                .flat_map(|r| &r.matches)
                .map(|w| w.location.clone())
                .collect::<Vec<_>>();
            let previews = self.index.recover_match_previews(&locations)?;
            for (witness, evidence) in records
                .iter_mut()
                .flat_map(|r| &mut r.matches)
                .zip(previews)
            {
                if let Some(evidence) = evidence {
                    witness.preview = evidence.preview;
                    witness.parent_heading = evidence.parent_heading;
                }
            }
        }
        let owned_locations = records
            .iter()
            .flat_map(|record| &record.matches)
            .filter(|witness| !witness.location.owners.is_empty())
            .map(|witness| witness.location.clone())
            .collect::<Vec<_>>();
        let mut owned_labels = vec![];
        for batch in owned_locations.chunks(256) {
            owned_labels.extend(self.index.owned_search_labels(batch)?);
        }
        for record in records {
            for witness in &mut record.matches {
                if !witness.location.owners.is_empty()
                    && let Some((_, _, owner_label)) =
                        owned_labels.iter().find(|(key, owners, _)| {
                            key == &record.record.key && owners == &witness.location.owners
                        })
                    && !owner_label.trim().is_empty()
                {
                    witness.label = Some(match witness.label.as_deref() {
                        Some(section) if !section.trim().is_empty() && section != owner_label => {
                            format!("{owner_label} — {section}")
                        }
                        _ => owner_label.clone(),
                    });
                }
                if witness.label.is_none() && witness.location.owners.is_empty() {
                    witness.label = record.record.name.as_value().cloned();
                }
            }
        }
        Ok(())
    }
    /// First witness with a displayable preview, falling back to the first witness.
    /// Probe passages in batches; identity/name locations cannot supply excerpts.
    fn select_compact_matches(&self, records: &mut [TextSearchRecord]) -> Result<(), SearchError> {
        let mut positions = vec![0; records.len()];
        loop {
            let mut pending = Vec::new();
            let mut locations = Vec::new();
            for (row, record) in records.iter().enumerate() {
                if let Some((index, witness)) = record
                    .matches
                    .iter()
                    .enumerate()
                    .skip(positions[row])
                    .find(|(_, w)| {
                        matches!(
                            w.location.address,
                            Some(
                                SourcePassageAddress::HtmlSection { .. }
                                    | SourcePassageAddress::PlainSection { .. }
                            )
                        )
                    })
                {
                    pending.push((row, index));
                    locations.push(witness.location.clone());
                    positions[row] = index + 1;
                }
            }
            if pending.is_empty() {
                break;
            }
            for ((row, index), evidence) in pending
                .into_iter()
                .zip(self.index.recover_match_previews(&locations)?)
            {
                let Some(evidence) = evidence else { continue };
                let witness = &mut records[row].matches[index];
                witness.preview = evidence.preview;
                witness.parent_heading = evidence.parent_heading;
                if witness.preview.is_some() {
                    records[row].matches.swap(0, index);
                    positions[row] = records[row].matches.len();
                }
            }
        }
        for record in records {
            record.matches.truncate(1);
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "text_inference_tests.rs"]
mod inference_tests;
