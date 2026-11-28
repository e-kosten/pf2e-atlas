use crate::{AppServiceError, AppServiceResult, AtlasAppService, ReadExecutionContext};
use atlas_app_model::{
    AppErrorCode, AppRecoverableAction, ProgressionDirection, ProgressionEntryView,
    ProgressionInlineLinkView, ProgressionPageView, ProgressionPoolView, ProgressionRequest,
    ProgressionRequirementKind, ProgressionRequirementView, ProgressionView,
    ProgressionWitnessView, RecordProgressionHintView,
};
use atlas_domain::RecordKey;
use atlas_record::progression::{
    Attribute, PrerequisiteExpression, ProficiencyRank, ProficiencySubject, ProgressionRecord,
};
use atlas_record::source_record::SourceBackedRecord;
use atlas_search::{AtlasRetrievalService, GetRecordsRequest, ProgressionPage};
use std::collections::BTreeMap;

const PAGE_SIZE: usize = 12;

impl AtlasAppService {
    pub fn record_progression(
        &self,
        request: ProgressionRequest,
        execution: &ReadExecutionContext,
    ) -> AppServiceResult<ProgressionView> {
        let selection = request.selection;
        if selection.selects_record_part() || selection.source_fingerprint.is_some() {
            return Err(AppServiceError::invalid_request(
                "Progression requires a whole compendium record.",
            ));
        }
        let key = RecordKey::parse(&selection.record_key).map_err(|error| {
            AppServiceError::new(AppErrorCode::InvalidRecordKey, error.to_string())
        })?;
        let direction = request.direction.unwrap_or_default();
        let offset = request.offset.unwrap_or_default();
        if offset > i64::MAX as usize - PAGE_SIZE
            || (direction == ProgressionDirection::Overview && offset != 0)
            || (direction == ProgressionDirection::Pool) != request.trait_key.is_some()
        {
            return Err(AppServiceError::invalid_request(
                "Invalid progression page.",
            ));
        }
        self.with_retrieval(execution, move |retrieval| {
            let generation = retrieval.artifact_generation()?;
            if request.artifact_generation != generation {
                return Err(AppServiceError::artifact_generation_changed(
                    &request.artifact_generation,
                    &generation,
                    AppRecoverableAction::ReloadRecord,
                ));
            }
            if retrieval
                .get_records(GetRecordsRequest {
                    record_keys: std::slice::from_ref(&key),
                })?
                .records
                .is_empty()
            {
                return Err(AppServiceError::new(
                    AppErrorCode::RecordNotFound,
                    "Progression record not found.",
                ));
            }
            let progression = retrieval.read_progression(&key)?;
            let mut view = ProgressionView {
                selection,
                artifact_generation: generation,
                prerequisites: Vec::new(),
                required_by: ProgressionPageView::default(),
                ancestry_feat_pools: Vec::new(),
                pool: ProgressionPageView::default(),
                available_through: ProgressionPageView::default(),
            };

            if direction == ProgressionDirection::Overview {
                if let Some(progression) = &progression {
                    view.prerequisites = requirement_views(progression, retrieval).0;
                    view.ancestry_feat_pools = progression
                        .ancestry_access
                        .iter()
                        .map(|access| ProgressionPoolView {
                            trait_key: access.trait_key.clone(),
                            label: retrieval
                                .trait_label(&access.trait_key)
                                .unwrap_or(&access.trait_key)
                                .to_owned(),
                        })
                        .collect();
                }
            }
            if matches!(
                direction,
                ProgressionDirection::Overview | ProgressionDirection::RequiredBy
            ) {
                let page = retrieval.read_prerequisite_dependents(&key, offset, PAGE_SIZE)?;
                view.required_by = project_page(
                    retrieval,
                    page.entries
                        .into_iter()
                        .map(|entry| (entry.record_key, entry.leaf_paths))
                        .collect(),
                    offset,
                    page.has_more,
                )?;
            }
            if matches!(
                direction,
                ProgressionDirection::Overview | ProgressionDirection::AvailableThrough
            ) {
                view.available_through = record_page(
                    retrieval,
                    retrieval.read_ancestry_feat_access(&key, offset, PAGE_SIZE)?,
                    offset,
                )?;
            }
            if let Some(trait_key) = request.trait_key {
                if !progression.as_ref().is_some_and(|record| {
                    record
                        .ancestry_access
                        .iter()
                        .any(|access| access.trait_key == trait_key)
                }) {
                    return Err(AppServiceError::invalid_request(
                        "Ancestry feat pool is unavailable.",
                    ));
                }
                view.pool = record_page(
                    retrieval,
                    retrieval.read_ancestry_feat_pool(&trait_key, offset, PAGE_SIZE)?,
                    offset,
                )?;
            }
            Ok(view)
        })
    }
}

pub(crate) fn record_hint(
    retrieval: &AtlasRetrievalService,
    source: &SourceBackedRecord,
) -> AppServiceResult<Option<RecordProgressionHintView>> {
    if !retrieval.has_progression(source.key())? {
        return Ok(None);
    }
    let mut inline_links = Vec::new();
    if let Some(progression) = retrieval.read_progression(source.key())? {
        for clause in &progression.requirements {
            if let Some(text) = source.authored_content_at(&clause.locator).value() {
                collect_inline_links(
                    &clause.expression,
                    &clause.locator.field,
                    text,
                    retrieval,
                    &mut inline_links,
                );
            }
        }
    }
    Ok(Some(RecordProgressionHintView { inline_links }))
}

fn collect_inline_links(
    expression: &PrerequisiteExpression,
    field: &str,
    text: &str,
    retrieval: &AtlasRetrievalService,
    links: &mut Vec<ProgressionInlineLinkView>,
) {
    match expression {
        PrerequisiteExpression::AllOf { children } | PrerequisiteExpression::AnyOf { children } => {
            for child in children {
                collect_inline_links(child, field, text, retrieval, links);
            }
        }
        PrerequisiteExpression::Record {
            record_key,
            text_range: Some(range),
            ..
        } => {
            if let Some(fragment) = text.get(range.start..range.end) {
                links.push(ProgressionInlineLinkView {
                    field: field.to_owned(),
                    text: fragment.to_owned(),
                    navigation: navigation(record_key, retrieval),
                });
            }
        }
        _ => {}
    }
}

fn navigation(
    key: &RecordKey,
    retrieval: &AtlasRetrievalService,
) -> atlas_app_model::RecordNavigationView {
    crate::definitions::record_navigation(key.clone(), Vec::new(), retrieval.source_fingerprint())
}

fn record_page(
    retrieval: &AtlasRetrievalService,
    page: ProgressionPage<RecordKey>,
    offset: usize,
) -> AppServiceResult<ProgressionPageView> {
    project_page(
        retrieval,
        page.entries
            .into_iter()
            .map(|key| (key, Vec::new()))
            .collect(),
        offset,
        page.has_more,
    )
}

fn project_page(
    retrieval: &AtlasRetrievalService,
    members: Vec<(RecordKey, Vec<String>)>,
    offset: usize,
    has_more: bool,
) -> AppServiceResult<ProgressionPageView> {
    if members.is_empty() {
        return Ok(ProgressionPageView::default());
    }
    let keys = members
        .iter()
        .map(|(key, _)| key.clone())
        .collect::<Vec<_>>();
    let summaries = retrieval
        .get_records(GetRecordsRequest { record_keys: &keys })?
        .records
        .into_iter()
        .map(|summary| (summary.key.clone(), summary))
        .collect::<BTreeMap<_, _>>();
    let related = retrieval.related_progression_keys(&keys)?;
    let next_offset = has_more.then_some(offset + members.len());
    let mut entries = Vec::with_capacity(members.len());
    for (key, evidence) in members {
        let summary = summaries.get(&key).ok_or_else(|| {
            atlas_search::SearchError::artifact_contract_violation(
                "Progression target disappeared.",
            )
        })?;
        let (requirements, witnesses) = retrieval
            .read_progression(&key)?
            .as_ref()
            .map(|value| requirement_views(value, retrieval))
            .unwrap_or_default();
        entries.push(ProgressionEntryView {
            navigation: navigation(&key, retrieval),
            title: summary
                .name
                .as_value()
                .cloned()
                .unwrap_or_else(|| "Unnamed record".to_owned()),
            level: summary
                .level
                .as_value()
                .and_then(serde_json::Number::as_u64)
                .and_then(|level| u8::try_from(level).ok()),
            has_related_progression: related.contains(&key),
            requirements,
            witnesses: witnesses
                .into_iter()
                .filter(|witness| evidence.contains(&witness.path))
                .collect(),
        });
    }
    Ok(ProgressionPageView {
        entries,
        next_offset,
    })
}

fn requirement_views(
    record: &ProgressionRecord,
    retrieval: &AtlasRetrievalService,
) -> (Vec<ProgressionRequirementView>, Vec<ProgressionWitnessView>) {
    let mut rendered = RequirementViews {
        retrieval,
        requirements: Vec::new(),
        witnesses: Vec::new(),
    };
    let group_depth = usize::from(record.requirements.len() > 1);
    if group_depth > 0 {
        rendered.requirements.push(ProgressionRequirementView {
            kind: ProgressionRequirementKind::AllOf,
            depth: 0,
            label: "All of".to_owned(),
            path: "/requirements".to_owned(),
            navigation: None,
        });
    }
    for (index, clause) in record.requirements.iter().enumerate() {
        rendered.append(
            &clause.expression,
            &format!("/requirements/{index}/expression"),
            group_depth,
            false,
        );
    }
    (rendered.requirements, rendered.witnesses)
}

struct RequirementViews<'a> {
    retrieval: &'a AtlasRetrievalService,
    requirements: Vec<ProgressionRequirementView>,
    witnesses: Vec<ProgressionWitnessView>,
}

impl RequirementViews<'_> {
    fn append(
        &mut self,
        expression: &PrerequisiteExpression,
        path: &str,
        depth: usize,
        alternative: bool,
    ) {
        use PrerequisiteExpression as Expr;
        use ProgressionRequirementKind as Kind;
        let (kind, label, destination) = match expression {
            Expr::AllOf { .. } => (Kind::AllOf, "All of".to_owned(), None),
            Expr::AnyOf { .. } => (Kind::AnyOf, "Any of".to_owned(), None),
            Expr::Record {
                record_key, label, ..
            } => {
                self.witnesses.push(ProgressionWitnessView {
                    path: path.to_owned(),
                    alternative,
                });
                (
                    Kind::Record,
                    label.clone(),
                    Some(navigation(record_key, self.retrieval)),
                )
            }
            Expr::Proficiency { subject, minimum } => {
                let subject = match subject {
                    ProficiencySubject::Skill { name } | ProficiencySubject::Lore { name } => name,
                    ProficiencySubject::Perception => "Perception",
                };
                let rank = match minimum {
                    ProficiencyRank::Trained => "Trained",
                    ProficiencyRank::Expert => "Expert",
                    ProficiencyRank::Master => "Master",
                    ProficiencyRank::Legendary => "Legendary",
                };
                (Kind::Proficiency, format!("{rank} in {subject}"), None)
            }
            Expr::AttributeMinimum { attribute, minimum } => {
                let attribute = match attribute {
                    Attribute::Strength => "Strength",
                    Attribute::Dexterity => "Dexterity",
                    Attribute::Constitution => "Constitution",
                    Attribute::Intelligence => "Intelligence",
                    Attribute::Wisdom => "Wisdom",
                    Attribute::Charisma => "Charisma",
                };
                (
                    Kind::AttributeMinimum,
                    format!("{attribute} {minimum:+} or higher"),
                    None,
                )
            }
            Expr::Unresolved { text } => (Kind::Unresolved, text.clone(), None),
        };
        self.requirements.push(ProgressionRequirementView {
            kind,
            depth,
            label,
            path: path.to_owned(),
            navigation: destination,
        });
        if let Expr::AllOf { children } | Expr::AnyOf { children } = expression {
            for (index, child) in children.iter().enumerate() {
                self.append(
                    child,
                    &format!("{path}/children/{index}"),
                    depth + 1,
                    alternative || kind == Kind::AnyOf,
                );
            }
        }
    }
}
