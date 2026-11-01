//! Bounded, revision-bound metadata reads between preparation phases.
use crate::{
    ArtifactInstance, IndexError, SourceArtifactBuildContext, SourceArtifactRecordInput,
    SourcePassageLayout,
};
use atlas_domain::RecordKey;
use atlas_storage::{ReadSession, Workspace};
use std::{collections::BTreeMap, path::PathBuf};

#[derive(Debug, Clone)]
pub struct ReusableSourceIdentity {
    pub key: RecordKey,
    pub source_path: String,
    pub content_hash: String,
    pub document_kind: String,
}

pub struct ReferenceReuseReader {
    path: PathBuf,
    schema: u32,
    expected: ArtifactInstance,
    context: Option<SourceArtifactBuildContext>,
    expected_context_hash: Option<String>,
}
impl ReferenceReuseReader {
    pub fn new(workspace: &Workspace, schema: u32) -> Result<Self, IndexError> {
        let read = workspace.read(schema)?;
        let expected = ArtifactInstance::new(read.workspace_key()?, read.reference_revision()?);
        let context = crate::read_stored_reference_context(read.connection())?;
        let context = match context {
            Some(context)
                if context.source_contract == atlas_foundry_model::SOURCE_CONTRACT_ID
                    && context.snapshot_version == atlas_foundry_model::SNAPSHOT_VERSION
                    && !matches!(
                        crate::classify_reference_upgrade(read.connection())?,
                        crate::ReferenceUpgrade::SourcePreparation
                            | crate::ReferenceUpgrade::SourceReplacement
                    ) =>
            {
                Some(context)
            }
            _ => None,
        };
        let expected_context_hash = context
            .as_ref()
            .map(SourceArtifactBuildContext::sha256)
            .transpose()?;
        Ok(Self {
            expected_context_hash,
            path: workspace.path().to_owned(),
            schema,
            expected,
            context,
        })
    }
    pub fn expected(&self) -> &ArtifactInstance {
        &self.expected
    }
    fn read(&self) -> Result<ReadSession, IndexError> {
        let session = Workspace::at(&self.path)?.read(self.schema)?;
        if session.workspace_key()? != self.expected.workspace_key()
            || session.reference_revision()? != self.expected.reference_revision()
        {
            return Err(IndexError::Invalid(
                "reference revision changed during preparation".into(),
            ));
        }
        let actual = crate::read_stored_reference_context(session.connection())?;
        if actual
            .as_ref()
            .map(SourceArtifactBuildContext::sha256)
            .transpose()?
            != self.expected_context_hash
            && self.context.is_some()
        {
            return Err(IndexError::Invalid(
                "reference context changed during preparation".into(),
            ));
        }
        Ok(session)
    }
    pub fn identities(
        &self,
    ) -> Result<BTreeMap<(String, String), ReusableSourceIdentity>, IndexError> {
        let mut output = BTreeMap::new();
        if self.context.is_none() {
            return Ok(output);
        }
        let mut after = 0i64;
        loop {
            let session = self.read()?;
            let mut statement=session.connection().prepare("SELECT record_id,key,source_path,content_hash,document_kind FROM records WHERE record_id>? ORDER BY record_id LIMIT 128")?;
            let rows = statement
                .query_map([after], |r| {
                    Ok((
                        r.get::<_, i64>(0)?,
                        r.get::<_, String>(1)?,
                        r.get::<_, String>(2)?,
                        r.get::<_, String>(3)?,
                        r.get::<_, String>(4)?,
                    ))
                })?
                .collect::<Result<Vec<_>, _>>()?;
            if rows.is_empty() {
                break;
            }
            for (id, key, path, hash, kind) in rows {
                after = id;
                let key: RecordKey = key
                    .parse()
                    .map_err(|e| IndexError::Invalid(format!("invalid reusable key: {e}")))?;
                output.insert(
                    (key.pack().as_str().into(), path.clone()),
                    ReusableSourceIdentity {
                        key,
                        source_path: path,
                        content_hash: hash,
                        document_kind: kind,
                    },
                );
            }
        }
        Ok(output)
    }
    pub fn record(&self, key: &RecordKey) -> Result<Option<SourceArtifactRecordInput>, IndexError> {
        let Some(context) = &self.context else {
            return Ok(None);
        };
        let session = self.read()?;
        crate::persisted_inputs::read_attested_persisted_input(session.connection(), context, key)
    }
    pub fn layouts(&self, key: &RecordKey) -> Result<Vec<SourcePassageLayout>, IndexError> {
        if self.context.is_none() {
            return Ok(Vec::new());
        }
        let session = self.read()?;
        Ok(
            crate::passage_layouts::read_selected_passage_layouts(&session, Some(key))?
                .remove(key)
                .unwrap_or_default(),
        )
    }
}

/// Current maintenance consumers visit one checked root at a time in their transaction.
pub fn visit_persisted_inputs(
    c: &rusqlite::Connection,
    context: &SourceArtifactBuildContext,
    visitor: &mut dyn FnMut(SourceArtifactRecordInput) -> Result<(), IndexError>,
) -> Result<(), IndexError> {
    let mut after = 0i64;
    loop {
        let rows = c
            .prepare(
                "SELECT record_id,key FROM records WHERE record_id>? ORDER BY record_id LIMIT 128",
            )?
            .query_map([after], |r| {
                Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        if rows.is_empty() {
            break;
        }
        for (id, key) in rows {
            after = id;
            let key = key
                .parse()
                .map_err(|e| IndexError::Invalid(format!("invalid stored key: {e}")))?;
            let record = crate::persisted_inputs::read_attested_persisted_input(c, context, &key)?
                .ok_or_else(|| IndexError::Invalid("stored root disappeared".into()))?;
            visitor(record)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::*;

    #[test]
    fn later_bounded_reads_recheck_actual_context_and_reference_revision() {
        let input = crate::foundation_tests::fixture(true);
        let (_directory, path) = crate::foundation_tests::build(&input);
        let workspace = initialize_workspace(&path).unwrap();
        let reader = ReferenceReuseReader::new(&workspace, TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
        let key = input.records[0].record.key();
        assert!(reader.record(key).unwrap().is_some());
        assert!(reader.layouts(key).is_ok());
        let mut changed = input.context.clone();
        changed.indexing_locale.push_str("-changed");
        {
            let write = workspace.write(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
            write
                .connection()
                .execute(
                    "UPDATE artifact_context SET context_json=?,context_hash=?",
                    rusqlite::params![
                        serde_json::to_string(&changed).unwrap(),
                        changed.sha256().unwrap()
                    ],
                )
                .unwrap();
            write.commit().unwrap();
        }
        assert!(
            reader
                .record(key)
                .unwrap_err()
                .to_string()
                .contains("reference context changed")
        );
        assert!(
            reader
                .layouts(key)
                .unwrap_err()
                .to_string()
                .contains("reference context changed")
        );
        {
            let write = workspace.write(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
            write
                .connection()
                .execute(
                    "UPDATE artifact_context SET context_json=?,context_hash=?",
                    rusqlite::params![
                        serde_json::to_string(&input.context).unwrap(),
                        input.context.sha256().unwrap()
                    ],
                )
                .unwrap();
            write.commit().unwrap();
        }
        assert!(reader.record(key).unwrap().is_some());
        {
            let write = workspace.write(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
            write
                .connection()
                .execute(
                    "UPDATE workspace SET reference_revision=reference_revision+1 WHERE id=1",
                    [],
                )
                .unwrap();
            write.commit().unwrap();
        }
        assert!(
            reader
                .record(key)
                .unwrap_err()
                .to_string()
                .contains("reference revision changed")
        );
        assert!(
            reader
                .layouts(key)
                .unwrap_err()
                .to_string()
                .contains("reference revision changed")
        );
    }
}
