use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FactPresentationRole {
    Gameplay,
    Provenance,
    Unmodeled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FactPresentationState {
    Missing,
    Null,
    KnownEmpty,
    Known,
    Unsupported,
    Malformed,
    Ambiguous,
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FactRequirement {
    Optional,
    Required,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FactIssueKind {
    RequiredMissing,
    RequiredNull,
    RequiredEmpty,
    Unsupported,
    Malformed,
    Ambiguous,
    Unavailable,
    Unmodeled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FactPresentationDisposition {
    Render,
    Omit,
    Provenance,
    Issue(FactIssueKind),
}

pub fn classify_fact_presentation(
    role: FactPresentationRole,
    state: FactPresentationState,
    requirement: FactRequirement,
) -> FactPresentationDisposition {
    use FactIssueKind as Issue;
    use FactPresentationDisposition as Disposition;
    use FactPresentationRole as Role;
    use FactPresentationState as State;
    use FactRequirement as Requirement;

    match state {
        State::Missing => match requirement {
            Requirement::Optional => Disposition::Omit,
            Requirement::Required => Disposition::Issue(Issue::RequiredMissing),
        },
        State::Null => match requirement {
            Requirement::Optional => Disposition::Omit,
            Requirement::Required => Disposition::Issue(Issue::RequiredNull),
        },
        State::KnownEmpty => match (role, requirement) {
            (Role::Provenance, _) => Disposition::Provenance,
            (_, Requirement::Optional) => Disposition::Omit,
            (_, Requirement::Required) => Disposition::Issue(Issue::RequiredEmpty),
        },
        State::Known => match role {
            Role::Gameplay => Disposition::Render,
            Role::Provenance => Disposition::Provenance,
            Role::Unmodeled => Disposition::Issue(Issue::Unmodeled),
        },
        State::Unsupported => Disposition::Issue(Issue::Unsupported),
        State::Malformed => Disposition::Issue(Issue::Malformed),
        State::Ambiguous => Disposition::Issue(Issue::Ambiguous),
        State::Unavailable => Disposition::Issue(Issue::Unavailable),
    }
}

pub fn classify_spell_fact<T>(
    fact: &crate::SpellFact<T>,
    role: FactPresentationRole,
    requirement: FactRequirement,
) -> FactPresentationDisposition {
    let state = match fact {
        crate::FactValue::Missing => FactPresentationState::Missing,
        crate::FactValue::Null => FactPresentationState::Null,
        crate::FactValue::Value(crate::SpellSourceValue::Known(_)) => FactPresentationState::Known,
        crate::FactValue::Value(crate::SpellSourceValue::Unsupported(_)) => {
            FactPresentationState::Unsupported
        }
    };
    classify_fact_presentation(role, state, requirement)
}

pub fn classify_spell_json_fact<T>(
    fact: &crate::SpellFactJson<T>,
    role: FactPresentationRole,
    requirement: FactRequirement,
) -> FactPresentationDisposition {
    let state = match fact {
        crate::SpellFactJson::Missing => FactPresentationState::Missing,
        crate::SpellFactJson::Null => FactPresentationState::Null,
        crate::SpellFactJson::Known(_) => FactPresentationState::Known,
        crate::SpellFactJson::Unsupported(_) => FactPresentationState::Unsupported,
    };
    classify_fact_presentation(role, state, requirement)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SpellPresentationIssuePlacement {
    Record,
    Classification,
    Casting,
    Targeting,
    Defenses,
    Damage,
    Duration,
    Heightening,
    Ritual,
    Rules,
    Forms,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SpellPresentationIssueField {
    SourceFact,
    ClassificationSourceFact,
    CastingSourceFact,
    TargetingSourceFact,
    DefenseSourceFact,
    DamageSourceFact,
    DurationSourceFact,
    HeighteningSourceFact,
    RitualSourceFact,
    OverlaySourceFact,
    Classification,
    Rank,
    Traits,
    Traditions,
    Casting,
    CastingTime,
    CastingCost,
    CastingRequirements,
    Counteraction,
    Targeting,
    Target,
    Range,
    Area,
    AreaValue,
    AreaType,
    AreaDetails,
    Defense,
    PassiveDefense,
    Save,
    SaveStatistic,
    BasicSave,
    Damage,
    DamageFormula,
    DamageType,
    DamageCategory,
    DamageKinds,
    DamageMaterials,
    DamageApplyModifier,
    Duration,
    DurationValue,
    Sustained,
    Heightening,
    HeighteningInterval,
    HeighteningArea,
    HeighteningDamage,
    FixedRank,
    Ritual,
    PrimaryCheck,
    SecondaryCasters,
    SecondaryChecks,
    Rules,
    DamageDiceSelector,
    DamageDicePredicate,
    DamageDiceNumber,
    DamageDiceSize,
    DamageDiceType,
    DamageDiceVisibility,
    EphemeralEffectPredicate,
    EphemeralEffectSelectors,
    EphemeralEffectTarget,
    DamageAlterationMode,
    DamageAlterationPredicate,
    DamageAlterationProperty,
    DamageAlterationSelectors,
    DamageAlterationSlug,
    DamageAlterationValue,
    RollOptionDomain,
    RollOptionLabel,
    RollOptionValue,
    RollOptionPlacement,
    RollOptionPredicate,
    RollOptionSuboptions,
    RollOptionSuboptionLabel,
    RollOptionSuboptionValue,
    RollOptionToggleable,
    ItemAlterationTarget,
    ItemAlterationMode,
    ItemAlterationPredicate,
    ItemAlterationProperty,
    ItemAlterationValue,
    UnsupportedRule,
    UnsupportedPatch,
    FormCatalog,
    SpellForm,
}

impl SpellPresentationIssueField {
    pub const fn placement(self) -> SpellPresentationIssuePlacement {
        use SpellPresentationIssueField as Field;
        match self {
            Field::SourceFact => SpellPresentationIssuePlacement::Record,
            Field::ClassificationSourceFact => SpellPresentationIssuePlacement::Classification,
            Field::CastingSourceFact => SpellPresentationIssuePlacement::Casting,
            Field::TargetingSourceFact => SpellPresentationIssuePlacement::Targeting,
            Field::DefenseSourceFact => SpellPresentationIssuePlacement::Defenses,
            Field::DamageSourceFact => SpellPresentationIssuePlacement::Damage,
            Field::DurationSourceFact => SpellPresentationIssuePlacement::Duration,
            Field::HeighteningSourceFact => SpellPresentationIssuePlacement::Heightening,
            Field::RitualSourceFact => SpellPresentationIssuePlacement::Ritual,
            Field::OverlaySourceFact | Field::FormCatalog | Field::SpellForm => {
                SpellPresentationIssuePlacement::Forms
            }
            Field::Classification | Field::Rank | Field::Traits | Field::Traditions => {
                SpellPresentationIssuePlacement::Classification
            }
            Field::Casting
            | Field::CastingTime
            | Field::CastingCost
            | Field::CastingRequirements
            | Field::Counteraction => SpellPresentationIssuePlacement::Casting,
            Field::Targeting
            | Field::Target
            | Field::Range
            | Field::Area
            | Field::AreaValue
            | Field::AreaType
            | Field::AreaDetails => SpellPresentationIssuePlacement::Targeting,
            Field::Defense
            | Field::PassiveDefense
            | Field::Save
            | Field::SaveStatistic
            | Field::BasicSave => SpellPresentationIssuePlacement::Defenses,
            Field::Damage
            | Field::DamageFormula
            | Field::DamageType
            | Field::DamageCategory
            | Field::DamageKinds
            | Field::DamageMaterials
            | Field::DamageApplyModifier => SpellPresentationIssuePlacement::Damage,
            Field::Duration | Field::DurationValue | Field::Sustained => {
                SpellPresentationIssuePlacement::Duration
            }
            Field::Heightening
            | Field::HeighteningInterval
            | Field::HeighteningArea
            | Field::HeighteningDamage
            | Field::FixedRank => SpellPresentationIssuePlacement::Heightening,
            Field::Ritual
            | Field::PrimaryCheck
            | Field::SecondaryCasters
            | Field::SecondaryChecks => SpellPresentationIssuePlacement::Ritual,
            Field::UnsupportedPatch => SpellPresentationIssuePlacement::Forms,
            Field::Rules
            | Field::DamageDiceSelector
            | Field::DamageDicePredicate
            | Field::DamageDiceNumber
            | Field::DamageDiceSize
            | Field::DamageDiceType
            | Field::DamageDiceVisibility
            | Field::EphemeralEffectPredicate
            | Field::EphemeralEffectSelectors
            | Field::EphemeralEffectTarget
            | Field::DamageAlterationMode
            | Field::DamageAlterationPredicate
            | Field::DamageAlterationProperty
            | Field::DamageAlterationSelectors
            | Field::DamageAlterationSlug
            | Field::DamageAlterationValue
            | Field::RollOptionDomain
            | Field::RollOptionLabel
            | Field::RollOptionValue
            | Field::RollOptionPlacement
            | Field::RollOptionPredicate
            | Field::RollOptionSuboptions
            | Field::RollOptionSuboptionLabel
            | Field::RollOptionSuboptionValue
            | Field::RollOptionToggleable
            | Field::ItemAlterationTarget
            | Field::ItemAlterationMode
            | Field::ItemAlterationPredicate
            | Field::ItemAlterationProperty
            | Field::ItemAlterationValue
            | Field::UnsupportedRule => SpellPresentationIssuePlacement::Rules,
        }
    }

    pub const fn label(self) -> &'static str {
        use SpellPresentationIssueField as Field;
        match self {
            Field::SourceFact => "Spell source fact",
            Field::ClassificationSourceFact => "Classification source fact",
            Field::CastingSourceFact => "Casting source fact",
            Field::TargetingSourceFact => "Targeting source fact",
            Field::DefenseSourceFact => "Defense source fact",
            Field::DamageSourceFact => "Damage source fact",
            Field::DurationSourceFact => "Duration source fact",
            Field::HeighteningSourceFact => "Heightening source fact",
            Field::RitualSourceFact => "Ritual source fact",
            Field::OverlaySourceFact => "Overlay source fact",
            Field::Classification => "Classification",
            Field::Rank => "Rank",
            Field::Traits => "Traits",
            Field::Traditions => "Traditions",
            Field::Casting => "Casting",
            Field::CastingTime => "Casting time",
            Field::CastingCost => "Casting cost",
            Field::CastingRequirements => "Casting requirements",
            Field::Counteraction => "Counteraction",
            Field::Targeting => "Targeting",
            Field::Target => "Target",
            Field::Range => "Range",
            Field::Area => "Area",
            Field::AreaValue => "Area value",
            Field::AreaType => "Area type",
            Field::AreaDetails => "Area details",
            Field::Defense => "Defense",
            Field::PassiveDefense => "Passive defense",
            Field::Save => "Save",
            Field::SaveStatistic => "Save statistic",
            Field::BasicSave => "Basic save",
            Field::Damage => "Damage",
            Field::DamageFormula => "Damage formula",
            Field::DamageType => "Damage type",
            Field::DamageCategory => "Damage category",
            Field::DamageKinds => "Damage kinds",
            Field::DamageMaterials => "Damage materials",
            Field::DamageApplyModifier => "Damage modifier application",
            Field::Duration => "Duration",
            Field::DurationValue => "Duration value",
            Field::Sustained => "Sustained duration",
            Field::Heightening => "Heightening",
            Field::HeighteningInterval => "Heightening interval",
            Field::HeighteningArea => "Heightening area",
            Field::HeighteningDamage => "Heightening damage",
            Field::FixedRank => "Fixed heightening rank",
            Field::Ritual => "Ritual",
            Field::PrimaryCheck => "Primary ritual check",
            Field::SecondaryCasters => "Secondary casters",
            Field::SecondaryChecks => "Secondary ritual checks",
            Field::Rules => "Rules",
            Field::DamageDiceSelector => "Damage dice selector",
            Field::DamageDicePredicate => "Damage dice predicate",
            Field::DamageDiceNumber => "Damage dice number",
            Field::DamageDiceSize => "Damage die size",
            Field::DamageDiceType => "Damage dice type",
            Field::DamageDiceVisibility => "Damage dice visibility",
            Field::EphemeralEffectPredicate => "Ephemeral effect predicate",
            Field::EphemeralEffectSelectors => "Ephemeral effect selectors",
            Field::EphemeralEffectTarget => "Ephemeral effect target",
            Field::DamageAlterationMode => "Damage alteration mode",
            Field::DamageAlterationPredicate => "Damage alteration predicate",
            Field::DamageAlterationProperty => "Damage alteration property",
            Field::DamageAlterationSelectors => "Damage alteration selectors",
            Field::DamageAlterationSlug => "Damage alteration slug",
            Field::DamageAlterationValue => "Damage alteration value",
            Field::RollOptionDomain => "Roll option domain",
            Field::RollOptionLabel => "Roll option label",
            Field::RollOptionValue => "Roll option value",
            Field::RollOptionPlacement => "Roll option placement",
            Field::RollOptionPredicate => "Roll option predicate",
            Field::RollOptionSuboptions => "Roll option suboptions",
            Field::RollOptionSuboptionLabel => "Roll option suboption label",
            Field::RollOptionSuboptionValue => "Roll option suboption value",
            Field::RollOptionToggleable => "Roll option toggleability",
            Field::ItemAlterationTarget => "Item alteration target",
            Field::ItemAlterationMode => "Item alteration mode",
            Field::ItemAlterationPredicate => "Item alteration predicate",
            Field::ItemAlterationProperty => "Item alteration property",
            Field::ItemAlterationValue => "Item alteration value",
            Field::UnsupportedRule => "Unsupported rule",
            Field::UnsupportedPatch => "Unsupported spell patch",
            Field::FormCatalog => "Spell form catalog",
            Field::SpellForm => "Spell form",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SpellPresentationIssue {
    pub field: SpellPresentationIssueField,
    pub kind: FactIssueKind,
}

impl SpellPresentationIssue {
    pub const fn placement(self) -> SpellPresentationIssuePlacement {
        self.field.placement()
    }

    pub fn message(self) -> String {
        let label = self.field.label();
        match self.kind {
            FactIssueKind::RequiredMissing
            | FactIssueKind::RequiredNull
            | FactIssueKind::RequiredEmpty => format!("{label} is required but unavailable."),
            FactIssueKind::Unsupported => format!("{label} is not supported for presentation."),
            FactIssueKind::Malformed => format!("{label} is malformed."),
            FactIssueKind::Ambiguous => format!("{label} is ambiguous."),
            FactIssueKind::Unavailable => format!("{label} is unavailable."),
            FactIssueKind::Unmodeled => format!("{label} is retained but not modeled."),
        }
    }
}

pub fn project_spell_presentation_issues(
    spell: &crate::SpellRecord,
) -> Vec<SpellPresentationIssue> {
    let json = crate::json_projection::spell_issue_presentation(spell);
    project_spell_json_presentation_issues(&json)
}

pub fn project_spell_form_result_presentation_issues(
    result: &Result<crate::ResolvedSpellForm, crate::SpellFormSelectionError>,
) -> Vec<SpellPresentationIssue> {
    let mut visitor = SpellIssueVisitor::default();
    match result {
        Ok(resolved) => {
            let json = crate::json_projection::resolved_spell_issue_presentation(resolved);
            visitor.resolved_definition(&json);
        }
        Err(_) => visitor.issue(
            SpellPresentationIssueField::SpellForm,
            FactIssueKind::Unavailable,
        ),
    }
    visitor.finish()
}

/// Produces a compact human label from an already-resolved spell form.
///
/// Opaque form identity and authored patch keys deliberately do not participate.
pub fn project_resolved_spell_form_label(resolved: &crate::ResolvedSpellForm) -> Option<String> {
    let casting = resolved_spell_known(&resolved.casting)
        .and_then(|casting| spell_known(&casting.time))
        .filter(|value| !value.trim().is_empty())
        .map(|value| match value.trim() {
            "1" => "1 action".to_string(),
            "2" => "2 actions".to_string(),
            "3" => "3 actions".to_string(),
            value => value.to_string(),
        });
    let target = resolved_spell_known(&resolved.targeting).and_then(|targeting| {
        spell_known(&targeting.area)
            .map(|area| {
                [
                    spell_known(&area.value).map(|value| format!("{value}-foot")),
                    spell_known(&area.area_type).map(|value| value.as_str().to_string()),
                    spell_known(&area.details)
                        .filter(|value| !value.trim().is_empty())
                        .cloned(),
                ]
                .into_iter()
                .flatten()
                .collect::<Vec<_>>()
                .join(" ")
            })
            .filter(|value| !value.is_empty())
            .or_else(|| {
                spell_known(&targeting.range)
                    .map(|range| range.authored_text.trim())
                    .filter(|value| !value.is_empty())
                    .map(str::to_string)
            })
            .or_else(|| {
                spell_known(&targeting.target)
                    .map(String::as_str)
                    .filter(|value| !value.trim().is_empty())
                    .map(str::to_string)
            })
    });
    let parts = [casting, target].into_iter().flatten().collect::<Vec<_>>();
    (!parts.is_empty()).then(|| parts.join(" — "))
}

fn resolved_spell_known<T>(field: &crate::SpellResolvedField<crate::SpellFact<T>>) -> Option<&T> {
    match field {
        crate::SpellResolvedField::Available(fact) => spell_known(fact),
        crate::SpellResolvedField::Unavailable(_) => None,
    }
}

fn spell_known<T>(fact: &crate::SpellFact<T>) -> Option<&T> {
    match fact {
        crate::FactValue::Value(crate::SpellSourceValue::Known(value)) => Some(value),
        crate::FactValue::Missing
        | crate::FactValue::Null
        | crate::FactValue::Value(crate::SpellSourceValue::Unsupported(_)) => None,
    }
}

pub fn merge_spell_presentation_issues(
    primary: impl IntoIterator<Item = SpellPresentationIssue>,
    additional: impl IntoIterator<Item = SpellPresentationIssue>,
) -> Vec<SpellPresentationIssue> {
    let mut visitor = SpellIssueVisitor::default();
    for issue in primary.into_iter().chain(additional) {
        visitor.issue(issue.field, issue.kind);
    }
    visitor.finish()
}

pub fn project_spell_json_presentation_issues(
    spell: &crate::SpellJson,
) -> Vec<SpellPresentationIssue> {
    let mut visitor = SpellIssueVisitor::default();
    visitor.definition(
        &spell.classification,
        &spell.casting,
        &spell.targeting,
        &spell.defense,
        &spell.damage,
        &spell.duration,
        &spell.heightening,
        &spell.ritual,
        &spell.rules,
    );
    for issue in &spell.presentation_issues {
        visitor.issue(issue.field, issue.kind);
    }
    if spell.form_catalog_unavailable.is_some() {
        visitor.issue(
            SpellPresentationIssueField::FormCatalog,
            FactIssueKind::Unavailable,
        );
    }
    for form in &spell.forms {
        match &form.result {
            crate::SpellFormResultJson::Available { definition } => {
                visitor.resolved_definition(definition);
            }
            crate::SpellFormResultJson::Unavailable { .. } => visitor.issue(
                SpellPresentationIssueField::SpellForm,
                FactIssueKind::Unavailable,
            ),
        }
    }
    visitor.finish()
}

pub(crate) fn spell_source_note_issues(spell: &crate::SpellRecord) -> Vec<SpellPresentationIssue> {
    let mut visitor = SpellIssueVisitor::default();
    for note in &spell.definition.unsupported_notes {
        let field = match note.field {
            crate::SpellUnsupportedSourceField::ClassificationMember
            | crate::SpellUnsupportedSourceField::LegacyTraitSelection => {
                SpellPresentationIssueField::ClassificationSourceFact
            }
            crate::SpellUnsupportedSourceField::CastingMember => {
                SpellPresentationIssueField::CastingSourceFact
            }
            crate::SpellUnsupportedSourceField::TargetingMember
            | crate::SpellUnsupportedSourceField::AreaMember => {
                SpellPresentationIssueField::TargetingSourceFact
            }
            crate::SpellUnsupportedSourceField::DefenseMember => {
                SpellPresentationIssueField::DefenseSourceFact
            }
            crate::SpellUnsupportedSourceField::DamageMember => {
                SpellPresentationIssueField::DamageSourceFact
            }
            crate::SpellUnsupportedSourceField::DurationMember => {
                SpellPresentationIssueField::DurationSourceFact
            }
            crate::SpellUnsupportedSourceField::HeighteningMember => {
                SpellPresentationIssueField::HeighteningSourceFact
            }
            crate::SpellUnsupportedSourceField::RitualMember => {
                SpellPresentationIssueField::RitualSourceFact
            }
            crate::SpellUnsupportedSourceField::OverlayMember => {
                SpellPresentationIssueField::OverlaySourceFact
            }
            crate::SpellUnsupportedSourceField::SystemMember
            | crate::SpellUnsupportedSourceField::ProvenanceMember
            | crate::SpellUnsupportedSourceField::LocationMember => {
                SpellPresentationIssueField::SourceFact
            }
        };
        visitor.issue(field, FactIssueKind::Unmodeled);
    }
    visitor.finish()
}

#[derive(Default)]
struct SpellIssueVisitor {
    issues: BTreeMap<SpellPresentationIssueField, FactIssueKind>,
}

impl SpellIssueVisitor {
    #[allow(clippy::too_many_arguments)]
    fn definition(
        &mut self,
        classification: &crate::SpellFactJson<crate::SpellClassificationJson>,
        casting: &crate::SpellFactJson<crate::SpellCastingJson>,
        targeting: &crate::SpellFactJson<crate::SpellTargetingJson>,
        defense: &crate::SpellFactJson<crate::SpellDefenseJson>,
        damage: &crate::SpellFactJson<Vec<crate::SpellDamageJson>>,
        duration: &crate::SpellFactJson<crate::SpellDurationJson>,
        heightening: &crate::SpellFactJson<crate::SpellHeighteningJson>,
        ritual: &crate::SpellFactJson<crate::SpellRitualJson>,
        rules: &crate::SpellFactJson<Vec<crate::SpellRuleJson>>,
    ) {
        self.classification(classification, FactRequirement::Required);
        self.casting(casting);
        self.targeting(targeting);
        self.defense(defense);
        self.damage(damage, FactRequirement::Required);
        self.duration(duration);
        self.heightening(heightening);
        self.ritual(ritual);
        self.rules(rules);
    }

    fn resolved_definition(&mut self, definition: &crate::SpellResolvedDefinitionJson) {
        match &definition.classification {
            crate::SpellResolvedFieldJson::Available { value } => {
                self.classification(value, FactRequirement::Required);
            }
            crate::SpellResolvedFieldJson::Unavailable { .. } => self.issue(
                SpellPresentationIssueField::Classification,
                FactIssueKind::Unavailable,
            ),
        }
        match &definition.casting {
            crate::SpellResolvedFieldJson::Available { value } => self.casting(value),
            crate::SpellResolvedFieldJson::Unavailable { .. } => self.issue(
                SpellPresentationIssueField::Casting,
                FactIssueKind::Unavailable,
            ),
        }
        match &definition.targeting {
            crate::SpellResolvedFieldJson::Available { value } => self.targeting(value),
            crate::SpellResolvedFieldJson::Unavailable { .. } => self.issue(
                SpellPresentationIssueField::Targeting,
                FactIssueKind::Unavailable,
            ),
        }
        match &definition.defense {
            crate::SpellResolvedFieldJson::Available { value } => self.defense(value),
            crate::SpellResolvedFieldJson::Unavailable { .. } => self.issue(
                SpellPresentationIssueField::Defense,
                FactIssueKind::Unavailable,
            ),
        }
        match &definition.damage {
            crate::SpellResolvedFieldJson::Available { value } => {
                self.damage(value, FactRequirement::Required);
            }
            crate::SpellResolvedFieldJson::Unavailable { .. } => self.issue(
                SpellPresentationIssueField::Damage,
                FactIssueKind::Unavailable,
            ),
        }
        match &definition.duration {
            crate::SpellResolvedFieldJson::Available { value } => self.duration(value),
            crate::SpellResolvedFieldJson::Unavailable { .. } => self.issue(
                SpellPresentationIssueField::Duration,
                FactIssueKind::Unavailable,
            ),
        }
        match &definition.heightening {
            crate::SpellResolvedFieldJson::Available { value } => self.heightening(value),
            crate::SpellResolvedFieldJson::Unavailable { .. } => self.issue(
                SpellPresentationIssueField::Heightening,
                FactIssueKind::Unavailable,
            ),
        }
        match &definition.rules {
            crate::SpellResolvedFieldJson::Available { value } => self.rules(value),
            crate::SpellResolvedFieldJson::Unavailable { .. } => self.issue(
                SpellPresentationIssueField::Rules,
                FactIssueKind::Unavailable,
            ),
        }
    }

    fn classification(
        &mut self,
        fact: &crate::SpellFactJson<crate::SpellClassificationJson>,
        rank_requirement: FactRequirement,
    ) {
        let Some(value) = self.fact(
            fact,
            SpellPresentationIssueField::Classification,
            FactRequirement::Optional,
            never_empty,
        ) else {
            return;
        };
        self.fact(
            &value.rank,
            SpellPresentationIssueField::Rank,
            rank_requirement,
            never_empty,
        );
        self.fact(
            &value.traits,
            SpellPresentationIssueField::Traits,
            FactRequirement::Optional,
            Vec::is_empty,
        );
        self.fact(
            &value.traditions,
            SpellPresentationIssueField::Traditions,
            FactRequirement::Optional,
            Vec::is_empty,
        );
    }

    fn casting(&mut self, fact: &crate::SpellFactJson<crate::SpellCastingJson>) {
        let Some(value) = self.fact(
            fact,
            SpellPresentationIssueField::Casting,
            FactRequirement::Optional,
            never_empty,
        ) else {
            return;
        };
        for (fact, field) in [
            (&value.time, SpellPresentationIssueField::CastingTime),
            (&value.cost, SpellPresentationIssueField::CastingCost),
            (
                &value.requirements,
                SpellPresentationIssueField::CastingRequirements,
            ),
        ] {
            self.fact(fact, field, FactRequirement::Optional, String::is_empty);
        }
        self.fact(
            &value.counteraction,
            SpellPresentationIssueField::Counteraction,
            FactRequirement::Optional,
            never_empty,
        );
    }

    fn targeting(&mut self, fact: &crate::SpellFactJson<crate::SpellTargetingJson>) {
        let Some(value) = self.fact(
            fact,
            SpellPresentationIssueField::Targeting,
            FactRequirement::Optional,
            never_empty,
        ) else {
            return;
        };
        self.fact(
            &value.target,
            SpellPresentationIssueField::Target,
            FactRequirement::Optional,
            String::is_empty,
        );
        self.fact(
            &value.range,
            SpellPresentationIssueField::Range,
            FactRequirement::Optional,
            never_empty,
        );
        let Some(area) = self.fact(
            &value.area,
            SpellPresentationIssueField::Area,
            FactRequirement::Optional,
            never_empty,
        ) else {
            return;
        };
        self.fact(
            &area.value,
            SpellPresentationIssueField::AreaValue,
            FactRequirement::Required,
            never_empty,
        );
        self.fact(
            &area.area_type,
            SpellPresentationIssueField::AreaType,
            FactRequirement::Required,
            String::is_empty,
        );
        self.fact(
            &area.details,
            SpellPresentationIssueField::AreaDetails,
            FactRequirement::Optional,
            String::is_empty,
        );
    }

    fn defense(&mut self, fact: &crate::SpellFactJson<crate::SpellDefenseJson>) {
        let Some(value) = self.fact(
            fact,
            SpellPresentationIssueField::Defense,
            FactRequirement::Optional,
            never_empty,
        ) else {
            return;
        };
        self.fact(
            &value.passive,
            SpellPresentationIssueField::PassiveDefense,
            FactRequirement::Optional,
            String::is_empty,
        );
        let Some(save) = self.fact(
            &value.save,
            SpellPresentationIssueField::Save,
            FactRequirement::Optional,
            never_empty,
        ) else {
            return;
        };
        self.fact(
            &save.statistic,
            SpellPresentationIssueField::SaveStatistic,
            FactRequirement::Required,
            String::is_empty,
        );
        self.fact(
            &save.basic,
            SpellPresentationIssueField::BasicSave,
            FactRequirement::Optional,
            never_empty,
        );
    }

    fn damage(
        &mut self,
        fact: &crate::SpellFactJson<Vec<crate::SpellDamageJson>>,
        member_requirement: FactRequirement,
    ) {
        let Some(values) = self.fact(
            fact,
            SpellPresentationIssueField::Damage,
            FactRequirement::Optional,
            Vec::is_empty,
        ) else {
            return;
        };
        for value in values {
            self.damage_member(value, member_requirement);
        }
    }

    fn damage_member(
        &mut self,
        value: &crate::SpellDamageJson,
        member_requirement: FactRequirement,
    ) {
        for (fact, field, requirement) in [
            (
                &value.formula,
                SpellPresentationIssueField::DamageFormula,
                member_requirement,
            ),
            (
                &value.damage_type,
                SpellPresentationIssueField::DamageType,
                member_requirement,
            ),
            (
                &value.category,
                SpellPresentationIssueField::DamageCategory,
                FactRequirement::Optional,
            ),
        ] {
            self.fact(fact, field, requirement, String::is_empty);
        }
        self.fact(
            &value.kinds,
            SpellPresentationIssueField::DamageKinds,
            FactRequirement::Optional,
            Vec::is_empty,
        );
        self.fact(
            &value.materials,
            SpellPresentationIssueField::DamageMaterials,
            FactRequirement::Optional,
            Vec::is_empty,
        );
        self.fact(
            &value.apply_modifier,
            SpellPresentationIssueField::DamageApplyModifier,
            FactRequirement::Optional,
            never_empty,
        );
    }

    fn duration(&mut self, fact: &crate::SpellFactJson<crate::SpellDurationJson>) {
        let Some(value) = self.fact(
            fact,
            SpellPresentationIssueField::Duration,
            FactRequirement::Optional,
            never_empty,
        ) else {
            return;
        };
        self.fact(
            &value.value,
            SpellPresentationIssueField::DurationValue,
            FactRequirement::Optional,
            String::is_empty,
        );
        self.fact(
            &value.sustained,
            SpellPresentationIssueField::Sustained,
            FactRequirement::Optional,
            never_empty,
        );
    }

    fn heightening(&mut self, fact: &crate::SpellFactJson<crate::SpellHeighteningJson>) {
        let Some(value) = self.fact(
            fact,
            SpellPresentationIssueField::Heightening,
            FactRequirement::Optional,
            never_empty,
        ) else {
            return;
        };
        match value {
            crate::SpellHeighteningJson::Interval {
                interval,
                area,
                damage,
            } => {
                self.fact(
                    interval,
                    SpellPresentationIssueField::HeighteningInterval,
                    FactRequirement::Required,
                    never_empty,
                );
                self.fact(
                    area,
                    SpellPresentationIssueField::HeighteningArea,
                    FactRequirement::Optional,
                    never_empty,
                );
                self.fact(
                    damage,
                    SpellPresentationIssueField::HeighteningDamage,
                    FactRequirement::Optional,
                    Vec::is_empty,
                );
            }
            crate::SpellHeighteningJson::Fixed { layers } => {
                for layer in layers {
                    self.fact(
                        &layer.rank,
                        SpellPresentationIssueField::FixedRank,
                        FactRequirement::Required,
                        never_empty,
                    );
                    self.patch(&layer.patch);
                }
            }
        }
    }

    fn ritual(&mut self, fact: &crate::SpellFactJson<crate::SpellRitualJson>) {
        let Some(value) = self.fact(
            fact,
            SpellPresentationIssueField::Ritual,
            FactRequirement::Optional,
            never_empty,
        ) else {
            return;
        };
        self.fact(
            &value.primary_check,
            SpellPresentationIssueField::PrimaryCheck,
            FactRequirement::Required,
            String::is_empty,
        );
        self.fact(
            &value.secondary_casters,
            SpellPresentationIssueField::SecondaryCasters,
            FactRequirement::Optional,
            never_empty,
        );
        self.fact(
            &value.secondary_checks,
            SpellPresentationIssueField::SecondaryChecks,
            FactRequirement::Optional,
            String::is_empty,
        );
    }

    fn rules(&mut self, fact: &crate::SpellFactJson<Vec<crate::SpellRuleJson>>) {
        let Some(values) = self.fact(
            fact,
            SpellPresentationIssueField::Rules,
            FactRequirement::Optional,
            Vec::is_empty,
        ) else {
            return;
        };
        for value in values {
            self.rule(&value.rule);
        }
    }

    fn rule(&mut self, rule: &crate::SpellRuleDetailJson) {
        use crate::SpellRuleDetailJson as Rule;
        match rule {
            Rule::DamageDice(value) => {
                self.required_string(
                    &value.selector,
                    SpellPresentationIssueField::DamageDiceSelector,
                );
                self.predicates(
                    &value.predicate,
                    SpellPresentationIssueField::DamageDicePredicate,
                );
                self.required_string(
                    &value.dice_number,
                    SpellPresentationIssueField::DamageDiceNumber,
                );
                self.required_string(&value.die_size, SpellPresentationIssueField::DamageDiceSize);
                self.required_string(
                    &value.damage_type,
                    SpellPresentationIssueField::DamageDiceType,
                );
                self.fact(
                    &value.hide_if_disabled,
                    SpellPresentationIssueField::DamageDiceVisibility,
                    FactRequirement::Optional,
                    never_empty,
                );
            }
            Rule::EphemeralEffect(value) => {
                self.predicates(
                    &value.predicate,
                    SpellPresentationIssueField::EphemeralEffectPredicate,
                );
                self.fact(
                    &value.selectors,
                    SpellPresentationIssueField::EphemeralEffectSelectors,
                    FactRequirement::Required,
                    Vec::is_empty,
                );
                self.required_string(
                    &value.uuid,
                    SpellPresentationIssueField::EphemeralEffectTarget,
                );
            }
            Rule::DamageAlteration(value) => {
                self.required_string(
                    &value.mode,
                    SpellPresentationIssueField::DamageAlterationMode,
                );
                self.predicates(
                    &value.predicate,
                    SpellPresentationIssueField::DamageAlterationPredicate,
                );
                self.required_string(
                    &value.property,
                    SpellPresentationIssueField::DamageAlterationProperty,
                );
                self.fact(
                    &value.selectors,
                    SpellPresentationIssueField::DamageAlterationSelectors,
                    FactRequirement::Optional,
                    Vec::is_empty,
                );
                self.fact(
                    &value.slug,
                    SpellPresentationIssueField::DamageAlterationSlug,
                    FactRequirement::Optional,
                    String::is_empty,
                );
                self.required_string(
                    &value.value,
                    SpellPresentationIssueField::DamageAlterationValue,
                );
            }
            Rule::RollOption(value) => {
                self.required_string(&value.domain, SpellPresentationIssueField::RollOptionDomain);
                self.fact(
                    &value.label,
                    SpellPresentationIssueField::RollOptionLabel,
                    FactRequirement::Optional,
                    String::is_empty,
                );
                self.required_string(&value.option, SpellPresentationIssueField::RollOptionValue);
                self.required_string(
                    &value.placement,
                    SpellPresentationIssueField::RollOptionPlacement,
                );
                self.predicates(
                    &value.predicate,
                    SpellPresentationIssueField::RollOptionPredicate,
                );
                if let Some(suboptions) = self.fact(
                    &value.suboptions,
                    SpellPresentationIssueField::RollOptionSuboptions,
                    FactRequirement::Optional,
                    Vec::is_empty,
                ) {
                    for suboption in suboptions {
                        self.required_string(
                            &suboption.label,
                            SpellPresentationIssueField::RollOptionSuboptionLabel,
                        );
                        self.required_string(
                            &suboption.value,
                            SpellPresentationIssueField::RollOptionSuboptionValue,
                        );
                    }
                }
                self.fact(
                    &value.toggleable,
                    SpellPresentationIssueField::RollOptionToggleable,
                    FactRequirement::Optional,
                    never_empty,
                );
            }
            Rule::ItemAlteration(value) => {
                self.required_string(
                    &value.item_id,
                    SpellPresentationIssueField::ItemAlterationTarget,
                );
                self.required_string(&value.mode, SpellPresentationIssueField::ItemAlterationMode);
                self.predicates(
                    &value.predicate,
                    SpellPresentationIssueField::ItemAlterationPredicate,
                );
                self.required_string(
                    &value.property,
                    SpellPresentationIssueField::ItemAlterationProperty,
                );
                self.required_string(
                    &value.value,
                    SpellPresentationIssueField::ItemAlterationValue,
                );
            }
            Rule::Unsupported(_) => self.issue(
                SpellPresentationIssueField::UnsupportedRule,
                FactIssueKind::Unsupported,
            ),
        }
    }

    fn predicates(
        &mut self,
        fact: &crate::SpellFactJson<Vec<crate::SpellRulePredicateJson>>,
        field: SpellPresentationIssueField,
    ) {
        let Some(values) = self.fact(fact, field, FactRequirement::Optional, Vec::is_empty) else {
            return;
        };
        if values
            .iter()
            .any(|value| matches!(value, crate::SpellRulePredicateJson::Unsupported(_)))
        {
            self.issue(field, FactIssueKind::Unsupported);
        }
    }

    fn patch(&mut self, patch: &crate::SpellPatchJson) {
        self.classification(&patch.classification, FactRequirement::Optional);
        self.casting(&patch.casting);
        self.targeting(&patch.targeting);
        self.defense(&patch.defense);
        self.damage_patch(&patch.damage);
        self.duration(&patch.duration);
        self.heightening_patch(&patch.heightening);
        self.rules(&patch.rules);
        if !patch.unsupported.is_empty() {
            self.issue(
                SpellPresentationIssueField::UnsupportedPatch,
                FactIssueKind::Unsupported,
            );
        }
    }

    fn damage_patch(&mut self, fact: &crate::SpellFactJson<crate::SpellDamagePatchSetJson>) {
        let Some(value) = self.fact(
            fact,
            SpellPresentationIssueField::Damage,
            FactRequirement::Optional,
            never_empty,
        ) else {
            return;
        };
        for member in &value.members {
            match &member.operation {
                crate::SpellDamagePatchOperationJson::Merge(value) => {
                    for (fact, field) in [
                        (&value.formula, SpellPresentationIssueField::DamageFormula),
                        (&value.damage_type, SpellPresentationIssueField::DamageType),
                        (&value.category, SpellPresentationIssueField::DamageCategory),
                    ] {
                        self.fact(fact, field, FactRequirement::Optional, String::is_empty);
                    }
                    self.fact(
                        &value.kinds,
                        SpellPresentationIssueField::DamageKinds,
                        FactRequirement::Optional,
                        Vec::is_empty,
                    );
                    self.fact(
                        &value.materials,
                        SpellPresentationIssueField::DamageMaterials,
                        FactRequirement::Optional,
                        Vec::is_empty,
                    );
                    self.fact(
                        &value.apply_modifier,
                        SpellPresentationIssueField::DamageApplyModifier,
                        FactRequirement::Optional,
                        never_empty,
                    );
                }
                crate::SpellDamagePatchOperationJson::Delete => {}
                crate::SpellDamagePatchOperationJson::Unsupported(_) => self.issue(
                    SpellPresentationIssueField::Damage,
                    FactIssueKind::Unsupported,
                ),
            }
        }
    }

    fn heightening_patch(&mut self, fact: &crate::SpellFactJson<crate::SpellHeighteningPatchJson>) {
        let Some(value) = self.fact(
            fact,
            SpellPresentationIssueField::Heightening,
            FactRequirement::Optional,
            never_empty,
        ) else {
            return;
        };
        self.fact(
            &value.kind,
            SpellPresentationIssueField::Heightening,
            FactRequirement::Optional,
            String::is_empty,
        );
        self.fact(
            &value.interval,
            SpellPresentationIssueField::HeighteningInterval,
            FactRequirement::Optional,
            never_empty,
        );
        self.fact(
            &value.area,
            SpellPresentationIssueField::HeighteningArea,
            FactRequirement::Optional,
            never_empty,
        );
        if let Some(damage) = self.fact(
            &value.damage,
            SpellPresentationIssueField::HeighteningDamage,
            FactRequirement::Optional,
            never_empty,
        ) {
            for member in &damage.members {
                match &member.operation {
                    crate::SpellTextPatchOperationJson::Merge(value) => {
                        self.fact(
                            value,
                            SpellPresentationIssueField::HeighteningDamage,
                            FactRequirement::Optional,
                            String::is_empty,
                        );
                    }
                    crate::SpellTextPatchOperationJson::Delete => {}
                    crate::SpellTextPatchOperationJson::Unsupported(_) => self.issue(
                        SpellPresentationIssueField::HeighteningDamage,
                        FactIssueKind::Unsupported,
                    ),
                }
            }
        }
    }

    fn required_string(
        &mut self,
        fact: &crate::SpellFactJson<String>,
        field: SpellPresentationIssueField,
    ) {
        self.fact(fact, field, FactRequirement::Required, String::is_empty);
    }

    fn fact<'a, T>(
        &mut self,
        fact: &'a crate::SpellFactJson<T>,
        field: SpellPresentationIssueField,
        requirement: FactRequirement,
        known_empty: fn(&T) -> bool,
    ) -> Option<&'a T> {
        let state = match fact {
            crate::SpellFactJson::Missing => FactPresentationState::Missing,
            crate::SpellFactJson::Null => FactPresentationState::Null,
            crate::SpellFactJson::Known(value) if known_empty(value) => {
                FactPresentationState::KnownEmpty
            }
            crate::SpellFactJson::Known(_) => FactPresentationState::Known,
            crate::SpellFactJson::Unsupported(_) => FactPresentationState::Unsupported,
        };
        if let FactPresentationDisposition::Issue(kind) =
            classify_fact_presentation(FactPresentationRole::Gameplay, state, requirement)
        {
            self.issue(field, kind);
        }
        match fact {
            crate::SpellFactJson::Known(value) => Some(value),
            crate::SpellFactJson::Missing
            | crate::SpellFactJson::Null
            | crate::SpellFactJson::Unsupported(_) => None,
        }
    }

    fn issue(&mut self, field: SpellPresentationIssueField, kind: FactIssueKind) {
        if kind == FactIssueKind::Unavailable
            && self
                .issues
                .keys()
                .any(|existing| existing.placement() == field.placement())
        {
            return;
        }
        self.issues.entry(field).or_insert(kind);
    }

    fn finish(self) -> Vec<SpellPresentationIssue> {
        self.issues
            .into_iter()
            .map(|(field, kind)| SpellPresentationIssue { field, kind })
            .collect()
    }
}

fn never_empty<T>(_value: &T) -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_fact_policy_covers_optional_required_and_provenance_states() {
        use FactPresentationDisposition as Disposition;
        use FactPresentationRole as Role;
        use FactPresentationState as State;
        use FactRequirement as Requirement;

        for state in [State::Missing, State::Null, State::KnownEmpty] {
            assert_eq!(
                classify_fact_presentation(Role::Gameplay, state, Requirement::Optional),
                Disposition::Omit
            );
        }

        for (state, issue) in [
            (State::Missing, FactIssueKind::RequiredMissing),
            (State::Null, FactIssueKind::RequiredNull),
            (State::KnownEmpty, FactIssueKind::RequiredEmpty),
            (State::Unsupported, FactIssueKind::Unsupported),
            (State::Malformed, FactIssueKind::Malformed),
            (State::Ambiguous, FactIssueKind::Ambiguous),
            (State::Unavailable, FactIssueKind::Unavailable),
        ] {
            assert_eq!(
                classify_fact_presentation(Role::Gameplay, state, Requirement::Required),
                Disposition::Issue(issue)
            );
        }

        assert_eq!(
            classify_fact_presentation(Role::Gameplay, State::Known, Requirement::Optional),
            Disposition::Render
        );
        assert_eq!(
            classify_fact_presentation(Role::Provenance, State::Known, Requirement::Optional),
            Disposition::Provenance
        );
        assert_eq!(
            classify_fact_presentation(Role::Provenance, State::KnownEmpty, Requirement::Optional,),
            Disposition::Provenance
        );
        assert_eq!(
            classify_fact_presentation(Role::Unmodeled, State::Known, Requirement::Optional),
            Disposition::Issue(FactIssueKind::Unmodeled)
        );
    }

    #[test]
    fn false_and_zero_are_known_values_not_empty_values() {
        fn state_of_known_value<T>(_value: T) -> FactPresentationState {
            FactPresentationState::Known
        }

        for state in [state_of_known_value(false), state_of_known_value(0_i64)] {
            assert_eq!(
                classify_fact_presentation(
                    FactPresentationRole::Gameplay,
                    state,
                    FactRequirement::Optional,
                ),
                FactPresentationDisposition::Render
            );
        }
    }

    #[test]
    fn resolved_spell_form_labels_use_only_typed_casting_and_targeting_facts() {
        let mut spell = spell_fixture();
        spell.definition.casting = known(crate::SpellCasting {
            time: known("3".to_string()),
            ..crate::SpellCasting::default()
        });
        spell.definition.targeting = known(crate::SpellTargeting {
            area: known(crate::SpellAreaValue {
                value: known(30),
                area_type: known(
                    crate::SpellAreaType::new("emanation").expect("semantic area type"),
                ),
                ..crate::SpellAreaValue::default()
            }),
            ..crate::SpellTargeting::default()
        });
        let form_id = crate::SpellFormId::base(&spell.identity.record_key);
        let resolved = spell
            .resolve_form(
                form_id.clone(),
                crate::SpellFormContext {
                    cast_rank: 2,
                    overlay_id: None,
                },
            )
            .expect("typed area form");
        assert_eq!(
            project_resolved_spell_form_label(&resolved).as_deref(),
            Some("3 actions — 30-foot emanation")
        );

        spell.definition.casting = known(crate::SpellCasting {
            time: known("1".to_string()),
            ..crate::SpellCasting::default()
        });
        spell.definition.targeting = known(crate::SpellTargeting {
            range: known(crate::SpellRangeValue::from_authored_text("touch")),
            ..crate::SpellTargeting::default()
        });
        let resolved = spell
            .resolve_form(
                form_id,
                crate::SpellFormContext {
                    cast_rank: 2,
                    overlay_id: None,
                },
            )
            .expect("typed range form");
        assert_eq!(
            project_resolved_spell_form_label(&resolved).as_deref(),
            Some("1 action — touch")
        );
    }

    #[test]
    fn canonical_and_machine_spell_facts_share_the_same_expectedness() {
        let unsupported = crate::UnsupportedSourceValue {
            shape: crate::UnsupportedSourceShape::Object,
            value: "{}".to_string(),
            reason: crate::UnsupportedSourceReason::SourceFieldDrift,
        };
        let canonical: crate::SpellFact<u8> =
            crate::FactValue::Value(crate::SpellSourceValue::Unsupported(unsupported.clone()));
        let machine = crate::SpellFactJson::<u8>::Unsupported(crate::SpellUnsupportedValueJson {
            shape: "object",
            value: unsupported.value,
            reason: "source_field_drift",
        });
        let expected = FactPresentationDisposition::Issue(FactIssueKind::Unsupported);
        assert_eq!(
            classify_spell_fact(
                &canonical,
                FactPresentationRole::Gameplay,
                FactRequirement::Optional,
            ),
            expected
        );
        assert_eq!(
            classify_spell_json_fact(
                &machine,
                FactPresentationRole::Gameplay,
                FactRequirement::Optional,
            ),
            expected
        );
    }

    #[test]
    fn spell_issue_projection_traverses_resolved_nested_facts_and_all_rule_shapes_once() {
        let mut spell = spell_fixture();
        spell.definition.rules = known(vec![
            rule_element(
                "DamageDice",
                crate::SpellRule::DamageDice(crate::SpellDamageDiceRule {
                    selector: unsupported_fact(),
                    predicate: crate::FactValue::Missing,
                    dice_number: known("1".to_string()),
                    die_size: known("d6".to_string()),
                    damage_type: known("fire".to_string()),
                    hide_if_disabled: known(false),
                }),
            ),
            rule_element(
                "EphemeralEffect",
                crate::SpellRule::EphemeralEffect(crate::SpellEphemeralEffectRule {
                    predicate: known(vec![unsupported_predicate()]),
                    selectors: known(vec!["target".to_string()]),
                    uuid: known("Compendium.pf2e.spell-effects.Item.test".to_string()),
                }),
            ),
            rule_element(
                "DamageAlteration",
                crate::SpellRule::DamageAlteration(crate::SpellDamageAlterationRule {
                    mode: known("override".to_string()),
                    predicate: crate::FactValue::Missing,
                    property: known("damage-type".to_string()),
                    selectors: crate::FactValue::Missing,
                    slug: crate::FactValue::Missing,
                    value: unsupported_fact(),
                }),
            ),
            rule_element(
                "RollOption",
                crate::SpellRule::RollOption(crate::SpellRollOptionRule {
                    domain: known("all".to_string()),
                    label: crate::FactValue::Missing,
                    option: known("test-option".to_string()),
                    placement: known("spellcasting".to_string()),
                    predicate: crate::FactValue::Missing,
                    suboptions: known(vec![crate::SpellRuleSuboption {
                        label: unsupported_fact(),
                        value: known("fire".to_string()),
                    }]),
                    toggleable: known(false),
                }),
            ),
            rule_element(
                "ItemAlteration",
                crate::SpellRule::ItemAlteration(crate::SpellItemAlterationRule {
                    item_id: known("target-item".to_string()),
                    mode: known("override".to_string()),
                    predicate: unsupported_fact(),
                    property: known("badge-value".to_string()),
                    value: known("1".to_string()),
                }),
            ),
        ]);

        let canonical = project_spell_presentation_issues(&spell);
        let mut json = crate::json_projection::spell_issue_presentation(&spell);
        let machine = project_spell_json_presentation_issues(&json);
        assert_eq!(canonical, machine);
        assert_eq!(canonical.len(), 5);
        for field in [
            SpellPresentationIssueField::DamageDiceSelector,
            SpellPresentationIssueField::EphemeralEffectPredicate,
            SpellPresentationIssueField::DamageAlterationValue,
            SpellPresentationIssueField::RollOptionSuboptionLabel,
            SpellPresentationIssueField::ItemAlterationPredicate,
        ] {
            assert_eq!(
                canonical
                    .iter()
                    .filter(|issue| issue.field == field)
                    .count(),
                1,
                "{field:?} should be reported exactly once"
            );
        }

        let serialized = serde_json::to_value(&json).expect("spell JSON should serialize");
        assert_eq!(
            serialized.pointer("/rules/value/0/value/selector/state"),
            Some(&serde_json::json!("unsupported"))
        );
        assert_eq!(
            serialized.pointer("/rules/value/0/value/selector/value/value"),
            Some(&serde_json::json!("{}"))
        );

        json.rules = crate::SpellFactJson::Missing;
        let resolved_only = project_spell_json_presentation_issues(&json);
        assert_eq!(resolved_only, canonical);
    }

    #[test]
    fn spell_issue_projection_localizes_nested_required_and_optional_facts() {
        let mut spell = spell_fixture();
        let crate::FactValue::Value(crate::SpellSourceValue::Known(classification)) =
            &mut spell.definition.classification
        else {
            panic!("known classification fixture");
        };
        classification.rank = unsupported_fact();
        spell.definition.casting = known(crate::SpellCasting {
            time: unsupported_fact(),
            cost: crate::FactValue::Null,
            requirements: known(String::new()),
            counteraction: known(false),
        });
        spell.definition.damage = known(vec![crate::SpellOrderedMember {
            key: "0".to_string(),
            authored_order: 0,
            value: crate::SpellDamage {
                apply_mod: known(false),
                category: crate::FactValue::Missing,
                formula: crate::FactValue::Missing,
                kinds: known(Vec::new()),
                materials: known(Vec::new()),
                damage_type: known("fire".to_string()),
            },
        }]);

        let issues = project_spell_presentation_issues(&spell);
        for (field, kind) in [
            (
                SpellPresentationIssueField::Rank,
                FactIssueKind::Unsupported,
            ),
            (
                SpellPresentationIssueField::CastingTime,
                FactIssueKind::Unsupported,
            ),
            (
                SpellPresentationIssueField::DamageFormula,
                FactIssueKind::RequiredMissing,
            ),
        ] {
            assert_eq!(
                issues
                    .iter()
                    .filter(|issue| issue.field == field && issue.kind == kind)
                    .count(),
                1,
                "{field:?} should retain its expectedness exactly once"
            );
        }
        for quiet in [
            SpellPresentationIssueField::CastingCost,
            SpellPresentationIssueField::CastingRequirements,
            SpellPresentationIssueField::Counteraction,
            SpellPresentationIssueField::DamageKinds,
            SpellPresentationIssueField::DamageMaterials,
            SpellPresentationIssueField::DamageApplyModifier,
        ] {
            assert!(issues.iter().all(|issue| issue.field != quiet));
        }
    }

    #[test]
    fn spell_source_notes_share_one_semantic_issue_without_changing_machine_json() {
        let mut spell = spell_fixture();
        spell
            .definition
            .unsupported_notes
            .push(crate::SpellUnsupportedSourceFact {
                field: crate::SpellUnsupportedSourceField::RitualMember,
                source_path: "/system/ritual/future".to_string(),
                authored_key: "future".to_string(),
                authored_order: Some(0),
                value: crate::UnsupportedSourceValue {
                    shape: crate::UnsupportedSourceShape::Object,
                    value: "{\"private\":true}".to_string(),
                    reason: crate::UnsupportedSourceReason::SourceFieldDrift,
                },
            });

        let canonical = project_spell_presentation_issues(&spell);
        assert_eq!(
            canonical
                .iter()
                .filter(|issue| {
                    issue.field == SpellPresentationIssueField::RitualSourceFact
                        && issue.kind == FactIssueKind::Unmodeled
                })
                .count(),
            1
        );
        let json = crate::json_projection::spell_issue_presentation(&spell);
        assert_eq!(project_spell_json_presentation_issues(&json), canonical);
        let machine = serde_json::to_value(&json).expect("spell JSON should serialize");
        assert!(machine.get("presentation_issues").is_none());
        assert!(!machine.to_string().contains("private"));
        assert_eq!(
            spell.definition.unsupported_notes[0].value.value,
            "{\"private\":true}"
        );
    }

    #[test]
    fn actual_overlay_selection_localizes_one_fixed_field_failure() {
        let mut spell = spell_fixture();
        spell.definition.heightening = known(crate::SpellHeightening::Fixed(vec![
            crate::SpellFixedHeighteningLayer {
                key: "5".to_string(),
                authored_order: 0,
                rank: crate::SpellSourceValue::Known(5),
                patch: crate::SpellPatch {
                    damage: known(crate::SpellKeyedPatch {
                        members: vec![
                            crate::SpellKeyedPatchMember {
                                key: "duplicate".to_string(),
                                authored_order: 0,
                                operation: crate::SpellKeyedPatchOperation::Delete,
                            },
                            crate::SpellKeyedPatchMember {
                                key: "duplicate".to_string(),
                                authored_order: 1,
                                operation: crate::SpellKeyedPatchOperation::Delete,
                            },
                        ],
                    }),
                    ..crate::SpellPatch::default()
                },
            },
        ]));
        let overlay_id = crate::SpellOverlayId::new("alternate").expect("overlay id");
        let overlay = crate::SpellOverlay {
            key: overlay_id.as_str().to_string(),
            authored_order: 0,
            overlay_id: overlay_id.clone(),
            source_id: known(overlay_id.clone()),
            sort: known(0),
            name: known("Alternate".to_string()),
            overlay_type: known(crate::SpellOverlayType::Override),
            patch: crate::SpellPatch::default(),
        };
        let form_id = overlay.form_id(&spell.identity.record_key);
        spell.definition.overlays = known(vec![overlay]);

        let result = spell.resolve_form(
            form_id,
            crate::SpellFormContext {
                cast_rank: 5,
                overlay_id: Some(overlay_id),
            },
        );
        let resolved = result.as_ref().expect("overall form remains available");
        assert!(matches!(
            resolved.damage,
            crate::SpellResolvedField::Unavailable(crate::SpellFormUnavailable {
                field: crate::SpellFormField::Damage,
                source: crate::SpellFormPatchSource::FixedHeightening,
                reason: crate::SpellFormUnavailableReason::DuplicateKey(ref key),
            })
            if key == "duplicate"
        ));
        assert_eq!(
            project_spell_form_result_presentation_issues(&result),
            vec![SpellPresentationIssue {
                field: SpellPresentationIssueField::Damage,
                kind: FactIssueKind::Unavailable,
            }]
        );

        let machine =
            serde_json::to_value(crate::json_projection::spell_issue_presentation(&spell))
                .expect("spell JSON should serialize");
        assert_eq!(
            machine.pointer("/heightening/value/layers/0/patch/damage/value/members/0/key"),
            Some(&serde_json::json!("duplicate"))
        );
        assert_eq!(
            machine.pointer("/heightening/value/layers/0/patch/damage/value/members/1/key"),
            Some(&serde_json::json!("duplicate"))
        );

        let unavailable = Err(crate::SpellFormSelectionError::CastRankBelowBase {
            base_rank: 2,
            cast_rank: 1,
        });
        assert_eq!(
            project_spell_form_result_presentation_issues(&unavailable),
            vec![SpellPresentationIssue {
                field: SpellPresentationIssueField::SpellForm,
                kind: FactIssueKind::Unavailable,
            }]
        );
    }

    fn spell_fixture() -> crate::SpellRecord {
        let key = atlas_domain::RecordKey::parse("spells:issue-fixture").expect("record key");
        let mut spell = crate::SpellRecord::new(
            crate::SpellIdentity {
                record_key: key,
                source_id: crate::SpellSourceId::new("issue-source").expect("source id"),
                name: "Issue Fixture".to_string(),
            },
            crate::SpellProvenance {
                source_path: "packs/spells/issue.json".to_string(),
                source_contract_version: "v1".to_string(),
                source_system_version: "7".to_string(),
                source_upstream_commit: "fixture".to_string(),
                standalone_location: crate::FactValue::Missing,
            },
        );
        spell.definition.classification = known(crate::SpellClassification {
            rank: known(2),
            traits: known(Vec::new()),
            traditions: known(Vec::new()),
        });
        spell
    }

    fn rule_element(authored_key: &str, rule: crate::SpellRule) -> crate::SpellRuleElement {
        crate::SpellRuleElement {
            authored_order: 0,
            source_path: "/system/rules/0".to_string(),
            authored_key: authored_key.to_string(),
            authored_object_json: "{}".to_string(),
            rule,
        }
    }

    fn unsupported_predicate() -> crate::SpellRulePredicate {
        crate::SpellRulePredicate::Unsupported(crate::SpellUnsupportedRulePredicate {
            source_path: "/system/rules/0/predicate/0".to_string(),
            authored_key: None,
            authored_order: 0,
            value: unsupported_source_value(),
        })
    }

    fn known<T>(value: T) -> crate::SpellFact<T> {
        crate::FactValue::Value(crate::SpellSourceValue::Known(value))
    }

    fn unsupported_fact<T>() -> crate::SpellFact<T> {
        crate::FactValue::Value(crate::SpellSourceValue::Unsupported(
            unsupported_source_value(),
        ))
    }

    fn unsupported_source_value() -> crate::UnsupportedSourceValue {
        crate::UnsupportedSourceValue {
            shape: crate::UnsupportedSourceShape::Object,
            value: "{}".to_string(),
            reason: crate::UnsupportedSourceReason::SourceFieldDrift,
        }
    }
}
