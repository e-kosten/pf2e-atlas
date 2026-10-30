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
    pub fn local_state(&self) -> LocalStateRead<'_> {
        LocalStateRead::new(self.retrieval.workspace_session())
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
    pub fn local_state(&self) -> LocalStateRead<'_> {
        LocalStateRead::new(&self.session)
    }
    pub fn reference_revision(&self) -> Result<u64, RuntimeError> {
        Ok(self.session.reference_revision()?)
    }
}
pub struct WorkspaceWrite {
    session: WriteSession,
}
impl WorkspaceWrite {
    pub fn local_state(&self) -> LocalStateWrite<'_> {
        LocalStateWrite::new(&self.session)
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
        crate::workspace::register_extensions()?;
        let session = Workspace::at(self.index_path())?.read(WORKSPACE_SCHEMA_VERSION)?;
        let index = atlas_index::SqliteIndexReader::from_session(session)?;
        let default_profile = self.resolve_default_search_profile(index.context())?;
        Ok(WorkspaceRead {
            retrieval: AtlasRetrievalService::from_index(index, default_profile, prepared)?
                .with_text_neighbor_reuse(std::sync::Arc::clone(&self.text_neighbor_reuse)),
        })
    }
    pub fn read_state(&self) -> Result<StateRead, RuntimeError> {
        crate::workspace::register_extensions()?;
        Ok(StateRead {
            session: Workspace::at(self.index_path())?.read(WORKSPACE_SCHEMA_VERSION)?,
        })
    }
    pub fn write_workspace(&self) -> Result<WorkspaceWrite, RuntimeError> {
        crate::workspace::register_extensions()?;
        Ok(WorkspaceWrite {
            session: Workspace::at(self.index_path())?.write(WORKSPACE_SCHEMA_VERSION)?,
        })
    }
}
