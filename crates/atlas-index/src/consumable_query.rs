use std::collections::BTreeSet;

use atlas_record::{ConsumableDefinition, ConsumablePrice, ConsumableSourceValue, FactValue};

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ConsumableQueryProjection {
    pub(crate) category: Option<String>,
    pub(crate) usage: Option<String>,
    pub(crate) base_item: Option<String>,
    pub(crate) bulk_value: Option<f64>,
    pub(crate) hands_requirement: Option<String>,
    pub(crate) price_cp: Option<i64>,
    pub(crate) damage_types: Vec<String>,
}

pub(crate) fn project_consumable_query(
    definition: &ConsumableDefinition,
) -> ConsumableQueryProjection {
    let usage = known(&definition.usage).cloned();
    ConsumableQueryProjection {
        category: known(&definition.category).cloned(),
        usage: usage.clone(),
        base_item: known(&definition.base_item).cloned(),
        bulk_value: known(&definition.bulk).and_then(atlas_record::ConsumableExactDecimal::as_f64),
        hands_requirement: usage
            .as_deref()
            .and_then(atlas_record::hands_requirement_from_usage)
            .map(str::to_string),
        price_cp: known(&definition.price).and_then(price_cp),
        damage_types: known(&definition.damage)
            .and_then(|damage| known(&damage.damage_type).cloned())
            .into_iter()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect(),
    }
}

fn known<T>(fact: &FactValue<ConsumableSourceValue<T>>) -> Option<&T> {
    fact.as_value().and_then(ConsumableSourceValue::known)
}

fn price_cp(price: &ConsumablePrice) -> Option<i64> {
    match known(&price.per) {
        Some(per) if *per <= 0 => return None,
        _ => {}
    }
    let denominations = known(&price.denominations)?;
    if denominations.is_empty() {
        return None;
    }
    denominations.iter().try_fold(0_i64, |total, value| {
        let multiplier = match value.denomination.as_str() {
            "cp" => 1_i64,
            "sp" => 10,
            "gp" => 100,
            "pp" => 1_000,
            _ => return None,
        };
        if value.amount < 0 {
            return None;
        }
        total.checked_add(value.amount.checked_mul(multiplier)?)
    })
}

#[cfg(test)]
mod tests {
    use atlas_record::{
        ConsumablePrice, ConsumablePriceDenomination, ConsumableSourceValue, FactValue,
    };

    use super::price_cp;

    fn known<T>(value: T) -> FactValue<ConsumableSourceValue<T>> {
        FactValue::Value(ConsumableSourceValue::Known(value))
    }

    #[test]
    fn price_projection_is_checked_and_never_fabricates_zero() {
        assert_eq!(
            price_cp(&ConsumablePrice {
                denominations: known(vec![
                    ConsumablePriceDenomination {
                        denomination: "gp".to_string(),
                        amount: 2,
                    },
                    ConsumablePriceDenomination {
                        denomination: "sp".to_string(),
                        amount: 3,
                    },
                ]),
                per: FactValue::Null,
            }),
            Some(230)
        );
        assert_eq!(
            price_cp(&ConsumablePrice {
                denominations: known(Vec::new()),
                per: FactValue::Null,
            }),
            None
        );
        assert_eq!(
            price_cp(&ConsumablePrice {
                denominations: known(vec![ConsumablePriceDenomination {
                    denomination: "gp".to_string(),
                    amount: i64::MAX,
                }]),
                per: FactValue::Null,
            }),
            None
        );
    }
}
