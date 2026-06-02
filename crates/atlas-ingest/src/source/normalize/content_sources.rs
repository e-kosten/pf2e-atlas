use atlas_record::{ContentDocument, ContentSourceKind, RecordContentDocument};
use serde_json::Value;

use super::{ContentParseDiagnostics, parse_foundry_content, pointer_string, string_field};

pub(super) struct SourceContentProjection {
    pub description: Option<ContentDocument>,
    pub blurb: Option<ContentDocument>,
    pub supplemental_content: Vec<(Option<String>, RecordContentDocument)>,
    pub diagnostics: Vec<ContentParseDiagnostics>,
}

pub(super) fn extract_content_sources(raw: &Value) -> SourceContentProjection {
    let source_description_raw = pointer_string(raw, "/system/description/value");
    let parsed_description = source_description_raw.as_deref().map(parse_foundry_content);
    let description = parsed_description
        .as_ref()
        .map(|parsed| parsed.document.clone())
        .filter(non_empty_document);

    let source_blurb_markup = pointer_string(raw, "/system/details/blurb");
    let parsed_blurb = source_blurb_markup.as_deref().map(parse_foundry_content);
    let blurb = parsed_blurb
        .as_ref()
        .map(|parsed| parsed.document.clone())
        .filter(non_empty_document);

    let (supplemental_content, supplemental_diagnostics) =
        extract_supplemental_content(raw, source_description_raw.as_deref());

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

fn non_empty_document(document: &ContentDocument) -> bool {
    !document.is_empty()
}

fn extract_supplemental_content(
    raw: &Value,
    source_description_raw: Option<&str>,
) -> (
    Vec<(Option<String>, RecordContentDocument)>,
    Vec<ContentParseDiagnostics>,
) {
    let mut content = Vec::new();
    let mut diagnostics = Vec::new();
    collect_content_at_pointer(
        raw,
        "/system/details/disable",
        ContentSourceKind::Disable,
        None,
        &mut content,
        &mut diagnostics,
    );
    collect_content_at_pointer(
        raw,
        "/system/details/routine",
        ContentSourceKind::Routine,
        None,
        &mut content,
        &mut diagnostics,
    );
    collect_content_at_pointer(
        raw,
        "/system/details/reset",
        ContentSourceKind::Reset,
        None,
        &mut content,
        &mut diagnostics,
    );
    collect_content_at_pointer(
        raw,
        "/system/attributes/stealth/details",
        ContentSourceKind::StealthDetails,
        Some("Stealth".to_string()),
        &mut content,
        &mut diagnostics,
    );
    if pointer_string(raw, "/system/details/description").as_deref() != source_description_raw {
        collect_content_at_pointer(
            raw,
            "/system/details/description",
            ContentSourceKind::DetailsFieldDescription,
            None,
            &mut content,
            &mut diagnostics,
        );
    }
    collect_content_at_pointer(
        raw,
        "/system/details/publicNotes",
        ContentSourceKind::PublicNotes,
        Some("Public Notes".to_string()),
        &mut content,
        &mut diagnostics,
    );
    collect_content_at_pointer(
        raw,
        "/system/description/gm",
        ContentSourceKind::GmNotes,
        Some("GM Notes".to_string()),
        &mut content,
        &mut diagnostics,
    );
    collect_content_at_pointer(
        raw,
        "/system/details/gmNotes",
        ContentSourceKind::GmNotes,
        Some("GM Notes".to_string()),
        &mut content,
        &mut diagnostics,
    );
    collect_content_at_pointer(
        raw,
        "/system/details/privateNotes",
        ContentSourceKind::PrivateNotes,
        Some("Private Notes".to_string()),
        &mut content,
        &mut diagnostics,
    );
    collect_embedded_item_content(raw, &mut content, &mut diagnostics);
    (content, diagnostics)
}

fn collect_content_at_pointer(
    raw: &Value,
    pointer: &str,
    source_kind: ContentSourceKind,
    label: Option<String>,
    content: &mut Vec<(Option<String>, RecordContentDocument)>,
    diagnostics: &mut Vec<ContentParseDiagnostics>,
) {
    let Some(markup) = pointer_string(raw, pointer) else {
        return;
    };
    let parsed = parse_foundry_content(&markup);
    if parsed.document.is_empty() {
        return;
    }
    diagnostics.push(parsed.diagnostics.clone());
    content.push((
        None,
        supplemental_content(source_kind, label, parsed.document),
    ));
}

fn collect_embedded_item_content(
    raw: &Value,
    content: &mut Vec<(Option<String>, RecordContentDocument)>,
    diagnostics: &mut Vec<ContentParseDiagnostics>,
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
            content,
            diagnostics,
        );
        collect_embedded_content_at_pointer(
            item,
            "/system/spell/system/description/value",
            ContentSourceKind::EmbeddedSpellDescription,
            label,
            embedded_item_content_key(&item_id, "spell-description"),
            content,
            diagnostics,
        );
    }
}

fn collect_embedded_content_at_pointer(
    raw: &Value,
    pointer: &str,
    source_kind: ContentSourceKind,
    label: Option<String>,
    local_key: String,
    content: &mut Vec<(Option<String>, RecordContentDocument)>,
    diagnostics: &mut Vec<ContentParseDiagnostics>,
) {
    let Some(markup) = pointer_string(raw, pointer) else {
        return;
    };
    let parsed = parse_foundry_content(&markup);
    if parsed.document.is_empty() {
        return;
    }
    diagnostics.push(parsed.diagnostics.clone());
    content.push((
        Some(local_key),
        supplemental_content(source_kind, label, parsed.document),
    ));
}

fn supplemental_content(
    source_kind: ContentSourceKind,
    label: Option<String>,
    document: ContentDocument,
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
