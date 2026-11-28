//! Root character-building progression prepared against a complete source inventory.
mod parse;
#[cfg(test)]
mod tests;

use crate::source_content::{PreparedSourceContent, SourceContentLocator, prepared_plain_text};
use crate::source_record::{
    ItemSourceView, SourceBackedRecord, SourceContentOutcome, SourceContentRole,
    SourceContentStatus, SourceNodeView,
};
use atlas_domain::{RecordKey, TraitKey, normalize_record_name};
use atlas_foundry_model::{
    RuleSource, SourceContext, SourcePresence,
    generated::{FeatSystemSourceCategory, JSONValue, SourceFromSchemaAELikeSchemaMode},
};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProgressionRecord {
    /// Separate source clauses are cumulative.
    pub requirements: Vec<PrerequisiteClause>,
    pub ancestry_access: Vec<AncestryFeatAccess>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrerequisiteClause {
    pub locator: SourceContentLocator,
    pub expression: PrerequisiteExpression,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PrerequisiteExpression {
    AllOf {
        children: Vec<Self>,
    },
    AnyOf {
        children: Vec<Self>,
    },
    Record {
        record_key: RecordKey,
        label: String,
        evidence: PrerequisiteEvidence,
        text_range: Option<TextRange>,
    },
    Proficiency {
        subject: ProficiencySubject,
        minimum: ProficiencyRank,
    },
    AttributeMinimum {
        attribute: Attribute,
        minimum: i8,
    },
    Unresolved {
        text: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrerequisiteEvidence {
    AuthoredReference,
    ExactTitle,
    SameArchetypeDedication,
    QualifiedTitle,
}

/// Byte offsets in the unmodified authored plain-text field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextRange {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ProficiencySubject {
    Skill { name: String },
    Lore { name: String },
    Perception,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProficiencyRank {
    Trained,
    Expert,
    Master,
    Legendary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Attribute {
    Strength,
    Dexterity,
    Constitution,
    Intelligence,
    Wisdom,
    Charisma,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AncestryFeatAccess {
    pub trait_key: String,
    pub locator: SourceContentLocator,
}

pub struct PrerequisiteTarget<'a> {
    pub locator: &'a SourceContentLocator,
    pub leaf_path: String,
    pub target_key: &'a RecordKey,
}

impl ProgressionRecord {
    pub fn record_targets(&self) -> Vec<PrerequisiteTarget<'_>> {
        fn visit<'a>(
            expression: &'a PrerequisiteExpression,
            locator: &'a SourceContentLocator,
            path: &mut String,
            output: &mut Vec<PrerequisiteTarget<'a>>,
        ) {
            match expression {
                PrerequisiteExpression::AllOf { children }
                | PrerequisiteExpression::AnyOf { children } => {
                    for (index, child) in children.iter().enumerate() {
                        let length = path.len();
                        path.push_str(&format!("/children/{index}"));
                        visit(child, locator, path, output);
                        path.truncate(length);
                    }
                }
                PrerequisiteExpression::Record { record_key, .. } => {
                    output.push(PrerequisiteTarget {
                        locator,
                        leaf_path: path.clone(),
                        target_key: record_key,
                    });
                }
                _ => {}
            }
        }

        let mut targets = Vec::new();
        for (index, clause) in self.requirements.iter().enumerate() {
            visit(
                &clause.expression,
                &clause.locator,
                &mut format!("/requirements/{index}/expression"),
                &mut targets,
            );
        }
        targets
    }
}

#[derive(Debug)]
pub struct ProgressionCandidate {
    key: RecordKey,
    family: &'static str,
    class_feature: bool,
    name: String,
    archetype: Option<String>,
    dedication: bool,
}

/// Retains compact identity evidence only. Excluded Item titles reserve their
/// entire lookup key because an unavailable body cannot establish its family.
#[derive(Debug, Default)]
pub struct ProgressionIndex {
    candidates: BTreeMap<String, Vec<ProgressionCandidate>>,
    excluded: BTreeSet<String>,
}

impl ProgressionCandidate {
    pub fn from_record(record: &SourceBackedRecord, source_path: &str) -> Option<Self> {
        let SourceNodeView::Item(item) = SourceNodeView::from(record.source()) else {
            return None;
        };
        let family = item.family();
        if !is_prerequisite_family(family) && !matches!(family, "action" | "spell") {
            return None;
        }
        let name = SourceNodeView::Item(item).name().value()?.to_owned();
        let class_feature = item
            .feat_category()
            .value()
            .is_some_and(|category| *category == FeatSystemSourceCategory::Classfeature);
        let dedication = item
            .traits()
            .value()
            .is_some_and(|values| values.iter().any(|trait_key| trait_key == "dedication"));
        Some(Self {
            key: record.key().clone(),
            family,
            class_feature,
            name,
            archetype: archetype_folder(source_path).map(str::to_owned),
            dedication,
        })
    }
}

impl ProgressionIndex {
    pub fn insert(&mut self, record: &SourceBackedRecord, source_path: &str) {
        if let Some(candidate) = ProgressionCandidate::from_record(record, source_path) {
            self.insert_candidate(candidate);
        }
    }

    pub fn insert_candidate(&mut self, candidate: ProgressionCandidate) {
        self.candidates
            .entry(normalize_title(&candidate.name))
            .or_default()
            .push(candidate);
    }

    pub fn exclude_name(&mut self, name: &str) {
        self.excluded.insert(normalize_title(name));
    }

    fn resolve(&self, title: &str, qualification: Qualification) -> Option<&ProgressionCandidate> {
        let title = normalize_title(title);
        if self.excluded.contains(&title) {
            return None;
        }
        let mut candidates = self
            .candidates
            .get(&title)?
            .iter()
            .filter(|candidate| qualification.accepts(candidate));
        let candidate = candidates.next()?;
        (candidates.next().is_none() && is_prerequisite_family(candidate.family))
            .then_some(candidate)
    }
}

fn archetype_folder(path: &str) -> Option<&str> {
    let (_, after) = path.split_once("/feats/archetype/")?;
    let (folder, _) = after.split_once('/')?;
    Some(folder)
}

fn normalize_title(text: &str) -> String {
    normalize_record_name(&text.replace('’', "'"))
}

fn is_prerequisite_family(family: &str) -> bool {
    matches!(
        family,
        "feat" | "heritage" | "ancestry" | "class" | "background" | "deity"
    )
}

#[derive(Clone, Copy)]
enum Qualification {
    Unqualified,
    Family(&'static str),
    ClassFeature,
}

impl Qualification {
    fn accepts(self, candidate: &ProgressionCandidate) -> bool {
        match self {
            Self::Unqualified => true,
            Self::Family(family) => candidate.family == family,
            Self::ClassFeature => candidate.family == "feat" && candidate.class_feature,
        }
    }
}

/// The visible prerequisite text used for interpretation and whole-clause fallback.
/// Standalone validation rebuilds prepared HTML and may omit the cached text.
pub fn clause_text(prepared: &PreparedSourceContent) -> Option<Cow<'_, str>> {
    if prepared.text.is_empty() {
        prepared_plain_text(&prepared.html).ok().map(Cow::Owned)
    } else {
        Some(Cow::Borrowed(prepared.text.as_str()))
    }
}

pub fn prepare(
    record: &SourceBackedRecord,
    source_path: &str,
    index: &ProgressionIndex,
    content: &[SourceContentOutcome],
) -> Option<ProgressionRecord> {
    let SourceNodeView::Item(item) = SourceNodeView::from(record.source()) else {
        return None;
    };
    let mut requirements = Vec::new();
    if item.family() == "feat" {
        for outcome in content {
            if outcome.role != SourceContentRole::Prerequisite
                || !outcome.locator().owners.is_empty()
            {
                continue;
            }
            let SourceContentStatus::Prepared(prepared) = &outcome.status else {
                continue;
            };
            let Some(raw) = record.authored_content_at(&prepared.locator).value() else {
                continue;
            };
            let Some(visible) = clause_text(prepared) else {
                continue;
            };
            if visible.trim().is_empty() {
                continue;
            }
            let parser = parse::ClauseParser {
                source: raw,
                visible: &visible,
                references: &prepared.references,
                owner: record.key(),
                owner_archetype: archetype_folder(source_path),
                index,
            };
            requirements.push(PrerequisiteClause {
                locator: prepared.locator.clone(),
                expression: parser.parse(),
            });
        }
    }
    requirements.sort_by_key(|clause| {
        clause
            .locator
            .field
            .split('/')
            .nth(4)
            .and_then(|index| index.parse::<usize>().ok())
    });
    let ancestry_access = ancestry_feat_access(record.key(), item);
    (!requirements.is_empty() || !ancestry_access.is_empty()).then_some(ProgressionRecord {
        requirements,
        ancestry_access,
    })
}

fn ancestry_feat_access(record: &RecordKey, item: ItemSourceView<'_>) -> Vec<AncestryFeatAccess> {
    if item.family() != "heritage" {
        return Vec::new();
    }
    let Some(rules) = item.rules().value() else {
        return Vec::new();
    };
    let mut output = Vec::new();
    let mut seen = BTreeSet::new();
    for (index, raw) in rules.iter().enumerate() {
        if raw
            .key
            .as_value()
            .is_none_or(|key| key != "ActiveEffectLike")
        {
            continue;
        }
        let field = format!("/system/rules/{index}");
        let Ok(RuleSource::ActiveEffectLike(rule)) = raw.interpret(SourceContext::new(
            record.to_string(),
            "progression",
            &field,
        )) else {
            continue;
        };
        let Some(JSONValue::String(value)) = rule.value.as_value() else {
            continue;
        };
        if !matches!(
            &rule.ignored,
            SourcePresence::Missing | SourcePresence::Value(false)
        ) || !matches!(&rule.predicate, SourcePresence::Missing)
            || !matches!(&rule.phase, SourcePresence::Missing)
            || !matches!(
                &rule.requires_equipped,
                SourcePresence::Missing | SourcePresence::Null | SourcePresence::Value(false)
            )
            || !matches!(
                &rule.requires_investment,
                SourcePresence::Missing | SourcePresence::Null | SourcePresence::Value(false)
            )
            || rule.mode.as_value() != Some(&SourceFromSchemaAELikeSchemaMode::Add)
            || rule
                .path
                .as_value()
                .is_none_or(|path| path != "system.details.ancestry.countsAs")
            || TraitKey::new(value.as_str()).is_err()
            || !value
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        {
            continue;
        }
        if seen.insert(value.clone()) {
            output.push(AncestryFeatAccess {
                trait_key: value.clone(),
                locator: SourceContentLocator {
                    record: record.clone(),
                    owners: Vec::new(),
                    field: format!("{field}/value"),
                },
            });
        }
    }
    output
}
