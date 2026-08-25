use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RetrievalPredicateInventoryEntry {
    pub id: &'static str,
    pub base_paths: &'static [&'static str],
    pub base_predicate: &'static str,
    pub target_disposition: &'static str,
    pub later_owner: &'static str,
    pub fixture: &'static str,
    pub validation_checkpoint: &'static str,
}

pub(super) fn retrieval_predicate_inventory() -> Vec<RetrievalPredicateInventoryEntry> {
    vec![
        RetrievalPredicateInventoryEntry {
            id: "generated_affliction_source_count",
            base_paths: &["crates/atlas-ingest/src/source_pipeline.rs"],
            base_predicate: "RecordVisibility::visible_by_default selects generated source-instance counts.",
            target_disposition: "GeneratedAfflictionRole canonical/source_instance owns count and pack routing; visibility remains metadata.",
            later_owner: "B4",
            fixture: "generated-afflictions/ghoul: one canonical, one source instance, three relationships",
            validation_checkpoint: "B4 focused construction tests; D3 residue audit",
        },
        RetrievalPredicateInventoryEntry {
            id: "generated_affliction_role_construction",
            base_paths: &[
                "crates/atlas-ingest/src/generated/afflictions/mod.rs",
                "crates/atlas-ingest/src/generated/afflictions/records.rs",
            ],
            base_predicate: "is_default_visible selects canonical versus source-instance identity, visibility, and pack label.",
            target_disposition: "Explicit generated role and typed host-instance-canonical relationships replace visibility-as-role.",
            later_owner: "B4",
            fixture: "generated-afflictions/ghoul: one canonical, one source instance, three relationships",
            validation_checkpoint: "B4 construction; C1 round trip; D1 retrieval; D3 audit",
        },
        RetrievalPredicateInventoryEntry {
            id: "ingest_record_visibility",
            base_paths: &["crates/atlas-ingest/src/records/visibility.rs"],
            base_predicate: "record.visibility.visible_by_default plus legacy-remaster suppression determines ordinary eligibility.",
            target_disposition: "Explicit GM-complete product retrieval disposition with rationale-bearing legacy duplicate control.",
            later_owner: "D1",
            fixture: "GM/private useful record plus linked legacy/remaster pair",
            validation_checkpoint: "D1 focused eligibility tests; D3 source/search audit",
        },
        RetrievalPredicateInventoryEntry {
            id: "ingest_analysis_catalog_counts",
            base_paths: &["crates/atlas-ingest/src/report.rs"],
            base_predicate: "default-visible records alone contribute analysis counts and metric value catalogs.",
            target_disposition: "Analysis and catalogs derive from explicit product disposition, never visibility classification.",
            later_owner: "B6 then D1",
            fixture: "useful non-default-visible creature facts",
            validation_checkpoint: "B6 canonical projection tests; D1/D3 coverage audit",
        },
        RetrievalPredicateInventoryEntry {
            id: "ingest_embedding_eligibility",
            base_paths: &["crates/atlas-ingest/src/embeddings.rs"],
            base_predicate: "default-visible records alone receive document embedding units.",
            target_disposition: "GM-complete explicit retrieval disposition; copied capability prose excluded only for duplicate control.",
            later_owner: "D1",
            fixture: "GM/private authored content plus copied embedded capability prose",
            validation_checkpoint: "D1 embedding coverage tests; D3 corpus audit",
        },
        RetrievalPredicateInventoryEntry {
            id: "content_participation",
            base_paths: &["crates/atlas-record/src/content/**"],
            base_predicate: "ContentVisibility::Public determines searchable/default participation.",
            target_disposition: "Useful authored content participates regardless of classification; exclusions carry non-auth rationales.",
            later_owner: "B5",
            fixture: "public, GM, private, internal, and copied embedded content",
            validation_checkpoint: "B5 owner/retrieval tests; D3 content audit",
        },
        RetrievalPredicateInventoryEntry {
            id: "reference_graph_policy",
            base_paths: &["crates/atlas-record/src/reference_policy.rs"],
            base_predicate: "Default graph and backlinks use ContentVisibility::Public.",
            target_disposition: "Typed visibility is preserved while useful references participate; copied capability edges may use duplicate-control mode.",
            later_owner: "B5 then D1",
            fixture: "public, GM/private, and copied embedded reference occurrences",
            validation_checkpoint: "B5 reference ownership tests; D1 graph tests; D3 audit",
        },
        RetrievalPredicateInventoryEntry {
            id: "artifact_generation_and_validation",
            base_paths: &[
                "crates/atlas-index/src/write/visibility.rs",
                "crates/atlas-index/src/write/sqlite/records.rs",
                "crates/atlas-index/src/write/sqlite/discovery_catalogs/**",
                "crates/atlas-index/src/write/sqlite/metric_catalogs.rs",
                "crates/atlas-index/src/artifact/validation/**",
                "crates/atlas-index/src/inspect.rs",
            ],
            base_predicate: "is_default_visible controls FTS, embedding, discovery, metric, relationship coverage, inspection, and validation expectations.",
            target_disposition: "Artifact rows and validation use typed product disposition plus rationale identity and generated role.",
            later_owner: "C1",
            fixture: "GM-complete eligibility matrix plus generated-afflictions/ghoul",
            validation_checkpoint: "C1 atomic round trip, corruption, inspection, and validation suite",
        },
        RetrievalPredicateInventoryEntry {
            id: "artifact_read_keysets_and_graph",
            base_paths: &[
                "crates/atlas-index/src/read/search/filters/**",
                "crates/atlas-index/src/read/discovery/**",
                "crates/atlas-index/src/read/graph/product.rs",
            ],
            base_predicate: "is_default_visible and public-only relationships constrain filter keysets, discovery, graph, and variants.",
            target_disposition: "Read projections lower the explicit GM-complete product policy without inventing authorization.",
            later_owner: "D1",
            fixture: "non-default-visible useful records/content/references",
            validation_checkpoint: "D1 read/search/graph/discovery tests; D3 audit",
        },
        RetrievalPredicateInventoryEntry {
            id: "search_resolution",
            base_paths: &["crates/atlas-search/src/records/resolution.rs"],
            base_predicate: "visible_by_default retains candidate records and aliases during fallback resolution.",
            target_disposition: "Exact and alias resolution use canonical explicit eligibility while retaining named legacy duplicate control.",
            later_owner: "D1",
            fixture: "GM/private exact and alias candidates plus legacy/remaster pair",
            validation_checkpoint: "D1 resolution tests; D3 CLI/agent audit",
        },
        RetrievalPredicateInventoryEntry {
            id: "downstream_cli_app_ui_projection",
            base_paths: &[
                "crates/atlas-cli/src/**",
                "crates/atlas-app-model/src/**",
                "crates/atlas-app-service/src/**",
                "web/atlas-ui/src/**",
            ],
            base_predicate: "Downstream projections inherit pinned-base retrieval suppression even where they do not repeat the predicate locally.",
            target_disposition: "CLI RecordJson, app DTOs/service, and UI project shared GM-complete canonical truth without classification-only filtering.",
            later_owner: "D2, E3, F1/F2",
            fixture: "hashed GM-complete creature surface matrix",
            validation_checkpoint: "D3 pre-UI; E3 app; F3 browser/runtime; Checkpoint E visual; G2 final",
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inventory_covers_every_checkpoint_b_base_owner() {
        let inventory = retrieval_predicate_inventory();
        let paths = inventory
            .iter()
            .flat_map(|entry| entry.base_paths.iter().copied())
            .collect::<Vec<_>>();
        for required in [
            "crates/atlas-ingest/src/source_pipeline.rs",
            "crates/atlas-ingest/src/generated/afflictions/mod.rs",
            "crates/atlas-ingest/src/generated/afflictions/records.rs",
            "crates/atlas-ingest/src/records/visibility.rs",
            "crates/atlas-ingest/src/report.rs",
            "crates/atlas-ingest/src/embeddings.rs",
            "crates/atlas-record/src/reference_policy.rs",
            "crates/atlas-index/src/write/visibility.rs",
            "crates/atlas-search/src/records/resolution.rs",
            "crates/atlas-cli/src/**",
            "crates/atlas-app-service/src/**",
            "web/atlas-ui/src/**",
        ] {
            assert!(
                paths.contains(&required),
                "missing inventory path {required}"
            );
        }
        assert!(inventory.iter().all(|entry| {
            !entry.target_disposition.is_empty()
                && !entry.later_owner.is_empty()
                && !entry.fixture.is_empty()
                && !entry.validation_checkpoint.is_empty()
        }));
    }
}
