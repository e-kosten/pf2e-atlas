use atlas_record::{ContentSourceKind, RecordContentDocument, RichDocument};
use serde_json::Value;

use super::{
    ContentParseDiagnostics, LocalizationResolver, parse_foundry_content_with_localization,
    pointer_string, string_field,
};

pub(super) struct SourceContentProjection {
    pub description: Option<RichDocument>,
    pub blurb: Option<RichDocument>,
    pub supplemental_content: Vec<(Option<String>, RecordContentDocument)>,
    pub diagnostics: Vec<ContentParseDiagnostics>,
}

struct ContentAccumulator {
    content: Vec<(Option<String>, RecordContentDocument)>,
    diagnostics: Vec<ContentParseDiagnostics>,
}

pub(super) fn extract_content_sources(
    raw: &Value,
    localization: Option<&dyn LocalizationResolver>,
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

    let (supplemental_content, supplemental_diagnostics) =
        extract_supplemental_content(raw, source_description_raw.as_deref(), localization);

    SourceContentProjection {
        description,
        blurb,
        supplemental_content,
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
) -> (
    Vec<(Option<String>, RecordContentDocument)>,
    Vec<ContentParseDiagnostics>,
) {
    let mut accumulator = ContentAccumulator {
        content: Vec::new(),
        diagnostics: Vec::new(),
    };
    collect_content_at_pointer(
        raw,
        "/system/details/disable",
        ContentSourceKind::Disable,
        None,
        localization,
        &mut accumulator,
    );
    collect_content_at_pointer(
        raw,
        "/system/details/routine",
        ContentSourceKind::Routine,
        None,
        localization,
        &mut accumulator,
    );
    collect_content_at_pointer(
        raw,
        "/system/details/reset",
        ContentSourceKind::Reset,
        None,
        localization,
        &mut accumulator,
    );
    collect_content_at_pointer(
        raw,
        "/system/attributes/stealth/details",
        ContentSourceKind::StealthDetails,
        Some("Stealth".to_string()),
        localization,
        &mut accumulator,
    );
    if pointer_string(raw, "/system/details/description").as_deref() != source_description_raw {
        collect_content_at_pointer(
            raw,
            "/system/details/description",
            ContentSourceKind::DetailsFieldDescription,
            None,
            localization,
            &mut accumulator,
        );
    }
    collect_content_at_pointer(
        raw,
        "/system/details/publicNotes",
        ContentSourceKind::PublicNotes,
        Some("Public Notes".to_string()),
        localization,
        &mut accumulator,
    );
    collect_content_at_pointer(
        raw,
        "/system/description/gm",
        ContentSourceKind::GmNotes,
        Some("GM Notes".to_string()),
        localization,
        &mut accumulator,
    );
    collect_content_at_pointer(
        raw,
        "/system/details/gmNotes",
        ContentSourceKind::GmNotes,
        Some("GM Notes".to_string()),
        localization,
        &mut accumulator,
    );
    collect_content_at_pointer(
        raw,
        "/system/details/privateNotes",
        ContentSourceKind::PrivateNotes,
        Some("Private Notes".to_string()),
        localization,
        &mut accumulator,
    );
    collect_embedded_item_content(raw, localization, &mut accumulator);
    (accumulator.content, accumulator.diagnostics)
}

fn collect_content_at_pointer(
    raw: &Value,
    pointer: &str,
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
    accumulator.content.push((
        None,
        supplemental_content(source_kind, label, parsed.document),
    ));
}

fn collect_embedded_item_content(
    raw: &Value,
    localization: Option<&dyn LocalizationResolver>,
    accumulator: &mut ContentAccumulator,
) {
    let Some(items) = raw.pointer("/items").and_then(Value::as_array) else {
        return;
    };
    for (index, item) in items.iter().enumerate() {
        let label = string_field(item, "name");
        let item_id = embedded_item_id(item, index);
        collect_embedded_content_at_pointer(
            item,
            "/system/description/value",
            ContentSourceKind::EmbeddedItemDescription,
            label.clone(),
            embedded_item_content_key(&item_id, "description"),
            localization,
            accumulator,
        );
        collect_embedded_content_at_pointer(
            item,
            "/system/spell/system/description/value",
            ContentSourceKind::EmbeddedSpellDescription,
            label,
            embedded_item_content_key(&item_id, "spell-description"),
            localization,
            accumulator,
        );
    }
}

fn collect_embedded_content_at_pointer(
    raw: &Value,
    pointer: &str,
    source_kind: ContentSourceKind,
    label: Option<String>,
    local_key: String,
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
    accumulator.content.push((
        Some(local_key),
        supplemental_content(source_kind, label, parsed.document),
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
    format!("#item:{item_id}:{suffix}")
}

pub(super) fn embedded_item_id(item: &Value, index: usize) -> String {
    string_field(item, "_id").unwrap_or_else(|| format!("item-{index}"))
}
