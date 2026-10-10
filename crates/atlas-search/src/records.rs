use crate::{AtlasRetrievalService, SearchError, SearchPage, SearchPageInfo};
use atlas_domain::{QueryPredicate, RecordKey, SourceRecordSummary};
use atlas_index::{SourceContentBundle, SourceSummaryBatch};
use atlas_record::{source_content::SourceContentLocator, source_record::SourceBackedRecord};
#[derive(Debug, Clone, Copy, Default)]
pub enum RecordScope<'a> {
    #[default]
    All,
    Keys(&'a [RecordKey]),
}
impl<'a> RecordScope<'a> {
    pub fn keys(self) -> Option<&'a [RecordKey]> {
        match self {
            Self::All => None,
            Self::Keys(k) => Some(k),
        }
    }
}
pub struct GetRecordRequest<'a> {
    pub record_key: &'a RecordKey,
    pub selected_content: &'a [SourceContentLocator],
}
pub struct GetRecordsRequest<'a> {
    pub record_keys: &'a [RecordKey],
}
pub struct ListRecordsRequest<'a> {
    pub filter: Option<&'a QueryPredicate>,
    pub scope: RecordScope<'a>,
    pub page: SearchPage,
    pub prefer_remaster: bool,
}
impl<'a> ListRecordsRequest<'a> {
    pub fn new(filter: Option<&'a QueryPredicate>, page: SearchPage) -> Self {
        Self {
            filter,
            scope: RecordScope::All,
            page,
            prefer_remaster: true,
        }
    }
    pub fn with_scope(mut self, scope: RecordScope<'a>) -> Self {
        self.scope = scope;
        self
    }
}
#[derive(Debug)]
pub struct SourceRecordDetail {
    pub summary: SourceRecordSummary,
    pub source: SourceBackedRecord,
    pub content: SourceContentBundle,
}
#[derive(Debug, Clone)]
pub struct ListRecordsResult {
    pub record_keys: Vec<RecordKey>,
    pub records: Vec<SourceRecordSummary>,
    pub total: u64,
    pub page: SearchPageInfo,
}
pub struct ResolveRecordRequest<'a> {
    pub query: &'a str,
    pub filter: Option<&'a QueryPredicate>,
}
pub struct ResolveRecordRefRequest<'a> {
    pub record_ref: &'a str,
    pub filter: Option<&'a QueryPredicate>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecordResolutionMatchKind {
    Name,
    NormalizedName,
    Alias,
}
#[derive(Debug, Clone)]
pub struct RecordResolutionResult {
    pub query: String,
    pub normalized_query: String,
    pub match_kind: RecordResolutionMatchKind,
    pub matched_text: String,
    pub evidence: Option<String>,
    pub record: SourceRecordSummary,
}
#[derive(Debug, Clone)]
pub enum RecordRefResolutionResult {
    Key(RecordKey),
    Miss,
    Ambiguous(Vec<RecordResolutionResult>),
}
impl AtlasRetrievalService {
    pub fn get_records(
        &self,
        request: GetRecordsRequest<'_>,
    ) -> Result<SourceSummaryBatch, SearchError> {
        let mut result = SourceSummaryBatch {
            records: Vec::new(),
            missing: Vec::new(),
        };
        for chunk in request.record_keys.chunks(1024) {
            let batch = self.index.read_summaries(chunk)?;
            result.records.extend(batch.records);
            result.missing.extend(batch.missing);
        }
        Ok(result)
    }
    pub fn get_record(
        &self,
        request: GetRecordRequest<'_>,
    ) -> Result<Option<SourceRecordDetail>, SearchError> {
        let Some(summary) = self.index.read_summary(request.record_key)? else {
            return Ok(None);
        };
        let source = self
            .index
            .read_source_record(request.record_key)?
            .ok_or_else(|| {
                SearchError::artifact_contract_violation("summary has no checked source body")
            })?;
        for locator in request.selected_content {
            if &locator.record != request.record_key || source.node_at(&locator.owners).is_none() {
                return Err(SearchError::invalid_search_options(
                    "selected content must belong to an address in this root",
                ));
            }
        }
        let content = self.index.read_content(request.selected_content)?;
        Ok(Some(SourceRecordDetail {
            summary,
            source,
            content,
        }))
    }
    pub fn read_content(
        &self,
        locators: &[SourceContentLocator],
    ) -> Result<SourceContentBundle, SearchError> {
        Ok(self.index.read_content(locators)?)
    }
    pub fn recover_passage(
        &self,
        location: &atlas_index::SourceUnitLocation,
    ) -> Result<Option<String>, SearchError> {
        Ok(self.index.recover_passage(location)?)
    }
    pub fn list_records(
        &self,
        request: ListRecordsRequest<'_>,
    ) -> Result<ListRecordsResult, SearchError> {
        let predicate = self.predicate(request.filter)?;
        let page = self.index.eligible_keys(
            &predicate,
            request.scope.keys(),
            request.prefer_remaster,
            request.page.offset()? as usize,
            request.page.size() as usize,
        )?;
        let records = self.index.read_summaries(&page.keys)?;
        if !records.missing.is_empty() {
            return Err(SearchError::artifact_contract_violation(
                "browse keys have missing summaries",
            ));
        }
        let info =
            SearchPageInfo::from_page(request.page, records.records.len(), page.total as u64)?;
        Ok(ListRecordsResult {
            record_keys: page.keys,
            records: records.records,
            total: page.total as u64,
            page: info,
        })
    }
    pub fn resolve_record(
        &self,
        request: ResolveRecordRequest<'_>,
    ) -> Result<Vec<RecordResolutionResult>, SearchError> {
        let matches = self.index.lookup_name_or_alias(request.query)?;
        let keys = matches.iter().map(|m| m.key.clone()).collect::<Vec<_>>();
        let query = self.predicate(request.filter)?;
        let eligible = self
            .index
            .eligible_keys(&query, Some(&keys), false, 0, 1024)?;
        if eligible.total > 1024 {
            return Err(SearchError::invalid_search_options(
                "name resolution exceeds 1024 exact candidates",
            ));
        }
        let summaries = self.summaries(&eligible.keys)?;
        let mut result = Vec::new();
        for record in summaries {
            let exact = matches
                .iter()
                .find(|m| m.key == record.key && m.alias.is_none())
                .or_else(|| matches.iter().find(|m| m.key == record.key))
                .ok_or_else(|| {
                    SearchError::artifact_contract_violation(
                        "eligible exact candidate has no indexed identity evidence",
                    )
                })?;
            let name = record.name.as_value().cloned().unwrap_or_default();
            let match_kind = if exact.alias.is_some() {
                RecordResolutionMatchKind::Alias
            } else if name == request.query {
                RecordResolutionMatchKind::Name
            } else {
                RecordResolutionMatchKind::NormalizedName
            };
            result.push(RecordResolutionResult {
                query: request.query.to_owned(),
                normalized_query: atlas_domain::normalize_record_name(request.query),
                match_kind,
                matched_text: exact.alias.clone().unwrap_or(name),
                evidence: exact.evidence.clone(),
                record,
            });
        }
        result.sort_by(|a, b| a.record.key.cmp(&b.record.key));
        Ok(result)
    }
    pub fn resolve_record_ref(
        &self,
        request: ResolveRecordRefRequest<'_>,
    ) -> Result<RecordRefResolutionResult, SearchError> {
        if let Ok(key) = RecordKey::parse(request.record_ref) {
            let query = self.predicate(request.filter)?;
            let eligible =
                self.index
                    .eligible_keys(&query, Some(std::slice::from_ref(&key)), false, 0, 1)?;
            return Ok(if !eligible.keys.is_empty() {
                RecordRefResolutionResult::Key(key)
            } else {
                RecordRefResolutionResult::Miss
            });
        }
        let matches = self.resolve_record(ResolveRecordRequest {
            query: request.record_ref,
            filter: request.filter,
        })?;
        Ok(match matches.len() {
            0 => RecordRefResolutionResult::Miss,
            1 => RecordRefResolutionResult::Key(matches[0].record.key.clone()),
            _ => RecordRefResolutionResult::Ambiguous(matches),
        })
    }
    pub(crate) fn summaries(
        &self,
        keys: &[RecordKey],
    ) -> Result<Vec<SourceRecordSummary>, SearchError> {
        let mut records = Vec::with_capacity(keys.len());
        for chunk in keys.chunks(1024) {
            let batch = self.index.read_summaries(chunk)?;
            if !batch.missing.is_empty() {
                return Err(SearchError::artifact_contract_violation(
                    "candidate summary is absent",
                ));
            }
            records.extend(batch.records);
        }
        Ok(records)
    }
}
