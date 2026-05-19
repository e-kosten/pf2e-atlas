use atlas_domain::RecordKey;
use serde_json::Value;

use crate::records::{JournalPageFact, JournalPageSkipReason, SkippedJournalPageFact};

use super::{
    ContentParseDiagnostics, normalize_text, parse_foundry_content, pointer_string, string_field,
};

pub(super) fn extract_journal_page_facts(
    raw: &Value,
    host_record_key: &RecordKey,
) -> (
    Vec<JournalPageFact>,
    Vec<SkippedJournalPageFact>,
    Vec<ContentParseDiagnostics>,
) {
    let Some(pages) = raw.pointer("/pages").and_then(Value::as_array) else {
        return (Vec::new(), Vec::new(), Vec::new());
    };
    let mut facts = Vec::new();
    let mut skipped = Vec::new();
    let mut diagnostics = Vec::new();
    for (index, page) in pages.iter().enumerate() {
        let name = string_field(page, "name").unwrap_or_else(|| format!("Page {}", index + 1));
        let page_id = string_field(page, "_id");
        let Some(markup) = pointer_string(page, "/text/content") else {
            skipped.push(skipped_journal_page(
                host_record_key,
                page_id,
                name,
                index,
                JournalPageSkipReason::MissingTextContent,
            ));
            continue;
        };
        if markup.trim().is_empty() {
            skipped.push(skipped_journal_page(
                host_record_key,
                page_id,
                name,
                index,
                JournalPageSkipReason::EmptyTextContent,
            ));
            continue;
        }
        let parsed = parse_foundry_content(&markup);
        if parsed.document.is_empty() {
            skipped.push(skipped_journal_page(
                host_record_key,
                page_id,
                name,
                index,
                JournalPageSkipReason::EmptyParsedDocument,
            ));
            continue;
        }
        diagnostics.push(parsed.diagnostics.clone());
        facts.push(JournalPageFact {
            host_record_key: host_record_key.clone(),
            page_id,
            normalized_name: normalize_text(&name),
            source_ref: format!("journal:{name}"),
            name,
            ordinal: index as i64,
            source_markup: markup,
            document: parsed.document,
        });
    }
    (facts, skipped, diagnostics)
}

fn skipped_journal_page(
    host_record_key: &RecordKey,
    page_id: Option<String>,
    name: String,
    index: usize,
    reason: JournalPageSkipReason,
) -> SkippedJournalPageFact {
    SkippedJournalPageFact {
        host_record_key: host_record_key.clone(),
        page_id,
        normalized_name: normalize_text(&name),
        name,
        ordinal: index as i64,
        reason,
    }
}
