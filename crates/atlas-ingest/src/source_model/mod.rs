//! Independently callable source slices. No production pipeline adoption.
mod generated;
mod parse;
mod presence;
mod value;

pub use generated::{
    Coins, EquipmentFields, EquippedData, EquippedDataCarryType, PartialPrice,
    PhysicalEquipmentFields, PhysicalEquipmentFieldsUsage, PhysicalItemHPSource,
};
pub use parse::{SourceContext, SourceDiagnostic};
pub use presence::SourcePresence;
pub use value::{SourceObject, SourceValue};

use parse::{Fields, string};
use serde::Serialize;

/// Only equipped, hp, price and usage are typed in this bounded equipment trial.
/// Other system fields and envelope fields remain ordered source values.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EquipmentSourceSlice {
    pub system: EquipmentFields,
    pub envelope_fields: SourceObject,
}

pub fn parse_equipment_source_slice(
    context: SourceContext,
    bytes: &[u8],
) -> Result<EquipmentSourceSlice, SourceDiagnostic> {
    let source = value::parse_source(bytes).map_err(|error| {
        context.message(
            &context.json_path,
            "complete JSON object",
            error.to_string(),
        )
    })?;
    let fields = Fields::new(&source, &context, &context.json_path)?;
    let item_type = fields.required("type", string)?;
    if item_type != "equipment" {
        return Err(context.message(
            &format!("{}.type", context.json_path),
            "equipment",
            item_type,
        ));
    }
    let system = fields.required("system", generated::parse_equipment_fields)?;
    Ok(EquipmentSourceSlice {
        system,
        envelope_fields: fields.remaining(&["type", "system"]),
    })
}

/// Parse the same shared fields from a physical system object, before defaults.
pub fn parse_physical_equipment_fields(
    context: SourceContext,
    bytes: &[u8],
) -> Result<PhysicalEquipmentFields, SourceDiagnostic> {
    let source = value::parse_source(bytes).map_err(|error| {
        context.message(
            &context.json_path,
            "complete JSON object",
            error.to_string(),
        )
    })?;
    generated::parse_physical_equipment_fields(&source, &context, &context.json_path)
}
