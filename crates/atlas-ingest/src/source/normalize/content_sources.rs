use atlas_record::{
    ContentIdentityStability, ContentSourceKind, RecordContentDocument, RichDocument,
};
use serde_json::Value;

use crate::records::SourceContentFact;
use crate::source::hazard_entities::HazardResolvedItemIdentity;

use super::{
    ContentParseDiagnostics, LocalizationResolver, parse_foundry_content_with_localization,
    pointer_string, string_field,
};

pub(super) struct SourceContentProjection {
    pub description: Option<RichDocument>,
    pub blurb: Option<RichDocument>,
    pub supplemental_content: Vec<(Option<String>, RecordContentDocument)>,
    pub owned_content: Vec<SourceContentFact>,
    pub diagnostics: Vec<ContentParseDiagnostics>,
}

struct ContentAccumulator {
    content: Vec<(Option<String>, RecordContentDocument)>,
    owned_content: Vec<SourceContentFact>,
    diagnostics: Vec<ContentParseDiagnostics>,
}

type SupplementalContentExtraction = (
    Vec<(Option<String>, RecordContentDocument)>,
    Vec<SourceContentFact>,
    Vec<ContentParseDiagnostics>,
);

struct EmbeddedContentPointer {
    pointer: &'static str,
    source_kind: ContentSourceKind,
    label: Option<String>,
    local_key: String,
    nested_source_id: String,
    authored_ordinal: usize,
    identity_stability: Option<ContentIdentityStability>,
}

pub(super) fn extract_content_sources(
    raw: &Value,
    localization: Option<&dyn LocalizationResolver>,
    hazard_identities: Option<&[HazardResolvedItemIdentity]>,
) -> SourceContentProjection {
    let source_description_raw = pointer_string(raw, "/system/description/value");
    let parsed_description = source_description_raw
        .as_deref()
        .map(|markup| parse_foundry_content_with_localization(markup, localization));
    let description = parsed_description
        .as_ref()
        .map(|parsed| parsed.document.clone())
        .filter(non_empty_document);

    let source_blurb_markup = pointer_string(raw, "/system/details/blurb");
    let parsed_blurb = source_blurb_markup
        .as_deref()
        .map(|markup| parse_foundry_content_with_localization(markup, localization));
    let blurb = parsed_blurb
        .as_ref()
        .map(|parsed| parsed.document.clone())
        .filter(non_empty_document);

    let (supplemental_content, mut owned_content, supplemental_diagnostics) =
        extract_supplemental_content(
            raw,
            source_description_raw.as_deref(),
            localization,
            hazard_identities,
        );
    if let (Some(document), Some(parsed)) = (&description, &parsed_description) {
        owned_content.insert(
            0,
            source_content_fact(
                "description".to_string(),
                ContentIdentityStability::StableSourceIdentity,
                ContentSourceKind::Description,
                "$.system.description.value",
                None,
                None,
                0,
                None,
                document.clone(),
                parsed.diagnostics.clone(),
            ),
        );
    }
    if let (Some(document), Some(parsed)) = (&blurb, &parsed_blurb) {
        let position = usize::from(!owned_content.is_empty());
        owned_content.insert(
            position,
            source_content_fact(
                "blurb".to_string(),
                ContentIdentityStability::StableSourceIdentity,
                ContentSourceKind::Blurb,
                "$.system.details.blurb",
                None,
                None,
                position as u32,
                None,
                document.clone(),
                parsed.diagnostics.clone(),
            ),
        );
    }
    for (order, content) in owned_content.iter_mut().enumerate() {
        content.authored_order = order as u32;
    }

    SourceContentProjection {
        description,
        blurb,
        supplemental_content,
        owned_content,
        diagnostics: parsed_description
            .into_iter()
            .chain(parsed_blurb)
            .map(|parsed| parsed.diagnostics)
            .chain(supplemental_diagnostics)
            .collect(),
    }
}

fn non_empty_document(document: &RichDocument) -> bool {
    !document.is_empty()
}

fn extract_supplemental_content(
    raw: &Value,
    source_description_raw: Option<&str>,
    localization: Option<&dyn LocalizationResolver>,
    hazard_identities: Option<&[HazardResolvedItemIdentity]>,
) -> SupplementalContentExtraction {
    let mut accumulator = ContentAccumulator {
        content: Vec::new(),
        owned_content: Vec::new(),
        diagnostics: Vec::new(),
    };
    collect_content_at_pointer(
        raw,
        "/system/details/disable",
        "disable",
        ContentSourceKind::Disable,
        None,
        localization,
        &mut accumulator,
    );
    collect_content_at_pointer(
        raw,
        "/system/details/routine",
        "routine",
        ContentSourceKind::Routine,
        None,
        localization,
        &mut accumulator,
    );
    collect_content_at_pointer(
        raw,
        "/system/details/reset",
        "reset",
        ContentSourceKind::Reset,
        None,
        localization,
        &mut accumulator,
    );
    collect_content_at_pointer(
        raw,
        "/system/attributes/stealth/details",
        "stealth-details",
        ContentSourceKind::StealthDetails,
        Some("Stealth".to_string()),
        localization,
        &mut accumulator,
    );
    if pointer_string(raw, "/system/details/description").as_deref() != source_description_raw {
        collect_content_at_pointer(
            raw,
            "/system/details/description",
            "details-description",
            ContentSourceKind::DetailsFieldDescription,
            None,
            localization,
            &mut accumulator,
        );
    }
    collect_content_at_pointer(
        raw,
        "/system/details/publicNotes",
        "public-notes",
        ContentSourceKind::PublicNotes,
        Some("Public Notes".to_string()),
        localization,
        &mut accumulator,
    );
    collect_content_at_pointer(
        raw,
        "/system/description/gm",
        "gm-notes:description",
        ContentSourceKind::GmNotes,
        Some("GM Notes".to_string()),
        localization,
        &mut accumulator,
    );
    collect_content_at_pointer(
        raw,
        "/system/details/gmNotes",
        "gm-notes:details",
        ContentSourceKind::GmNotes,
        Some("GM Notes".to_string()),
        localization,
        &mut accumulator,
    );
    collect_content_at_pointer(
        raw,
        "/system/details/privateNotes",
        "private-notes",
        ContentSourceKind::PrivateNotes,
        Some("Private Notes".to_string()),
        localization,
        &mut accumulator,
    );
    collect_embedded_item_content(raw, localization, hazard_identities, &mut accumulator);
    collect_consumable_spell_content(raw, localization, &mut accumulator);
    (
        accumulator.content,
        accumulator.owned_content,
        accumulator.diagnostics,
    )
}

fn collect_consumable_spell_content(
    raw: &Value,
    localization: Option<&dyn LocalizationResolver>,
    accumulator: &mut ContentAccumulator,
) {
    if string_field(raw, "type").as_deref() != Some("consumable") {
        return;
    }
    let Some(spell) = raw.pointer("/system/spell") else {
        return;
    };
    let Some(child_id) = string_field(spell, "_id") else {
        return;
    };
    collect_embedded_content_at_pointer(
        raw,
        EmbeddedContentPointer {
            pointer: "/system/spell/system/description/value",
            source_kind: ContentSourceKind::EmbeddedSpellDescription,
            label: string_field(spell, "name"),
            local_key: embedded_item_content_key(&child_id, "spell-description"),
            nested_source_id: child_id,
            authored_ordinal: 0,
            identity_stability: None,
        },
        localization,
        accumulator,
    );
}

fn collect_content_at_pointer(
    raw: &Value,
    pointer: &str,
    content_key: &str,
    source_kind: ContentSourceKind,
    label: Option<String>,
    localization: Option<&dyn LocalizationResolver>,
    accumulator: &mut ContentAccumulator,
) {
    let Some(markup) = pointer_string(raw, pointer) else {
        return;
    };
    let parsed = parse_foundry_content_with_localization(&markup, localization);
    if parsed.document.is_empty() {
        return;
    }
    accumulator.diagnostics.push(parsed.diagnostics.clone());
    let order = accumulator.owned_content.len() as u32;
    accumulator.owned_content.push(source_content_fact(
        content_key.to_string(),
        ContentIdentityStability::StableSourceIdentity,
        source_kind,
        &json_path(pointer),
        None,
        None,
        order,
        label.clone(),
        parsed.document.clone(),
        parsed.diagnostics.clone(),
    ));
    accumulator.content.push((
        None,
        supplemental_content(source_kind, label, parsed.document),
    ));
}

fn collect_embedded_item_content(
    raw: &Value,
    localization: Option<&dyn LocalizationResolver>,
    hazard_identities: Option<&[HazardResolvedItemIdentity]>,
    accumulator: &mut ContentAccumulator,
) {
    let Some(items) = raw.pointer("/items").and_then(Value::as_array) else {
        return;
    };
    for (index, item) in items.iter().enumerate() {
        let label = string_field(item, "name");
        let hazard_identity = hazard_identities.and_then(|identities| identities.get(index));
        let item_id = hazard_identity
            .map(|identity| identity.occurrence_id.as_str().to_string())
            .unwrap_or_else(|| embedded_item_id(item, index));
        let identity_stability = hazard_identity.map(|identity| match identity.stability {
            atlas_record::HazardOccurrenceIdentityStability::StableSourceIdentity => {
                ContentIdentityStability::StableSourceIdentity
            }
            atlas_record::HazardOccurrenceIdentityStability::UnstableAuthoredOrdinal => {
                ContentIdentityStability::UnstableAuthoredOrdinal
            }
        });
        collect_embedded_content_at_pointer(
            item,
            EmbeddedContentPointer {
                pointer: "/system/description/value",
                source_kind: ContentSourceKind::EmbeddedItemDescription,
                label: label.clone(),
                local_key: embedded_item_content_key(&item_id, "description"),
                nested_source_id: item_id.clone(),
                authored_ordinal: index,
                identity_stability,
            },
            localization,
            accumulator,
        );
        collect_embedded_content_at_pointer(
            item,
            EmbeddedContentPointer {
                pointer: "/system/description/gm",
                source_kind: ContentSourceKind::EmbeddedGmDescription,
                label: label.clone(),
                local_key: embedded_item_content_key(&item_id, "gm-description"),
                nested_source_id: item_id.clone(),
                authored_ordinal: index,
                identity_stability,
            },
            localization,
            accumulator,
        );
        collect_embedded_content_at_pointer(
            item,
            EmbeddedContentPointer {
                pointer: "/system/spell/system/description/value",
                source_kind: ContentSourceKind::EmbeddedSpellDescription,
                label,
                local_key: embedded_item_content_key(&item_id, "spell-description"),
                nested_source_id: item_id,
                authored_ordinal: index,
                identity_stability,
            },
            localization,
            accumulator,
        );
    }
}

fn collect_embedded_content_at_pointer(
    raw: &Value,
    source: EmbeddedContentPointer,
    localization: Option<&dyn LocalizationResolver>,
    accumulator: &mut ContentAccumulator,
) {
    let Some(markup) = pointer_string(raw, source.pointer) else {
        return;
    };
    let parsed = parse_foundry_content_with_localization(&markup, localization);
    if parsed.document.is_empty() {
        return;
    }
    accumulator.diagnostics.push(parsed.diagnostics.clone());
    let identity_stability = source.identity_stability.unwrap_or_else(|| {
        raw.get("_id")
            .and_then(Value::as_str)
            .filter(|value| !value.trim().is_empty())
            .map(|_| ContentIdentityStability::StableSourceIdentity)
            .unwrap_or(ContentIdentityStability::UnstableAuthoredOrdinal)
    });
    let order = accumulator.owned_content.len() as u32;
    accumulator.owned_content.push(source_content_fact(
        source.local_key.clone(),
        identity_stability,
        source.source_kind,
        &format!(
            "$.items[_id={}].{}",
            source.nested_source_id,
            json_path(source.pointer).trim_start_matches("$.")
        ),
        Some(source.nested_source_id),
        Some(source.authored_ordinal.to_string()),
        order,
        source.label.clone(),
        parsed.document.clone(),
        parsed.diagnostics.clone(),
    ));
    accumulator.content.push((
        Some(source.local_key),
        supplemental_content(source.source_kind, source.label, parsed.document),
    ));
}

fn supplemental_content(
    source_kind: ContentSourceKind,
    label: Option<String>,
    document: RichDocument,
) -> RecordContentDocument {
    RecordContentDocument {
        source_kind,
        label,
        document,
    }
}

pub(super) fn embedded_item_content_key(item_id: &str, suffix: &str) -> String {
    format!("item:{item_id}:{suffix}")
}

pub(super) fn embedded_item_id(item: &Value, index: usize) -> String {
    string_field(item, "_id").unwrap_or_else(|| format!("item-{index}"))
}

#[allow(clippy::too_many_arguments)]
fn source_content_fact(
    content_key: String,
    identity_stability: ContentIdentityStability,
    source_kind: ContentSourceKind,
    relative_source_path: &str,
    nested_source_id: Option<String>,
    authored_ordinal_or_range: Option<String>,
    authored_order: u32,
    label: Option<String>,
    document: RichDocument,
    diagnostics: ContentParseDiagnostics,
) -> SourceContentFact {
    SourceContentFact {
        content_key,
        identity_stability,
        source_kind,
        relative_source_path: relative_source_path.to_string(),
        nested_source_id,
        authored_ordinal_or_range,
        authored_order,
        label,
        document,
        diagnostics,
    }
}

fn json_path(pointer: &str) -> String {
    format!("$.{}", pointer.trim_start_matches('/').replace('/', "."))
}
