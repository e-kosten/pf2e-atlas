//! Source-free contextual help. Query execution and stored content retain their owners.
use crate::{AppServiceError, AppServiceResult, AtlasAppService, ReadExecutionContext};
use atlas_app_model::*;
use atlas_domain::{
    DefinitionKey, DefinitionNamespace, QueryCapability, QueryCompare, QueryExpression,
    QueryFieldType, QueryLiteral, QueryPredicate, QuerySetMatch, RecordKey,
};
use atlas_search::{
    AtlasRetrievalService, ReferenceBacking, ReferenceCoverage, ReferenceFilterBinding,
    ReferenceSummary, ReferenceSummaryBacking,
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
            let catalog = r.discover_filter_fields()?;
            for binding in &entry.filter_bindings {
                if matches!(binding.relation.as_str(), "property" | "trait_membership")
                    && let Some(action) = property_action(binding, &catalog)?
                {
                    detail.related_filters.push(action);
                }
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
            for id in &entry.permission.notices {
                let notice = r.reference_notice(id)?.ok_or_else(|| {
                    AppServiceError::new(
                        AppErrorCode::ArtifactIncompatible,
                        format!("Definition notice {id} is unavailable"),
                    )
                })?;
                detail.notices.push(DefinitionNoticeView {
                    id: notice.id,
                    text: notice.text,
                    source_url: notice.source_url,
                });
            }
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

/// Published bindings select compiled fields; they never supply executable SQL.
fn property_action(
    binding: &ReferenceFilterBinding,
    catalog: &QueryCapability,
) -> AppServiceResult<Option<ReferenceFilterActionView>> {
    let path = binding.field.as_deref().ok_or_else(|| {
        AppServiceError::new(
            AppErrorCode::ArtifactIncompatible,
            "Property filter has no field",
        )
    })?;
    let field = catalog
        .fields
        .iter()
        .find(|f| f.scope.is_none() && (f.id == path || f.path == path))
        .ok_or_else(|| {
            AppServiceError::new(
                AppErrorCode::ArtifactIncompatible,
                format!("Property filter field {path} is unavailable"),
            )
        })?;
    if field.field_type == QueryFieldType::Number {
        return Ok(None);
    }
    let value = binding.value.as_ref().ok_or_else(|| {
        AppServiceError::new(
            AppErrorCode::ArtifactIncompatible,
            "Property filter has no value",
        )
    })?;
    let (predicate, label) = match field.field_type {
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
            let predicate = QueryPredicate::new(QueryExpression::Exists {
                collection: field.path.clone(),
                scope_id: "definition_value".into(),
                predicate: Box::new(QueryPredicate::new(QueryExpression::Compare {
                    field: member.path.clone(),
                    op: QueryCompare::Eq,
                    value: QueryLiteral::String(value.clone()),
                })),
            });
            (
                predicate,
                match path {
                    "senses" => "With this sense".to_owned(),
                    "defenses.immunities" => "With this immunity".into(),
                    "defenses.weaknesses" => "With this weakness".into(),
                    "defenses.resistances" => "With this resistance".into(),
                    _ => field.label.clone(),
                },
            )
        }
        QueryFieldType::Set => (
            QueryPredicate::new(QueryExpression::SetMatch {
                field: field.path.clone(),
                op: QuerySetMatch::Includes,
                values: vec![value.clone()],
            }),
            match path {
                "languages" => "With this language".into(),
                "deity.domains.primary" => "With this primary domain".into(),
                "deity.domains.alternate" => "With this alternate domain".into(),
                "traits" => "With this trait".into(),
                "spell.traditions" => "With this tradition".into(),
                _ => field.label.clone(),
            },
        ),
        QueryFieldType::String => (
            QueryPredicate::new(QueryExpression::Compare {
                field: field.path.clone(),
                op: QueryCompare::Eq,
                value: QueryLiteral::String(value.clone()),
            }),
            match path {
                "weapon.group" => "In this weapon group".into(),
                "armor.group" => "In this armor group".into(),
                "body.size" => "With this body size".into(),
                "physical.size" => "With this item size".into(),
                "ancestry.default_size" => "With this ancestry size".into(),
                "weapon.damage.type" => "With this weapon damage".into(),
                "consumable.damage.type" => "With this consumable damage".into(),
                "rarity" => "With this rarity".into(),
                _ => field.label.clone(),
            },
        ),
        QueryFieldType::Boolean | QueryFieldType::Number => {
            return Err(AppServiceError::new(
                AppErrorCode::ArtifactIncompatible,
                format!("Property filter {path} has an incompatible value type"),
            ));
        }
    };
    Ok(Some(ReferenceFilterActionView { label, predicate }))
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
            for summary in r.reference_summaries(keys)?.entries {
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
        let binding = |field: &str, value: &str| ReferenceFilterBinding {
            id: field.into(),
            relation: "property".into(),
            field: Some(field.into()),
            value: Some(value.into()),
        };
        for (field, value, expected_label) in [
            ("languages", "common", "With this language"),
            ("senses", "darkvision", "With this sense"),
            ("defenses.immunities", "fire", "With this immunity"),
            ("defenses.weaknesses", "fire", "With this weakness"),
            ("defenses.resistances", "fire", "With this resistance"),
            ("deity.domains.primary", "air", "With this primary domain"),
            (
                "deity.domains.alternate",
                "air",
                "With this alternate domain",
            ),
            ("traits", "fire", "With this trait"),
            ("weapon.group", "sword", "In this weapon group"),
            ("armor.group", "chain", "In this armor group"),
            ("body.size", "med", "With this body size"),
            ("physical.size", "med", "With this item size"),
            ("ancestry.default_size", "med", "With this ancestry size"),
            ("weapon.damage.type", "fire", "With this weapon damage"),
            (
                "consumable.damage.type",
                "fire",
                "With this consumable damage",
            ),
            ("rarity", "uncommon", "With this rarity"),
            ("spell.traditions", "arcane", "With this tradition"),
        ] {
            let action = property_action(&binding(field, value), &catalog)
                .unwrap()
                .unwrap();
            retrieval.validate_filter(&action.predicate).unwrap();
            assert_eq!(action.label, expected_label);
        }
        assert!(
            property_action(&binding("abilities.str.modifier", "str"), &catalog)
                .unwrap()
                .is_none()
        );
        assert!(property_action(&binding("invented.field", "x"), &catalog).is_err());
    }
}
