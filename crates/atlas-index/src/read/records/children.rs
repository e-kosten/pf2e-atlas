use std::collections::BTreeMap;

use atlas_domain::{MetricDomain, MetricValueType, RecordKey};
use atlas_record::{
    ActivityRoll, ActivityRollAbility, ActivityRollSurface, DamageEffectKind, DamageExpression,
    MechanicActivity, MechanicActivityKind, MechanicActivityMode, MechanicActivityUsage, MetricRow,
    MetricValue, SpellcastingEntryMechanics, SpellcastingPreparation,
};
use diesel::prelude::*;
use diesel::sqlite::Sqlite;
use diesel::{Queryable, Selectable, SelectableHelper, SqliteConnection};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::schema::{record_activities, record_spellcasting_entries};

use super::RecordLoadError;

pub(crate) fn read_activities(
    connection: &mut SqliteConnection,
) -> Result<BTreeMap<String, Vec<MechanicActivity>>, RecordLoadError> {
    let rows = record_activities::table
        .select(ActivityRow::as_select())
        .order((
            record_activities::record_key.asc(),
            record_activities::ordinal.asc(),
        ))
        .load::<ActivityRow>(connection)
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?;
    activities_from_rows(rows)
}

pub(crate) fn read_activities_by_keys(
    connection: &mut SqliteConnection,
    keys: &[RecordKey],
) -> Result<BTreeMap<String, Vec<MechanicActivity>>, RecordLoadError> {
    if keys.is_empty() {
        return Ok(BTreeMap::new());
    }
    let key_strings = keys.iter().map(ToString::to_string).collect::<Vec<_>>();
    let rows = record_activities::table
        .filter(record_activities::record_key.eq_any(key_strings))
        .select(ActivityRow::as_select())
        .order((
            record_activities::record_key.asc(),
            record_activities::ordinal.asc(),
        ))
        .load::<ActivityRow>(connection)
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?;
    activities_from_rows(rows)
}

pub(crate) fn read_spellcasting_entries(
    connection: &mut SqliteConnection,
) -> Result<BTreeMap<String, Vec<SpellcastingEntryMechanics>>, RecordLoadError> {
    let rows = record_spellcasting_entries::table
        .select(SpellcastingRow::as_select())
        .order((
            record_spellcasting_entries::record_key.asc(),
            record_spellcasting_entries::ordinal.asc(),
        ))
        .load::<SpellcastingRow>(connection)
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?;
    spellcasting_from_rows(rows)
}

pub(crate) fn read_spellcasting_entries_by_keys(
    connection: &mut SqliteConnection,
    keys: &[RecordKey],
) -> Result<BTreeMap<String, Vec<SpellcastingEntryMechanics>>, RecordLoadError> {
    if keys.is_empty() {
        return Ok(BTreeMap::new());
    }
    let key_strings = keys.iter().map(ToString::to_string).collect::<Vec<_>>();
    let rows = record_spellcasting_entries::table
        .filter(record_spellcasting_entries::record_key.eq_any(key_strings))
        .select(SpellcastingRow::as_select())
        .order((
            record_spellcasting_entries::record_key.asc(),
            record_spellcasting_entries::ordinal.asc(),
        ))
        .load::<SpellcastingRow>(connection)
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?;
    spellcasting_from_rows(rows)
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = record_activities)]
#[diesel(check_for_backend(Sqlite))]
struct ActivityRow {
    record_key: String,
    activity_id: String,
    ordinal: i64,
    payload_json: String,
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = record_spellcasting_entries)]
#[diesel(check_for_backend(Sqlite))]
struct SpellcastingRow {
    record_key: String,
    entry_id: String,
    ordinal: i64,
    payload_json: String,
}

fn activities_from_rows(
    rows: Vec<ActivityRow>,
) -> Result<BTreeMap<String, Vec<MechanicActivity>>, RecordLoadError> {
    let mut grouped = BTreeMap::<String, Vec<MechanicActivity>>::new();
    for row in rows {
        let expected = grouped.get(&row.record_key).map_or(0, Vec::len);
        validate_ordinal("record_activities", &row.record_key, row.ordinal, expected)?;
        let path = format!(
            "record_activities[{}:{}].payload_json",
            row.record_key, row.ordinal
        );
        let activity =
            decode_activity(&row.payload_json, &path).map_err(RecordLoadError::InvalidData)?;
        if activity.activity_id != row.activity_id {
            return Err(RecordLoadError::InvalidData(format!(
                "{path}.activity_id `{}` does not match stored activity_id `{}`",
                activity.activity_id, row.activity_id
            )));
        }
        grouped.entry(row.record_key).or_default().push(activity);
    }
    Ok(grouped)
}

fn spellcasting_from_rows(
    rows: Vec<SpellcastingRow>,
) -> Result<BTreeMap<String, Vec<SpellcastingEntryMechanics>>, RecordLoadError> {
    let mut grouped = BTreeMap::<String, Vec<SpellcastingEntryMechanics>>::new();
    for row in rows {
        let expected = grouped.get(&row.record_key).map_or(0, Vec::len);
        validate_ordinal(
            "record_spellcasting_entries",
            &row.record_key,
            row.ordinal,
            expected,
        )?;
        let path = format!(
            "record_spellcasting_entries[{}:{}].payload_json",
            row.record_key, row.entry_id
        );
        let entry = decode_spellcasting_entry(&row.payload_json, &path)
            .map_err(RecordLoadError::InvalidData)?;
        if entry.entry_id != row.entry_id {
            return Err(RecordLoadError::InvalidData(format!(
                "{path}.entry_id `{}` does not match relational child ID `{}`",
                entry.entry_id, row.entry_id
            )));
        }
        grouped.entry(row.record_key).or_default().push(entry);
    }
    Ok(grouped)
}

fn validate_ordinal(
    table: &str,
    record_key: &str,
    actual: i64,
    expected: usize,
) -> Result<(), RecordLoadError> {
    let expected = i64::try_from(expected).map_err(|_| {
        RecordLoadError::InvalidData(format!("{table}[{record_key}] has too many child rows"))
    })?;
    if actual != expected {
        return Err(RecordLoadError::InvalidData(format!(
            "{table}[{record_key}] expected ordinal {expected}, found {actual}"
        )));
    }
    Ok(())
}

pub(crate) fn encode_activity(value: &MechanicActivity) -> Result<String, String> {
    encode_canonical(&ActivityPayload::from(value))
}

pub(crate) fn decode_activity(value: &str, path: &str) -> Result<MechanicActivity, String> {
    decode_canonical::<ActivityPayload>(value, path).map(Into::into)
}

pub(crate) fn encode_spellcasting_entry(
    value: &SpellcastingEntryMechanics,
) -> Result<String, String> {
    encode_canonical(&SpellcastingPayload::from(value))
}

pub(crate) fn decode_spellcasting_entry(
    value: &str,
    path: &str,
) -> Result<SpellcastingEntryMechanics, String> {
    decode_canonical::<SpellcastingPayload>(value, path).map(Into::into)
}

pub(crate) fn metric_order_digest(values: &[MetricRow]) -> Result<String, String> {
    digest_payloads(&values.iter().map(MetricPayload::from).collect::<Vec<_>>())
}

pub(crate) fn activity_order_digest(values: &[MechanicActivity]) -> Result<String, String> {
    digest_payloads(&values.iter().map(ActivityPayload::from).collect::<Vec<_>>())
}

pub(crate) fn spellcasting_order_digest(
    values: &[SpellcastingEntryMechanics],
) -> Result<String, String> {
    digest_payloads(
        &values
            .iter()
            .map(SpellcastingPayload::from)
            .collect::<Vec<_>>(),
    )
}

fn encode_canonical<T>(value: &T) -> Result<String, String>
where
    T: Serialize + DeserializeOwned + PartialEq,
{
    let encoded = serde_json::to_string(value).map_err(|error| error.to_string())?;
    let decoded = serde_json::from_str::<T>(&encoded).map_err(|error| error.to_string())?;
    if &decoded != value {
        return Err("typed mechanics payload failed canonical round trip".to_string());
    }
    Ok(encoded)
}

fn decode_canonical<T>(value: &str, path: &str) -> Result<T, String>
where
    T: Serialize + DeserializeOwned,
{
    let decoded = serde_json::from_str::<T>(value).map_err(|error| format!("{path}: {error}"))?;
    let canonical = serde_json::to_string(&decoded).map_err(|error| format!("{path}: {error}"))?;
    if canonical != value {
        return Err(format!(
            "{path}: typed mechanics payload is not canonical JSON"
        ));
    }
    Ok(decoded)
}

fn digest_payloads<T: Serialize>(values: &[T]) -> Result<String, String> {
    let encoded = serde_json::to_vec(values).map_err(|error| error.to_string())?;
    Ok(format!("{:x}", Sha256::digest(encoded)))
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct MetricPayload {
    domain: MetricDomain,
    key: String,
    value: MetricValuePayload,
}

impl From<&MetricRow> for MetricPayload {
    fn from(value: &MetricRow) -> Self {
        Self {
            domain: value.domain,
            key: value.key.clone(),
            value: MetricValuePayload::from(&value.value),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
enum MetricValuePayload {
    Number(f64),
    Text(String),
    Boolean(bool),
}

impl From<&MetricValue> for MetricValuePayload {
    fn from(value: &MetricValue) -> Self {
        match value {
            MetricValue::Number(value) => Self::Number(*value),
            MetricValue::Text(value) => Self::Text(value.clone()),
            MetricValue::Boolean(value) => Self::Boolean(*value),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SpellcastingPayload {
    entry_id: String,
    label: String,
    preparation: PreparationPayload,
    spell_attack: Option<i64>,
    spell_dc: Option<i64>,
}

impl From<&SpellcastingEntryMechanics> for SpellcastingPayload {
    fn from(value: &SpellcastingEntryMechanics) -> Self {
        Self {
            entry_id: value.entry_id.clone(),
            label: value.label.clone(),
            preparation: PreparationPayload::from(&value.preparation),
            spell_attack: value.spell_attack,
            spell_dc: value.spell_dc,
        }
    }
}

impl From<SpellcastingPayload> for SpellcastingEntryMechanics {
    fn from(value: SpellcastingPayload) -> Self {
        Self {
            entry_id: value.entry_id,
            label: value.label,
            preparation: value.preparation.into(),
            spell_attack: value.spell_attack,
            spell_dc: value.spell_dc,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
enum PreparationPayload {
    Prepared,
    Spontaneous,
    Focus,
    Innate,
    Other(String),
}

impl From<&SpellcastingPreparation> for PreparationPayload {
    fn from(value: &SpellcastingPreparation) -> Self {
        match value {
            SpellcastingPreparation::Prepared => Self::Prepared,
            SpellcastingPreparation::Spontaneous => Self::Spontaneous,
            SpellcastingPreparation::Focus => Self::Focus,
            SpellcastingPreparation::Innate => Self::Innate,
            SpellcastingPreparation::Other(value) => Self::Other(value.clone()),
        }
    }
}

impl From<PreparationPayload> for SpellcastingPreparation {
    fn from(value: PreparationPayload) -> Self {
        match value {
            PreparationPayload::Prepared => Self::Prepared,
            PreparationPayload::Spontaneous => Self::Spontaneous,
            PreparationPayload::Focus => Self::Focus,
            PreparationPayload::Innate => Self::Innate,
            PreparationPayload::Other(value) => Self::Other(value),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ActivityPayload {
    activity_id: String,
    label: String,
    kind: ActivityKindPayload,
    traits: Vec<String>,
    compendium_source: Option<String>,
    usage: ActivityUsagePayload,
    rolls: Vec<ActivityRollPayload>,
    damage: Vec<DamagePayload>,
    modes: Vec<ActivityModePayload>,
}

impl From<&MechanicActivity> for ActivityPayload {
    fn from(value: &MechanicActivity) -> Self {
        Self {
            activity_id: value.activity_id.clone(),
            label: value.label.clone(),
            kind: value.kind.into(),
            traits: value.traits.clone(),
            compendium_source: value.compendium_source.clone(),
            usage: value.usage.into(),
            rolls: value.rolls.iter().map(ActivityRollPayload::from).collect(),
            damage: value.damage.iter().map(DamagePayload::from).collect(),
            modes: value.modes.iter().map(ActivityModePayload::from).collect(),
        }
    }
}

impl From<ActivityPayload> for MechanicActivity {
    fn from(value: ActivityPayload) -> Self {
        Self {
            activity_id: value.activity_id,
            label: value.label,
            kind: value.kind.into(),
            traits: value.traits,
            compendium_source: value.compendium_source,
            usage: value.usage.into(),
            rolls: value.rolls.into_iter().map(Into::into).collect(),
            damage: value.damage.into_iter().map(Into::into).collect(),
            modes: value.modes.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ActivityKindPayload {
    Strike,
    Spell,
    Other,
}

impl From<MechanicActivityKind> for ActivityKindPayload {
    fn from(value: MechanicActivityKind) -> Self {
        match value {
            MechanicActivityKind::Strike => Self::Strike,
            MechanicActivityKind::Spell => Self::Spell,
            MechanicActivityKind::Other => Self::Other,
        }
    }
}

impl From<ActivityKindPayload> for MechanicActivityKind {
    fn from(value: ActivityKindPayload) -> Self {
        match value {
            ActivityKindPayload::Strike => Self::Strike,
            ActivityKindPayload::Spell => Self::Spell,
            ActivityKindPayload::Other => Self::Other,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ActivityUsagePayload {
    Unlimited,
    Limited,
    Ambiguous,
}

impl From<MechanicActivityUsage> for ActivityUsagePayload {
    fn from(value: MechanicActivityUsage) -> Self {
        match value {
            MechanicActivityUsage::Unlimited => Self::Unlimited,
            MechanicActivityUsage::Limited => Self::Limited,
            MechanicActivityUsage::Ambiguous => Self::Ambiguous,
        }
    }
}

impl From<ActivityUsagePayload> for MechanicActivityUsage {
    fn from(value: ActivityUsagePayload) -> Self {
        match value {
            ActivityUsagePayload::Unlimited => Self::Unlimited,
            ActivityUsagePayload::Limited => Self::Limited,
            ActivityUsagePayload::Ambiguous => Self::Ambiguous,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ActivityRollPayload {
    roll_id: String,
    label: String,
    base_value: i64,
    surface: RollSurfacePayload,
    ability: Option<AbilityPayload>,
}

impl From<&ActivityRoll> for ActivityRollPayload {
    fn from(value: &ActivityRoll) -> Self {
        Self {
            roll_id: value.roll_id.clone(),
            label: value.label.clone(),
            base_value: value.base_value,
            surface: value.surface.into(),
            ability: value.ability.map(Into::into),
        }
    }
}

impl From<ActivityRollPayload> for ActivityRoll {
    fn from(value: ActivityRollPayload) -> Self {
        Self {
            roll_id: value.roll_id,
            label: value.label,
            base_value: value.base_value,
            surface: value.surface.into(),
            ability: value.ability.map(Into::into),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum RollSurfacePayload {
    AttackRoll,
    Dc,
}

impl From<ActivityRollSurface> for RollSurfacePayload {
    fn from(value: ActivityRollSurface) -> Self {
        match value {
            ActivityRollSurface::AttackRoll => Self::AttackRoll,
            ActivityRollSurface::Dc => Self::Dc,
        }
    }
}

impl From<RollSurfacePayload> for ActivityRollSurface {
    fn from(value: RollSurfacePayload) -> Self {
        match value {
            RollSurfacePayload::AttackRoll => Self::AttackRoll,
            RollSurfacePayload::Dc => Self::Dc,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum AbilityPayload {
    Strength,
    Dexterity,
    Constitution,
    Intelligence,
    Wisdom,
    Charisma,
}

impl From<ActivityRollAbility> for AbilityPayload {
    fn from(value: ActivityRollAbility) -> Self {
        match value {
            ActivityRollAbility::Strength => Self::Strength,
            ActivityRollAbility::Dexterity => Self::Dexterity,
            ActivityRollAbility::Constitution => Self::Constitution,
            ActivityRollAbility::Intelligence => Self::Intelligence,
            ActivityRollAbility::Wisdom => Self::Wisdom,
            ActivityRollAbility::Charisma => Self::Charisma,
        }
    }
}

impl From<AbilityPayload> for ActivityRollAbility {
    fn from(value: AbilityPayload) -> Self {
        match value {
            AbilityPayload::Strength => Self::Strength,
            AbilityPayload::Dexterity => Self::Dexterity,
            AbilityPayload::Constitution => Self::Constitution,
            AbilityPayload::Intelligence => Self::Intelligence,
            AbilityPayload::Wisdom => Self::Wisdom,
            AbilityPayload::Charisma => Self::Charisma,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct DamagePayload {
    damage_id: String,
    label: Option<String>,
    formula: String,
    damage_type: Option<String>,
    effect_kind: DamageEffectPayload,
    ability: Option<AbilityPayload>,
}

impl From<&DamageExpression> for DamagePayload {
    fn from(value: &DamageExpression) -> Self {
        Self {
            damage_id: value.damage_id.clone(),
            label: value.label.clone(),
            formula: value.formula.clone(),
            damage_type: value.damage_type.clone(),
            effect_kind: value.effect_kind.into(),
            ability: value.ability.map(Into::into),
        }
    }
}

impl From<DamagePayload> for DamageExpression {
    fn from(value: DamagePayload) -> Self {
        Self {
            damage_id: value.damage_id,
            label: value.label,
            formula: value.formula,
            damage_type: value.damage_type,
            effect_kind: value.effect_kind.into(),
            ability: value.ability.map(Into::into),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum DamageEffectPayload {
    Damage,
    Healing,
    DamageOrHealing,
    Unknown,
}

impl From<DamageEffectKind> for DamageEffectPayload {
    fn from(value: DamageEffectKind) -> Self {
        match value {
            DamageEffectKind::Damage => Self::Damage,
            DamageEffectKind::Healing => Self::Healing,
            DamageEffectKind::DamageOrHealing => Self::DamageOrHealing,
            DamageEffectKind::Unknown => Self::Unknown,
        }
    }
}

impl From<DamageEffectPayload> for DamageEffectKind {
    fn from(value: DamageEffectPayload) -> Self {
        match value {
            DamageEffectPayload::Damage => Self::Damage,
            DamageEffectPayload::Healing => Self::Healing,
            DamageEffectPayload::DamageOrHealing => Self::DamageOrHealing,
            DamageEffectPayload::Unknown => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ActivityModePayload {
    mode_id: String,
    label: String,
    sort: i64,
    target: Option<String>,
    range: Option<String>,
    time: Option<String>,
    damage: Vec<DamagePayload>,
}

impl From<&MechanicActivityMode> for ActivityModePayload {
    fn from(value: &MechanicActivityMode) -> Self {
        Self {
            mode_id: value.mode_id.clone(),
            label: value.label.clone(),
            sort: value.sort,
            target: value.target.clone(),
            range: value.range.clone(),
            time: value.time.clone(),
            damage: value.damage.iter().map(DamagePayload::from).collect(),
        }
    }
}

impl From<ActivityModePayload> for MechanicActivityMode {
    fn from(value: ActivityModePayload) -> Self {
        Self {
            mode_id: value.mode_id,
            label: value.label,
            sort: value.sort,
            target: value.target,
            range: value.range,
            time: value.time,
            damage: value.damage.into_iter().map(Into::into).collect(),
        }
    }
}

pub(crate) fn metric_from_storage(
    domain: &str,
    key: String,
    value_type: &str,
    number_value: Option<f64>,
    text_value: Option<String>,
    bool_value: Option<bool>,
) -> Result<MetricRow, String> {
    let domain = MetricDomain::from_canonical(domain)
        .ok_or_else(|| format!("record_metrics.metric_domain: invalid value `{domain}`"))?;
    let value = match MetricValueType::from_canonical(value_type)
        .ok_or_else(|| format!("record_metrics.value_type: invalid value `{value_type}`"))?
    {
        MetricValueType::Number => {
            MetricValue::Number(required("record_metrics.number_value", number_value)?)
        }
        MetricValueType::Text => {
            MetricValue::Text(required("record_metrics.text_value", text_value)?)
        }
        MetricValueType::Boolean => {
            MetricValue::Boolean(required("record_metrics.bool_value", bool_value)?)
        }
    };
    Ok(MetricRow { domain, key, value })
}

fn required<T>(path: &str, value: Option<T>) -> Result<T, String> {
    value.ok_or_else(|| format!("{path}: required value is missing"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typed_mechanics_payloads_round_trip_and_bind_vector_order() {
        let first = MechanicActivity {
            activity_id: "first".to_string(),
            label: "First".to_string(),
            kind: MechanicActivityKind::Other,
            traits: Vec::new(),
            compendium_source: None,
            usage: MechanicActivityUsage::Unlimited,
            rolls: Vec::new(),
            damage: Vec::new(),
            modes: Vec::new(),
        };
        let second = MechanicActivity {
            activity_id: "second".to_string(),
            label: "Second".to_string(),
            ..first.clone()
        };
        let encoded = encode_activity(&first).expect("activity should encode");
        assert_eq!(
            decode_activity(&encoded, "fixture.activity").expect("activity should decode"),
            first
        );
        assert_ne!(
            activity_order_digest(&[first.clone(), second.clone()]).unwrap(),
            activity_order_digest(&[second, first]).unwrap()
        );

        let entry = SpellcastingEntryMechanics {
            entry_id: "entry".to_string(),
            label: "Entry".to_string(),
            preparation: SpellcastingPreparation::Other("ritual".to_string()),
            spell_attack: None,
            spell_dc: Some(23),
        };
        let encoded = encode_spellcasting_entry(&entry).expect("entry should encode");
        assert_eq!(
            decode_spellcasting_entry(&encoded, "fixture.entry").expect("entry should decode"),
            entry
        );
    }

    #[test]
    fn activity_rows_use_parent_ordinal_identity_and_preserve_duplicate_payload_ids() {
        let first = MechanicActivity {
            activity_id: "repeated-source-id".to_string(),
            label: "First occurrence".to_string(),
            kind: MechanicActivityKind::Other,
            traits: Vec::new(),
            compendium_source: None,
            usage: MechanicActivityUsage::Ambiguous,
            rolls: Vec::new(),
            damage: Vec::new(),
            modes: Vec::new(),
        };
        let second = MechanicActivity {
            label: "Second occurrence".to_string(),
            ..first.clone()
        };
        let rows = vec![
            ActivityRow {
                record_key: "actions:duplicate".to_string(),
                activity_id: first.activity_id.clone(),
                ordinal: 0,
                payload_json: encode_activity(&first).expect("first activity should encode"),
            },
            ActivityRow {
                record_key: "actions:duplicate".to_string(),
                activity_id: second.activity_id.clone(),
                ordinal: 1,
                payload_json: encode_activity(&second).expect("second activity should encode"),
            },
        ];

        let activities = activities_from_rows(rows).expect("duplicate payload IDs should hydrate");
        let activities = activities
            .get("actions:duplicate")
            .expect("parent should be grouped");
        assert_eq!(
            activities
                .iter()
                .map(|activity| activity.activity_id.as_str())
                .collect::<Vec<_>>(),
            vec!["repeated-source-id", "repeated-source-id"]
        );
        assert_eq!(
            activities
                .iter()
                .map(|activity| activity.label.as_str())
                .collect::<Vec<_>>(),
            vec!["First occurrence", "Second occurrence"]
        );

        let reordered = vec![
            ActivityRow {
                record_key: "actions:duplicate".to_string(),
                activity_id: first.activity_id.clone(),
                ordinal: 1,
                payload_json: encode_activity(&first).expect("first activity should encode"),
            },
            ActivityRow {
                record_key: "actions:duplicate".to_string(),
                activity_id: second.activity_id.clone(),
                ordinal: 0,
                payload_json: encode_activity(&second).expect("second activity should encode"),
            },
        ];
        assert!(
            activities_from_rows(reordered)
                .expect_err("out-of-order rows must fail closed")
                .to_string()
                .contains("expected ordinal 0, found 1")
        );
    }

    #[test]
    fn typed_mechanics_payloads_reject_noncanonical_and_unknown_fields() {
        let activity = MechanicActivity {
            activity_id: "activity".to_string(),
            label: "Activity".to_string(),
            kind: MechanicActivityKind::Other,
            traits: Vec::new(),
            compendium_source: None,
            usage: MechanicActivityUsage::Unlimited,
            rolls: Vec::new(),
            damage: Vec::new(),
            modes: Vec::new(),
        };
        let encoded = encode_activity(&activity).expect("activity should encode");
        let noncanonical = serde_json::to_string_pretty(
            &serde_json::from_str::<serde_json::Value>(&encoded).unwrap(),
        )
        .unwrap();
        assert!(
            decode_activity(&noncanonical, "fixture.activity")
                .expect_err("noncanonical JSON must fail")
                .contains("not canonical JSON")
        );

        let unknown = format!("{},\"unknown\":true}}", encoded.trim_end_matches('}'));
        assert!(
            decode_activity(&unknown, "fixture.activity")
                .expect_err("unknown field must fail")
                .contains("unknown field")
        );
    }
}
