use std::collections::BTreeMap;

use atlas_domain::RecordKey;
use atlas_record::{
    ContentChildIdentity, ContentChildKind, ContentChildLocator, ContentDiagnostic,
    ContentDiagnosticKind, ContentId, ContentIdentityStability, ContentKey, ContentOrigin,
    ContentOwner, ContentProvenance, ContentRole, ContentSourceKind, DuplicateContentStatus,
    FactValue, H8ExactSourceObject, H8FieldValue, H8Identity, H8Number, H8PageSourceMetadata,
    H8Provenance, H8RecordSourceMetadata, H8UnsupportedChild, H8UnsupportedField, JournalPage,
    JournalPageEntry, JournalPageKind, JournalPageText, JournalPageTitle, JournalPageVideo,
    JournalRecord, MediaLocator, OwnedRichContent, OwnedRichContentDocument, RecordBody,
    RollTableRecord, SourceDocumentId, TableResult, TableResultEntry, TableResultKind,
    TableResultRange, TableResultSourceMetadata, TableResultTarget, UnsupportedSourceReason,
    UnsupportedSourceValue,
};

use super::dto::{
    JournalSource, PF2E_SOURCE_CONTRACT_VERSION, PF2E_SOURCE_PINNED_COMMIT,
    PF2E_SOURCE_PINNED_SYSTEM_VERSION, RollTableSource, SerializedSourceMember,
    SerializedSourceObject, SerializedSourceValue, SourcePresence,
};
use super::normalize::{
    ContentParseDiagnostics, LocalizationResolver, parse_foundry_content_with_localization,
};

pub(crate) struct H8Conversion {
    pub(crate) body: RecordBody,
    pub(crate) content_diagnostics: Vec<ContentParseDiagnostics>,
}

pub(crate) fn convert_journal(
    source: &JournalSource,
    record_key: RecordKey,
    source_path: &str,
    name: String,
    localization: Option<&dyn LocalizationResolver>,
) -> Result<H8Conversion, String> {
    let source_id = required_id(&source.root, "_id", "$")?;
    let metadata = record_metadata(&source.root, "$")?;
    let unsupported_fields = unknown_fields(
        &source.root,
        "$",
        &[
            "_id",
            "name",
            "pages",
            "folder",
            "sort",
            "ownership",
            "flags",
            "_stats",
        ],
    );
    let (pages, content, diagnostics) = match &source.pages {
        SourcePresence::Missing => (FactValue::Missing, OwnedRichContent::default(), Vec::new()),
        SourcePresence::Null => (FactValue::Null, OwnedRichContent::default(), Vec::new()),
        SourcePresence::Value(values) => {
            let stable_ids = stable_ids(values, "journal page")?;
            let mut pages = Vec::with_capacity(values.len());
            let mut content = OwnedRichContent::default();
            let mut diagnostics = Vec::new();
            for (ordinal, value) in values.iter().enumerate() {
                let ordinal = u32::try_from(ordinal)
                    .map_err(|_| "journal page ordinal is out of range".to_string())?;
                match parse_page(value, &record_key, ordinal, &stable_ids, localization)? {
                    ParsedPage::Page(parsed_page) => {
                        let ParsedPageValue {
                            page,
                            document,
                            diagnostics: parsed,
                        } = *parsed_page;
                        if let Some(document) = document {
                            content.documents.push(document);
                        }
                        if let Some(parsed) = parsed {
                            diagnostics.push(parsed);
                        }
                        pages.push(JournalPageEntry::Page(Box::new(page)));
                    }
                    ParsedPage::Unsupported(value) => {
                        pages.push(JournalPageEntry::Unsupported(value))
                    }
                }
            }
            (
                FactValue::Value(H8FieldValue::Known(pages)),
                content,
                diagnostics,
            )
        }
    };
    Ok(H8Conversion {
        body: RecordBody::Journal(JournalRecord {
            identity: H8Identity {
                record_key,
                source_id,
                name,
            },
            pages,
            source_metadata: metadata,
            content,
            unsupported_fields,
            provenance: provenance(source_path),
        }),
        content_diagnostics: diagnostics,
    })
}

pub(crate) fn convert_roll_table(
    source: &RollTableSource,
    record_key: RecordKey,
    source_path: &str,
    name: String,
    localization: Option<&dyn LocalizationResolver>,
) -> Result<H8Conversion, String> {
    let source_id = required_id(&source.root, "_id", "$")?;
    let metadata = record_metadata(&source.root, "$")?;
    let image = media_fact(&source.root, "img", "$.img")?;
    let formula = string_fact(&source.root, "formula", "$.formula")?;
    let replacement = bool_fact(&source.root, "replacement", "$.replacement")?;
    let display_roll = bool_fact(&source.root, "displayRoll", "$.displayRoll")?;
    let (description, parsed_description) =
        rich_document_fact(&source.root, "description", "$.description", localization)?;
    let mut diagnostics = parsed_description
        .as_ref()
        .map(|value| value.1.clone())
        .into_iter()
        .collect::<Vec<_>>();
    let mut content = OwnedRichContent::default();
    if let (FactValue::Value(H8FieldValue::Known(document)), Some((_, parsed_diagnostics))) =
        (&description, parsed_description.as_ref())
    {
        content.documents.push(owned_document(
            &record_key,
            ContentOwner::Record(record_key.clone()),
            ContentIdentityStability::StableSourceIdentity,
            ContentRole::PrimaryDescription,
            ContentOrigin::RecordField {
                source_kind: ContentSourceKind::Description,
                relative_source_path: "$.description".to_string(),
            },
            ContentSourceKind::Description,
            0,
            Some(name.clone()),
            document.clone(),
            "description",
            "$.description",
            None,
            diagnostics_for_content(parsed_diagnostics),
        )?);
    }
    let unsupported_fields = unknown_fields(
        &source.root,
        "$",
        &[
            "_id",
            "name",
            "description",
            "results",
            "formula",
            "replacement",
            "displayRoll",
            "img",
            "folder",
            "sort",
            "ownership",
            "flags",
            "_stats",
        ],
    );
    let results = match &source.results {
        SourcePresence::Missing => FactValue::Missing,
        SourcePresence::Null => FactValue::Null,
        SourcePresence::Value(values) => {
            let stable_ids = stable_ids(values, "table result")?;
            let mut results = Vec::with_capacity(values.len());
            for (ordinal, value) in values.iter().enumerate() {
                let ordinal = u32::try_from(ordinal)
                    .map_err(|_| "table result ordinal is out of range".to_string())?;
                match parse_result(value, &record_key, ordinal, &stable_ids, localization)? {
                    ParsedResult::Result(parsed_result) => {
                        let ParsedResultValue {
                            result,
                            document,
                            diagnostics: parsed,
                        } = *parsed_result;
                        if let Some(document) = document {
                            content.documents.push(document);
                        }
                        if let Some(parsed) = parsed {
                            diagnostics.push(parsed);
                        }
                        results.push(TableResultEntry::Result(Box::new(result)));
                    }
                    ParsedResult::Unsupported(value) => {
                        results.push(TableResultEntry::Unsupported(value))
                    }
                }
            }
            FactValue::Value(H8FieldValue::Known(results))
        }
    };
    content
        .documents
        .sort_by_key(|document| document.authored_order);
    Ok(H8Conversion {
        body: RecordBody::RollTable(RollTableRecord {
            identity: H8Identity {
                record_key,
                source_id,
                name,
            },
            description,
            results,
            formula,
            replacement,
            display_roll,
            image,
            source_metadata: metadata,
            content,
            unsupported_fields,
            provenance: provenance(source_path),
        }),
        content_diagnostics: diagnostics,
    })
}

struct ParsedPageValue {
    page: JournalPage,
    document: Option<OwnedRichContentDocument>,
    diagnostics: Option<ContentParseDiagnostics>,
}

enum ParsedPage {
    Page(Box<ParsedPageValue>),
    Unsupported(H8UnsupportedChild),
}

fn parse_page(
    value: &SerializedSourceValue,
    parent: &RecordKey,
    ordinal: u32,
    stable_ids: &BTreeMap<String, usize>,
    localization: Option<&dyn LocalizationResolver>,
) -> Result<ParsedPage, String> {
    let Some(object) = value.object() else {
        return Ok(ParsedPage::Unsupported(unsupported_child(
            value,
            parent,
            ContentChildKind::JournalPage,
            ordinal,
            "journal page is not an object",
        )));
    };
    reject_duplicate(object, "_id", "journal page identity")?;
    reject_duplicate(object, "type", "journal page dispatch")?;
    let locator = child_locator(
        object,
        parent,
        ContentChildKind::JournalPage,
        ordinal,
        stable_ids,
    );
    let source_id = id_fact(object, "_id", &format!("$.pages[{ordinal}]._id"))?;
    if page_has_duplicate_owned_member(object) {
        return Ok(ParsedPage::Unsupported(unsupported_object_child(
            value,
            locator,
            source_id,
            ordinal,
            "journal page has a duplicated owned member",
        )));
    }
    let page_kind_value = match object.member("type") {
        SerializedSourceMember::Value(SerializedSourceValue::String(kind)) => match kind.as_str() {
            "text" => JournalPageKind::Text,
            "image" => JournalPageKind::Image,
            "pdf" => JournalPageKind::Pdf,
            "video" => JournalPageKind::Video,
            _ => {
                return Ok(ParsedPage::Unsupported(unsupported_object_child(
                    value,
                    locator,
                    source_id,
                    ordinal,
                    "unknown journal page type",
                )));
            }
        },
        _ => return Err(format!("journal page {ordinal} has malformed type")),
    };
    let page_kind = FactValue::Value(H8FieldValue::Known(page_kind_value));
    if nested_object_has_unknown_member(object, "title", &["show", "level"])
        || (page_kind_value == JournalPageKind::Text
            && nested_object_has_unknown_member(object, "text", &["content", "format", "markdown"]))
        || (page_kind_value == JournalPageKind::Video
            && nested_object_has_unknown_member(
                object,
                "video",
                &[
                    "controls",
                    "loop",
                    "autoplay",
                    "volume",
                    "timestamp",
                    "width",
                    "height",
                ],
            ))
    {
        return Ok(ParsedPage::Unsupported(unsupported_object_child(
            value,
            locator,
            source_id,
            ordinal,
            "journal page has an undeclared member in an active typed object",
        )));
    }
    let name = string_fact(object, "name", &format!("$.pages[{ordinal}].name"))?;
    if !matches!(
        &name,
        FactValue::Value(H8FieldValue::Known(value)) if !value.trim().is_empty()
    ) {
        return Ok(ParsedPage::Unsupported(unsupported_object_child(
            value,
            locator,
            source_id,
            ordinal,
            "journal page has no usable authored name",
        )));
    }
    let sort = integer_fact(object, "sort", &format!("$.pages[{ordinal}].sort"))?;
    let title = object_fact(
        object,
        "title",
        &format!("$.pages[{ordinal}].title"),
        |title, path| {
            Ok(JournalPageTitle {
                show: bool_fact(title, "show", &format!("{path}.show"))?,
                level: integer_fact(title, "level", &format!("{path}.level"))?,
            })
        },
    )?;
    let text = variant_object_fact(
        object,
        "text",
        &format!("$.pages[{ordinal}].text"),
        page_kind_value == JournalPageKind::Text,
        |text, path| {
            let (content, _) =
                rich_document_fact(text, "content", &format!("{path}.content"), localization)?;
            Ok(JournalPageText {
                content,
                format: text_format_fact(text, "format", &format!("{path}.format"))?,
                markdown: string_fact(text, "markdown", &format!("{path}.markdown"))?,
            })
        },
    )?;
    let source = inactive_scalar_fact(
        object,
        "src",
        &format!("$.pages[{ordinal}].src"),
        page_kind_value != JournalPageKind::Text,
        |value| match value {
            SerializedSourceValue::String(value) => {
                MediaLocator::new(value.clone()).map_err(|_| ())
            }
            _ => Err(()),
        },
    )?;
    let image_source = exact_object_fact(object, "image", &format!("$.pages[{ordinal}].image"))?;
    let image_caption = flatten_fact(variant_object_fact(
        object,
        "image",
        &format!("$.pages[{ordinal}].image"),
        page_kind_value == JournalPageKind::Image,
        |image, path| string_fact(image, "caption", &format!("{path}.caption")),
    )?);
    let video = variant_object_fact(
        object,
        "video",
        &format!("$.pages[{ordinal}].video"),
        page_kind_value == JournalPageKind::Video,
        |video, path| {
            Ok(JournalPageVideo {
                controls: bool_fact(video, "controls", &format!("{path}.controls"))?,
                loop_playback: bool_fact(video, "loop", &format!("{path}.loop"))?,
                autoplay: bool_fact(video, "autoplay", &format!("{path}.autoplay"))?,
                volume: bounded_number_fact(video, "volume", &format!("{path}.volume"), 0.0, 1.0)?,
                timestamp: number_fact(video, "timestamp", &format!("{path}.timestamp"))?,
                width: number_fact(video, "width", &format!("{path}.width"))?,
                height: number_fact(video, "height", &format!("{path}.height"))?,
            })
        },
    )?;
    let source_system =
        journal_page_system_fact(object, "system", &format!("$.pages[{ordinal}].system"))?;
    let source_metadata = H8PageSourceMetadata {
        ownership: exact_object_fact(
            object,
            "ownership",
            &format!("$.pages[{ordinal}].ownership"),
        )?,
        flags: exact_object_fact(object, "flags", &format!("$.pages[{ordinal}].flags"))?,
        stats: exact_object_fact(object, "_stats", &format!("$.pages[{ordinal}]._stats"))?,
    };
    let mut unsupported_fields = unknown_fields(
        object,
        &format!("$.pages[{ordinal}]"),
        &[
            "_id",
            "name",
            "type",
            "sort",
            "title",
            "text",
            "src",
            "image",
            "video",
            "system",
            "ownership",
            "flags",
            "_stats",
        ],
    );
    if let SerializedSourceMember::Value(SerializedSourceValue::Object(image)) =
        object.member("image")
    {
        unsupported_fields.extend(unknown_fields(
            image,
            &format!("$.pages[{ordinal}].image"),
            &["caption"],
        ));
    }
    let (document, diagnostics) = match &text {
        FactValue::Value(H8FieldValue::Known(text)) => match &text.content {
            FactValue::Value(H8FieldValue::Known(document)) => {
                let parsed = parse_nested_string_content(object, "text", "content", localization);
                let diagnostics = parsed.as_ref().map(|value| value.1.clone());
                let owned = owned_document(
                    parent,
                    ContentOwner::Child(locator.clone()),
                    locator_stability(&locator),
                    ContentRole::JournalPage,
                    ContentOrigin::ChildField {
                        locator: locator.clone(),
                        relative_source_path: format!("$.pages[{ordinal}].text.content"),
                    },
                    ContentSourceKind::JournalPage,
                    ordinal,
                    known_string(&name).map(str::to_string),
                    document.clone(),
                    &format!(
                        "journal-page-{}",
                        atlas_record::encode_content_child_locator(&locator)
                    ),
                    &format!("$.pages[{ordinal}].text.content"),
                    source_id_value(&source_id),
                    parsed
                        .as_ref()
                        .map(|value| diagnostics_for_content(&value.1))
                        .unwrap_or_default(),
                )?;
                (Some(owned), diagnostics)
            }
            _ => (None, None),
        },
        _ => (None, None),
    };
    Ok(ParsedPage::Page(Box::new(ParsedPageValue {
        page: JournalPage {
            locator,
            source_id,
            source_ordinal: ordinal,
            name,
            page_kind,
            sort,
            title,
            text,
            source,
            image_source,
            image_caption,
            video,
            source_system,
            source_metadata,
            unsupported_fields,
        },
        document,
        diagnostics,
    })))
}

struct ParsedResultValue {
    result: TableResult,
    document: Option<OwnedRichContentDocument>,
    diagnostics: Option<ContentParseDiagnostics>,
}

enum ParsedResult {
    Result(Box<ParsedResultValue>),
    Unsupported(H8UnsupportedChild),
}

fn parse_result(
    value: &SerializedSourceValue,
    parent: &RecordKey,
    ordinal: u32,
    stable_ids: &BTreeMap<String, usize>,
    localization: Option<&dyn LocalizationResolver>,
) -> Result<ParsedResult, String> {
    let Some(object) = value.object() else {
        return Ok(ParsedResult::Unsupported(unsupported_child(
            value,
            parent,
            ContentChildKind::TableResult,
            ordinal,
            "table result is not an object",
        )));
    };
    reject_duplicate(object, "_id", "table result identity")?;
    reject_duplicate(object, "type", "table result dispatch")?;
    let locator = child_locator(
        object,
        parent,
        ContentChildKind::TableResult,
        ordinal,
        stable_ids,
    );
    let source_id = id_fact(object, "_id", &format!("$.results[{ordinal}]._id"))?;
    if result_has_duplicate_owned_member(object) {
        return Ok(ParsedResult::Unsupported(unsupported_object_child(
            value,
            locator,
            source_id,
            ordinal,
            "table result has a duplicated owned member",
        )));
    }
    let result_kind_value = match object.member("type") {
        SerializedSourceMember::Value(SerializedSourceValue::String(kind)) => match kind.as_str() {
            "text" => TableResultKind::Text,
            "pack" => TableResultKind::Pack,
            "document" => TableResultKind::Document,
            _ => {
                return Ok(ParsedResult::Unsupported(unsupported_object_child(
                    value,
                    locator,
                    source_id,
                    ordinal,
                    "unknown table result type",
                )));
            }
        },
        _ => return Err(format!("table result {ordinal} has malformed type")),
    };
    let result_kind = FactValue::Value(H8FieldValue::Known(result_kind_value));
    let (text, parsed) = rich_document_fact(
        object,
        "text",
        &format!("$.results[{ordinal}].text"),
        localization,
    )?;
    let target = TableResultTarget {
        collection: inactive_scalar_fact(
            object,
            "documentCollection",
            &format!("$.results[{ordinal}].documentCollection"),
            result_kind_value != TableResultKind::Text,
            |value| match value {
                SerializedSourceValue::String(value) => Ok(value.clone()),
                _ => Err(()),
            },
        )?,
        document_id: inactive_scalar_fact(
            object,
            "documentId",
            &format!("$.results[{ordinal}].documentId"),
            result_kind_value != TableResultKind::Text,
            |value| match value {
                SerializedSourceValue::String(value) => {
                    SourceDocumentId::new(value.clone()).map_err(|_| ())
                }
                _ => Err(()),
            },
        )?,
    };
    let weight = positive_number_fact(object, "weight", &format!("$.results[{ordinal}].weight"))?;
    let range = range_fact(object, "range", &format!("$.results[{ordinal}].range"))?;
    let drawn = bool_fact(object, "drawn", &format!("$.results[{ordinal}].drawn"))?;
    let image = media_fact(object, "img", &format!("$.results[{ordinal}].img"))?;
    let source_metadata = TableResultSourceMetadata {
        flags: exact_object_fact(object, "flags", &format!("$.results[{ordinal}].flags"))?,
    };
    let unsupported_fields = unknown_fields(
        object,
        &format!("$.results[{ordinal}]"),
        &[
            "_id",
            "type",
            "text",
            "documentCollection",
            "documentId",
            "weight",
            "range",
            "drawn",
            "img",
            "flags",
        ],
    );
    let document = match &text {
        FactValue::Value(H8FieldValue::Known(document)) => Some(owned_document(
            parent,
            ContentOwner::Child(locator.clone()),
            locator_stability(&locator),
            ContentRole::TableResult,
            ContentOrigin::ChildField {
                locator: locator.clone(),
                relative_source_path: format!("$.results[{ordinal}].text"),
            },
            ContentSourceKind::TableResult,
            ordinal.saturating_add(1),
            None,
            document.clone(),
            &format!(
                "table-result-{}",
                atlas_record::encode_content_child_locator(&locator)
            ),
            &format!("$.results[{ordinal}].text"),
            source_id_value(&source_id),
            parsed
                .as_ref()
                .map(|value| diagnostics_for_content(&value.1))
                .unwrap_or_default(),
        )?),
        _ => None,
    };
    Ok(ParsedResult::Result(Box::new(ParsedResultValue {
        result: TableResult {
            locator,
            source_id,
            source_ordinal: ordinal,
            result_kind,
            text,
            target,
            weight,
            range,
            drawn,
            image,
            source_metadata,
            unsupported_fields,
        },
        document,
        diagnostics: parsed.map(|value| value.1),
    })))
}

fn provenance(source_path: &str) -> H8Provenance {
    H8Provenance {
        source_path: source_path.to_string(),
        source_contract_version: PF2E_SOURCE_CONTRACT_VERSION.to_string(),
        source_system_version: PF2E_SOURCE_PINNED_SYSTEM_VERSION.to_string(),
        source_upstream_commit: PF2E_SOURCE_PINNED_COMMIT.to_string(),
    }
}

fn stable_ids(
    values: &[SerializedSourceValue],
    label: &str,
) -> Result<BTreeMap<String, usize>, String> {
    let mut counts = BTreeMap::new();
    for value in values {
        let Some(object) = value.object() else {
            continue;
        };
        reject_duplicate(object, "_id", &format!("{label} identity"))?;
        if let SerializedSourceMember::Value(SerializedSourceValue::String(value)) =
            object.member("_id")
            && SourceDocumentId::new(value.clone()).is_ok()
        {
            *counts.entry(value.clone()).or_insert(0) += 1;
        }
    }
    Ok(counts)
}

fn child_locator(
    object: &SerializedSourceObject,
    parent: &RecordKey,
    kind: ContentChildKind,
    ordinal: u32,
    stable_ids: &BTreeMap<String, usize>,
) -> ContentChildLocator {
    let identity = match object.member("_id") {
        SerializedSourceMember::Value(SerializedSourceValue::String(value))
            if stable_ids.get(value) == Some(&1) =>
        {
            SourceDocumentId::new(value.clone())
                .map(ContentChildIdentity::Stable)
                .unwrap_or(ContentChildIdentity::Unstable {
                    source_ordinal: ordinal,
                })
        }
        _ => ContentChildIdentity::Unstable {
            source_ordinal: ordinal,
        },
    };
    ContentChildLocator {
        parent: parent.clone(),
        kind,
        identity,
    }
}

fn required_id(
    object: &SerializedSourceObject,
    key: &str,
    path: &str,
) -> Result<SourceDocumentId, String> {
    reject_duplicate(object, key, "record identity")?;
    match object.member(key) {
        SerializedSourceMember::Value(SerializedSourceValue::String(value)) => {
            SourceDocumentId::new(value.clone()).map_err(|_| format!("{path}.{key} is invalid"))
        }
        _ => Err(format!("{path}.{key} must be a nonempty document ID")),
    }
}

fn record_metadata(
    object: &SerializedSourceObject,
    path: &str,
) -> Result<H8RecordSourceMetadata, String> {
    Ok(H8RecordSourceMetadata {
        folder: id_fact(object, "folder", &format!("{path}.folder"))?,
        sort: integer_fact(object, "sort", &format!("{path}.sort"))?,
        ownership: exact_object_fact(object, "ownership", &format!("{path}.ownership"))?,
        flags: exact_object_fact(object, "flags", &format!("{path}.flags"))?,
        stats: exact_object_fact(object, "_stats", &format!("{path}._stats"))?,
    })
}

fn id_fact(
    object: &SerializedSourceObject,
    key: &str,
    path: &str,
) -> Result<atlas_record::H8Fact<SourceDocumentId>, String> {
    member_fact(object, key, path, |value| match value {
        SerializedSourceValue::String(value) => {
            SourceDocumentId::new(value.clone()).map_err(|_| ())
        }
        _ => Err(()),
    })
}

fn media_fact(
    object: &SerializedSourceObject,
    key: &str,
    path: &str,
) -> Result<atlas_record::H8Fact<MediaLocator>, String> {
    member_fact(object, key, path, |value| match value {
        SerializedSourceValue::String(value) => MediaLocator::new(value.clone()).map_err(|_| ()),
        _ => Err(()),
    })
}

fn string_fact(
    object: &SerializedSourceObject,
    key: &str,
    path: &str,
) -> Result<atlas_record::H8Fact<String>, String> {
    member_fact(object, key, path, |value| match value {
        SerializedSourceValue::String(value) => Ok(value.clone()),
        _ => Err(()),
    })
}

fn bool_fact(
    object: &SerializedSourceObject,
    key: &str,
    path: &str,
) -> Result<atlas_record::H8Fact<bool>, String> {
    member_fact(object, key, path, |value| match value {
        SerializedSourceValue::Boolean(value) => Ok(*value),
        _ => Err(()),
    })
}

fn integer_fact(
    object: &SerializedSourceObject,
    key: &str,
    path: &str,
) -> Result<atlas_record::H8Fact<i64>, String> {
    member_fact(object, key, path, |value| match value {
        SerializedSourceValue::Number(value) => value.as_i64().ok_or(()),
        _ => Err(()),
    })
}

fn text_format_fact(
    object: &SerializedSourceObject,
    key: &str,
    path: &str,
) -> Result<atlas_record::H8Fact<i64>, String> {
    member_fact(object, key, path, |value| match value {
        SerializedSourceValue::Number(value) => match value.as_i64() {
            Some(format @ (1 | 2)) => Ok(format),
            _ => Err(()),
        },
        _ => Err(()),
    })
}

fn number_fact(
    object: &SerializedSourceObject,
    key: &str,
    path: &str,
) -> Result<atlas_record::H8Fact<H8Number>, String> {
    member_fact(object, key, path, |value| match value {
        SerializedSourceValue::Number(value) => Ok(H8Number {
            canonical: value.to_string(),
        }),
        _ => Err(()),
    })
}

fn positive_number_fact(
    object: &SerializedSourceObject,
    key: &str,
    path: &str,
) -> Result<atlas_record::H8Fact<H8Number>, String> {
    Ok(validate_number(number_fact(object, key, path)?, |value| {
        value > 0.0
    }))
}

fn bounded_number_fact(
    object: &SerializedSourceObject,
    key: &str,
    path: &str,
    minimum: f64,
    maximum: f64,
) -> Result<atlas_record::H8Fact<H8Number>, String> {
    Ok(validate_number(number_fact(object, key, path)?, |value| {
        value >= minimum && value <= maximum
    }))
}

fn validate_number(
    fact: atlas_record::H8Fact<H8Number>,
    predicate: impl FnOnce(f64) -> bool,
) -> atlas_record::H8Fact<H8Number> {
    match fact {
        FactValue::Value(H8FieldValue::Known(value)) => match value.as_f64() {
            Some(number) if number.is_finite() && predicate(number) => {
                FactValue::Value(H8FieldValue::Known(value))
            }
            _ => FactValue::Value(H8FieldValue::Unsupported(UnsupportedSourceValue {
                shape: atlas_record::UnsupportedSourceShape::Number,
                value: value.canonical,
                reason: UnsupportedSourceReason::NonCanonicalRuntimeValue,
            })),
        },
        other => other,
    }
}

fn range_fact(
    object: &SerializedSourceObject,
    key: &str,
    path: &str,
) -> Result<atlas_record::H8Fact<TableResultRange>, String> {
    member_fact(object, key, path, |value| match value {
        SerializedSourceValue::Array(values) if values.len() == 2 => {
            let SerializedSourceValue::Number(first) = &values[0] else {
                return Err(());
            };
            let SerializedSourceValue::Number(last) = &values[1] else {
                return Err(());
            };
            let (Some(first), Some(last)) = (first.as_i64(), last.as_i64()) else {
                return Err(());
            };
            (first <= last)
                .then_some(TableResultRange { first, last })
                .ok_or(())
        }
        _ => Err(()),
    })
}

fn exact_object_fact(
    object: &SerializedSourceObject,
    key: &str,
    path: &str,
) -> Result<atlas_record::H8Fact<H8ExactSourceObject>, String> {
    member_fact(object, key, path, |value| match value {
        SerializedSourceValue::Object(value) => Ok(H8ExactSourceObject {
            compact_json: SerializedSourceValue::Object(value.clone()).compact_json(),
        }),
        _ => Err(()),
    })
}

fn journal_page_system_fact(
    object: &SerializedSourceObject,
    key: &str,
    path: &str,
) -> Result<atlas_record::H8Fact<H8ExactSourceObject>, String> {
    match object.member(key) {
        SerializedSourceMember::Missing => Ok(FactValue::Missing),
        SerializedSourceMember::Null => Ok(FactValue::Null),
        SerializedSourceMember::Value(SerializedSourceValue::Object(value))
            if value.fields().is_empty() =>
        {
            Ok(FactValue::Value(H8FieldValue::Known(H8ExactSourceObject {
                compact_json: "{}".to_string(),
            })))
        }
        SerializedSourceMember::Value(value) => Ok(FactValue::Value(H8FieldValue::Unsupported(
            unsupported(value),
        ))),
        SerializedSourceMember::Duplicate(values) => Err(format!(
            "{path} is duplicated: [{}]",
            values
                .iter()
                .map(|value| value.compact_json())
                .collect::<Vec<_>>()
                .join(",")
        )),
    }
}

fn object_fact<T>(
    object: &SerializedSourceObject,
    key: &str,
    path: &str,
    parse: impl FnOnce(&SerializedSourceObject, &str) -> Result<T, String>,
) -> Result<atlas_record::H8Fact<T>, String> {
    match object.member(key) {
        SerializedSourceMember::Missing => Ok(FactValue::Missing),
        SerializedSourceMember::Null => Ok(FactValue::Null),
        SerializedSourceMember::Value(SerializedSourceValue::Object(value)) => {
            parse(value, path).map(|value| FactValue::Value(H8FieldValue::Known(value)))
        }
        SerializedSourceMember::Value(value) => Ok(FactValue::Value(H8FieldValue::Unsupported(
            unsupported(value),
        ))),
        SerializedSourceMember::Duplicate(values) => Err(format!(
            "{path} is duplicated: [{}]",
            values
                .iter()
                .map(|value| value.compact_json())
                .collect::<Vec<_>>()
                .join(",")
        )),
    }
}

fn variant_object_fact<T>(
    object: &SerializedSourceObject,
    key: &str,
    path: &str,
    active: bool,
    parse: impl FnOnce(&SerializedSourceObject, &str) -> Result<T, String>,
) -> Result<atlas_record::H8Fact<T>, String> {
    match object.member(key) {
        SerializedSourceMember::Missing => Ok(FactValue::Missing),
        SerializedSourceMember::Null => Ok(FactValue::Null),
        SerializedSourceMember::Value(SerializedSourceValue::Object(value))
            if active || value.fields().is_empty() =>
        {
            parse(value, path).map(|value| FactValue::Value(H8FieldValue::Known(value)))
        }
        SerializedSourceMember::Value(value) => Ok(FactValue::Value(H8FieldValue::Unsupported(
            unsupported(value),
        ))),
        SerializedSourceMember::Duplicate(values) => Err(format!(
            "{path} is duplicated: [{}]",
            values
                .iter()
                .map(|value| value.compact_json())
                .collect::<Vec<_>>()
                .join(",")
        )),
    }
}

fn inactive_scalar_fact<T>(
    object: &SerializedSourceObject,
    key: &str,
    path: &str,
    active: bool,
    parse: impl FnOnce(&SerializedSourceValue) -> Result<T, ()>,
) -> Result<atlas_record::H8Fact<T>, String> {
    match object.member(key) {
        SerializedSourceMember::Missing => Ok(FactValue::Missing),
        SerializedSourceMember::Null => Ok(FactValue::Null),
        SerializedSourceMember::Value(value) if active => Ok(match parse(value) {
            Ok(value) => FactValue::Value(H8FieldValue::Known(value)),
            Err(()) => FactValue::Value(H8FieldValue::Unsupported(unsupported(value))),
        }),
        SerializedSourceMember::Value(value) => Ok(FactValue::Value(H8FieldValue::Unsupported(
            unsupported(value),
        ))),
        SerializedSourceMember::Duplicate(values) => Err(format!(
            "{path} is duplicated: [{}]",
            values
                .iter()
                .map(|value| value.compact_json())
                .collect::<Vec<_>>()
                .join(",")
        )),
    }
}

fn member_fact<T>(
    object: &SerializedSourceObject,
    key: &str,
    path: &str,
    parse: impl FnOnce(&SerializedSourceValue) -> Result<T, ()>,
) -> Result<atlas_record::H8Fact<T>, String> {
    match object.member(key) {
        SerializedSourceMember::Missing => Ok(FactValue::Missing),
        SerializedSourceMember::Null => Ok(FactValue::Null),
        SerializedSourceMember::Value(value) => Ok(match parse(value) {
            Ok(value) => FactValue::Value(H8FieldValue::Known(value)),
            Err(()) => FactValue::Value(H8FieldValue::Unsupported(unsupported(value))),
        }),
        SerializedSourceMember::Duplicate(values) => Err(format!(
            "{path} is duplicated: [{}]",
            values
                .iter()
                .map(|value| value.compact_json())
                .collect::<Vec<_>>()
                .join(",")
        )),
    }
}

type ParsedRichDocumentFact = (
    atlas_record::H8Fact<atlas_record::RichDocument>,
    Option<(atlas_record::RichDocument, ContentParseDiagnostics)>,
);

fn rich_document_fact(
    object: &SerializedSourceObject,
    key: &str,
    path: &str,
    localization: Option<&dyn LocalizationResolver>,
) -> Result<ParsedRichDocumentFact, String> {
    match object.member(key) {
        SerializedSourceMember::Missing => Ok((FactValue::Missing, None)),
        SerializedSourceMember::Null => Ok((FactValue::Null, None)),
        SerializedSourceMember::Value(SerializedSourceValue::String(value)) => {
            let parsed = parse_foundry_content_with_localization(value, localization);
            Ok((
                FactValue::Value(H8FieldValue::Known(parsed.document.clone())),
                Some((parsed.document, parsed.diagnostics)),
            ))
        }
        SerializedSourceMember::Value(value) => Ok((
            FactValue::Value(H8FieldValue::Unsupported(unsupported(value))),
            None,
        )),
        SerializedSourceMember::Duplicate(values) => Err(format!(
            "{path} is duplicated: [{}]",
            values
                .iter()
                .map(|value| value.compact_json())
                .collect::<Vec<_>>()
                .join(",")
        )),
    }
}

fn parse_string_content(
    object: &SerializedSourceObject,
    key: &str,
    localization: Option<&dyn LocalizationResolver>,
) -> Option<(atlas_record::RichDocument, ContentParseDiagnostics)> {
    match object.member(key) {
        SerializedSourceMember::Value(SerializedSourceValue::String(value)) => {
            let parsed = parse_foundry_content_with_localization(value, localization);
            Some((parsed.document, parsed.diagnostics))
        }
        _ => None,
    }
}

fn parse_nested_string_content(
    object: &SerializedSourceObject,
    parent: &str,
    key: &str,
    localization: Option<&dyn LocalizationResolver>,
) -> Option<(atlas_record::RichDocument, ContentParseDiagnostics)> {
    let SerializedSourceMember::Value(SerializedSourceValue::Object(parent)) =
        object.member(parent)
    else {
        return None;
    };
    parse_string_content(parent, key, localization)
}

fn unsupported(value: &SerializedSourceValue) -> UnsupportedSourceValue {
    UnsupportedSourceValue {
        shape: value.shape(),
        value: value.compact_json(),
        reason: UnsupportedSourceReason::SourceFieldDrift,
    }
}

fn unknown_fields(
    object: &SerializedSourceObject,
    path: &str,
    known: &[&str],
) -> Vec<H8UnsupportedField> {
    object
        .fields()
        .iter()
        .enumerate()
        .filter(|(_, (key, _))| !known.contains(&key.as_str()))
        .map(|(order, (key, value))| H8UnsupportedField {
            relative_path: format!("{path}.{key}"),
            authored_order: u32::try_from(order).unwrap_or(u32::MAX),
            value: UnsupportedSourceValue {
                shape: value.shape(),
                value: value.compact_json(),
                reason: UnsupportedSourceReason::OpenVocabulary,
            },
        })
        .collect()
}

fn nested_object_has_unknown_member(
    object: &SerializedSourceObject,
    key: &str,
    known: &[&str],
) -> bool {
    match object.member(key) {
        SerializedSourceMember::Value(SerializedSourceValue::Object(value)) => value
            .fields()
            .iter()
            .any(|(member, _)| !known.contains(&member.as_str())),
        _ => false,
    }
}

fn unsupported_child(
    value: &SerializedSourceValue,
    parent: &RecordKey,
    kind: ContentChildKind,
    ordinal: u32,
    reason: &str,
) -> H8UnsupportedChild {
    H8UnsupportedChild {
        locator: ContentChildLocator {
            parent: parent.clone(),
            kind,
            identity: ContentChildIdentity::Unstable {
                source_ordinal: ordinal,
            },
        },
        source_id: FactValue::Missing,
        source_ordinal: ordinal,
        exact_source: H8ExactSourceObject {
            compact_json: value.compact_json(),
        },
        reason: reason.to_string(),
    }
}

fn unsupported_object_child(
    value: &SerializedSourceValue,
    locator: ContentChildLocator,
    source_id: atlas_record::H8Fact<SourceDocumentId>,
    ordinal: u32,
    reason: &str,
) -> H8UnsupportedChild {
    H8UnsupportedChild {
        locator,
        source_id,
        source_ordinal: ordinal,
        exact_source: H8ExactSourceObject {
            compact_json: value.compact_json(),
        },
        reason: reason.to_string(),
    }
}

fn reject_duplicate(object: &SerializedSourceObject, key: &str, label: &str) -> Result<(), String> {
    if let SerializedSourceMember::Duplicate(values) = object.member(key) {
        return Err(format!(
            "duplicate {label}: [{}]",
            values
                .iter()
                .map(|value| value.compact_json())
                .collect::<Vec<_>>()
                .join(",")
        ));
    }
    Ok(())
}

fn page_has_duplicate_owned_member(object: &SerializedSourceObject) -> bool {
    has_duplicate_member(
        object,
        &[
            "name",
            "sort",
            "title",
            "text",
            "src",
            "image",
            "video",
            "system",
            "ownership",
            "flags",
            "_stats",
        ],
    ) || nested_has_duplicate_member(object, "title", &["show", "level"])
        || nested_has_duplicate_member(object, "text", &["content", "format", "markdown"])
        || nested_has_duplicate_member(object, "image", &["caption"])
        || nested_has_duplicate_member(
            object,
            "video",
            &[
                "controls",
                "loop",
                "autoplay",
                "volume",
                "timestamp",
                "width",
                "height",
            ],
        )
}

fn result_has_duplicate_owned_member(object: &SerializedSourceObject) -> bool {
    has_duplicate_member(
        object,
        &[
            "text",
            "documentCollection",
            "documentId",
            "weight",
            "range",
            "drawn",
            "img",
            "flags",
        ],
    )
}

fn has_duplicate_member(object: &SerializedSourceObject, keys: &[&str]) -> bool {
    keys.iter()
        .any(|key| matches!(object.member(key), SerializedSourceMember::Duplicate(_)))
}

fn nested_has_duplicate_member(
    object: &SerializedSourceObject,
    parent: &str,
    keys: &[&str],
) -> bool {
    match object.member(parent) {
        SerializedSourceMember::Value(SerializedSourceValue::Object(object)) => {
            has_duplicate_member(object, keys)
        }
        _ => false,
    }
}

#[allow(clippy::too_many_arguments)]
fn owned_document(
    parent: &RecordKey,
    owner: ContentOwner,
    stability: ContentIdentityStability,
    role: ContentRole,
    origin: ContentOrigin,
    source_kind: ContentSourceKind,
    authored_order: u32,
    label: Option<String>,
    document: atlas_record::RichDocument,
    content_key: &str,
    field_path: &str,
    nested_source_id: Option<String>,
    diagnostics: Vec<ContentDiagnostic>,
) -> Result<OwnedRichContentDocument, String> {
    let content_key = ContentKey::new(content_key.to_string())
        .map_err(|_| format!("invalid H8 content key `{content_key}`"))?;
    Ok(OwnedRichContentDocument::new(
        ContentId::new(parent.clone(), content_key),
        stability,
        owner,
        role,
        origin,
        source_kind.default_visibility(),
        ContentProvenance {
            source_record_key: parent.clone(),
            relative_source_path: field_path.to_string(),
            field_or_pointer_family: field_path.to_string(),
            nested_source_id,
            authored_ordinal_or_range: Some(authored_order.to_string()),
            authored_label: label.clone(),
        },
        source_kind,
        authored_order,
        label,
        document,
        DuplicateContentStatus::Unique,
        diagnostics,
    ))
}

fn diagnostics_for_content(value: &ContentParseDiagnostics) -> Vec<ContentDiagnostic> {
    value
        .unsupported_tags
        .iter()
        .map(|value| ContentDiagnostic {
            kind: ContentDiagnosticKind::UnsupportedTag,
            subject: value.clone(),
            detail: None,
        })
        .chain(
            value
                .unsupported_attributes
                .iter()
                .map(|value| ContentDiagnostic {
                    kind: ContentDiagnosticKind::UnsupportedAttribute,
                    subject: value.name.clone(),
                    detail: Some(value.tag.clone()),
                }),
        )
        .chain(value.unknown_macros.iter().map(|value| ContentDiagnostic {
            kind: ContentDiagnosticKind::UnknownFoundryMacro,
            subject: value.clone(),
            detail: None,
        }))
        .collect()
}

fn locator_stability(locator: &ContentChildLocator) -> ContentIdentityStability {
    match locator.identity {
        ContentChildIdentity::Stable(_) => ContentIdentityStability::StableSourceIdentity,
        ContentChildIdentity::Unstable { .. } => ContentIdentityStability::UnstableAuthoredOrdinal,
    }
}

fn source_id_value(value: &atlas_record::H8Fact<SourceDocumentId>) -> Option<String> {
    match value {
        FactValue::Value(H8FieldValue::Known(value)) => Some(value.as_str().to_string()),
        _ => None,
    }
}

fn known_string(value: &atlas_record::H8Fact<String>) -> Option<&str> {
    match value {
        FactValue::Value(H8FieldValue::Known(value)) => Some(value),
        _ => None,
    }
}

fn flatten_fact<T>(
    value: atlas_record::H8Fact<atlas_record::H8Fact<T>>,
) -> atlas_record::H8Fact<T> {
    match value {
        FactValue::Missing => FactValue::Missing,
        FactValue::Null => FactValue::Null,
        FactValue::Value(H8FieldValue::Unsupported(value)) => {
            FactValue::Value(H8FieldValue::Unsupported(value))
        }
        FactValue::Value(H8FieldValue::Known(value)) => value,
    }
}
