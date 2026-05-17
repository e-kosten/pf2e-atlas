use std::path::{Path, PathBuf};

use atlas_domain::{RecordKey, SearchFilterNode};
use atlas_record::{PersistedRecord, PersistedRecordSet};
use rusqlite::{Connection, OpenFlags, params_from_iter};

use crate::filters::{
    FilterCompileError, FilteredRecordKeysQuery, FilteredRecordSort as SqlFilteredRecordSort,
    compile_eligible_records_query, compile_filtered_record_keys_query,
};
use crate::vector::register_sqlite_vec_extension;
use crate::{
    ArtifactValidationReport, IndexInspectionReport, IndexValidationError, RecordLoadError,
    VectorQueryError, VectorSearchHit, inspect, records, validate_index_connection, vector,
};

pub struct AtlasIndex {
    path: PathBuf,
    connection: Connection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilteredRecordSort {
    RecordKey,
    Alphabetical,
    LevelAsc,
    LevelDesc,
    Random { seed: u64 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilteredRecordKeyPage {
    pub record_keys: Vec<RecordKey>,
    pub total: u64,
}

impl AtlasIndex {
    pub fn open_read_only(path: impl AsRef<Path>) -> Result<Self, IndexValidationError> {
        let path = path.as_ref().to_path_buf();
        let connection = Connection::open_with_flags(&path, OpenFlags::SQLITE_OPEN_READ_ONLY)
            .map_err(|error| IndexValidationError::Unavailable(error.to_string()))?;
        Ok(Self { path, connection })
    }

    pub fn open_read_only_with_vectors(
        path: impl AsRef<Path>,
    ) -> Result<Self, IndexValidationError> {
        register_sqlite_vec_extension().map_err(IndexValidationError::Unavailable)?;
        Self::open_read_only(path)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn validate(&self) -> Result<ArtifactValidationReport, IndexValidationError> {
        validate_index_connection(self.path.display().to_string(), &self.connection)
    }

    pub fn validate_report(&self) -> ArtifactValidationReport {
        match self.validate() {
            Ok(report) => report,
            Err(error) => crate::validation_report_from_error(&self.path, error),
        }
    }

    pub fn inspect(&self) -> Result<IndexInspectionReport, IndexValidationError> {
        let validation = self.validate()?;
        inspect::inspect_index_connection(
            self.path.display().to_string(),
            validation,
            &self.connection,
        )
    }

    pub fn load_records(&self) -> Result<Vec<PersistedRecord>, RecordLoadError> {
        records::load_persisted_records_from_connection(&self.connection)
    }

    pub fn load_record_set(&self) -> Result<PersistedRecordSet, RecordLoadError> {
        records::load_persisted_record_set_from_connection(&self.connection)
    }

    pub fn load_records_by_key(
        &self,
        keys: &[RecordKey],
    ) -> Result<Vec<PersistedRecord>, RecordLoadError> {
        records::load_persisted_records_by_key_from_connection(&self.connection, keys)
    }

    pub fn list_filtered_record_keys(
        &self,
        filter: Option<&SearchFilterNode>,
        sort: FilteredRecordSort,
        limit: u32,
        offset: u32,
    ) -> Result<FilteredRecordKeyPage, FilterCompileError> {
        match sort {
            FilteredRecordSort::Random { seed } => {
                let query = compile_filtered_record_keys_query(
                    filter,
                    SqlFilteredRecordSort::RecordKeyAsc,
                    None,
                    None,
                )?;
                let mut record_keys = read_record_keys(&self.connection, &query)?;
                record_keys.sort_by_key(|key| seeded_key_hash(seed, &key.to_string()));
                let total = record_keys.len() as u64;
                let record_keys = record_keys
                    .into_iter()
                    .skip(offset as usize)
                    .take(limit as usize)
                    .collect();
                Ok(FilteredRecordKeyPage { record_keys, total })
            }
            sort => {
                let total = count_filtered_records(&self.connection, filter)?;
                let query = compile_filtered_record_keys_query(
                    filter,
                    sql_sort(sort),
                    Some(limit),
                    Some(offset),
                )?;
                Ok(FilteredRecordKeyPage {
                    record_keys: read_record_keys(&self.connection, &query)?,
                    total,
                })
            }
        }
    }

    pub fn validate_vector_index(&self) -> Result<ArtifactValidationReport, IndexValidationError> {
        vector::validate_vector_index_connection(
            self.path.display().to_string(),
            self.validate()?,
            &self.connection,
        )
    }

    pub fn vector_validation_report(&self) -> ArtifactValidationReport {
        match self.validate_vector_index() {
            Ok(report) => report,
            Err(error) => crate::validation_report_from_error(&self.path, error),
        }
    }

    pub fn query_vector_index(
        &self,
        query_vector: &[f32],
        filter: Option<&SearchFilterNode>,
        limit: u32,
        include_child_units: bool,
    ) -> Result<Vec<VectorSearchHit>, VectorQueryError> {
        vector::query_vector_index(
            &self.connection,
            query_vector,
            filter,
            limit,
            include_child_units,
        )
    }
}

fn count_filtered_records(
    connection: &Connection,
    filter: Option<&SearchFilterNode>,
) -> Result<u64, FilterCompileError> {
    let eligible = compile_eligible_records_query(filter)?;
    let sql = format!(
        "WITH eligible(record_key) AS ({}) SELECT COUNT(*) FROM eligible",
        eligible.sql
    );
    connection
        .query_row(&sql, params_from_iter(eligible.parameters.iter()), |row| {
            row.get::<_, u64>(0)
        })
        .map_err(|error| FilterCompileError::QueryFailed(error.to_string()))
}

fn read_record_keys(
    connection: &Connection,
    query: &FilteredRecordKeysQuery,
) -> Result<Vec<RecordKey>, FilterCompileError> {
    let mut statement = connection
        .prepare(&query.sql)
        .map_err(|error| FilterCompileError::QueryFailed(error.to_string()))?;
    let keys = statement
        .query_map(params_from_iter(query.parameters.iter()), |row| {
            row.get::<_, String>(0)
        })
        .map_err(|error| FilterCompileError::QueryFailed(error.to_string()))?
        .map(|row| {
            row.map_err(|error| FilterCompileError::QueryFailed(error.to_string()))
                .and_then(|value| {
                    RecordKey::parse(&value)
                        .map_err(|error| FilterCompileError::InvalidValue(error.to_string()))
                })
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(keys)
}

fn sql_sort(sort: FilteredRecordSort) -> SqlFilteredRecordSort {
    match sort {
        FilteredRecordSort::RecordKey => SqlFilteredRecordSort::RecordKeyAsc,
        FilteredRecordSort::Alphabetical => SqlFilteredRecordSort::NameAsc,
        FilteredRecordSort::LevelAsc => SqlFilteredRecordSort::LevelAsc,
        FilteredRecordSort::LevelDesc => SqlFilteredRecordSort::LevelDesc,
        FilteredRecordSort::Random { .. } => SqlFilteredRecordSort::RecordKeyAsc,
    }
}

fn seeded_key_hash(seed: u64, key: &str) -> u64 {
    let mut hash = 0xcbf29ce484222325_u64 ^ seed;
    for byte in key.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}
