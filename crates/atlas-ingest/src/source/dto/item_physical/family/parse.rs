use super::super::super::fields::{
    Fields, ParseResult, boolean, malformed, number, shape_error, string, strings,
};
use super::super::super::item_common::{CommonItemSource, number_value};
use super::super::super::{ItemType, SerializedSourceValue, SourceIdentity, SourcePresence};
use super::super::{PhysicalSystemSource, text_value};
use super::*;

pub(in crate::source::dto::item_physical) fn validate_field_applicability(
    item_type: ItemType,
    fields: &Fields<'_>,
) -> ParseResult<()> {
    let forbidden: &[&str] = match item_type {
        ItemType::Backpack | ItemType::Book => &["subitems"],
        ItemType::Treasure => &["apex", "subitems", "usage"],
        _ => &[],
    };
    for key in forbidden {
        fields.absent(key)?;
    }
    Ok(())
}

pub(in crate::source::dto::item_physical) fn refine(
    common: &mut CommonItemSource,
    physical: &mut PhysicalSystemSource,
    physical_fields: &Fields<'_>,
) -> ParseResult<PhysicalFamilySource> {
    let remaining = std::mem::take(&mut common.system.pending_family_fields);
    let f = Fields {
        object: &remaining,
        identity: physical_fields.identity,
        path: physical_fields.path,
    };
    let family = match common.envelope.item_type {
        ItemType::Equipment => {
            let legacy = EquipmentLegacySource {
                stowing: f.presence("stowing", boolean)?,
                ability: f.presence("ability", text_value)?,
                map: f.presence("MAP", text_value)?,
                bonus: f.presence("bonus", number_value)?,
                bonus_damage: f.presence("bonusDamage", number_value)?,
                splash_damage: f.presence("splashDamage", number_value)?,
                damage: f.presence("damage", legacy_damage)?,
                range: f.presence("range", text_value)?,
                reload: f.presence("reload", text_value)?,
                weapon_type: f.presence("weaponType", text_value)?,
            };
            common.system.pending_family_fields = f.rest(&[
                "stowing",
                "ability",
                "MAP",
                "bonus",
                "bonusDamage",
                "splashDamage",
                "damage",
                "range",
                "reload",
                "weaponType",
            ]);
            PhysicalFamilySource::Equipment(Box::new(EquipmentFamilySource { legacy }))
        }
        ItemType::Backpack => {
            let family = BackpackFamilySource {
                stowing: f.presence("stowing", boolean)?,
                collapsed: f.presence("collapsed", boolean)?,
                bulk: backpack_bulk(physical, &f)?,
            };
            common.system.pending_family_fields = f.rest(&["stowing", "collapsed"]);
            PhysicalFamilySource::Backpack(family)
        }
        ItemType::Book => {
            let family = BookFamilySource {
                category: f.presence("category", |v, i, p| match string(v, i, p)?.as_str() {
                    "formula" => Ok(BookCategorySource::Formula),
                    "spell" => Ok(BookCategorySource::Spell),
                    _ => Err(shape_error(v, i, p, "formula | spell")),
                })?,
                capacity: f.presence("capacity", number)?,
                contents: f.presence("contents", strings)?,
            };
            common.system.pending_family_fields = f.rest(&["category", "capacity", "contents"]);
            PhysicalFamilySource::Book(family)
        }
        ItemType::Treasure => {
            if let Some(traits) = common.system.traits.as_value()
                && let Some(values) = traits.value.as_value()
                && !values.is_empty()
            {
                return Err(malformed(
                    f.identity,
                    &format!("{}.traits.value", f.path),
                    "empty trait array for treasure",
                    SerializedSourceValue::Array(
                        values
                            .iter()
                            .cloned()
                            .map(SerializedSourceValue::String)
                            .collect(),
                    )
                    .compact_json(),
                ));
            }
            let family = TreasureFamilySource {
                stack_group: f.presence("stackGroup", |v, i, p| {
                    match string(v, i, p)?.as_str() {
                        "coins" => Ok(TreasureStackGroupSource::Coins),
                        "gems" => Ok(TreasureStackGroupSource::Gems),
                        _ => Err(shape_error(v, i, p, "coins | gems")),
                    }
                })?,
            };
            common.system.pending_family_fields = f.rest(&["stackGroup"]);
            PhysicalFamilySource::Treasure(family)
        }
        _ => {
            common.system.pending_family_fields = remaining;
            PhysicalFamilySource::Pending
        }
    };
    Ok(family)
}

fn backpack_bulk(
    physical: &mut PhysicalSystemSource,
    parent: &Fields<'_>,
) -> ParseResult<SourcePresence<BackpackBulkSource>> {
    match &mut physical.bulk {
        SourcePresence::Missing => Ok(SourcePresence::Missing),
        SourcePresence::Null => Ok(SourcePresence::Null),
        SourcePresence::Value(bulk) => {
            let path = format!("{}.bulk", parent.path);
            let f = Fields {
                object: &bulk.additional_fields,
                identity: parent.identity,
                path: &path,
            };
            let extra = BackpackBulkSource {
                held_or_stowed: f.presence("heldOrStowed", number)?,
                capacity: f.presence("capacity", number)?,
                ignored: f.presence("ignored", number)?,
            };
            bulk.additional_fields = f.rest(&["heldOrStowed", "capacity", "ignored"]);
            Ok(SourcePresence::Value(extra))
        }
    }
}

fn legacy_damage(
    v: &SerializedSourceValue,
    i: &SourceIdentity,
    p: &str,
) -> ParseResult<EquipmentLegacyDamageSource> {
    let f = Fields::new(v, i, p)?;
    Ok(EquipmentLegacyDamageSource {
        damage_type: f.presence("damageType", string)?,
        dice: f.presence("dice", number)?,
        die: f.presence("die", string)?,
        value: f.presence("value", string)?,
        additional_fields: f.rest(&["damageType", "dice", "die", "value"]),
    })
}
