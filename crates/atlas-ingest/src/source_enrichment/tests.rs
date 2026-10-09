use super::*;
use crate::{LoadedFoundryDocument, LoadedFoundryPack};
use atlas_foundry_model::{SourceContext, admit_document_source, encode_snapshot};
use atlas_record::source_content::{ContentAudience, ContentVisibilityRule};
use serde_json::json;

fn document(value: serde_json::Value, path: &str) -> LoadedFoundryDocument {
    let bytes = serde_json::to_vec(&value).unwrap();
    LoadedFoundryDocument {
        provenance: SourceFileProvenance {
            pack_name: "test".into(),
            document_type: "Item".into(),
            source_path: path.into(),
        },
        content_hash: format!("hash:{path}"),
        admission: admit_document_source("Item", SourceContext::new(path, path, "$"), &bytes)
            .unwrap(),
        bytes,
    }
}
fn loaded(documents: Vec<LoadedFoundryDocument>) -> LoadedFoundrySource {
    LoadedFoundrySource {
        source_root: PathBuf::from("source"),
        manifest_path: PathBuf::from("source/module.json"),
        manifest_content_hash: "manifest-hash".into(),
        packs: vec![LoadedFoundryPack {
            name: "test".into(),
            label: "Test label".into(),
            document_type: "Item".into(),
            declared_path: "packs/test".into(),
            resolved_path: PathBuf::from("source/packs/test"),
            discovered_file_count: documents.len(),
            documents,
            quarantined_files: vec![],
            discovery_failure: None,
        }],
    }
}
fn context() -> SourceEnrichmentContext<'static> {
    SourceEnrichmentContext {
        audience: ContentAudience {
            include_gm: false,
            include_owner: false,
            implicit_check_dc: ContentVisibilityRule::Gm,
        },
        localization: None,
    }
}

#[test]
fn moves_model_and_preserves_exact_bytes_provenance_pack_context_and_diagnostics() {
    let document = document(
        json!({"_id":"aaaaaaaaaaaaaaaa","type":"spell","system":{"level":{"value":"3"},"description":{"value":"Useful @Localize[missing]"}}}),
        "packs/test/a.json",
    );
    let bytes = document.bytes.clone();
    let snapshot = encode_snapshot(document.admission.model.as_ref().unwrap()).unwrap();
    let diagnostics = document.admission.diagnostics.clone();
    let source = enrich_loaded_source(loaded(vec![document]), context());
    assert_eq!(source.packs[0].label, "Test label");
    assert_eq!(source.manifest_content_hash, "manifest-hash");
    let doc = &source.packs[0].documents[0];
    assert_eq!(doc.bytes, bytes);
    assert_eq!(doc.admission_diagnostics, diagnostics);
    assert_eq!(doc.content_hash, "hash:packs/test/a.json");
    assert_eq!(doc.provenance.source_path, "packs/test/a.json");
    let EnrichedDocumentOutcome::Addressed { record, .. } = &doc.outcome else {
        panic!();
    };
    assert_eq!(encode_snapshot(&record.source).unwrap(), snapshot);
    let report = source.report();
    assert_eq!(report.discovered_files, 1);
    assert_eq!(report.addressed_documents, 1);
    assert_eq!(report.partial_documents, 1);
    assert_eq!(report.content_diagnostics["UnresolvedLocalization"], 1);
}

#[test]
fn every_root_collision_is_unavailable_and_retains_its_typed_body() {
    let source = enrich_loaded_source(
        loaded(vec![
            document(json!({"_id":"aaaaaaaaaaaaaaaa","type":"spell"}), "a.json"),
            document(json!({"_id":"aaaaaaaaaaaaaaaa","type":"weapon"}), "b.json"),
        ]),
        context(),
    );
    for document in &source.packs[0].documents {
        assert!(matches!(
            &document.outcome,
            EnrichedDocumentOutcome::Unavailable {
                reason: SourceIdentityError::DuplicateId,
                source: Some(_)
            }
        ));
        assert!(!document.bytes.is_empty());
    }
    let report = source.report();
    assert_eq!(report.identity_unavailable["DuplicateId"], 2);
    assert_eq!(report.typed_documents, 2);
    assert_eq!(report.addressed_documents, 0);
}

#[test]
fn raw_only_invalid_id_and_quarantined_outcomes_partition_every_file() {
    let mut source = loaded(vec![
        document(
            json!({"type":"unknown","_id":"aaaaaaaaaaaaaaaa"}),
            "unknown.json",
        ),
        document(json!({"type":"spell","_id":"invalid"}), "invalid.json"),
        document(json!({"type":"spell"}), "missing.json"),
    ]);
    source.packs[0]
        .quarantined_files
        .push(QuarantinedSourceFile {
            failure: SourceLoadFailure {
                provenance: SourceFileProvenance {
                    pack_name: "test".into(),
                    document_type: "Item".into(),
                    source_path: "broken.json".into(),
                },
                stage: crate::SourceLoadFailureStage::Parse,
                message: "bad JSON".into(),
            },
            bytes: Some(b"broken".to_vec()),
            content_hash: Some("broken-hash".into()),
        });
    source.packs[0].discovered_file_count += 1;
    let source = enrich_loaded_source(source, context());
    let report = source.report();
    assert_eq!(
        report.discovered_files,
        report.retained_documents + report.quarantined_files
    );
    assert_eq!(report.raw_only_documents, 1);
    assert_eq!(report.typed_documents, 2);
    assert_eq!(report.identity_unavailable["InvalidId"], 1);
    assert_eq!(report.identity_unavailable["MissingId"], 1);
    assert_eq!(
        source.packs[0].quarantined_files[0].bytes.as_deref(),
        Some(b"broken".as_slice())
    );
}

#[test]
fn actual_loader_and_enrichment_account_for_fixture_files_without_source_loss() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/foundry-source/core-record-types");
    let loaded = crate::load_foundry_documents(&root, None).unwrap();
    let before = loaded
        .packs
        .iter()
        .flat_map(|p| p.documents.iter())
        .map(|d| (d.provenance.clone(), d.bytes.clone()))
        .collect::<Vec<_>>();
    let source = enrich_loaded_source(loaded, context());
    let report = source.report();
    assert_eq!(
        report.discovered_files,
        report.retained_documents + report.quarantined_files
    );
    let after = source
        .packs
        .iter()
        .flat_map(|p| p.documents.iter())
        .map(|d| (d.provenance.clone(), d.bytes.clone()))
        .collect::<Vec<_>>();
    assert_eq!(before, after);
}

#[test]
fn unsupported_root_also_reserves_its_authored_identity_against_collisions() {
    let source = enrich_loaded_source(
        loaded(vec![
            document(json!({"_id":"aaaaaaaaaaaaaaaa","type":"spell"}), "a.json"),
            document(
                json!({"_id":"aaaaaaaaaaaaaaaa","type":"unknown-family"}),
                "b.json",
            ),
        ]),
        context(),
    );
    assert_eq!(source.report().addressed_documents, 0);
    assert!(matches!(
        source.packs[0].documents[0].outcome,
        EnrichedDocumentOutcome::Unavailable {
            reason: SourceIdentityError::DuplicateId,
            ..
        }
    ));
    assert_eq!(source.report().raw_only_documents, 1);
}

#[test]
fn unavailable_roots_exclude_ambiguous_names_without_poisoning_exact_ids() {
    use atlas_record::source_content::{ContentReferenceResolution, ContentReferenceTarget};
    use atlas_record::source_record::SourceContentStatus;
    for unavailable in [
        json!({"_id":"bbbbbbbbbbbbbbbb","type":"unknown-family","name":"Same"}),
        json!({"type":"spell","name":"Same"}),
    ] {
        // Exercise both orders: a later addressed source must not overwrite
        // an exclusion, and a later exclusion must invalidate a prior alias.
        for reverse in [false, true] {
            let mut documents = vec![
                document(
                    json!({"_id":"aaaaaaaaaaaaaaaa","type":"spell","name":"Same","system":{"description":{"value":"@UUID[Compendium.pf2e.test.Item.Same] @UUID[Compendium.pf2e.test.Item.aaaaaaaaaaaaaaaa]"}}}),
                    "a.json",
                ),
                document(unavailable.clone(), "b.json"),
            ];
            if reverse {
                documents.reverse();
            }
            let source = enrich_loaded_source(loaded(documents), context());
            let content = source.packs[0]
                .documents
                .iter()
                .find_map(|d| match &d.outcome {
                    EnrichedDocumentOutcome::Addressed { content, .. } => Some(content),
                    _ => None,
                })
                .unwrap();
            let prepared = content
                .iter()
                .find_map(|c| match &c.status {
                    SourceContentStatus::Prepared(p) if p.references.len() == 2 => Some(p),
                    _ => None,
                })
                .unwrap();
            assert_eq!(
                prepared.references[0].resolution,
                ContentReferenceResolution::Unresolved
            );
            assert!(matches!(
                prepared.references[1].resolution,
                ContentReferenceResolution::Resolved(ContentReferenceTarget::Record { .. })
            ));
        }
    }
}

#[test]
fn addressed_handoff_keeps_sparse_outputs_separate_from_the_unchanged_record() {
    let document = document(
        json!({"_id":"aaaaaaaaaaaaaaaa","type":"spell","name":"Plain","system":{"description":{"value":""}}}),
        "a.json",
    );
    let snapshot = encode_snapshot(document.admission.model.as_ref().unwrap()).unwrap();
    let source = enrich_loaded_source(loaded(vec![document]), context());
    let EnrichedDocumentOutcome::Addressed {
        record,
        content,
        relationships,
    } = &source.packs[0].documents[0].outcome
    else {
        panic!("addressed");
    };
    assert_eq!(encode_snapshot(&record.source).unwrap(), snapshot);
    assert_eq!(content.len(), 1);
    assert_eq!(content[0].locator().field, "/system/description/value");
    assert!(
        matches!(&content[0].status,atlas_record::source_record::SourceContentStatus::Prepared(p) if p.html.is_empty() && p.text.is_empty())
    );
    assert!(relationships.is_empty());
    assert_eq!(source.report().content_outcomes.get("prepared"), Some(&1));
    assert!(!source.report().content_outcomes.contains_key("unavailable"));
}
