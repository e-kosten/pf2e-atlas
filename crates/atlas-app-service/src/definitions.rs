//! Source-free contextual help. Query execution and stored content retain their owners.
use crate::{AppServiceError, AppServiceResult, AtlasAppService, ReadExecutionContext};
use atlas_app_model::*;
use atlas_domain::{
    DefinitionKey, DefinitionNamespace, QueryCapability, QueryCompare, QueryDefinitionReference,
    QueryDefinitionReferenceTarget, QueryExpression, QueryFieldType, QueryLiteral, QueryPredicate,
    QuerySetMatch, RecordKey,
};
use atlas_search::{
    AtlasRetrievalService, DefinitionReferenceSource, DefinitionReferenceTarget,
    ReferenceAvailability, ReferenceBacking, ReferenceCoverage, ReferenceFilterBinding,
    ReferenceSummary, ReferenceSummaryBacking, SearchPage,
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
            let Some(entry) = r.reference_definition(&key)? else {
                return Ok(DefinitionDetailView {
                    artifact_generation: generation,
                    definition_key: key.to_string(),
                    label: key.identifier().into(),
                    help: unavailable_help(&key, DefinitionAvailabilityView::UnavailableUnknown),
                    content: None,
                    attribution: None,
                    notices: vec![],
                    references: vec![],
                    references_availability: DefinitionAvailabilityView::UnavailableUnknown,
                    references_truncated: false,
                });
            };
            let catalog = r.discover_filter_fields()?;
            let help = summary_help(&ReferenceSummary::from(&entry), &catalog, &generation, r)?;
            let mut notices = Vec::with_capacity(entry.permission.notices.len());
            for id in &entry.permission.notices {
                let notice = r.reference_notice(id)?.ok_or_else(|| {
                    AppServiceError::new(
                        AppErrorCode::ArtifactIncompatible,
                        format!("Definition notice {id} is unavailable"),
                    )
                })?;
                notices.push(DefinitionNoticeView {
                    id: notice.id,
                    sha256: notice.sha256,
                    text: notice.text,
                    source_url: notice.source_url,
                });
            }
            let (references, references_truncated, references_availability) =
                if r.reference_coverage() == ReferenceCoverage::Complete {
                    let page = r.definition_references_from_source(
                        &DefinitionReferenceSource::Definition { key: key.clone() },
                        SearchPage::new(1, 100)?,
                    )?;
                    (
                        outgoing_references(page.entries, r)?,
                        page.page.has_more,
                        DefinitionAvailabilityView::Available,
                    )
                } else {
                    (vec![], false, DefinitionAvailabilityView::SetupRequired)
                };
            projection_execution.check()?;
            let attribution = attribution(&entry.provenance, &entry.permission);
            let content = match entry.backing {
                ReferenceBacking::Prepared { content, .. } => Some(prepared_content(content, r)?),
                ReferenceBacking::Record { .. } | ReferenceBacking::None => None,
            };
            Ok(DefinitionDetailView {
                artifact_generation: generation,
                definition_key: key.to_string(),
                label: entry.label,
                help,
                content,
                attribution: Some(attribution),
                notices,
                references,
                references_availability,
                references_truncated,
            })
        })
    }

    pub fn definition_filter_options(
        &self,
        request: DefinitionFilterOptionsRequest,
        execution: &ReadExecutionContext,
    ) -> AppServiceResult<DefinitionFilterOptionsView> {
        execution.check()?;
        let projection_execution = execution.clone();
        self.with_retrieval(execution, move |r| {
            let generation = r.artifact_generation()?;
            check_generation(request.expected_artifact_generation.as_deref(), &generation)?;
            let page =
                r.definition_filter_options(atlas_search::DefinitionFilterOptionsRequest {
                    text: request.text,
                    offset: request.offset,
                    limit: request.limit,
                    selected: request.selected,
                })?;
            let catalog = r.discover_filter_fields()?;
            let options = page
                .options
                .into_iter()
                .map(|o| filter_option(o, &catalog, &generation, r))
                .collect::<AppServiceResult<Vec<_>>>()?;
            let selected = page
                .selected
                .into_iter()
                .map(|selection| selected_option(selection, &catalog, &generation, r))
                .collect::<AppServiceResult<Vec<_>>>()?;
            projection_execution.check()?;
            Ok(DefinitionFilterOptionsView {
                artifact_generation: generation,
                options,
                selected,
                total: page.total,
            })
        })
    }
}

fn selected_option(
    selection: atlas_search::DefinitionFilterSelection,
    catalog: &QueryCapability,
    generation: &str,
    r: &AtlasRetrievalService,
) -> AppServiceResult<DefinitionFilterOptionView> {
    let target = selection.target;
    if let Some(option) = selection.option {
        let mut view = filter_option(option, catalog, generation, r)?;
        view.target = target.clone();
        if matches!(target, QueryDefinitionReferenceTarget::Record { .. }) {
            let condition = ReferenceClauseConditionView::Definition {
                reference: QueryDefinitionReference {
                    target: target.clone(),
                    include_contents: true,
                },
            };
            match crate::relationship_search::clause_summary(r, "selected_definition", &condition) {
                Ok(checked) => {
                    view.label = checked.title;
                    view.help.destination = checked.destination;
                    view.help.related_filters = vec![definition_reference_action(
                        target,
                        generation,
                        "References to this definition",
                    )?];
                }
                Err(error) => {
                    let error = error.into_app_error();
                    match error.code {
                        AppErrorCode::InvalidRequest
                        | AppErrorCode::InvalidRecordKey
                        | AppErrorCode::RecordNotFound => {
                            view.label = "Unavailable record".into();
                            view.help.availability = DefinitionAvailabilityView::MissingTarget;
                            view.help.destination = None;
                            view.help.related_filters.clear();
                        }
                        _ => return Err(error.into()),
                    }
                }
            }
        }
        if selection.availability
            == atlas_search::DefinitionFilterSelectionAvailability::NotInCatalog
            && view.help.availability == DefinitionAvailabilityView::Available
        {
            view.help.availability = DefinitionAvailabilityView::NotInCatalog;
            view.help.destination = None;
            view.help.related_filters.clear();
        }
        if selection.availability
            == atlas_search::DefinitionFilterSelectionAvailability::SourceInapplicable
        {
            view.help.availability = DefinitionAvailabilityView::SourceInapplicable;
            view.help.destination = None;
            view.help.related_filters.clear();
        }
        return Ok(view);
    }
    let (label, key) = match &target {
        QueryDefinitionReferenceTarget::Definition { key } => {
            (key.identifier().to_owned(), Some(key.to_string()))
        }
        QueryDefinitionReferenceTarget::Record { .. } => ("Unavailable record".into(), None),
    };
    Ok(DefinitionFilterOptionView {
        target,
        label,
        help: ReferenceHelpView {
            definition_key: key,
            availability: match selection.availability {
                atlas_search::DefinitionFilterSelectionAvailability::SourceInapplicable => {
                    DefinitionAvailabilityView::SourceInapplicable
                }
                atlas_search::DefinitionFilterSelectionAvailability::Missing => {
                    DefinitionAvailabilityView::MissingTarget
                }
                _ => DefinitionAvailabilityView::UnavailableUnknown,
            },
            destination: None,
            related_filters: vec![],
        },
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

fn filter_option(
    option: atlas_search::DefinitionFilterOption,
    catalog: &QueryCapability,
    generation: &str,
    r: &AtlasRetrievalService,
) -> AppServiceResult<DefinitionFilterOptionView> {
    match option {
        atlas_search::DefinitionFilterOption::Definition { summary } => {
            let target = QueryDefinitionReferenceTarget::Definition {
                key: summary.key.clone(),
            };
            let help = summary_help(&summary, catalog, generation, r)?;
            Ok(DefinitionFilterOptionView {
                target,
                label: summary.label.clone(),
                help,
            })
        }
        atlas_search::DefinitionFilterOption::Record { summary } => {
            let navigation = record_navigation(summary.key.clone(), vec![], r.source_fingerprint());
            let target = QueryDefinitionReferenceTarget::Record {
                selection: Box::new(crate::relationship_search::query_selection(&navigation)?),
            };
            let related_filters = vec![definition_reference_action(
                target.clone(),
                generation,
                "References to this definition",
            )?];
            Ok(DefinitionFilterOptionView {
                target,
                label: summary
                    .name
                    .as_value()
                    .cloned()
                    .unwrap_or_else(|| "Unnamed record".into()),
                help: ReferenceHelpView {
                    definition_key: None,
                    availability: DefinitionAvailabilityView::Available,
                    destination: Some(ReferenceDestinationView::Record {
                        navigation: navigation.into(),
                        reading_context: Some(RecordReadingContextView::GeneralDefinition),
                    }),
                    related_filters,
                },
            })
        }
    }
}

pub(crate) fn unavailable_help(
    key: &DefinitionKey,
    availability: DefinitionAvailabilityView,
) -> ReferenceHelpView {
    ReferenceHelpView {
        definition_key: Some(key.to_string()),
        availability,
        destination: None,
        related_filters: vec![],
    }
}

pub(crate) fn summary_help(
    summary: &ReferenceSummary,
    catalog: &QueryCapability,
    generation: &str,
    r: &AtlasRetrievalService,
) -> AppServiceResult<ReferenceHelpView> {
    let destination = definition_reading_destination(summary, r.source_fingerprint());
    let mut related_filters = Vec::with_capacity(summary.filter_bindings.len() + 1);
    for binding in &summary.filter_bindings {
        match binding.relation.as_str() {
            "references_definition" => related_filters.push(definition_reference_action(
                QueryDefinitionReferenceTarget::Definition {
                    key: summary.key.clone(),
                },
                generation,
                "References to this definition",
            )?),
            // Legacy trait references are symmetric and retain their specialized
            // trait workflow; an incoming-only predicate would change their meaning.
            "definition_reference" if summary.key.namespace() == DefinitionNamespace::Trait => {}
            "property" | "trait_membership" => {
                related_filters.push(property_action(binding, catalog, generation)?)
            }
            _ => {
                return Err(AppServiceError::new(
                    AppErrorCode::ArtifactIncompatible,
                    format!("Unknown definition relation {}", binding.relation),
                ));
            }
        }
    }
    Ok(ReferenceHelpView {
        definition_key: Some(summary.key.to_string()),
        availability: availability(summary.availability),
        destination,
        related_filters,
    })
}

pub(crate) fn definition_reading_destination(
    summary: &ReferenceSummary,
    source_fingerprint: &str,
) -> Option<ReferenceDestinationView> {
    match &summary.backing {
        ReferenceSummaryBacking::Prepared => Some(ReferenceDestinationView::Definition {
            key: summary.key.to_string(),
        }),
        ReferenceSummaryBacking::Record { key } => Some(ReferenceDestinationView::Record {
            navigation: record_navigation(key.clone(), vec![], source_fingerprint).into(),
            reading_context: Some(RecordReadingContextView::GeneralDefinition),
        }),
        ReferenceSummaryBacking::None => None,
    }
}

pub(crate) fn definition_reference_action(
    target: QueryDefinitionReferenceTarget,
    generation: &str,
    label: &str,
) -> AppServiceResult<ReferenceFilterActionView> {
    let mut predicate = QueryPredicate::new(QueryExpression::DefinitionReference {
        reference: QueryDefinitionReference {
            target,
            include_contents: true,
        },
    });
    atlas_domain::assign_relationship_clause_ids(&mut predicate)?;
    Ok(ReferenceFilterActionView::Search {
        label: label.into(),
        predicate,
        artifact_generation: generation.into(),
    })
}

/// Published bindings select compiled fields; they never supply executable SQL.
pub(crate) fn property_action(
    binding: &ReferenceFilterBinding,
    catalog: &QueryCapability,
    generation: &str,
) -> AppServiceResult<ReferenceFilterActionView> {
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
        return Ok(ReferenceFilterActionView::ConfigureField {
            label: field.label.clone(),
            field: field.clone(),
        });
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
    Ok(ReferenceFilterActionView::Search {
        label,
        predicate,
        artifact_generation: generation.into(),
    })
}

fn availability(value: ReferenceAvailability) -> DefinitionAvailabilityView {
    match value {
        ReferenceAvailability::Available => DefinitionAvailabilityView::Available,
        ReferenceAvailability::LabelOnly => DefinitionAvailabilityView::LabelOnly,
        ReferenceAvailability::MissingLocalizedBody => {
            DefinitionAvailabilityView::MissingLocalizedBody
        }
        ReferenceAvailability::MissingTarget => DefinitionAvailabilityView::MissingTarget,
        ReferenceAvailability::WrongFamily => DefinitionAvailabilityView::WrongFamily,
        ReferenceAvailability::ConflictingEvidence => {
            DefinitionAvailabilityView::ConflictingEvidence
        }
        ReferenceAvailability::AmbiguousMapping => DefinitionAvailabilityView::AmbiguousMapping,
        ReferenceAvailability::SourceInapplicable => DefinitionAvailabilityView::SourceInapplicable,
        ReferenceAvailability::PermissionUnresolved => {
            DefinitionAvailabilityView::PermissionUnresolved
        }
        ReferenceAvailability::Withdrawn => DefinitionAvailabilityView::Withdrawn,
        ReferenceAvailability::UnavailableUnknown => DefinitionAvailabilityView::UnavailableUnknown,
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

pub(crate) fn reference_target(
    target: DefinitionReferenceTarget,
    source_fingerprint: &str,
) -> QueryDefinitionReferenceTarget {
    match target {
        DefinitionReferenceTarget::Record { key, owners } => {
            QueryDefinitionReferenceTarget::Record {
                selection: Box::new(atlas_domain::QueryRecordSelection {
                    source_fingerprint: crate::projection::navigation_fingerprint(
                        &owners,
                        source_fingerprint,
                    ),
                    record_key: key,
                    owners,
                    field: None,
                    passage: None,
                    spell_rank: None,
                }),
            }
        }
        DefinitionReferenceTarget::Definition { key } => {
            QueryDefinitionReferenceTarget::Definition { key }
        }
    }
}

pub(crate) fn reference_reading_destination(
    target: &QueryDefinitionReferenceTarget,
    summaries: &BTreeMap<DefinitionKey, ReferenceSummary>,
    r: &AtlasRetrievalService,
) -> Option<ReferenceDestinationView> {
    match target {
        QueryDefinitionReferenceTarget::Record { selection } => {
            Some(crate::relationship_search::record_destination(
                r,
                selection,
                Some(RecordReadingContextView::GeneralDefinition),
            ))
        }
        QueryDefinitionReferenceTarget::Definition { key } => summaries
            .get(key)
            .and_then(|summary| definition_reading_destination(summary, r.source_fingerprint())),
    }
}

fn outgoing_references(
    references: Vec<atlas_search::DefinitionReferenceInput>,
    r: &AtlasRetrievalService,
) -> AppServiceResult<Vec<DefinitionReferenceView>> {
    let references = references
        .into_iter()
        .filter(|reference| reference.visible)
        .collect::<Vec<_>>();
    let keys = references
        .iter()
        .filter_map(|reference| match &reference.target {
            DefinitionReferenceTarget::Definition { key } => Some(key.clone()),
            DefinitionReferenceTarget::Record { .. } => None,
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let mut summaries = BTreeMap::new();
    for chunk in keys.chunks(100) {
        for summary in r.reference_summaries(chunk)?.entries {
            summaries.insert(summary.key.clone(), summary);
        }
    }
    Ok(references
        .into_iter()
        .map(|reference| {
            let target = reference_target(reference.target, r.source_fingerprint());
            let destination = reference_reading_destination(&target, &summaries, r);
            DefinitionReferenceView {
                target,
                destination,
                occurrence_path: reference.occurrence_path,
                label: reference.label,
            }
        })
        .collect())
}

fn prepared_content(
    content: atlas_record::source_content::PreparedReferenceDefinition,
    r: &AtlasRetrievalService,
) -> AppServiceResult<ReferenceContentView> {
    Ok(ReferenceContentView {
        section_rendering: Some(PreparedSectionRenderingView {
            html: atlas_record::source_record::single_section_annotated_prepared_html(
                &content.html,
            ),
            selection_version: "reference-definition/v1".into(),
        }),
        body: PreparedFieldBodyView::Html {
            inclusions: vec![],
            controls: crate::prepared_content::content_controls(&content.interactions, None, r)?,
            html: content.html,
        },
        references: content
            .references
            .into_iter()
            .filter(|reference| reference.visible)
            .map(|reference| {
                let (status, target, url) = crate::projection::content_reference_destination(
                    reference.resolution,
                    r.source_fingerprint(),
                );
                PreparedReferenceView {
                    self_action_reference: None,
                    ordinal: reference.ordinal,
                    status: status.into(),
                    target,
                    url,
                    effect_label: None,
                    contextual_target: None,
                }
            })
            .collect(),
    })
}

fn attribution(
    p: &atlas_search::ReferenceProvenance,
    permission: &atlas_search::ReferencePermission,
) -> DefinitionAttributionView {
    DefinitionAttributionView {
        source_package: p.source_package.clone(),
        source_repository: p.source_repository.clone(),
        source_revision: p.source_revision.clone(),
        source_file: p.source_file.clone(),
        source_sha256: p.source_sha256.clone(),
        label_key: p.label_key.clone(),
        description_key: p.description_key.clone(),
        catalogs: p.catalogs.clone(),
        source_locations: p
            .source_locations
            .iter()
            .map(|s| DefinitionSourceLocationView {
                file: s.file.clone(),
                line: s.line,
                column: s.column,
            })
            .collect(),
        intended_record_key: p.record_mapping.as_ref().map(|m| m.target.to_string()),
        indexing_locale: p.indexing_locale.clone(),
        content_locale: p.content_locale.clone(),
        publications: p
            .publications
            .iter()
            .map(|p| DefinitionPublicationView {
                title: p.title.clone(),
                page: p.page,
                license: p.license.clone(),
            })
            .collect(),
        permission_disposition: match permission.disposition {
            atlas_search::ReferencePermissionDisposition::ExistingTraitPolicy => {
                DefinitionPermissionDispositionView::ExistingTraitPolicy
            }
            atlas_search::ReferencePermissionDisposition::ExistingRecordPolicy => {
                DefinitionPermissionDispositionView::ExistingRecordPolicy
            }
            atlas_search::ReferencePermissionDisposition::ApprovedContent => {
                DefinitionPermissionDispositionView::ApprovedContent
            }
            atlas_search::ReferencePermissionDisposition::MetadataOnlyUnresolved => {
                DefinitionPermissionDispositionView::MetadataOnlyUnresolved
            }
            atlas_search::ReferencePermissionDisposition::ExcludedOrWithdrawn => {
                DefinitionPermissionDispositionView::ExcludedOrWithdrawn
            }
        },
        body_grants: permission.body_grants.iter().map(grant).collect(),
        label_grants: permission.label_grants.iter().map(grant).collect(),
    }
}
fn grant(g: &atlas_search::ReferenceGrantUse) -> DefinitionGrantView {
    DefinitionGrantView {
        license: g.license.clone(),
        scope: match g.scope {
            atlas_search::ReferenceGrantScope::FunctionalContent => {
                DefinitionGrantScopeView::FunctionalContent
            }
            atlas_search::ReferenceGrantScope::DescriptiveReferences => {
                DefinitionGrantScopeView::DescriptiveReferences
            }
        },
    }
}

/// One body-free metadata batch per selected presentation (split only at the
/// reader's fixed 100-key bound), shared by all explicit semantic fact families.
pub(crate) struct HelpContext<'a> {
    r: &'a AtlasRetrievalService,
    summaries: BTreeMap<DefinitionKey, ReferenceSummary>,
    catalog: QueryCapability,
    generation: String,
    coverage: ReferenceCoverage,
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
        Ok(Self {
            r,
            summaries,
            catalog: r.discover_filter_fields()?,
            generation: r.artifact_generation()?,
            coverage,
        })
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
            value.help = Some(summary_help(
                summary,
                &self.catalog,
                &self.generation,
                self.r,
            )?);
        } else {
            value.help = Some(unavailable_help(
                &key,
                if self.coverage == ReferenceCoverage::TraitsOnly
                    && namespace != DefinitionNamespace::Trait
                {
                    DefinitionAvailabilityView::SetupRequired
                } else {
                    DefinitionAvailabilityView::UnavailableUnknown
                },
            ));
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
        assert_eq!(
            unknown.help.availability,
            DefinitionAvailabilityView::UnavailableUnknown
        );
        assert!(unknown.help.destination.is_none());
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
        assert_eq!(
            skill.help.unwrap().availability,
            DefinitionAvailabilityView::SetupRequired
        );
    }

    #[test]
    fn related_property_actions_are_executable_and_numeric_properties_require_input() {
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
            let ReferenceFilterActionView::Search {
                label,
                predicate,
                artifact_generation,
                ..
            } = property_action(&binding(field, value), &catalog, "generation").unwrap()
            else {
                panic!("complete predicate");
            };
            retrieval.validate_filter(&predicate).unwrap();
            assert_eq!(artifact_generation, "generation");
            assert_eq!(label, expected_label);
        }
        let ReferenceFilterActionView::ConfigureField { label, field } = property_action(
            &binding("abilities.str.modifier", "str"),
            &catalog,
            "generation",
        )
        .unwrap() else {
            panic!("numeric field");
        };
        assert_eq!(field.field_type, QueryFieldType::Number);
        assert_eq!(label, field.label);
        assert!(property_action(&binding("invented.field", "x"), &catalog, "generation").is_err());
    }
}
