use std::fs;
use std::path::Path;

use rayon::prelude::*;
use sha2::{Digest, Sha256};

use super::metadata::{SourceMetadata, SourcePackMetadata};
use super::model::{
    LoadedFoundryDocument, LoadedFoundryPack, LoadedFoundrySource, QuarantinedSourceFile,
    SourceFileProvenance, SourceLoadFailure, SourceLoadFailureStage,
};
use crate::error::IngestError;
use crate::source::discovery::{
    default_manifest_path, json_files, parse_manifest, relative_source_path, resolve_pack_path,
};
use atlas_foundry_model::{SourceContext, admit_document_source};

/// Load every manifest pack without constructing Atlas records, resolving
/// references, applying defaults, extracting metrics or writing artifacts.
/// Root/manifest failures are errors; pack/file failures are explicit outcomes.
pub fn load_foundry_documents(
    source_root: impl AsRef<Path>,
    manifest_path: Option<&Path>,
) -> Result<LoadedFoundrySource, IngestError> {
    load_captured_source(source_root.as_ref(), manifest_path, None)
}

pub(crate) fn load_captured_source(
    source_root: &Path,
    manifest_path: Option<&Path>,
    capture: Option<&crate::source_fingerprint::SourceCapture>,
) -> Result<LoadedFoundrySource, IngestError> {
    if !source_root.is_dir() {
        return Err(IngestError::SourceUnavailable(format!(
            "{} is not a readable directory",
            source_root.display()
        )));
    }
    let manifest_path = manifest_path
        .map(Path::to_path_buf)
        .unwrap_or_else(|| default_manifest_path(source_root));
    let parsed_manifest = match capture {
        Some(c) => crate::source::discovery::parse_manifest_bytes(&c.read(&manifest_path)?)?,
        None => parse_manifest(&manifest_path)?,
    };
    let mut packs = Vec::new();
    for declared in parsed_manifest.manifest.packs {
        let resolved_path = capture
            .map(|c| c.pack(&declared.path).0.clone())
            .unwrap_or_else(|| resolve_pack_path(source_root, &declared));
        let discovery = match capture {
            Some(c) if c.pack(&declared.path).1 => Ok(c
                .paths(&resolved_path)
                .into_iter()
                .filter(|p| {
                    p.extension().is_some_and(|x| x == "json")
                        && p.file_name().is_none_or(|x| x != "_folders.json")
                })
                .collect()),
            Some(_) => Err(IngestError::SourceUnavailable(
                "pack absent from captured source".into(),
            )),
            None => json_files(&resolved_path),
        };
        let discovery_failure = discovery.as_ref().err().map(|error| SourceLoadFailure {
            provenance: SourceFileProvenance {
                pack_name: declared.name.clone(),
                document_type: declared.document_type.clone(),
                source_path: relative_source_path(source_root, &resolved_path),
            },
            stage: SourceLoadFailureStage::Discovery,
            message: error.to_string(),
        });
        let paths = discovery.unwrap_or_default();
        let discovered_file_count = paths.len();
        let inputs = paths
            .into_iter()
            .map(|path| {
                let provenance = SourceFileProvenance {
                    pack_name: declared.name.clone(),
                    document_type: declared.document_type.clone(),
                    source_path: relative_source_path(source_root, &path),
                };
                (path, provenance)
            })
            .collect::<Vec<_>>();
        let outcomes: Vec<_> = inputs
            .into_par_iter()
            .map(|(path, provenance)| load_document(&path, provenance, capture))
            .collect();
        let mut documents = Vec::new();
        let mut quarantined_files = Vec::new();
        for outcome in outcomes {
            match outcome {
                Ok(document) => documents.push(document),
                Err(quarantined) => quarantined_files.push(*quarantined),
            }
        }
        packs.push(LoadedFoundryPack {
            metadata: SourcePackMetadata {
                name: declared.name,
                label: declared.label,
                document_type: declared.document_type,
                declared_path: declared.path,
                resolved_path,
                discovered_file_count,
            },
            documents,
            quarantined_files,
            discovery_failure,
        });
    }
    if let Some(c) = capture {
        c.verify_reads()?;
    }
    Ok(LoadedFoundrySource {
        metadata: SourceMetadata {
            source_root: source_root.to_path_buf(),
            manifest_path,
            manifest_content_hash: parsed_manifest.content_hash,
        },
        packs,
    })
}

fn load_document(
    path: &Path,
    provenance: SourceFileProvenance,
    capture: Option<&crate::source_fingerprint::SourceCapture>,
) -> Result<LoadedFoundryDocument, Box<QuarantinedSourceFile>> {
    let bytes = match capture {
        Some(c) => c.read(path),
        None => fs::read(path).map_err(|e| IngestError::SourceUnavailable(e.to_string())),
    }
    .map_err(|error| {
        Box::new(QuarantinedSourceFile {
            failure: SourceLoadFailure {
                provenance: provenance.clone(),
                stage: SourceLoadFailureStage::Read,
                message: error.to_string(),
            },
            bytes: None,
            content_hash: None,
        })
    })?;
    let content_hash = format!("{:x}", Sha256::digest(&bytes));
    // A path-based locator works even when _id is missing, invalid or duplicated.
    // It is diagnostic context, not an invented canonical Atlas RecordKey.
    let context = SourceContext::new(
        format!("{}:{}", provenance.pack_name, provenance.source_path),
        &provenance.source_path,
        "$",
    );
    let admission = admit_document_source(&provenance.document_type, context, &bytes);
    match admission {
        Ok(admission) => Ok(LoadedFoundryDocument {
            provenance,
            content_hash,
            bytes,
            admission,
        }),
        Err(error) => Err(Box::new(QuarantinedSourceFile {
            failure: SourceLoadFailure {
                provenance,
                stage: SourceLoadFailureStage::Parse,
                message: error.to_string(),
            },
            bytes: Some(bytes),
            content_hash: Some(content_hash),
        })),
    }
}
