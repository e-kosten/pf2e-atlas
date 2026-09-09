use std::fs;

use atlas_record::ContentSourceKind;

use crate::document_input::hash_document_embedding_input;
use crate::document_renderer::{
    EmbeddingInputChunk, EmbeddingInputSection, render_embedding_chunks_for_embedding,
};
use crate::tokenization::TextEmbeddingTokenizer;
use crate::unit_kind::EmbeddingUnitKind;

use super::children::is_child_embedding_source;
use super::{
    apply_document_embedding_token_budget,
    apply_document_embedding_token_budget_with_diagnostic_jsonl,
};
use crate::document_units::model::{PendingDocumentEmbedding, PendingDocumentEmbeddingCandidate};

#[test]
fn child_embedding_sources_are_limited_to_rich_content() {
    assert!(is_child_embedding_source(ContentSourceKind::Description));
    assert!(is_child_embedding_source(
        ContentSourceKind::DetailsFieldDescription
    ));
    assert!(is_child_embedding_source(ContentSourceKind::PublicNotes));
    assert!(is_child_embedding_source(ContentSourceKind::JournalPage));
    assert!(is_child_embedding_source(ContentSourceKind::TableResult));
    assert!(!is_child_embedding_source(ContentSourceKind::Blurb));
    assert!(!is_child_embedding_source(ContentSourceKind::Routine));
}

#[test]
fn h8_child_units_materialize_only_when_their_parent_content_overflows() {
    for (source_kind, label, group_key) in [
        (
            ContentSourceKind::JournalPage,
            "Journal Page",
            "journal_page:0:content",
        ),
        (
            ContentSourceKind::TableResult,
            "Table Result",
            "table_result:0:text",
        ),
    ] {
        let tokenizer = TextEmbeddingTokenizer::whitespace_wordlevel_for_tests(40);
        let long_chunks = vec![
            identity_chunk(),
            rich_chunk(source_kind, group_key, label, 80),
        ];
        let long_text = render_embedding_chunks_for_embedding(&long_chunks);
        let child_key = format!("packs:test#{}", source_kind.as_str());
        let mut long_pending = vec![PendingDocumentEmbedding {
            embedding_unit_key: "packs:test#parent".to_string(),
            record_key: "packs:test".to_string(),
            unit_kind: EmbeddingUnitKind::Parent,
            label: None,
            source_kind: None,
            ordinal: 0,
            input_chunks: long_chunks,
            input_hash: hash_document_embedding_input(&long_text),
            input_text: long_text,
            child_candidates: vec![child_candidate(
                &child_key,
                source_kind,
                group_key,
                label,
                80,
            )],
        }];

        apply_document_embedding_token_budget(&mut long_pending, &tokenizer)
            .expect("H8 overflow budgeting should succeed");

        let child = long_pending
            .iter()
            .find(|entry| entry.embedding_unit_key == child_key)
            .expect("the impacted H8 child must materialize");
        assert_eq!(child.record_key, "packs:test");
        assert_eq!(child.source_kind, Some(source_kind));
        assert_eq!(child.label.as_deref(), Some(label));

        let short_chunks = vec![
            identity_chunk(),
            rich_chunk(source_kind, group_key, label, 4),
        ];
        let short_text = render_embedding_chunks_for_embedding(&short_chunks);
        let mut short_pending = vec![PendingDocumentEmbedding {
            embedding_unit_key: "packs:test#parent".to_string(),
            record_key: "packs:test".to_string(),
            unit_kind: EmbeddingUnitKind::Parent,
            label: None,
            source_kind: None,
            ordinal: 0,
            input_chunks: short_chunks,
            input_hash: hash_document_embedding_input(&short_text),
            input_text: short_text,
            child_candidates: vec![child_candidate(
                &child_key,
                source_kind,
                group_key,
                label,
                4,
            )],
        }];

        apply_document_embedding_token_budget(&mut short_pending, &tokenizer)
            .expect("short H8 budgeting should succeed");

        assert_eq!(short_pending.len(), 1);
        assert_eq!(short_pending[0].unit_kind, EmbeddingUnitKind::Parent);
        assert!(short_pending[0].child_candidates.is_empty());
    }
}

#[test]
fn token_budget_materializes_only_impacted_rich_child_groups() {
    let tokenizer = TextEmbeddingTokenizer::whitespace_wordlevel_for_tests(180);
    let input_chunks = vec![
        identity_chunk(),
        rich_chunk(
            ContentSourceKind::Description,
            "description:0:description",
            "Description accepted rich content",
            70,
        ),
        rich_chunk(
            ContentSourceKind::PublicNotes,
            "public_notes:1:public_notes",
            "Public notes impacted rich content",
            130,
        ),
        rich_chunk(
            ContentSourceKind::Blurb,
            "blurb:2:summary",
            "Summary non rich content",
            130,
        ),
    ];
    let input_text = render_embedding_chunks_for_embedding(&input_chunks);
    let mut pending = vec![PendingDocumentEmbedding {
        embedding_unit_key: "packs:test#parent".to_string(),
        record_key: "packs:test".to_string(),
        unit_kind: EmbeddingUnitKind::Parent,
        label: None,
        source_kind: None,
        ordinal: 0,
        input_chunks,
        input_hash: hash_document_embedding_input(&input_text),
        input_text,
        child_candidates: vec![
            child_candidate(
                "packs:test#heading_section:1",
                ContentSourceKind::Description,
                "description:0:description",
                "Description",
                70,
            ),
            child_candidate(
                "packs:test#heading_section:2",
                ContentSourceKind::PublicNotes,
                "public_notes:1:public_notes",
                "Public Notes",
                130,
            ),
            child_candidate(
                "packs:test#heading_section:3",
                ContentSourceKind::Blurb,
                "blurb:2:summary",
                "Summary",
                130,
            ),
        ],
    }];

    let telemetry = apply_document_embedding_token_budget(&mut pending, &tokenizer)
        .expect("budgeting should succeed");

    assert_eq!(
        pending
            .iter()
            .map(|entry| entry.embedding_unit_key.as_str())
            .collect::<Vec<_>>(),
        vec!["packs:test#parent", "packs:test#heading_section:2"]
    );
    assert!(
        pending
            .iter()
            .all(|entry| entry.child_candidates.is_empty())
    );
    let child = pending
        .iter()
        .find(|entry| entry.embedding_unit_key == "packs:test#heading_section:2")
        .expect("impacted public notes child materializes");
    assert_eq!(child.source_kind, Some(ContentSourceKind::PublicNotes));
    assert!(child.input_text.contains("Public Notes"));
    assert!(child.input_text.contains("token0"));
    assert!(child.input_text.contains("token129"));
    assert_eq!(telemetry.document_count, 2);
    assert_eq!(
        telemetry
            .record_truncation_coverage
            .records_with_child_units,
        1
    );
}

#[test]
fn token_budget_materializes_short_impacted_rich_child_groups() {
    let tokenizer = TextEmbeddingTokenizer::whitespace_wordlevel_for_tests(40);
    let input_chunks = vec![
        identity_chunk(),
        rich_chunk(
            ContentSourceKind::Description,
            "description:0:description",
            "Description",
            20,
        ),
        rich_chunk(
            ContentSourceKind::PublicNotes,
            "public_notes:1:public_notes",
            "Short Public Notes",
            20,
        ),
    ];
    let input_text = render_embedding_chunks_for_embedding(&input_chunks);
    let mut pending = vec![PendingDocumentEmbedding {
        embedding_unit_key: "packs:test#parent".to_string(),
        record_key: "packs:test".to_string(),
        unit_kind: EmbeddingUnitKind::Parent,
        label: None,
        source_kind: None,
        ordinal: 0,
        input_chunks,
        input_hash: hash_document_embedding_input(&input_text),
        input_text,
        child_candidates: vec![child_candidate(
            "packs:test#heading_section:1",
            ContentSourceKind::PublicNotes,
            "public_notes:1:public_notes",
            "Short Public Notes",
            20,
        )],
    }];

    apply_document_embedding_token_budget(&mut pending, &tokenizer)
        .expect("budgeting should succeed");

    assert_eq!(pending.len(), 2);
    assert!(
        pending
            .iter()
            .any(|entry| entry.embedding_unit_key == "packs:test#heading_section:1")
    );
}

#[test]
fn token_budget_writes_opt_in_chunk_diagnostics_jsonl() {
    let tokenizer = TextEmbeddingTokenizer::whitespace_wordlevel_for_tests(40);
    let input_chunks = vec![
        identity_chunk(),
        rich_chunk(
            ContentSourceKind::Description,
            "description:0:description",
            "Description",
            80,
        ),
    ];
    let input_text = render_embedding_chunks_for_embedding(&input_chunks);
    let mut pending = vec![PendingDocumentEmbedding {
        embedding_unit_key: "packs:test#parent".to_string(),
        record_key: "packs:test".to_string(),
        unit_kind: EmbeddingUnitKind::Parent,
        label: None,
        source_kind: None,
        ordinal: 0,
        input_chunks,
        input_hash: hash_document_embedding_input(&input_text),
        input_text,
        child_candidates: vec![child_candidate(
            "packs:test#heading_section:1",
            ContentSourceKind::Description,
            "description:0:description",
            "Description",
            80,
        )],
    }];
    let path = std::env::temp_dir().join(format!(
        "atlas-embedding-diagnostics-{}-{}.jsonl",
        std::process::id(),
        "token-budget"
    ));
    let _ = fs::remove_file(&path);

    apply_document_embedding_token_budget_with_diagnostic_jsonl(&mut pending, &tokenizer, &path)
        .expect("budgeting should write diagnostics");

    let contents = fs::read_to_string(&path).expect("diagnostics should be readable");
    let _ = fs::remove_file(&path);
    let lines = contents.lines().collect::<Vec<_>>();
    assert_eq!(lines.len(), 2);
    let parent: serde_json::Value =
        serde_json::from_str(lines[0]).expect("diagnostic JSON is valid");
    assert_eq!(parent["embedding_unit_key"], "packs:test#parent");
    assert_eq!(parent["record_key"], "packs:test");
    assert_eq!(parent["chunks"][1]["source_kind"], "description");
    assert_eq!(
        parent["chunks"][1]["group_key"],
        "description:0:description"
    );
    assert!(matches!(
        parent["chunks"][1]["outcome"].as_str(),
        Some("trimmed" | "dropped")
    ));
}

fn identity_chunk() -> EmbeddingInputChunk {
    EmbeddingInputChunk::line(EmbeddingInputSection::Identity, "Name: Test Record")
}

fn child_candidate(
    embedding_unit_key: &str,
    source_kind: ContentSourceKind,
    group_key: &str,
    label: &str,
    repeated_words: usize,
) -> PendingDocumentEmbeddingCandidate {
    PendingDocumentEmbeddingCandidate {
        embedding_unit_key: embedding_unit_key.to_string(),
        record_key: "packs:test".to_string(),
        unit_kind: EmbeddingUnitKind::HeadingSection,
        label: Some(label.to_string()),
        source_kind,
        group_key: group_key.to_string(),
        ordinal: 1,
        input_chunks: vec![
            identity_chunk(),
            rich_chunk(source_kind, group_key, label, repeated_words),
        ],
    }
}

fn rich_chunk(
    source_kind: ContentSourceKind,
    group_key: &str,
    label: &str,
    repeated_words: usize,
) -> EmbeddingInputChunk {
    EmbeddingInputChunk::truncatable_line(
        EmbeddingInputSection::Description,
        format!("{label}: {}", repeated_tokens(repeated_words)),
    )
    .with_source_kind(source_kind)
    .with_group_key(group_key)
}

fn repeated_tokens(count: usize) -> String {
    (0..count)
        .map(|index| format!("token{index}"))
        .collect::<Vec<_>>()
        .join(" ")
}
