use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::{
    AssignmentFile, OntologySuggestionFile, TagCatalog, TagCatalogFile, TagValidationError,
    TagYamlError, assignment_file_from_str, catalog_file_from_str,
    ontology_suggestion_file_from_str, validate_assignments_against_catalog,
    validate_ontology_suggestion_file,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagCorpus {
    pub catalog: TagCatalog,
    pub assignments: AssignmentFile,
    pub ontology_suggestions: OntologySuggestionFile,
}

impl TagCorpus {
    pub fn load_from_dir(root: impl AsRef<Path>) -> Result<Self, TagCorpusLoadError> {
        load_tag_corpus(root)
    }
}

pub fn load_tag_corpus(root: impl AsRef<Path>) -> Result<TagCorpus, TagCorpusLoadError> {
    validate_tag_corpus(root)
}

pub fn validate_tag_corpus(root: impl AsRef<Path>) -> Result<TagCorpus, TagCorpusLoadError> {
    let root = root.as_ref();
    let catalog_files = load_catalog_files(&root.join("catalog"))?;
    let catalog = TagCatalog::from_files(catalog_files).map_err(TagCorpusLoadError::Validation)?;

    let assignments = load_assignment_files_if_present(&root.join("assignments"))?;
    validate_assignments_against_catalog(&assignments, &catalog)
        .map_err(TagCorpusLoadError::Validation)?;

    let ontology_suggestions =
        load_ontology_suggestion_files_if_present(&root.join("ontology-suggestions"))?;
    validate_ontology_suggestion_file(&ontology_suggestions)
        .map_err(TagCorpusLoadError::Validation)?;

    Ok(TagCorpus {
        catalog,
        assignments,
        ontology_suggestions,
    })
}

fn load_catalog_files(path: &Path) -> Result<Vec<TagCatalogFile>, TagCorpusLoadError> {
    if !path.is_dir() {
        return Err(TagCorpusLoadError::MissingCatalogDirectory {
            path: path.to_path_buf(),
        });
    }
    yaml_files(path)?
        .into_iter()
        .map(|file| parse_file(&file, catalog_file_from_str))
        .collect()
}

fn load_assignment_files_if_present(path: &Path) -> Result<AssignmentFile, TagCorpusLoadError> {
    if !path.exists() {
        return Ok(AssignmentFile {
            records: Vec::new(),
        });
    }
    let records = yaml_files(path)?
        .into_iter()
        .map(|file| parse_file(&file, assignment_file_from_str))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .flat_map(|file: AssignmentFile| file.records)
        .collect();
    Ok(AssignmentFile { records })
}

fn load_ontology_suggestion_files_if_present(
    path: &Path,
) -> Result<OntologySuggestionFile, TagCorpusLoadError> {
    if !path.exists() {
        return Ok(OntologySuggestionFile {
            suggestions: Vec::new(),
        });
    }
    let suggestions = yaml_files(path)?
        .into_iter()
        .map(|file| parse_file(&file, ontology_suggestion_file_from_str))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .flat_map(|file: OntologySuggestionFile| file.suggestions)
        .collect();
    Ok(OntologySuggestionFile { suggestions })
}

fn parse_file<T>(
    path: &Path,
    parse: impl FnOnce(&str) -> Result<T, TagYamlError>,
) -> Result<T, TagCorpusLoadError> {
    let input = fs::read_to_string(path).map_err(|source| TagCorpusLoadError::ReadFile {
        path: path.to_path_buf(),
        source,
    })?;
    parse(&input).map_err(|source| TagCorpusLoadError::ParseFile {
        path: path.to_path_buf(),
        source,
    })
}

fn yaml_files(root: &Path) -> Result<Vec<PathBuf>, TagCorpusLoadError> {
    if !root.is_dir() {
        return Err(TagCorpusLoadError::ReadDirectory {
            path: root.to_path_buf(),
            source: io::Error::new(io::ErrorKind::NotFound, "directory does not exist"),
        });
    }
    let mut files = Vec::new();
    collect_yaml_files(root, &mut files)?;
    files.sort();
    Ok(files)
}

fn collect_yaml_files(root: &Path, files: &mut Vec<PathBuf>) -> Result<(), TagCorpusLoadError> {
    for entry in fs::read_dir(root).map_err(|source| TagCorpusLoadError::ReadDirectory {
        path: root.to_path_buf(),
        source,
    })? {
        let entry = entry.map_err(|source| TagCorpusLoadError::ReadDirectory {
            path: root.to_path_buf(),
            source,
        })?;
        let path = entry.path();
        let file_type = entry
            .file_type()
            .map_err(|source| TagCorpusLoadError::ReadDirectory {
                path: path.clone(),
                source,
            })?;
        if file_type.is_dir() {
            collect_yaml_files(&path, files)?;
        } else if file_type.is_file() && is_yaml_file(&path) {
            files.push(path);
        }
    }
    Ok(())
}

fn is_yaml_file(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| matches!(extension, "yaml" | "yml"))
}

#[derive(Debug, Error)]
pub enum TagCorpusLoadError {
    #[error("tag corpus requires catalog directory `{path:?}`")]
    MissingCatalogDirectory { path: PathBuf },
    #[error("failed to read tag corpus directory `{path:?}`: {source}")]
    ReadDirectory { path: PathBuf, source: io::Error },
    #[error("failed to read tag corpus file `{path:?}`: {source}")]
    ReadFile { path: PathBuf, source: io::Error },
    #[error("failed to parse tag corpus file `{path:?}`: {source}")]
    ParseFile { path: PathBuf, source: TagYamlError },
    #[error(transparent)]
    Validation(TagValidationError),
}
