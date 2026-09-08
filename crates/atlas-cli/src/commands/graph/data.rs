use atlas_domain::DetailLevel;
use atlas_record::RecordJsonOptions;
use atlas_search::{
    GraphContextEdge, GraphContextResult, GraphContextSection, RemasterLinksResult,
    VariantGroupResult,
};
use serde::Serialize;

use crate::client::AtlasClient;
use crate::commands::record::context::project_record;

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

pub(super) fn graph_links_data(
    client: &impl AtlasClient,
    result: &GraphContextResult,
    detail: DetailLevel,
) -> Result<GraphLinksData, atlas_app_model::AppError> {
    let options = RecordJsonOptions {
        detail,
        include_source_json: false,
    };
    Ok(GraphLinksData {
        detail: detail.to_string(),
        seed: GraphSeedJson {
            record: project_record(client, &result.seed, options)?,
        },
        outgoing: graph_section_json(client, &result.outgoing, options)?,
        backlinks: graph_section_json(client, &result.backlinks, options)?,
    })
}

pub(super) fn graph_uses_data(
    client: &impl AtlasClient,
    result: &GraphContextResult,
    detail: DetailLevel,
) -> Result<GraphUsesData, atlas_app_model::AppError> {
    let options = RecordJsonOptions {
        detail,
        include_source_json: false,
    };
    Ok(GraphUsesData {
        detail: detail.to_string(),
        seed: GraphSeedJson {
            record: project_record(client, &result.seed, options)?,
        },
        uses: graph_section_json(client, &result.backlinks, options)?,
    })
}

pub(super) fn graph_variants_data(
    client: &impl AtlasClient,
    result: &VariantGroupResult,
    detail: DetailLevel,
) -> Result<GraphVariantsData, atlas_app_model::AppError> {
    let options = RecordJsonOptions {
        detail,
        include_source_json: false,
    };
    Ok(GraphVariantsData {
        detail: detail.to_string(),
        seed: result
            .seed
            .as_ref()
            .map(|seed| {
                project_record(client, seed, options).map(|record| GraphSeedJson { record })
            })
            .transpose()?,
        variant_group_key: result.variant_group_key.clone(),
        variants: result
            .variants
            .iter()
            .map(|record| {
                Ok(GraphVariantJson {
                    record: project_record(client, record, options)?,
                    is_seed: result
                        .seed
                        .as_ref()
                        .is_some_and(|seed| record.record.identity.key == seed.record.identity.key),
                    variant_label: record
                        .record
                        .variant
                        .as_ref()
                        .and_then(|variant| variant.label.clone()),
                    variant_axes: record
                        .record
                        .variant
                        .as_ref()
                        .map(|variant| variant.axes.clone())
                        .unwrap_or_default(),
                    variant_source: record
                        .record
                        .variant
                        .as_ref()
                        .map(|variant| variant.source.as_str())
                        .unwrap_or("none")
                        .to_string(),
                })
            })
            .collect::<Result<Vec<_>, atlas_app_model::AppError>>()?,
    })
}

pub(super) fn graph_remaster_data(
    client: &impl AtlasClient,
    result: &RemasterLinksResult,
    detail: DetailLevel,
) -> Result<GraphRemasterData, atlas_app_model::AppError> {
    let options = RecordJsonOptions {
        detail,
        include_source_json: false,
    };
    Ok(GraphRemasterData {
        detail: detail.to_string(),
        seed: GraphSeedJson {
            record: project_record(client, &result.seed, options)?,
        },
        links: result
            .links
            .iter()
            .map(|link| {
                let direction =
                    if result.seed.record.identity.key == link.legacy_record.record.identity.key {
                        "legacy_to_remaster"
                    } else {
                        "remaster_to_legacy"
                    };
                Ok(GraphRemasterLinkJson {
                    direction,
                    remaster: project_record(client, &link.remaster_record, options)?,
                    legacy: project_record(client, &link.legacy_record, options)?,
                    source: GraphRemasterSourceJson {
                        kind: link.source.as_str().to_string(),
                        reference: link.source_ref.clone(),
                    },
                })
            })
            .collect::<Result<Vec<_>, atlas_app_model::AppError>>()?,
    })
}

fn graph_section_json(
    client: &impl AtlasClient,
    section: &GraphContextSection,
    options: RecordJsonOptions,
) -> Result<GraphSectionJson, atlas_app_model::AppError> {
    Ok(GraphSectionJson {
        records: section
            .records
            .iter()
            .map(|record| project_record(client, record, options))
            .collect::<Result<Vec<_>, atlas_app_model::AppError>>()?,
        edges: section.edges.iter().map(graph_edge_json).collect(),
        truncated: section.truncated,
        total_records: section.total_records,
        total_edges: section.total_edges,
    })
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
