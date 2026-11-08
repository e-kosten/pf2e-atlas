//! Bounded typed workspace operations. No database session survives a request.
use crate::{AtlasRuntime, RuntimeError, WORKSPACE_SCHEMA_VERSION};
use atlas_local_state::{LocalStateRead, LocalStateWrite};
use atlas_search::{AtlasRetrievalService, PreparedQueryEmbedding};
use atlas_storage::{ReadSession, Workspace, WriteSession};

pub struct WorkspaceRead {
    retrieval: AtlasRetrievalService,
}
impl WorkspaceRead {
    pub fn retrieval(&self) -> &AtlasRetrievalService {
        &self.retrieval
    }
    pub fn retrieval_mut(&mut self) -> &mut AtlasRetrievalService {
        &mut self.retrieval
    }
    pub fn local_state(&self) -> atlas_local_state::LocalStateResult<LocalStateRead<'_>> {
        LocalStateRead::for_local(self.retrieval.workspace_session())
    }
    pub fn reference_revision(&self) -> Result<u64, RuntimeError> {
        Ok(self.retrieval.workspace_session().reference_revision()?)
    }
    pub fn generation(&self) -> Result<String, RuntimeError> {
        Ok(self.retrieval.artifact_generation()?)
    }
}

/// Saved domains remain usable before reference data has been populated.
pub struct StateRead {
    session: ReadSession,
}
impl StateRead {
    pub fn exact_records(&self) -> Result<ExactRecordRead<'_>, RuntimeError> {
        ExactRecordRead::open(self.session.connection())
    }
    pub fn local_state(&self) -> atlas_local_state::LocalStateResult<LocalStateRead<'_>> {
        LocalStateRead::for_local(&self.session)
    }
    pub fn reference_revision(&self) -> Result<u64, RuntimeError> {
        Ok(self.session.reference_revision()?)
    }
}
/// Exact summaries on an admitted snapshot, independent of search admission.
pub struct ExactRecordRead<'a> {
    connection: &'a atlas_storage::Connection,
    context: atlas_index::SourceArtifactBuildContext,
}
impl<'a> ExactRecordRead<'a> {
    fn open(connection: &'a atlas_storage::Connection) -> Result<Self, RuntimeError> {
        Ok(Self {
            connection,
            context: atlas_index::record_projection_context(connection)?,
        })
    }

    pub fn source_fingerprint(&self) -> &str {
        &self.context.source_fingerprint
    }

    pub fn trait_label(&self, identifier: &str) -> Option<&str> {
        self.context
            .used_trait_labels
            .get(identifier)
            .map(String::as_str)
    }

    pub fn summaries(
        &self,
        keys: &[atlas_domain::RecordKey],
    ) -> Result<Vec<atlas_domain::SourceRecordSummary>, RuntimeError> {
        let mut records = Vec::with_capacity(keys.len());
        for chunk in keys.chunks(1024) {
            records.extend(atlas_index::read_product_summaries(self.connection, chunk)?.records);
        }
        Ok(records)
    }
}

pub struct WorkspaceWrite {
    session: WriteSession,
}
impl WorkspaceWrite {
    pub fn local_state(&self) -> atlas_local_state::LocalStateResult<LocalStateWrite<'_>> {
        LocalStateWrite::for_local(&self.session)
    }
    pub fn exact_records(&self) -> Result<ExactRecordRead<'_>, RuntimeError> {
        ExactRecordRead::open(self.session.connection())
    }

    /// Resolve one product key without opening a separate read transaction.
    pub fn exact_record(
        &self,
        key: &atlas_domain::RecordKey,
    ) -> Result<Option<atlas_domain::SourceRecordSummary>, RuntimeError> {
        Ok(ExactRecordRead::open(self.session.connection())?
            .summaries(std::slice::from_ref(key))?
            .into_iter()
            .next())
    }

    /// Count live product records using the same snapshot as the saved mutation.
    pub fn resolved_record_count(
        &self,
        keys: &[atlas_domain::RecordKey],
    ) -> Result<Option<usize>, RuntimeError> {
        if keys.is_empty() {
            return Ok(Some(0));
        }
        match ExactRecordRead::open(self.session.connection()) {
            Ok(reader) => Ok(Some(reader.summaries(keys)?.len())),
            Err(error) if error.reference_unavailable() => Ok(None),
            Err(error) => Err(error),
        }
    }

    /// Call after durable receipt recovery, before applying source-derived results.
    pub fn require_reference_revision(&self, expected: u64) -> Result<(), RuntimeError> {
        let actual = self.session.reference_revision()?;
        if actual != expected {
            return Err(RuntimeError::stale_reference(expected, actual));
        }
        Ok(())
    }
    pub fn commit(self) -> Result<(), RuntimeError> {
        Ok(self.session.commit()?)
    }
}
impl AtlasRuntime {
    pub fn read_workspace(
        &self,
        prepared: Option<PreparedQueryEmbedding>,
    ) -> Result<WorkspaceRead, RuntimeError> {
        self.reference_read(self.read_state()?, prepared)
    }

    /// Attach reference capabilities to the saved-state snapshot already admitted.
    pub fn reference_read(
        &self,
        state: StateRead,
        prepared: Option<PreparedQueryEmbedding>,
    ) -> Result<WorkspaceRead, RuntimeError> {
        let index = atlas_index::SqliteIndexReader::from_session(state.session)?;
        self.retrieval_read(index, prepared)
    }

    pub fn record_detail_read(&self, state: StateRead) -> Result<WorkspaceRead, RuntimeError> {
        let index = atlas_index::SqliteIndexReader::from_session_for_record_detail(state.session)?;
        Ok(WorkspaceRead {
            retrieval: AtlasRetrievalService::from_index(
                index,
                atlas_domain::SearchProfileId::new(atlas_embedding::DEFAULT_SEARCH_PROFILE),
                None,
            )?,
        })
    }

    fn retrieval_read(
        &self,
        index: atlas_index::SqliteIndexReader,
        prepared: Option<PreparedQueryEmbedding>,
    ) -> Result<WorkspaceRead, RuntimeError> {
        let default_profile = self.resolve_default_search_profile(index.context())?;
        Ok(WorkspaceRead {
            retrieval: AtlasRetrievalService::from_index(index, default_profile, prepared)?
                .with_native_neighbor_reuse(std::sync::Arc::clone(&self.native_neighbor_reuse)),
        })
    }
    pub fn read_state(&self) -> Result<StateRead, RuntimeError> {
        crate::workspace::register_extensions()?;
        Ok(StateRead {
            session: Workspace::existing(self.index_path())?.read(WORKSPACE_SCHEMA_VERSION)?,
        })
    }
    pub fn write_workspace(&self) -> Result<WorkspaceWrite, RuntimeError> {
        crate::workspace::register_extensions()?;
        Ok(WorkspaceWrite {
            session: Workspace::existing(self.index_path())?.write(WORKSPACE_SCHEMA_VERSION)?,
        })
    }
}
