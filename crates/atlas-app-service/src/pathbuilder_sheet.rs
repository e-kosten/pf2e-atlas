//! On-demand reads of facts supplied by a saved Pathbuilder export.
use crate::{AppServiceError, AppServiceResult, AtlasAppService};
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

// Serde also accepts positional arrays for named structs. Export fields with
// named properties are only interpreted when the source actually supplies names.
trait ExportShape {
    const OBJECT: bool = false;
}
impl ExportShape for String {}
impl ExportShape for i64 {}
impl ExportShape for bool {}
impl ExportShape for (String, i64) {}
impl ExportShape for ImportedEquipment {}
impl ExportShape for ImportedFeat {}
impl<T> ExportShape for Vec<T> {}
impl<T> ExportShape for BTreeMap<String, T> {
    const OBJECT: bool = true;
}

macro_rules! object_fields {
    ($($ty:ty),+ $(,)?) => {
        $(impl ExportShape for $ty {
            const OBJECT: bool = true;
        })+
    };
}
object_fields!(
    ImportedAttributes,
    ImportedBoosts,
    ImportedHpAndSpeed,
    ImportedArmorClass,
    ImportedCaster,
    ImportedSpellRank,
    ImportedFocus,
    ImportedContainer,
    ImportedWeapon,
    ImportedArmor,
    ImportedMoney,
    ImportedFormulas,
    ImportedCompanion,
);

fn read_field<T: DeserializeOwned + ExportShape>(raw: &RawValue) -> ImportedField<T> {
    if raw.get() == "null" {
        ImportedField::Null
    } else if T::OBJECT && !raw.get().starts_with('{') {
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
pub struct PathbuilderSheet {
    pub name: String,
    pub level: i64,
    pub class: ImportedField<String>,
    pub xp: ImportedField<i64>,
    pub ancestry: ImportedField<String>,
    pub heritage: ImportedField<String>,
    pub background: ImportedField<String>,
    pub alignment: ImportedField<String>,
    pub gender: ImportedField<String>,
    pub age: ImportedField<String>,
    pub deity: ImportedField<String>,
    pub size: ImportedField<i64>,
    pub size_name: ImportedField<String>,
    pub key_ability: ImportedField<String>,
    pub languages: SourceList<String>,
    pub resistances: SourceList<String>,
    pub abilities: ImportedField<ImportedAttributes>,
    pub attributes: ImportedField<ImportedHpAndSpeed>,
    pub proficiencies: SourceMap<i64>,
    pub specific_proficiencies: SourceMap<Vec<ImportedField<String>>>,
    pub lores: SourceList<(String, i64)>,
    pub mods: SourceMap<BTreeMap<String, ImportedField<i64>>>,
    pub ac_total: ImportedField<ImportedArmorClass>,
    pub feats: SourceList<ImportedFeat>,
    pub specials: SourceList<String>,
    pub spell_casters: SourceList<ImportedCaster>,
    pub focus_points: ImportedField<i64>,
    pub focus: SourceMap<BTreeMap<String, ImportedField<ImportedFocus>>>,
    pub equipment: SourceList<ImportedEquipment>,
    pub equipment_containers: SourceMap<ImportedContainer>,
    pub weapons: SourceList<ImportedWeapon>,
    pub armor: SourceList<ImportedArmor>,
    pub money: ImportedField<ImportedMoney>,
    pub formula: SourceList<ImportedFormulas>,
    pub pets: SourceList<ImportedCompanion>,
}

#[derive(Default, Deserialize)]
#[serde(default)]
pub struct ImportedAttributes {
    pub str: ImportedField<i64>,
    pub dex: ImportedField<i64>,
    pub con: ImportedField<i64>,
    pub int: ImportedField<i64>,
    pub wis: ImportedField<i64>,
    pub cha: ImportedField<i64>,
    pub breakdown: ImportedField<ImportedBoosts>,
}

#[derive(Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ImportedBoosts {
    pub ancestry_free: SourceList<String>,
    pub ancestry_boosts: SourceList<String>,
    pub ancestry_flaws: SourceList<String>,
    pub background_boosts: SourceList<String>,
    pub class_boosts: SourceList<String>,
    pub map_levelled_boosts: SourceMap<Vec<ImportedField<String>>>,
}

#[derive(Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ImportedHpAndSpeed {
    #[serde(rename = "ancestryhp")]
    pub ancestry_hp: ImportedField<i64>,
    #[serde(rename = "classhp")]
    pub class_hp: ImportedField<i64>,
    #[serde(rename = "bonushp")]
    pub bonus_hp: ImportedField<i64>,
    #[serde(rename = "bonushpPerLevel")]
    pub bonus_hp_per_level: ImportedField<i64>,
    pub speed: ImportedField<i64>,
    pub speed_bonus: ImportedField<i64>,
}

#[derive(Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ImportedArmorClass {
    pub ac_total: ImportedField<i64>,
    pub ac_prof_bonus: ImportedField<i64>,
    pub ac_ability_bonus: ImportedField<i64>,
    pub ac_item_bonus: ImportedField<i64>,
    pub shield_bonus: ImportedField<i64>,
}

pub struct ImportedFeat {
    pub name: String,
    pub choice: Option<String>,
    pub category: String,
    pub level: i64,
    pub slot: ImportedField<String>,
    pub selection_kind: ImportedField<String>,
    pub selection_detail: ImportedField<String>,
}

impl<'de> Deserialize<'de> for ImportedFeat {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::de::Error as _;
        let row = Vec::<Box<RawValue>>::deserialize(deserializer)?;
        let [name, choice, category, level, ..] = row.as_slice() else {
            return Err(D::Error::custom("incomplete Pathbuilder feat"));
        };
        Ok(Self {
            name: serde_json::from_str(name.get()).map_err(D::Error::custom)?,
            choice: serde_json::from_str(choice.get()).map_err(D::Error::custom)?,
            category: serde_json::from_str(category.get()).map_err(D::Error::custom)?,
            level: serde_json::from_str(level.get()).map_err(D::Error::custom)?,
            slot: row
                .get(4)
                .map_or(ImportedField::Missing, |raw| read_field(raw)),
            selection_kind: row
                .get(5)
                .map_or(ImportedField::Missing, |raw| read_field(raw)),
            selection_detail: row
                .get(6)
                .map_or(ImportedField::Missing, |raw| read_field(raw)),
        })
    }
}

#[derive(Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ImportedCaster {
    pub name: ImportedField<String>,
    pub magic_tradition: ImportedField<String>,
    pub spellcasting_type: ImportedField<String>,
    pub ability: ImportedField<String>,
    pub proficiency: ImportedField<i64>,
    pub focus_points: ImportedField<i64>,
    pub innate: ImportedField<bool>,
    pub per_day: SourceList<i64>,
    pub spells: SourceList<ImportedSpellRank>,
    pub prepared: SourceList<ImportedSpellRank>,
}

#[derive(Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ImportedSpellRank {
    pub spell_level: ImportedField<i64>,
    pub list: SourceList<String>,
}

#[derive(Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ImportedFocus {
    pub ability_bonus: ImportedField<i64>,
    pub proficiency: ImportedField<i64>,
    pub item_bonus: ImportedField<i64>,
    pub focus_cantrips: SourceList<String>,
    pub focus_spells: SourceList<String>,
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum ImportedEquipment {
    InContainer(String, i64, String, String),
    Uncontained(String, i64, String),
}

#[derive(Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ImportedContainer {
    pub container_name: ImportedField<String>,
    pub bag_of_holding: ImportedField<bool>,
    pub backpack: ImportedField<bool>,
    pub augmentations: ImportedField<bool>,
}

#[derive(Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ImportedWeapon {
    pub name: ImportedField<String>,
    pub display: ImportedField<String>,
    #[serde(rename = "qty")]
    pub quantity: ImportedField<i64>,
    #[serde(rename = "prof")]
    pub proficiency: ImportedField<String>,
    pub attack: ImportedField<i64>,
    pub die: ImportedField<String>,
    pub damage_type: ImportedField<String>,
    pub damage_bonus: ImportedField<i64>,
    #[serde(rename = "pot")]
    pub potency: ImportedField<i64>,
    #[serde(rename = "str")]
    pub striking: ImportedField<String>,
    #[serde(rename = "mat")]
    pub material: ImportedField<String>,
    pub grade: ImportedField<String>,
    pub runes: SourceList<String>,
    pub increased_dice: ImportedField<bool>,
    pub is_inventor: ImportedField<bool>,
}

#[derive(Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ImportedArmor {
    pub name: ImportedField<String>,
    pub display: ImportedField<String>,
    #[serde(rename = "qty")]
    pub quantity: ImportedField<i64>,
    #[serde(rename = "prof")]
    pub proficiency: ImportedField<String>,
    #[serde(rename = "pot")]
    pub potency: ImportedField<i64>,
    #[serde(rename = "res")]
    pub resilient: ImportedField<String>,
    #[serde(rename = "mat")]
    pub material: ImportedField<String>,
    pub grade: ImportedField<String>,
    pub runes: SourceList<String>,
    pub worn: ImportedField<bool>,
}

#[derive(Default, Deserialize)]
#[serde(default)]
pub struct ImportedMoney {
    pub pp: ImportedField<i64>,
    pub gp: ImportedField<i64>,
    pub sp: ImportedField<i64>,
    pub cp: ImportedField<i64>,
}

#[derive(Default, Deserialize)]
#[serde(default)]
pub struct ImportedFormulas {
    #[serde(rename = "type")]
    pub kind: ImportedField<String>,
    pub known: SourceList<String>,
}

#[derive(Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ImportedCompanion {
    #[serde(rename = "type")]
    pub kind: ImportedField<String>,
    pub name: ImportedField<String>,
    pub animal: ImportedField<String>,
    pub mature: ImportedField<bool>,
    pub incredible: ImportedField<bool>,
    pub incredible_type: ImportedField<String>,
    pub specializations: SourceList<String>,
    pub armor: ImportedField<String>,
}

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
    fn field<T: DeserializeOwned + ExportShape>(
        build: &BTreeMap<String, &RawValue>,
        key: &str,
    ) -> ImportedField<T> {
        build
            .get(key)
            .map_or(ImportedField::Missing, |raw| read_field(raw))
    }
    let (ImportedField::Value(name), ImportedField::Value(level)) =
        (field(&build, "name"), field(&build, "level"))
    else {
        return Err(invalid());
    };
    Ok(PathbuilderSheet {
        name,
        level,
        class: field(&build, "class"),
        xp: field(&build, "xp"),
        ancestry: field(&build, "ancestry"),
        heritage: field(&build, "heritage"),
        background: field(&build, "background"),
        alignment: field(&build, "alignment"),
        gender: field(&build, "gender"),
        age: field(&build, "age"),
        deity: field(&build, "deity"),
        size: field(&build, "size"),
        size_name: field(&build, "sizeName"),
        key_ability: field(&build, "keyability"),
        languages: field(&build, "languages"),
        resistances: field(&build, "resistances"),
        abilities: field(&build, "abilities"),
        attributes: field(&build, "attributes"),
        proficiencies: field(&build, "proficiencies"),
        specific_proficiencies: field(&build, "specificProficiencies"),
        lores: field(&build, "lores"),
        mods: field(&build, "mods"),
        ac_total: field(&build, "acTotal"),
        feats: field(&build, "feats"),
        specials: field(&build, "specials"),
        spell_casters: field(&build, "spellCasters"),
        focus_points: field(&build, "focusPoints"),
        focus: field(&build, "focus"),
        equipment: field(&build, "equipment"),
        equipment_containers: field(&build, "equipmentContainers"),
        weapons: field(&build, "weapons"),
        armor: field(&build, "armor"),
        money: field(&build, "money"),
        formula: field(&build, "formula"),
        pets: field(&build, "pets"),
    })
}
