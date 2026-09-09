use serde::Serialize;

use crate::{
    ContentChildIdentity, ContentChildLocator, ContentOwner, FactValue, H8ExactSourceObject,
    H8Fact, H8FieldValue, H8Number, H8Provenance, H8RecordSourceMetadata, H8UnsupportedChild,
    H8UnsupportedField, JournalPage, JournalPageEntry, JournalPageKind, JournalPageText,
    JournalPageTitle, JournalPageVideo, JournalRecord, OwnedRichContent, RichDocument,
    RollTableRecord, TableResult, TableResultEntry, TableResultKind, UnsupportedSourceReason,
    UnsupportedSourceShape, UnsupportedSourceValue, encode_content_child_locator,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "state", content = "value", rename_all = "snake_case")]
pub enum H8FactJson<T> {
    Missing,
    Null,
    Known(T),
    Unsupported(H8UnsupportedValueJson),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct H8UnsupportedValueJson {
    pub shape: &'static str,
    pub value: String,
    pub reason: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct H8UnsupportedFieldJson {
    pub relative_path: String,
    pub authored_order: u32,
    pub value: H8UnsupportedValueJson,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct H8SourceMetadataJson {
    pub folder: H8FactJson<String>,
    pub sort: H8FactJson<i64>,
    pub ownership: H8FactJson<String>,
    pub flags: H8FactJson<String>,
    pub stats: H8FactJson<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct H8ProvenanceJson {
    pub source_path: String,
    pub source_contract_version: String,
    pub source_system_version: String,
    pub source_upstream_commit: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct JournalJson {
    pub source_id: String,
    pub pages: H8FactJson<Vec<JournalPageEntryJson>>,
    pub source_metadata: H8SourceMetadataJson,
    pub unsupported_fields: Vec<H8UnsupportedFieldJson>,
    pub provenance: H8ProvenanceJson,
    pub content: Vec<H8ContentJson>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "entry_type", rename_all = "snake_case")]
pub enum JournalPageEntryJson {
    Page { page: Box<JournalPageJson> },
    Unsupported { unsupported: H8UnsupportedChildJson },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct JournalPageJson {
    pub locator: String,
    pub identity_stability: &'static str,
    pub source_id: H8FactJson<String>,
    pub source_ordinal: u32,
    pub name: H8FactJson<String>,
    pub page_kind: H8FactJson<&'static str>,
    pub sort: H8FactJson<i64>,
    pub title: H8FactJson<JournalPageTitleJson>,
    pub text: H8FactJson<JournalPageTextJson>,
    pub source: H8FactJson<String>,
    pub image_source: H8FactJson<String>,
    pub image_caption: H8FactJson<String>,
    pub video: H8FactJson<JournalPageVideoJson>,
    pub source_system: H8FactJson<String>,
    pub source_metadata: H8PageSourceMetadataJson,
    pub unsupported_fields: Vec<H8UnsupportedFieldJson>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct JournalPageTitleJson {
    pub show: H8FactJson<bool>,
    pub level: H8FactJson<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct JournalPageTextJson {
    pub content: H8FactJson<RichDocument>,
    pub format: H8FactJson<i64>,
    pub markdown: H8FactJson<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct JournalPageVideoJson {
    pub controls: H8FactJson<bool>,
    pub loop_playback: H8FactJson<bool>,
    pub autoplay: H8FactJson<bool>,
    pub volume: H8FactJson<String>,
    pub timestamp: H8FactJson<String>,
    pub width: H8FactJson<String>,
    pub height: H8FactJson<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct H8PageSourceMetadataJson {
    pub ownership: H8FactJson<String>,
    pub flags: H8FactJson<String>,
    pub stats: H8FactJson<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RollTableJson {
    pub source_id: String,
    pub description: H8FactJson<RichDocument>,
    pub results: H8FactJson<Vec<TableResultEntryJson>>,
    pub formula: H8FactJson<String>,
    pub replacement: H8FactJson<bool>,
    pub display_roll: H8FactJson<bool>,
    pub image: H8FactJson<String>,
    pub source_metadata: H8SourceMetadataJson,
    pub unsupported_fields: Vec<H8UnsupportedFieldJson>,
    pub provenance: H8ProvenanceJson,
    pub content: Vec<H8ContentJson>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "entry_type", rename_all = "snake_case")]
pub enum TableResultEntryJson {
    Result { result: Box<TableResultJson> },
    Unsupported { unsupported: H8UnsupportedChildJson },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TableResultJson {
    pub locator: String,
    pub identity_stability: &'static str,
    pub source_id: H8FactJson<String>,
    pub source_ordinal: u32,
    pub result_kind: H8FactJson<&'static str>,
    pub text: H8FactJson<RichDocument>,
    pub collection: H8FactJson<String>,
    pub document_id: H8FactJson<String>,
    pub weight: H8FactJson<String>,
    pub range: H8FactJson<TableResultRangeJson>,
    pub drawn: H8FactJson<bool>,
    pub image: H8FactJson<String>,
    pub flags: H8FactJson<String>,
    pub unsupported_fields: Vec<H8UnsupportedFieldJson>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TableResultRangeJson {
    pub first: i64,
    pub last: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct H8UnsupportedChildJson {
    pub locator: String,
    pub identity_stability: &'static str,
    pub source_id: H8FactJson<String>,
    pub source_ordinal: u32,
    pub exact_source: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct H8ContentJson {
    pub content_key: String,
    pub child_locator: Option<String>,
    pub role: &'static str,
    pub authored_order: u32,
    pub label: Option<String>,
    pub document: RichDocument,
    pub content_hash: String,
    pub relative_source_path: String,
}

pub(super) fn journal(value: &JournalRecord) -> JournalJson {
    JournalJson {
        source_id: value.identity.source_id.as_str().to_string(),
        pages: fact(&value.pages, |pages| {
            pages.iter().map(journal_page_entry).collect()
        }),
        source_metadata: source_metadata(&value.source_metadata),
        unsupported_fields: value
            .unsupported_fields
            .iter()
            .map(unsupported_field)
            .collect(),
        provenance: provenance(&value.provenance),
        content: content(&value.content),
    }
}

pub(super) fn roll_table(value: &RollTableRecord) -> RollTableJson {
    RollTableJson {
        source_id: value.identity.source_id.as_str().to_string(),
        description: fact(&value.description, Clone::clone),
        results: fact(&value.results, |results| {
            results.iter().map(table_result_entry).collect()
        }),
        formula: fact(&value.formula, Clone::clone),
        replacement: fact(&value.replacement, |value| *value),
        display_roll: fact(&value.display_roll, |value| *value),
        image: fact(&value.image, |value| value.as_str().to_string()),
        source_metadata: source_metadata(&value.source_metadata),
        unsupported_fields: value
            .unsupported_fields
            .iter()
            .map(unsupported_field)
            .collect(),
        provenance: provenance(&value.provenance),
        content: content(&value.content),
    }
}

fn journal_page_entry(value: &JournalPageEntry) -> JournalPageEntryJson {
    match value {
        JournalPageEntry::Page(value) => JournalPageEntryJson::Page {
            page: Box::new(journal_page(value)),
        },
        JournalPageEntry::Unsupported(value) => JournalPageEntryJson::Unsupported {
            unsupported: unsupported_child(value),
        },
    }
}

fn journal_page(value: &JournalPage) -> JournalPageJson {
    JournalPageJson {
        locator: encode_content_child_locator(&value.locator),
        identity_stability: identity_stability(&value.locator),
        source_id: fact(&value.source_id, |value| value.as_str().to_string()),
        source_ordinal: value.source_ordinal,
        name: fact(&value.name, Clone::clone),
        page_kind: fact(&value.page_kind, |value| match value {
            JournalPageKind::Text => "text",
            JournalPageKind::Image => "image",
            JournalPageKind::Pdf => "pdf",
            JournalPageKind::Video => "video",
        }),
        sort: fact(&value.sort, |value| *value),
        title: fact(&value.title, title),
        text: fact(&value.text, text),
        source: fact(&value.source, |value| value.as_str().to_string()),
        image_source: exact_object(&value.image_source),
        image_caption: fact(&value.image_caption, Clone::clone),
        video: fact(&value.video, video),
        source_system: fact(&value.source_system, |value| value.compact_json.clone()),
        source_metadata: H8PageSourceMetadataJson {
            ownership: exact_object(&value.source_metadata.ownership),
            flags: exact_object(&value.source_metadata.flags),
            stats: exact_object(&value.source_metadata.stats),
        },
        unsupported_fields: value
            .unsupported_fields
            .iter()
            .map(unsupported_field)
            .collect(),
    }
}

fn title(value: &JournalPageTitle) -> JournalPageTitleJson {
    JournalPageTitleJson {
        show: fact(&value.show, |value| *value),
        level: fact(&value.level, |value| *value),
    }
}

fn text(value: &JournalPageText) -> JournalPageTextJson {
    JournalPageTextJson {
        content: fact(&value.content, Clone::clone),
        format: fact(&value.format, |value| *value),
        markdown: fact(&value.markdown, Clone::clone),
    }
}

fn video(value: &JournalPageVideo) -> JournalPageVideoJson {
    JournalPageVideoJson {
        controls: fact(&value.controls, |value| *value),
        loop_playback: fact(&value.loop_playback, |value| *value),
        autoplay: fact(&value.autoplay, |value| *value),
        volume: number(&value.volume),
        timestamp: number(&value.timestamp),
        width: number(&value.width),
        height: number(&value.height),
    }
}

fn table_result_entry(value: &TableResultEntry) -> TableResultEntryJson {
    match value {
        TableResultEntry::Result(value) => TableResultEntryJson::Result {
            result: Box::new(table_result(value)),
        },
        TableResultEntry::Unsupported(value) => TableResultEntryJson::Unsupported {
            unsupported: unsupported_child(value),
        },
    }
}

fn table_result(value: &TableResult) -> TableResultJson {
    TableResultJson {
        locator: encode_content_child_locator(&value.locator),
        identity_stability: identity_stability(&value.locator),
        source_id: fact(&value.source_id, |value| value.as_str().to_string()),
        source_ordinal: value.source_ordinal,
        result_kind: fact(&value.result_kind, |value| match value {
            TableResultKind::Text => "text",
            TableResultKind::Pack => "pack",
            TableResultKind::Document => "document",
        }),
        text: fact(&value.text, Clone::clone),
        collection: fact(&value.target.collection, Clone::clone),
        document_id: fact(&value.target.document_id, |value| {
            value.as_str().to_string()
        }),
        weight: number(&value.weight),
        range: fact(&value.range, |value| TableResultRangeJson {
            first: value.first,
            last: value.last,
        }),
        drawn: fact(&value.drawn, |value| *value),
        image: fact(&value.image, |value| value.as_str().to_string()),
        flags: exact_object(&value.source_metadata.flags),
        unsupported_fields: value
            .unsupported_fields
            .iter()
            .map(unsupported_field)
            .collect(),
    }
}

fn unsupported_child(value: &H8UnsupportedChild) -> H8UnsupportedChildJson {
    H8UnsupportedChildJson {
        locator: encode_content_child_locator(&value.locator),
        identity_stability: identity_stability(&value.locator),
        source_id: fact(&value.source_id, |value| value.as_str().to_string()),
        source_ordinal: value.source_ordinal,
        exact_source: value.exact_source.compact_json.clone(),
        reason: value.reason.clone(),
    }
}

fn source_metadata(value: &H8RecordSourceMetadata) -> H8SourceMetadataJson {
    H8SourceMetadataJson {
        folder: fact(&value.folder, |value| value.as_str().to_string()),
        sort: fact(&value.sort, |value| *value),
        ownership: exact_object(&value.ownership),
        flags: exact_object(&value.flags),
        stats: exact_object(&value.stats),
    }
}

fn exact_object(value: &H8Fact<H8ExactSourceObject>) -> H8FactJson<String> {
    fact(value, |value| value.compact_json.clone())
}

fn number(value: &H8Fact<H8Number>) -> H8FactJson<String> {
    fact(value, |value| value.canonical.clone())
}

fn fact<T, U>(value: &H8Fact<T>, map: impl FnOnce(&T) -> U) -> H8FactJson<U> {
    match value {
        FactValue::Missing => H8FactJson::Missing,
        FactValue::Null => H8FactJson::Null,
        FactValue::Value(H8FieldValue::Known(value)) => H8FactJson::Known(map(value)),
        FactValue::Value(H8FieldValue::Unsupported(value)) => {
            H8FactJson::Unsupported(unsupported_value(value))
        }
    }
}

fn unsupported_field(value: &H8UnsupportedField) -> H8UnsupportedFieldJson {
    H8UnsupportedFieldJson {
        relative_path: value.relative_path.clone(),
        authored_order: value.authored_order,
        value: unsupported_value(&value.value),
    }
}

fn unsupported_value(value: &UnsupportedSourceValue) -> H8UnsupportedValueJson {
    H8UnsupportedValueJson {
        shape: match value.shape {
            UnsupportedSourceShape::Missing => "missing",
            UnsupportedSourceShape::Null => "null",
            UnsupportedSourceShape::String => "string",
            UnsupportedSourceShape::Number => "number",
            UnsupportedSourceShape::Boolean => "boolean",
            UnsupportedSourceShape::Array => "array",
            UnsupportedSourceShape::Object => "object",
        },
        value: value.value.clone(),
        reason: match value.reason {
            UnsupportedSourceReason::OpenVocabulary => "open_vocabulary",
            UnsupportedSourceReason::AmbiguousLegacyShape => "ambiguous_legacy_shape",
            UnsupportedSourceReason::InvalidPredicate => "invalid_predicate",
            UnsupportedSourceReason::NonCanonicalRuntimeValue => "non_canonical_runtime_value",
            UnsupportedSourceReason::SourceFieldDrift => "source_field_drift",
        },
    }
}

fn provenance(value: &H8Provenance) -> H8ProvenanceJson {
    H8ProvenanceJson {
        source_path: value.source_path.clone(),
        source_contract_version: value.source_contract_version.clone(),
        source_system_version: value.source_system_version.clone(),
        source_upstream_commit: value.source_upstream_commit.clone(),
    }
}

fn identity_stability(locator: &ContentChildLocator) -> &'static str {
    match locator.identity {
        ContentChildIdentity::Stable(_) => "stable_source_identity",
        ContentChildIdentity::Unstable { .. } => "unstable_authored_ordinal",
    }
}

fn content(value: &OwnedRichContent) -> Vec<H8ContentJson> {
    value
        .documents
        .iter()
        .map(|document| H8ContentJson {
            content_key: document.id.content_key.as_str().to_string(),
            child_locator: match &document.owner {
                ContentOwner::Child(locator) => Some(encode_content_child_locator(locator)),
                _ => None,
            },
            role: match document.role {
                crate::ContentRole::PrimaryDescription => "primary_description",
                crate::ContentRole::Summary => "summary",
                crate::ContentRole::SupplementalRules => "supplemental_rules",
                crate::ContentRole::EmbeddedCapability => "embedded_capability",
                crate::ContentRole::JournalPage => "journal_page",
                crate::ContentRole::TableResult => "table_result",
                crate::ContentRole::GeneratedNarrative => "generated_narrative",
                crate::ContentRole::Provenance => "provenance",
            },
            authored_order: document.authored_order,
            label: document.label.clone(),
            document: document.document.clone(),
            content_hash: document.content_hash.as_str().to_string(),
            relative_source_path: document.provenance.relative_source_path.clone(),
        })
        .collect()
}
