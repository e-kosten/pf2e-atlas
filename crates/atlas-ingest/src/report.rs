//! Developer-only source preparation census; no heuristic product pipeline.
use crate::{
    EnrichedDocumentOutcome, IngestError, SourceEnrichmentContext, SourceEnrichmentReport,
    enrich_loaded_source, load_foundry_documents,
};
use crate::{build::default_audience, source::localization::LocalizationCatalog};
use atlas_record::source_record::select_record_search;
use serde::Serialize;
use std::path::Path;
#[derive(Debug, Serialize)]
pub struct SourceAnalysisReport {
    pub source: SourceEnrichmentReport,
    pub product_records: usize,
    pub selected_fields: usize,
    pub selected_sections: usize,
    pub selected_body_bytes: usize,
    pub lexical_definitions: usize,
    pub indexing_locale: String,
}
pub fn analyze_foundry_source(
    root: &Path,
    manifest: Option<&Path>,
) -> Result<SourceAnalysisReport, IngestError> {
    let catalog = LocalizationCatalog::load(root, "en")?;
    let source = enrich_loaded_source(
        load_foundry_documents(root, manifest)?,
        SourceEnrichmentContext {
            audience: default_audience(),
            localization: Some(&catalog),
        },
    );
    let mut report = SourceAnalysisReport {
        source: source.report(),
        product_records: 0,
        selected_fields: 0,
        selected_sections: 0,
        selected_body_bytes: 0,
        lexical_definitions: 0,
        indexing_locale: catalog.locale,
    };
    for pack in &source.packs {
        for document in &pack.documents {
            if let EnrichedDocumentOutcome::Addressed {
                record, content, ..
            } = &document.outcome
            {
                let selection = select_record_search(
                    record,
                    content,
                    default_audience(),
                    &pack.metadata.label,
                    &Default::default(),
                )
                .map_err(|e| IngestError::SourceSelectionFailed(e.to_string()))?;
                report.product_records += usize::from(!selection.identities.is_empty());
                report.selected_fields += selection.fields.len();
                for field in &selection.fields {
                    report.selected_sections += field.sections.len();
                    report.selected_body_bytes += field
                        .sections
                        .iter()
                        .map(|section| section.text.len())
                        .sum::<usize>();
                    report.lexical_definitions += field
                        .sections
                        .iter()
                        .filter(|section| section.is_lexical_definition())
                        .count();
                }
            }
        }
    }
    Ok(report)
}
