//! Readable source-backed action metadata; no actor action execution.
use super::{ContentAudience, ContentVisibilityRule, LocalizationResolver};

pub(super) struct InlineAction {
    pub slug: String,
    pub name: String,
    pub label: String,
    pub details: String,
    pub cost: Option<&'static str>,
}

pub(super) fn inline_action(
    arguments: &str,
    options: &std::collections::BTreeMap<String, String>,
    localization: Option<&dyn LocalizationResolver>,
    audience: ContentAudience,
) -> Option<InlineAction> {
    let slug = arguments.split_whitespace().next()?;
    let definition = atlas_foundry_model::generated::action_definition(slug)?;
    let localize = |key: &str| localization?.localized_value(key).map(str::to_owned);
    let name = localize(definition.name_key)?;
    let mut label = name.clone();
    if let Some(slug) = options.get("variant") {
        let variant = definition
            .variants
            .iter()
            .find(|variant| variant.slug == slug)?;
        if let Some(key) = variant.name_key {
            label.push_str(&format!(" - {}", localize(key)?));
        }
    }
    let statistic = options
        .get("statistic")
        .or_else(|| options.get("stat"))
        .or_else(|| options.get("skill"));
    let dc = options
        .get("difficulty-class")
        .or_else(|| options.get("dc"));
    let show_dc = options
        .get("show-dc")
        .map_or(audience.implicit_check_dc, |value| {
            ContentVisibilityRule::from_source(value).unwrap_or(ContentVisibilityRule::None)
        });
    let mut details = Vec::new();
    if audience.permits(show_dc)
        && let Some(dc) = dc
    {
        details.push(format!("DC {dc}"));
    }
    if let Some(statistic) = statistic {
        details.push(statistic.clone());
    }
    let details = if details.is_empty() {
        String::new()
    } else {
        format!(" ({})", details.join(" "))
    };
    Some(InlineAction {
        slug: slug.into(),
        name,
        label,
        details,
        cost: definition.cost,
    })
}
