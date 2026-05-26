use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use atlas_record::{
    ContentReference, ContentReferenceLocator, NormalizedRecord, ReferenceEdge, ReferenceEdgeFacts,
    ReferenceGraphMode, RemasterLink, SupplementalContentDocument, iter_content_references,
    reference_edge_matches_mode,
};
use graphqlite::Graph;

use crate::graph_projection::evidence::{EvidenceUnit, evidence_units};
use crate::{IndexBuildInput, IndexWriteError};

const PROJECTION_VERSION: &str = "atlas-graphqlite-graph/v1";

type GraphNode = (String, Vec<(&'static str, String)>, &'static str);
type GraphEdge = (String, String, Vec<(&'static str, String)>, &'static str);

#[derive(Debug, Clone, Copy)]
struct ProjectionCounts {
    nodes: usize,
    edges: usize,
}

pub(super) fn write_graphqlite_projection(
    path: &Path,
    input: &IndexBuildInput<'_>,
) -> Result<(), IndexWriteError> {
    let graph = Graph::open(path).map_err(write_error)?;
    let evidence_units = evidence_units(&input.records);
    let nodes = graph_nodes(input, &evidence_units);
    let id_map = graph.insert_nodes_bulk(nodes).map_err(write_error)?;
    let edges = graph_edges(input, &evidence_units);
    let edge_count = graph
        .insert_edges_bulk(edges, &id_map)
        .map_err(write_error)?;
    let counts = ProjectionCounts {
        nodes: id_map.len(),
        edges: edge_count,
    };
    write_projection_metadata(&graph, counts)
}

fn graph_nodes(input: &IndexBuildInput<'_>, evidence_units: &[EvidenceUnit]) -> Vec<GraphNode> {
    let mut nodes = Vec::new();
    let mut traits = BTreeSet::new();
    let mut publications = BTreeMap::new();
    let mut taxonomy_families = BTreeSet::new();
    let mut variant_groups = BTreeMap::new();
    let mut variant_axes = BTreeSet::new();

    nodes.extend(input.packs.iter().map(pack_node));

    for record in &input.records {
        nodes.push(record_node(record));
        for trait_value in &record.traits {
            traits.insert(trait_value.clone());
        }
        if let Some(title) = &record.publication_title {
            publications
                .entry(publication_node_id(
                    title,
                    record.publication_family.as_str(),
                    record.publication_remaster,
                ))
                .or_insert_with(|| {
                    (
                        title.clone(),
                        record.publication_family.as_str().to_string(),
                        record.publication_remaster,
                    )
                });
        }
        for family in &record.taxonomy_families {
            taxonomy_families.insert(family.clone());
        }
        for (ordinal, supplemental) in record.supplemental_content.iter().enumerate() {
            nodes.push(content_unit_node(record, ordinal, supplemental));
        }
        if let Some(group_key) = &record.variant_group_key {
            variant_groups
                .entry(group_key.clone())
                .or_insert_with(|| record.variant_base_name.clone().unwrap_or_default());
        }
        for axis in &record.variant_axes {
            variant_axes.insert(axis.clone());
        }
    }
    nodes.extend(evidence_units.iter().map(evidence_unit_node));

    nodes.extend(traits.into_iter().map(trait_node));
    nodes.extend(
        publications
            .into_values()
            .map(|(title, family, remaster)| publication_node(&title, &family, remaster)),
    );
    nodes.extend(taxonomy_families.into_iter().map(taxonomy_family_node));
    nodes.extend(
        variant_groups
            .into_iter()
            .map(|(group_key, base_name)| variant_group_node(&group_key, &base_name)),
    );
    nodes.extend(variant_axes.into_iter().map(variant_axis_node));
    nodes
}

fn graph_edges(input: &IndexBuildInput<'_>, evidence_units: &[EvidenceUnit]) -> Vec<GraphEdge> {
    let mut edges = Vec::new();
    let record_keys: BTreeSet<String> = input
        .records
        .iter()
        .map(|record| record.key.to_string())
        .collect();
    for record in &input.records {
        let record_id = record_node_id(record);
        edges.push((
            record_id.clone(),
            pack_node_id(record.pack_name.as_str()),
            vec![("source", "pack_name".to_string())],
            "IN_PACK",
        ));
        if let Some(title) = &record.publication_title {
            edges.push((
                record_id.clone(),
                publication_node_id(
                    title,
                    record.publication_family.as_str(),
                    record.publication_remaster,
                ),
                vec![("source", "publication_title".to_string())],
                "PUBLISHED_IN",
            ));
        }
        for trait_value in &record.traits {
            edges.push((
                record_id.clone(),
                trait_node_id(trait_value),
                vec![("source", "record_traits".to_string())],
                "HAS_TRAIT",
            ));
        }
        for family in &record.taxonomy_families {
            edges.push((
                record_id.clone(),
                taxonomy_family_node_id(family),
                vec![("source", "taxonomy_families".to_string())],
                "HAS_TAXONOMY_FAMILY",
            ));
        }
        for (ordinal, supplemental) in record.supplemental_content.iter().enumerate() {
            let content_unit_id = content_unit_node_id(record, ordinal);
            edges.push((
                record_id.clone(),
                content_unit_id.clone(),
                vec![
                    ("source_kind", supplemental.source_kind.as_str().to_string()),
                    ("visibility", supplemental.visibility.as_str().to_string()),
                    (
                        "contributes_to_references",
                        supplemental.contributes_to_references.to_string(),
                    ),
                ],
                "HAS_CONTENT_UNIT",
            ));
            edges.extend(content_mention_edges(
                &content_unit_id,
                supplemental,
                &record_keys,
            ));
        }
        if let Some(group_key) = &record.variant_group_key {
            edges.push((
                record_id.clone(),
                variant_group_node_id(group_key),
                vec![
                    ("source", "variant_group_key".to_string()),
                    ("variant_source", record.variant_source.clone()),
                ],
                "VARIANT_OF",
            ));
        }
        for axis in &record.variant_axes {
            edges.push((
                record_id.clone(),
                variant_axis_node_id(axis),
                vec![("source", "variant_axes".to_string())],
                "HAS_VARIANT_AXIS",
            ));
        }
    }
    for unit in evidence_units {
        edges.push((
            record_node_id_from_key(&unit.record_key),
            evidence_unit_node_id(&unit.evidence_unit_key),
            vec![("source", "evidence_units".to_string())],
            "HAS_EVIDENCE_UNIT",
        ));
        edges.extend(evidence_reference_edges(unit, &record_keys));
    }
    edges.extend(input.references.iter().map(reference_edge));
    edges.extend(input.remaster_links.iter().map(remaster_edge));
    edges
}

fn record_node(record: &NormalizedRecord) -> GraphNode {
    (
        record_node_id(record),
        vec![
            ("record_key", record.key.to_string()),
            ("name", record.name.clone()),
            ("normalized_name", record.normalized_name.clone()),
            ("family", record.record_family.as_str().to_string()),
            ("pack_name", record.pack_name.as_str().to_string()),
            ("default_visible", record.is_default_visible.to_string()),
        ],
        "Record",
    )
}

fn pack_node(pack: &crate::IndexBuildPack<'_>) -> GraphNode {
    (
        pack_node_id(pack.name.as_str()),
        vec![
            ("name", pack.name.to_string()),
            ("label", pack.label.to_string()),
            ("document_type", pack.document_type.to_string()),
            ("declared_path", pack.declared_path.to_string()),
            ("record_count", pack.record_count.to_string()),
        ],
        "Pack",
    )
}

fn trait_node(value: String) -> GraphNode {
    (
        trait_node_id(&value),
        vec![("value", value), ("field", "traits".to_string())],
        "Trait",
    )
}

fn publication_node(title: &str, family: &str, remaster: bool) -> GraphNode {
    (
        publication_node_id(title, family, remaster),
        vec![
            ("title", title.to_string()),
            ("family", family.to_string()),
            ("remaster", remaster.to_string()),
        ],
        "Publication",
    )
}

fn taxonomy_family_node(value: String) -> GraphNode {
    (
        taxonomy_family_node_id(&value),
        vec![("value", value)],
        "TaxonomyFamily",
    )
}

fn content_unit_node(
    record: &NormalizedRecord,
    ordinal: usize,
    supplemental: &SupplementalContentDocument,
) -> GraphNode {
    let content_key = content_unit_node_id(record, ordinal);
    (
        content_key.clone(),
        vec![
            ("content_key", content_key),
            ("record_key", record.key.to_string()),
            ("source_kind", supplemental.source_kind.as_str().to_string()),
            ("visibility", supplemental.visibility.as_str().to_string()),
            ("label", supplemental.label.clone().unwrap_or_default()),
        ],
        "ContentUnit",
    )
}

fn evidence_unit_node(unit: &EvidenceUnit) -> GraphNode {
    (
        evidence_unit_node_id(&unit.evidence_unit_key),
        vec![
            ("evidence_unit_key", unit.evidence_unit_key.clone()),
            ("record_key", unit.record_key.clone()),
            (
                "source_content_unit_key",
                unit.source_content_unit_key.clone().unwrap_or_default(),
            ),
            ("source_kind", unit.source_kind.as_str().to_string()),
            ("visibility", unit.visibility.as_str().to_string()),
            ("unit_kind", unit.unit_kind.clone()),
            ("label", unit.label.clone().unwrap_or_default()),
            ("ordinal", unit.ordinal.to_string()),
            ("search_text", unit.search_text.clone()),
        ],
        "EvidenceUnit",
    )
}

fn variant_axis_node(axis: String) -> GraphNode {
    (
        variant_axis_node_id(&axis),
        vec![("value", axis)],
        "VariantAxis",
    )
}

fn variant_group_node(group_key: &str, base_name: &str) -> GraphNode {
    (
        variant_group_node_id(group_key),
        vec![
            ("key", group_key.to_string()),
            ("base_name", base_name.to_string()),
        ],
        "VariantGroup",
    )
}

fn evidence_reference_edges(unit: &EvidenceUnit, record_keys: &BTreeSet<String>) -> Vec<GraphEdge> {
    let mut mentioned_records = BTreeSet::new();
    for reference in iter_content_references(&unit.document) {
        if let Some(target_key) = &reference.resolved_key {
            let target_key = target_key.to_string();
            if record_keys.contains(&target_key) {
                mentioned_records.insert((
                    target_key,
                    reference_display_text(reference),
                    reference_text(reference),
                ));
            }
        }
    }

    mentioned_records
        .into_iter()
        .map(|(target_key, display_text, reference_text)| {
            (
                evidence_unit_node_id(&unit.evidence_unit_key),
                record_node_id_from_key(&target_key),
                vec![
                    ("source_kind", unit.source_kind.as_str().to_string()),
                    ("visibility", unit.visibility.as_str().to_string()),
                    ("display_text", display_text),
                    ("reference_text", reference_text),
                ],
                "EVIDENCE_REFERENCES",
            )
        })
        .collect()
}

fn content_mention_edges(
    content_unit_id: &str,
    supplemental: &SupplementalContentDocument,
    record_keys: &BTreeSet<String>,
) -> Vec<GraphEdge> {
    let mut mentioned_records = BTreeSet::new();
    for reference in iter_content_references(&supplemental.document) {
        if let Some(target_key) = &reference.resolved_key {
            let target_key = target_key.to_string();
            if record_keys.contains(&target_key) {
                mentioned_records.insert((target_key, reference_display_text(reference)));
            }
        }
    }

    mentioned_records
        .into_iter()
        .map(|(target_key, display_text)| {
            (
                content_unit_id.to_string(),
                record_node_id_from_key(&target_key),
                vec![
                    ("source_kind", supplemental.source_kind.as_str().to_string()),
                    ("visibility", supplemental.visibility.as_str().to_string()),
                    ("display_text", display_text),
                ],
                "MENTIONS",
            )
        })
        .collect()
}

fn reference_display_text(reference: &ContentReference) -> String {
    reference
        .resolved_name
        .clone()
        .or_else(|| reference.resolved_key.as_ref().map(ToString::to_string))
        .unwrap_or_default()
}

fn reference_text(reference: &ContentReference) -> String {
    match &reference.locator {
        ContentReferenceLocator::FoundryUuid { raw_target }
        | ContentReferenceLocator::Compendium { raw_target } => raw_target.clone(),
        ContentReferenceLocator::PackAndLocator { pack_name, locator } => {
            format!("{pack_name}:{locator}")
        }
        ContentReferenceLocator::Unknown { raw } => raw.clone(),
    }
}

fn reference_edge(reference: &ReferenceEdge) -> GraphEdge {
    let default_graph = reference_edge_matches_mode(
        ReferenceEdgeFacts {
            source_kind: reference.source_kind,
            visibility: reference.visibility,
        },
        ReferenceGraphMode::Default,
    );

    (
        record_node_id_from_key(&reference.from_record_key.to_string()),
        record_node_id_from_key(&reference.to_record_key.to_string()),
        vec![
            ("source", "reference_edges".to_string()),
            ("source_kind", reference.source_kind.as_str().to_string()),
            ("visibility", reference.visibility.as_str().to_string()),
            ("reference_text", reference.reference_text.clone()),
            (
                "display_text",
                reference.display_text.clone().unwrap_or_default(),
            ),
            ("default_graph", default_graph.to_string()),
        ],
        "REFERENCES",
    )
}

fn remaster_edge(link: &RemasterLink) -> GraphEdge {
    (
        record_node_id_from_key(&link.legacy_record_key.to_string()),
        record_node_id_from_key(&link.remaster_record_key.to_string()),
        vec![
            ("source", link.source.as_str().to_string()),
            ("source_ref", link.source_ref.clone()),
        ],
        "REMASTERED_BY",
    )
}

fn write_projection_metadata(
    graph: &Graph,
    counts: ProjectionCounts,
) -> Result<(), IndexWriteError> {
    let metadata = [
        (
            "graphqlite_projection_version",
            PROJECTION_VERSION.to_string(),
        ),
        ("graphqlite_node_count", counts.nodes.to_string()),
        ("graphqlite_edge_count", counts.edges.to_string()),
    ];
    let connection = graph.connection().sqlite_connection();
    for (key, value) in metadata {
        connection
            .execute(
                "INSERT OR REPLACE INTO artifact_metadata (key, value) VALUES (?1, ?2)",
                (key, value),
            )
            .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
    }
    Ok(())
}

fn record_node_id(record: &NormalizedRecord) -> String {
    record_node_id_from_key(&record.key.to_string())
}

fn record_node_id_from_key(record_key: &str) -> String {
    format!("record:{record_key}")
}

fn pack_node_id(pack_name: &str) -> String {
    format!("pack:{pack_name}")
}

fn trait_node_id(value: &str) -> String {
    format!("trait:{value}")
}

fn publication_node_id(title: &str, family: &str, remaster: bool) -> String {
    format!("publication:{family}:{remaster}:{title}")
}

fn taxonomy_family_node_id(value: &str) -> String {
    format!("taxonomy-family:{value}")
}

fn content_unit_node_id(record: &NormalizedRecord, ordinal: usize) -> String {
    format!("content:{}#{ordinal}", record.key)
}

fn evidence_unit_node_id(evidence_unit_key: &str) -> String {
    format!("evidence-unit:{evidence_unit_key}")
}

fn variant_axis_node_id(axis: &str) -> String {
    format!("variant-axis:{axis}")
}

fn variant_group_node_id(group_key: &str) -> String {
    format!("variant-group:{group_key}")
}

fn write_error(error: graphqlite::Error) -> IndexWriteError {
    IndexWriteError::WriteFailed(format!("GraphQLite projection write failed: {error}"))
}
