use std::collections::{BTreeMap, BTreeSet};

use atlas_domain::{Rarity, RecordKey};
use sha2::{Digest, Sha256};

use crate::{FactValue, OwnedRichContent, StableSourceLocator, UnsupportedSourceValue};

pub const SPELL_RANGE_DERIVATION_RULE: &str = "pf2e-spell-range/6.12.4-v1";

pub type SpellFact<T> = FactValue<SpellSourceValue<T>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpellSourceValue<T> {
    Known(T),
    Unsupported(UnsupportedSourceValue),
}

impl<T> SpellSourceValue<T> {
    pub fn as_known(&self) -> Option<&T> {
        match self {
            Self::Known(value) => Some(value),
            Self::Unsupported(_) => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpellRecord {
    pub identity: SpellIdentity,
    pub definition: SpellDefinition,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpellDefinition {
    pub source_context: SpellSourceContext,
    pub classification: SpellFact<SpellClassification>,
    pub casting: SpellFact<SpellCasting>,
    pub targeting: SpellFact<SpellTargeting>,
    pub defense: SpellFact<SpellDefenseValue>,
    pub damage: SpellFact<Vec<SpellOrderedMember<SpellDamage>>>,
    pub duration: SpellFact<SpellDuration>,
    pub heightening: SpellFact<SpellHeightening>,
    pub overlays: SpellFact<Vec<SpellOverlay>>,
    pub ritual: SpellFact<SpellRitual>,
    pub rules: SpellFact<Vec<SpellRuleElement>>,
    pub content: OwnedRichContent,
    pub unsupported_notes: Vec<SpellUnsupportedSourceFact>,
    pub provenance: SpellProvenance,
}

impl SpellRecord {
    pub fn new(identity: SpellIdentity, provenance: SpellProvenance) -> Self {
        Self {
            identity,
            definition: SpellDefinition::new(provenance),
        }
    }

    pub fn definition(&self) -> &SpellDefinition {
        &self.definition
    }

    pub fn definition_mut(&mut self) -> &mut SpellDefinition {
        &mut self.definition
    }

    pub fn ordered_overlays(&self) -> Result<Vec<&SpellOverlay>, SpellOverlayOrderError> {
        self.definition.ordered_overlays()
    }

    pub fn resolve_form(
        &self,
        form_id: SpellFormId,
        context: SpellFormContext,
    ) -> Result<ResolvedSpellForm, SpellFormSelectionError> {
        self.definition
            .resolve_form(&self.identity.record_key, form_id, context)
    }
}

impl SpellDefinition {
    pub fn new(provenance: SpellProvenance) -> Self {
        Self {
            source_context: SpellSourceContext::default(),
            classification: FactValue::Missing,
            casting: FactValue::Missing,
            targeting: FactValue::Missing,
            defense: FactValue::Missing,
            damage: FactValue::Missing,
            duration: FactValue::Missing,
            heightening: FactValue::Missing,
            overlays: FactValue::Missing,
            ritual: FactValue::Missing,
            rules: FactValue::Missing,
            content: OwnedRichContent::default(),
            unsupported_notes: Vec::new(),
            provenance,
        }
    }

    pub fn ordered_overlays(&self) -> Result<Vec<&SpellOverlay>, SpellOverlayOrderError> {
        let overlays = match &self.overlays {
            FactValue::Missing | FactValue::Null => return Ok(Vec::new()),
            FactValue::Value(SpellSourceValue::Known(overlays)) => overlays,
            FactValue::Value(SpellSourceValue::Unsupported(_)) => {
                return Err(SpellOverlayOrderError {
                    overlay_id: None,
                    reason: SpellOverlayOrderFailure::UnsupportedRoot,
                });
            }
        };
        let mut ordered = overlays
            .iter()
            .map(|overlay| {
                known_fact(&overlay.sort)
                    .copied()
                    .map(|sort| (sort, overlay.authored_order, overlay))
                    .ok_or_else(|| SpellOverlayOrderError {
                        overlay_id: Some(overlay.overlay_id.clone()),
                        reason: SpellOverlayOrderFailure::UnavailableSort,
                    })
            })
            .collect::<Result<Vec<_>, _>>()?;
        ordered.sort_by_key(|(sort, authored_order, _)| (*sort, *authored_order));
        Ok(ordered.into_iter().map(|(_, _, overlay)| overlay).collect())
    }

    pub fn resolve_form(
        &self,
        record_key: &RecordKey,
        form_id: SpellFormId,
        context: SpellFormContext,
    ) -> Result<ResolvedSpellForm, SpellFormSelectionError> {
        let base_rank = known_fact(&self.classification)
            .and_then(|classification| known_fact(&classification.rank))
            .copied();
        if has_rank_dependent_heightening(&self.heightening) && base_rank.is_none() {
            return Err(SpellFormSelectionError::BaseRankUnavailable);
        }
        if let Some(base_rank) = base_rank
            && context.cast_rank < base_rank
        {
            return Err(SpellFormSelectionError::CastRankBelowBase {
                base_rank,
                cast_rank: context.cast_rank,
            });
        }

        let selected_overlay = match &context.overlay_id {
            Some(overlay_id) => Some(self.overlay(overlay_id)?),
            None => None,
        };
        if selected_overlay.is_none() && form_id != SpellFormId::base(record_key) {
            return Err(SpellFormSelectionError::FormIdMismatch);
        }
        if let Some(overlay) = selected_overlay {
            if overlay.form_id(record_key) != form_id {
                return Err(SpellFormSelectionError::FormIdMismatch);
            }
            validate_overlay(overlay)?;
        }

        let mut resolved = ResolvedSpellForm {
            form_id,
            context,
            applied_fixed_ranks: Vec::new(),
            classification: SpellResolvedField::Available(self.classification.clone()),
            casting: SpellResolvedField::Available(self.casting.clone()),
            targeting: SpellResolvedField::Available(self.targeting.clone()),
            defense: SpellResolvedField::Available(self.defense.clone()),
            damage: SpellResolvedField::Available(self.damage.clone()),
            duration: SpellResolvedField::Available(self.duration.clone()),
            heightening: SpellResolvedField::Available(self.heightening.clone()),
            rules: SpellResolvedField::Available(self.rules.clone()),
        };

        resolved.localize_base_unsupported(self);

        if let Some(overlay) = selected_overlay {
            resolved.apply_patch(&overlay.patch, SpellFormPatchSource::Overlay);
        }
        resolved.apply_fixed_heightening();
        Ok(resolved)
    }

    fn overlay(
        &self,
        overlay_id: &SpellOverlayId,
    ) -> Result<&SpellOverlay, SpellFormSelectionError> {
        let overlays = match &self.overlays {
            FactValue::Value(SpellSourceValue::Known(overlays)) => overlays,
            FactValue::Value(SpellSourceValue::Unsupported(_)) => {
                return Err(SpellFormSelectionError::OverlayRootUnavailable);
            }
            FactValue::Missing | FactValue::Null => {
                return Err(SpellFormSelectionError::UnknownOverlay(overlay_id.clone()));
            }
        };
        let mut matches = overlays
            .iter()
            .filter(|overlay| &overlay.overlay_id == overlay_id);
        let overlay = matches
            .next()
            .ok_or_else(|| SpellFormSelectionError::UnknownOverlay(overlay_id.clone()))?;
        if matches.next().is_some() {
            return Err(SpellFormSelectionError::DuplicateOverlay(
                overlay_id.clone(),
            ));
        }
        Ok(overlay)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SpellSourceContext {
    pub image: SpellFact<String>,
    pub publication_license: SpellFact<crate::PublicationLicense>,
    pub consumable_child: FactValue<ConsumableSpellSourceContext>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConsumableSpellSourceContext {
    pub slug: SpellFact<String>,
    pub publication_title: SpellFact<String>,
    pub publication_remaster: SpellFact<bool>,
    pub rarity: SpellFact<Rarity>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpellIdentity {
    pub record_key: RecordKey,
    pub source_id: SpellSourceId,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsumableSpellChild {
    pub parent_record_key: RecordKey,
    pub child_id: SpellChildId,
    pub name: SpellFact<String>,
    pub authored_order: u32,
    pub location: SpellFact<ConsumableSpellLocation>,
    pub standalone_locator: SpellFact<StableSourceLocator>,
    pub standalone_target: FactValue<SpellStandaloneTarget>,
    pub definition: SpellDefinition,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConsumableSpellLocation {
    pub value: SpellFact<String>,
    pub heightened_rank: SpellFact<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpellStandaloneTarget {
    Resolved(RecordKey),
    Unresolved(UnsupportedSourceValue),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SpellSourceId(String);
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SpellChildId(String);
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SpellTrait(String);
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SpellTradition(String);
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SpellAreaType(String);
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SpellStatistic(String);
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SpellOverlayId(String);
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SpellFormId(String);

impl SpellFormId {
    pub fn base(record_key: &RecordKey) -> Self {
        Self::opaque("base", record_key, None)
    }

    pub fn overlay(record_key: &RecordKey, overlay_id: &SpellOverlayId) -> Self {
        Self::opaque("overlay", record_key, Some(overlay_id.as_str()))
    }

    fn opaque(kind: &str, record_key: &RecordKey, overlay_id: Option<&str>) -> Self {
        let mut digest = Sha256::new();
        digest.update(b"atlas-spell-form/v1\0");
        digest.update(kind.as_bytes());
        digest.update(b"\0");
        digest.update(record_key.to_string().as_bytes());
        if let Some(overlay_id) = overlay_id {
            digest.update(b"\0");
            digest.update(overlay_id.as_bytes());
        }
        Self(format!("spell-form:{:x}", digest.finalize()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidSpellIdentity;

macro_rules! define_string_newtype {
    ($name:ident) => {
        impl $name {
            pub fn new(value: impl Into<String>) -> Result<Self, InvalidSpellIdentity> {
                let value = value.into();
                if value.trim().is_empty() || value.chars().any(char::is_whitespace) {
                    Err(InvalidSpellIdentity)
                } else {
                    Ok(Self(value))
                }
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }
    };
}

define_string_newtype!(SpellSourceId);
define_string_newtype!(SpellChildId);
define_string_newtype!(SpellTrait);
define_string_newtype!(SpellTradition);
define_string_newtype!(SpellAreaType);
define_string_newtype!(SpellStatistic);
define_string_newtype!(SpellOverlayId);
define_string_newtype!(SpellFormId);

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SpellClassification {
    pub rank: SpellFact<u8>,
    pub traits: SpellFact<Vec<SpellTrait>>,
    pub traditions: SpellFact<Vec<SpellTradition>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SpellCasting {
    pub time: SpellFact<String>,
    pub cost: SpellFact<String>,
    pub requirements: SpellFact<String>,
    pub counteraction: SpellFact<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SpellTargeting {
    pub target: SpellFact<String>,
    pub range: SpellFact<SpellRangeValue>,
    pub area: SpellFact<SpellAreaValue>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpellRangeValue {
    pub authored_text: String,
    pub numeric: Option<SpellNumericRange>,
}

impl SpellRangeValue {
    pub fn from_authored_text(authored_text: impl Into<String>) -> Self {
        let authored_text = authored_text.into();
        let numeric = derive_numeric_range(&authored_text);
        Self {
            authored_text,
            numeric,
        }
    }

    pub fn has_valid_projection(&self) -> bool {
        self.numeric == derive_numeric_range(&self.authored_text)
    }
}

fn derive_numeric_range(authored_text: &str) -> Option<SpellNumericRange> {
    let normalized = authored_text.trim();
    if normalized == "touch" {
        return Some(SpellNumericRange {
            feet: 5,
            kind: SpellNumericRangeKind::TouchMelee,
            rule: SPELL_RANGE_DERIVATION_RULE.to_string(),
        });
    }
    let number = normalized
        .strip_suffix(" feet")
        .or_else(|| normalized.strip_suffix(" ft"))?;
    if number.is_empty() || !number.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    let feet = number.parse::<u32>().ok()?;
    Some(SpellNumericRange {
        feet,
        kind: SpellNumericRangeKind::Distance,
        rule: SPELL_RANGE_DERIVATION_RULE.to_string(),
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpellNumericRange {
    pub feet: u32,
    pub kind: SpellNumericRangeKind,
    pub rule: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpellNumericRangeKind {
    TouchMelee,
    Distance,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SpellAreaValue {
    pub value: SpellFact<u32>,
    pub area_type: SpellFact<SpellAreaType>,
    pub legacy_area_type: SpellFact<SpellLegacyAreaType>,
    pub details: SpellFact<String>,
    pub unsupported_notes: Vec<SpellUnsupportedSourceFact>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SpellDefenseValue {
    pub passive: SpellFact<SpellStatistic>,
    pub save: SpellFact<SpellSave>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SpellSave {
    pub statistic: SpellFact<SpellStatistic>,
    pub basic: SpellFact<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpellOrderedMember<T> {
    pub key: String,
    pub authored_order: u32,
    pub value: T,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SpellDamage {
    pub apply_mod: SpellFact<bool>,
    pub category: SpellFact<String>,
    pub formula: SpellFact<String>,
    pub kinds: SpellFact<Vec<String>>,
    pub materials: SpellFact<Vec<String>>,
    pub damage_type: SpellFact<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SpellDuration {
    pub value: SpellFact<String>,
    pub sustained: SpellFact<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpellHeightening {
    Interval(SpellIntervalHeightening),
    Fixed(Vec<SpellFixedHeighteningLayer>),
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SpellIntervalHeightening {
    pub interval: SpellFact<u8>,
    pub area: SpellFact<u32>,
    pub damage: SpellFact<Vec<SpellOrderedMember<String>>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpellFixedHeighteningLayer {
    pub key: String,
    pub authored_order: u32,
    pub rank: SpellSourceValue<u8>,
    pub patch: SpellPatch,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpellOverlay {
    pub key: String,
    pub authored_order: u32,
    pub overlay_id: SpellOverlayId,
    pub source_id: SpellFact<SpellOverlayId>,
    pub sort: SpellFact<i64>,
    pub name: SpellFact<String>,
    pub overlay_type: SpellFact<SpellOverlayType>,
    pub patch: SpellPatch,
}

impl SpellOverlay {
    pub fn form_id(&self, record_key: &RecordKey) -> SpellFormId {
        SpellFormId::overlay(record_key, &self.overlay_id)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpellOverlayType {
    Override,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SpellPatch {
    pub classification: SpellFact<SpellClassificationPatch>,
    pub casting: SpellFact<SpellCastingPatch>,
    pub targeting: SpellFact<SpellTargetingPatch>,
    pub defense: SpellFact<SpellDefensePatch>,
    pub damage: SpellFact<SpellKeyedPatch<SpellDamagePatch>>,
    pub duration: SpellFact<SpellDurationPatch>,
    pub heightening: SpellFact<SpellHeighteningPatch>,
    pub rules: SpellFact<Vec<SpellRuleElement>>,
    pub unsupported: Vec<SpellUnsupportedPatchField>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SpellClassificationPatch {
    pub rank: SpellFact<u8>,
    pub traits: SpellFact<Vec<SpellTrait>>,
    pub traditions: SpellFact<Vec<SpellTradition>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SpellCastingPatch {
    pub time: SpellFact<String>,
    pub cost: SpellFact<String>,
    pub requirements: SpellFact<String>,
    pub counteraction: SpellFact<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SpellTargetingPatch {
    pub target: SpellFact<String>,
    pub range: SpellFact<SpellRangeValue>,
    pub area: SpellFact<SpellAreaPatch>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SpellAreaPatch {
    pub value: SpellFact<u32>,
    pub area_type: SpellFact<SpellAreaType>,
    pub legacy_area_type: SpellFact<SpellLegacyAreaType>,
    pub details: SpellFact<String>,
    pub unsupported_notes: Vec<SpellUnsupportedSourceFact>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SpellDefensePatch {
    pub passive: SpellFact<SpellStatistic>,
    pub save: SpellFact<SpellSavePatch>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SpellSavePatch {
    pub statistic: SpellFact<SpellStatistic>,
    pub basic: SpellFact<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SpellDamagePatch {
    pub apply_mod: SpellFact<bool>,
    pub category: SpellFact<String>,
    pub formula: SpellFact<String>,
    pub kinds: SpellFact<Vec<String>>,
    pub materials: SpellFact<Vec<String>>,
    pub damage_type: SpellFact<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SpellDurationPatch {
    pub value: SpellFact<String>,
    pub sustained: SpellFact<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SpellHeighteningPatch {
    pub kind: SpellFact<SpellHeighteningType>,
    pub interval: SpellFact<u8>,
    pub area: SpellFact<u32>,
    pub interval_damage: SpellFact<SpellKeyedPatch<SpellTextPatch>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpellHeighteningType {
    Interval,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SpellTextPatch {
    pub value: SpellFact<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SpellKeyedPatch<T> {
    pub members: Vec<SpellKeyedPatchMember<T>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpellKeyedPatchMember<T> {
    pub key: String,
    pub authored_order: u32,
    pub operation: SpellKeyedPatchOperation<T>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpellKeyedPatchOperation<T> {
    Merge(T),
    Delete,
    Unsupported(UnsupportedSourceValue),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpellRitual {
    pub primary_check: SpellFact<String>,
    pub secondary_casters: SpellFact<u32>,
    pub secondary_checks: SpellFact<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpellRuleElement {
    pub authored_order: u32,
    pub source_path: String,
    pub authored_key: String,
    pub authored_object_json: String,
    pub rule: SpellRule,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpellLegacyAreaType {
    pub source_path: String,
    pub authored_key: String,
    pub authored_order: u32,
    pub value: SpellAreaType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpellRule {
    DamageDice(SpellDamageDiceRule),
    EphemeralEffect(SpellEphemeralEffectRule),
    DamageAlteration(SpellDamageAlterationRule),
    RollOption(SpellRollOptionRule),
    ItemAlteration(SpellItemAlterationRule),
    Unsupported(SpellUnsupportedRule),
}

impl SpellRule {
    fn expected_authored_key(&self) -> Option<&'static str> {
        Some(match self {
            Self::DamageDice(_) => "DamageDice",
            Self::EphemeralEffect(_) => "EphemeralEffect",
            Self::DamageAlteration(_) => "DamageAlteration",
            Self::RollOption(_) => "RollOption",
            Self::ItemAlteration(_) => "ItemAlteration",
            Self::Unsupported(_) => return None,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SpellDamageDiceRule {
    pub selector: SpellFact<String>,
    pub predicate: SpellFact<Vec<SpellRulePredicate>>,
    pub dice_number: SpellFact<String>,
    pub die_size: SpellFact<String>,
    pub damage_type: SpellFact<String>,
    pub hide_if_disabled: SpellFact<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SpellEphemeralEffectRule {
    pub predicate: SpellFact<Vec<SpellRulePredicate>>,
    pub selectors: SpellFact<Vec<String>>,
    pub uuid: SpellFact<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SpellDamageAlterationRule {
    pub mode: SpellFact<String>,
    pub predicate: SpellFact<Vec<SpellRulePredicate>>,
    pub property: SpellFact<String>,
    pub selectors: SpellFact<Vec<String>>,
    pub slug: SpellFact<String>,
    pub value: SpellFact<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SpellRollOptionRule {
    pub domain: SpellFact<String>,
    pub label: SpellFact<String>,
    pub option: SpellFact<String>,
    pub placement: SpellFact<String>,
    pub predicate: SpellFact<Vec<SpellRulePredicate>>,
    pub suboptions: SpellFact<Vec<SpellRuleSuboption>>,
    pub toggleable: SpellFact<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SpellItemAlterationRule {
    pub item_id: SpellFact<String>,
    pub mode: SpellFact<String>,
    pub predicate: SpellFact<Vec<SpellRulePredicate>>,
    pub property: SpellFact<String>,
    pub value: SpellFact<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpellRulePredicate {
    Term(String),
    Or(Vec<String>),
    Unsupported(SpellUnsupportedRulePredicate),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpellUnsupportedRulePredicate {
    pub source_path: String,
    pub authored_key: Option<String>,
    pub authored_order: u32,
    pub value: UnsupportedSourceValue,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SpellRuleSuboption {
    pub label: SpellFact<String>,
    pub value: SpellFact<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpellUnsupportedRule {
    pub authored_key: String,
    pub source_path: String,
    pub value: UnsupportedSourceValue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpellUnsupportedSourceFact {
    pub field: SpellUnsupportedSourceField,
    pub source_path: String,
    pub authored_key: String,
    pub authored_order: Option<u32>,
    pub value: UnsupportedSourceValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SpellUnsupportedSourceField {
    SystemMember,
    ProvenanceMember,
    ClassificationMember,
    CastingMember,
    TargetingMember,
    DefenseMember,
    DurationMember,
    LocationMember,
    LegacyTraitSelection,
    AreaMember,
    DamageMember,
    HeighteningMember,
    OverlayMember,
    RitualMember,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpellProvenance {
    pub source_path: String,
    pub source_contract_version: String,
    pub source_system_version: String,
    pub source_upstream_commit: String,
    pub standalone_location: FactValue<UnsupportedSourceValue>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpellFormContext {
    pub cast_rank: u8,
    pub overlay_id: Option<SpellOverlayId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedSpellForm {
    pub form_id: SpellFormId,
    pub context: SpellFormContext,
    pub applied_fixed_ranks: Vec<u8>,
    pub classification: SpellResolvedField<SpellFact<SpellClassification>>,
    pub casting: SpellResolvedField<SpellFact<SpellCasting>>,
    pub targeting: SpellResolvedField<SpellFact<SpellTargeting>>,
    pub defense: SpellResolvedField<SpellFact<SpellDefenseValue>>,
    pub damage: SpellResolvedField<SpellFact<Vec<SpellOrderedMember<SpellDamage>>>>,
    pub duration: SpellResolvedField<SpellFact<SpellDuration>>,
    pub heightening: SpellResolvedField<SpellFact<SpellHeightening>>,
    pub rules: SpellResolvedField<SpellFact<Vec<SpellRuleElement>>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpellResolvedField<T> {
    Available(T),
    Unavailable(SpellFormUnavailable),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpellFormUnavailable {
    pub field: SpellFormField,
    pub source: SpellFormPatchSource,
    pub reason: SpellFormUnavailableReason,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SpellFormField {
    Classification,
    Casting,
    Targeting,
    Defense,
    Damage,
    Duration,
    Heightening,
    Rules,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpellFormPatchSource {
    Base,
    Overlay,
    FixedHeightening,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpellFormUnavailableReason {
    UnsupportedPatch,
    DuplicateKey(String),
    IncompatibleHeightening,
    UnknownFixedRank(String),
    FixedRankKeyMismatch { key: String, rank: u8 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpellFormSelectionError {
    BaseRankUnavailable,
    CastRankBelowBase {
        base_rank: u8,
        cast_rank: u8,
    },
    UnknownOverlay(SpellOverlayId),
    OverlayRootUnavailable,
    DuplicateOverlay(SpellOverlayId),
    OverlayIdentityUnavailable(SpellOverlayId),
    OverlayIdentityMismatch {
        key: String,
        overlay_id: String,
        source_id: String,
    },
    UnsupportedOverlayType(SpellOverlayId),
    FormIdMismatch,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpellOverlayOrderError {
    pub overlay_id: Option<SpellOverlayId>,
    pub reason: SpellOverlayOrderFailure,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpellOverlayOrderFailure {
    UnsupportedRoot,
    UnavailableSort,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpellUnsupportedPatchField {
    pub field: SpellFormField,
    pub source_path: String,
    pub authored_key: String,
    pub authored_order: Option<u32>,
    pub value: UnsupportedSourceValue,
}

impl ResolvedSpellForm {
    fn localize_base_unsupported(&mut self, definition: &SpellDefinition) {
        for (field, unsupported) in [
            (
                SpellFormField::Classification,
                classification_has_unsupported(&definition.classification),
            ),
            (
                SpellFormField::Casting,
                casting_has_unsupported(&definition.casting),
            ),
            (
                SpellFormField::Targeting,
                targeting_has_unsupported(&definition.targeting),
            ),
            (
                SpellFormField::Defense,
                defense_has_unsupported(&definition.defense),
            ),
            (
                SpellFormField::Damage,
                damage_has_unsupported(&definition.damage),
            ),
            (
                SpellFormField::Duration,
                duration_has_unsupported(&definition.duration),
            ),
            (
                SpellFormField::Heightening,
                heightening_has_unsupported(&definition.heightening),
            ),
            (
                SpellFormField::Rules,
                rules_have_unsupported(&definition.rules),
            ),
        ] {
            if unsupported {
                self.mark_unavailable(
                    field,
                    SpellFormPatchSource::Base,
                    SpellFormUnavailableReason::UnsupportedPatch,
                );
            }
        }
        for note in &definition.unsupported_notes {
            let fields: &[SpellFormField] = match note.field {
                SpellUnsupportedSourceField::SystemMember => &[
                    SpellFormField::Classification,
                    SpellFormField::Casting,
                    SpellFormField::Targeting,
                    SpellFormField::Defense,
                    SpellFormField::Damage,
                    SpellFormField::Duration,
                    SpellFormField::Heightening,
                    SpellFormField::Rules,
                ],
                SpellUnsupportedSourceField::ClassificationMember => {
                    &[SpellFormField::Classification]
                }
                SpellUnsupportedSourceField::CastingMember => &[SpellFormField::Casting],
                SpellUnsupportedSourceField::TargetingMember
                | SpellUnsupportedSourceField::AreaMember => &[SpellFormField::Targeting],
                SpellUnsupportedSourceField::DefenseMember => &[SpellFormField::Defense],
                SpellUnsupportedSourceField::DamageMember => &[SpellFormField::Damage],
                SpellUnsupportedSourceField::DurationMember => &[SpellFormField::Duration],
                SpellUnsupportedSourceField::HeighteningMember => &[SpellFormField::Heightening],
                SpellUnsupportedSourceField::ProvenanceMember
                | SpellUnsupportedSourceField::LocationMember
                | SpellUnsupportedSourceField::LegacyTraitSelection
                | SpellUnsupportedSourceField::OverlayMember
                | SpellUnsupportedSourceField::RitualMember => &[],
            };
            for field in fields {
                self.mark_unavailable(
                    *field,
                    SpellFormPatchSource::Base,
                    SpellFormUnavailableReason::UnsupportedPatch,
                );
            }
        }
    }

    fn apply_fixed_heightening(&mut self) {
        let heightening = match &self.heightening {
            SpellResolvedField::Available(FactValue::Value(SpellSourceValue::Known(
                SpellHeightening::Fixed(layers),
            ))) => layers.clone(),
            _ => return,
        };
        let mut applicable = Vec::new();
        let mut key_counts = BTreeMap::new();
        let mut rank_counts = BTreeMap::new();
        for layer in &heightening {
            *key_counts.entry(layer.key.clone()).or_insert(0_u32) += 1;
            if let SpellSourceValue::Known(rank) = &layer.rank {
                *rank_counts.entry(*rank).or_insert(0_u32) += 1;
            }
        }
        for layer in heightening {
            if let SpellSourceValue::Known(rank) = &layer.rank
                && layer.key != rank.to_string()
            {
                self.unavailable_for_patch(
                    &layer.patch,
                    SpellFormPatchSource::FixedHeightening,
                    SpellFormUnavailableReason::FixedRankKeyMismatch {
                        key: layer.key,
                        rank: *rank,
                    },
                );
                continue;
            }
            let duplicate = key_counts.get(&layer.key).copied().unwrap_or(0) > 1
                || match &layer.rank {
                    SpellSourceValue::Known(rank) => {
                        rank_counts.get(rank).copied().unwrap_or(0) > 1
                    }
                    SpellSourceValue::Unsupported(_) => false,
                };
            if duplicate {
                self.unavailable_for_patch(
                    &layer.patch,
                    SpellFormPatchSource::FixedHeightening,
                    SpellFormUnavailableReason::DuplicateKey(layer.key),
                );
                continue;
            }
            match layer.rank {
                SpellSourceValue::Known(rank) if rank <= self.context.cast_rank => {
                    applicable.push((rank, layer.authored_order, layer.patch));
                }
                SpellSourceValue::Known(_) => {}
                SpellSourceValue::Unsupported(_) => {
                    self.unavailable_for_patch(
                        &layer.patch,
                        SpellFormPatchSource::FixedHeightening,
                        SpellFormUnavailableReason::UnknownFixedRank(layer.key),
                    );
                }
            }
        }
        applicable.sort_by_key(|(rank, authored_order, _)| (*rank, *authored_order));
        for (rank, _, patch) in applicable {
            self.apply_patch(&patch, SpellFormPatchSource::FixedHeightening);
            self.applied_fixed_ranks.push(rank);
        }
    }

    fn apply_patch(&mut self, patch: &SpellPatch, source: SpellFormPatchSource) {
        apply_object_patch(
            &mut self.classification,
            &patch.classification,
            SpellFormField::Classification,
            source,
            apply_classification_patch,
        );
        apply_object_patch(
            &mut self.casting,
            &patch.casting,
            SpellFormField::Casting,
            source,
            apply_casting_patch,
        );
        apply_object_patch(
            &mut self.targeting,
            &patch.targeting,
            SpellFormField::Targeting,
            source,
            apply_targeting_patch,
        );
        apply_object_patch(
            &mut self.defense,
            &patch.defense,
            SpellFormField::Defense,
            source,
            apply_defense_patch,
        );
        apply_damage_patch(&mut self.damage, &patch.damage, source);
        apply_object_patch(
            &mut self.duration,
            &patch.duration,
            SpellFormField::Duration,
            source,
            apply_duration_patch,
        );
        apply_heightening_patch(&mut self.heightening, &patch.heightening, source);
        apply_rule_patch(&mut self.rules, &patch.rules, source);
        for unsupported in &patch.unsupported {
            self.mark_unavailable(
                unsupported.field,
                source,
                SpellFormUnavailableReason::UnsupportedPatch,
            );
        }
    }

    fn unavailable_for_patch(
        &mut self,
        patch: &SpellPatch,
        source: SpellFormPatchSource,
        reason: SpellFormUnavailableReason,
    ) {
        for field in patch.affected_fields() {
            self.mark_unavailable(field, source, reason.clone());
        }
    }

    fn mark_unavailable(
        &mut self,
        field: SpellFormField,
        source: SpellFormPatchSource,
        reason: SpellFormUnavailableReason,
    ) {
        let unavailable = SpellFormUnavailable {
            field,
            source,
            reason,
        };
        match field {
            SpellFormField::Classification => {
                self.classification = SpellResolvedField::Unavailable(unavailable)
            }
            SpellFormField::Casting => self.casting = SpellResolvedField::Unavailable(unavailable),
            SpellFormField::Targeting => {
                self.targeting = SpellResolvedField::Unavailable(unavailable)
            }
            SpellFormField::Defense => self.defense = SpellResolvedField::Unavailable(unavailable),
            SpellFormField::Damage => self.damage = SpellResolvedField::Unavailable(unavailable),
            SpellFormField::Duration => {
                self.duration = SpellResolvedField::Unavailable(unavailable)
            }
            SpellFormField::Heightening => {
                self.heightening = SpellResolvedField::Unavailable(unavailable)
            }
            SpellFormField::Rules => self.rules = SpellResolvedField::Unavailable(unavailable),
        }
    }
}

impl SpellPatch {
    fn affected_fields(&self) -> BTreeSet<SpellFormField> {
        let mut fields = BTreeSet::new();
        for (field, present) in [
            (
                SpellFormField::Classification,
                !matches!(self.classification, FactValue::Missing),
            ),
            (
                SpellFormField::Casting,
                !matches!(self.casting, FactValue::Missing),
            ),
            (
                SpellFormField::Targeting,
                !matches!(self.targeting, FactValue::Missing),
            ),
            (
                SpellFormField::Defense,
                !matches!(self.defense, FactValue::Missing),
            ),
            (
                SpellFormField::Damage,
                !matches!(self.damage, FactValue::Missing),
            ),
            (
                SpellFormField::Duration,
                !matches!(self.duration, FactValue::Missing),
            ),
            (
                SpellFormField::Heightening,
                !matches!(self.heightening, FactValue::Missing),
            ),
            (
                SpellFormField::Rules,
                !matches!(self.rules, FactValue::Missing),
            ),
        ] {
            if present {
                fields.insert(field);
            }
        }
        fields.extend(self.unsupported.iter().map(|field| field.field));
        fields
    }
}

fn validate_overlay(overlay: &SpellOverlay) -> Result<(), SpellFormSelectionError> {
    match &overlay.source_id {
        FactValue::Value(SpellSourceValue::Known(source_id))
            if source_id.as_str() == overlay.key && overlay.overlay_id.as_str() == overlay.key => {}
        FactValue::Missing if overlay.overlay_id.as_str() == overlay.key => {}
        FactValue::Value(SpellSourceValue::Known(source_id)) => {
            return Err(SpellFormSelectionError::OverlayIdentityMismatch {
                key: overlay.key.clone(),
                overlay_id: overlay.overlay_id.as_str().to_string(),
                source_id: source_id.as_str().to_string(),
            });
        }
        FactValue::Missing
        | FactValue::Null
        | FactValue::Value(SpellSourceValue::Unsupported(_)) => {
            return Err(SpellFormSelectionError::OverlayIdentityUnavailable(
                overlay.overlay_id.clone(),
            ));
        }
    }
    if !matches!(
        overlay.overlay_type,
        FactValue::Value(SpellSourceValue::Known(SpellOverlayType::Override))
    ) {
        return Err(SpellFormSelectionError::UnsupportedOverlayType(
            overlay.overlay_id.clone(),
        ));
    }
    Ok(())
}

fn known_fact<T>(fact: &SpellFact<T>) -> Option<&T> {
    match fact {
        FactValue::Value(SpellSourceValue::Known(value)) => Some(value),
        FactValue::Missing
        | FactValue::Null
        | FactValue::Value(SpellSourceValue::Unsupported(_)) => None,
    }
}

fn has_rank_dependent_heightening(heightening: &SpellFact<SpellHeightening>) -> bool {
    matches!(
        heightening,
        FactValue::Value(SpellSourceValue::Known(SpellHeightening::Fixed(_)))
    )
}

fn fact_has_unsupported<T>(fact: &SpellFact<T>) -> bool {
    matches!(fact, FactValue::Value(SpellSourceValue::Unsupported(_)))
}

fn classification_has_unsupported(classification: &SpellFact<SpellClassification>) -> bool {
    match classification {
        FactValue::Value(SpellSourceValue::Known(value)) => {
            fact_has_unsupported(&value.rank)
                || fact_has_unsupported(&value.traits)
                || fact_has_unsupported(&value.traditions)
        }
        other => fact_has_unsupported(other),
    }
}

fn casting_has_unsupported(casting: &SpellFact<SpellCasting>) -> bool {
    match casting {
        FactValue::Value(SpellSourceValue::Known(value)) => {
            fact_has_unsupported(&value.time)
                || fact_has_unsupported(&value.cost)
                || fact_has_unsupported(&value.requirements)
                || fact_has_unsupported(&value.counteraction)
        }
        other => fact_has_unsupported(other),
    }
}

fn targeting_has_unsupported(targeting: &SpellFact<SpellTargeting>) -> bool {
    let FactValue::Value(SpellSourceValue::Known(targeting)) = targeting else {
        return matches!(
            targeting,
            FactValue::Value(SpellSourceValue::Unsupported(_))
        );
    };
    if fact_has_unsupported(&targeting.target) || fact_has_unsupported(&targeting.range) {
        return true;
    }
    let FactValue::Value(SpellSourceValue::Known(area)) = &targeting.area else {
        return matches!(
            targeting.area,
            FactValue::Value(SpellSourceValue::Unsupported(_))
        );
    };
    area_has_unsupported(area)
}

fn area_has_unsupported(area: &SpellAreaValue) -> bool {
    if !area.unsupported_notes.is_empty()
        || fact_has_unsupported(&area.value)
        || fact_has_unsupported(&area.area_type)
        || fact_has_unsupported(&area.legacy_area_type)
        || fact_has_unsupported(&area.details)
    {
        return true;
    }
    match &area.legacy_area_type {
        FactValue::Missing => false,
        FactValue::Null => true,
        FactValue::Value(SpellSourceValue::Known(legacy)) => {
            legacy.authored_key != "areaType"
                || legacy.source_path.is_empty()
                || !matches!(
                    &area.area_type,
                    FactValue::Value(SpellSourceValue::Known(current)) if current == &legacy.value
                )
        }
        FactValue::Value(SpellSourceValue::Unsupported(_)) => true,
    }
}

fn defense_has_unsupported(defense: &SpellFact<SpellDefenseValue>) -> bool {
    let FactValue::Value(SpellSourceValue::Known(defense)) = defense else {
        return fact_has_unsupported(defense);
    };
    if fact_has_unsupported(&defense.passive) {
        return true;
    }
    match &defense.save {
        FactValue::Value(SpellSourceValue::Known(save)) => {
            fact_has_unsupported(&save.statistic) || fact_has_unsupported(&save.basic)
        }
        other => fact_has_unsupported(other),
    }
}

fn damage_has_unsupported(damage: &SpellFact<Vec<SpellOrderedMember<SpellDamage>>>) -> bool {
    match damage {
        FactValue::Value(SpellSourceValue::Known(members)) => members.iter().any(|member| {
            let value = &member.value;
            fact_has_unsupported(&value.apply_mod)
                || fact_has_unsupported(&value.category)
                || fact_has_unsupported(&value.formula)
                || fact_has_unsupported(&value.kinds)
                || fact_has_unsupported(&value.materials)
                || fact_has_unsupported(&value.damage_type)
        }),
        other => fact_has_unsupported(other),
    }
}

fn duration_has_unsupported(duration: &SpellFact<SpellDuration>) -> bool {
    match duration {
        FactValue::Value(SpellSourceValue::Known(value)) => {
            fact_has_unsupported(&value.value) || fact_has_unsupported(&value.sustained)
        }
        other => fact_has_unsupported(other),
    }
}

fn heightening_has_unsupported(heightening: &SpellFact<SpellHeightening>) -> bool {
    match heightening {
        FactValue::Value(SpellSourceValue::Known(SpellHeightening::Interval(value))) => {
            fact_has_unsupported(&value.interval)
                || fact_has_unsupported(&value.area)
                || fact_has_unsupported(&value.damage)
        }
        FactValue::Value(SpellSourceValue::Known(SpellHeightening::Fixed(_))) => false,
        other => fact_has_unsupported(other),
    }
}

fn rules_have_unsupported(rules: &SpellFact<Vec<SpellRuleElement>>) -> bool {
    match rules {
        FactValue::Value(SpellSourceValue::Known(rules)) => {
            rules.iter().any(|element| !rule_element_is_safe(element))
        }
        FactValue::Value(SpellSourceValue::Unsupported(_)) => true,
        FactValue::Missing | FactValue::Null => false,
    }
}

fn rule_element_is_safe(element: &SpellRuleElement) -> bool {
    let Some(expected_key) = element.rule.expected_authored_key() else {
        return false;
    };
    element.authored_key == expected_key
        && !element.source_path.is_empty()
        && !element.authored_object_json.is_empty()
}

fn apply_leaf<T: Clone>(current: &mut SpellFact<T>, patch: &SpellFact<T>) -> Result<(), ()> {
    match patch {
        FactValue::Missing => Ok(()),
        FactValue::Null => {
            *current = FactValue::Null;
            Ok(())
        }
        FactValue::Value(SpellSourceValue::Known(value)) => {
            *current = FactValue::Value(SpellSourceValue::Known(value.clone()));
            Ok(())
        }
        FactValue::Value(SpellSourceValue::Unsupported(_)) => Err(()),
    }
}

fn apply_object_patch<T: Clone + Default, P>(
    resolved: &mut SpellResolvedField<SpellFact<T>>,
    patch: &SpellFact<P>,
    field: SpellFormField,
    source: SpellFormPatchSource,
    apply: fn(&mut T, &P) -> Result<(), ()>,
) {
    if matches!(resolved, SpellResolvedField::Unavailable(_)) || matches!(patch, FactValue::Missing)
    {
        return;
    }
    let unavailable = || {
        SpellResolvedField::Unavailable(SpellFormUnavailable {
            field,
            source,
            reason: SpellFormUnavailableReason::UnsupportedPatch,
        })
    };
    match patch {
        FactValue::Missing => {}
        FactValue::Null => *resolved = SpellResolvedField::Available(FactValue::Null),
        FactValue::Value(SpellSourceValue::Unsupported(_)) => *resolved = unavailable(),
        FactValue::Value(SpellSourceValue::Known(patch)) => {
            let SpellResolvedField::Available(current) = resolved else {
                return;
            };
            let mut value = match current {
                FactValue::Value(SpellSourceValue::Known(value)) => value.clone(),
                FactValue::Missing | FactValue::Null => T::default(),
                FactValue::Value(SpellSourceValue::Unsupported(_)) => {
                    *resolved = unavailable();
                    return;
                }
            };
            if apply(&mut value, patch).is_err() {
                *resolved = unavailable();
            } else {
                *current = FactValue::Value(SpellSourceValue::Known(value));
            }
        }
    }
}

macro_rules! apply_fields {
    ($current:expr, $patch:expr, $($field:ident),+ $(,)?) => {{
        $(apply_leaf(&mut $current.$field, &$patch.$field)?;)+
        Ok(())
    }};
}

fn apply_classification_patch(
    current: &mut SpellClassification,
    patch: &SpellClassificationPatch,
) -> Result<(), ()> {
    apply_fields!(current, patch, rank, traits, traditions)
}

fn apply_casting_patch(current: &mut SpellCasting, patch: &SpellCastingPatch) -> Result<(), ()> {
    apply_fields!(current, patch, time, cost, requirements, counteraction)
}

fn apply_targeting_patch(
    current: &mut SpellTargeting,
    patch: &SpellTargetingPatch,
) -> Result<(), ()> {
    apply_leaf(&mut current.target, &patch.target)?;
    apply_leaf(&mut current.range, &patch.range)?;
    match &patch.area {
        FactValue::Missing => {}
        FactValue::Null => current.area = FactValue::Null,
        FactValue::Value(SpellSourceValue::Unsupported(_)) => return Err(()),
        FactValue::Value(SpellSourceValue::Known(patch)) => {
            let mut area = match &current.area {
                FactValue::Value(SpellSourceValue::Known(area)) => area.clone(),
                FactValue::Missing | FactValue::Null => SpellAreaValue::default(),
                FactValue::Value(SpellSourceValue::Unsupported(_)) => return Err(()),
            };
            if !area.unsupported_notes.is_empty() || !patch.unsupported_notes.is_empty() {
                return Err(());
            }
            apply_fields!(area, patch, value, area_type, legacy_area_type, details)?;
            if area_has_unsupported(&area) {
                return Err(());
            }
            current.area = FactValue::Value(SpellSourceValue::Known(area));
        }
    }
    Ok(())
}

fn apply_defense_patch(
    current: &mut SpellDefenseValue,
    patch: &SpellDefensePatch,
) -> Result<(), ()> {
    apply_leaf(&mut current.passive, &patch.passive)?;
    match &patch.save {
        FactValue::Missing => {}
        FactValue::Null => current.save = FactValue::Null,
        FactValue::Value(SpellSourceValue::Unsupported(_)) => return Err(()),
        FactValue::Value(SpellSourceValue::Known(patch)) => {
            let mut save = match &current.save {
                FactValue::Value(SpellSourceValue::Known(save)) => save.clone(),
                FactValue::Missing | FactValue::Null => SpellSave::default(),
                FactValue::Value(SpellSourceValue::Unsupported(_)) => return Err(()),
            };
            apply_fields!(save, patch, statistic, basic)?;
            current.save = FactValue::Value(SpellSourceValue::Known(save));
        }
    }
    Ok(())
}

fn apply_duration_patch(current: &mut SpellDuration, patch: &SpellDurationPatch) -> Result<(), ()> {
    apply_fields!(current, patch, value, sustained)
}

fn apply_damage_patch(
    resolved: &mut SpellResolvedField<SpellFact<Vec<SpellOrderedMember<SpellDamage>>>>,
    patch: &SpellFact<SpellKeyedPatch<SpellDamagePatch>>,
    source: SpellFormPatchSource,
) {
    if matches!(resolved, SpellResolvedField::Unavailable(_)) || matches!(patch, FactValue::Missing)
    {
        return;
    }
    let unavailable = |reason| {
        SpellResolvedField::Unavailable(SpellFormUnavailable {
            field: SpellFormField::Damage,
            source,
            reason,
        })
    };
    match patch {
        FactValue::Missing => {}
        FactValue::Null => *resolved = SpellResolvedField::Available(FactValue::Null),
        FactValue::Value(SpellSourceValue::Unsupported(_)) => {
            *resolved = unavailable(SpellFormUnavailableReason::UnsupportedPatch)
        }
        FactValue::Value(SpellSourceValue::Known(patch)) => {
            let SpellResolvedField::Available(current) = resolved else {
                return;
            };
            let mut members = match current {
                FactValue::Value(SpellSourceValue::Known(members)) => members.clone(),
                FactValue::Missing | FactValue::Null => Vec::new(),
                FactValue::Value(SpellSourceValue::Unsupported(_)) => {
                    *resolved = unavailable(SpellFormUnavailableReason::UnsupportedPatch);
                    return;
                }
            };
            match merge_keyed(&mut members, patch, apply_damage_member_patch) {
                Ok(()) => *current = FactValue::Value(SpellSourceValue::Known(members)),
                Err(reason) => *resolved = unavailable(reason),
            }
        }
    }
}

fn apply_damage_member_patch(
    current: &mut SpellDamage,
    patch: &SpellDamagePatch,
) -> Result<(), ()> {
    apply_fields!(
        current,
        patch,
        apply_mod,
        category,
        formula,
        kinds,
        materials,
        damage_type
    )
}

fn apply_heightening_patch(
    resolved: &mut SpellResolvedField<SpellFact<SpellHeightening>>,
    patch: &SpellFact<SpellHeighteningPatch>,
    source: SpellFormPatchSource,
) {
    if matches!(resolved, SpellResolvedField::Unavailable(_)) || matches!(patch, FactValue::Missing)
    {
        return;
    }
    let unavailable = |reason| {
        SpellResolvedField::Unavailable(SpellFormUnavailable {
            field: SpellFormField::Heightening,
            source,
            reason,
        })
    };
    match patch {
        FactValue::Missing => {}
        FactValue::Null => *resolved = SpellResolvedField::Available(FactValue::Null),
        FactValue::Value(SpellSourceValue::Unsupported(_)) => {
            *resolved = unavailable(SpellFormUnavailableReason::UnsupportedPatch)
        }
        FactValue::Value(SpellSourceValue::Known(patch)) => {
            let SpellResolvedField::Available(current) = resolved else {
                return;
            };
            let Some(SpellHeightening::Interval(interval)) = known_fact(current).cloned() else {
                *resolved = unavailable(SpellFormUnavailableReason::IncompatibleHeightening);
                return;
            };
            let mut interval = interval;
            match &patch.kind {
                FactValue::Missing
                | FactValue::Value(SpellSourceValue::Known(SpellHeighteningType::Interval)) => {}
                FactValue::Null | FactValue::Value(SpellSourceValue::Unsupported(_)) => {
                    *resolved = unavailable(SpellFormUnavailableReason::UnsupportedPatch);
                    return;
                }
            }
            if apply_leaf(&mut interval.interval, &patch.interval).is_err()
                || apply_leaf(&mut interval.area, &patch.area).is_err()
            {
                *resolved = unavailable(SpellFormUnavailableReason::UnsupportedPatch);
                return;
            }
            match &patch.interval_damage {
                FactValue::Missing => {}
                FactValue::Null => interval.damage = FactValue::Null,
                FactValue::Value(SpellSourceValue::Unsupported(_)) => {
                    *resolved = unavailable(SpellFormUnavailableReason::UnsupportedPatch);
                    return;
                }
                FactValue::Value(SpellSourceValue::Known(patch)) => {
                    let mut members = match &interval.damage {
                        FactValue::Value(SpellSourceValue::Known(members)) => members.clone(),
                        FactValue::Missing | FactValue::Null => Vec::new(),
                        FactValue::Value(SpellSourceValue::Unsupported(_)) => {
                            *resolved = unavailable(SpellFormUnavailableReason::UnsupportedPatch);
                            return;
                        }
                    };
                    if let Err(reason) = merge_keyed(&mut members, patch, apply_text_member_patch) {
                        *resolved = unavailable(reason);
                        return;
                    }
                    interval.damage = FactValue::Value(SpellSourceValue::Known(members));
                }
            }
            *current = FactValue::Value(SpellSourceValue::Known(SpellHeightening::Interval(
                interval,
            )));
        }
    }
}

fn apply_rule_patch(
    resolved: &mut SpellResolvedField<SpellFact<Vec<SpellRuleElement>>>,
    patch: &SpellFact<Vec<SpellRuleElement>>,
    source: SpellFormPatchSource,
) {
    if matches!(resolved, SpellResolvedField::Unavailable(_)) || matches!(patch, FactValue::Missing)
    {
        return;
    }
    let unavailable = || {
        SpellResolvedField::Unavailable(SpellFormUnavailable {
            field: SpellFormField::Rules,
            source,
            reason: SpellFormUnavailableReason::UnsupportedPatch,
        })
    };
    match patch {
        FactValue::Missing => {}
        FactValue::Null => *resolved = SpellResolvedField::Available(FactValue::Null),
        FactValue::Value(SpellSourceValue::Unsupported(_)) => *resolved = unavailable(),
        FactValue::Value(SpellSourceValue::Known(rules)) => {
            if rules.iter().any(|element| !rule_element_is_safe(element)) {
                *resolved = unavailable();
            } else {
                *resolved = SpellResolvedField::Available(FactValue::Value(
                    SpellSourceValue::Known(rules.clone()),
                ));
            }
        }
    }
}

fn apply_text_member_patch(current: &mut String, patch: &SpellTextPatch) -> Result<(), ()> {
    match &patch.value {
        FactValue::Value(SpellSourceValue::Known(value)) => {
            *current = value.clone();
            Ok(())
        }
        FactValue::Missing
        | FactValue::Null
        | FactValue::Value(SpellSourceValue::Unsupported(_)) => Err(()),
    }
}

fn merge_keyed<T: Clone + Default, P>(
    current: &mut Vec<SpellOrderedMember<T>>,
    patch: &SpellKeyedPatch<P>,
    apply: fn(&mut T, &P) -> Result<(), ()>,
) -> Result<(), SpellFormUnavailableReason> {
    duplicate_key(current.iter().map(|member| member.key.as_str()))?;
    duplicate_key(patch.members.iter().map(|member| member.key.as_str()))?;
    let mut positions = current
        .iter()
        .enumerate()
        .map(|(index, member)| (member.key.clone(), index))
        .collect::<BTreeMap<_, _>>();
    for member in &patch.members {
        match &member.operation {
            SpellKeyedPatchOperation::Delete => {
                if let Some(index) = positions.remove(&member.key) {
                    current.remove(index);
                    positions = current
                        .iter()
                        .enumerate()
                        .map(|(index, member)| (member.key.clone(), index))
                        .collect();
                }
            }
            SpellKeyedPatchOperation::Unsupported(_) => {
                return Err(SpellFormUnavailableReason::UnsupportedPatch);
            }
            SpellKeyedPatchOperation::Merge(patch) => {
                if let Some(index) = positions.get(&member.key).copied() {
                    apply(&mut current[index].value, patch)
                        .map_err(|_| SpellFormUnavailableReason::UnsupportedPatch)?;
                } else {
                    let mut value = T::default();
                    apply(&mut value, patch)
                        .map_err(|_| SpellFormUnavailableReason::UnsupportedPatch)?;
                    current.push(SpellOrderedMember {
                        key: member.key.clone(),
                        authored_order: member.authored_order,
                        value,
                    });
                    positions.insert(member.key.clone(), current.len() - 1);
                }
            }
        }
    }
    Ok(())
}

fn duplicate_key<'a>(
    keys: impl Iterator<Item = &'a str>,
) -> Result<(), SpellFormUnavailableReason> {
    let mut seen = BTreeSet::new();
    for key in keys {
        if !seen.insert(key) {
            return Err(SpellFormUnavailableReason::DuplicateKey(key.to_string()));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use atlas_domain::{PackName, RecordId, RecordKey};

    use super::*;

    fn known<T>(value: T) -> SpellFact<T> {
        FactValue::Value(SpellSourceValue::Known(value))
    }

    fn unsupported(value: &str) -> UnsupportedSourceValue {
        UnsupportedSourceValue {
            shape: crate::UnsupportedSourceShape::String,
            value: value.to_string(),
            reason: crate::UnsupportedSourceReason::SourceFieldDrift,
        }
    }

    fn record_key() -> RecordKey {
        RecordKey::new(
            PackName::new("spells-srd").expect("pack"),
            RecordId::new("rfZpqmj0AIIdkVIs").expect("id"),
        )
    }

    fn fixture_spell(heightening: SpellHeightening, overlays: Vec<SpellOverlay>) -> SpellRecord {
        SpellRecord {
            identity: SpellIdentity {
                record_key: record_key(),
                source_id: SpellSourceId::new("rfZpqmj0AIIdkVIs").expect("source id"),
                name: "Heal".to_string(),
            },
            definition: SpellDefinition {
                source_context: SpellSourceContext::default(),
                classification: known(SpellClassification {
                    rank: known(1),
                    traits: known(vec![SpellTrait::new("healing").expect("trait")]),
                    traditions: known(vec![SpellTradition::new("divine").expect("tradition")]),
                }),
                casting: known(SpellCasting {
                    time: known("1 to 3".to_string()),
                    ..SpellCasting::default()
                }),
                targeting: known(SpellTargeting {
                    range: known(SpellRangeValue::from_authored_text("varies")),
                    area: FactValue::Null,
                    ..SpellTargeting::default()
                }),
                defense: known(SpellDefenseValue::default()),
                damage: known(vec![SpellOrderedMember {
                    key: "0".to_string(),
                    authored_order: 0,
                    value: SpellDamage {
                        formula: known("1d8".to_string()),
                        ..SpellDamage::default()
                    },
                }]),
                duration: known(SpellDuration::default()),
                heightening: known(heightening),
                overlays: known(overlays),
                ritual: FactValue::Null,
                rules: known(Vec::new()),
                content: OwnedRichContent::default(),
                unsupported_notes: Vec::new(),
                provenance: SpellProvenance {
                    source_path: "packs/spells/1st-rank/heal.json".to_string(),
                    source_contract_version: "pf2e-serialized-source/v1".to_string(),
                    source_system_version: "6.12.4".to_string(),
                    source_upstream_commit: "4cbdaa37d6c33e9519561bae2c59a23e0288cbce".to_string(),
                    standalone_location: FactValue::Missing,
                },
            },
        }
    }

    fn touch_overlay() -> SpellOverlay {
        let key = "7qUa78M9vP8T3Z6S";
        let overlay_id = SpellOverlayId::new(key).expect("overlay id");
        SpellOverlay {
            key: key.to_string(),
            authored_order: 1,
            overlay_id,
            source_id: known(SpellOverlayId::new(key).expect("source id")),
            sort: known(1),
            name: FactValue::Missing,
            overlay_type: known(SpellOverlayType::Override),
            patch: SpellPatch {
                casting: known(SpellCastingPatch {
                    time: known("1".to_string()),
                    ..SpellCastingPatch::default()
                }),
                targeting: known(SpellTargetingPatch {
                    range: known(SpellRangeValue::from_authored_text("touch")),
                    ..SpellTargetingPatch::default()
                }),
                ..SpellPatch::default()
            },
        }
    }

    #[test]
    fn range_derivation_is_bounded_and_preserves_authored_text() {
        let touch = SpellRangeValue::from_authored_text("touch");
        assert_eq!(touch.authored_text, "touch");
        assert_eq!(touch.numeric.as_ref().map(|range| range.feet), Some(5));
        assert_eq!(
            touch.numeric.as_ref().map(|range| range.kind),
            Some(SpellNumericRangeKind::TouchMelee)
        );

        let feet = SpellRangeValue::from_authored_text("120 feet");
        assert_eq!(feet.numeric.as_ref().map(|range| range.feet), Some(120));
        assert_eq!(
            feet.numeric.as_ref().map(|range| range.rule.as_str()),
            Some(SPELL_RANGE_DERIVATION_RULE)
        );
        for unsupported in ["varies", "within 30 feet", "30-foot cone", "1 mile"] {
            assert_eq!(
                SpellRangeValue::from_authored_text(unsupported).numeric,
                None,
                "{unsupported} must not be normalized"
            );
        }
    }

    #[test]
    fn consumable_child_keeps_parent_local_identity_location_and_authenticated_target() {
        let child = ConsumableSpellChild {
            parent_record_key: RecordKey::parse("equipment-srd:eOtQtVRLeGH39dNx").expect("parent"),
            child_id: SpellChildId::new("7w37duycMs4YOBeu").expect("child id"),
            name: known("Heal".to_string()),
            authored_order: 0,
            location: known(ConsumableSpellLocation {
                value: FactValue::Null,
                heightened_rank: known(4),
            }),
            standalone_locator: known(
                StableSourceLocator::new("Compendium.pf2e.spells-srd.Item.rfZpqmj0AIIdkVIs")
                    .expect("locator"),
            ),
            standalone_target: FactValue::Value(SpellStandaloneTarget::Resolved(record_key())),
            definition: fixture_spell(
                SpellHeightening::Interval(SpellIntervalHeightening::default()),
                Vec::new(),
            )
            .definition,
        };

        assert_eq!(
            child.parent_record_key.to_string(),
            "equipment-srd:eOtQtVRLeGH39dNx"
        );
        assert_eq!(child.child_id.as_str(), "7w37duycMs4YOBeu");
        assert!(matches!(
            child.standalone_target,
            FactValue::Value(SpellStandaloneTarget::Resolved(ref key)) if key == &record_key()
        ));
        assert_eq!(
            known_fact(&child.location).and_then(|location| known_fact(&location.heightened_rank)),
            Some(&4)
        );
    }

    #[test]
    fn fixed_heightening_requires_known_base_rank() {
        let mut spell = fixture_spell(SpellHeightening::Fixed(Vec::new()), Vec::new());
        let FactValue::Value(SpellSourceValue::Known(classification)) =
            &mut spell.definition.classification
        else {
            panic!("classification")
        };
        classification.rank = FactValue::Missing;
        assert_eq!(
            spell.resolve_form(
                SpellFormId::base(&record_key()),
                SpellFormContext {
                    cast_rank: 5,
                    overlay_id: None,
                }
            ),
            Err(SpellFormSelectionError::BaseRankUnavailable)
        );
    }

    #[test]
    fn authored_overlay_map_key_is_identity_when_nested_source_id_is_missing() {
        let mut overlay = touch_overlay();
        overlay.source_id = FactValue::Missing;
        let overlay_id = overlay.overlay_id.clone();
        let spell = fixture_spell(
            SpellHeightening::Interval(SpellIntervalHeightening::default()),
            vec![overlay],
        );
        let form = spell
            .resolve_form(
                SpellFormId::overlay(&record_key(), &overlay_id),
                SpellFormContext {
                    cast_rank: 1,
                    overlay_id: Some(overlay_id),
                },
            )
            .expect("map-key-identified overlay");
        let SpellResolvedField::Available(FactValue::Value(SpellSourceValue::Known(casting))) =
            form.casting
        else {
            panic!("casting remains available")
        };
        assert_eq!(
            casting
                .time
                .as_value()
                .and_then(SpellSourceValue::as_known)
                .map(String::as_str),
            Some("1")
        );
    }

    #[test]
    fn unknown_fixed_rank_localizes_every_field_its_patch_could_change() {
        let spell = fixture_spell(
            SpellHeightening::Fixed(vec![SpellFixedHeighteningLayer {
                key: "future-rank".to_string(),
                authored_order: 0,
                rank: SpellSourceValue::Unsupported(unsupported("future-rank")),
                patch: SpellPatch {
                    casting: known(SpellCastingPatch {
                        time: known("reaction".to_string()),
                        ..SpellCastingPatch::default()
                    }),
                    targeting: known(SpellTargetingPatch {
                        target: known("one creature".to_string()),
                        ..SpellTargetingPatch::default()
                    }),
                    ..SpellPatch::default()
                },
            }]),
            Vec::new(),
        );
        let form = spell
            .resolve_form(
                SpellFormId::base(&record_key()),
                SpellFormContext {
                    cast_rank: 5,
                    overlay_id: None,
                },
            )
            .expect("localized unknown fixed rank");
        for unavailable in [
            match form.casting {
                SpellResolvedField::Unavailable(value) => value,
                SpellResolvedField::Available(_) => panic!("casting must be unavailable"),
            },
            match form.targeting {
                SpellResolvedField::Unavailable(value) => value,
                SpellResolvedField::Available(_) => panic!("targeting must be unavailable"),
            },
        ] {
            assert_eq!(unavailable.source, SpellFormPatchSource::FixedHeightening);
            assert_eq!(
                unavailable.reason,
                SpellFormUnavailableReason::UnknownFixedRank("future-rank".to_string())
            );
        }
        assert!(matches!(form.heightening, SpellResolvedField::Available(_)));
    }

    #[test]
    fn fixed_rank_key_mismatch_localizes_every_field_its_patch_could_change() {
        let spell = fixture_spell(
            SpellHeightening::Fixed(vec![SpellFixedHeighteningLayer {
                key: "5".to_string(),
                authored_order: 0,
                rank: SpellSourceValue::Known(8),
                patch: SpellPatch {
                    casting: known(SpellCastingPatch {
                        time: known("three-actions".to_string()),
                        ..SpellCastingPatch::default()
                    }),
                    targeting: known(SpellTargetingPatch {
                        target: known("one creature".to_string()),
                        ..SpellTargetingPatch::default()
                    }),
                    ..SpellPatch::default()
                },
            }]),
            Vec::new(),
        );
        let form = spell
            .resolve_form(
                SpellFormId::base(&record_key()),
                SpellFormContext {
                    cast_rank: 8,
                    overlay_id: None,
                },
            )
            .expect("localized fixed-rank identity mismatch");
        for unavailable in [
            match form.casting {
                SpellResolvedField::Unavailable(value) => value,
                SpellResolvedField::Available(_) => panic!("casting must be unavailable"),
            },
            match form.targeting {
                SpellResolvedField::Unavailable(value) => value,
                SpellResolvedField::Available(_) => panic!("targeting must be unavailable"),
            },
        ] {
            assert_eq!(unavailable.source, SpellFormPatchSource::FixedHeightening);
            assert_eq!(
                unavailable.reason,
                SpellFormUnavailableReason::FixedRankKeyMismatch {
                    key: "5".to_string(),
                    rank: 8,
                }
            );
        }
        assert!(form.applied_fixed_ranks.is_empty());
        assert!(matches!(form.heightening, SpellResolvedField::Available(_)));
    }

    #[test]
    fn unsupported_base_members_localize_form_fields_without_defaulting() {
        let interval = SpellHeightening::Interval(SpellIntervalHeightening {
            interval: known(1),
            area: known(5),
            damage: FactValue::Value(SpellSourceValue::Unsupported(unsupported(
                "unknown interval damage",
            ))),
        });
        let mut overlay = touch_overlay();
        overlay.patch.defense = known(SpellDefensePatch {
            save: known(SpellSavePatch {
                basic: known(true),
                ..SpellSavePatch::default()
            }),
            ..SpellDefensePatch::default()
        });
        overlay.patch.damage = known(SpellKeyedPatch {
            members: Vec::new(),
        });
        let mut spell = fixture_spell(interval, vec![overlay.clone()]);
        spell.definition.targeting = known(SpellTargeting {
            area: known(SpellAreaValue {
                unsupported_notes: vec![SpellUnsupportedSourceFact {
                    field: SpellUnsupportedSourceField::AreaMember,
                    source_path: "system.area.futureAreaMember".to_string(),
                    authored_key: "futureAreaMember".to_string(),
                    authored_order: Some(2),
                    value: unsupported("burst"),
                }],
                ..SpellAreaValue::default()
            }),
            ..SpellTargeting::default()
        });
        spell.definition.defense = known(SpellDefenseValue {
            save: FactValue::Value(SpellSourceValue::Unsupported(unsupported("save drift"))),
            ..SpellDefenseValue::default()
        });
        spell.definition.damage =
            FactValue::Value(SpellSourceValue::Unsupported(unsupported("damage drift")));
        let form = spell
            .resolve_form(
                overlay.form_id(&record_key()),
                SpellFormContext {
                    cast_rank: 1,
                    overlay_id: Some(overlay.overlay_id),
                },
            )
            .expect("localized unavailable fields");

        macro_rules! assert_base_unavailable {
            ($field:expr) => {
                assert!(matches!(
                    $field,
                    SpellResolvedField::Unavailable(SpellFormUnavailable {
                        source: SpellFormPatchSource::Base,
                        reason: SpellFormUnavailableReason::UnsupportedPatch,
                        ..
                    })
                ))
            };
        }
        assert_base_unavailable!(form.targeting);
        assert_base_unavailable!(form.defense);
        assert_base_unavailable!(form.damage);
        assert_base_unavailable!(form.heightening);
        assert!(matches!(form.casting, SpellResolvedField::Available(_)));
    }

    #[test]
    fn matching_legacy_area_type_is_fidelity_evidence_but_mismatch_is_unavailable() {
        let burst = SpellAreaType::new("burst").expect("area type");
        let layer = |legacy_area_type| SpellFixedHeighteningLayer {
            key: "5".to_string(),
            authored_order: 0,
            rank: SpellSourceValue::Known(5),
            patch: SpellPatch {
                targeting: known(SpellTargetingPatch {
                    area: known(SpellAreaPatch {
                        value: known(10),
                        area_type: known(burst.clone()),
                        legacy_area_type: known(SpellLegacyAreaType {
                            source_path: "system.heightening.levels.5.area.areaType".to_string(),
                            authored_key: "areaType".to_string(),
                            authored_order: 2,
                            value: legacy_area_type,
                        }),
                        details: FactValue::Missing,
                        unsupported_notes: Vec::new(),
                    }),
                    ..SpellTargetingPatch::default()
                }),
                ..SpellPatch::default()
            },
        };
        let matching = fixture_spell(
            SpellHeightening::Fixed(vec![layer(burst.clone())]),
            Vec::new(),
        )
        .resolve_form(
            SpellFormId::base(&record_key()),
            SpellFormContext {
                cast_rank: 5,
                overlay_id: None,
            },
        )
        .expect("matching legacy area type");
        let SpellResolvedField::Available(FactValue::Value(SpellSourceValue::Known(targeting))) =
            matching.targeting
        else {
            panic!("matching legacy area type must remain available")
        };
        let FactValue::Value(SpellSourceValue::Known(area)) = targeting.area else {
            panic!("resolved area")
        };
        let FactValue::Value(SpellSourceValue::Known(legacy)) = area.legacy_area_type else {
            panic!("legacy area fidelity")
        };
        assert_eq!(legacy.authored_key, "areaType");
        assert_eq!(
            legacy.source_path,
            "system.heightening.levels.5.area.areaType"
        );
        assert_eq!(legacy.authored_order, 2);
        assert_eq!(legacy.value, burst);

        let mismatched = fixture_spell(
            SpellHeightening::Fixed(vec![layer(
                SpellAreaType::new("emanation").expect("legacy area type"),
            )]),
            Vec::new(),
        )
        .resolve_form(
            SpellFormId::base(&record_key()),
            SpellFormContext {
                cast_rank: 5,
                overlay_id: None,
            },
        )
        .expect("mismatch localizes targeting");
        assert!(matches!(
            mismatched.targeting,
            SpellResolvedField::Unavailable(SpellFormUnavailable {
                field: SpellFormField::Targeting,
                source: SpellFormPatchSource::FixedHeightening,
                reason: SpellFormUnavailableReason::UnsupportedPatch,
            })
        ));
    }

    #[test]
    fn unsupported_rule_keeps_exact_key_path_and_order_and_blocks_only_rules() {
        let mut spell = fixture_spell(
            SpellHeightening::Interval(SpellIntervalHeightening::default()),
            Vec::new(),
        );
        spell.definition.rules = known(vec![SpellRuleElement {
            authored_order: 3,
            source_path: "system.rules.3".to_string(),
            authored_key: "FutureRule".to_string(),
            authored_object_json: r#"{"key":"FutureRule"}"#.to_string(),
            rule: SpellRule::Unsupported(SpellUnsupportedRule {
                authored_key: "FutureRule".to_string(),
                source_path: "system.rules.3".to_string(),
                value: unsupported(r#"{"key":"FutureRule"}"#),
            }),
        }]);
        let form = spell
            .resolve_form(
                SpellFormId::base(&record_key()),
                SpellFormContext {
                    cast_rank: 1,
                    overlay_id: None,
                },
            )
            .expect("base form");
        assert!(matches!(
            form.rules,
            SpellResolvedField::Unavailable(SpellFormUnavailable {
                field: SpellFormField::Rules,
                source: SpellFormPatchSource::Base,
                ..
            })
        ));
        assert!(matches!(form.targeting, SpellResolvedField::Available(_)));

        spell.definition.rules = known(vec![SpellRuleElement {
            authored_order: 0,
            source_path: "system.rules.0".to_string(),
            authored_key: "FutureRule".to_string(),
            authored_object_json: r#"{"key":"FutureRule"}"#.to_string(),
            rule: SpellRule::DamageDice(SpellDamageDiceRule::default()),
        }]);
        let conflicting = spell
            .resolve_form(
                SpellFormId::base(&record_key()),
                SpellFormContext {
                    cast_rank: 1,
                    overlay_id: None,
                },
            )
            .expect("conflicting typed rule envelope");
        assert!(matches!(
            conflicting.rules,
            SpellResolvedField::Unavailable(SpellFormUnavailable {
                field: SpellFormField::Rules,
                source: SpellFormPatchSource::Base,
                ..
            })
        ));
    }

    #[test]
    fn overlay_applies_before_sorted_cumulative_fixed_layers() {
        let overlay = touch_overlay();
        let fixed = SpellHeightening::Fixed(vec![
            SpellFixedHeighteningLayer {
                key: "8".to_string(),
                authored_order: 0,
                rank: SpellSourceValue::Known(8),
                patch: SpellPatch {
                    casting: known(SpellCastingPatch {
                        time: known("three-actions".to_string()),
                        ..SpellCastingPatch::default()
                    }),
                    ..SpellPatch::default()
                },
            },
            SpellFixedHeighteningLayer {
                key: "5".to_string(),
                authored_order: 1,
                rank: SpellSourceValue::Known(5),
                patch: SpellPatch {
                    targeting: known(SpellTargetingPatch {
                        target: FactValue::Null,
                        ..SpellTargetingPatch::default()
                    }),
                    ..SpellPatch::default()
                },
            },
        ]);
        let spell = fixture_spell(fixed, vec![overlay.clone()]);
        let form = spell
            .resolve_form(
                overlay.form_id(&record_key()),
                SpellFormContext {
                    cast_rank: 8,
                    overlay_id: Some(overlay.overlay_id),
                },
            )
            .expect("resolved form");
        assert_eq!(form.applied_fixed_ranks, vec![5, 8]);
        let SpellResolvedField::Available(FactValue::Value(SpellSourceValue::Known(casting))) =
            form.casting
        else {
            panic!("casting available");
        };
        assert_eq!(
            known_fact(&casting.time),
            Some(&"three-actions".to_string())
        );
        let SpellResolvedField::Available(FactValue::Value(SpellSourceValue::Known(targeting))) =
            form.targeting
        else {
            panic!("targeting available");
        };
        assert_eq!(targeting.target, FactValue::Null);
        assert_eq!(
            known_fact(&targeting.range).map(|range| range.authored_text.as_str()),
            Some("touch")
        );
    }

    #[test]
    fn keyed_patch_preserves_order_and_localizes_duplicate_failure() {
        let interval = SpellHeightening::Interval(SpellIntervalHeightening {
            interval: known(1),
            area: FactValue::Missing,
            damage: known(vec![SpellOrderedMember {
                key: "0".to_string(),
                authored_order: 0,
                value: "1d8".to_string(),
            }]),
        });
        let mut overlay = touch_overlay();
        overlay.patch.damage = known(SpellKeyedPatch {
            members: vec![
                SpellKeyedPatchMember {
                    key: "0".to_string(),
                    authored_order: 0,
                    operation: SpellKeyedPatchOperation::Merge(SpellDamagePatch {
                        formula: known("2d8".to_string()),
                        ..SpellDamagePatch::default()
                    }),
                },
                SpellKeyedPatchMember {
                    key: "bonus".to_string(),
                    authored_order: 1,
                    operation: SpellKeyedPatchOperation::Merge(SpellDamagePatch {
                        formula: known("1".to_string()),
                        ..SpellDamagePatch::default()
                    }),
                },
                SpellKeyedPatchMember {
                    key: "remove".to_string(),
                    authored_order: 2,
                    operation: SpellKeyedPatchOperation::Delete,
                },
            ],
        });
        let mut spell = fixture_spell(interval.clone(), vec![overlay.clone()]);
        let FactValue::Value(SpellSourceValue::Known(damage)) = &mut spell.definition.damage else {
            panic!("base damage")
        };
        damage.push(SpellOrderedMember {
            key: "remove".to_string(),
            authored_order: 1,
            value: SpellDamage {
                formula: known("remove me".to_string()),
                ..SpellDamage::default()
            },
        });
        let form = spell
            .resolve_form(
                overlay.form_id(&record_key()),
                SpellFormContext {
                    cast_rank: 1,
                    overlay_id: Some(overlay.overlay_id.clone()),
                },
            )
            .expect("resolved form");
        let SpellResolvedField::Available(FactValue::Value(SpellSourceValue::Known(damage))) =
            form.damage
        else {
            panic!("damage available");
        };
        assert_eq!(
            damage
                .iter()
                .map(|member| member.key.as_str())
                .collect::<Vec<_>>(),
            vec!["0", "bonus"]
        );
        assert_eq!(
            known_fact(&damage[0].value.formula),
            Some(&"2d8".to_string())
        );

        let mut duplicate = overlay;
        let FactValue::Value(SpellSourceValue::Known(patch)) = &mut duplicate.patch.damage else {
            panic!("damage patch");
        };
        patch.members.push(patch.members[0].clone());
        let spell = fixture_spell(interval, vec![duplicate.clone()]);
        let form = spell
            .resolve_form(
                duplicate.form_id(&record_key()),
                SpellFormContext {
                    cast_rank: 1,
                    overlay_id: Some(duplicate.overlay_id),
                },
            )
            .expect("form with localized unavailable damage");
        assert!(matches!(
            form.damage,
            SpellResolvedField::Unavailable(SpellFormUnavailable {
                field: SpellFormField::Damage,
                reason: SpellFormUnavailableReason::DuplicateKey(_),
                ..
            })
        ));
        assert!(matches!(form.targeting, SpellResolvedField::Available(_)));
    }

    #[test]
    fn overlay_identity_mismatch_blocks_selection_without_label_matching() {
        let interval = SpellHeightening::Interval(SpellIntervalHeightening::default());
        let mut overlay = touch_overlay();
        overlay.source_id = known(SpellOverlayId::new("different-id").expect("source id"));
        let spell = fixture_spell(interval, vec![overlay.clone()]);
        assert!(matches!(
            spell.resolve_form(
                overlay.form_id(&record_key()),
                SpellFormContext {
                    cast_rank: 1,
                    overlay_id: Some(overlay.overlay_id),
                }
            ),
            Err(SpellFormSelectionError::OverlayIdentityMismatch { .. })
        ));
    }

    #[test]
    fn overlay_order_uses_source_sort_then_authored_order() {
        let interval = SpellHeightening::Interval(SpellIntervalHeightening::default());
        let mut later = touch_overlay();
        later.key = "later".to_string();
        later.overlay_id = SpellOverlayId::new("later").expect("overlay id");
        later.source_id = known(SpellOverlayId::new("later").expect("source id"));
        later.sort = known(20);
        later.authored_order = 0;
        let mut first = touch_overlay();
        first.key = "first".to_string();
        first.overlay_id = SpellOverlayId::new("first").expect("overlay id");
        first.source_id = known(SpellOverlayId::new("first").expect("source id"));
        first.sort = known(10);
        first.authored_order = 1;
        let spell = fixture_spell(interval, vec![later, first]);
        assert_eq!(
            spell
                .ordered_overlays()
                .expect("ordered overlays")
                .iter()
                .map(|overlay| overlay.key.as_str())
                .collect::<Vec<_>>(),
            vec!["first", "later"]
        );
    }

    #[test]
    fn interval_heightening_keeps_authored_increments_without_evaluation() {
        let interval = SpellHeightening::Interval(SpellIntervalHeightening {
            interval: known(1),
            area: known(5),
            damage: known(vec![SpellOrderedMember {
                key: "0".to_string(),
                authored_order: 0,
                value: "1d8".to_string(),
            }]),
        });
        let spell = fixture_spell(interval, Vec::new());
        let form = spell
            .resolve_form(
                SpellFormId::base(&spell.identity.record_key),
                SpellFormContext {
                    cast_rank: 9,
                    overlay_id: None,
                },
            )
            .expect("base interval form");
        let SpellResolvedField::Available(FactValue::Value(SpellSourceValue::Known(
            SpellHeightening::Interval(interval),
        ))) = form.heightening
        else {
            panic!("interval heightening")
        };
        assert_eq!(interval.interval, known(1u8));
        assert_eq!(
            known_fact(&interval.damage).map(|members| members[0].value.as_str()),
            Some("1d8")
        );
        assert!(form.applied_fixed_ranks.is_empty());
    }
}
