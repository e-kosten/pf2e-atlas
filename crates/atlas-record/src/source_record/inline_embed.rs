//! Pinned Item embed selection. This does not resolve targets or execute rules.
use super::{SourceContentRole, SourceContentSelection, SourceNodeView};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy)]
pub struct InlineEmbedOptions {
    pub header: bool,
    pub traits: bool,
    pub publication: bool,
    pub prerequisite_rule: bool,
    pub journal_link: bool,
}
impl InlineEmbedOptions {
    pub fn parse(options: &BTreeMap<String, String>) -> Result<Self, &'static str> {
        if options.get("inline").map(String::as_str) != Some("true") {
            return Err("This content insertion format is unavailable.");
        }
        for (key, value) in options {
            if !matches!(
                key.as_str(),
                "inline" | "header" | "traits" | "publication" | "hr" | "journalLink"
            ) || !matches!(value.as_str(), "true" | "false")
            {
                return Err("This content insertion option is unavailable.");
            }
        }
        let enabled = |key| options.get(key).is_some_and(|v| v == "true");
        Ok(Self {
            header: enabled("header"),
            traits: enabled("traits"),
            publication: enabled("publication"),
            prerequisite_rule: options.get("hr").is_none_or(|v| v != "false"),
            journal_link: options.get("journalLink").is_none_or(|v| v != "false"),
        })
    }
}

/// PF2e Item.description plus feat/campaign prerequisites; not a full stat block.
pub fn inline_embed_fields(node: SourceNodeView<'_>) -> Option<Vec<SourceContentSelection<'_>>> {
    if !matches!(node, SourceNodeView::Item(_)) {
        return None;
    }
    let mut fields: Vec<_> = node
        .content_selections()
        .into_iter()
        .filter(|s| {
            matches!(
                s.role,
                SourceContentRole::Description | SourceContentRole::Prerequisite
            ) && (s.role != SourceContentRole::Description
                || s.field == "/system/description/value")
        })
        .collect();
    fields.sort_by_key(|s| s.role != SourceContentRole::Prerequisite);
    Some(fields)
}
