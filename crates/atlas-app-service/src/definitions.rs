//! Source-free contextual help. Query execution and stored content retain their owners.
use crate::{AppServiceError, AppServiceResult, AtlasAppService, ReadExecutionContext};
use atlas_app_model::*;
use atlas_domain::{
    DefinitionKey, DefinitionNamespace, QueryCapability, QueryCompare, QueryExpression,
    QueryFieldType, QueryLiteral, QueryPredicate, QuerySetMatch, RecordKey,
};
use atlas_search::{
    AtlasRetrievalService, ReferenceBacking, ReferenceCoverage, ReferenceSummary,
    ReferenceSummaryBacking,
};
use std::collections::{BTreeMap, BTreeSet};

impl AtlasAppService {
    pub fn definition_detail(
        &self,
        request: DefinitionDetailRequest,
        execution: &ReadExecutionContext,
    ) -> AppServiceResult<DefinitionDetailView> {
        execution.check()?;
        let key = request
            .definition_key
            .parse::<DefinitionKey>()
            .map_err(|e| AppServiceError::invalid_request(e.to_string()))?;
        let projection_execution = execution.clone();
        self.with_retrieval(execution, move |r| {
            let generation = r.artifact_generation()?;
            check_generation(request.expected_artifact_generation.as_deref(), &generation)?;
            if r.reference_coverage() == ReferenceCoverage::TraitsOnly
                && key.namespace() != DefinitionNamespace::Trait
            {
                return Err(AppServiceError::new(
                    AppErrorCode::SetupRequired,
                    "Run Atlas setup to prepare these definitions.",
                ));
            }
            let mut detail = DefinitionDetailView {
                artifact_generation: generation,
                definition_key: key.to_string(),
                label: key.identifier().into(),
                help: None,
                content: None,
                publications: vec![],
                notices: vec![],
                related_filters: vec![],
            };
            let Some(entry) = r.reference_definition(&key)? else {
                return Ok(detail);
            };
            detail.label = entry.label.clone();
            detail.help = summary_help(&ReferenceSummary::from(&entry), r.source_fingerprint());
            if detail.help.is_none() {
                return Ok(detail);
            }
            let fields = crate::definition_values::association_fields(key.namespace());
            if !fields.is_empty() {
                let catalog = r.discover_filter_fields()?;
                detail.related_filters = fields
                    .iter()
                    .map(|(field, label)| property_action(field, key.identifier(), label, &catalog))
                    .collect::<AppServiceResult<_>>()?;
            }
            detail.publications = entry
                .provenance
                .publications
                .into_iter()
                .map(|p| DefinitionPublicationView {
                    title: p.title,
                    page: p.page,
                    license: p.license,
                })
                .collect();
            let notices = r.reference_notices(&entry.permission.notices)?;
            if notices.len() != entry.permission.notices.len() {
                return Err(AppServiceError::new(
                    AppErrorCode::ArtifactIncompatible,
                    "A required definition notice is unavailable",
                ));
            }
            detail.notices = notices
                .into_iter()
                .map(|notice| DefinitionNoticeView {
                    id: notice.id,
                    text: notice.text,
                    source_url: notice.source_url,
                })
                .collect();
            projection_execution.check()?;
            if let ReferenceBacking::Prepared { content, .. } = entry.backing {
                detail.content = Some(crate::prepared_content::reference_body(
                    content,
                    r,
                    "reference-definition/v1",
                )?);
            }
            Ok(detail)
        })
    }
}

pub(crate) fn summary_help(
    summary: &ReferenceSummary,
    source_fingerprint: &str,
) -> Option<ReferenceHelpView> {
    Some(ReferenceHelpView {
        definition_key: Some(summary.key.to_string()),
        destination: definition_reading_destination(summary, source_fingerprint)?,
    })
}

pub(crate) fn check_generation(expected: Option<&str>, actual: &str) -> AppServiceResult<()> {
    if let Some(expected) = expected
        && expected != actual
    {
        return Err(AppServiceError::artifact_generation_changed(
            expected,
            actual,
            AppRecoverableAction::ReloadRecord,
        ));
    }
    Ok(())
}

fn definition_reading_destination(
    summary: &ReferenceSummary,
    source_fingerprint: &str,
) -> Option<ReferenceDestinationView> {
    match &summary.backing {
        ReferenceSummaryBacking::Prepared { readable: true } => {
            Some(ReferenceDestinationView::Definition {
                key: summary.key.to_string(),
            })
        }
        ReferenceSummaryBacking::Record { key } => Some(ReferenceDestinationView::Record {
            navigation: record_navigation(key.clone(), vec![], source_fingerprint).into(),
        }),
        ReferenceSummaryBacking::None | ReferenceSummaryBacking::Prepared { readable: false } => {
            None
        }
    }
}

pub(crate) fn record_navigation(
    key: RecordKey,
    owners: Vec<atlas_domain::OwnedContentLocator>,
    source_fingerprint: &str,
) -> RecordNavigationView {
    RecordNavigationView {
        spell_rank: None,
        record_key: key.to_string(),
        source_fingerprint: crate::projection::navigation_fingerprint(&owners, source_fingerprint),
        owners,
        field: None,
        passage: None,
    }
}

fn property_action(
    path: &str,
    value: &str,
    label: &str,
    catalog: &QueryCapability,
) -> AppServiceResult<ReferenceFilterActionView> {
    let field = catalog
        .fields
        .iter()
        .find(|f| f.scope.is_none() && f.id == path)
        .ok_or_else(|| {
            AppServiceError::new(
                AppErrorCode::ArtifactIncompatible,
                format!("Property filter field {path} is unavailable"),
            )
        })?;
    let expression = match field.field_type {
        QueryFieldType::Collection => {
            let member = catalog
                .fields
                .iter()
                .find(|f| f.scope.as_deref() == Some(path) && f.path == "type")
                .ok_or_else(|| {
                    AppServiceError::new(
                        AppErrorCode::ArtifactIncompatible,
                        format!("Property filter member for {path} is unavailable"),
                    )
                })?;
            QueryExpression::Exists {
                collection: field.path.clone(),
                scope_id: "definition_value".into(),
                predicate: Box::new(QueryPredicate::new(QueryExpression::Compare {
                    field: member.path.clone(),
                    op: QueryCompare::Eq,
                    value: QueryLiteral::String(value.into()),
                })),
            }
        }
        QueryFieldType::Set => QueryExpression::SetMatch {
            field: field.path.clone(),
            op: QuerySetMatch::Includes,
            values: vec![value.into()],
        },
        QueryFieldType::String => QueryExpression::Compare {
            field: field.path.clone(),
            op: QueryCompare::Eq,
            value: QueryLiteral::String(value.into()),
        },
        QueryFieldType::Boolean | QueryFieldType::Number => {
            return Err(AppServiceError::new(
                AppErrorCode::ArtifactIncompatible,
                format!("Property filter {path} has an incompatible value type"),
            ));
        }
    };
    Ok(ReferenceFilterActionView {
        label: label.into(),
        predicate: QueryPredicate::new(expression),
    })
}

/// One body-free metadata batch per selected presentation (split only at the
/// reader's fixed 100-key bound), shared by all explicit semantic fact families.
pub(crate) struct HelpContext<'a> {
    r: &'a AtlasRetrievalService,
    summaries: BTreeMap<DefinitionKey, ReferenceSummary>,
}
impl<'a> HelpContext<'a> {
    pub(crate) fn load(
        r: &'a AtlasRetrievalService,
        keys: BTreeSet<DefinitionKey>,
    ) -> AppServiceResult<Self> {
        let coverage = r.reference_coverage();
        let keys = keys
            .into_iter()
            .filter(|k| {
                coverage == ReferenceCoverage::Complete
                    || k.namespace() == DefinitionNamespace::Trait
            })
            .collect::<Vec<_>>();
        let mut summaries = BTreeMap::new();
        for keys in keys.chunks(100) {
            for summary in r.reference_summaries(keys)? {
                summaries.insert(summary.key.clone(), summary);
            }
        }
        Ok(Self { r, summaries })
    }
    pub(crate) fn value(
        &self,
        namespace: DefinitionNamespace,
        value: &mut ReferenceValueView,
    ) -> AppServiceResult<()> {
        let Ok(key) = DefinitionKey::new(namespace, value.value.clone()) else {
            return Ok(());
        };
        if let Some(summary) = self.summaries.get(&key) {
            value.label = summary.label.clone();
            value.help = summary_help(summary, self.r.source_fingerprint());
        }
        Ok(())
    }
    pub(crate) fn keyed(
        &self,
        namespace: DefinitionNamespace,
        value: &str,
        label: &mut String,
        help: &mut Option<ReferenceHelpView>,
    ) -> AppServiceResult<()> {
        let mut reference = ReferenceValueView {
            value: value.into(),
            label: label.clone(),
            help: None,
        };
        self.value(namespace, &mut reference)?;
        *label = reference.label;
        *help = reference.help;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retained_reads_keep_unknown_and_setup_required_distinct_and_guard_generation() {
        let fixture = crate::test_support::fixture_worker();
        let execution = ReadExecutionContext::uncancellable();
        let unknown = fixture
            .worker
            .definition_detail(
                DefinitionDetailRequest {
                    definition_key: "trait:atlas-custom".into(),
                    expected_artifact_generation: None,
                },
                &execution,
            )
            .unwrap();
        assert!(unknown.help.is_none());
        assert!(unknown.content.is_none());
        let error = fixture
            .worker
            .definition_detail(
                DefinitionDetailRequest {
                    definition_key: "language:common".into(),
                    expected_artifact_generation: None,
                },
                &execution,
            )
            .unwrap_err()
            .into_app_error();
        assert_eq!(error.code, AppErrorCode::SetupRequired);
        let error = fixture
            .worker
            .definition_detail(
                DefinitionDetailRequest {
                    definition_key: "language:common".into(),
                    expected_artifact_generation: Some("old-generation".into()),
                },
                &execution,
            )
            .unwrap_err()
            .into_app_error();
        assert_eq!(error.code, AppErrorCode::ArtifactGenerationChanged);
        let actor = fixture
            .worker
            .record_detail("actors:testCreature1000")
            .unwrap();
        let RecordBodyView::Creature(actor) = actor.presentation.body else {
            panic!("actor");
        };
        let skill = actor
            .skills
            .value
            .unwrap()
            .into_iter()
            .find(|s| s.key == "athletics")
            .unwrap();
        assert!(skill.help.is_none());
    }

    #[test]
    fn related_property_actions_are_executable_and_exclude_numeric_fields() {
        let (retrieval, _artifact) = crate::test_support::fixture_retrieval().unwrap();
        let catalog = retrieval.discover_filter_fields().unwrap();
        for (definition, expected_labels) in [
            ("language:common", &["With this language"][..]),
            ("sense:darkvision", &["With this sense"]),
            ("immunity_type:fire", &["With this immunity"]),
            ("weakness_type:fire", &["With this weakness"]),
            ("resistance_type:fire", &["With this resistance"]),
            (
                "domain:air",
                &["With this primary domain", "With this alternate domain"],
            ),
            ("trait:fire", &["With this trait"]),
            ("weapon_group:sword", &["In this weapon group"]),
            ("armor_group:chain", &["In this armor group"]),
            (
                "size:med",
                &[
                    "With this body size",
                    "With this item size",
                    "With this ancestry size",
                ],
            ),
            (
                "damage_type:fire",
                &["With this weapon damage", "With this consumable damage"],
            ),
            ("rarity:uncommon", &["With this rarity"]),
            ("tradition:arcane", &["With this tradition"]),
            ("ability:str", &[]),
        ] {
            let key: DefinitionKey = definition.parse().unwrap();
            let actions = crate::definition_values::association_fields(key.namespace())
                .iter()
                .map(|(field, label)| {
                    property_action(field, key.identifier(), label, &catalog).unwrap()
                })
                .collect::<Vec<_>>();
            for action in &actions {
                retrieval.validate_filter(&action.predicate).unwrap();
            }
            assert_eq!(
                actions
                    .iter()
                    .map(|action| action.label.as_str())
                    .collect::<Vec<_>>(),
                expected_labels
            );
        }
        assert!(property_action("invented.field", "x", "", &catalog).is_err());
    }
}
