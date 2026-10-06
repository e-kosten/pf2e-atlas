use atlas_record::{CreatureFrequencyPeriod, CreatureSize};

use super::super::fields::{
    Fields, ParseResult, array, boolean, keyed, malformed, number, object, shape_error, string,
    strings, structured_value,
};
use super::super::item_common::{ItemParentSource, parse_common_item_fields};
use super::super::{
    ItemType, SerializedSourceObject, SerializedSourceValue, SourceIdentity, SourceVersionMetadata,
    parse_serialized_source_object, validate_pinned_source_version,
};
use super::*;

/// Parse shared physical fields for roots, Actor.items and recursive subitems.
/// Concrete family admission/refinements and product conversion remain separate.
pub fn parse_physical_item_source(
    version: SourceVersionMetadata,
    identity: SourceIdentity,
    parent: Option<ItemParentSource>,
    bytes: &[u8],
) -> ParseResult<VersionedPhysicalItemSource> {
    validate_pinned_source_version(
        &identity,
        version.system_id(),
        version.system_version(),
        version.upstream_commit(),
    )?;
    let path = parent.map_or_else(
        || "$".to_string(),
        |p| format!("$.items[{}]", p.item_ordinal),
    );
    let serialized = parse_serialized_source_object(bytes)
        .map_err(|e| malformed(&identity, &path, "physical Item source JSON object", e))?;
    let source = physical_item(&serialized, &identity, &path)?;
    Ok(VersionedPhysicalItemSource {
        version,
        identity,
        parent,
        source,
        serialized,
    })
}

fn physical_item(
    v: &SerializedSourceObject,
    i: &SourceIdentity,
    p: &str,
) -> ParseResult<PhysicalItemSource> {
    let mut common = parse_common_item_fields(v, i, p)?;
    if !matches!(
        common.envelope.item_type,
        ItemType::Armor
            | ItemType::Backpack
            | ItemType::Book
            | ItemType::Consumable
            | ItemType::Equipment
            | ItemType::Shield
            | ItemType::Treasure
            | ItemType::Weapon
    ) {
        return Err(malformed(
            i,
            &format!("{p}.type"),
            "armor | backpack | book | consumable | equipment | shield | treasure | weapon",
            common.envelope.item_type.as_str(),
        ));
    }
    // Common fields have already been parsed from this tree. Consume only the
    // remaining fields, then leave concrete family fields in the common owner.
    let path = format!("{p}.system");
    let f = Fields {
        object: &common.system.pending_family_fields,
        identity: i,
        path: &path,
    };
    let physical = PhysicalSystemSource {
        quantity: f.presence("quantity", number)?,
        base_item: f.presence("baseItem", string)?,
        bulk: f.presence("bulk", bulk)?,
        hp: f.presence("hp", hp)?,
        hardness: f.presence("hardness", number)?,
        price: f.presence("price", price)?,
        equipped: f.presence("equipped", equipped)?,
        identification: f.presence("identification", identification)?,
        container_id: f.presence("containerId", string)?,
        material: f.presence("material", material)?,
        size: f.presence("size", size)?,
        usage: f.presence("usage", usage)?,
        activations: f.presence("activations", |v, i, p| keyed(v, i, p, activation))?,
        temporary: f.presence("temporary", boolean)?,
        subitems: f.presence("subitems", |v, i, p| {
            array(v, i, p, |v, i, p| {
                let v = object(v, i, p)?;
                physical_item(&v, i, p)
            })
        })?,
        apex: f.presence("apex", apex)?,
    };
    common.system.pending_family_fields = f.rest(&[
        "quantity",
        "baseItem",
        "bulk",
        "hp",
        "hardness",
        "price",
        "equipped",
        "identification",
        "containerId",
        "material",
        "size",
        "usage",
        "activations",
        "temporary",
        "subitems",
        "apex",
    ]);
    Ok(PhysicalItemSource { common, physical })
}

fn bulk(v: &SerializedSourceValue, i: &SourceIdentity, p: &str) -> ParseResult<PhysicalBulkSource> {
    let f = Fields::new(v, i, p)?;
    Ok(PhysicalBulkSource {
        value: f.presence("value", number)?,
        additional_fields: f.rest(&["value"]),
    })
}

fn hp(
    v: &SerializedSourceValue,
    i: &SourceIdentity,
    p: &str,
) -> ParseResult<PhysicalHitPointsSource> {
    let f = Fields::new(v, i, p)?;
    Ok(PhysicalHitPointsSource {
        value: f.presence("value", number)?,
        max: f.presence("max", number)?,
        additional_fields: f.rest(&["value", "max"]),
    })
}

fn price(v: &SerializedSourceValue, i: &SourceIdentity, p: &str) -> ParseResult<ItemPriceSource> {
    let f = Fields::new(v, i, p)?;
    Ok(ItemPriceSource {
        value: f.presence("value", coins)?,
        per: f.presence("per", number)?,
        size_sensitive: f.presence("sizeSensitive", boolean)?,
        additional_fields: f.rest(&["value", "per", "sizeSensitive"]),
    })
}

fn coins(v: &SerializedSourceValue, i: &SourceIdentity, p: &str) -> ParseResult<ItemCoinsSource> {
    let f = Fields::new(v, i, p)?;
    Ok(ItemCoinsSource {
        pp: f.presence("pp", number)?,
        gp: f.presence("gp", number)?,
        sp: f.presence("sp", number)?,
        cp: f.presence("cp", number)?,
        additional_fields: f.rest(&["pp", "gp", "sp", "cp"]),
    })
}

fn equipped(
    v: &SerializedSourceValue,
    i: &SourceIdentity,
    p: &str,
) -> ParseResult<ItemEquippedSource> {
    let f = Fields::new(v, i, p)?;
    Ok(ItemEquippedSource {
        carry_type: f.presence("carryType", |v, i, p| match string(v, i, p)?.as_str() {
            "attached" => Ok(ItemCarryTypeSource::Attached),
            "dropped" => Ok(ItemCarryTypeSource::Dropped),
            "held" => Ok(ItemCarryTypeSource::Held),
            "stowed" => Ok(ItemCarryTypeSource::Stowed),
            "worn" => Ok(ItemCarryTypeSource::Worn),
            _ => Err(shape_error(
                v,
                i,
                p,
                "attached | dropped | held | stowed | worn",
            )),
        })?,
        in_slot: f.presence("inSlot", boolean)?,
        hands_held: f.presence("handsHeld", |v, i, p| bounded_count(v, i, p, 0, 2))?,
        invested: f.presence("invested", boolean)?,
        additional_fields: f.rest(&["carryType", "inSlot", "handsHeld", "invested"]),
    })
}

fn identification(
    v: &SerializedSourceValue,
    i: &SourceIdentity,
    p: &str,
) -> ParseResult<ItemIdentificationSource> {
    let f = Fields::new(v, i, p)?;
    Ok(ItemIdentificationSource {
        status: f.presence("status", |v, i, p| match string(v, i, p)?.as_str() {
            "identified" => Ok(ItemIdentificationStatusSource::Identified),
            "unidentified" => Ok(ItemIdentificationStatusSource::Unidentified),
            _ => Err(shape_error(v, i, p, "identified | unidentified")),
        })?,
        unidentified: f.presence("unidentified", mystified)?,
        misidentified: f.presence("misidentified", structured_value)?,
        additional_fields: f.rest(&["status", "unidentified", "misidentified"]),
    })
}

fn mystified(
    v: &SerializedSourceValue,
    i: &SourceIdentity,
    p: &str,
) -> ParseResult<ItemMystifiedSource> {
    let f = Fields::new(v, i, p)?;
    Ok(ItemMystifiedSource {
        name: f.presence("name", string)?,
        image: f.presence("img", string)?,
        data: f.presence("data", mystified_data)?,
        additional_fields: f.rest(&["name", "img", "data"]),
    })
}

fn mystified_data(
    v: &SerializedSourceValue,
    i: &SourceIdentity,
    p: &str,
) -> ParseResult<ItemMystifiedDataSource> {
    let f = Fields::new(v, i, p)?;
    Ok(ItemMystifiedDataSource {
        description: f.presence("description", text_value)?,
        additional_fields: f.rest(&["description"]),
    })
}

fn text_value(
    v: &SerializedSourceValue,
    i: &SourceIdentity,
    p: &str,
) -> ParseResult<ItemTextValueSource> {
    let f = Fields::new(v, i, p)?;
    Ok(ItemTextValueSource {
        value: f.presence("value", string)?,
        additional_fields: f.rest(&["value"]),
    })
}

fn material(
    v: &SerializedSourceValue,
    i: &SourceIdentity,
    p: &str,
) -> ParseResult<ItemMaterialSource> {
    let f = Fields::new(v, i, p)?;
    Ok(ItemMaterialSource {
        grade: f.presence("grade", |v, i, p| match string(v, i, p)?.as_str() {
            "low" => Ok(ItemMaterialGradeSource::Low),
            "standard" => Ok(ItemMaterialGradeSource::Standard),
            "high" => Ok(ItemMaterialGradeSource::High),
            _ => Err(shape_error(v, i, p, "low | standard | high")),
        })?,
        material_type: f.presence("type", material_type)?,
        additional_fields: f.rest(&["grade", "type"]),
    })
}

fn material_type(
    v: &SerializedSourceValue,
    i: &SourceIdentity,
    p: &str,
) -> ParseResult<ItemMaterialTypeSource> {
    use ItemMaterialTypeSource::*;
    match string(v, i, p)?.as_str() {
        "abysium" => Ok(Abysium),
        "adamantine" => Ok(Adamantine),
        "cold-iron" => Ok(ColdIron),
        "duskwood" => Ok(Duskwood),
        "djezet" => Ok(Djezet),
        "dragonhide" => Ok(Dragonhide),
        "dreamweb" => Ok(Dreamweb),
        "grisantian-pelt" => Ok(GrisantianPelt),
        "inubrix" => Ok(Inubrix),
        "keep-stone" => Ok(KeepStone),
        "dawnsilver" => Ok(Dawnsilver),
        "noqual" => Ok(Noqual),
        "orichalcum" => Ok(Orichalcum),
        "peachwood" => Ok(Peachwood),
        "siccatite" => Ok(Siccatite),
        "silver" => Ok(Silver),
        "sisterstone" => Ok(Sisterstone),
        "sisterstone-dusk" => Ok(SisterstoneDusk),
        "sisterstone-scarlet" => Ok(SisterstoneScarlet),
        "sloughstone" => Ok(Sloughstone),
        "sovereign-steel" => Ok(SovereignSteel),
        "warpglass" => Ok(Warpglass),
        _ => Err(shape_error(v, i, p, "pinned precious material type")),
    }
}

fn size(v: &SerializedSourceValue, i: &SourceIdentity, p: &str) -> ParseResult<CreatureSize> {
    let token = string(v, i, p)?;
    // CreatureSize also accepts long aliases; PhysicalSystemSource declares
    // exactly these six tokens. Do not silently normalize extra spellings.
    if matches!(
        token.as_str(),
        "tiny" | "sm" | "med" | "lg" | "huge" | "grg"
    ) {
        CreatureSize::from_source(&token).ok_or_else(|| shape_error(v, i, p, "PF2e size"))
    } else {
        Err(shape_error(v, i, p, "tiny | sm | med | lg | huge | grg"))
    }
}

fn usage(
    v: &SerializedSourceValue,
    i: &SourceIdentity,
    p: &str,
) -> ParseResult<PhysicalUsageSource> {
    let f = Fields::new(v, i, p)?;
    Ok(PhysicalUsageSource {
        value: f.presence("value", string)?,
        additional_fields: f.rest(&["value"]),
    })
}

fn apex(v: &SerializedSourceValue, i: &SourceIdentity, p: &str) -> ParseResult<SourceApex> {
    let f = Fields::new(v, i, p)?;
    Ok(SourceApex {
        attribute: f.presence("attribute", |v, i, p| match string(v, i, p)?.as_str() {
            "str" => Ok(ApexAttributeSource::Strength),
            "dex" => Ok(ApexAttributeSource::Dexterity),
            "con" => Ok(ApexAttributeSource::Constitution),
            "int" => Ok(ApexAttributeSource::Intelligence),
            "wis" => Ok(ApexAttributeSource::Wisdom),
            "cha" => Ok(ApexAttributeSource::Charisma),
            _ => Err(shape_error(v, i, p, "str | dex | con | int | wis | cha")),
        })?,
        selected: f.presence("selected", boolean)?,
        additional_fields: f.rest(&["attribute", "selected"]),
    })
}

fn activation(
    v: &SerializedSourceValue,
    i: &SourceIdentity,
    p: &str,
) -> ParseResult<ItemActivationSource> {
    let f = Fields::new(v, i, p)?;
    Ok(ItemActivationSource {
        id: f.presence("id", string)?,
        description: f.presence("description", text_value)?,
        action_cost: f.presence("actionCost", action_cost)?,
        components: f.presence("components", components)?,
        frequency: f.presence("frequency", frequency)?,
        traits: f.presence("traits", activation_traits)?,
        additional_fields: f.rest(&[
            "id",
            "description",
            "actionCost",
            "components",
            "frequency",
            "traits",
        ]),
    })
}

fn action_cost(
    v: &SerializedSourceValue,
    i: &SourceIdentity,
    p: &str,
) -> ParseResult<SourceActionCost> {
    let f = Fields::new(v, i, p)?;
    Ok(SourceActionCost {
        action_type: f.presence("type", |v, i, p| match string(v, i, p)?.as_str() {
            "action" => Ok(SourceActionType::Action),
            "reaction" => Ok(SourceActionType::Reaction),
            "free" => Ok(SourceActionType::Free),
            _ => Err(shape_error(v, i, p, "action | reaction | free")),
        })?,
        value: f.presence("value", |v, i, p| bounded_count(v, i, p, 1, 3))?,
        additional_fields: f.rest(&["type", "value"]),
    })
}

fn bounded_count(
    v: &SerializedSourceValue,
    i: &SourceIdentity,
    p: &str,
    min: u8,
    max: u8,
) -> ParseResult<u8> {
    let n = number(v, i, p)?.as_f64();
    match n {
        Some(n) if n.fract() == 0.0 && n >= f64::from(min) && n <= f64::from(max) => Ok(n as u8),
        _ => Err(shape_error(v, i, p, &format!("integer {min}..={max}"))),
    }
}

fn components(
    v: &SerializedSourceValue,
    i: &SourceIdentity,
    p: &str,
) -> ParseResult<ItemActivationComponentsSource> {
    let f = Fields::new(v, i, p)?;
    Ok(ItemActivationComponentsSource {
        command: f.presence("command", boolean)?,
        envision: f.presence("envision", boolean)?,
        interact: f.presence("interact", boolean)?,
        cast: f.presence("cast", boolean)?,
        additional_fields: f.rest(&["command", "envision", "interact", "cast"]),
    })
}

fn frequency(
    v: &SerializedSourceValue,
    i: &SourceIdentity,
    p: &str,
) -> ParseResult<ItemFrequencySource> {
    let f = Fields::new(v, i, p)?;
    Ok(ItemFrequencySource {
        value: f.presence("value", number)?,
        max: f.presence("max", number)?,
        per: f.presence("per", |v, i, p| {
            CreatureFrequencyPeriod::from_source_token(&string(v, i, p)?)
                .ok_or_else(|| shape_error(v, i, p, "pinned frequency interval"))
        })?,
        additional_fields: f.rest(&["value", "max", "per"]),
    })
}

fn activation_traits(
    v: &SerializedSourceValue,
    i: &SourceIdentity,
    p: &str,
) -> ParseResult<ItemActivationTraitsSource> {
    let f = Fields::new(v, i, p)?;
    Ok(ItemActivationTraitsSource {
        value: f.presence("value", strings)?,
        additional_fields: f.rest(&["value"]),
    })
}
