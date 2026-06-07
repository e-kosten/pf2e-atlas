use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{TagDefinition, TagId, TagValidationError, validate_catalog_file};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TagCatalogFile {
    pub tags: Vec<TagDefinition>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagCatalog {
    definitions: Vec<TagDefinition>,
    by_id: BTreeMap<TagId, usize>,
}

impl TagCatalog {
    pub fn new(file: TagCatalogFile) -> Result<Self, TagValidationError> {
        Self::from_files([file])
    }

    pub fn from_files(
        files: impl IntoIterator<Item = TagCatalogFile>,
    ) -> Result<Self, TagValidationError> {
        let definitions = files
            .into_iter()
            .flat_map(|file| file.tags)
            .collect::<Vec<_>>();
        let file = TagCatalogFile { tags: definitions };
        validate_catalog_file(&file)?;
        let mut by_id = BTreeMap::new();
        for (index, definition) in file.tags.iter().enumerate() {
            by_id.insert(definition.id.clone(), index);
        }
        Ok(Self {
            definitions: file.tags,
            by_id,
        })
    }

    pub fn definitions(&self) -> &[TagDefinition] {
        &self.definitions
    }

    pub fn ids(&self) -> impl Iterator<Item = &TagId> {
        self.definitions.iter().map(|definition| &definition.id)
    }

    pub fn contains_id(&self, id: &TagId) -> bool {
        self.by_id.contains_key(id)
    }

    pub fn get(&self, id: &TagId) -> Option<&TagDefinition> {
        self.by_id.get(id).map(|index| &self.definitions[*index])
    }
}
