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
        ContentSourceKind::DetailsDescription
    ));
    assert!(is_child_embedding_source(ContentSourceKind::PublicNotes));
    assert!(!is_child_embedding_source(ContentSourceKind::Blurb));
    assert!(!is_child_embedding_source(ContentSourceKind::Routine));
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
