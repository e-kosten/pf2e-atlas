use std::collections::BTreeMap;

use atlas_domain::{RecordFamily, RecordKey};
use atlas_record::{
    ContentBlock, ContentDocument, ContentInline, PresentationBadge, PresentationBadgeKind,
    PresentationBlock, PresentationFact, PresentationSection, PresentationSectionKind,
    PresentationText, RecordPresentationDocument,
};

use crate::document_input::hash_document_embedding_input;
use crate::document_renderer::{EmbeddingInputChunk, EmbeddingInputSection};
use crate::document_units::builder::pending_embedding_unit;
use crate::document_units::token_budget::summarize_document_embedding_tokenization;
use crate::document_units::{
    DocumentEmbeddingContentSource, DocumentEmbeddingRecordTruncationCoverage,
    DocumentEmbeddingSectionTruncation, DocumentEmbeddingSource,
    DocumentEmbeddingTruncationExample, DocumentEmbeddingUnitKindTruncation,
    PendingDocumentEmbedding, ReusableDocumentEmbedding, build_document_embedding_units,
    generate_document_embeddings_with_reuse_using,
    generate_document_embeddings_with_reuse_using_batch,
};
use crate::tokenization::{EmbeddingInputTokenization, EmbeddingSectionTruncation};
use crate::unit_kind::EmbeddingUnitKind;

#[test]
fn builds_pending_inputs_from_document_sources() {
    let pending = build_document_embedding_units(&[DocumentEmbeddingSource {
        record_key: "packs:visible1".to_string(),
        record_name: "Visible Record".to_string(),
        document: test_document("packs:visible1", "Visible Record"),
        aliases: vec!["Legacy Visible".to_string()],
        content_documents: Vec::new(),
    }]);

    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].record_key, "packs:visible1");
    assert_eq!(
        pending[0].input_text,
        "Name: Visible Record\n\
Family: Rule\n\
Type: Action\n\
Traits: Healing\n\
Classification: Fixture family\n\
Description: Fixture description.\n\
Families: Fixture family\n\
Publication Family: Unknown\n\
Pack: packs\n\
Foundry Document Type: Item\n\
Aliases: Legacy Visible"
    );
}

#[test]
fn content_documents_build_child_units() {
    let content_document = ContentDocument::new(vec![
        ContentBlock::Heading {
            level: 2,
            content: vec![ContentInline::Text {
                text: "Visible Record".to_string(),
            }],
        },
        paragraph_with_repeated_word("overview", 220),
        ContentBlock::Heading {
            level: 2,
            content: vec![ContentInline::Text {
                text: "Specific Section".to_string(),
            }],
        },
        paragraph_with_repeated_word("section", 220),
    ]);
    let pending = build_document_embedding_units(&[DocumentEmbeddingSource {
        record_key: "packs:visible1".to_string(),
        record_name: "Visible Record".to_string(),
        document: test_document("packs:visible1", "Visible Record"),
        aliases: Vec::new(),
        content_documents: vec![DocumentEmbeddingContentSource {
            source_kind: atlas_record::ContentSourceKind::Description,
            label: Some("Description".to_string()),
            document: content_document,
        }],
    }]);

    assert_eq!(pending.len(), 1);
    assert!(
        pending[0]
            .child_candidates
            .iter()
            .any(|entry| entry.embedding_unit_key == "packs:visible1#heading_section:1")
    );
}

#[test]
fn content_documents_preserve_leading_text_before_explicit_headings() {
    let content_document = ContentDocument::new(vec![
        ContentBlock::Paragraph {
            content: vec![ContentInline::Text {
                text: "Opening context before any heading.".to_string(),
            }],
        },
        ContentBlock::Heading {
            level: 2,
            content: vec![ContentInline::Text {
                text: "Specific Section".to_string(),
            }],
        },
        ContentBlock::Paragraph {
            content: vec![ContentInline::Text {
                text: "Section body.".to_string(),
            }],
        },
    ]);
    let pending = build_document_embedding_units(&[DocumentEmbeddingSource {
        record_key: "packs:visible1".to_string(),
        record_name: "Visible Record".to_string(),
        document: test_document("packs:visible1", "Visible Record"),
        aliases: Vec::new(),
        content_documents: vec![DocumentEmbeddingContentSource {
            source_kind: atlas_record::ContentSourceKind::Description,
            label: Some("Description".to_string()),
            document: content_document,
        }],
    }]);

    assert_eq!(pending.len(), 1);
    let parent_text =
        crate::document_renderer::render_embedding_chunks_for_embedding(&pending[0].input_chunks);
    assert!(parent_text.contains("Opening context before any heading."));
    assert!(parent_text.contains("Section body."));
    assert_eq!(pending[0].child_candidates.len(), 2);
    assert!(
        pending[0]
            .child_candidates
            .iter()
            .any(|entry| entry.label.as_deref() == Some("Description"))
    );
    assert!(
        pending[0]
            .child_candidates
            .iter()
            .any(|entry| entry.label.as_deref() == Some("Specific Section"))
    );
}

#[test]
fn content_documents_do_not_build_child_units_from_synthetic_sections() {
    let content_document = ContentDocument::new(vec![
        ContentBlock::Paragraph {
            content: vec![
                ContentInline::Strong {
                    content: vec![ContentInline::Text {
                        text: "Critical Success".to_string(),
                    }],
                },
                ContentInline::Text {
                    text: " You win.".to_string(),
                },
            ],
        },
        ContentBlock::Table {
            caption: Some(vec![ContentInline::Text {
                text: "Treasure by Level".to_string(),
            }]),
            headers: vec![vec![ContentInline::Text {
                text: "Level".to_string(),
            }]],
            rows: vec![vec![vec![ContentInline::Text {
                text: "1".to_string(),
            }]]],
        },
    ]);
    let pending = build_document_embedding_units(&[DocumentEmbeddingSource {
        record_key: "packs:visible1".to_string(),
        record_name: "Visible Record".to_string(),
        document: test_document("packs:visible1", "Visible Record"),
        aliases: Vec::new(),
        content_documents: vec![DocumentEmbeddingContentSource {
            source_kind: atlas_record::ContentSourceKind::Description,
            label: Some("Description".to_string()),
            document: content_document,
        }],
    }]);

    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].embedding_unit_key, "packs:visible1#parent");
}

#[test]
fn explicit_heading_child_units_include_synthetic_section_blocks() {
    let content_document = ContentDocument::new(vec![
        ContentBlock::Heading {
            level: 2,
            content: vec![ContentInline::Text {
                text: "Outcomes".to_string(),
            }],
        },
        ContentBlock::Paragraph {
            content: vec![ContentInline::Text {
                text: "Intro text.".to_string(),
            }],
        },
        ContentBlock::Paragraph {
            content: vec![
                ContentInline::Strong {
                    content: vec![ContentInline::Text {
                        text: "Critical Success".to_string(),
                    }],
                },
                ContentInline::Text {
                    text: " You win.".to_string(),
                },
            ],
        },
        ContentBlock::Table {
            caption: Some(vec![ContentInline::Text {
                text: "Treasure by Level".to_string(),
            }]),
            headers: vec![vec![ContentInline::Text {
                text: "Level".to_string(),
            }]],
            rows: vec![vec![vec![ContentInline::Text {
                text: "1".to_string(),
            }]]],
        },
        ContentBlock::Paragraph {
            content: vec![ContentInline::Text {
                text: "Follow-up text.".to_string(),
            }],
        },
    ]);
    let pending = build_document_embedding_units(&[DocumentEmbeddingSource {
        record_key: "packs:visible1".to_string(),
        record_name: "Visible Record".to_string(),
        document: test_document("packs:visible1", "Visible Record"),
        aliases: Vec::new(),
        content_documents: vec![DocumentEmbeddingContentSource {
            source_kind: atlas_record::ContentSourceKind::Description,
            label: Some("Description".to_string()),
            document: content_document,
        }],
    }]);

    assert_eq!(pending.len(), 1);
    let heading = pending[0]
        .child_candidates
        .iter()
        .find(|entry| entry.embedding_unit_key == "packs:visible1#heading_section:1")
        .expect("explicit heading child unit exists");
    assert_eq!(heading.label.as_deref(), Some("Outcomes"));
    let input_text =
        crate::document_renderer::render_embedding_chunks_for_embedding(&heading.input_chunks);
    assert!(input_text.contains("Critical Success You win."));
    assert!(input_text.contains("Treasure by Level"));
    let intro_index = input_text.find("Intro text.").expect("intro text");
    let outcome_index = heading
        .input_chunks
        .iter()
        .map(|chunk| chunk.text.as_str())
        .collect::<Vec<_>>()
        .join("\n")
        .find("Critical Success You win.")
        .expect("synthetic strong-lead text");
    let table_index = heading
        .input_chunks
        .iter()
        .map(|chunk| chunk.text.as_str())
        .collect::<Vec<_>>()
        .join("\n")
        .find("Treasure by Level")
        .expect("synthetic table text");
    let follow_up_index = heading
        .input_chunks
        .iter()
        .map(|chunk| chunk.text.as_str())
        .collect::<Vec<_>>()
        .join("\n")
        .find("Follow-up text.")
        .expect("follow-up text");
    assert!(intro_index < outcome_index);
    assert!(outcome_index < table_index);
    assert!(table_index < follow_up_index);
}

fn paragraph_with_repeated_word(word: &str, count: usize) -> ContentBlock {
    ContentBlock::Paragraph {
        content: vec![ContentInline::Text {
            text: std::iter::repeat_n(word, count)
                .collect::<Vec<_>>()
                .join(" "),
        }],
    }
}

#[test]
fn generates_document_vectors_from_pending_inputs() {
    let pending = vec![
        pending_embedding_with_hash("packs:first", "first input", "first-hash"),
        pending_embedding_with_hash("packs:second", "second input", "second-hash"),
    ];

    let generated = generate_document_embeddings_with_reuse_using(&pending, None, |input| {
        Ok::<_, std::convert::Infallible>(vec![input.len() as f32, 1.0])
    })
    .expect("fixture embedding should succeed")
    .embeddings;

    assert_eq!(generated.len(), 2);
    assert_eq!(generated[0].record_key, "packs:first");
    assert_eq!(generated[0].input_hash, "first-hash");
    assert_eq!(generated[0].dimensions, 2);
    assert_eq!(generated[0].vector, vec![11.0, 1.0]);
    assert_eq!(generated[1].record_key, "packs:second");
    assert_eq!(generated[1].input_hash, "second-hash");
    assert_eq!(generated[1].dimensions, 2);
    assert_eq!(generated[1].vector, vec![12.0, 1.0]);
}

#[test]
fn reuses_matching_document_vectors() {
    let pending = vec![
        pending_embedding_with_hash("packs:first", "first input", "first-hash"),
        pending_embedding_with_hash("packs:second", "second input", "second-hash"),
    ];
    let reusable = BTreeMap::from([(
        "packs:first#parent".to_string(),
        ReusableDocumentEmbedding {
            input_hash: "first-hash".to_string(),
            dimensions: 2,
            vector: vec![9.0, 1.0],
        },
    )]);

    let generated =
        generate_document_embeddings_with_reuse_using(&pending, Some(&reusable), |input| {
            Ok::<_, std::convert::Infallible>(vec![input.len() as f32, 1.0])
        })
        .expect("fixture embedding should succeed");

    assert_eq!(generated.reused_count, 1);
    assert_eq!(generated.generated_count, 1);
    assert_eq!(generated.embeddings[0].vector, vec![9.0, 1.0]);
    assert_eq!(generated.embeddings[1].vector, vec![12.0, 1.0]);
}

#[test]
fn generates_missing_document_vectors_in_batches() {
    let pending = vec![
        pending_embedding_with_hash("packs:first", "first input", "first-hash"),
        pending_embedding_with_hash("packs:second", "second input", "second-hash"),
        pending_embedding_with_hash("packs:third", "third input", "third-hash"),
    ];
    let mut batch_lengths = Vec::new();

    let generated =
        generate_document_embeddings_with_reuse_using_batch(&pending, None, 2, |inputs| {
            batch_lengths.push(inputs.len());
            Ok::<_, crate::EmbeddingError>(
                inputs
                    .iter()
                    .map(|input| vec![input.len() as f32, inputs.len() as f32])
                    .collect(),
            )
        })
        .expect("fixture embedding should succeed");

    assert_eq!(batch_lengths, vec![2, 1]);
    assert_eq!(generated.generated_count, 3);
    assert_eq!(
        generated
            .embeddings
            .iter()
            .map(|entry| entry.record_key.as_str())
            .collect::<Vec<_>>(),
        vec!["packs:first", "packs:second", "packs:third"]
    );
    assert_eq!(generated.embeddings[0].vector, vec![11.0, 2.0]);
    assert_eq!(generated.embeddings[2].vector, vec![11.0, 1.0]);
}

#[test]
fn summarizes_document_embedding_tokenization_truncation_examples() {
    let pending = vec![
        pending_embedding("packs:short", "short"),
        pending_embedding("packs:long", "long"),
        pending_embedding("packs:longer", "longer"),
        PendingDocumentEmbedding {
            embedding_unit_key: "packs:longer#heading_section:1".to_string(),
            record_key: "packs:longer".to_string(),
            unit_kind: EmbeddingUnitKind::HeadingSection,
            label: Some("Longer Section".to_string()),
            source_kind: Some(atlas_record::ContentSourceKind::Description),
            ordinal: 1,
            input_chunks: vec![EmbeddingInputChunk::line(
                EmbeddingInputSection::Description,
                "longer child",
            )],
            input_text: "longer child".to_string(),
            input_hash: hash_document_embedding_input("longer child"),
            child_candidates: Vec::new(),
        },
    ];
    let tokenizations = vec![
        EmbeddingInputTokenization {
            token_count: 8,
            max_token_count: Some(10),
            truncated: false,
        },
        EmbeddingInputTokenization {
            token_count: 12,
            max_token_count: Some(10),
            truncated: true,
        },
        EmbeddingInputTokenization {
            token_count: 20,
            max_token_count: Some(10),
            truncated: true,
        },
        EmbeddingInputTokenization {
            token_count: 9,
            max_token_count: Some(10),
            truncated: false,
        },
    ];

    let truncated_sections_by_unit = BTreeMap::from([(
        "packs:longer#parent".to_string(),
        vec![EmbeddingSectionTruncation {
            section: EmbeddingInputSection::Description,
            dropped_chunk_count: 1,
        }],
    )]);

    let telemetry = summarize_document_embedding_tokenization(
        &pending,
        &tokenizations,
        &truncated_sections_by_unit,
    );

    assert_eq!(telemetry.document_count, 4);
    assert_eq!(telemetry.truncated_document_count, 2);
    assert_eq!(telemetry.max_token_count, Some(10));
    assert_eq!(telemetry.max_observed_token_count, 20);
    assert_eq!(telemetry.total_observed_token_count, 49);
    assert_eq!(telemetry.total_tokens_over_limit, 12);
    let expected_truncated_examples = vec![
        DocumentEmbeddingTruncationExample {
            embedding_unit_key: "packs:longer#parent".to_string(),
            record_key: "packs:longer".to_string(),
            unit_kind: EmbeddingUnitKind::Parent,
            label: None,
            token_count: 20,
            max_token_count: 10,
            truncated_sections: vec!["description".to_string()],
        },
        DocumentEmbeddingTruncationExample {
            embedding_unit_key: "packs:long#parent".to_string(),
            record_key: "packs:long".to_string(),
            unit_kind: EmbeddingUnitKind::Parent,
            label: None,
            token_count: 12,
            max_token_count: 10,
            truncated_sections: Vec::new(),
        },
    ];
    assert_eq!(
        telemetry.unit_kind_truncations,
        vec![DocumentEmbeddingUnitKindTruncation {
            unit_kind: "parent".to_string(),
            unit_count: 2,
            record_count: 2,
            total_tokens_over_limit: 12,
            max_observed_token_count: 20,
            examples: expected_truncated_examples.clone(),
        }]
    );
    assert_eq!(
        telemetry.record_truncation_coverage,
        DocumentEmbeddingRecordTruncationCoverage {
            record_count: 3,
            records_with_child_units: 1,
            records_with_any_truncated_unit: 2,
            records_with_truncated_parent_unit: 2,
            records_with_truncated_child_unit: 0,
            records_with_truncated_parent_and_child_units: 1,
            records_with_truncated_parent_and_all_child_units_fit: 1,
            records_with_truncated_parent_without_child_units: 1,
        }
    );
    assert_eq!(
        telemetry.section_truncations,
        vec![DocumentEmbeddingSectionTruncation {
            section: "description".to_string(),
            document_count: 1,
            dropped_chunk_count: 1,
        }]
    );
    assert_eq!(telemetry.truncated_examples, expected_truncated_examples);
}

fn pending_embedding(record_key: &str, input_text: &str) -> PendingDocumentEmbedding {
    pending_embedding_with_hash(
        record_key,
        input_text,
        &hash_document_embedding_input(input_text),
    )
}

fn pending_embedding_with_hash(
    record_key: &str,
    input_text: &str,
    input_hash: &str,
) -> PendingDocumentEmbedding {
    pending_embedding_unit(
        format!("{record_key}#parent"),
        record_key.to_string(),
        EmbeddingUnitKind::Parent,
        None,
        None,
        0,
        vec![EmbeddingInputChunk::line(
            EmbeddingInputSection::Identity,
            input_text,
        )],
    )
    .tap_input_hash(input_hash)
}

trait PendingEmbeddingTestExt {
    fn tap_input_hash(self, input_hash: &str) -> Self;
}

impl PendingEmbeddingTestExt for PendingDocumentEmbedding {
    fn tap_input_hash(mut self, input_hash: &str) -> Self {
        self.input_hash = input_hash.to_string();
        self
    }
}

fn test_document(key: &str, name: &str) -> RecordPresentationDocument {
    RecordPresentationDocument {
        record_key: RecordKey::parse(key).expect("fixture key is valid"),
        record_family: RecordFamily::Rule,
        title: name.to_string(),
        identity: vec![
            PresentationFact {
                key: "family".to_string(),
                label: "Family".to_string(),
                value: "Rule".to_string(),
            },
            PresentationFact {
                key: "type".to_string(),
                label: "Type".to_string(),
                value: "Action".to_string(),
            },
        ],
        badges: vec![
            PresentationBadge {
                kind: PresentationBadgeKind::Trait,
                label: "Trait".to_string(),
                value: "Healing".to_string(),
            },
            PresentationBadge {
                kind: PresentationBadgeKind::Classification,
                label: "Classification".to_string(),
                value: "Fixture family".to_string(),
            },
        ],
        sections: vec![
            PresentationSection::new(
                PresentationSectionKind::Description,
                vec![PresentationBlock::Prose(PresentationText {
                    text: "Fixture description.".to_string(),
                })],
            ),
            PresentationSection::new(
                PresentationSectionKind::Classification,
                vec![PresentationBlock::FactList(vec![
                    PresentationFact {
                        key: "families".to_string(),
                        label: "Families".to_string(),
                        value: "Fixture family".to_string(),
                    },
                    PresentationFact {
                        key: "publication_family".to_string(),
                        label: "Publication Family".to_string(),
                        value: "Unknown".to_string(),
                    },
                    PresentationFact {
                        key: "pack".to_string(),
                        label: "Pack".to_string(),
                        value: "packs".to_string(),
                    },
                    PresentationFact {
                        key: "foundry_document_type".to_string(),
                        label: "Foundry Document Type".to_string(),
                        value: "Item".to_string(),
                    },
                ])],
            ),
        ],
    }
}
