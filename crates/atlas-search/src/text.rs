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
    pub(crate) fn uses_vector(self) -> bool {
        matches!(self, Self::Vector | Self::Hybrid)
    }
}
pub struct TextSearchRequest<'a> {
    pub query: &'a str,
    pub mode: RetrievalMode,
    pub filter: Option<&'a QueryPredicate>,
    pub scope: RecordScope<'a>,
    pub page: SearchPage,
    pub prefer_remaster: bool,
}
impl<'a> TextSearchRequest<'a> {
    pub fn new(query: &'a str, page: SearchPage) -> Self {
        Self {
            query,
            mode: RetrievalMode::Hybrid,
            filter: None,
            scope: RecordScope::All,
            page,
            prefer_remaster: true,
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
    pub snippet: Option<String>,
    pub lexical_rank: Option<f64>,
    pub semantic_similarity: Option<f64>,
}
#[derive(Debug, Clone)]
pub struct TextSearchRecord {
    pub record: SourceRecordSummary,
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
    pub records: Vec<TextSearchRecord>,
    pub page: SearchPageInfo,
    pub coverage: SearchCoverage,
}
use crate::{AtlasRetrievalService, SearchError};
use atlas_domain::{RecordKey, SourcePassageAddress};
use atlas_index::{SourceLexicalHit, SourceVectorHit, ValidatedQuery};
use std::collections::{BTreeMap, BTreeSet};
const SEMANTIC_WINDOWS: [usize; 3] = [1024, 2048, 4096];
const TARGET_ROOTS: usize = 200;
const RRF_CONSTANT: f64 = 60.0;
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
    pub text: &'a str,
    pub mode: RetrievalMode,
    pub query: &'a ValidatedQuery,
    pub scope: RecordScope<'a>,
    pub prefer: bool,
    pub suppress: bool,
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
        snippet: None,
        lexical_rank: Some(hit.rank),
        semantic_similarity: None,
    }
}
fn semantic_witness(hit: SourceVectorHit) -> SearchWitness {
    SearchWitness {
        label: label(&hit.location),
        location: hit.location,
        lane: SearchLane::Semantic,
        snippet: None,
        lexical_rank: None,
        semantic_similarity: Some(1.0 - hit.distance),
    }
}
pub(crate) fn semantic_roots(hits: Vec<SourceVectorHit>) -> Vec<RankedRoot> {
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
                witnesses: hits.into_iter().take(3).map(semantic_witness).collect(),
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
        if request.query.trim().is_empty() {
            return Err(SearchError::invalid_search_options(
                "text search requires nonempty text; use list_records for browse",
            ));
        }
        let predicate = self.predicate(request.filter)?;
        let vector = if request.mode.uses_vector() {
            Some(
                self.embedder
                    .as_mut()
                    .ok_or_else(|| {
                        SearchError::vector_readiness_required(
                            "query embeddings are unavailable in this runtime mode",
                        )
                    })?
                    .embed_query(request.query)
                    .map_err(|e| SearchError::embedding(e.to_string()))?,
            )
        } else {
            None
        };
        let (roots, window) = self.rank_candidates(CandidateRequest {
            text: request.query,
            mode: request.mode,
            query: &predicate,
            scope: request.scope,
            prefer: request.prefer_remaster,
            suppress: true,
            vector: vector.as_deref(),
            exclude: None,
        })?;
        self.finish_results(roots, request.page, request.mode.uses_vector(), window)
    }
    pub(crate) fn suppress_pairs(
        &self,
        mut roots: Vec<RankedRoot>,
        prefer: bool,
    ) -> Result<Vec<RankedRoot>, SearchError> {
        if prefer {
            let keys = roots.iter().map(|r| r.key.clone()).collect::<Vec<_>>();
            let pairs = self.index.matched_remaster_pairs(&keys)?;
            let removed = pairs.into_iter().map(|p| p.legacy).collect::<BTreeSet<_>>();
            roots.retain(|r| !removed.contains(&r.key));
        }
        Ok(roots)
    }
    pub(crate) fn vector_window(
        &self,
        vector: &[f32],
        query: &ValidatedQuery,
        scope: RecordScope<'_>,
        prefer: bool,
        exclude: Option<&RecordKey>,
    ) -> Result<(Vec<RankedRoot>, usize), SearchError> {
        for window in SEMANTIC_WINDOWS {
            let hits = if let Some(seed) = exclude {
                self.index
                    .vector_candidates_excluding(vector, query, scope.keys(), seed, window)?
            } else {
                self.index
                    .vector_candidates(vector, query, scope.keys(), window)?
            };
            let roots = semantic_roots(hits);
            let displayed = self.suppress_pairs(roots.clone(), prefer)?;
            if displayed.len() >= TARGET_ROOTS || window == 4096 {
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
            text,
            mode,
            query,
            scope,
            prefer,
            suppress,
            vector,
            exclude,
        } = request;
        let mut roots = BTreeMap::<RecordKey, RankedRoot>::new();
        if mode.uses_fts() {
            let identity_text = text
                .trim()
                .strip_prefix('"')
                .and_then(|s| s.strip_suffix('"'))
                .unwrap_or(text.trim());
            let identities = self.index.lookup_name_or_alias(identity_text)?;
            let mut lexical = self
                .index
                .lexical_root_candidates(text, query, scope.keys())?;
            let tier = |key: &RecordKey, witnesses: &[SourceLexicalHit]| {
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
                } else if witnesses.iter().any(|h| h.exact_label) {
                    2
                } else {
                    3
                }
            };
            lexical.sort_by(|a, b| {
                tier(&a.record, &a.witnesses)
                    .cmp(&tier(&b.record, &b.witnesses))
                    .then(a.best_rank.total_cmp(&b.best_rank))
                    .then(a.record.cmp(&b.record))
            });
            for (i, hit) in lexical.into_iter().enumerate() {
                let t = tier(&hit.record, &hit.witnesses);
                let rank = i + 1;
                roots.insert(
                    hit.record.clone(),
                    RankedRoot {
                        key: hit.record,
                        lexical_rank: Some(rank),
                        similarity: None,
                        score: 1.0 / (RRF_CONSTANT + rank as f64),
                        tier: t,
                        witnesses: hit.witnesses.into_iter().map(lexical_witness).collect(),
                    },
                );
            }
        }
        let mut window = None;
        if mode.uses_vector() {
            let vector = vector.ok_or_else(|| {
                SearchError::vector_readiness_required(
                    "semantic retrieval requires a compatible vector",
                )
            })?;
            let (semantic, k) = self.vector_window(vector, query, scope, prefer, exclude)?;
            window = Some(k);
            for (i, semantic) in semantic.into_iter().enumerate() {
                let rank = i + 1;
                let semantic_score = if mode == RetrievalMode::Hybrid {
                    1.0 / (RRF_CONSTANT + rank as f64)
                } else {
                    semantic.score
                };
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
        let roots = self.suppress_pairs(roots, prefer && suppress)?;
        Ok((roots, window))
    }
    pub(crate) fn finish_results(
        &self,
        roots: Vec<RankedRoot>,
        page: SearchPage,
        bounded: bool,
        window: Option<usize>,
    ) -> Result<TextSearchResult, SearchError> {
        let total = roots.len();
        let selected = roots
            .into_iter()
            .skip(page.offset()? as usize)
            .take(page.size() as usize)
            .collect::<Vec<_>>();
        let keys = selected.iter().map(|r| r.key.clone()).collect::<Vec<_>>();
        let summaries = self.summaries(&keys)?;
        let mut locators = selected
            .iter()
            .flat_map(|r| &r.witnesses)
            .filter(|m| {
                matches!(
                    m.location.address,
                    Some(SourcePassageAddress::HtmlSection { .. })
                )
            })
            .filter_map(|m| {
                m.location.field.as_ref().map(|f| {
                    atlas_record::source_content::SourceContentLocator {
                        record: m.location.record.clone(),
                        owners: m.location.owners.clone(),
                        field: f.clone(),
                    }
                })
            })
            .collect::<Vec<_>>();
        let mut unique = Vec::with_capacity(locators.len());
        locators.retain(|locator| {
            if unique.contains(locator) {
                false
            } else {
                unique.push(locator.clone());
                true
            }
        });
        let mut prepared = Vec::new();
        for chunk in locators.chunks(256) {
            for field in self.index.read_content(chunk)?.fields {
                prepared.push(field);
            }
        }
        let mut records = Vec::new();
        for (root, record) in selected.into_iter().zip(summaries) {
            let mut matches = root.witnesses;
            let needs_body = matches.iter().any(|m| {
                matches!(
                    m.location.address,
                    Some(SourcePassageAddress::PlainSection { .. })
                )
            });
            let source = if needs_body {
                self.index.read_source_record(&root.key)?
            } else {
                None
            };
            for witness in &mut matches {
                if witness.label.is_none() && witness.location.owners.is_empty() {
                    witness.label = record.name.as_value().cloned();
                }
                if let (Some(address), Some(field)) =
                    (&witness.location.address, &witness.location.field)
                {
                    let locator = atlas_record::source_content::SourceContentLocator {
                        record: root.key.clone(),
                        owners: witness.location.owners.clone(),
                        field: field.clone(),
                    };
                    let html = prepared
                        .iter()
                        .find(|h| h.locator == locator)
                        .and_then(|h| h.html.as_deref());
                    let plain = source
                        .as_ref()
                        .and_then(|s| s.authored_content_at(&locator).value());
                    witness.snippet = Some(
                        atlas_record::source_record::recover_source_passage(html, plain, address)
                            .map_err(|e| SearchError::artifact_contract_violation(e.to_string()))?
                            .chars()
                            .take(240)
                            .collect(),
                    );
                }
            }
            records.push(TextSearchRecord {
                record,
                matches,
                lexical_rank: root.lexical_rank,
                semantic_similarity: root.similarity,
                score: root.score,
            });
        }
        let info = SearchPageInfo::from_page(page, records.len(), total as u64)?;
        Ok(TextSearchResult {
            records,
            page: info,
            coverage: SearchCoverage {
                exhaustive: !bounded,
                semantic_unit_window: window,
                candidate_roots: total,
                count_basis: if bounded {
                    "accepted bounded root candidates"
                } else {
                    "complete eligible lexical roots"
                }
                .to_owned(),
            },
        })
    }
}
