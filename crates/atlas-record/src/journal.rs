use atlas_domain::RecordKey;
use serde::{Deserialize, Serialize};

use crate::{FactValue, OwnedRichContent, RichDocument, UnsupportedSourceValue};

pub type H8Fact<T> = FactValue<H8FieldValue<T>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum H8FieldValue<T> {
    Known(T),
    Unsupported(UnsupportedSourceValue),
}

impl<T> H8FieldValue<T> {
    pub fn known(&self) -> Option<&T> {
        match self {
            Self::Known(value) => Some(value),
            Self::Unsupported(_) => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SourceDocumentId(String);

impl SourceDocumentId {
    pub fn new(value: impl Into<String>) -> Result<Self, InvalidH8Value> {
        let value = value.into();
        if value.is_empty() || value.chars().any(char::is_whitespace) {
            return Err(InvalidH8Value);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MediaLocator(String);

impl MediaLocator {
    pub fn new(value: impl Into<String>) -> Result<Self, InvalidH8Value> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(InvalidH8Value);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidH8Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct H8ExactSourceObject {
    pub compact_json: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct H8Number {
    pub canonical: String,
}

impl H8Number {
    pub fn as_f64(&self) -> Option<f64> {
        self.canonical.parse().ok()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct H8UnsupportedField {
    pub relative_path: String,
    pub authored_order: u32,
    pub value: UnsupportedSourceValue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct H8RecordSourceMetadata {
    pub folder: H8Fact<SourceDocumentId>,
    pub sort: H8Fact<i64>,
    pub ownership: H8Fact<H8ExactSourceObject>,
    pub flags: H8Fact<H8ExactSourceObject>,
    pub stats: H8Fact<H8ExactSourceObject>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct H8PageSourceMetadata {
    pub ownership: H8Fact<H8ExactSourceObject>,
    pub flags: H8Fact<H8ExactSourceObject>,
    pub stats: H8Fact<H8ExactSourceObject>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableResultSourceMetadata {
    pub flags: H8Fact<H8ExactSourceObject>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContentChildKind {
    JournalPage,
    TableResult,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum ContentChildIdentity {
    Stable(SourceDocumentId),
    Unstable { source_ordinal: u32 },
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentChildLocator {
    pub parent: RecordKey,
    pub kind: ContentChildKind,
    pub identity: ContentChildIdentity,
}

/// Encodes the complete parent-scoped locator used by canonical persistence and machine JSON.
pub fn encode_content_child_locator(locator: &ContentChildLocator) -> String {
    let kind = match locator.kind {
        ContentChildKind::JournalPage => "j",
        ContentChildKind::TableResult => "t",
    };
    let (stability, identity) = match &locator.identity {
        ContentChildIdentity::Stable(id) => ("s", hex(id.as_str().as_bytes())),
        ContentChildIdentity::Unstable { source_ordinal } => ("u", source_ordinal.to_string()),
    };
    format!(
        "v1~{kind}~{stability}~{}~{identity}",
        hex(locator.parent.to_string().as_bytes())
    )
}

pub fn decode_content_child_locator(value: &str) -> Result<ContentChildLocator, InvalidH8Value> {
    let mut parts = value.split('~');
    let (Some("v1"), Some(kind), Some(stability), Some(parent), Some(identity), None) = (
        parts.next(),
        parts.next(),
        parts.next(),
        parts.next(),
        parts.next(),
        parts.next(),
    ) else {
        return Err(InvalidH8Value);
    };
    let kind = match kind {
        "j" => ContentChildKind::JournalPage,
        "t" => ContentChildKind::TableResult,
        _ => return Err(InvalidH8Value),
    };
    let parent = String::from_utf8(unhex(parent)?).map_err(|_| InvalidH8Value)?;
    let parent = RecordKey::parse(&parent).map_err(|_| InvalidH8Value)?;
    let identity = match stability {
        "s" => ContentChildIdentity::Stable(SourceDocumentId::new(
            String::from_utf8(unhex(identity)?).map_err(|_| InvalidH8Value)?,
        )?),
        "u" => ContentChildIdentity::Unstable {
            source_ordinal: identity.parse().map_err(|_| InvalidH8Value)?,
        },
        _ => return Err(InvalidH8Value),
    };
    Ok(ContentChildLocator {
        parent,
        kind,
        identity,
    })
}

/// Encodes the parent-relative child selector used by record routes and app views.
///
/// The route already carries the parent record key, so repeating it inside the opaque
/// selector adds length without adding an ownership check. The app service reconstructs
/// the full locator from the authenticated route parent and still verifies that the
/// resulting child belongs to the loaded canonical body.
pub fn encode_content_child_selector(locator: &ContentChildLocator) -> String {
    let kind = match locator.kind {
        ContentChildKind::JournalPage => "j",
        ContentChildKind::TableResult => "t",
    };
    let (stability, identity) = match &locator.identity {
        ContentChildIdentity::Stable(id) => ("s", hex(id.as_str().as_bytes())),
        ContentChildIdentity::Unstable { source_ordinal } => ("u", source_ordinal.to_string()),
    };
    format!("v1~{kind}~{stability}~{identity}")
}

pub fn decode_content_child_selector(
    parent: RecordKey,
    value: &str,
) -> Result<ContentChildLocator, InvalidH8Value> {
    let mut parts = value.split('~');
    let (Some("v1"), Some(kind), Some(stability), Some(identity), None) = (
        parts.next(),
        parts.next(),
        parts.next(),
        parts.next(),
        parts.next(),
    ) else {
        return Err(InvalidH8Value);
    };
    let kind = match kind {
        "j" => ContentChildKind::JournalPage,
        "t" => ContentChildKind::TableResult,
        _ => return Err(InvalidH8Value),
    };
    let identity = match stability {
        "s" => {
            let bytes = unhex(identity)?;
            if hex(&bytes) != identity {
                return Err(InvalidH8Value);
            }
            ContentChildIdentity::Stable(SourceDocumentId::new(
                String::from_utf8(bytes).map_err(|_| InvalidH8Value)?,
            )?)
        }
        "u" => {
            let source_ordinal: u32 = identity.parse().map_err(|_| InvalidH8Value)?;
            if source_ordinal.to_string() != identity {
                return Err(InvalidH8Value);
            }
            ContentChildIdentity::Unstable { source_ordinal }
        }
        _ => return Err(InvalidH8Value),
    };
    Ok(ContentChildLocator {
        parent,
        kind,
        identity,
    })
}

fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(DIGITS[usize::from(byte >> 4)] as char);
        output.push(DIGITS[usize::from(byte & 0x0f)] as char);
    }
    output
}

fn unhex(value: &str) -> Result<Vec<u8>, InvalidH8Value> {
    if !value.len().is_multiple_of(2) {
        return Err(InvalidH8Value);
    }
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let high = digit(pair[0])?;
            let low = digit(pair[1])?;
            Ok((high << 4) | low)
        })
        .collect()
}

fn digit(value: u8) -> Result<u8, InvalidH8Value> {
    match value {
        b'0'..=b'9' => Ok(value - b'0'),
        b'a'..=b'f' => Ok(value - b'a' + 10),
        _ => Err(InvalidH8Value),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct H8Identity {
    pub record_key: RecordKey,
    pub source_id: SourceDocumentId,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct H8Provenance {
    pub source_path: String,
    pub source_contract_version: String,
    pub source_system_version: String,
    pub source_upstream_commit: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JournalPageKind {
    Text,
    Image,
    Pdf,
    Video,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JournalPageTitle {
    pub show: H8Fact<bool>,
    pub level: H8Fact<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JournalPageText {
    pub content: H8Fact<RichDocument>,
    pub format: H8Fact<i64>,
    pub markdown: H8Fact<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JournalPageVideo {
    pub controls: H8Fact<bool>,
    pub loop_playback: H8Fact<bool>,
    pub autoplay: H8Fact<bool>,
    pub volume: H8Fact<H8Number>,
    pub timestamp: H8Fact<H8Number>,
    pub width: H8Fact<H8Number>,
    pub height: H8Fact<H8Number>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JournalPage {
    pub locator: ContentChildLocator,
    pub source_id: H8Fact<SourceDocumentId>,
    pub source_ordinal: u32,
    pub name: H8Fact<String>,
    pub page_kind: H8Fact<JournalPageKind>,
    pub sort: H8Fact<i64>,
    pub title: H8Fact<JournalPageTitle>,
    pub text: H8Fact<JournalPageText>,
    pub source: H8Fact<MediaLocator>,
    pub image_source: H8Fact<H8ExactSourceObject>,
    pub image_caption: H8Fact<String>,
    pub video: H8Fact<JournalPageVideo>,
    pub source_system: H8Fact<H8ExactSourceObject>,
    pub source_metadata: H8PageSourceMetadata,
    pub unsupported_fields: Vec<H8UnsupportedField>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct H8UnsupportedChild {
    pub locator: ContentChildLocator,
    pub source_id: H8Fact<SourceDocumentId>,
    pub source_ordinal: u32,
    pub exact_source: H8ExactSourceObject,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JournalPageEntry {
    Page(Box<JournalPage>),
    Unsupported(H8UnsupportedChild),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JournalRecord {
    pub identity: H8Identity,
    pub pages: H8Fact<Vec<JournalPageEntry>>,
    pub source_metadata: H8RecordSourceMetadata,
    pub content: OwnedRichContent,
    pub unsupported_fields: Vec<H8UnsupportedField>,
    pub provenance: H8Provenance,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TableResultKind {
    Text,
    Pack,
    Document,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableResultTarget {
    pub collection: H8Fact<String>,
    pub document_id: H8Fact<SourceDocumentId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableResultRange {
    pub first: i64,
    pub last: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableResult {
    pub locator: ContentChildLocator,
    pub source_id: H8Fact<SourceDocumentId>,
    pub source_ordinal: u32,
    pub result_kind: H8Fact<TableResultKind>,
    pub text: H8Fact<RichDocument>,
    pub target: TableResultTarget,
    pub weight: H8Fact<H8Number>,
    pub range: H8Fact<TableResultRange>,
    pub drawn: H8Fact<bool>,
    pub image: H8Fact<MediaLocator>,
    pub source_metadata: TableResultSourceMetadata,
    pub unsupported_fields: Vec<H8UnsupportedField>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TableResultEntry {
    Result(Box<TableResult>),
    Unsupported(H8UnsupportedChild),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RollTableRecord {
    pub identity: H8Identity,
    pub description: H8Fact<RichDocument>,
    pub results: H8Fact<Vec<TableResultEntry>>,
    pub formula: H8Fact<String>,
    pub replacement: H8Fact<bool>,
    pub display_roll: H8Fact<bool>,
    pub image: H8Fact<MediaLocator>,
    pub source_metadata: H8RecordSourceMetadata,
    pub content: OwnedRichContent,
    pub unsupported_fields: Vec<H8UnsupportedField>,
    pub provenance: H8Provenance,
}

#[cfg(test)]
mod tests {
    use atlas_domain::RecordKey;

    use super::{
        ContentChildIdentity, ContentChildKind, ContentChildLocator, SourceDocumentId,
        decode_content_child_locator, decode_content_child_selector, encode_content_child_locator,
        encode_content_child_selector,
    };

    #[test]
    fn h8_opaque_child_locators_round_trip_stable_and_unstable_identities() {
        let parent = RecordKey::parse("journals:hero-points").expect("record key");
        for locator in [
            ContentChildLocator {
                parent: parent.clone(),
                kind: ContentChildKind::JournalPage,
                identity: ContentChildIdentity::Stable(
                    SourceDocumentId::new("page-id").expect("source ID"),
                ),
            },
            ContentChildLocator {
                parent,
                kind: ContentChildKind::TableResult,
                identity: ContentChildIdentity::Unstable { source_ordinal: 17 },
            },
        ] {
            let encoded = encode_content_child_locator(&locator);
            assert_eq!(decode_content_child_locator(&encoded), Ok(locator));
        }
    }

    #[test]
    fn h8_opaque_child_locators_reject_unowned_or_noncanonical_encodings() {
        for value in [
            "",
            "v2~j~s~6a6f75726e616c733a6865726f~70616765",
            "v1~x~s~6a6f75726e616c733a6865726f~70616765",
            "v1~j~x~6a6f75726e616c733a6865726f~70616765",
            "v1~j~s~not-hex~70616765",
            "v1~j~s~6a6f75726e616c733a6865726f~70616765~extra",
        ] {
            assert!(decode_content_child_locator(value).is_err(), "{value}");
        }
    }

    #[test]
    fn h8_route_child_selectors_omit_the_parent_and_reconstruct_it_from_the_route() {
        let parent = RecordKey::parse("rollable-tables:hero-points").expect("record key");
        let locator = ContentChildLocator {
            parent: parent.clone(),
            kind: ContentChildKind::TableResult,
            identity: ContentChildIdentity::Stable(
                SourceDocumentId::new("E1cjgAqFZIzCjDuU").expect("source ID"),
            ),
        };

        let selector = encode_content_child_selector(&locator);
        assert_eq!(selector, "v1~t~s~4531636a674171465a497a436a447555");
        assert!(!selector.contains("rollable-tables"));
        assert_eq!(
            decode_content_child_selector(parent.clone(), &selector),
            Ok(locator.clone())
        );
        let other_parent = RecordKey::parse("rollable-tables:other").expect("record key");
        let rebound = decode_content_child_selector(other_parent.clone(), &selector)
            .expect("selector is bound by its route parent");
        assert_eq!(rebound.parent, other_parent);
        assert_eq!(rebound.kind, locator.kind);
        assert_eq!(rebound.identity, locator.identity);
    }

    #[test]
    fn h8_route_child_selectors_reject_full_locators_and_noncanonical_encodings() {
        let parent = RecordKey::parse("journals:hero-points").expect("record key");
        for value in [
            "",
            "v2~j~s~70616765",
            "v1~x~s~70616765",
            "v1~j~x~70616765",
            "v1~j~s~not-hex",
            "v1~j~s~7061676A",
            "v1~j~u~01",
            "v1~j~s~70616765~extra",
            "v1~j~s~6a6f75726e616c733a6865726f~70616765",
        ] {
            assert!(
                decode_content_child_selector(parent.clone(), value).is_err(),
                "{value}"
            );
        }
    }
}
