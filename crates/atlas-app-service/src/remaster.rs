//! Product projection of attributed changes, independent of identity and preference.
use crate::{AppServiceError, AppServiceResult, AtlasAppService, ReadExecutionContext};
use atlas_app_model::*;
use atlas_domain::RecordKey;
use atlas_record::source_content::{ContentReferenceResolution, ContentReferenceTarget};
use atlas_search::{
    AtlasRetrievalService, RemasterChangeKind, RemasterCitationSource, RemasterEndpointResolution,
    RemasterOrigin,
};

const INFORMATION_LIMIT: usize = 20;
const TEXT_LIMIT: usize = 400;
pub(crate) struct ProjectionBudget {
    citations: usize,
}
impl Default for ProjectionBudget {
    fn default() -> Self {
        Self { citations: 200 }
    }
}

impl AtlasAppService {
    pub fn remaster_relationships(
        &self,
        request: RemasterRelationshipsRequest,
        execution: &ReadExecutionContext,
    ) -> AppServiceResult<RemasterRelationshipsResponse> {
        execution.check()?;
        if !(1..=100).contains(&request.limit) {
            return Err(AppServiceError::invalid_request(
                "Remaster information limit must be between 1 and 100",
            ));
        }
        match &request.subject {
            RemasterSubjectRequest::Term { term }
                if term.trim().is_empty() || term.chars().count() > 256 =>
            {
                return Err(AppServiceError::invalid_request(
                    "Remaster information requires a nonempty term of at most 256 characters",
                ));
            }
            RemasterSubjectRequest::Record { selection } => {
                validate_navigation(selection)?;
            }
            _ => {}
        }
        let projection_execution = execution.clone();
        self.with_retrieval(execution, move |retrieval| {
            let generation = retrieval.artifact_generation()?;
            if let Some(expected)=request.expected_artifact_generation.as_deref() && expected!=generation {
                return Err(AppServiceError::artifact_generation_changed(expected,&generation,if matches!(&request.subject, RemasterSubjectRequest::Term{..}) {AppRecoverableAction::RefreshResults} else {AppRecoverableAction::ReloadRecord}));
            }
            let mut budget=ProjectionBudget::default();
            let (subject, relationships, source_remaster_context)=match request.subject {
                RemasterSubjectRequest::Record{selection} => {
                    check_fingerprint(&selection,retrieval)?;
                    let key=parse_key(&selection.record_key)?;
                    let result=retrieval.record_remaster_information(atlas_search::RecordRemasterInformationRequest {record_key:&key,owners:&selection.owners,limit:request.limit})?;
                    let title=result.seed.as_ref().map(|seed|crate::projection::localized_summary_view(seed,retrieval).title);
                    let available=result.seed.is_some();
                    let relationships=information_category(result.information,Some(&selection),retrieval,&mut budget);
                    let source=source_context(&selection,retrieval,&mut budget);
                    (RemasterSubjectView::Record{selection,title,available},relationships,source)
                }
                RemasterSubjectRequest::Term{term} => {
                    let result=retrieval.remaster_term_information(atlas_search::RemasterTermInformationRequest{term:&term,limit:request.limit})?;
                    let relationships=information_category(result.information,None,retrieval,&mut budget);
                    (RemasterSubjectView::Term{query:term,normalized_term:result.term,scope_label:"Source statements about this term; independent of record filters and result counts".into()},relationships,None)
                }
            };
            projection_execution.check()?;
            Ok(RemasterRelationshipsResponse{artifact_generation:generation,subject,relationships,source_remaster_context})
        })
    }
}
fn parse_key(key: &str) -> AppServiceResult<RecordKey> {
    RecordKey::parse(key)
        .map_err(|error| AppServiceError::new(AppErrorCode::InvalidRecordKey, error.to_string()))
}
fn validate_navigation(selection: &RecordNavigationView) -> AppServiceResult<()> {
    crate::records::validate_request(&RecordDetailRequest {
        spell_rank: selection.spell_rank,
        record_key: selection.record_key.clone(),
        owners: selection.owners.clone(),
        fields: selection.field.iter().cloned().collect(),
        passage: selection.passage.clone(),
        source_fingerprint: selection.source_fingerprint.clone(),
    })?;
    parse_key(&selection.record_key)?;
    Ok(())
}
fn check_fingerprint(
    selection: &RecordNavigationView,
    retrieval: &AtlasRetrievalService,
) -> AppServiceResult<()> {
    if crate::projection::navigation_fingerprint(&selection.owners, retrieval.source_fingerprint())
        .is_some()
        && selection.source_fingerprint.as_deref() != Some(retrieval.source_fingerprint())
    {
        return Err(AppServiceError::invalid_request(
            "Snapshot-local remaster navigation belongs to another artifact; reopen the record",
        ));
    }
    Ok(())
}
pub(crate) fn reading_context(
    selection: &RecordNavigationView,
    retrieval: &AtlasRetrievalService,
) -> (
    RelatedCategoryView<RemasterRelationshipsView>,
    Option<RelatedCategoryView<RemasterSourceContextView>>,
) {
    let mut budget = ProjectionBudget::default();
    let relationships = category((|| {
        let key = parse_key(&selection.record_key)?;
        let result = retrieval.record_remaster_information(
            atlas_search::RecordRemasterInformationRequest {
                record_key: &key,
                owners: &selection.owners,
                limit: INFORMATION_LIMIT,
            },
        )?;
        Ok(information_category(
            result.information,
            Some(selection),
            retrieval,
            &mut budget,
        ))
    })());
    let source = source_context(selection, retrieval, &mut budget);
    (relationships, source)
}
fn category<T>(result: AppServiceResult<RelatedCategoryView<T>>) -> RelatedCategoryView<T> {
    result.unwrap_or_else(|error| RelatedCategoryView::Error {
        error: error.into_app_error(),
    })
}
fn source_context(
    selection: &RecordNavigationView,
    retrieval: &AtlasRetrievalService,
    budget: &mut ProjectionBudget,
) -> Option<RelatedCategoryView<RemasterSourceContextView>> {
    if selection.owners.is_empty() {
        return None;
    }
    Some(category((|| {
        let key = parse_key(&selection.record_key)?;
        let provenance =
            retrieval.read_non_content_relationships(&atlas_search::SourceRelationshipRequest {
                record: key,
                owners: Some(selection.owners.clone()),
                field: None,
                direction: atlas_search::SourceRelationshipDirection::Outgoing,
                limit: 1024,
            })?;
        if provenance.truncated {
            return Ok(RelatedCategoryView::Unavailable {
                reason: "The source-entry references are incomplete".into(),
            });
        }
        let mut origin = None;
        for edge in provenance
            .occurrences
            .iter()
            .filter(|edge| edge.kind == "CompendiumProvenance")
        {
            let key = match &edge.resolution {
                ContentReferenceResolution::Resolved(ContentReferenceTarget::Record { key }) => {
                    Some(key.clone())
                }
                ContentReferenceResolution::Unresolved => edge
                    .authored_target
                    .as_deref()
                    .and_then(atlas_record::source_record::SourceReferenceIndex::explicit_root_key),
                _ => None,
            };
            let Some(key) = key else {
                return Ok(RelatedCategoryView::Unavailable {
                    reason: "The source entry could not be identified from its explicit reference"
                        .into(),
                });
            };
            if origin.as_ref().is_some_and(|prior| prior != &key) {
                return Ok(RelatedCategoryView::Unavailable {
                    reason: "The selected document identifies conflicting source entries".into(),
                });
            }
            origin = Some(key);
        }
        let Some(origin) = origin else {
            return Ok(RelatedCategoryView::Empty);
        };
        let source_selection = navigation(&origin, &[], retrieval);
        let summaries = retrieval.get_records(atlas_search::GetRecordsRequest {
            record_keys: std::slice::from_ref(&origin),
        })?;
        let summary = summaries
            .records
            .first()
            .map(|record| crate::projection::localized_summary_view(record, retrieval));
        let label = summary
            .as_ref()
            .map_or_else(|| origin.to_string(), |record| record.title.clone());
        let resolution = summary.map_or_else(
            || RemasterEndpointResolutionView::Absent {
                record_key: origin.to_string(),
                message: "This source entry is not available in the current collection".into(),
            },
            |record| RemasterEndpointResolutionView::Current {
                record,
                navigation: source_selection.clone(),
            },
        );
        // A known origin remains visible when its secondary declaration read fails.
        let relationships = category(
            retrieval
                .record_remaster_information(atlas_search::RecordRemasterInformationRequest {
                    record_key: &origin,
                    owners: &[],
                    limit: INFORMATION_LIMIT,
                })
                .map_err(AppServiceError::from)
                .map(|result| {
                    information_category(
                        result.information,
                        Some(&source_selection),
                        retrieval,
                        budget,
                    )
                }),
        );
        Ok(RelatedCategoryView::Content {
            content: RemasterSourceContextView {
                heading: format!("Source entry: {label}"),
                source: RemasterEndpointView {
                    label,
                    context: None,
                    resolution,
                },
                relationships,
            },
        })
    })()))
}
fn shorten(value: &str) -> (String, bool) {
    if value.chars().count() <= TEXT_LIMIT {
        (value.to_owned(), false)
    } else {
        (
            value
                .chars()
                .take(TEXT_LIMIT - 1)
                .chain(std::iter::once('…'))
                .collect(),
            true,
        )
    }
}
fn text(value: &str) -> String {
    shorten(value).0
}
fn navigation(
    key: &RecordKey,
    owners: &[atlas_domain::OwnedContentLocator],
    retrieval: &AtlasRetrievalService,
) -> RecordNavigationView {
    RecordNavigationView {
        spell_rank: None,
        record_key: key.to_string(),
        owners: owners.to_vec(),
        field: None,
        passage: None,
        source_fingerprint: crate::projection::navigation_fingerprint(
            owners,
            retrieval.source_fingerprint(),
        ),
    }
}
fn context(context: &atlas_search::RemasterContext) -> Vec<String> {
    let mut values = vec![text(&context.section), text(&context.namespace)];
    if let Some(qualifier) = &context.qualifier {
        values.push(text(qualifier));
    }
    values.retain(|value| !value.is_empty());
    values.dedup();
    values
}
fn kind(value: RemasterChangeKind) -> RemasterChangeKindView {
    match value {
        RemasterChangeKind::Renamed => RemasterChangeKindView::Renamed,
        RemasterChangeKind::Merged => RemasterChangeKindView::Merged,
        RemasterChangeKind::Replaced => RemasterChangeKindView::Replaced,
        RemasterChangeKind::Removed => RemasterChangeKindView::Removed,
        RemasterChangeKind::MechanicsChanged => RemasterChangeKindView::MechanicsChanged,
        RemasterChangeKind::Discussion => RemasterChangeKindView::Discussion,
        RemasterChangeKind::Unclassified => RemasterChangeKindView::Unclassified,
    }
}
fn kind_label(value: RemasterChangeKind) -> &'static str {
    match value {
        RemasterChangeKind::Renamed => "Renamed",
        RemasterChangeKind::Merged => "Merged",
        RemasterChangeKind::Replaced => "Replaced",
        RemasterChangeKind::Removed => "Removed",
        RemasterChangeKind::MechanicsChanged => "Mechanics changed",
        RemasterChangeKind::Discussion => "Rules discussion",
        RemasterChangeKind::Unclassified => "Source-described change",
    }
}
fn endpoint(
    value: atlas_search::RemasterEndpointResult,
    retrieval: &AtlasRetrievalService,
) -> (RemasterEndpointView, bool) {
    let (label, mut truncated) = shorten(&value.label);
    let resolution = match value.resolution {
        RemasterEndpointResolution::Current { record, owners } => {
            RemasterEndpointResolutionView::Current {
                navigation: navigation(&record.key, &owners, retrieval),
                record: crate::projection::localized_summary_view(&record, retrieval),
            }
        }
        RemasterEndpointResolution::MissingExplicitKey { record_key } => {
            RemasterEndpointResolutionView::Absent {
                record_key: record_key.to_string(),
                message: "This identified entry is not available in the current collection".into(),
            }
        }
        RemasterEndpointResolution::Unresolved { reference } => {
            RemasterEndpointResolutionView::Unresolved {
                reference: reference.map(|value| text(&value)),
                message:
                    "The source does not establish an available record for this name or reference"
                        .into(),
            }
        }
        RemasterEndpointResolution::Ambiguous {
            candidates,
            truncated: lower_cut,
        } => {
            let cut = lower_cut || candidates.len() > 10;
            truncated |= cut;
            RemasterEndpointResolutionView::Ambiguous{candidates:candidates.into_iter().take(10).map(|value|RemasterEndpointCandidateView{navigation:navigation(&value.record.key,&value.owners,retrieval),record:crate::projection::localized_summary_view(&value.record,retrieval)}).collect(),truncated:cut,message:"Several current records could match this source reference; no replacement was selected".into()}
        }
        RemasterEndpointResolution::Term => RemasterEndpointResolutionView::Term { message: None },
        RemasterEndpointResolution::SourceText => RemasterEndpointResolutionView::SourceText,
    };
    (
        RemasterEndpointView {
            label,
            context: None,
            resolution,
        },
        truncated,
    )
}
fn sides(
    values: Vec<atlas_search::RemasterSideResult>,
    retrieval: &AtlasRetrievalService,
) -> (Vec<RemasterSideView>, bool) {
    let mut truncated = values.len() > 10;
    let values = values
        .into_iter()
        .take(10)
        .map(|side| {
            let (body, short) = shorten(&side.text);
            let mut cut = short || side.endpoints.len() > 20 || side.terms.len() > 20;
            let terms = side
                .terms
                .into_iter()
                .take(20)
                .map(|value| {
                    let (value, short) = shorten(&value);
                    cut |= short;
                    value
                })
                .collect();
            let endpoints = side
                .endpoints
                .into_iter()
                .take(20)
                .map(|value| {
                    let (value, short) = endpoint(value, retrieval);
                    cut |= short;
                    value
                })
                .collect();
            truncated |= cut;
            RemasterSideView {
                text: body,
                terms,
                endpoints,
                truncated: cut,
            }
        })
        .collect();
    (values, truncated)
}
fn citations(
    values: Vec<atlas_search::RemasterCitationResult>,
    retrieval: &AtlasRetrievalService,
    budget: &mut ProjectionBudget,
) -> (Vec<RemasterCitationView>, bool) {
    let take = values.len().min(20).min(budget.citations);
    let cut = take < values.len();
    budget.citations -= take;
    (
        values
            .into_iter()
            .take(take)
            .map(|value| {
                let (statement, shortened) = shorten(&value.statement);
                let origin = match value.origin {
                    RemasterOrigin::Foundry => RemasterCitationOriginView::Foundry,
                    RemasterOrigin::AtlasReviewed => RemasterCitationOriginView::AtlasReviewed,
                };
                let destination = match value.source {
                    RemasterCitationSource::Journal { locator } => {
                        let mut navigation =
                            navigation(&locator.record, &locator.owners, retrieval);
                        navigation.field = Some(locator.field);
                        RemasterCitationDestinationView::Internal { navigation }
                    }
                    RemasterCitationSource::CapturedFile { path } => {
                        let revision_ok = (40..=64).contains(&value.revision.len())
                            && value.revision.chars().all(|c| c.is_ascii_hexdigit());
                        let path_ok = !path.starts_with('/')
                            && path.split('/').all(|part| {
                                !part.is_empty()
                                    && part != "."
                                    && part != ".."
                                    && part
                                        .chars()
                                        .all(|c| c.is_ascii_alphanumeric() || "-_.".contains(c))
                            });
                        if origin == RemasterCitationOriginView::Foundry && revision_ok && path_ok {
                            RemasterCitationDestinationView::External {
                                url: format!(
                                    "https://github.com/foundryvtt/pf2e/blob/{}/{path}",
                                    value.revision
                                ),
                            }
                        } else {
                            RemasterCitationDestinationView::Attribution { reason: None }
                        }
                    }
                };
                RemasterCitationView {
                    origin,
                    attribution: if origin == RemasterCitationOriginView::AtlasReviewed {
                        format!("Atlas-reviewed — {}", text(&value.attribution))
                    } else {
                        text(&value.attribution)
                    },
                    revision: text(&value.revision),
                    statement,
                    shortened,
                    destination,
                }
            })
            .collect(),
        cut,
    )
}
fn contains_selection(sides: &[RemasterSideView], selection: &RecordNavigationView) -> bool {
    sides
        .iter()
        .flat_map(|side| &side.endpoints)
        .any(|value| match &value.resolution {
            RemasterEndpointResolutionView::Current { navigation, .. } => {
                navigation.record_key == selection.record_key
                    && navigation.owners == selection.owners
            }
            RemasterEndpointResolutionView::Absent { record_key, .. } => {
                record_key == &selection.record_key && selection.owners.is_empty()
            }
            _ => false,
        })
}
fn side_text(sides: &[RemasterSideView]) -> String {
    text(
        &sides
            .iter()
            .map(|side| side.text.as_str())
            .collect::<Vec<_>>()
            .join("; "),
    )
}
fn notice(
    value: &RemasterDeclarationView,
    selection: Option<&RecordNavigationView>,
) -> RemasterNoticeView {
    let before = selection.is_some_and(|selection| contains_selection(&value.before, selection));
    let after = selection.is_some_and(|selection| contains_selection(&value.after, selection));
    let kinds = &value.kinds;
    let (heading, destinations, description) =
        if kinds.contains(&RemasterChangeKindView::Discussion) {
            (
                "Remaster rules context",
                &value.after,
                Some(side_text(&value.after)),
            )
        } else if kinds.contains(&RemasterChangeKindView::Merged) {
            if after {
                ("Merged from", &value.before, Some(side_text(&value.before)))
            } else {
                ("Merged into", &value.after, None)
            }
        } else if kinds.contains(&RemasterChangeKindView::Removed) {
            if after && !before {
                (
                    "Related removed entry",
                    &value.before,
                    Some(text(&format!(
                        "{} is listed as removed in the remaster.",
                        side_text(&value.before)
                    ))),
                )
            } else {
                (
                    "Removed in the remaster",
                    &value.after,
                    Some("Listed as removed in the remaster.".into()),
                )
            }
        } else if kinds.contains(&RemasterChangeKindView::Renamed) {
            if after {
                ("Former name", &value.before, Some(side_text(&value.before)))
            } else {
                ("Renamed to", &value.after, Some(side_text(&value.after)))
            }
        } else if kinds.contains(&RemasterChangeKindView::Replaced) {
            if after {
                ("Replaces", &value.before, Some(side_text(&value.before)))
            } else {
                ("Replacement options", &value.after, None)
            }
        } else if kinds.contains(&RemasterChangeKindView::MechanicsChanged) {
            (
                "Mechanics changed",
                &value.after,
                Some(side_text(&value.after)),
            )
        } else {
            (
                "Remaster relationship",
                if before { &value.after } else { &value.before },
                value.authored_status.clone(),
            )
        };
    let mut links = Vec::new();
    for endpoint in destinations.iter().flat_map(|side| &side.endpoints) {
        if let RemasterEndpointResolutionView::Current { record, navigation } = &endpoint.resolution
        {
            if selection.is_some_and(|selected| {
                selected.record_key == navigation.record_key && selected.owners == navigation.owners
            }) {
                continue;
            }
            if !links
                .iter()
                .any(|link: &RemasterNoticeLinkView| link.navigation == *navigation)
            {
                let label = match &record.publication {
                    Some(publication) => format!("{} — {publication}", record.title),
                    None => format!(
                        "{} — {}",
                        record.title,
                        record.pack.as_deref().unwrap_or(&record.record_key)
                    ),
                };
                links.push(RemasterNoticeLinkView {
                    label,
                    navigation: navigation.clone(),
                });
            }
        }
    }
    let description = description
        .filter(|description| !description.is_empty())
        .map(|description| {
            if kinds.contains(&RemasterChangeKindView::MechanicsChanged)
                && heading != "Mechanics changed"
            {
                text(&format!(
                    "{description} The source also notes changed mechanics."
                ))
            } else {
                description
            }
        });
    RemasterNoticeView {
        declaration_id: value.id.clone(),
        heading: heading.into(),
        description,
        links,
        attribution: value
            .citations
            .iter()
            .any(|citation| citation.origin == RemasterCitationOriginView::AtlasReviewed)
            .then(|| "Atlas-reviewed correspondence".into()),
    }
}
fn information_category(
    information: atlas_search::RemasterInformation,
    selection: Option<&RecordNavigationView>,
    retrieval: &AtlasRetrievalService,
    budget: &mut ProjectionBudget,
) -> RelatedCategoryView<RemasterRelationshipsView> {
    if information.declarations.is_empty() && !information.truncated {
        return RelatedCategoryView::Empty;
    }
    let mut truncated = information.truncated;
    let mut notices = Vec::new();
    let declarations = information
        .declarations
        .into_iter()
        .map(|value| {
            let atlas_reviewed = value
                .citations
                .iter()
                .any(|citation| citation.origin == RemasterOrigin::AtlasReviewed);
            let (before, before_cut) = sides(value.before, retrieval);
            let (after, after_cut) = sides(value.after, retrieval);
            let (citations, citations_truncated) = citations(value.citations, retrieval, budget);
            let mut value = RemasterDeclarationView {
                id: value.id,
                heading: String::new(),
                summary: None,
                kinds: value.kinds.into_iter().map(kind).collect(),
                authored_status: value.authored_status.map(|value| text(&value)),
                context: context(&value.context),
                before,
                after,
                citations,
                citations_truncated,
                truncated: before_cut || after_cut || citations_truncated,
            };
            let mut projected = notice(&value, selection);
            if atlas_reviewed {
                projected.attribution = Some("Atlas-reviewed correspondence".into());
            }
            value.heading = projected.heading.clone();
            value.summary = projected.description.clone();
            notices.push(projected);
            truncated |= value.truncated;
            value
        })
        .collect::<Vec<_>>();
    RelatedCategoryView::Content {
        content: RemasterRelationshipsView {
            notices,
            declarations,
            truncated,
        },
    }
}
pub(crate) fn lookup_context(
    support: atlas_search::RemasterLookupSupport,
    retrieval: &AtlasRetrievalService,
    budget: &mut ProjectionBudget,
) -> RemasterLookupContextView {
    let mut truncated = support.truncated || support.declarations.len() > 20;
    let mut successor = false;
    let supports = support
        .declarations
        .into_iter()
        .take(20)
        .map(|value| {
            successor |= value.kinds.iter().any(|kind| {
                matches!(
                    kind,
                    RemasterChangeKind::Merged | RemasterChangeKind::Replaced
                )
            });
            let (citations, citations_truncated) = citations(value.citations, retrieval, budget);
            truncated |= citations_truncated;
            RemasterLookupSupportView {
                declaration_id: value.id,
                kind_labels: value
                    .kinds
                    .into_iter()
                    .map(|kind| kind_label(kind).into())
                    .collect(),
                context: context(&value.context),
                citations,
                citations_truncated,
            }
        })
        .collect();
    RemasterLookupContextView {
        label: if successor {
            "Successor found by source-described former name"
        } else {
            "Former name"
        }
        .into(),
        supports,
        truncated,
    }
}
pub(crate) fn alias_matches(
    values: Vec<atlas_search::RemasterAliasMatch>,
    retrieval: &AtlasRetrievalService,
    budget: &mut ProjectionBudget,
) -> Vec<RemasterAliasMatchView> {
    let cut = values.len() > 10;
    let mut values = values
        .into_iter()
        .take(10)
        .map(|value| RemasterAliasMatchView {
            matched_text: text(&value.alias),
            lookup_context: lookup_context(value.support, retrieval, budget),
        })
        .collect::<Vec<_>>();
    if cut && let Some(last) = values.last_mut() {
        last.lookup_context.truncated = true;
    }
    values
}

#[cfg(test)]
mod tests {
    use super::*;
    fn side(label: &str, key: &str) -> RemasterSideView {
        RemasterSideView {
            text: label.into(),
            terms: vec![label.into()],
            endpoints: vec![RemasterEndpointView {
                label: label.into(),
                context: None,
                resolution: RemasterEndpointResolutionView::Absent {
                    record_key: key.into(),
                    message: "Unavailable".into(),
                },
            }],
            truncated: false,
        }
    }
    fn declaration(kinds: Vec<RemasterChangeKindView>) -> RemasterDeclarationView {
        RemasterDeclarationView {
            id: "test".into(),
            heading: String::new(),
            summary: None,
            kinds,
            authored_status: None,
            context: vec![],
            before: vec![side("Old name", "test:aaaaaaaaaaaaaaaa")],
            after: vec![side("New name", "test:bbbbbbbbbbbbbbbb")],
            citations: vec![],
            citations_truncated: false,
            truncated: false,
        }
    }
    fn selected(key: &str) -> RecordNavigationView {
        RecordNavigationView {
            spell_rank: None,
            record_key: key.into(),
            owners: vec![],
            field: None,
            passage: None,
            source_fingerprint: None,
        }
    }
    #[test]
    fn reverse_merger_and_same_identity_rename_have_projected_meaning() {
        let merged = declaration(vec![RemasterChangeKindView::Merged]);
        assert_eq!(
            notice(&merged, Some(&selected("test:bbbbbbbbbbbbbbbb"))).heading,
            "Merged from"
        );
        assert_eq!(
            notice(&merged, Some(&selected("test:aaaaaaaaaaaaaaaa"))).heading,
            "Merged into"
        );
        let mut renamed = declaration(vec![
            RemasterChangeKindView::Renamed,
            RemasterChangeKindView::MechanicsChanged,
        ]);
        renamed.after = vec![side("New name", "test:aaaaaaaaaaaaaaaa")];
        let projected = notice(&renamed, Some(&selected("test:aaaaaaaaaaaaaaaa")));
        assert_eq!(projected.heading, "Former name");
        assert!(projected.description.unwrap().contains("changed mechanics"));
        assert!(projected.links.is_empty());
    }
    #[test]
    fn discussion_and_removed_copy_do_not_claim_mechanical_equivalence() {
        let discussion = declaration(vec![RemasterChangeKindView::Discussion]);
        assert_eq!(notice(&discussion, None).heading, "Remaster rules context");
        let removed = declaration(vec![RemasterChangeKindView::Removed]);
        assert_eq!(
            notice(&removed, None).description.as_deref(),
            Some("Listed as removed in the remaster.")
        );
    }
    #[test]
    fn source_excerpt_shortening_counts_unicode_characters() {
        let (short, cut) = shorten(&"é".repeat(401));
        assert!(cut);
        assert_eq!(short.chars().count(), 400);
        assert!(short.ends_with('…'));
        assert_eq!(shorten(&"é".repeat(400)).1, false);
    }
    #[test]
    fn declaration_and_global_citation_cuts_are_explicit() {
        let fixture = crate::test_support::fixture_worker();
        fixture
            .worker
            .with_retrieval(&ReadExecutionContext::uncancellable(), |retrieval| {
                let make = |ordinal| atlas_search::RemasterDeclarationResult {
                    id: format!("declaration-{ordinal}"),
                    kinds: vec![RemasterChangeKind::Discussion],
                    authored_status: None,
                    context: atlas_search::RemasterContext {
                        section: "Rules".into(),
                        namespace: "rules".into(),
                        qualifier: None,
                        source_type: None,
                    },
                    before: vec![],
                    after: vec![],
                    citations: (0..21)
                        .map(|_| atlas_search::RemasterCitationResult {
                            origin: RemasterOrigin::Foundry,
                            attribution: "Source file".into(),
                            revision: "a".repeat(40),
                            statement: "é".repeat(401),
                            source: RemasterCitationSource::CapturedFile {
                                path: "src/module/migration.ts".into(),
                            },
                        })
                        .collect(),
                };
                let information = atlas_search::RemasterInformation {
                    declarations: (0..11).map(make).collect(),
                    truncated: false,
                };
                let RelatedCategoryView::Content { content } = information_category(
                    information,
                    None,
                    retrieval,
                    &mut ProjectionBudget::default(),
                ) else {
                    panic!("content");
                };
                assert_eq!(
                    content
                        .declarations
                        .iter()
                        .map(|value| value.citations.len())
                        .sum::<usize>(),
                    200
                );
                assert!(content.truncated);
                assert!(
                    content
                        .declarations
                        .iter()
                        .all(|value| value.citations_truncated)
                );
                assert!(content.declarations[0].citations[0].shortened);
                assert!(content.declarations[10].citations.is_empty());
                Ok(())
            })
            .unwrap();
    }
}

#[cfg(test)]
mod owned_tests {
    use super::*;
    use atlas_search::test_support::{
        add_remaster_relationship, input, open_input_fixture, record,
    };
    use serde_json::json;
    const OLD: &str = "actions:aaaaaaaaaaaaaaaa";
    const NEW: &str = "actions:bbbbbbbbbbbbbbbb";
    const MISSING: &str = "actions:missing000000001";
    const ACTOR: &str = "actors:cccccccccccccccc";
    fn fixture() -> (
        AtlasRetrievalService,
        atlas_search::test_support::FixtureArtifact,
    ) {
        let child = |id: &str, origin: Option<&str>, other: Option<&str>| {
            let mut value = json!({"_id":id,"name":"Customized owned action","type":"action","system":{"publication":{"remaster":false},"description":{"value":"<p>Keep customized rules.</p>"}}});
            if let Some(origin) = origin {
                value["_stats"] = json!({"compendiumSource":origin});
            }
            if let Some(other) = other {
                value["flags"] = json!({"core":{"sourceId":other}});
            }
            value
        };
        let mut data = input(
            vec![
                record(
                    "actions",
                    "Item",
                    json!({"_id":"aaaaaaaaaaaaaaaa","name":"Old action","type":"action","system":{"publication":{"remaster":false}}}),
                ),
                record(
                    "actions",
                    "Item",
                    json!({"_id":"bbbbbbbbbbbbbbbb","name":"New action","type":"action","system":{"publication":{"remaster":true}}}),
                ),
                record(
                    "actors",
                    "Actor",
                    json!({"_id":"cccccccccccccccc","name":"Parent creature","type":"npc","system":{"publication":{"remaster":true}},"items":[
                        child("owned00000000001",Some("Compendium.pf2e.actions.Item.aaaaaaaaaaaaaaaa"),None),
                        child("owned00000000002",None,None),
                        child("owned00000000003",Some("Compendium.pf2e.actions.Item.missing000000001"),None),
                        child("owned00000000004",Some("Compendium.pf2e.actions.Item.aaaaaaaaaaaaaaaa"),Some("Compendium.pf2e.actions.Item.bbbbbbbbbbbbbbbb")),
                        child("owned00000000005",Some("Compendium.pf2e.actions.Item.Not a native ID"),None),
                        child("owned00000000006",Some("Compendium.pf2e.actions.Item.bbbbbbbbbbbbbbbb"),None)
                    ]}),
                ),
            ],
            false,
        );
        add_remaster_relationship(&mut data, &OLD.parse().unwrap(), &NEW.parse().unwrap()).unwrap();
        add_remaster_relationship(&mut data, &MISSING.parse().unwrap(), &NEW.parse().unwrap())
            .unwrap();
        open_input_fixture(data).unwrap()
    }
    fn selection(id: &str) -> RecordNavigationView {
        RecordNavigationView {
            spell_rank: None,
            record_key: ACTOR.into(),
            owners: vec![atlas_domain::OwnedContentLocator {
                collection: "/items".into(),
                identity: atlas_domain::OwnedContentIdentity::Stable(id.into()),
            }],
            field: None,
            passage: None,
            source_fingerprint: None,
        }
    }
    #[test]
    fn remaster_owned_source_context_is_separate_exact_and_honest_about_origins() {
        let (retrieval, _artifact) = fixture();
        let (direct, source) = reading_context(&selection("owned00000000001"), &retrieval);
        assert!(matches!(direct, RelatedCategoryView::Empty));
        let Some(RelatedCategoryView::Content { content }) = source else {
            panic!("explicit source");
        };
        assert!(
            matches!(content.source.resolution,RemasterEndpointResolutionView::Current{ref navigation,..} if navigation.record_key==OLD)
        );
        let RelatedCategoryView::Content { content } = content.relationships else {
            panic!("source relationships");
        };
        assert_eq!(content.notices[0].links[0].navigation.record_key, NEW);
        assert_eq!(
            content.notices[0].attribution.as_deref(),
            Some("Atlas-reviewed correspondence")
        );
        assert!(matches!(
            reading_context(&selection("owned00000000002"), &retrieval).1,
            Some(RelatedCategoryView::Empty)
        ));
        for id in ["owned00000000004", "owned00000000005"] {
            assert!(matches!(
                reading_context(&selection(id), &retrieval).1,
                Some(RelatedCategoryView::Unavailable { .. })
            ));
        }
        let Some(RelatedCategoryView::Content { content }) =
            reading_context(&selection("owned00000000003"), &retrieval).1
        else {
            panic!("known absent origin");
        };
        assert!(
            matches!(content.source.resolution,RemasterEndpointResolutionView::Absent{ref record_key,..} if record_key==MISSING)
        );
        assert!(matches!(
            content.relationships,
            RelatedCategoryView::Content { .. }
        ));
    }
}

#[cfg(test)]
mod projection_bounds_tests {
    use super::*;
    fn source_citation(origin: RemasterOrigin) -> atlas_search::RemasterCitationResult {
        atlas_search::RemasterCitationResult {
            origin,
            attribution: "Evidence source".into(),
            revision: "a".repeat(40),
            statement: "Complete source statement".into(),
            source: RemasterCitationSource::CapturedFile {
                path: "src/module/migration.ts".into(),
            },
        }
    }
    fn source_declaration(id: usize) -> atlas_search::RemasterDeclarationResult {
        atlas_search::RemasterDeclarationResult {
            id: format!("declaration-{id}"),
            kinds: vec![RemasterChangeKind::Renamed],
            authored_status: Some("Renamed".into()),
            context: atlas_search::RemasterContext {
                section: "Class Features".into(),
                namespace: "class_feature".into(),
                qualifier: Some("Fixture class".into()),
                source_type: Some("action".into()),
            },
            before: vec![],
            after: vec![],
            citations: vec![],
        }
    }
    #[test]
    fn remaster_side_and_reference_overflow_is_visible_at_every_container() {
        let fixture = crate::test_support::fixture_worker();
        fixture
            .worker
            .with_retrieval(&ReadExecutionContext::uncancellable(), |retrieval| {
                let mut declaration = source_declaration(0);
                declaration.before = (0..11)
                    .map(|group| atlas_search::RemasterSideResult {
                        text: format!("Group {group}"),
                        terms: (0..21).map(|term| format!("Term {group}/{term}")).collect(),
                        endpoints: (0..21)
                            .map(|ordinal| atlas_search::RemasterEndpointResult {
                                label: format!("Reference {ordinal}"),
                                reference: None,
                                resolution: RemasterEndpointResolution::Term,
                            })
                            .collect(),
                    })
                    .collect();
                let RelatedCategoryView::Content { content } = information_category(
                    atlas_search::RemasterInformation {
                        declarations: vec![declaration],
                        truncated: false,
                    },
                    None,
                    retrieval,
                    &mut ProjectionBudget::default(),
                ) else {
                    panic!("bounded information");
                };
                assert!(content.truncated);
                let declaration = &content.declarations[0];
                assert!(declaration.truncated);
                assert_eq!(declaration.before.len(), 10);
                assert_eq!(declaration.before[9].text, "Group 9");
                for group in &declaration.before {
                    assert!(group.truncated);
                    assert_eq!(group.terms.len(), 20);
                    assert_eq!(group.endpoints.len(), 20);
                }
                Ok(())
            })
            .unwrap();
    }
    #[test]
    fn remaster_alias_supports_share_one_response_budget_without_losing_cut_indicators() {
        let fixture = crate::test_support::fixture_worker();
        fixture.worker.with_retrieval(&ReadExecutionContext::uncancellable(),|retrieval|{
            let declarations=(0..21).map(|id|{let mut declaration=source_declaration(id);declaration.citations=vec![source_citation(RemasterOrigin::Foundry);21];declaration}).collect::<Vec<_>>();
            let aliases=(0..11).map(|id|atlas_search::RemasterAliasMatch{alias:format!("Former name {id}"),support:atlas_search::RemasterLookupSupport{declarations:declarations.clone(),truncated:false}}).collect();
            let projected=alias_matches(aliases,retrieval,&mut ProjectionBudget::default());
            assert_eq!(projected.len(),10);assert_eq!(projected[9].matched_text,"Former name 9");
            assert!(projected.iter().all(|matched|matched.lookup_context.truncated));
            assert!(projected.iter().all(|matched|matched.lookup_context.supports.len()==20));
            let supports=projected.iter().flat_map(|matched|&matched.lookup_context.supports).collect::<Vec<_>>();
            assert_eq!(supports.iter().map(|support|support.citations.len()).sum::<usize>(),200);
            assert!(supports.iter().all(|support|support.citations.len()<=20&&support.citations_truncated));
            assert_eq!(projected[0].lookup_context.supports[19].declaration_id,"declaration-19");
            assert_eq!(projected[0].lookup_context.supports[0].context.last().map(String::as_str),Some("Fixture class"));
            Ok(())
        }).unwrap();
    }
    #[test]
    fn remaster_reviewed_notice_keeps_its_origin_when_citation_budget_is_exhausted() {
        let fixture = crate::test_support::fixture_worker();
        fixture
            .worker
            .with_retrieval(&ReadExecutionContext::uncancellable(), |retrieval| {
                let mut declaration = source_declaration(0);
                declaration.citations = vec![source_citation(RemasterOrigin::AtlasReviewed)];
                let RelatedCategoryView::Content { content } = information_category(
                    atlas_search::RemasterInformation {
                        declarations: vec![declaration],
                        truncated: false,
                    },
                    None,
                    retrieval,
                    &mut ProjectionBudget { citations: 0 },
                ) else {
                    panic!("retained statement");
                };
                assert!(content.declarations[0].citations.is_empty());
                assert!(content.declarations[0].citations_truncated);
                assert!(content.truncated);
                assert_eq!(
                    content.notices[0].attribution.as_deref(),
                    Some("Atlas-reviewed correspondence")
                );
                Ok(())
            })
            .unwrap();
    }
    #[test]
    fn remaster_ambiguous_candidate_overflow_never_selects_a_record() {
        let fixture = crate::test_support::fixture_worker();
        fixture
            .worker
            .with_retrieval(&ReadExecutionContext::uncancellable(), |retrieval| {
                let records = retrieval.list_records(atlas_search::ListRecordsRequest::new(
                    None,
                    atlas_search::SearchPage::new(1, 1)?,
                ))?;
                let record = records.records.into_iter().next().expect("fixture record");
                let (endpoint, truncated) = endpoint(
                    atlas_search::RemasterEndpointResult {
                        label: "Ambiguous old entry".into(),
                        reference: Some("authored old name".into()),
                        resolution: RemasterEndpointResolution::Ambiguous {
                            candidates: (0..11)
                                .map(|_| atlas_search::RemasterEndpointCandidate {
                                    record: record.clone(),
                                    owners: vec![],
                                })
                                .collect(),
                            truncated: false,
                        },
                    },
                    retrieval,
                );
                assert!(truncated);
                let RemasterEndpointResolutionView::Ambiguous {
                    candidates,
                    truncated,
                    message,
                } = endpoint.resolution
                else {
                    panic!("ambiguity retained");
                };
                assert_eq!(candidates.len(), 10);
                assert!(truncated);
                assert!(message.contains("no replacement was selected"));
                Ok(())
            })
            .unwrap();
    }
}
