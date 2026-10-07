use serde::Serialize;

use super::generated;
use super::parse::{Fields, ParseResult, string};
use super::{
    ItemDescriptionSource, ItemGrantFields, ItemTraits, PublicationData, SourceContext,
    SourceDiagnostic, SourceObject, SourcePresence, SourceValue,
};

/// Common Item source fields before defaults; all unselected fields remain ordered data.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ItemSourceSlice {
    pub family: String,
    pub description: SourcePresence<ItemDescriptionSource>,
    pub publication: SourcePresence<PublicationData>,
    pub traits: SourcePresence<ItemTraits>,
    pub flags: SourcePresence<ItemGrantSlice>,
    pub system_fields: SourceObject,
    pub envelope_fields: SourceObject,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ItemGrantSlice {
    pub pf2e: SourcePresence<ItemGrantFields>,
    pub additional_fields: SourceObject,
}

fn grant_flags(
    value: &SourceValue,
    context: &SourceContext,
    path: &str,
) -> ParseResult<ItemGrantSlice> {
    let fields = Fields::new(value, context, path)?;
    Ok(ItemGrantSlice {
        pf2e: fields.presence("pf2e", generated::parse_item_grant_fields)?,
        additional_fields: fields.remaining(&["pf2e"]),
    })
}

pub fn parse_item_source_slice(
    context: SourceContext,
    bytes: &[u8],
) -> Result<ItemSourceSlice, SourceDiagnostic> {
    let source = super::value::parse_source(bytes).map_err(|error| {
        context.message(
            &context.json_path,
            "complete JSON object",
            error.to_string(),
        )
    })?;
    let envelope = Fields::new(&source, &context, &context.json_path)?;
    let family = envelope.required("type", string)?;
    if !generated::ITEM_FAMILIES.contains(&family.as_str()) {
        return Err(context.message(
            &format!("{}.type", context.json_path),
            "registered Item family",
            &family,
        ));
    }
    let system_path = format!("{}.system", context.json_path);
    let system = envelope.required("system", |value, c, p| {
        Fields::new(value, c, p)?;
        Ok(value.clone())
    })?;
    let fields = Fields::new(&system, &context, &system_path)?;
    Ok(ItemSourceSlice {
        description: fields.presence("description", generated::parse_item_description_source)?,
        publication: fields.presence("publication", generated::parse_publication_data)?,
        traits: fields.presence("traits", |v, c, p| {
            generated::parse_item_traits(&family, v, c, p)
        })?,
        flags: envelope.presence("flags", grant_flags)?,
        system_fields: fields.remaining(&["description", "publication", "traits"]),
        envelope_fields: envelope.remaining(&["type", "system", "flags"]),
        family,
    })
}
