//! Snapshot-local passage navigation shared by lexical and semantic consumers.
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ts_rs::TS)]
#[serde(deny_unknown_fields)]
pub struct SourceByteRange {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error("source passage range is inverted, out of bounds, or not on UTF-8 boundaries")]
pub struct InvalidSourceByteRange;

impl SourceByteRange {
    /// Ranges are verified against their source rather than trusted after decode.
    pub fn extract(self, text: &str) -> Result<&str, InvalidSourceByteRange> {
        text.get(self.start..self.end).ok_or(InvalidSourceByteRange)
    }
}

/// Root, owner chain and field belong to the outer hit/field locator. Hashes bind
/// sections to one source/preparation context; these are not cross-build IDs.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, ts_rs::TS)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum SourcePassageAddress {
    Identity {},
    HtmlSection {
        prepared_html_sha256: String,
        canonical_text_sha256: String,
        selection_version: String,
        section_ordinal: usize,
        label: Option<String>,
        chunk_bytes: SourceByteRange,
    },
    PlainSection {
        source_text_sha256: String,
        selection_version: String,
        section_ordinal: usize,
        label: Option<String>,
        chunk_bytes: SourceByteRange,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn byte_ranges_require_source_bounds_and_utf8_boundaries() {
        let text = "a🐉é";
        assert_eq!(SourceByteRange { start: 1, end: 5 }.extract(text), Ok("🐉"));
        for range in [
            SourceByteRange { start: 2, end: 5 },
            SourceByteRange { start: 5, end: 4 },
            SourceByteRange { start: 0, end: 8 },
        ] {
            assert_eq!(range.extract(text), Err(InvalidSourceByteRange));
        }
    }

    #[test]
    fn identity_has_no_fabricated_range_and_rejects_extra_fields() {
        assert_eq!(
            serde_json::to_string(&SourcePassageAddress::Identity {}).unwrap(),
            "{\"kind\":\"identity\"}"
        );
        assert!(
            serde_json::from_str::<SourcePassageAddress>(
                r#"{"kind":"identity","chunk_bytes":{"start":0,"end":1}}"#
            )
            .is_err()
        );
    }
}
