//! Exact CLI selections bind the shared navigation address to the read generation.
use atlas_app_model::{RecordDetailRequest, RecordNavigationView};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Locator {
    pub artifact_generation: String,
    #[serde(flatten)]
    pub address: RecordNavigationView,
}
impl Locator {
    pub(crate) fn new(generation: &str, address: &RecordNavigationView) -> Self {
        Self {
            artifact_generation: generation.into(),
            address: address.clone(),
        }
    }

    /// Optional members omitted here deserialize to the same navigation address.
    pub(crate) fn compact(&self) -> serde_json::Value {
        let mut value = serde_json::json!(self);
        if let Some(fields) = value.as_object_mut() {
            fields.retain(|_, member| !member.is_null());
        }
        value
    }

    pub(super) fn parse(text: &str) -> Result<Self, String> {
        let locator: Self = serde_json::from_str(text).map_err(|error| error.to_string())?;
        if locator.artifact_generation.is_empty() {
            return Err("Navigation requires its artifact generation.".into());
        }
        if locator
            .address
            .passage
            .as_ref()
            .is_some_and(|p| !matches!(p, atlas_domain::SourcePassageAddress::Identity {}))
            && locator.address.field.is_none()
        {
            return Err("Passage navigation requires a field.".into());
        }
        if locator
            .address
            .spell_rank
            .is_some_and(|rank| !(1..=10).contains(&rank))
        {
            return Err("Spell rank must be between 1 and 10.".into());
        }
        Ok(locator)
    }

    pub(super) fn request(&self) -> RecordDetailRequest {
        RecordDetailRequest {
            include_section_rendering: false,
            expected_artifact_generation: Some(self.artifact_generation.clone()),
            spell_rank: self.address.spell_rank,
            record_key: self.address.record_key.clone(),
            owners: self.address.owners.clone(),
            fields: self.address.field.clone().into_iter().collect(),
            passage: self.address.passage.clone(),
            source_fingerprint: self.address.source_fingerprint.clone(),
        }
    }
}

pub(crate) fn root(key: &str) -> RecordNavigationView {
    RecordNavigationView {
        record_key: key.into(),
        owners: vec![],
        field: None,
        passage: None,
        source_fingerprint: None,
        spell_rank: None,
    }
}

/// Full reference projections retain nested app-model DTOs such as casting entries,
/// table rows and relationship destinations. Bind their shared RecordNavigationView
/// wire shape to the same read snapshot as the enclosing CLI response.
pub(crate) fn bind_nested(value: &mut serde_json::Value, generation: &str) {
    use serde_json::Value;
    if let Some(fields) = value.as_object()
        && fields.contains_key("field")
        && fields.contains_key("passage")
        && fields.contains_key("source_fingerprint")
        && fields.len() == 5 + usize::from(fields.contains_key("spell_rank"))
        && RecordNavigationView::deserialize(&*value).is_ok()
    {
        value["artifact_generation"] = Value::String(generation.to_owned());
        return;
    }
    match value {
        Value::Array(rows) => {
            for row in rows {
                bind_nested(row, generation);
            }
        }
        Value::Object(fields) => {
            for member in fields.values_mut() {
                bind_nested(member, generation);
            }
        }
        _ => {}
    }
}
