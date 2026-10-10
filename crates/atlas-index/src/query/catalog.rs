use atlas_domain::query::QueryValueDiscovery;
use atlas_domain::{QueryCapability, QueryFieldDefinition, QueryFieldType, QueryLimits};
use atlas_foundry_model::generated::*;
use serde::de::{self, DeserializeOwned};
use std::{fmt, sync::OnceLock};

pub const QUERY_CAPABILITY_VERSION: u32 = 1;
const ACTORS: &[&str] = &[
    "npc",
    "hazard",
    "character",
    "army",
    "vehicle",
    "loot",
    "familiar",
    "party",
];
const IWR: &[&str] = &["npc", "hazard", "character", "army", "vehicle"];
const PHYSICAL: &[&str] = &[
    "armor",
    "backpack",
    "book",
    "consumable",
    "equipment",
    "shield",
    "treasure",
    "weapon",
];
const ITEMS: &[&str] = &[
    "action",
    "affliction",
    "ancestry",
    "armor",
    "background",
    "book",
    "campaignFeature",
    "class",
    "condition",
    "consumable",
    "backpack",
    "deity",
    "effect",
    "equipment",
    "feat",
    "heritage",
    "kit",
    "lore",
    "melee",
    "shield",
    "spell",
    "spellcastingEntry",
    "treasure",
    "weapon",
];
const ITEM_TRAITS: &[&str] = &[
    "action",
    "affliction",
    "ancestry",
    "armor",
    "background",
    "book",
    "campaignFeature",
    "condition",
    "consumable",
    "backpack",
    "effect",
    "equipment",
    "feat",
    "heritage",
    "kit",
    "melee",
    "shield",
    "spell",
    "treasure",
    "weapon",
];
const ITEM_RARITY: &[&str] = &[
    "ancestry",
    "armor",
    "background",
    "book",
    "class",
    "consumable",
    "backpack",
    "equipment",
    "feat",
    "heritage",
    "shield",
    "spell",
    "treasure",
    "weapon",
];

/// The generated Deserialize implementation supplies its declared variants via
/// Serde's public Error interface. No corpus vocabulary or manual enum copy.
#[derive(Debug)]
struct EnumChoices(Vec<String>);
impl fmt::Display for EnumChoices {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "generated enum metadata probe")
    }
}
impl std::error::Error for EnumChoices {}
impl de::Error for EnumChoices {
    fn custom<T: fmt::Display>(_: T) -> Self {
        Self(Vec::new())
    }
    fn unknown_variant(_: &str, variants: &'static [&'static str]) -> Self {
        Self(variants.iter().map(|s| (*s).to_owned()).collect())
    }
}
fn choices<T: DeserializeOwned>() -> Result<Vec<String>, String> {
    match T::deserialize(de::value::StrDeserializer::<EnumChoices>::new("\0")) {
        Err(EnumChoices(values)) if !values.is_empty() => Ok(values),
        _ => Err(format!(
            "catalog requires a generated closed string enum; {} did not provide variants",
            std::any::type_name::<T>()
        )),
    }
}

#[derive(Debug, Clone)]
pub(crate) enum Binding {
    Common {
        column: &'static str,
        known: bool,
    },
    Family {
        table: &'static str,
        column: &'static str,
        physical_child: bool,
    },
    Set {
        parent: Box<Binding>,
        table: &'static str,
        owner: &'static str,
        kind: Option<&'static str>,
    },
    Collection {
        parent: Box<Binding>,
        table: &'static str,
        owner: &'static str,
        kind: Option<&'static str>,
    },
    Member {
        column: &'static str,
    },
    Cantrip,
    Duration {
        unit: &'static str,
    },
}
#[derive(Debug, Clone)]
pub(crate) struct Descriptor {
    pub definition: QueryFieldDefinition,
    pub binding: Binding,
}
fn field(
    path: &str,
    ty: QueryFieldType,
    families: &[&str],
    binding: Binding,
    enum_choices: Vec<String>,
    unit: Option<&str>,
) -> Descriptor {
    let operators = match ty {
        QueryFieldType::String => vec!["eq", "neq", "in"],
        QueryFieldType::Number => vec!["eq", "neq", "lt", "lte", "gt", "gte", "between"],
        QueryFieldType::Boolean => vec!["eq", "neq"],
        QueryFieldType::Set => vec!["includes", "includes_any", "includes_all", "excludes_any"],
        QueryFieldType::Collection => vec!["exists"],
    }
    .into_iter()
    .chain(["state"])
    .map(str::to_owned)
    .collect();
    Descriptor {
        definition: QueryFieldDefinition {
            id: path.to_owned(),
            path: path.to_owned(),
            scope: None,
            field_type: ty,
            label: path.replace(['_', '.'], " "),
            family_types: families.iter().map(|f| (*f).to_owned()).collect(),
            units: unit.map(str::to_owned),
            basis: match path {
                "physical.price.per_item_cp" => "authored coin value divided by authored per; denomination omissions=0, per omission=1; bounded correctly rounded cp",
                "physical.bulk" => "authored normal Bulk; 0 negligible, 0.1 Light; no encumbrance or per-individual adjustment",
                "spell.casting_form" => "upstream trim/lowercase glyph lookup; unmatched authored time is other",
                "spell.focus" => "authored focus OR cantrip with known-empty traditions",
                "spell.ritual" => "typed ritual object true; explicit null false; omission unavailable",
                "heritage.versatile" => "typed ancestry object false; explicit null true; omission unavailable",
                "actor.immunities" | "actor.weaknesses" | "actor.resistances" => "authored collection; omission-only empty default inside known system/attributes",
                p if p.starts_with("effect.duration.") => "authored duration in the selected fixed unit; other known units not applicable",
                _ => "authored source; known values only",
            }.to_owned(),
            value_discovery: match ty {
                QueryFieldType::Number => QueryValueDiscovery::NumericStatistics,
                QueryFieldType::Boolean => QueryValueDiscovery::BooleanCounts,
                QueryFieldType::Collection => QueryValueDiscovery::CollectionStates,
                _ if enum_choices.is_empty() => QueryValueDiscovery::OpenValues,
                _ => QueryValueDiscovery::ClosedChoices,
            },
            choices: enum_choices,
            operators,
            examples: vec![format!("availability[\"{path}\"] == \"value\"")],
        },
        binding,
    }
}
fn common(column: &'static str) -> Binding {
    Binding::Common {
        column,
        known: false,
    }
}
fn family(table: &'static str, column: &'static str) -> Binding {
    Binding::Family {
        table,
        column,
        physical_child: false,
    }
}
fn set(
    parent: Binding,
    table: &'static str,
    owner: &'static str,
    kind: Option<&'static str>,
) -> Binding {
    Binding::Set {
        parent: Box::new(parent),
        table,
        owner,
        kind,
    }
}
fn collection(
    parent: Binding,
    table: &'static str,
    owner: &'static str,
    kind: Option<&'static str>,
) -> Binding {
    Binding::Collection {
        parent: Box::new(parent),
        table,
        owner,
        kind,
    }
}

fn build() -> Result<Vec<Descriptor>, String> {
    use QueryFieldType::*;
    let mut fields = Vec::new();
    for (path, col) in [
        ("source.pack.id", "pack_id"),
        ("source.pack.label", "pack_label"),
        ("source.document_kind", "document_kind"),
    ] {
        fields.push(field(
            path,
            String,
            &[],
            Binding::Common {
                column: col,
                known: true,
            },
            if path == "source.document_kind" {
                ["Actor", "Item", "JournalEntry", "RollTable"]
                    .into_iter()
                    .map(str::to_owned)
                    .collect()
            } else {
                Vec::new()
            },
            None,
        ));
    }
    let mut source_types = ITEMS.to_vec();
    source_types.extend_from_slice(ACTORS);
    fields.push(field(
        "source.type",
        String,
        &[],
        common("source_type"),
        source_types.into_iter().map(str::to_owned).collect(),
        None,
    ));
    fields.push(field(
        "record.kind",
        String,
        &[],
        common("record_kind"),
        choices::<atlas_domain::RecordKind>()?
            .into_iter()
            .filter(|s| s != "tooling")
            .collect(),
        None,
    ));
    let mut traits = ITEM_TRAITS.to_vec();
    traits.extend(["npc", "hazard", "army", "vehicle"]);
    let mut rarity = ITEM_RARITY.to_vec();
    rarity.extend(["npc", "hazard", "army", "vehicle"]);
    let mut publication = ITEMS.to_vec();
    publication.extend(["npc", "hazard", "vehicle"]);
    fields.push(field(
        "traits",
        Set,
        &traits,
        set(common("traits"), "record_traits", "record_id", None),
        Vec::new(),
        None,
    ));
    fields.push(field(
        "rarity",
        String,
        &rarity,
        common("rarity"),
        choices::<ItemTraitsItemTraitRarity>()?,
        None,
    ));
    fields.push(field(
        "publication.title",
        String,
        &publication,
        common("publication_title"),
        Vec::new(),
        None,
    ));
    fields.push(field(
        "publication.remaster",
        Boolean,
        &publication,
        common("publication_remaster"),
        Vec::new(),
        None,
    ));
    fields.push(field(
        "actor.level",
        Number,
        &["npc", "hazard", "character", "army", "vehicle", "loot"],
        common("level"),
        Vec::new(),
        Some("level"),
    ));
    let mut levels = PHYSICAL.to_vec();
    levels.extend(["feat", "campaignFeature", "effect", "affliction"]);
    fields.push(field(
        "item.level",
        Number,
        &levels,
        common("level"),
        Vec::new(),
        Some("level"),
    ));
    for (path, families) in [
        ("actor.size", &["npc", "hazard", "vehicle"][..]),
        ("physical.size", PHYSICAL),
        ("ancestry.size", &["ancestry"][..]),
    ] {
        fields.push(field(
            path,
            String,
            families,
            common("size"),
            choices::<PhysicalSystemSourceSize>()?,
            None,
        ));
    }
    for (path, column, families, unit) in [
        (
            "actor.armor_class",
            "armor_class",
            &["npc", "hazard"][..],
            None,
        ),
        (
            "actor.hp.maximum",
            "hp_maximum",
            &["npc", "hazard"][..],
            Some("hit points"),
        ),
        (
            "actor.saves.fortitude",
            "fortitude",
            &["npc", "hazard"][..],
            None,
        ),
        ("actor.saves.reflex", "reflex", &["npc", "hazard"][..], None),
        ("actor.saves.will", "will", &["npc", "hazard"][..], None),
        ("actor.perception", "perception", &["npc"][..], None),
        (
            "actor.speed.land",
            "land_speed",
            &["npc", "character"][..],
            Some("feet"),
        ),
        ("hazard.hardness", "hardness", &["hazard"][..], None),
    ] {
        fields.push(field(
            path,
            Number,
            families,
            family("actor_projection", column),
            Vec::new(),
            unit,
        ));
    }
    fields.push(field(
        "hazard.complexity",
        Boolean,
        &["hazard"],
        family("actor_projection", "complexity"),
        Vec::new(),
        None,
    ));
    fields.push(field(
        "actor.languages",
        Set,
        &["npc", "character"],
        set(
            family("actor_projection", "languages"),
            "actor_languages",
            "record_id",
            None,
        ),
        choices::<Language>()?,
        None,
    ));
    for (path, col, table, owner, families, kind) in [
        (
            "actor.speeds",
            "speeds",
            "actor_speeds",
            "record_id",
            &["npc", "character"][..],
            None,
        ),
        (
            "actor.senses",
            "senses",
            "actor_senses",
            "record_id",
            &["npc"][..],
            None,
        ),
        (
            "actor.immunities",
            "immunities",
            "actor_iwr_entries",
            "record_id",
            IWR,
            Some("immunity"),
        ),
        (
            "actor.weaknesses",
            "weaknesses",
            "actor_iwr_entries",
            "record_id",
            IWR,
            Some("weakness"),
        ),
        (
            "actor.resistances",
            "resistances",
            "actor_iwr_entries",
            "record_id",
            IWR,
            Some("resistance"),
        ),
        (
            "actor.items",
            "items",
            "actor_items",
            "record_id",
            ACTORS,
            None,
        ),
    ] {
        fields.push(field(
            path,
            Collection,
            families,
            collection(family("actor_projection", col), table, owner, kind),
            Vec::new(),
            None,
        ));
    }
    let mut item_fields = Vec::new();
    for (path, col, ty, values, unit) in [
        ("spell.rank", "rank", Number, Vec::new(), Some("rank")),
        ("spell.focus", "focus", Boolean, Vec::new(), None),
        ("spell.ritual", "ritual", Boolean, Vec::new(), None),
        (
            "spell.casting_time",
            "casting_time",
            String,
            Vec::new(),
            None,
        ),
        (
            "spell.casting_form",
            "casting_form",
            String,
            [
                "free",
                "reaction",
                "one_action",
                "two_actions",
                "three_actions",
                "one_or_two",
                "one_to_three",
                "two_or_three",
                "two_rounds",
                "other",
            ]
            .into_iter()
            .map(str::to_owned)
            .collect(),
            None,
        ),
        (
            "spell.defense.save",
            "save",
            String,
            choices::<SpellDefenseSourceSaveStatistic>()?,
            None,
        ),
        (
            "spell.defense.basic",
            "basic_save",
            Boolean,
            Vec::new(),
            None,
        ),
        (
            "spell.defense.passive",
            "passive_defense",
            String,
            choices::<SpellDefenseSourcePassiveStatistic>()?,
            None,
        ),
        (
            "spell.area.type",
            "area_type",
            String,
            choices::<SpellAreaType>()?,
            None,
        ),
        (
            "spell.area.size",
            "area_size",
            Number,
            Vec::new(),
            Some("feet"),
        ),
        ("spell.sustained", "sustained", Boolean, Vec::new(), None),
        (
            "spell.duration_text",
            "duration_text",
            String,
            Vec::new(),
            None,
        ),
    ] {
        item_fields.push(field(
            path,
            ty,
            &["spell"],
            family("spell_projection", col),
            values,
            unit,
        ));
    }
    item_fields.push(field(
        "spell.cantrip",
        Boolean,
        &["spell"],
        Binding::Cantrip,
        Vec::new(),
        None,
    ));
    item_fields.push(field(
        "spell.traditions",
        Set,
        &["spell"],
        set(
            family("spell_projection", "traditions"),
            "spell_traditions",
            "spell_id",
            None,
        ),
        choices::<PatchSpellOverlayOverrideSystemTraitsTraditionsEntry>()?,
        None,
    ));
    for (path, col, ty, families, values, unit) in [
        (
            "physical.price.per_item_cp",
            "price_per_item_cp",
            Number,
            PHYSICAL,
            Vec::new(),
            Some("copper per authored item"),
        ),
        (
            "physical.bulk",
            "bulk",
            Number,
            PHYSICAL,
            Vec::new(),
            Some("authored bulk"),
        ),
        (
            "physical.usage",
            "usage",
            String,
            &["backpack", "book", "consumable", "equipment", "weapon"][..],
            Vec::new(),
            None,
        ),
        (
            "consumable.category",
            "consumable_category",
            String,
            &["consumable"][..],
            choices::<ConsumableSystemSourceCategory>()?,
            None,
        ),
    ] {
        item_fields.push(field(
            path,
            ty,
            families,
            family("physical_projection", col),
            values,
            unit,
        ));
    }
    for (path, table, col, families, ty, values, unit) in [
        (
            "weapon.category",
            "weapon_projection",
            "category",
            &["weapon"][..],
            String,
            choices::<WeaponSystemSourceCategory>()?,
            None,
        ),
        (
            "weapon.group",
            "weapon_projection",
            "weapon_group",
            &["weapon"][..],
            String,
            choices::<WeaponSystemSourceGroup>()?,
            None,
        ),
        (
            "weapon.damage.type",
            "weapon_projection",
            "damage_type",
            &["weapon"][..],
            String,
            choices::<ConsumableDamageHealingType>()?,
            None,
        ),
        (
            "weapon.range.increment",
            "weapon_projection",
            "range",
            &["weapon"][..],
            Number,
            Vec::new(),
            Some("feet"),
        ),
        (
            "weapon.reload",
            "weapon_projection",
            "reload",
            &["weapon"][..],
            String,
            choices::<WeaponSystemSourceReloadValue>()?,
            None,
        ),
        (
            "armor.category",
            "armor_projection",
            "category",
            &["armor"][..],
            String,
            choices::<ArmorSystemSourceCategory>()?,
            None,
        ),
        (
            "armor.ac_bonus",
            "armor_projection",
            "ac_bonus",
            &["armor"][..],
            Number,
            Vec::new(),
            None,
        ),
        (
            "armor.dex_cap",
            "armor_projection",
            "dex_cap",
            &["armor"][..],
            Number,
            Vec::new(),
            None,
        ),
        (
            "shield.hardness",
            "shield_projection",
            "hardness",
            &["shield"][..],
            Number,
            Vec::new(),
            None,
        ),
        (
            "shield.hp.maximum",
            "shield_projection",
            "hp_maximum",
            &["shield"][..],
            Number,
            Vec::new(),
            Some("hit points"),
        ),
    ] {
        item_fields.push(field(
            path,
            ty,
            families,
            Binding::Family {
                table,
                column: col,
                physical_child: true,
            },
            values,
            unit,
        ));
    }
    for (path, table, col, families, ty, values) in [
        (
            "action.type",
            "ability_projection",
            "action_type",
            &["action", "feat", "campaignFeature"][..],
            String,
            choices::<AbilitySystemSourceActionTypeSourceFromSchemaValue>()?,
        ),
        (
            "action.count",
            "ability_projection",
            "action_count",
            &["action", "feat", "campaignFeature"][..],
            Number,
            Vec::new(),
        ),
        (
            "action.category",
            "ability_projection",
            "category",
            &["action"][..],
            String,
            choices::<AbilitySystemSourceCategory>()?,
        ),
        (
            "feat.category",
            "ability_projection",
            "category",
            &["feat"][..],
            String,
            choices::<FeatSystemSourceCategory>()?,
        ),
        (
            "campaign_feature.category",
            "ability_projection",
            "category",
            &["campaignFeature"][..],
            String,
            choices::<CampaignFeatureSystemSourceCategory>()?,
        ),
        (
            "heritage.ancestry.slug",
            "heritage_projection",
            "ancestry_slug",
            &["heritage"][..],
            String,
            Vec::new(),
        ),
        (
            "heritage.ancestry.uuid",
            "heritage_projection",
            "ancestry_uuid",
            &["heritage"][..],
            String,
            Vec::new(),
        ),
        (
            "heritage.versatile",
            "heritage_projection",
            "versatile",
            &["heritage"][..],
            Boolean,
            Vec::new(),
        ),
        (
            "effect.duration.unit",
            "effect_projection",
            "duration_unit",
            &["effect"][..],
            String,
            choices::<DurationDataUnit>()?,
        ),
        (
            "condition.is_valued",
            "condition_projection",
            "is_valued",
            &["condition"][..],
            Boolean,
            Vec::new(),
        ),
    ] {
        item_fields.push(field(path, ty, families, family(table, col), values, None));
    }
    for unit in ["rounds", "minutes", "hours", "days"] {
        item_fields.push(field(
            &format!("effect.duration.{unit}"),
            Number,
            &["effect"],
            Binding::Duration { unit },
            Vec::new(),
            Some(unit),
        ));
    }
    for (path, col, table, kind, values) in [
        (
            "deity.domains.primary",
            "primary_domains",
            "deity_domains",
            Some("primary"),
            choices::<DeitySystemSourceDomainsAlternateEntry>()?,
        ),
        (
            "deity.domains.alternate",
            "alternate_domains",
            "deity_domains",
            Some("alternate"),
            choices::<DeitySystemSourceDomainsAlternateEntry>()?,
        ),
        (
            "deity.fonts",
            "fonts",
            "deity_fonts",
            None,
            choices::<DivineFontsAlternative2Entry1>()?
                .into_iter()
                .chain(choices::<DivineFontsAlternative2Entry2>()?)
                .collect(),
        ),
    ] {
        item_fields.push(field(
            path,
            Set,
            &["deity"],
            set(family("deity_projection", col), table, "deity_id", kind),
            values,
            None,
        ));
    }
    // The same maintained descriptors instantiate root and immediate Item scope.
    let mut child_items: Vec<_> = fields
        .iter()
        .filter(|d| {
            matches!(
                d.definition.path.as_str(),
                "source.type"
                    | "traits"
                    | "rarity"
                    | "publication.title"
                    | "publication.remaster"
                    | "item.level"
                    | "physical.size"
                    | "ancestry.size"
            )
        })
        .cloned()
        .collect();
    child_items.extend(item_fields.clone());
    fields.extend(item_fields);
    fields.push(field(
        "spell.damage",
        Collection,
        &["spell"],
        collection(
            family("spell_projection", "damage"),
            "spell_damage_entries",
            "spell_id",
            None,
        ),
        Vec::new(),
        None,
    ));
    for mut descriptor in child_items {
        if descriptor.definition.path == "source.type" {
            descriptor.definition.choices = ITEMS.iter().map(|s| (*s).to_owned()).collect();
        }
        descriptor.definition.scope = Some("actor.items".to_owned());
        descriptor.definition.id = format!("actor.items[].{}", descriptor.definition.path);
        descriptor
            .definition
            .family_types
            .retain(|f| ITEMS.contains(&f.as_str()));
        if let Binding::Set { table, owner, .. } = &mut descriptor.binding
            && *table == "record_traits"
        {
            *table = "actor_item_traits";
            *owner = "item_id";
        }
        fields.push(descriptor);
    }
    for (scope, path, ty, values, unit) in [
        (
            "actor.speeds",
            "type",
            String,
            choices::<CharacterAttributesSourceSpeedOtherSpeedsEntryType>()?,
            None,
        ),
        ("actor.speeds", "value", Number, Vec::new(), Some("feet")),
        (
            "actor.senses",
            "type",
            String,
            choices::<SenseConstructorParamsType>()?,
            None,
        ),
        (
            "actor.immunities",
            "type",
            String,
            choices::<ActorAttributesSourceImmunitiesEntryImmunitySourceType>()?,
            None,
        ),
        (
            "actor.weaknesses",
            "type",
            String,
            choices::<WeaknessSourceType>()?,
            None,
        ),
        ("actor.weaknesses", "value", Number, Vec::new(), None),
        (
            "actor.resistances",
            "type",
            String,
            choices::<ResistanceSourceType>()?,
            None,
        ),
        ("actor.resistances", "value", Number, Vec::new(), None),
        (
            "spell.damage",
            "type",
            String,
            choices::<ConsumableDamageHealingType>()?,
            None,
        ),
    ] {
        let mut d = field(
            path,
            ty,
            &[],
            Binding::Member {
                column: if path == "type" { "type" } else { "value" },
            },
            values,
            unit,
        );
        d.definition.scope = Some(scope.to_owned());
        d.definition.id = format!("{scope}[].{path}");
        fields.push(d);
    }
    let mut kinds = field(
        "kinds",
        Set,
        &[],
        set(
            Binding::Member { column: "kinds" },
            "spell_damage_kinds",
            "entry_id",
            None,
        ),
        choices::<DamageKind>()?,
        None,
    );
    kinds.definition.scope = Some("spell.damage".to_owned());
    kinds.definition.id = "spell.damage[].kinds".to_owned();
    fields.push(kinds);
    for descriptor in &mut fields {
        let d = &mut descriptor.definition;
        let prefix = if d.scope.is_some() { "entry." } else { "" };
        let field = format!("{prefix}{}", d.path);
        let quoted = serde_json::Value::String(
            d.choices
                .first()
                .cloned()
                .unwrap_or_else(|| "example".into()),
        )
        .to_string();
        let example = match d.field_type {
            Number => format!("{field} >= 0"),
            Boolean => format!("{field} == true"),
            String => format!("{field} == {quoted}"),
            Set => format!("{quoted} in {field}"),
            Collection => format!("{field}.exists(entry, true)"),
        };
        let available = format!("{prefix}availability[\"{}\"] == \"value\"", d.path);
        d.examples = vec![example, available]
            .into_iter()
            .map(|e| match &d.scope {
                Some(scope) => format!("{scope}.exists(entry, {e})"),
                None => e,
            })
            .collect();
    }
    Ok(fields)
}
pub(crate) fn descriptors() -> Result<&'static [Descriptor], atlas_domain::QueryError> {
    static FIELDS: OnceLock<Result<Vec<Descriptor>, String>> = OnceLock::new();
    FIELDS
        .get_or_init(build)
        .as_ref()
        .map(|v| v.as_slice())
        .map_err(|e| super::validation::error("catalog_initialization", e, None))
}
pub(crate) fn descriptor(
    scope: Option<&str>,
    field: &str,
) -> Result<Option<&'static Descriptor>, atlas_domain::QueryError> {
    Ok(descriptors()?
        .iter()
        .find(|d| d.definition.scope.as_deref() == scope && d.definition.path == field))
}
pub fn query_capabilities() -> Result<QueryCapability, atlas_domain::QueryError> {
    Ok(QueryCapability {
        version: QUERY_CAPABILITY_VERSION,
        fields: descriptors()?
            .iter()
            .map(|d| d.definition.clone())
            .collect(),
        limits: QueryLimits::default(),
    })
}
#[cfg(test)]
mod metadata_tests {
    #[test]
    fn a_non_enum_metadata_probe_fails_explicitly() {
        assert!(
            super::choices::<String>()
                .unwrap_err()
                .contains("did not provide variants")
        );
        assert!(
            !super::choices::<atlas_foundry_model::generated::WeaponSystemSourceCategory>()
                .unwrap()
                .is_empty()
        );
    }
}
