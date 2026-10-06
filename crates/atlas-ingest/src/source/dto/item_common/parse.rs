use atlas_domain::Rarity;

use super::super::{
    ItemType, SourceIdentity, SourceVersionMetadata, parse_serialized_source_object,
    validate_pinned_source_version,
};
use super::*;

use super::super::fields::*;

/// Parse common Item facts from bytes, before defaults or product conversion.
///
/// Both standalone and actor-embedded documents use this parser. Parent context
/// labels the occurrence and diagnostic path; actor-specific admission rules
/// and family-specific system parsing are separate later responsibilities.
/// Duplicate scalar/structural members are rejected. Ordered keyed collections
/// and pending/extension payloads retain repeated members without choosing one.
pub fn parse_common_item_source(
    version: SourceVersionMetadata,
    identity: SourceIdentity,
    parent: Option<ItemParentSource>,
    bytes: &[u8],
) -> ParseResult<VersionedCommonItemSource> {
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
        .map_err(|error| malformed(&identity, &path, "Item source JSON object", error))?;
    let source = parse_common_item_fields(&serialized, &identity, &path)?;
    Ok(VersionedCommonItemSource {
        version,
        identity,
        parent,
        source,
        serialized,
    })
}

pub(in crate::source::dto) fn parse_common_item_fields(
    serialized: &SerializedSourceObject,
    identity: &SourceIdentity,
    path: &str,
) -> ParseResult<CommonItemSource> {
    let fields = Fields {
        object: serialized,
        identity,
        path,
    };
    let id = fields.presence("_id", string)?;
    let name = fields.required("name", string)?;
    let item_type = fields.required("type", |value, identity, path| {
        ItemType::parse(&string(value, identity, path)?, identity, path)
    })?;
    Ok(CommonItemSource {
        envelope: ItemSourceEnvelope {
            id,
            name,
            item_type,
            image: fields.presence("img", string)?,
            folder: fields.presence("folder", string)?,
            sort: fields.presence("sort", integer)?,
            ownership: fields.presence("ownership", |v, i, p| keyed(v, i, p, ownership))?,
            flags: fields.presence("flags", flags)?,
            stats: fields.presence("_stats", stats)?,
            effects: fields.presence("effects", objects)?,
            additional_fields: fields.rest(&[
                "_id",
                "name",
                "type",
                "img",
                "system",
                "effects",
                "folder",
                "sort",
                "ownership",
                "flags",
                "_stats",
            ]),
        },
        system: fields.required("system", system)?,
    })
}

fn system(v: &SerializedSourceValue, i: &SourceIdentity, p: &str) -> ParseResult<ItemSystemSource> {
    let f = Fields::new(v, i, p)?;
    Ok(ItemSystemSource {
        level: f.presence("level", number_value)?,
        description: f.presence("description", description)?,
        traits: f.presence("traits", traits)?,
        rules: f.presence("rules", objects)?,
        slug: f.presence("slug", string)?,
        publication: f.presence("publication", publication)?,
        migration: f.presence("_migration", migration)?,
        legacy_schema: f.presence("schema", structured_value)?,
        pending_family_fields: f.rest(&[
            "level",
            "description",
            "traits",
            "rules",
            "slug",
            "publication",
            "_migration",
            "schema",
        ]),
    })
}

pub(in crate::source::dto) fn number_value(
    v: &SerializedSourceValue,
    i: &SourceIdentity,
    p: &str,
) -> ParseResult<ItemNumberValueSource> {
    let f = Fields::new(v, i, p)?;
    Ok(ItemNumberValueSource {
        value: f.presence("value", number)?,
        additional_fields: f.rest(&["value"]),
    })
}

fn description(
    v: &SerializedSourceValue,
    i: &SourceIdentity,
    p: &str,
) -> ParseResult<ItemDescriptionSource> {
    let f = Fields::new(v, i, p)?;
    Ok(ItemDescriptionSource {
        gm: f.presence("gm", string)?,
        value: f.presence("value", string)?,
        additional_fields: f.rest(&["gm", "value"]),
    })
}

fn traits(v: &SerializedSourceValue, i: &SourceIdentity, p: &str) -> ParseResult<ItemTraitsSource> {
    let f = Fields::new(v, i, p)?;
    Ok(ItemTraitsSource {
        value: f.presence("value", strings)?,
        rarity: f.presence("rarity", |v, i, p| {
            Rarity::from_canonical(&string(v, i, p)?)
                .ok_or_else(|| shape_error(v, i, p, "common | uncommon | rare | unique"))
        })?,
        other_tags: f.presence("otherTags", strings)?,
        additional_fields: f.rest(&["value", "rarity", "otherTags"]),
    })
}

fn publication(
    v: &SerializedSourceValue,
    i: &SourceIdentity,
    p: &str,
) -> ParseResult<ItemPublicationSource> {
    let f = Fields::new(v, i, p)?;
    Ok(ItemPublicationSource {
        title: f.presence("title", string)?,
        authors: f.presence("authors", string)?,
        license: f.presence("license", |v, i, p| match string(v, i, p)?.as_str() {
            "ORC" => Ok(ItemLicenseSource::Orc),
            "OGL" => Ok(ItemLicenseSource::Ogl),
            _ => Err(shape_error(v, i, p, "ORC | OGL")),
        })?,
        remaster: f.presence("remaster", boolean)?,
        additional_fields: f.rest(&["title", "authors", "license", "remaster"]),
    })
}

fn migration(
    v: &SerializedSourceValue,
    i: &SourceIdentity,
    p: &str,
) -> ParseResult<ItemMigrationSource> {
    let f = Fields::new(v, i, p)?;
    Ok(ItemMigrationSource {
        version: f.presence("version", number)?,
        previous: f.presence("previous", migration_previous)?,
        additional_fields: f.rest(&["version", "previous"]),
    })
}

fn migration_previous(
    v: &SerializedSourceValue,
    i: &SourceIdentity,
    p: &str,
) -> ParseResult<ItemMigrationPreviousSource> {
    let f = Fields::new(v, i, p)?;
    Ok(ItemMigrationPreviousSource {
        foundry: f.presence("foundry", string)?,
        system: f.presence("system", string)?,
        schema: f.presence("schema", number)?,
        additional_fields: f.rest(&["foundry", "system", "schema"]),
    })
}

fn stats(v: &SerializedSourceValue, i: &SourceIdentity, p: &str) -> ParseResult<ItemStatsSource> {
    let f = Fields::new(v, i, p)?;
    Ok(ItemStatsSource {
        system_id: f.presence("systemId", string)?,
        system_version: f.presence("systemVersion", string)?,
        core_version: f.presence("coreVersion", string)?,
        created_time: f.presence("createdTime", number)?,
        modified_time: f.presence("modifiedTime", number)?,
        last_modified_by: f.presence("lastModifiedBy", string)?,
        compendium_source: f.presence("compendiumSource", string)?,
        duplicate_source: f.presence("duplicateSource", string)?,
        additional_fields: f.rest(&[
            "systemId",
            "systemVersion",
            "coreVersion",
            "createdTime",
            "modifiedTime",
            "lastModifiedBy",
            "compendiumSource",
            "duplicateSource",
        ]),
    })
}

fn flags(v: &SerializedSourceValue, i: &SourceIdentity, p: &str) -> ParseResult<ItemFlagsSource> {
    let f = Fields::new(v, i, p)?;
    Ok(ItemFlagsSource {
        pf2e: f.presence("pf2e", pf2e_flags)?,
        extensions: f.rest(&["pf2e"]),
    })
}

fn pf2e_flags(
    v: &SerializedSourceValue,
    i: &SourceIdentity,
    p: &str,
) -> ParseResult<Pf2eItemFlagsSource> {
    let f = Fields::new(v, i, p)?;
    Ok(Pf2eItemFlagsSource {
        rules_selections: f.presence("rulesSelections", |v, i, p| keyed(v, i, p, selection))?,
        item_grants: f.presence("itemGrants", |v, i, p| keyed(v, i, p, granter))?,
        granted_by: f.presence("grantedBy", grant)?,
        extensions: f.rest(&["rulesSelections", "itemGrants", "grantedBy"]),
    })
}

fn grant_fields(f: &Fields<'_>, known: &[&str]) -> ParseResult<ItemGrantSource> {
    Ok(ItemGrantSource {
        id: f.required("id", string)?,
        on_delete: f.presence("onDelete", |v, i, p| match string(v, i, p)?.as_str() {
            "cascade" => Ok(ItemGrantDeleteAction::Cascade),
            "detach" => Ok(ItemGrantDeleteAction::Detach),
            "restrict" => Ok(ItemGrantDeleteAction::Restrict),
            _ => Err(shape_error(v, i, p, "cascade | detach | restrict")),
        })?,
        additional_fields: f.rest(known),
    })
}

fn grant(v: &SerializedSourceValue, i: &SourceIdentity, p: &str) -> ParseResult<ItemGrantSource> {
    grant_fields(&Fields::new(v, i, p)?, &["id", "onDelete"])
}

fn granter(
    v: &SerializedSourceValue,
    i: &SourceIdentity,
    p: &str,
) -> ParseResult<ItemGranterSource> {
    let f = Fields::new(v, i, p)?;
    Ok(ItemGranterSource {
        grant: grant_fields(&f, &["id", "onDelete", "nested"])?,
        nested: f.presence("nested", boolean)?,
    })
}

fn selection(
    v: &SerializedSourceValue,
    i: &SourceIdentity,
    p: &str,
) -> ParseResult<ItemRuleSelectionSource> {
    match v {
        SerializedSourceValue::String(v) => Ok(ItemRuleSelectionSource::String(v.clone())),
        SerializedSourceValue::Number(v) => Ok(ItemRuleSelectionSource::Number(v.clone())),
        SerializedSourceValue::Object(v) => Ok(ItemRuleSelectionSource::Object(v.clone())),
        SerializedSourceValue::Array(v) => Ok(ItemRuleSelectionSource::Array(v.clone())),
        _ => Err(shape_error(v, i, p, "string | number | object | array")),
    }
}

fn ownership(
    v: &SerializedSourceValue,
    i: &SourceIdentity,
    p: &str,
) -> ParseResult<ItemOwnershipLevel> {
    match integer(v, i, p)? {
        -1 => Ok(ItemOwnershipLevel::Inherit),
        0 => Ok(ItemOwnershipLevel::None),
        1 => Ok(ItemOwnershipLevel::Limited),
        2 => Ok(ItemOwnershipLevel::Observer),
        3 => Ok(ItemOwnershipLevel::Owner),
        _ => Err(shape_error(v, i, p, "ownership level -1 | 0 | 1 | 2 | 3")),
    }
}
