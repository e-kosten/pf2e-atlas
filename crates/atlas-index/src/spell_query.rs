use atlas_record::{
    FactValue, SpellFact, SpellNumericRangeKind, SpellRangeValue, SpellRecord, SpellSourceValue,
};

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SpellQueryProjection {
    pub(crate) rank: Option<i64>,
    pub(crate) traditions: Vec<SpellTraditionProjection>,
    pub(crate) spell_kinds: Vec<String>,
    pub(crate) range_text: Option<String>,
    pub(crate) range_value: Option<f64>,
    pub(crate) range_kind: Option<String>,
    pub(crate) range_rule: Option<String>,
    pub(crate) target_text: Option<String>,
    pub(crate) area_type: Option<String>,
    pub(crate) area_value: Option<f64>,
    pub(crate) save_type: Option<String>,
    pub(crate) sustained: bool,
    pub(crate) basic_save: Option<bool>,
    pub(crate) damage_types: Vec<SpellDamageTypeProjection>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct SpellTraditionProjection {
    pub(crate) authored_order: i64,
    pub(crate) tradition: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct SpellDamageTypeProjection {
    pub(crate) damage_key: String,
    pub(crate) damage_authored_order: i64,
    pub(crate) type_authored_order: i64,
    pub(crate) damage_type: String,
}

impl SpellQueryProjection {
    pub(crate) fn from_spell(spell: &SpellRecord) -> Result<Self, String> {
        let classification = known(&spell.definition.classification);
        let rank = classification
            .and_then(|value| known(&value.rank))
            .map(|value| i64::from(*value));
        let traditions = classification
            .and_then(|value| known(&value.traditions))
            .map(|values| {
                values
                    .iter()
                    .enumerate()
                    .map(|(authored_order, value)| {
                        Ok(SpellTraditionProjection {
                            authored_order: i64::try_from(authored_order).map_err(|_| {
                                "spell tradition authored order exceeds SQLite range".to_string()
                            })?,
                            tradition: value.as_str().to_string(),
                        })
                    })
                    .collect::<Result<Vec<_>, String>>()
            })
            .transpose()?
            .unwrap_or_default();
        let spell_kinds = classification
            .and_then(|value| known(&value.traits))
            .map(|traits| {
                ["focus", "ritual", "cantrip"]
                    .into_iter()
                    .filter(|kind| traits.iter().any(|value| value.as_str() == *kind))
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default();

        let targeting = known(&spell.definition.targeting);
        let range = targeting.and_then(|value| known(&value.range));
        if range.is_some_and(|value| !value.has_valid_projection()) {
            return Err(format!(
                "spell `{}` has a stale numeric range projection",
                spell.identity.record_key
            ));
        }
        let numeric_range = range.and_then(|value| {
            SpellRangeValue::from_authored_text(value.authored_text.clone()).numeric
        });
        let (range_kind, range_rule, range_value) =
            numeric_range.as_ref().map_or((None, None, None), |value| {
                (
                    Some(
                        match value.kind {
                            SpellNumericRangeKind::TouchMelee => "touch_melee",
                            SpellNumericRangeKind::Distance => "distance",
                        }
                        .to_string(),
                    ),
                    Some(value.rule.clone()),
                    Some(f64::from(value.feet)),
                )
            });
        let area = targeting.and_then(|value| known(&value.area));
        let defense = known(&spell.definition.defense);
        let save = defense.and_then(|value| known(&value.save));
        let duration = known(&spell.definition.duration);

        let mut damage_types = Vec::new();
        if let Some(damage) = known(&spell.definition.damage) {
            for member in damage {
                if let Some(damage_type) = known(&member.value.damage_type) {
                    damage_types.push(SpellDamageTypeProjection {
                        damage_key: member.key.clone(),
                        damage_authored_order: i64::from(member.authored_order),
                        type_authored_order: 0,
                        damage_type: damage_type.clone(),
                    });
                }
            }
        }

        Ok(Self {
            rank,
            traditions,
            spell_kinds,
            range_text: range.map(|value| value.authored_text.clone()),
            range_value,
            range_kind,
            range_rule,
            target_text: targeting.and_then(|value| known(&value.target)).cloned(),
            area_type: area
                .and_then(|value| known(&value.area_type))
                .map(|value| value.as_str().to_string()),
            area_value: area
                .and_then(|value| known(&value.value))
                .map(|value| f64::from(*value)),
            save_type: save
                .and_then(|value| known(&value.statistic))
                .map(|value| value.as_str().to_string()),
            sustained: duration
                .and_then(|value| known(&value.sustained))
                .copied()
                .unwrap_or(false),
            basic_save: save.and_then(|value| known(&value.basic)).copied(),
            damage_types,
        })
    }

    pub(crate) fn tradition_values(&self) -> Vec<String> {
        self.traditions
            .iter()
            .map(|value| value.tradition.clone())
            .collect()
    }

    pub(crate) fn damage_type_values(&self) -> Vec<String> {
        self.damage_types
            .iter()
            .map(|value| value.damage_type.clone())
            .collect()
    }
}

fn known<T>(fact: &SpellFact<T>) -> Option<&T> {
    match fact {
        FactValue::Value(SpellSourceValue::Known(value)) => Some(value),
        FactValue::Missing
        | FactValue::Null
        | FactValue::Value(SpellSourceValue::Unsupported(_)) => None,
    }
}
