//! On-demand reads of facts supplied by a saved Pathbuilder export.
use crate::character_context::facts::{FactSource, ToFact, object, present};
use crate::{AppServiceError, AppServiceResult, AtlasAppService};
use atlas_app_model::ImportedFactValue;
use serde::{Deserialize, Deserializer, de::DeserializeOwned};
use serde_json::value::RawValue;
use std::collections::BTreeMap;

/// A source fact and its availability, without speculative defaults.
#[derive(Debug, Default, PartialEq, Eq)]
pub enum ImportedField<T> {
    #[default]
    Missing,
    Null,
    Unrecognized,
    Value(T),
}

impl<'de, T: DeserializeOwned + ExportShape> Deserialize<'de> for ImportedField<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = Box::<RawValue>::deserialize(deserializer)?;
        Ok(read_field(&raw))
    }
}

/// Supported source fields, shared by the typed decoder and import details.
#[derive(Clone, Copy)]
pub(super) enum FieldShape {
    Hidden,
    Leaf(fn(&str) -> bool),
    Object(&'static [(&'static str, Self)]),
    Map(&'static Self),
    List(&'static Self),
    Feat,
}

trait ExportShape {
    const SHAPE: FieldShape;
}

fn accepts<T: DeserializeOwned>(raw: &str) -> bool {
    serde_json::from_str::<T>(raw).is_ok()
}

macro_rules! export_leaf {
    ($($ty:ty),+ $(,)?) => {
        $(impl ExportShape for $ty {
            const SHAPE: FieldShape = FieldShape::Leaf(accepts::<Self>);
        })+
    };
}

export_leaf!(String, i64, bool, (String, i64), ImportedEquipment);

impl ExportShape for ImportedFeat {
    const SHAPE: FieldShape = FieldShape::Feat;
}

impl<T: ExportShape> ExportShape for ImportedField<T> {
    const SHAPE: FieldShape = T::SHAPE;
}

impl<T: ExportShape> ExportShape for Vec<T> {
    const SHAPE: FieldShape = FieldShape::List(&T::SHAPE);
}

impl<T: ExportShape> ExportShape for BTreeMap<String, T> {
    const SHAPE: FieldShape = FieldShape::Map(&T::SHAPE);
}

macro_rules! export_object {
    ($name:ident { $($field:ident: $ty:ty => $key:literal),+ $(,)? }) => {
        #[derive(Default, Deserialize)]
        #[serde(default)]
        pub struct $name {
            $(#[serde(rename = $key)] pub $field: $ty),+
        }

        impl ExportShape for $name {
            const SHAPE: FieldShape = FieldShape::Object(&[
                $(($key, <$ty as ExportShape>::SHAPE)),+
            ]);
        }

        impl FactSource for $name {
            fn fact_value(&self) -> ImportedFactValue {
                object([$( (stringify!($field), self.$field.to_fact()) ),+])
            }
        }
    };
}

fn read_field<T: DeserializeOwned + ExportShape>(raw: &RawValue) -> ImportedField<T> {
    if raw.get() == "null" {
        ImportedField::Null
    } else if matches!(T::SHAPE, FieldShape::Object(_) | FieldShape::Map(_))
        && !raw.get().starts_with('{')
    {
        ImportedField::Unrecognized
    } else {
        serde_json::from_str(raw.get()).map_or(ImportedField::Unrecognized, ImportedField::Value)
    }
}

type SourceList<T> = ImportedField<Vec<ImportedField<T>>>;
type SourceMap<T> = ImportedField<BTreeMap<String, ImportedField<T>>>;

#[derive(Deserialize)]
struct Export<'a> {
    #[serde(borrow)]
    build: BTreeMap<String, &'a RawValue>,
}

/// Verified sheet fields, organized by the export's structure. All original
/// fields remain available to the saved-state owner through the source export.
macro_rules! export_build {
    ($($field:ident: $ty:ty => $key:literal),+ $(,)?) => {
        pub struct PathbuilderSheet {
            pub name: String,
            pub level: i64,
            pub xp: ImportedField<i64>,
            pub alignment: ImportedField<String>,
            $(pub $field: $ty),+
        }

        pub(super) const BUILD_FIELDS: &[(&str, FieldShape)] = &[
            ("name", String::SHAPE),
            ("level", i64::SHAPE),
            // Known export fields omitted from the character sheet.
            ("xp", FieldShape::Hidden),
            ("alignment", FieldShape::Hidden),
            $(($key, <$ty as ExportShape>::SHAPE)),+
        ];

        impl PathbuilderSheet {
            fn from_build(build: &BTreeMap<String, &RawValue>) -> Option<Self> {
                let (ImportedField::Value(name), ImportedField::Value(level)) =
                    (read_build_field(build, "name"), read_build_field(build, "level"))
                else {
                    return None;
                };
                Some(Self {
                    name,
                    level,
                    xp: read_build_field(build, "xp"),
                    alignment: read_build_field(build, "alignment"),
                    $($field: read_build_field(build, $key)),+
                })
            }
        }

        impl FactSource for PathbuilderSheet {
            fn fact_value(&self) -> ImportedFactValue {
                object([
                    ("name", present(self.name.fact_value())),
                    ("level", present(self.level.fact_value())),
                    ("xp", self.xp.to_fact()),
                    ("alignment", self.alignment.to_fact()),
                    $( (stringify!($field), self.$field.to_fact()) ),+
                ])
            }
        }
    };
}

export_build! {
    class: ImportedField<String> => "class",
    ancestry: ImportedField<String> => "ancestry",
    heritage: ImportedField<String> => "heritage",
    background: ImportedField<String> => "background",
    gender: ImportedField<String> => "gender",
    age: ImportedField<String> => "age",
    deity: ImportedField<String> => "deity",
    size: ImportedField<i64> => "size",
    size_name: ImportedField<String> => "sizeName",
    key_ability: ImportedField<String> => "keyability",
    languages: SourceList<String> => "languages",
    resistances: SourceList<String> => "resistances",
    abilities: ImportedField<ImportedAttributes> => "abilities",
    attributes: ImportedField<ImportedHpAndSpeed> => "attributes",
    proficiencies: SourceMap<i64> => "proficiencies",
    specific_proficiencies: SourceMap<Vec<ImportedField<String>>> => "specificProficiencies",
    lores: SourceList<(String, i64)> => "lores",
    mods: SourceMap<BTreeMap<String, ImportedField<i64>>> => "mods",
    ac_total: ImportedField<ImportedArmorClass> => "acTotal",
    feats: SourceList<ImportedFeat> => "feats",
    specials: SourceList<String> => "specials",
    spell_casters: SourceList<ImportedCaster> => "spellCasters",
    focus_points: ImportedField<i64> => "focusPoints",
    focus: SourceMap<BTreeMap<String, ImportedField<ImportedFocus>>> => "focus",
    equipment: SourceList<ImportedEquipment> => "equipment",
    equipment_containers: SourceMap<ImportedContainer> => "equipmentContainers",
    weapons: SourceList<ImportedWeapon> => "weapons",
    armor: SourceList<ImportedArmor> => "armor",
    money: ImportedField<ImportedMoney> => "money",
    formula: SourceList<ImportedFormulas> => "formula",
    pets: SourceList<ImportedCompanion> => "pets",
}

export_object!(ImportedAttributes {
    str: ImportedField<i64> => "str",
    dex: ImportedField<i64> => "dex",
    con: ImportedField<i64> => "con",
    int: ImportedField<i64> => "int",
    wis: ImportedField<i64> => "wis",
    cha: ImportedField<i64> => "cha",
    breakdown: ImportedField<ImportedBoosts> => "breakdown",
});

export_object!(ImportedBoosts {
    ancestry_free: SourceList<String> => "ancestryFree",
    ancestry_boosts: SourceList<String> => "ancestryBoosts",
    ancestry_flaws: SourceList<String> => "ancestryFlaws",
    background_boosts: SourceList<String> => "backgroundBoosts",
    class_boosts: SourceList<String> => "classBoosts",
    map_levelled_boosts: SourceMap<Vec<ImportedField<String>>> => "mapLevelledBoosts",
});

export_object!(ImportedHpAndSpeed {
    ancestry_hp: ImportedField<i64> => "ancestryhp",
    class_hp: ImportedField<i64> => "classhp",
    bonus_hp: ImportedField<i64> => "bonushp",
    bonus_hp_per_level: ImportedField<i64> => "bonushpPerLevel",
    speed: ImportedField<i64> => "speed",
    speed_bonus: ImportedField<i64> => "speedBonus",
});

export_object!(ImportedArmorClass {
    ac_total: ImportedField<i64> => "acTotal",
    ac_prof_bonus: ImportedField<i64> => "acProfBonus",
    ac_ability_bonus: ImportedField<i64> => "acAbilityBonus",
    ac_item_bonus: ImportedField<i64> => "acItemBonus",
    shield_bonus: ImportedField<i64> => "shieldBonus",
});

pub struct ImportedFeat {
    pub name: String,
    pub choice: Option<String>,
    pub category: String,
    pub level: i64,
    pub slot: ImportedField<String>,
    pub selection_kind: ImportedField<String>,
    pub selection_detail: ImportedField<String>,
}

impl ImportedFeat {
    pub(super) const REQUIRED_FIELDS: usize = 4;
    pub(super) const OPTIONAL_FIELDS: usize = 3;

    pub(super) fn accepts(raw: &str) -> bool {
        accepts::<Self>(raw)
    }
}

impl<'de> Deserialize<'de> for ImportedFeat {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::de::Error as _;
        let row = Vec::<Box<RawValue>>::deserialize(deserializer)?;
        let [name, choice, category, level, ..] = row.as_slice() else {
            return Err(D::Error::custom("incomplete Pathbuilder feat"));
        };
        let [slot, selection_kind, selection_detail] =
            std::array::from_fn::<_, { Self::OPTIONAL_FIELDS }, _>(|index| {
                row.get(Self::REQUIRED_FIELDS + index)
                    .map_or(ImportedField::Missing, |raw| read_field(raw))
            });
        Ok(Self {
            name: serde_json::from_str(name.get()).map_err(D::Error::custom)?,
            choice: serde_json::from_str(choice.get()).map_err(D::Error::custom)?,
            category: serde_json::from_str(category.get()).map_err(D::Error::custom)?,
            level: serde_json::from_str(level.get()).map_err(D::Error::custom)?,
            slot,
            selection_kind,
            selection_detail,
        })
    }
}

export_object!(ImportedCaster {
    name: ImportedField<String> => "name",
    magic_tradition: ImportedField<String> => "magicTradition",
    spellcasting_type: ImportedField<String> => "spellcastingType",
    ability: ImportedField<String> => "ability",
    proficiency: ImportedField<i64> => "proficiency",
    focus_points: ImportedField<i64> => "focusPoints",
    innate: ImportedField<bool> => "innate",
    per_day: SourceList<i64> => "perDay",
    spells: SourceList<ImportedSpellRank> => "spells",
    prepared: SourceList<ImportedSpellRank> => "prepared",
});

export_object!(ImportedSpellRank {
    spell_level: ImportedField<i64> => "spellLevel",
    list: SourceList<String> => "list",
});

export_object!(ImportedFocus {
    ability_bonus: ImportedField<i64> => "abilityBonus",
    proficiency: ImportedField<i64> => "proficiency",
    item_bonus: ImportedField<i64> => "itemBonus",
    focus_cantrips: SourceList<String> => "focusCantrips",
    focus_spells: SourceList<String> => "focusSpells",
});

#[derive(Deserialize)]
#[serde(untagged)]
pub enum ImportedEquipment {
    InContainer(String, i64, String, String),
    Uncontained(String, i64, String),
}

export_object!(ImportedContainer {
    container_name: ImportedField<String> => "containerName",
    bag_of_holding: ImportedField<bool> => "bagOfHolding",
    backpack: ImportedField<bool> => "backpack",
    augmentations: ImportedField<bool> => "augmentations",
});

export_object!(ImportedWeapon {
    name: ImportedField<String> => "name",
    display: ImportedField<String> => "display",
    quantity: ImportedField<i64> => "qty",
    proficiency: ImportedField<String> => "prof",
    attack: ImportedField<i64> => "attack",
    die: ImportedField<String> => "die",
    damage_type: ImportedField<String> => "damageType",
    damage_bonus: ImportedField<i64> => "damageBonus",
    potency: ImportedField<i64> => "pot",
    striking: ImportedField<String> => "str",
    material: ImportedField<String> => "mat",
    grade: ImportedField<String> => "grade",
    runes: SourceList<String> => "runes",
    increased_dice: ImportedField<bool> => "increasedDice",
    is_inventor: ImportedField<bool> => "isInventor",
});

export_object!(ImportedArmor {
    name: ImportedField<String> => "name",
    display: ImportedField<String> => "display",
    quantity: ImportedField<i64> => "qty",
    proficiency: ImportedField<String> => "prof",
    potency: ImportedField<i64> => "pot",
    resilient: ImportedField<String> => "res",
    material: ImportedField<String> => "mat",
    grade: ImportedField<String> => "grade",
    runes: SourceList<String> => "runes",
    worn: ImportedField<bool> => "worn",
});

export_object!(ImportedMoney {
    pp: ImportedField<i64> => "pp",
    gp: ImportedField<i64> => "gp",
    sp: ImportedField<i64> => "sp",
    cp: ImportedField<i64> => "cp",
});

export_object!(ImportedFormulas {
    kind: ImportedField<String> => "type",
    known: SourceList<String> => "known",
});

export_object!(ImportedCompanion {
    kind: ImportedField<String> => "type",
    name: ImportedField<String> => "name",
    animal: ImportedField<String> => "animal",
    mature: ImportedField<bool> => "mature",
    incredible: ImportedField<bool> => "incredible",
    incredible_type: ImportedField<String> => "incredibleType",
    specializations: SourceList<String> => "specializations",
    armor: ImportedField<String> => "armor",
});

impl AtlasAppService {
    /// Read the owned character's latest saved Pathbuilder sheet, if available.
    pub fn character_pathbuilder_sheet(
        &self,
        character_key: &str,
    ) -> AppServiceResult<Option<PathbuilderSheet>> {
        self.with_state(|session| {
            session
                .local_state()?
                .characters()
                .pathbuilder_source(character_key)?
                .map(|source| read_sheet(source.json()))
                .transpose()
        })
    }
}

pub(crate) fn read_sheet(source: &str) -> AppServiceResult<PathbuilderSheet> {
    let invalid =
        || AppServiceError::invalid_request("Could not read the saved Pathbuilder export.");
    let Export { build } = serde_json::from_str::<Export<'_>>(source).map_err(|_| invalid())?;
    PathbuilderSheet::from_build(&build).ok_or_else(invalid)
}

fn read_build_field<T: DeserializeOwned + ExportShape>(
    build: &BTreeMap<String, &RawValue>,
    key: &str,
) -> ImportedField<T> {
    build
        .get(key)
        .map_or(ImportedField::Missing, |raw| read_field(raw))
}
