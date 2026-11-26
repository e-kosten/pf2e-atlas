use crate::projection::{navigation_fingerprint, relationship_views};
use crate::{AppServiceError, AppServiceResult, AtlasAppService};
use atlas_app_model::{
    AppErrorCode, RecordDetailRequest, RecordDetailView, RecordNavigationView, RecordSelection,
};
use atlas_domain::{QueryPredicate, RecordKey, SourcePassageAddress};
use atlas_record::source_content::SourceContentLocator;
use atlas_record::source_record::{
    SourceContentFormat, SourceContentRole, SourceFieldView, recover_source_passage,
};
use atlas_search::{
    GetRecordRequest, GetRecordsRequest, RecordRefResolutionResult, ResolveRecordRefRequest,
    ResolveRecordRequest, SourceRelationshipDirection, SourceRelationshipRequest,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecordReadSection {
    Rules,
    Defenses,
    Strikes,
    Abilities,
    Spellcasting,
}

impl RecordReadSection {
    pub fn includes_activity(self, kind: &atlas_app_model::ActorActivityKindView) -> bool {
        use atlas_app_model::ActorActivityKindView as Activity;
        matches!(
            (self, kind),
            (Self::Abilities, Activity::Ability)
                | (Self::Strikes, Activity::Strike)
                | (Self::Spellcasting, Activity::Spell | Activity::CastingEntry)
        )
    }
}

#[derive(Clone, Copy)]
pub enum RecordReadDetail {
    Summary,
    Standard { include_grants: bool },
    Full,
}

#[derive(Clone, Copy)]
pub(crate) enum ReadingProjection<'a> {
    Full,
    Standard { include_grants: bool },
    Selected,
    Sections(&'a [RecordReadSection]),
    Table,
    TableMatch,
}
impl ReadingProjection<'_> {
    fn includes_content(self, body: &atlas_app_model::RecordBodyView) -> bool {
        match self {
            Self::Full | Self::Standard { .. } | Self::TableMatch => true,
            Self::Selected | Self::Table => false,
            Self::Sections(sections) => {
                sections.contains(&RecordReadSection::Rules)
                    || matches!(body, atlas_app_model::RecordBodyView::Activity(activity) if sections.iter().any(|section| section.includes_activity(&activity.kind)))
            }
        }
    }
}

impl AtlasAppService {
    /// Explicit source inspection bypasses presentation, using the checked codec.
    /// Product record eligibility still applies; tooling sources stay developer-only.
    pub fn record_source(&self, key: &str) -> AppServiceResult<serde_json::Value> {
        let key = RecordKey::parse(key).map_err(|error| {
            AppServiceError::new(AppErrorCode::InvalidRecordKey, error.to_string())
        })?;
        self.with_retrieval(&crate::ReadExecutionContext::uncancellable(), move |retrieval| {
            let detail = retrieval.get_record(GetRecordRequest {
                record_key: &key,
                selected_content: &[],
            })?.ok_or_else(|| AppServiceError::new(AppErrorCode::RecordNotFound, format!("record `{key}` was not found")))?;
            let snapshot = atlas_foundry_model::snapshot_json_value(detail.source.source())
                .map_err(|error| AppServiceError::new(AppErrorCode::InternalError, error.to_string()))?;
            let mut output = serde_json::json!({
                "key": key,
                "provenance": { "path": detail.summary.source_path, "sha256": detail.summary.content_hash }
            });
            output["checked_snapshot"] = snapshot;
            Ok(output)
        })
    }

    pub fn get_records(
        &self,
        record_keys: Vec<RecordKey>,
    ) -> AppServiceResult<Vec<atlas_app_model::RecordSummaryView>> {
        self.record_summaries_at(record_keys, None)
            .map(|(_, rows)| rows)
    }

    /// Bind body-free identities to the snapshot that supplied them.
    pub fn record_summaries_at(
        &self,
        record_keys: Vec<RecordKey>,
        expected_generation: Option<String>,
    ) -> AppServiceResult<(String, Vec<atlas_app_model::RecordSummaryView>)> {
        self.with_retrieval(&crate::ReadExecutionContext::uncancellable(), move |r| {
            let generation = r.artifact_generation()?;
            crate::definitions::check_generation(expected_generation.as_deref(), &generation)?;
            let summaries = r
                .get_records(GetRecordsRequest {
                    record_keys: &record_keys,
                })?
                .records
                .iter()
                .map(|row| crate::projection::localized_summary_view(row, r))
                .collect();
            Ok((generation, summaries))
        })
    }

    pub fn resolve_record(
        &self,
        query: String,
        filter: Option<QueryPredicate>,
    ) -> AppServiceResult<Vec<atlas_app_model::RecordResolutionCandidateView>> {
        self.resolve_record_with_generation(query, filter)
            .map(|(_, rows)| rows)
    }

    pub fn resolve_record_with_generation(
        &self,
        query: String,
        filter: Option<QueryPredicate>,
    ) -> AppServiceResult<(String, Vec<atlas_app_model::RecordResolutionCandidateView>)> {
        self.with_retrieval(&crate::ReadExecutionContext::uncancellable(), move |r| {
            let matches = r
                .resolve_record(ResolveRecordRequest {
                    query: &query,
                    filter: filter.as_ref(),
                })?
                .into_iter()
                .map(|v| crate::projection::resolution_view(v, r))
                .collect();
            Ok((r.artifact_generation()?, matches))
        })
    }
    pub fn resolve_record_ref(
        &self,
        record_ref: String,
        filter: Option<QueryPredicate>,
        expected_generation: Option<String>,
    ) -> AppServiceResult<atlas_app_model::RecordRefResolutionView> {
        self.with_retrieval(&crate::ReadExecutionContext::uncancellable(), move |r| {
            if let Some(expected) = expected_generation.as_deref() {
                crate::connection_scope::check_generation(
                    Some(expected),
                    &r.artifact_generation()?,
                )?;
            }
            Ok(
                match r.resolve_record_ref(ResolveRecordRefRequest {
                    record_ref: &record_ref,
                    filter: filter.as_ref(),
                })? {
                    RecordRefResolutionResult::Key(k) => {
                        atlas_app_model::RecordRefResolutionView::Key(k)
                    }
                    RecordRefResolutionResult::Miss => {
                        atlas_app_model::RecordRefResolutionView::Miss
                    }
                    RecordRefResolutionResult::Ambiguous(v) => {
                        atlas_app_model::RecordRefResolutionView::Ambiguous(
                            v.into_iter()
                                .map(|v| crate::projection::resolution_view(v, r))
                                .collect(),
                        )
                    }
                },
            )
        })
    }
    pub fn record_detail(&self, key: &str) -> AppServiceResult<RecordDetailView> {
        self.record_detail_at(RecordDetailRequest {
            include_section_rendering: false,
            expected_artifact_generation: None,
            spell_rank: None,
            record_key: key.into(),
            owners: vec![],
            fields: vec![],
            passage: None,
            source_fingerprint: None,
        })
    }
    pub fn record_detail_at(
        &self,
        request: RecordDetailRequest,
    ) -> AppServiceResult<RecordDetailView> {
        self.with_retrieval(&crate::ReadExecutionContext::uncancellable(), move |r| {
            crate::definitions::check_generation(
                request.expected_artifact_generation.as_deref(),
                &r.artifact_generation()?,
            )?;
            let include_section_rendering = request.include_section_rendering;
            read_record_detail(r, request.into_selection(), include_section_rendering)
                .map(|(_, view)| view)
        })
    }
    /// Return a citation's exact checked text alongside the selected reference.
    pub fn record_read_at(
        &self,
        request: RecordDetailRequest,
        detail: RecordReadDetail,
        sections: Vec<RecordReadSection>,
    ) -> AppServiceResult<(RecordDetailView, Option<String>)> {
        self.with_retrieval(&crate::ReadExecutionContext::uncancellable(), move |r| {
            let generation = request.expected_artifact_generation.clone();
            check_retained_location(
                r,
                generation.as_deref(),
                request.source_fingerprint.as_deref(),
            )?;
            let include_section_rendering = request.include_section_rendering;
            let selection = request.into_selection();
            let loaded = LoadedRecordDetail::load(r, &selection)?;
            let projection = if !sections.is_empty() {
                ReadingProjection::Sections(&sections)
            } else if !selection.fields.is_empty() || selection.passage.is_some() {
                ReadingProjection::Selected
            } else {
                match detail {
                    RecordReadDetail::Summary => ReadingProjection::Selected,
                    RecordReadDetail::Standard { include_grants } => {
                        ReadingProjection::Standard { include_grants }
                    }
                    RecordReadDetail::Full => ReadingProjection::Full,
                }
            };
            loaded.project_with_excerpt(selection, projection, include_section_rendering)
        })
    }
}

pub(crate) fn check_retained_location(
    retrieval: &atlas_search::AtlasRetrievalService,
    generation: Option<&str>,
    fingerprint: Option<&str>,
) -> AppServiceResult<()> {
    crate::definitions::check_generation(generation, &retrieval.artifact_generation()?)?;
    if fingerprint.is_some_and(|supplied| supplied != retrieval.source_fingerprint()) {
        return Err(AppServiceError::invalid_request(
            "The selected source fingerprint has changed. Reopen the record or search result.",
        ));
    }
    Ok(())
}

/// Decode the selected parent once, retaining it for workflows that compose runtime facts.
pub(crate) fn read_record_detail(
    r: &atlas_search::AtlasRetrievalService,
    request: RecordSelection,
    include_section_rendering: bool,
) -> AppServiceResult<(atlas_search::SourceRecordDetail, RecordDetailView)> {
    let loaded = LoadedRecordDetail::load(r, &request)?;
    let view = loaded.project(request, include_section_rendering)?;
    Ok((loaded.detail, view))
}

#[derive(Default)]
struct SelectedContent {
    fields: Vec<atlas_app_model::PreparedContentFieldView>,
    relationships: Vec<atlas_search::SourceStoredRelationship>,
    truncated: bool,
    excerpt: Option<String>,
}

pub(crate) struct LoadedRecordDetail<'a> {
    retrieval: &'a atlas_search::AtlasRetrievalService,
    detail: atlas_search::SourceRecordDetail,
}

impl<'a> LoadedRecordDetail<'a> {
    pub(crate) fn load(
        r: &'a atlas_search::AtlasRetrievalService,
        request: &RecordSelection,
    ) -> AppServiceResult<Self> {
        let key = validate_selection(r, request)?;
        let detail = r
            .get_record(GetRecordRequest {
                record_key: &key,
                selected_content: &[],
            })?
            .ok_or_else(|| {
                AppServiceError::new(
                    AppErrorCode::RecordNotFound,
                    format!("record `{key}` was not found"),
                )
            })?;
        Ok(Self {
            retrieval: r,
            detail,
        })
    }

    pub(crate) fn source(&self) -> &atlas_record::source_record::SourceBackedRecord {
        &self.detail.source
    }

    /// Rebuild only the offered interaction under its source and optional live encounter context.
    pub(crate) fn roll_control(
        &self,
        request: &RecordSelection,
        ordinal: usize,
        runtime: Option<crate::prepared_content::RuntimeContentContext>,
    ) -> AppServiceResult<atlas_app_model::ContentControlView> {
        let unavailable = || {
            AppServiceError::invalid_request(
                "This roll expression is unavailable. Reopen the record.",
            )
        };
        let node = self
            .source()
            .node_at(&request.address.owners)
            .ok_or_else(|| {
                AppServiceError::invalid_request(
                    "selected document does not exist in this source snapshot",
                )
            })?;
        let field = node
            .content_selections()
            .into_iter()
            .find(|f| {
                request
                    .fields
                    .first()
                    .is_some_and(|name| name.as_str() == f.field)
            })
            .ok_or_else(|| {
                AppServiceError::invalid_request(
                    "selected field is not declared content of the selected document",
                )
            })?;
        let rank =
            crate::presentation::content_casting_rank(self.source(), node, request.spell_rank)?;
        if !crate::prepared_content::field_visible(
            &field,
            self.retrieval.artifact_context().audience,
        ) || !matches!(
            (field.text, field.format),
            (
                SourceFieldView::Value(_),
                SourceFieldView::Value(SourceContentFormat::Html)
            )
        ) {
            return Err(unavailable());
        }
        let locator = SourceContentLocator {
            record: self.source().key().clone(),
            owners: request.address.owners.clone(),
            field: field.field.to_string(),
        };
        let content = self
            .retrieval
            .read_attributed_content(std::slice::from_ref(&locator))?;
        if content.bounded.contains(&locator) {
            return Err(unavailable());
        }
        let cached = content.content.fields.first().ok_or_else(|| {
            AppServiceError::new(
                AppErrorCode::ArtifactIncompatible,
                "selected rich field has no prepared outcome",
            )
        })?;
        let interaction = cached
            .html
            .as_ref()
            .and_then(|_| {
                cached
                    .interactions
                    .iter()
                    .find(|control| control.ordinal == ordinal)
            })
            .ok_or_else(unavailable)?;
        let mut context = crate::prepared_content::PreparedContentContext::source(
            self.retrieval,
            self.source(),
            &request.address.owners,
            rank,
        );
        if context.formula.item_is_spell {
            context.spell_rank = rank;
            context.formula.item_rank = rank;
        }
        if let Some(runtime) = runtime {
            context.apply_runtime(&runtime, &cached.interactions);
        }
        crate::prepared_content::content_controls(
            std::slice::from_ref(interaction),
            Some((&locator, &context)),
            self.retrieval,
        )?
        .pop()
        .ok_or_else(unavailable)
    }

    fn read_fields(
        &self,
        key: &RecordKey,
        request: &RecordSelection,
        selected: &[atlas_record::source_record::SourceContentSelection<'_>],
        spell_rank: Option<u8>,
        include_section_rendering: bool,
    ) -> AppServiceResult<SelectedContent> {
        let r = self.retrieval;
        let mut excerpt = None;
        let locators: Vec<_> = selected
            .iter()
            .filter(|s| {
                matches!(s.text, SourceFieldView::Value(_))
                    && matches!(s.format, SourceFieldView::Value(SourceContentFormat::Html))
            })
            .map(|s| SourceContentLocator {
                record: key.clone(),
                owners: request.address.owners.clone(),
                field: s.field.to_string(),
            })
            .collect();
        let content = r.read_attributed_content(&locators)?;
        let mut context = crate::prepared_content::PreparedContentContext::source(
            r,
            self.source(),
            &request.address.owners,
            spell_rank,
        )
        .with_section_rendering(include_section_rendering);
        if context.formula.item_is_spell {
            context.formula.item_rank = spell_rank;
            context.spell_rank = spell_rank;
        }
        let mut fields = crate::prepared_content::prepared_fields(
            key,
            &request.address.owners,
            selected,
            &content.content.fields,
            &content.bounded,
            r.artifact_context().audience,
            &context,
        )?;
        if let Some(address) = &request.passage {
            if matches!(address, SourcePassageAddress::Identity {}) {
                if !request.fields.is_empty() {
                    return Err(AppServiceError::invalid_request(
                        "identity navigation cannot select a prose field",
                    ));
                }
            } else {
                if request.fields.len() != 1 {
                    return Err(AppServiceError::invalid_request(
                        "passage navigation requires exactly one selected field",
                    ));
                }
                let field = fields
                    .iter_mut()
                    .find(|f| f.locator.field == request.fields[0])
                    .ok_or_else(|| {
                        AppServiceError::invalid_request("selected passage field is unavailable")
                    })?;
                let (html, plain, interactions) = match &field.body {
                    atlas_app_model::PreparedFieldBodyView::Html { .. } => {
                        // Search passages bind canonical prepared content, before
                        // selected-rank or runtime substitutions change its text.
                        let cached = content
                            .content
                            .fields
                            .iter()
                            .find(|cached| {
                                cached.locator.field == field.locator.field
                                    && cached.locator.owners == field.locator.owners
                            })
                            .ok_or_else(|| {
                                AppServiceError::invalid_request(
                                    "selected passage field is unavailable",
                                )
                            })?;
                        let canonical = cached.html.as_deref().ok_or_else(|| {
                            AppServiceError::invalid_request(
                                "selected passage field is unavailable",
                            )
                        })?;
                        (Some(canonical), None, Some(cached.interactions.as_slice()))
                    }
                    atlas_app_model::PreparedFieldBodyView::Plain { text } => {
                        (None, Some(text.as_str()), None)
                    }
                    _ => {
                        return Err(AppServiceError::invalid_request(
                            "selected passage field is unavailable",
                        ));
                    }
                };
                excerpt = Some(recover_source_passage(html, plain, address).map_err(|e| {
                    AppServiceError::invalid_request(format!("stale or invalid passage: {e}"))
                })?);
                if let (
                    atlas_app_model::PreparedFieldBodyView::Html { html, .. },
                    Some(canonical),
                    Some(interactions),
                ) = (&mut field.body, html, interactions)
                {
                    let marked =
                        atlas_record::source_record::mark_source_html_passage(canonical, address)
                            .map_err(|e| {
                            AppServiceError::invalid_request(format!(
                                "stale or invalid passage: {e}"
                            ))
                        })?;
                    *html = crate::prepared_content::render_field_content(
                        &marked,
                        interactions,
                        &context,
                    );
                    if include_section_rendering {
                        field.section_rendering =
                            crate::prepared_content::contextual_section_rendering(
                                &marked,
                                interactions,
                                &context.formula,
                            )?;
                    }
                    if let Some(rendering) = &mut field.section_rendering {
                        rendering.html = crate::prepared_content::render_field_content(
                            &rendering.html,
                            interactions,
                            &context,
                        );
                    }
                    crate::prepared_content::project_routine_header(field, r)?;
                }
            }
        }
        crate::inline_content::compose_inline_content(
            r,
            fields.iter_mut(),
            &content.relationships,
        )?;
        Ok(SelectedContent {
            fields,
            relationships: content.relationships,
            truncated: !content.bounded.is_empty(),
            excerpt,
        })
    }

    pub(crate) fn project(
        &self,
        request: RecordSelection,
        include_section_rendering: bool,
    ) -> AppServiceResult<RecordDetailView> {
        self.project_with_excerpt(request, ReadingProjection::Full, include_section_rendering)
            .map(|(view, _)| view)
    }

    pub(crate) fn project_with_excerpt(
        &self,
        mut request: RecordSelection,
        projection: ReadingProjection<'_>,
        include_section_rendering: bool,
    ) -> AppServiceResult<(RecordDetailView, Option<String>)> {
        let r = self.retrieval;
        let detail = &self.detail;
        let key = validate_selection(r, &request)?;
        if &key != detail.source.key() {
            return Err(AppServiceError::invalid_request(
                "detail selection belongs to another loaded record",
            ));
        }
        let whole_document = request.fields.is_empty() && request.passage.is_none();
        let node = detail
            .source
            .node_at(&request.address.owners)
            .ok_or_else(|| {
                AppServiceError::invalid_request(
                    "selected document does not exist in this source snapshot",
                )
            })?;
        let selections = node.content_selections();
        if request
            .fields
            .iter()
            .any(|f| !selections.iter().any(|s| s.field == f.as_str()))
        {
            return Err(AppServiceError::invalid_request(
                "selected field is not declared content of the selected document",
            ));
        }
        let selected: Vec<_> = selections
            .into_iter()
            .filter(|s| {
                if request.fields.is_empty() {
                    s.role != SourceContentRole::Name
                        && (matches!(s.text, SourceFieldView::Value(_))
                            || s.role == SourceContentRole::Prerequisite)
                } else {
                    request.fields.iter().any(|f| f.as_str() == s.field)
                }
            })
            .collect();
        if !request.fields.is_empty() {
            request.fields = selected
                .iter()
                .map(|field| field.field.to_string())
                .collect();
        }
        if selected.len() > 256 {
            return Err(AppServiceError::invalid_request(
                "select at most 256 content fields per detail request",
            ));
        }
        let summary = crate::projection::localized_detail_summary_view(detail, r);
        let mut presentation = crate::presentation::record_presentation(
            &detail.source,
            node,
            &summary,
            &request.address.owners,
            match projection {
                ReadingProjection::Sections(sections) => Some(sections),
                _ => None,
            },
            r,
        )?;
        let spell_rank = crate::presentation::apply_casting_rank(
            &detail.source,
            node,
            request.spell_rank,
            &mut presentation,
            r,
        )?;
        if matches!(
            projection,
            ReadingProjection::Full
                | ReadingProjection::Sections(_)
                | ReadingProjection::Standard { .. }
                | ReadingProjection::Selected
        ) {
            crate::presentation::reference_help::annotate(&mut presentation, r)?;
        }
        let include_content = !request.fields.is_empty()
            || request.passage.is_some()
            || projection.includes_content(&presentation.body);
        let content = if include_content {
            self.read_fields(
                &key,
                &request,
                &selected,
                spell_rank,
                include_section_rendering,
            )?
        } else {
            SelectedContent::default()
        };
        presentation.content = content.fields;
        let relationships = if include_content {
            r.read_non_content_relationships(&SourceRelationshipRequest {
                record: key.clone(),
                owners: Some(request.address.owners.clone()),
                fields: request.fields.clone(),
                direction: SourceRelationshipDirection::Outgoing,
                limit: 1024,
            })?
        } else {
            atlas_search::SourceRelationshipBundle {
                occurrences: vec![],
                truncated: false,
            }
        };
        let full = matches!(projection, ReadingProjection::Full);
        let whole_reference = full || matches!(projection, ReadingProjection::Standard { .. });
        let grants_truncated = relationships.truncated;
        let equipment_family = if full && request.address.owners.is_empty() {
            match r.verified_equipment_family_reference(detail) {
                Ok(family) => {
                    family.map(|reference| atlas_app_model::EquipmentFamilyReferenceView {
                        id: reference.id,
                        name: reference.name,
                    })
                }
                Err(error) => {
                    eprintln!("equipment family verification failed for {key}: {error}");
                    None
                }
            }
        } else {
            None
        };
        let creature_family = if full && request.address.owners.is_empty() {
            r.creature_family_for(&detail.summary)?
        } else {
            None
        };
        let creature_overrides = if creature_family.is_some() {
            crate::creature_family::overrides(
                r,
                &detail.summary,
                summary.creature_display.as_ref(),
            )?
        } else {
            Vec::new()
        };
        let title_link = |summary: &atlas_domain::SourceRecordSummary| {
            let record = crate::projection::localized_summary_view(summary, r);
            atlas_app_model::RecordTitleLinkView {
                record_key: record.record_key,
                title: record.title,
            }
        };
        let canonical_record =
            if whole_reference && whole_document && request.address.owners.is_empty() {
                r.canonical_creature_copy(&detail.summary)?
                    .as_ref()
                    .map(title_link)
            } else {
                None
            };
        let named_creature_links = if whole_reference && request.address.owners.is_empty() {
            let links = r.named_creature_links(&detail.summary)?;
            (links.base.is_some() || !links.named.is_empty()).then(|| {
                atlas_app_model::NamedCreatureLinksView {
                    base: links.base.as_ref().map(title_link),
                    named: links.named.iter().map(title_link).collect(),
                }
            })
        } else {
            None
        };
        let (creature_family, creature_link_placement) = match creature_family {
            Some(family) => {
                let declared_member_count = family.declared_member_count;
                let family = crate::creature_family::family_view(r, family);
                let placement = named_creature_links.as_ref().and_then(|links| {
                    crate::creature_family::link_placement(&family, declared_member_count, links)
                });
                (Some(family), placement)
            }
            None => (None, None),
        };
        let mut view = RecordDetailView {
            selected_fields: if request.fields.len() > 1 {
                request.fields.clone()
            } else {
                vec![]
            },
            has_remaster_versions: false,
            canonical_record,
            named_creature_links,
            creature_link_placement,
            creature_family,
            creature_overrides,
            equipment_family,
            remaster_relationships: atlas_app_model::RelatedCategoryView::Empty,
            source_remaster_context: None,
            encounter_adjustment: (full
                && request.address.owners.is_empty()
                && matches!(
                    detail.summary.record_kind.as_value(),
                    Some(atlas_domain::RecordKind::Creature)
                ))
            .then(|| {
                match crate::encounters::mechanics::record_add_variant(&detail.source).0 {
                    atlas_local_state::ParticipantVariant::Normal => {
                        atlas_app_model::EncounterParticipantVariantView::Normal
                    }
                    atlas_local_state::ParticipantVariant::Elite => {
                        atlas_app_model::EncounterParticipantVariantView::Elite
                    }
                    atlas_local_state::ParticipantVariant::Weak => {
                        atlas_app_model::EncounterParticipantVariantView::Weak
                    }
                }
            }),
            artifact_generation: r.artifact_generation()?,
            record: summary,
            presentation,
            selected: RecordNavigationView {
                spell_rank,
                record_key: key.to_string(),
                source_fingerprint: navigation_fingerprint(
                    &request.address.owners,
                    r.source_fingerprint(),
                ),
                owners: request.address.owners,
                field: if request.fields.len() == 1 {
                    Some(request.fields[0].clone())
                } else {
                    None
                },
                passage: request.passage,
            },
            relationships: relationship_views(
                content
                    .relationships
                    .into_iter()
                    .chain(relationships.occurrences)
                    .collect(),
                r.source_fingerprint(),
            )
            .into_iter()
            .map(|mut relationship| {
                if relationship.kind == "CompendiumProvenance"
                    && let atlas_app_model::RecordReferenceDestinationView::ResolvedRecord {
                        target,
                    } = &mut relationship.destination
                {
                    target.spell_rank = spell_rank;
                }
                relationship
            })
            .collect(),
            relationships_truncated: relationships.truncated || content.truncated,
        };
        if whole_reference {
            (view.remaster_relationships, view.source_remaster_context) =
                crate::remaster::reading_context(&view.selected, r);
            view.has_remaster_versions = crate::related::remaster_versions(
                &view.selected,
                &view.remaster_relationships,
                view.source_remaster_context.as_ref(),
            );
        }
        if include_content {
            crate::contextual_references::attach_consumable_spell(
                &mut view,
                &detail.source,
                node,
                r,
            )?;
            crate::contextual_references::attach_ranked_effects(&mut view, node, r)?;
            if whole_document
                && (full
                    || matches!(
                        projection,
                        ReadingProjection::Standard {
                            include_grants: true
                        }
                    ))
            {
                crate::record_grants::attach(&detail.source, &mut view, r, grants_truncated)?;
            }
        }
        Ok((view, content.excerpt))
    }
}

fn validate_selection(
    r: &atlas_search::AtlasRetrievalService,
    request: &RecordSelection,
) -> AppServiceResult<RecordKey> {
    validate_address_bounds(&request.address.owners)?;
    validate_field_bounds(&request.fields)?;
    let key = RecordKey::parse(&request.address.record_key)
        .map_err(|e| AppServiceError::new(AppErrorCode::InvalidRecordKey, e.to_string()))?;
    if !address_fingerprint_matches(
        &request.address.owners,
        request.address.source_fingerprint.as_deref(),
        r.source_fingerprint(),
    ) {
        return Err(AppServiceError::invalid_request(
            "snapshot-local navigation belongs to a different artifact; reopen the search result or parent record",
        ));
    }
    Ok(key)
}
pub(crate) fn address_fingerprint_matches(
    owners: &[atlas_domain::OwnedContentLocator],
    supplied: Option<&str>,
    current: &str,
) -> bool {
    !atlas_domain::OwnedContentLocator::requires_fingerprint(owners) || supplied == Some(current)
}

pub(crate) fn validate_address_bounds(
    owners: &[atlas_domain::OwnedContentLocator],
) -> AppServiceResult<()> {
    if !atlas_domain::OwnedContentLocator::within_address_bounds(owners) {
        return Err(AppServiceError::invalid_request(
            "detail selection exceeds its embedded path or field bounds",
        ));
    }
    Ok(())
}

pub(crate) fn validate_field_bounds(fields: &[String]) -> AppServiceResult<()> {
    if fields.len() > 256 || fields.iter().any(|field| field.len() > 512) {
        return Err(AppServiceError::invalid_request(
            "detail selection exceeds its embedded path or field bounds",
        ));
    }
    Ok(())
}
