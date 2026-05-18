use atlas_discovery::{
    DiscoveryFieldDefinition, all_discovery_field_definitions, discovery_field_definition,
    metric_filter_field_info,
};
use atlas_domain::{FilterFieldInfo, FilterValuePolicy, FilterValueSort};

use super::error::DiscoveryError;

pub(super) type FieldDefinition = DiscoveryFieldDefinition;

pub(super) fn definition_for(field: &str) -> Option<FieldDefinition> {
    discovery_field_definition(field)
}

pub(super) fn all_definitions() -> &'static [FieldDefinition] {
    all_discovery_field_definitions()
}

pub(super) fn metric_field_info(catalog_available: bool) -> FilterFieldInfo {
    metric_filter_field_info(catalog_available)
}

pub(super) fn validate_options(
    definition: FieldDefinition,
    sort: Option<FilterValueSort>,
    sample_limit: Option<usize>,
) -> Result<(), DiscoveryError> {
    if sort.is_some() && definition.value_policy != FilterValuePolicy::Enumerable {
        return Err(DiscoveryError::InvalidOption(
            "--sort applies only to enumerable value fields".to_string(),
        ));
    }
    if sample_limit.is_some() && definition.value_policy != FilterValuePolicy::Sample {
        return Err(DiscoveryError::InvalidOption(
            "--sample-limit applies only to sampled text fields".to_string(),
        ));
    }
    Ok(())
}

pub(super) fn unknown_field_error(field: &str) -> DiscoveryError {
    let suggestion = match field {
        "packs" | "pack" => " Did you mean `pack_name` or `pack_label`?",
        "sources" | "source" => " Did you mean `publication_title`?",
        "actorMetrics" | "itemMetrics" | "actor_metrics" | "item_metrics" => {
            " Did you mean `metric`?"
        }
        _ => "",
    };
    DiscoveryError::InvalidField(format!(
        "unknown filter field `{field}`.{suggestion} Run `atlas filters fields` to discover supported fields."
    ))
}
