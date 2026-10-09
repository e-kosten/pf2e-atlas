use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::Serialize;

use super::model::{LoadedFoundrySource, SourceLoadFailure};
use atlas_foundry_model::SourceDiagnostic;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct SourceDocumentCounts {
    pub discovered_files: usize,
    pub retained_documents: usize,
    pub modeled_documents: usize,
    pub diagnostic_free_documents: usize,
    pub partial_documents: usize,
    pub raw_only_documents: usize,
    pub quarantined_files: usize,
    pub unavailable_packs: usize,
    pub diagnostics: usize,
}

/// Compact inspection of loading outcomes, without serializing the full DTOs or
/// raw corpus. Diagnostics and failures are complete, not sampled or suppressed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceLoadingReport {
    pub source_root: PathBuf,
    pub manifest_path: PathBuf,
    pub manifest_content_hash: String,
    pub pack_count: usize,
    pub counts: SourceDocumentCounts,
    pub by_document_type: BTreeMap<String, SourceDocumentCounts>,
    pub diagnostics: Vec<SourceDiagnostic>,
    pub failures: Vec<SourceLoadFailure>,
}

impl LoadedFoundrySource {
    pub fn report(&self) -> SourceLoadingReport {
        let mut counts = SourceDocumentCounts::default();
        let mut by_document_type = BTreeMap::<String, SourceDocumentCounts>::new();
        let mut diagnostics = Vec::new();
        let mut failures = Vec::new();
        for pack in &self.packs {
            let group = by_document_type
                .entry(pack.document_type.clone())
                .or_default();
            for counts in [&mut counts, group] {
                counts.discovered_files += pack.discovered_file_count;
                counts.retained_documents += pack.documents.len();
                counts.quarantined_files += pack.quarantined_files.len();
                counts.unavailable_packs += usize::from(pack.discovery_failure.is_some());
                for document in &pack.documents {
                    let admission = &document.admission;
                    counts.diagnostics += admission.diagnostics.len();
                    if admission.model.is_none() {
                        counts.raw_only_documents += 1;
                    } else {
                        counts.modeled_documents += 1;
                        if admission.diagnostics.is_empty() {
                            counts.diagnostic_free_documents += 1;
                        } else {
                            counts.partial_documents += 1;
                        }
                    }
                }
            }
            diagnostics.extend(
                pack.documents
                    .iter()
                    .flat_map(|document| document.admission.diagnostics.clone()),
            );
            failures.extend(pack.discovery_failure.iter().cloned());
            failures.extend(
                pack.quarantined_files
                    .iter()
                    .map(|file| file.failure.clone()),
            );
        }
        SourceLoadingReport {
            source_root: self.source_root.clone(),
            manifest_path: self.manifest_path.clone(),
            manifest_content_hash: self.manifest_content_hash.clone(),
            pack_count: self.packs.len(),
            counts,
            by_document_type,
            diagnostics,
            failures,
        }
    }
}
