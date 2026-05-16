use std::path::Path;

use atlas_domain::SearchFilterNode;
use rusqlite::{Connection, OpenFlags};

use crate::vector::register_sqlite_vec_extension;
use crate::{IndexValidationError, VectorQueryError, VectorSearchHit, query_vector_index};

pub struct AtlasIndex {
    connection: Connection,
}

impl AtlasIndex {
    pub fn open_read_only(path: impl AsRef<Path>) -> Result<Self, IndexValidationError> {
        register_sqlite_vec_extension()
            .map_err(|error| IndexValidationError::Unavailable(error.to_string()))?;
        let connection = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)
            .map_err(|error| IndexValidationError::Unavailable(error.to_string()))?;
        Ok(Self { connection })
    }

    pub fn query_vector_index(
        &self,
        query_vector: &[f32],
        filter: Option<&SearchFilterNode>,
        limit: u32,
        include_child_units: bool,
    ) -> Result<Vec<VectorSearchHit>, VectorQueryError> {
        query_vector_index(
            &self.connection,
            query_vector,
            filter,
            limit,
            include_child_units,
        )
    }
}
