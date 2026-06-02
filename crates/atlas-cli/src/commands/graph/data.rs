use atlas_domain::DetailLevel;
use atlas_record::{RecordJsonOptions, record_json};
use atlas_search::{
    GraphContextEdge, GraphContextResult, GraphContextSection, RemasterLinksResult,
    VariantGroupResult,
};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub(super) struct GraphLinksData {
    pub(super) detail: String,
    pub(super) seed: GraphSeedJson,
    pub(super) outgoing: GraphSectionJson,
    pub(super) backlinks: GraphSectionJson,
}

#[derive(Debug, Serialize)]
pub(super) struct GraphUsesData {
    pub(super) detail: String,
    pub(super) seed: GraphSeedJson,
    pub(super) uses: GraphSectionJson,
}

#[derive(Debug, Serialize)]
pub(super) struct GraphVariantsData {
    pub(super) detail: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) seed: Option<GraphSeedJson>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) variant_group_key: Option<String>,
    pub(super) variants: Vec<GraphVariantJson>,
}

#[derive(Debug, Serialize)]
pub(super) struct GraphVariantJson {
    pub(super) record: atlas_record::RecordJson,
    pub(super) is_seed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) variant_label: Option<String>,
    pub(super) variant_axes: Vec<String>,
    pub(super) variant_source: String,
}

#[derive(Debug, Serialize)]
pub(super) struct GraphRemasterData {
    pub(super) detail: String,
    pub(super) seed: GraphSeedJson,
    pub(super) links: Vec<GraphRemasterLinkJson>,
}

#[derive(Debug, Serialize)]
pub(super) struct GraphRemasterLinkJson {
    pub(super) direction: &'static str,
    pub(super) remaster: atlas_record::RecordJson,
    pub(super) legacy: atlas_record::RecordJson,
    pub(super) source: GraphRemasterSourceJson,
}

#[derive(Debug, Serialize)]
pub(super) struct GraphRemasterSourceJson {
    pub(super) kind: String,
    pub(super) reference: String,
}

#[derive(Debug, Serialize)]
pub(super) struct GraphSeedJson {
    pub(super) record: atlas_record::RecordJson,
}

#[derive(Debug, Serialize)]
pub(super) struct GraphSectionJson {
    pub(super) records: Vec<atlas_record::RecordJson>,
    pub(super) edges: Vec<GraphEdgeJson>,
    pub(super) truncated: bool,
    pub(super) total_records: usize,
    pub(super) total_edges: usize,
}

#[derive(Debug, Serialize)]
pub(super) struct GraphEdgeJson {
    pub(super) from: String,
    pub(super) to: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) display_text: Option<String>,
    pub(super) reference_text: String,
    pub(super) source: GraphEdgeSourceJson,
}

#[derive(Debug, Serialize)]
pub(super) struct GraphEdgeSourceJson {
    pub(super) kind: String,
    pub(super) visibility: String,
}

pub(super) fn graph_links_data(result: &GraphContextResult, detail: DetailLevel) -> GraphLinksData {
    let options = RecordJsonOptions {
        detail,
        include_source_json: false,
    };
    GraphLinksData {
        detail: detail.to_string(),
        seed: GraphSeedJson {
            record: record_json(&result.seed, options),
        },
        outgoing: graph_section_json(&result.outgoing, options),
        backlinks: graph_section_json(&result.backlinks, options),
    }
}

pub(super) fn graph_uses_data(result: &GraphContextResult, detail: DetailLevel) -> GraphUsesData {
    let options = RecordJsonOptions {
        detail,
        include_source_json: false,
    };
    GraphUsesData {
        detail: detail.to_string(),
        seed: GraphSeedJson {
            record: record_json(&result.seed, options),
        },
        uses: graph_section_json(&result.backlinks, options),
    }
}

pub(super) fn graph_variants_data(
    result: &VariantGroupResult,
    detail: DetailLevel,
) -> GraphVariantsData {
    let options = RecordJsonOptions {
        detail,
        include_source_json: false,
    };
    GraphVariantsData {
        detail: detail.to_string(),
        seed: result.seed.as_ref().map(|seed| GraphSeedJson {
            record: record_json(seed, options),
        }),
        variant_group_key: result.variant_group_key.clone(),
        variants: result
            .variants
            .iter()
            .map(|record| GraphVariantJson {
                record: record_json(record, options),
                is_seed: result
                    .seed
                    .as_ref()
                    .is_some_and(|seed| record.identity.key == seed.identity.key),
                variant_label: record
                    .variant
                    .as_ref()
                    .and_then(|variant| variant.label.clone()),
                variant_axes: record
                    .variant
                    .as_ref()
                    .map(|variant| variant.axes.clone())
                    .unwrap_or_default(),
                variant_source: record
                    .variant
                    .as_ref()
                    .map(|variant| variant.source.as_str())
                    .unwrap_or("none")
                    .to_string(),
            })
            .collect(),
    }
}

pub(super) fn graph_remaster_data(
    result: &RemasterLinksResult,
    detail: DetailLevel,
) -> GraphRemasterData {
    let options = RecordJsonOptions {
        detail,
        include_source_json: false,
    };
    GraphRemasterData {
        detail: detail.to_string(),
        seed: GraphSeedJson {
            record: record_json(&result.seed, options),
        },
        links: result
            .links
            .iter()
            .map(|link| {
                let direction = if result.seed.identity.key == link.legacy_record.identity.key {
                    "legacy_to_remaster"
                } else {
                    "remaster_to_legacy"
                };
                GraphRemasterLinkJson {
                    direction,
                    remaster: record_json(&link.remaster_record, options),
                    legacy: record_json(&link.legacy_record, options),
                    source: GraphRemasterSourceJson {
                        kind: link.source.as_str().to_string(),
                        reference: link.source_ref.clone(),
                    },
                }
            })
            .collect(),
    }
}

fn graph_section_json(
    section: &GraphContextSection,
    options: RecordJsonOptions,
) -> GraphSectionJson {
    GraphSectionJson {
        records: section
            .records
            .iter()
            .map(|record| record_json(record, options))
            .collect(),
        edges: section.edges.iter().map(graph_edge_json).collect(),
        truncated: section.truncated,
        total_records: section.total_records,
        total_edges: section.total_edges,
    }
}

fn graph_edge_json(edge: &GraphContextEdge) -> GraphEdgeJson {
    GraphEdgeJson {
        from: edge.from.to_string(),
        to: edge.to.to_string(),
        display_text: edge.display_text.clone(),
        reference_text: edge.reference_text.clone(),
        source: GraphEdgeSourceJson {
            kind: edge.source.kind.clone(),
            visibility: edge.source.visibility.clone(),
        },
    }
}
