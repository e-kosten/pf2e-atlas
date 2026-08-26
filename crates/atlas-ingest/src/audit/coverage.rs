use serde::Serialize;

use super::SourcePathCoverageDisposition;

pub(super) const COVERAGE_POLICY_VERSION: &str = "pf2e-source-coverage/v1";
pub(super) const SOURCE_COVERAGE_REGISTRY_ASSIGNMENTS: usize = 313;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(super) struct CoverageDeclaration {
    pub id: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_type: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub record_type: Option<&'static str>,
    pub path_family: &'static str,
    pub disposition: SourcePathCoverageDisposition,
    pub owner: &'static str,
    pub product_rationale: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub future_owner: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub future_plan: Option<&'static str>,
}

impl CoverageDeclaration {
    fn matches(&self, document_type: &str, record_type: &str, path: &str) -> bool {
        self.document_type
            .is_none_or(|expected| expected == document_type)
            && self
                .record_type
                .is_none_or(|expected| expected == record_type)
            && path_family_matches(self.path_family, path)
    }

    fn specificity(&self) -> usize {
        let literal_segments = self
            .path_family
            .split('.')
            .filter(|segment| !matches!(*segment, "$" | "*" | "**"))
            .count();
        let wildcard_segments = self
            .path_family
            .split('.')
            .filter(|segment| matches!(*segment, "*" | "**"))
            .count();
        literal_segments * 10_000
            + usize::from(self.document_type.is_some()) * 100
            + usize::from(self.record_type.is_some()) * 10
            + (10usize.saturating_sub(wildcard_segments))
    }

    pub(super) fn is_recursive(&self) -> bool {
        self.path_family.split('.').any(|segment| segment == "**")
    }

    pub(super) fn is_complete_family_assignment(&self) -> bool {
        (self.path_family == "$.**"
            && self.document_type.is_some()
            && self.future_owner.is_some()
            && self.future_plan.is_some())
            || matches!(self.id, "migration_bookkeeping" | "editor_cache")
    }
}

pub(super) fn declaration_for(
    document_type: &str,
    record_type: &str,
    path: &str,
) -> Option<CoverageDeclaration> {
    coverage_declarations()
        .into_iter()
        .filter(|declaration| {
            document_type != "Actor" || record_type != "npc" || !declaration.is_recursive()
        })
        .filter(|declaration| declaration.matches(document_type, record_type, path))
        .max_by(|left, right| {
            left.specificity()
                .cmp(&right.specificity())
                .then_with(|| right.id.cmp(left.id))
        })
}

pub(super) fn coverage_declarations() -> Vec<CoverageDeclaration> {
    let mut declarations = vec![
        consumed("document_id", "$._id", "source::dto + source::normalize"),
        consumed("document_name", "$.name", "source::dto + source::normalize"),
        consumed(
            "document_type",
            "$.type",
            "source::dto + source::normalize::kind",
        ),
        consumed(
            "document_folder",
            "$.folder",
            "source::dto + source::normalize",
        ),
        provenance(
            "document_image",
            "$.img",
            "source::dto",
            "Portrait/token art ingestion is outside the approved record-fidelity slice; the source envelope retains the locator for audit.",
        ),
        provenance(
            "document_sort",
            "$.sort",
            "source::dto",
            "Foundry collection sort is retained as source provenance until an authored-order owner needs it.",
        ),
        provenance(
            "document_ownership",
            "$.ownership.*",
            "source::dto",
            "Atlas has no authorization boundary; Foundry ownership is retained as provenance and must not become a visibility shortcut.",
        ),
        provenance(
            "source_stats",
            "$._stats.*",
            "source::dto",
            "Foundry source bookkeeping is retained for provenance and refresh review, not runtime semantic fallback.",
        ),
        consumed(
            "compendium_source",
            "$._stats.compendiumSource",
            "source::normalize::embedded_items + records::aliases",
        ),
        consumed(
            "source_slug",
            "$.system.slug",
            "source::normalize + records::aliases",
        ),
        consumed(
            "publication",
            "$.system.publication.*",
            "source::normalize::publication",
        ),
        consumed(
            "actor_publication",
            "$.system.details.publication.*",
            "source::normalize::publication",
        ),
        consumed(
            "description",
            "$.system.description.value",
            "source::normalize::content_sources",
        ),
        consumed(
            "description_gm",
            "$.system.description.gm",
            "source::normalize::content_sources",
        ),
        consumed(
            "details_description",
            "$.system.details.description",
            "source::normalize::content_sources",
        ),
        consumed(
            "details_blurb",
            "$.system.details.blurb",
            "source::normalize::content_sources",
        ),
        consumed(
            "details_notes",
            "$.system.details.*Notes",
            "source::normalize::content_sources",
        ),
        consumed(
            "traits_values",
            "$.system.traits.value[]",
            "source::normalize::system",
        ),
        consumed(
            "traits_rarity",
            "$.system.traits.rarity",
            "source::normalize",
        ),
        consumed(
            "record_level",
            "$.system.level.value",
            "source::normalize + records::metrics",
        ),
        consumed(
            "actor_level",
            "$.system.details.level.value",
            "source::normalize + records::metrics",
        ),
        consumed(
            "record_prerequisites",
            "$.system.prerequisites.value[]",
            "source::normalize::system",
        ),
        consumed(
            "record_category",
            "$.system.category",
            "source::normalize + source::mechanics",
        ),
        consumed(
            "record_group",
            "$.system.group",
            "source::normalize + source::mechanics",
        ),
        consumed(
            "record_base_item",
            "$.system.baseItem",
            "source::normalize + source::mechanics",
        ),
        consumed(
            "record_usage",
            "$.system.usage.value",
            "source::normalize + source::mechanics",
        ),
        consumed(
            "record_price",
            "$.system.price.value.*",
            "source::normalize::system + source::mechanics",
        ),
        consumed(
            "record_actions",
            "$.system.actions.value",
            "source::normalize::time",
        ),
        consumed(
            "record_time",
            "$.system.time.value",
            "source::normalize::time",
        ),
        consumed(
            "record_duration",
            "$.system.duration.value",
            "source::normalize::time",
        ),
        ignored(
            "migration_bookkeeping",
            "$.system._migration.**",
            "source::dto",
            "The pinned serialized source is already migrated; migration bookkeeping has no authored product meaning and remains reviewable in raw provenance.",
        ),
        ignored(
            "editor_cache",
            "$.system._source.**",
            "source::dto",
            "Foundry editor/cache scaffolding is not authored record meaning and is retained only in the immutable audit source.",
        ),
    ];

    declarations.extend(npc_declarations());
    declarations.extend(future_family_declarations());
    declarations
}

fn npc_declarations() -> Vec<CoverageDeclaration> {
    let mut declarations = vec![
        npc_consumed(
            "npc_ability_modifiers",
            "$.system.abilities.*.mod",
            "records::metrics::NPC_REMAINDER_DYNAMIC_SPECS",
        ),
        npc_consumed(
            "npc_ability_modifier_aliases",
            "$.system.abilities.*.modifier",
            "records::metrics::NPC_REMAINDER_DYNAMIC_SPECS",
        ),
        npc_consumed(
            "npc_hp_broken_threshold",
            "$.system.attributes.hp.brokenThreshold",
            "records::metrics::NPC_REMAINDER_STATIC_SPECS",
        ),
        npc_consumed(
            "npc_hp_broken",
            "$.system.attributes.hp.broken",
            "records::metrics::NPC_REMAINDER_STATIC_SPECS",
        ),
        npc_consumed(
            "npc_hp_bt",
            "$.system.attributes.hp.bt",
            "records::metrics::NPC_REMAINDER_STATIC_SPECS",
        ),
        npc_consumed(
            "npc_hardness_legacy_scalar",
            "$.system.attributes.hardness",
            "records::metrics::NPC_REMAINDER_STATIC_SPECS",
        ),
        npc_consumed(
            "npc_stealth_value",
            "$.system.attributes.stealth.value",
            "records::metrics::NPC_REMAINDER_STATIC_SPECS",
        ),
        npc_consumed(
            "npc_stealth_mod",
            "$.system.attributes.stealth.mod",
            "records::metrics::NPC_REMAINDER_STATIC_SPECS",
        ),
        npc_consumed(
            "npc_stealth_modifier",
            "$.system.attributes.stealth.modifier",
            "records::metrics::NPC_REMAINDER_STATIC_SPECS",
        ),
        npc_consumed(
            "npc_perception_mod",
            "$.system.perception.mod",
            "source::dto + source::npc_core + atlas-record::creature_projection",
        ),
        npc_provenance(
            "npc_perception_modifier",
            "$.system.perception.modifier",
            "The typed NPC contract accepts mod and the reviewed legacy value field; unsupported modifier aliases remain only in retained raw provenance.",
        ),
        npc_consumed(
            "npc_perception_value",
            "$.system.perception.value",
            "source::dto + source::npc_core + atlas-record::creature_projection",
        ),
        npc_provenance(
            "npc_saves_mod",
            "$.system.saves.*.mod",
            "The pinned serialized NPC save contract uses value; prepared-data mod aliases remain only in retained raw provenance.",
        ),
        npc_provenance(
            "npc_saves_modifier",
            "$.system.saves.*.modifier",
            "The pinned serialized NPC save contract uses value; prepared-data modifier aliases remain only in retained raw provenance.",
        ),
        npc_consumed(
            "npc_saves_value",
            "$.system.saves.*.value",
            "source::dto + source::npc_core + atlas-record::creature_projection",
        ),
        npc_provenance(
            "npc_saves_total",
            "$.system.saves.*.totalModifier",
            "The pinned serialized NPC save contract uses value; prepared-data totalModifier aliases remain only in retained raw provenance.",
        ),
        npc_provenance(
            "npc_skills_mod",
            "$.system.skills.*.mod",
            "The pinned serialized NPC skill contract uses base; prepared-data mod aliases remain only in retained raw provenance.",
        ),
        npc_provenance(
            "npc_skills_modifier",
            "$.system.skills.*.modifier",
            "The pinned serialized NPC skill contract uses base; prepared-data modifier aliases remain only in retained raw provenance.",
        ),
        npc_provenance(
            "npc_skills_value",
            "$.system.skills.*.value",
            "The pinned serialized NPC skill contract uses base; prepared-data value aliases remain only in retained raw provenance.",
        ),
        npc_consumed(
            "npc_skills_base",
            "$.system.skills.*.base",
            "source::dto + source::npc_core + atlas-record::creature_projection",
        ),
        npc_consumed(
            "npc_skills_rank",
            "$.system.skills.*.rank",
            "records::metrics::NPC_REMAINDER_DYNAMIC_SPECS + records::metrics::actor::extract_skill_proficiency_metrics",
        ),
        npc_consumed(
            "embedded_item_id",
            "$.items[]._id",
            "source::dto + source::normalize::embedded_items",
        ),
        npc_consumed(
            "embedded_item_name",
            "$.items[].name",
            "source::dto + source::normalize::embedded_items",
        ),
        npc_consumed(
            "embedded_item_type",
            "$.items[].type",
            "source::dto + source::normalize::embedded_items",
        ),
        npc_provenance(
            "embedded_item_folder",
            "$.items[].folder",
            "Foundry folder membership is non-addressable source-container provenance. Embedded item identity, authored order, ownership, and typed relationships are consumed separately, so this path hides no product field.",
        ),
        npc_consumed(
            "embedded_item_sort",
            "$.items[].sort",
            "source::npc_entities::authored_order",
        ),
        npc_consumed(
            "embedded_item_slug",
            "$.items[].system.slug",
            "source::normalize::embedded_items",
        ),
        npc_consumed(
            "embedded_item_category",
            "$.items[].system.category",
            "source::normalize::embedded_items",
        ),
        npc_consumed(
            "embedded_item_traits",
            "$.items[].system.traits.value[]",
            "source::normalize::embedded_items",
        ),
        npc_consumed(
            "embedded_item_publication",
            "$.items[].system.publication.remaster",
            "source::normalize::embedded_items",
        ),
        npc_consumed(
            "embedded_item_description",
            "$.items[].system.description.value",
            "source::normalize::content_sources",
        ),
        npc_consumed(
            "embedded_spell_description",
            "$.items[].system.spell.system.description.value",
            "source::normalize::content_sources",
        ),
        npc_consumed(
            "embedded_item_compendium",
            "$.items[]._stats.compendiumSource",
            "source::normalize::embedded_items",
        ),
        npc_provenance(
            "embedded_item_image",
            "$.items[].img",
            "The image string is a provenance-only Foundry source locator because reuse licensing is not established. Atlas does not copy, fetch, embed, or display it; that policy may be revisited if licensing changes.",
        ),
        npc_provenance(
            "npc_actor_image_locator",
            "$.img",
            "The image string is a provenance-only Foundry source locator because reuse licensing is not established. Atlas does not copy, fetch, embed, or display it; that policy may be revisited if licensing changes.",
        ),
    ];
    declarations.extend(npc_exact_recursive_replacements());
    declarations
}

fn npc_exact_recursive_replacements() -> Vec<CoverageDeclaration> {
    let mut declarations = Vec::new();
    declarations.extend(
        [
            "$.system.abilities.cha.value",
            "$.system.abilities.con.value",
            "$.system.abilities.dex.value",
            "$.system.abilities.int.value",
            "$.system.abilities.str.value",
            "$.system.abilities.wis.value",
            "$.system.attributes.adjustment",
            "$.system.attributes.allSaves.value",
            "$.system.attributes.hardness.value",
            "$.system.attributes.shield.ac",
            "$.system.attributes.shield.brokenThreshold",
            "$.system.attributes.shield.hardness",
            "$.system.attributes.shield.max",
            "$.system.attributes.shield.value",
            "$.system.details.alliance",
            "$.system.initiative.statistic",
            "$.system.perception.details",
            "$.system.perception.vision",
            "$.system.resources.*.max",
            "$.system.resources.*.max.max",
            "$.system.resources.*.max.value",
            "$.system.resources.*.maxx",
            "$.system.resources.*.value",
            "$.system.saves.*.saveDetail",
            "$.system.skills.*.note",
            "$.system.skills.*.special[].base",
            "$.system.skills.*.special[].label",
            "$.system.skills.*.special[].predicate[]",
            "$.system.skills.*.special[].predicate[].gte[]",
            "$.system.skills.*.special[].predicate[].not",
            "$.system.skills.*.special[].predicate[].or[]",
        ]
        .into_iter()
        .map(|path| npc_consumed("npc_core_canonical_fact", path, "source::npc_core")),
    );
    declarations.extend(
        [
            ("$.system.attributes.ac.details", "source::npc_core"),
            (
                "$.system.attributes.ac.value",
                "source::dto + source::npc_core + atlas-record::creature_projection",
            ),
            ("$.system.attributes.hp.details", "source::npc_core"),
            (
                "$.system.attributes.hp.max",
                "source::dto + source::npc_core + atlas-record::creature_projection",
            ),
            ("$.system.attributes.hp.temp", "source::npc_core"),
            ("$.system.attributes.hp.tempmax", "source::npc_core"),
            (
                "$.system.attributes.hp.value",
                "source::dto + source::npc_core + atlas-record::creature_projection",
            ),
            (
                "$.system.attributes.immunities[].exceptions[]",
                "source::npc_core",
            ),
            (
                "$.system.attributes.immunities[].type",
                "source::dto + source::npc_core + atlas-record::creature_projection",
            ),
            (
                "$.system.attributes.resistances[].doubleVs[]",
                "source::npc_core",
            ),
            (
                "$.system.attributes.resistances[].exceptions[]",
                "source::npc_core",
            ),
            (
                "$.system.attributes.resistances[].type",
                "source::dto + source::npc_core + atlas-record::creature_projection",
            ),
            (
                "$.system.attributes.resistances[].value",
                "source::npc_core",
            ),
            ("$.system.attributes.speed.details", "source::npc_core"),
            (
                "$.system.attributes.speed.otherSpeeds[].label",
                "source::npc_core",
            ),
            (
                "$.system.attributes.speed.otherSpeeds[].type",
                "source::dto + source::npc_core + atlas-record::creature_projection",
            ),
            (
                "$.system.attributes.speed.otherSpeeds[].value",
                "source::dto + source::npc_core + atlas-record::creature_projection",
            ),
            (
                "$.system.attributes.speed.value",
                "source::dto + source::npc_core + atlas-record::creature_projection",
            ),
            (
                "$.system.attributes.weaknesses[].type",
                "source::dto + source::npc_core + atlas-record::creature_projection",
            ),
            ("$.system.attributes.weaknesses[].value", "source::npc_core"),
            ("$.system.details.languages.details", "source::npc_core"),
            (
                "$.system.details.languages.value[]",
                "source::dto + source::npc_core + atlas-record::creature_projection",
            ),
            (
                "$.system.details.publication.license",
                "source::normalize::publication",
            ),
            (
                "$.system.details.publication.remaster",
                "source::normalize::publication",
            ),
            (
                "$.system.details.publication.title",
                "source::normalize::publication",
            ),
            ("$.system.perception.senses[].acuity", "source::npc_core"),
            (
                "$.system.perception.senses[].range",
                "source::dto + source::npc_core + atlas-record::creature_projection",
            ),
            (
                "$.system.perception.senses[].type",
                "source::dto + source::npc_core + atlas-record::creature_projection",
            ),
            (
                "$.system.traits.size.value",
                "source::dto + source::npc_core + atlas-record::creature_projection",
            ),
        ]
        .into_iter()
        .map(|(path, owner)| npc_consumed("npc_exact_consumed_leaf", path, owner)),
    );
    declarations.push(npc_provenance(
        "npc_publication_authors_provenance",
        "$.system.details.publication.authors",
        "Two exact author strings occur on only nine of 5,492 pinned NPC records and do not currently provide sufficient product usefulness for a canonical field or projection. Preserve exact source attribution and corpus counts as provenance; revisit if coverage or product usefulness materially increases.",
    ));
    #[rustfmt::skip]
    declarations.extend(
        [
            ("$.items[].system.acBonus", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.actions.value", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.actionType.value", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.active", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.apex.attribute", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.area.details", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.area.type", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.area.value", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.attackEffects.custom", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.attackEffects.value[]", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.autoHeightenLevel.value", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.badge.type", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.badge.value", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.baseItem", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.bonus.total", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.bonus.value", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.bonusDamage.value", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.bulk.capacity", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.bulk.heldOrStowed", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.bulk.ignored", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.bulk.value", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.checkPenalty", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.collapsed", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.containerId", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.cost.value", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.counteraction", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.damage.*", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.damage.*.applyMod", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.damage.*.category", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.damage.*.faces", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.damage.*.formula", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.damage.*.kinds[]", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.damage.*.materials[]", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.damage.*.number", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.damage.*.type", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.damageRolls.*.category", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.damageRolls.*.damage", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.damageRolls.*.damageType", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.deathNote", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.defense.passive.statistic", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.defense.save.basic", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.defense.save.statistic", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.description.gm", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.dexCap", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.displayLevels.[\"0\"]", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.displayLevels.[\"1\"]", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.duration.expiry", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.duration.perpetual", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.duration.sustained", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.duration.unit", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.duration.value", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.equipped.carryType", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.equipped.handsHeld", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.equipped.inSlot", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.equipped.invested", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.focus.points", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.focus.pool", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.frequency.max", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.frequency.per", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.frequency.value", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.group", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.hardness", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.heightening.area", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.heightening.damage.*", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.heightening.interval", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.heightening.levels.[\"10\"].area.areaType", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.heightening.levels.[\"10\"].area.type", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.heightening.levels.[\"10\"].area.value", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.heightening.levels.[\"10\"].damage.*.applyMod", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.heightening.levels.[\"10\"].damage.*.formula", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.heightening.levels.[\"10\"].damage.*.type", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.heightening.levels.[\"10\"].range.value", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.heightening.levels.[\"2\"].target.value", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.heightening.levels.[\"3\"].area.areaType", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.heightening.levels.[\"3\"].area.type", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.heightening.levels.[\"3\"].area.value", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.heightening.levels.[\"3\"].damage.*.applyMod", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.heightening.levels.[\"3\"].damage.*.category", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.heightening.levels.[\"3\"].damage.*.formula", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.heightening.levels.[\"3\"].damage.*.type", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.heightening.levels.[\"3\"].range.value", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.heightening.levels.[\"3\"].target.value", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.heightening.levels.[\"3\"].time.value", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.heightening.levels.[\"4\"].area.areaType", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.heightening.levels.[\"4\"].area.type", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.heightening.levels.[\"4\"].area.value", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.heightening.levels.[\"4\"].damage.*.applyMod", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.heightening.levels.[\"4\"].damage.*.formula", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.heightening.levels.[\"4\"].damage.*.type", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.heightening.levels.[\"4\"].range.value", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.heightening.levels.[\"4\"].target.value", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.heightening.levels.[\"5\"].area.type", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.heightening.levels.[\"5\"].area.value", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.heightening.levels.[\"5\"].damage.*.applyMod", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.heightening.levels.[\"5\"].damage.*.category", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.heightening.levels.[\"5\"].damage.*.formula", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.heightening.levels.[\"5\"].damage.*.type", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.heightening.levels.[\"5\"].range.value", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.heightening.levels.[\"5\"].target.value", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.heightening.levels.[\"5\"].time.value", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.heightening.levels.[\"5\"].traits.value[]", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.heightening.levels.[\"6\"].range.value", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.heightening.levels.[\"6\"].target.value", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.heightening.levels.[\"7\"].area.areaType", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.heightening.levels.[\"7\"].area.type", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.heightening.levels.[\"7\"].area.value", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.heightening.levels.[\"7\"].damage.*.applyMod", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.heightening.levels.[\"7\"].damage.*.category", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.heightening.levels.[\"7\"].damage.*.formula", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.heightening.levels.[\"7\"].damage.*.type", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.heightening.levels.[\"7\"].range.value", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.heightening.levels.[\"7\"].target.value", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.heightening.levels.[\"8\"].damage.*.applyMod", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.heightening.levels.[\"8\"].damage.*.formula", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.heightening.levels.[\"8\"].damage.*.type", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.heightening.levels.[\"8\"].target.value", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.heightening.levels.[\"8\"].time.value", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.heightening.levels.[\"9\"].area.areaType", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.heightening.levels.[\"9\"].area.type", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.heightening.levels.[\"9\"].area.value", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.heightening.levels.[\"9\"].damage.*.applyMod", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.heightening.levels.[\"9\"].damage.*.category", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.heightening.levels.[\"9\"].damage.*.formula", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.heightening.levels.[\"9\"].damage.*.type", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.heightening.levels.[\"9\"].range.value", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.heightening.levels.[\"9\"].target.value", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.heightening.levels.[\"9\"].time.value", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.heightening.type", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.hp.max", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.hp.value", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.item.value", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.level.value", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.location.autoHeightenLevel", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.location.heightenedLevel", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.location.signature", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.location.uses.max", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.location.uses.value", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.location.value", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.material.grade", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.material.type", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.meleeUsage.damage.*", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.meleeUsage.group", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.meleeUsage.traits[]", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.mod.value", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.overlays.*._id", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.overlays.*.name", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.overlays.*.overlayType", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.overlays.*.sort", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.overlays.*.system.area.areaType", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.overlays.*.system.area.type", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.overlays.*.system.area.value", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.overlays.*.system.damage.*.applyMod", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.overlays.*.system.damage.*.formula", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.overlays.*.system.damage.*.kinds[]", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.overlays.*.system.damage.*.materials[]", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.overlays.*.system.damage.*.type", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.overlays.*.system.defense.save.basic", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.overlays.*.system.defense.save.statistic", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.overlays.*.system.heightening.damage.*", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.overlays.*.system.heightening.interval", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.overlays.*.system.heightening.type", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.overlays.*.system.range.value", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.overlays.*.system.target.value", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.overlays.*.system.time.value", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.overlays.*.system.traits.value[]", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.overrides[]", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.prepared.flexible", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.prepared.label", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.prepared.type", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.prepared.validItems", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.prepared.value", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.price.per", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.price.value.cp", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.price.value.gp", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.price.value.pp", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.price.value.sp", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.proficiency.value", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.proficient.value", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.publication.license", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.publication.title", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.quantity", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.range", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.range.value", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.reload.value", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.removable", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.requirements", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.ritual.primary.check", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.ritual.secondary.casters", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.ritual.secondary.checks", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].acuity", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].add[]", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].adjustment.all", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].adjustment.criticalFailure", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].adjustment.criticalSuccess", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].adjustment.failure", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].adjustment.success", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].adjustName", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].affects", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].allowDuplicate", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].alterations[].mode", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].alterations[].property", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].alterations[].value", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].alternate", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].alwaysActive", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].attackModifier", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].attribute", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].baseModifier.check", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].baseModifier.dc", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].baseType", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].category", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].choices[].label", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].choices[].value", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].critical", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].damage.*.damageType", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].damage.*.dice", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].damage.*.die", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].damage.*.modifier", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].damageCategory", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].damageType", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].deactivatedBy[]", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].definition[]", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].definition[].nor[]", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].definition[].not", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].definition[].or[]", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].definition[].or[].and[]", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].details", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].diceNumber", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].diesize", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].dieSize", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].disabledIf[].not", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].disabledValue", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].domain", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].doubleVs[]", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].doubleVS[]", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].effects[].affects", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].effects[].events[]", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].effects[].includesSelf", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].effects[].predicate[]", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].effects[].predicate[].lt[]", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].effects[].predicate[].not", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].effects[].predicate[].or[]", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].effects[].uuid", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].events.onTurnStart", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].exceptions[]", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].exceptions[].definition[]", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].exceptions[].definition[].not", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].exceptions[].definition[].or[]", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].exceptions[].label", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].fist", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].flag", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].fromEquipment", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].group", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].hideIfDisabled", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].img", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].inMemoryOnly", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].itemCasting.tradition", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].itemId", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].itemType", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].keep", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].key", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].label", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].level", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].mergeable", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].mode", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].option", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].otherTags[]", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].outcome[]", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].override.damageType", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].override.diceNumber", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].override.dieSize", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].path", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].phase", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].placement", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].predicate[]", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].predicate[].gt[]", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].predicate[].gte[]", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].predicate[].lt[]", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].predicate[].lte[]", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].predicate[].nor[]", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].predicate[].nor[].lt[]", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].predicate[].not", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].predicate[].not.gte[]", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].predicate[].or[]", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].predicate[].or[].and[]", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].predicate[].or[].and[].gt[]", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].predicate[].or[].and[].gte[]", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].predicate[].or[].and[].lte[]", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].predicate[].or[].and[].not", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].predicate[].or[].and[].or[]", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].predicate[].or[].gte[]", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].predicate[].or[].lt[]", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].predicate[].or[].lte[]", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].predicate[].or[].not", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].predicate[].or[].or[]", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].predicate[].xor[]", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].priority", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].prompt", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].property", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].radius", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].range", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].range.increment", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].range.max", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].reach.override", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].reevaluateOnUpdate", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].relabel", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].remove[]", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].replaceAll", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].required", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].requireInvestment", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].requiresEquipped", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].resizeEquipment", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].rollOption", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].selection", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].selector", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].selector[]", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].selectors[]", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].slug", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].suboptions[].label", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].suboptions[].value", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].suppress", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].text", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].title", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].toggleable", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].traits", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].traits[]", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].type", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].type[]", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].types", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].uuid", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].value", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].value.ac", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].value.alpha", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].value.angle", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].value.animation.intensity", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].value.animation.reverse", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].value.animation.speed", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].value.animation.type", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].value.attenuation", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].value.brackets[].end", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].value.brackets[].start", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].value.brackets[].value", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].value.bright", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].value.broken", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].value.brokenThreshold", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].value.color", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].value.coloration", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].value.contrast", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].value.darkness.max", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].value.darkness.min", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].value.destroyed", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].value.dim", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].value.field", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].value.gradual", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].value.hardness", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].value.hp.max", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].value.hp.value", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].value.icon", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].value.itemId", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].value.luminosity", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].value.name", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].value.raised", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].value.saturation", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].value.shadows", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].value.slug", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].value.source", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].value[].divider", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].value[].text", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].value[].title", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.rules[].visibility", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.runes.potency", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.runes.property[]", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.runes.reinforcing", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.runes.resilient", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.runes.striking", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.selfEffect.name", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.selfEffect.uuid", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.showSlotlessLevels.value", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.showUnpreparedSpells.value", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.size", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.slots.*.max", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.slots.*.prepared[]", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.slots.*.prepared[].expended", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.slots.*.prepared[].id", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.slots.*.prepared[].name", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.slots.*.prepared[].prepared", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.slots.*.value", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.specific.integrated.runes.potency", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.specific.integrated.runes.striking", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.specific.material.base[].thickness", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.specific.material.base[].type", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.specific.material.grade", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.specific.material.type", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.specific.runes.potency", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.specific.runes.property[]", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.specific.runes.reinforcing", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.specific.runes.resilient", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.specific.runes.striking", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.speedPenalty", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.spell._id", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.spell._stats.compendiumSource", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.spell.flags.core.sourceId", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.spell.folder", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.spell.img", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.spell.name", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.spell.sort", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.spell.system.area.details", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.spell.system.area.type", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.spell.system.area.value", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.spell.system.counteraction", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.spell.system.damage.*.applyMod", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.spell.system.damage.*.formula", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.spell.system.damage.*.kinds[]", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.spell.system.damage.*.type", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.spell.system.defense.save.basic", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.spell.system.defense.save.statistic", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.spell.system.duration.sustained", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.spell.system.duration.value", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.spell.system.heightening.area", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.spell.system.heightening.damage.*", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.spell.system.heightening.interval", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.spell.system.heightening.levels.[\"10\"].damage.*.applyMod", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.spell.system.heightening.levels.[\"10\"].damage.*.formula", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.spell.system.heightening.levels.[\"10\"].damage.*.type", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.spell.system.heightening.levels.[\"3\"].target.value", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.spell.system.heightening.levels.[\"4\"].area.areaType", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.spell.system.heightening.levels.[\"4\"].area.type", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.spell.system.heightening.levels.[\"4\"].area.value", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.spell.system.heightening.levels.[\"4\"].range.value", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.spell.system.heightening.levels.[\"5\"].range.value", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.spell.system.heightening.levels.[\"5\"].target.value", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.spell.system.heightening.levels.[\"5\"].time.value", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.spell.system.heightening.levels.[\"5\"].traits.value[]", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.spell.system.heightening.levels.[\"5\"].traits.value[].id", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.spell.system.heightening.levels.[\"5\"].traits.value[].readonly", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.spell.system.heightening.levels.[\"5\"].traits.value[].value", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.spell.system.heightening.levels.[\"7\"].target.value", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.spell.system.heightening.levels.[\"8\"].target.value", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.spell.system.heightening.type", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.spell.system.level.value", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.spell.system.location.heightenedLevel", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.spell.system.overlays.*._id", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.spell.system.overlays.*.name", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.spell.system.overlays.*.overlayType", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.spell.system.overlays.*.sort", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.spell.system.overlays.*.system.area.type", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.spell.system.overlays.*.system.area.value", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.spell.system.overlays.*.system.damage.*.applyMod", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.spell.system.overlays.*.system.damage.*.formula", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.spell.system.overlays.*.system.damage.*.kinds[]", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.spell.system.overlays.*.system.damage.*.type", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.spell.system.overlays.*.system.heightening.damage.*", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.spell.system.overlays.*.system.heightening.interval", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.spell.system.overlays.*.system.heightening.type", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.spell.system.overlays.*.system.range.value", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.spell.system.overlays.*.system.target.value", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.spell.system.overlays.*.system.time.value", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.spell.system.overlays.*.system.traits.value[]", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.spell.system.publication.license", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.spell.system.publication.remaster", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.spell.system.publication.title", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.spell.system.range.value", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.spell.system.rules[].damageType", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.spell.system.rules[].diceNumber", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.spell.system.rules[].dieSize", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.spell.system.rules[].key", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.spell.system.rules[].predicate[]", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.spell.system.rules[].selector", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.spell.system.slug", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.spell.system.target.value", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.spell.system.time.value", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.spell.system.traits.rarity", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.spell.system.traits.traditions[]", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.spell.system.traits.value[]", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.spell.type", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.spelldc.dc", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.spelldc.item", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.spelldc.label", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.spelldc.mod", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.spelldc.type", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.spelldc.value", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.splashDamage.value", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.stackGroup", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.start.initiative", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.start.value", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.stowing", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.strength", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.target.value", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.time.value", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.tokenIcon.show", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.tradition.value", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.traits.integrated.runes.potency", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.traits.integrated.runes.striking", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.traits.otherTags[]", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.traits.rarity", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.traits.selected.cantrip", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.traits.selected.curse", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.traits.selected.darkness", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.traits.selected.death", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.traits.selected.evocation", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.traits.selected.necromancy", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.traits.selected.witch", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.traits.traditions[]", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.usage.canBeAmmo", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.usage.value", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.uses.autoDestroy", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.uses.max", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.uses.value", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.value.immutable", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.value.isValued", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.value.value", "B4", "NPC embedded entity conversion"),
            ("$.items[].system.weaponType.value", "B4", "NPC embedded entity conversion"),
            ("$.system.spellcasting.rituals.dc", "B4", "NPC embedded entity conversion"),
        ]
        .into_iter()
        .map(|(path, _task, _owner)| {
            if path == "$.items[].system.description.gm" {
                npc_consumed(
                    "embedded_gm_content",
                    path,
                    "source::normalize::content_sources",
                )
            } else {
                npc_consumed(
                    "npc_embedded_entity_fact",
                    path,
                    "source::npc_entities::typed_capability_or_unsupported",
                )
            }
        }),
    );
    declarations.extend(
        [
            "$.items[].flags.core.sourceId",
            "$.items[].flags.pf2e.grantedBy.id",
            "$.items[].flags.pf2e.itemGrants.*.id",
            "$.items[].flags.pf2e.itemGrants.knockdown.id",
            "$.items[].flags.pf2e.itemGrants.knockdown2.id",
            "$.items[].flags.pf2e.itemGrants.knockdown3.id",
            "$.items[].flags.pf2e.itemGrants.prone.id",
            "$.items[].flags.pf2e.itemGrants.reactiveStrike.id",
            "$.items[].flags.pf2e.itemGrants.reactiveStrike2.id",
            "$.items[].flags.pf2e.itemGrants.reinforcedStock.id",
            "$.items[].flags.pf2e.linkedWeapon",
        ]
        .into_iter()
        .map(|path| {
            npc_consumed(
                "npc_embedded_relationship_or_locator",
                path,
                "source::npc_entities::identity_relationships",
            )
        }),
    );
    declarations.extend(
        [
            "$.items[].flags.pf2e.grantedBy.onDelete",
            "$.items[].flags.pf2e.itemGrants.*.onDelete",
            "$.items[].flags.pf2e.itemGrants.knockdown.onDelete",
            "$.items[].flags.pf2e.itemGrants.knockdown2.onDelete",
            "$.items[].flags.pf2e.itemGrants.knockdown3.onDelete",
            "$.items[].flags.pf2e.itemGrants.prone.onDelete",
            "$.items[].flags.pf2e.itemGrants.reactiveStrike.onDelete",
            "$.items[].flags.pf2e.itemGrants.reactiveStrike2.onDelete",
            "$.items[].flags.pf2e.itemGrants.reinforcedStock.onDelete",
        ]
        .into_iter()
        .map(|path| {
            npc_provenance(
                "npc_grant_lifecycle_provenance",
                path,
                "Foundry onDelete is source lifecycle-cascade configuration retained only for provenance; Atlas does not execute it. Grant and itemGrant endpoint IDs are consumed separately as typed relationships, so no product relationship is hidden here.",
            )
        }),
    );
    declarations.push(npc_provenance(
        "npc_prototype_token_name_provenance",
        "$.prototypeToken.name",
        "Foundry prototypeToken.name is a token-instance presentation label, not authoritative creature identity. $.name is consumed as canonical identity, so this path hides no product field.",
    ));
    declarations
}

fn future_family_declarations() -> Vec<CoverageDeclaration> {
    let families = [
        (
            "Actor",
            "army",
            "H9",
            "future/actors/army/plan.md",
            "H9 army actor-family plan",
        ),
        (
            "Actor",
            "character",
            "H9",
            "future/actors/character/plan.md",
            "H9 character actor-family plan",
        ),
        (
            "Actor",
            "familiar",
            "H9",
            "future/actors/familiar/plan.md",
            "H9 familiar actor-family plan",
        ),
        (
            "Actor",
            "hazard",
            "H1",
            "future/hazards/plan.md",
            "H1 hazard family plan",
        ),
        (
            "Actor",
            "loot",
            "H9",
            "future/actors/loot/plan.md",
            "H9 loot actor-family plan",
        ),
        (
            "Actor",
            "party",
            "H9",
            "future/actors/party/plan.md",
            "H9 party actor-family plan",
        ),
        (
            "Actor",
            "vehicle",
            "H9",
            "future/actors/vehicle/plan.md",
            "H9 vehicle actor-family plan",
        ),
        (
            "Item",
            "action",
            "H7",
            "future/rules-content/action/plan.md",
            "H7 action rules/content plan",
        ),
        (
            "Item",
            "affliction",
            "H7",
            "future/rules-content/affliction/plan.md",
            "H7 affliction rules/content plan",
        ),
        (
            "Item",
            "ancestry",
            "H11",
            "future/discovered/item-ancestry/plan.md",
            "H11 Item ancestry bounded plan",
        ),
        (
            "Item",
            "armor",
            "H4",
            "future/physical-items/armor-shields/plan.md",
            "H4 armor and shield plan",
        ),
        (
            "Item",
            "background",
            "H11",
            "future/discovered/item-background/plan.md",
            "H11 Item background bounded plan",
        ),
        (
            "Item",
            "backpack",
            "H6",
            "future/physical-items/backpack/plan.md",
            "H6 backpack plan",
        ),
        (
            "Item",
            "book",
            "H6",
            "future/physical-items/book/plan.md",
            "H6 book plan",
        ),
        (
            "Item",
            "campaignFeature",
            "H11",
            "future/discovered/item-campaignfeature/plan.md",
            "H11 Item campaignFeature bounded plan",
        ),
        (
            "Item",
            "class",
            "H11",
            "future/discovered/item-class/plan.md",
            "H11 Item class bounded plan",
        ),
        (
            "Item",
            "condition",
            "H7",
            "future/rules-content/condition/plan.md",
            "H7 condition rules/content plan",
        ),
        (
            "Item",
            "consumable",
            "H5",
            "future/physical-items/consumables/plan.md",
            "H5 consumable plan",
        ),
        (
            "Item",
            "deity",
            "H11",
            "future/discovered/item-deity/plan.md",
            "H11 Item deity bounded plan",
        ),
        (
            "Item",
            "effect",
            "H7",
            "future/rules-content/effect/plan.md",
            "H7 effect rules/content plan",
        ),
        (
            "Item",
            "equipment",
            "H6",
            "future/physical-items/equipment/plan.md",
            "H6 equipment plan",
        ),
        (
            "Item",
            "feat",
            "H7",
            "future/rules-content/feat/plan.md",
            "H7 feat rules/content plan",
        ),
        (
            "Item",
            "heritage",
            "H11",
            "future/discovered/item-heritage/plan.md",
            "H11 Item heritage bounded plan",
        ),
        (
            "Item",
            "kit",
            "H6",
            "future/physical-items/kit/plan.md",
            "H6 kit plan",
        ),
        (
            "Item",
            "lore",
            "H11",
            "future/discovered/item-lore/plan.md",
            "H11 Item lore bounded plan",
        ),
        (
            "Item",
            "melee",
            "H3",
            "future/physical-items/weapons-ammunition/plan.md",
            "H3 weapon and ammunition plan",
        ),
        (
            "Item",
            "shield",
            "H4",
            "future/physical-items/armor-shields/plan.md",
            "H4 armor and shield plan",
        ),
        (
            "Item",
            "spell",
            "H2",
            "future/spells-rituals/plan.md",
            "H2 spell and ritual plan",
        ),
        (
            "Item",
            "spellcastingEntry",
            "H2",
            "future/spells-rituals/plan.md",
            "H2 spell and ritual plan",
        ),
        (
            "Item",
            "treasure",
            "H6",
            "future/physical-items/treasure/plan.md",
            "H6 treasure plan",
        ),
        (
            "Item",
            "weapon",
            "H3",
            "future/physical-items/weapons-ammunition/plan.md",
            "H3 weapon and ammunition plan",
        ),
        (
            "Cards",
            "deck",
            "H11",
            "future/discovered/cards-deck/plan.md",
            "H11 cards-deck bounded plan",
        ),
        (
            "Cards",
            "hand",
            "H11",
            "future/discovered/cards-hand/plan.md",
            "H11 cards-hand bounded plan",
        ),
        (
            "Cards",
            "pile",
            "H11",
            "future/discovered/cards-pile/plan.md",
            "H11 cards-pile bounded plan",
        ),
        (
            "Macro",
            "chat",
            "H11",
            "future/discovered/macro-chat/plan.md",
            "H11 macro-chat bounded plan",
        ),
        (
            "Macro",
            "script",
            "H11",
            "future/discovered/macro-script/plan.md",
            "H11 macro-script bounded plan",
        ),
    ];
    let mut declarations = families
        .into_iter()
        .map(|(document_type, record_type, task, plan, owner)| CoverageDeclaration {
            id: owner,
            document_type: Some(document_type),
            record_type: Some(record_type),
            path_family: "$.**",
            disposition: SourcePathCoverageDisposition::Deferred,
            owner: "approved exhaustive type registry",
            product_rationale: "Implementation is creature-first, while this exact family remains assigned to a bounded future plan with fixtures, acceptance, and approval gates.",
            future_owner: Some(task),
            future_plan: Some(plan),
        })
        .collect::<Vec<_>>();

    for (document_type, task, plan, owner) in [
        (
            "JournalEntry",
            "H8",
            "future/journals-tables/journals-pages/plan.md",
            "H8 journals-pages plan",
        ),
        (
            "RollTable",
            "H8",
            "future/journals-tables/roll-tables-results/plan.md",
            "H8 roll-tables-results plan",
        ),
        (
            "Adventure",
            "H11",
            "future/discovered/adventure-root/plan.md",
            "H11 adventure-root bounded plan",
        ),
        (
            "ChatMessage",
            "H11",
            "future/discovered/chatmessage-root/plan.md",
            "H11 chatmessage-root bounded plan",
        ),
        (
            "Combat",
            "H11",
            "future/discovered/combat-root/plan.md",
            "H11 combat-root bounded plan",
        ),
        (
            "FogExploration",
            "H11",
            "future/discovered/fogexploration-root/plan.md",
            "H11 fogexploration-root bounded plan",
        ),
        (
            "Playlist",
            "H11",
            "future/discovered/playlist-root/plan.md",
            "H11 playlist-root bounded plan",
        ),
        (
            "Scene",
            "H11",
            "future/discovered/scene-root/plan.md",
            "H11 scene-root bounded plan",
        ),
        (
            "Setting",
            "H11",
            "future/discovered/setting-root/plan.md",
            "H11 setting-root bounded plan",
        ),
        (
            "User",
            "H11",
            "future/discovered/user-root/plan.md",
            "H11 user-root bounded plan",
        ),
    ] {
        declarations.push(CoverageDeclaration {
            id: owner,
            document_type: Some(document_type),
            record_type: None,
            path_family: "$.**",
            disposition: SourcePathCoverageDisposition::Deferred,
            owner: "approved exhaustive type registry",
            product_rationale: "This exact document family remains assigned to bounded future planning; B2 preserves its coverage without implementing it.",
            future_owner: Some(task),
            future_plan: Some(plan),
        });
    }
    declarations
}

fn consumed(
    id: &'static str,
    path_family: &'static str,
    owner: &'static str,
) -> CoverageDeclaration {
    CoverageDeclaration {
        id,
        document_type: None,
        record_type: None,
        path_family,
        disposition: SourcePathCoverageDisposition::Consumed,
        owner,
        product_rationale: "The named extractor consumes this source family into an existing typed source, content, mechanic, metric, identity, or provenance projection.",
        future_owner: None,
        future_plan: None,
    }
}

fn provenance(
    id: &'static str,
    path_family: &'static str,
    owner: &'static str,
    rationale: &'static str,
) -> CoverageDeclaration {
    CoverageDeclaration {
        id,
        document_type: None,
        record_type: None,
        path_family,
        disposition: SourcePathCoverageDisposition::ProvenanceOnly,
        owner,
        product_rationale: rationale,
        future_owner: None,
        future_plan: None,
    }
}

fn ignored(
    id: &'static str,
    path_family: &'static str,
    owner: &'static str,
    rationale: &'static str,
) -> CoverageDeclaration {
    CoverageDeclaration {
        id,
        document_type: None,
        record_type: None,
        path_family,
        disposition: SourcePathCoverageDisposition::IgnoredWithRationale,
        owner,
        product_rationale: rationale,
        future_owner: None,
        future_plan: None,
    }
}

fn npc_consumed(
    id: &'static str,
    path_family: &'static str,
    owner: &'static str,
) -> CoverageDeclaration {
    CoverageDeclaration {
        id,
        document_type: Some("Actor"),
        record_type: Some("npc"),
        path_family,
        disposition: SourcePathCoverageDisposition::Consumed,
        owner,
        product_rationale: "The named real extractor consumes this NPC source family into canonical facts and their one-way product projections.",
        future_owner: None,
        future_plan: None,
    }
}

fn npc_provenance(
    id: &'static str,
    path_family: &'static str,
    rationale: &'static str,
) -> CoverageDeclaration {
    CoverageDeclaration {
        id,
        document_type: Some("Actor"),
        record_type: Some("npc"),
        path_family,
        disposition: SourcePathCoverageDisposition::ProvenanceOnly,
        owner: "source::dto",
        product_rationale: rationale,
        future_owner: None,
        future_plan: None,
    }
}

fn path_family_matches(pattern: &str, path: &str) -> bool {
    let pattern = pattern.split('.').collect::<Vec<_>>();
    let path = path.split('.').collect::<Vec<_>>();
    matches_segments(&pattern, &path)
}

fn matches_segments(pattern: &[&str], path: &[&str]) -> bool {
    match (pattern.split_first(), path.split_first()) {
        (None, None) => true,
        (None, Some(_)) => false,
        (Some((&"**", pattern_rest)), _) => {
            matches_segments(pattern_rest, path)
                || path
                    .split_first()
                    .is_some_and(|(_, path_rest)| matches_segments(pattern, path_rest))
        }
        (Some((&pattern_head, pattern_rest)), Some((&path_head, path_rest))) => {
            (pattern_head == "*" || wildcard_segment_matches(pattern_head, path_head))
                && matches_segments(pattern_rest, path_rest)
        }
        (Some(_), None) => false,
    }
}

fn wildcard_segment_matches(pattern: &str, value: &str) -> bool {
    if !pattern.contains('*') {
        return pattern == value;
    }
    let mut remainder = value;
    let mut first = true;
    for part in pattern.split('*') {
        if part.is_empty() {
            first = false;
            continue;
        }
        let Some(index) = remainder.find(part) else {
            return false;
        };
        if first && index != 0 {
            return false;
        }
        remainder = &remainder[index + part.len()..];
        first = false;
    }
    pattern.ends_with('*') || remainder.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wildcard_families_match_dynamic_and_recursive_paths() {
        assert!(path_family_matches(
            "$.system.skills.*.mod",
            "$.system.skills.acrobatics.mod"
        ));
        assert!(path_family_matches(
            "$.items[].system.**",
            "$.items[].system.damage.*.damageType"
        ));
        assert!(!path_family_matches(
            "$.system.skills.*.mod",
            "$.system.skills.acrobatics.base"
        ));
    }

    #[test]
    fn exact_real_owner_beats_deferred_family() {
        let declaration = declaration_for("Actor", "npc", "$.items[].system.description.value")
            .expect("embedded description declaration");
        assert_eq!(
            declaration.disposition,
            SourcePathCoverageDisposition::Consumed
        );
        assert_eq!(declaration.owner, "source::normalize::content_sources");
    }

    #[test]
    fn every_recursive_declaration_names_a_complete_assignment_owner() {
        let incomplete = coverage_declarations()
            .into_iter()
            .filter(|declaration| {
                declaration.is_recursive() && !declaration.is_complete_family_assignment()
            })
            .map(|declaration| declaration.id)
            .collect::<Vec<_>>();
        assert!(
            incomplete.is_empty(),
            "recursive declarations without complete owners: {incomplete:?}"
        );
    }

    #[test]
    fn npc_metric_coverage_distinguishes_canonical_fields_from_prepared_aliases() {
        let canonical = declaration_for("Actor", "npc", "$.system.skills.arcana.base")
            .expect("canonical skill declaration");
        assert_eq!(
            canonical.disposition,
            SourcePathCoverageDisposition::Consumed
        );
        assert_eq!(
            canonical.owner,
            "source::dto + source::npc_core + atlas-record::creature_projection"
        );

        let prepared_alias = declaration_for("Actor", "npc", "$.system.skills.arcana.mod")
            .expect("prepared alias declaration");
        assert_eq!(
            prepared_alias.disposition,
            SourcePathCoverageDisposition::ProvenanceOnly
        );
        assert_eq!(prepared_alias.owner, "source::dto");

        let non_migrated =
            declaration_for("Actor", "npc", "$.system.attributes.hp.brokenThreshold")
                .expect("non-migrated broken-threshold declaration");
        assert_eq!(
            non_migrated.owner,
            "records::metrics::NPC_REMAINDER_STATIC_SPECS"
        );
    }

    #[test]
    fn creature_provenance_leaves_are_exact_and_product_complete() {
        let expected = [
            ("$.img", "licensing is not established"),
            ("$.items[].img", "licensing is not established"),
            ("$.items[].folder", "hides no product field"),
            (
                "$.items[].flags.pf2e.grantedBy.onDelete",
                "does not execute it",
            ),
            (
                "$.items[].flags.pf2e.itemGrants.*.onDelete",
                "does not execute it",
            ),
            ("$.prototypeToken.name", "hides no product field"),
        ];

        for (path, rationale_fragment) in expected {
            let declaration = declaration_for("Actor", "npc", path)
                .unwrap_or_else(|| panic!("missing creature provenance declaration for {path}"));
            assert_eq!(
                declaration.disposition,
                SourcePathCoverageDisposition::ProvenanceOnly,
                "{path}"
            );
            assert_eq!(declaration.owner, "source::dto", "{path}");
            assert!(
                declaration.product_rationale.contains(rationale_fragment),
                "{path}: {}",
                declaration.product_rationale
            );
            assert!(!declaration.is_recursive(), "{path}");
            assert!(!declaration.is_complete_family_assignment(), "{path}");
        }

        for path in [
            "$.items[].flags.pf2e.grantedBy.id",
            "$.items[].flags.pf2e.itemGrants.*.id",
        ] {
            let declaration = declaration_for("Actor", "npc", path)
                .unwrap_or_else(|| panic!("missing creature relationship declaration for {path}"));
            assert_eq!(
                declaration.disposition,
                SourcePathCoverageDisposition::Consumed,
                "{path}"
            );
            assert_eq!(
                declaration.owner, "source::npc_entities::identity_relationships",
                "{path}"
            );
        }
    }

    #[test]
    fn non_creature_family_has_exact_future_owner() {
        let declaration = declaration_for("Item", "weapon", "$.system.runes.potency")
            .expect("weapon future declaration");
        assert_eq!(
            declaration.disposition,
            SourcePathCoverageDisposition::Deferred
        );
        assert_eq!(declaration.future_owner, Some("H3"));
        assert_eq!(
            declaration.future_plan,
            Some("future/physical-items/weapons-ammunition/plan.md")
        );
    }

    #[test]
    fn multi_type_document_families_keep_exact_future_plans() {
        let cards = declaration_for("Cards", "hand", "$.cards[].name")
            .expect("cards-hand future declaration");
        assert_eq!(cards.future_owner, Some("H11"));
        assert_eq!(
            cards.future_plan,
            Some("future/discovered/cards-hand/plan.md")
        );

        let macro_script = declaration_for("Macro", "script", "$.command")
            .expect("macro-script future declaration");
        assert_eq!(macro_script.future_owner, Some("H11"));
        assert_eq!(
            macro_script.future_plan,
            Some("future/discovered/macro-script/plan.md")
        );
    }

    #[test]
    fn canonical_npc_core_owns_every_approved_actor_root_leaf() {
        let paths = [
            "$.system.abilities.cha.value",
            "$.system.abilities.con.value",
            "$.system.abilities.dex.value",
            "$.system.abilities.int.value",
            "$.system.abilities.str.value",
            "$.system.abilities.wis.value",
            "$.system.attributes.adjustment",
            "$.system.attributes.allSaves.value",
            "$.system.attributes.hardness.value",
            "$.system.attributes.shield.ac",
            "$.system.attributes.shield.brokenThreshold",
            "$.system.attributes.shield.hardness",
            "$.system.attributes.shield.max",
            "$.system.attributes.shield.value",
            "$.system.details.alliance",
            "$.system.initiative.statistic",
            "$.system.perception.details",
            "$.system.perception.vision",
            "$.system.resources.*.max",
            "$.system.resources.*.max.max",
            "$.system.resources.*.max.value",
            "$.system.resources.*.maxx",
            "$.system.resources.*.value",
            "$.system.saves.*.saveDetail",
            "$.system.skills.*.base",
            "$.system.skills.*.note",
            "$.system.skills.*.special[].base",
            "$.system.skills.*.special[].label",
            "$.system.skills.*.special[].predicate[]",
            "$.system.skills.*.special[].predicate[].gte[]",
            "$.system.skills.*.special[].predicate[].not",
            "$.system.skills.*.special[].predicate[].or[]",
        ];
        assert_eq!(paths.len(), 32);
        for path in paths {
            let declaration = declaration_for("Actor", "npc", path)
                .unwrap_or_else(|| panic!("missing B3 declaration for {path}"));
            assert_eq!(
                declaration.disposition,
                SourcePathCoverageDisposition::Consumed,
                "{path}"
            );
            let expected_owner = if path == "$.system.skills.*.base" {
                "source::dto + source::npc_core + atlas-record::creature_projection"
            } else {
                "source::npc_core"
            };
            assert_eq!(declaration.owner, expected_owner, "{path}");
            assert_eq!(declaration.future_owner, None, "{path}");
        }

        let rituals = declaration_for("Actor", "npc", "$.system.spellcasting.rituals.dc")
            .expect("B4 ritual DC declaration");
        assert_eq!(rituals.disposition, SourcePathCoverageDisposition::Consumed);
        assert_eq!(
            rituals.owner,
            "source::npc_entities::typed_capability_or_unsupported"
        );
        assert_eq!(rituals.future_owner, None);
    }
}
