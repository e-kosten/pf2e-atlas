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

/// Maximum owned reuse results returned by one authenticated read snapshot.
pub const SOURCE_REUSE_BATCH_LIMIT: usize = 128;

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
                    && (crate::classify_reference_upgrade(read.connection())?
                        != crate::ReferenceUpgrade::SourceReplacement)
                    && crate::component_upgrade::source_preparation_is_current(&context) =>
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
        let session = self.read()?;
        let mut statement = session.connection().prepare(
            "SELECT key,source_path,content_hash,document_kind FROM records ORDER BY record_id",
        )?;
        let mut rows = statement.query([])?;
        while let Some(row) = rows.next()? {
            let key: String = row.get(0)?;
            let path: String = row.get(1)?;
            let key: RecordKey = key
                .parse()
                .map_err(|e| IndexError::Invalid(format!("invalid reusable key: {e}")))?;
            output.insert(
                (key.pack().as_str().into(), path.clone()),
                ReusableSourceIdentity {
                    key,
                    source_path: path,
                    content_hash: row.get(2)?,
                    document_kind: row.get(3)?,
                },
            );
        }
        Ok(output)
    }
    pub fn preparation_matches(&self, preparation_hash: &str) -> Result<bool, IndexError> {
        self.context
            .as_ref()
            .map(|context| {
                context
                    .preparation_sha256()
                    .map(|hash| hash == preparation_hash)
            })
            .transpose()
            .map(|matches| matches.unwrap_or(false))
    }

    pub fn read_admissions(
        &self,
        keys: &[&RecordKey],
    ) -> Result<Vec<Option<crate::ReusableSourceAdmission>>, IndexError> {
        let Some((session, context)) = self.batch_session(keys.len())? else {
            return Ok(keys.iter().map(|_| None).collect());
        };
        keys.iter()
            .map(|key| {
                crate::persisted_inputs::read_attested_source_admission(
                    session.connection(),
                    context,
                    key,
                )
            })
            .collect()
    }

    pub fn read_records(
        &self,
        keys: &[&RecordKey],
    ) -> Result<Vec<Option<SourceArtifactRecordInput>>, IndexError> {
        let Some((session, context)) = self.batch_session(keys.len())? else {
            return Ok(keys.iter().map(|_| None).collect());
        };
        keys.iter()
            .map(|key| {
                crate::persisted_inputs::read_attested_persisted_input(
                    session.connection(),
                    context,
                    key,
                )
            })
            .collect()
    }

    pub fn read_layouts(
        &self,
        keys: &[&RecordKey],
    ) -> Result<Vec<Vec<SourcePassageLayout>>, IndexError> {
        let Some((session, context)) = self.batch_session(keys.len())? else {
            return Ok(keys.iter().map(|_| Vec::new()).collect());
        };
        let Some(layouts) =
            crate::passage_layouts::PassageLayoutReader::new(session.connection(), context)?
        else {
            return Ok(keys.iter().map(|_| Vec::new()).collect());
        };
        keys.iter().map(|key| layouts.read(key)).collect()
    }

    fn batch_session(
        &self,
        count: usize,
    ) -> Result<Option<(ReadSession, &SourceArtifactBuildContext)>, IndexError> {
        if count > SOURCE_REUSE_BATCH_LIMIT {
            return Err(IndexError::Invalid(
                "source reuse batch exceeds 128 records".into(),
            ));
        }
        let Some(context) = &self.context else {
            return Ok(None);
        };
        if count == 0 {
            return Ok(None);
        }
        Ok(Some((self.read()?, context)))
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
        assert!(reader.read_records(&[key]).unwrap()[0].is_some());
        assert!(reader.read_layouts(&[key]).is_ok());
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
                .read_records(&[key])
                .unwrap_err()
                .to_string()
                .contains("reference context changed")
        );
        assert!(
            reader
                .read_layouts(&[key])
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
        assert!(reader.read_records(&[key]).unwrap()[0].is_some());
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
                .read_records(&[key])
                .unwrap_err()
                .to_string()
                .contains("reference revision changed")
        );
        assert!(
            reader
                .read_layouts(&[key])
                .unwrap_err()
                .to_string()
                .contains("reference revision changed")
        );
    }
    #[test]
    fn owned_batches_preserve_order_repeated_keys_missing_records_and_io_bounds() {
        use atlas_storage::diagnostics::count_workspace_connections;
        let input = crate::foundation_tests::fixture(true);
        let (_directory, path) = crate::foundation_tests::build(&input);
        let workspace = initialize_workspace(&path).unwrap();
        let mut reader =
            ReferenceReuseReader::new(&workspace, TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
        let (identities, connections) =
            count_workspace_connections(|| reader.identities()).unwrap();
        assert_eq!(identities.unwrap().len(), input.records.len());
        assert_eq!(connections, 1);
        let key = input.records[1].record.key();
        let first = input.records[0].record.key();
        let absent: RecordKey = "items:0000000000000000".parse().unwrap();
        let keys = [key, &absent, key, first];
        let (records, connections) =
            count_workspace_connections(|| reader.read_records(&keys)).unwrap();
        let records = records.unwrap();
        assert_eq!(connections, 1);
        assert_eq!(records.len(), keys.len());
        for (record, expected) in records.iter().zip(keys) {
            if expected == &absent {
                assert!(record.is_none());
            } else {
                assert_eq!(record.as_ref().unwrap().record.key(), expected);
            }
        }
        assert_eq!(
            records[0].as_ref().unwrap().content,
            records[2].as_ref().unwrap().content
        );
        let (admissions, connections) =
            count_workspace_connections(|| reader.read_admissions(&keys)).unwrap();
        let admissions = admissions.unwrap();
        assert_eq!(connections, 1);
        for (admission, record) in admissions.iter().zip(&records) {
            assert_eq!(
                admission.as_ref().map(|a| (
                    &a.record,
                    &a.source_path,
                    &a.content_hash,
                    &a.diagnostics
                )),
                record.as_ref().map(|r| (
                    &r.record,
                    &r.source_path,
                    &r.content_hash,
                    &r.diagnostics
                )),
            );
        }
        let (layouts, connections) =
            count_workspace_connections(|| reader.read_layouts(&keys)).unwrap();
        let layouts = layouts.unwrap();
        assert_eq!(connections, 1);
        assert_eq!(layouts.len(), keys.len());
        assert!(!layouts[0].is_empty());
        assert!(layouts[1].is_empty());
        assert!(layouts[3].is_empty());
        assert_eq!(layouts[0].len(), layouts[2].len());
        assert!(layouts[0].iter().zip(&layouts[2]).all(|(left, right)| {
            left.owners == right.owners
                && left.field == right.field
                && left.chunk_ordinal == right.chunk_ordinal
                && left.passage.address == right.passage.address
                && left.passage.input_sha256 == right.passage.input_sha256
        }));
        for count in [1, 32, 128] {
            let keys = vec![key; count];
            let (records, connections) =
                count_workspace_connections(|| reader.read_records(&keys)).unwrap();
            assert_eq!(records.unwrap().len(), count);
            assert_eq!(connections, 1);
            let (layouts, connections) =
                count_workspace_connections(|| reader.read_layouts(&keys)).unwrap();
            assert_eq!(layouts.unwrap().len(), count);
            assert_eq!(connections, 1);
        }
        let (records, connections) =
            count_workspace_connections(|| reader.read_records(&[])).unwrap();
        assert!(records.unwrap().is_empty());
        assert_eq!(connections, 0);
        let (layouts, connections) =
            count_workspace_connections(|| reader.read_layouts(&[])).unwrap();
        assert!(layouts.unwrap().is_empty());
        assert_eq!(connections, 0);
        reader.context = None;
        let (records, connections) =
            count_workspace_connections(|| reader.read_records(&[key; 128])).unwrap();
        assert!(records.unwrap().iter().all(Option::is_none));
        assert_eq!(connections, 0);
        let (layouts, connections) =
            count_workspace_connections(|| reader.read_layouts(&[key; 128])).unwrap();
        assert!(layouts.unwrap().iter().all(Vec::is_empty));
        assert_eq!(connections, 0);
        let (records, connections) =
            count_workspace_connections(|| reader.read_records(&[key; 129])).unwrap();
        assert!(records.unwrap_err().to_string().contains("exceeds 128"));
        assert_eq!(connections, 0);
        let (layouts, connections) =
            count_workspace_connections(|| reader.read_layouts(&[key; 129])).unwrap();
        assert!(layouts.unwrap_err().to_string().contains("exceeds 128"));
        assert_eq!(connections, 0);
    }

    #[test]
    fn requested_payloads_are_independent_and_unrequested_corruption_is_isolated() {
        let input = crate::core_tests::fixture(
            vec![
                (
                    "items",
                    "Item",
                    serde_json::json!({"_id":"aaaaaaaaaaaaaaaa","name":"First","type":"effect","system":{"description":{"value":"<p>First body</p>"}}}),
                ),
                (
                    "items",
                    "Item",
                    serde_json::json!({"_id":"bbbbbbbbbbbbbbbb","name":"Second","type":"effect","system":{"description":{"value":"<p>Second body</p>"}}}),
                ),
            ],
            false,
        );
        let (_directory, path) = crate::foundation_tests::build(&input);
        let workspace = initialize_workspace(&path).unwrap();
        let reader = ReferenceReuseReader::new(&workspace, TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
        let body_bad = input.records[1].record.key();
        let layout_bad = input.records[0].record.key();
        let c = rusqlite::Connection::open(&path).unwrap();
        c.execute("UPDATE record_bodies SET snapshot=X'00' WHERE record_id=(SELECT record_id FROM records WHERE key=?)", [body_bad.to_string()]).unwrap();
        assert_eq!(c.execute("UPDATE semantic_locations SET section_id=NULL,chunk_ordinal=NULL,chunk_start=NULL,chunk_end=NULL WHERE location_id=(SELECT min(su.location_id) FROM semantic_units su JOIN records r USING(record_id) WHERE r.key=? AND su.unit_kind='passage')", [layout_bad.to_string()]).unwrap(), 1);
        assert!(!reader.read_layouts(&[body_bad]).unwrap()[0].is_empty());
        assert!(reader.read_records(&[layout_bad]).unwrap()[0].is_some());
        assert!(reader.read_records(&[body_bad]).is_err());
        assert!(reader.read_admissions(&[body_bad]).is_err());
        assert!(reader.read_layouts(&[layout_bad]).is_err());
        assert_eq!(c.execute("UPDATE prepared_content SET html_gzip=X'00' WHERE record_id=(SELECT record_id FROM records WHERE key=?) AND html_gzip IS NOT NULL",[layout_bad.to_string()]).unwrap(), 1);
        assert_eq!(
            reader.read_admissions(&[layout_bad]).unwrap()[0]
                .as_ref()
                .unwrap()
                .record
                .key(),
            layout_bad
        );
        assert!(reader.read_records(&[layout_bad]).is_err());
        assert_eq!(c.execute("DELETE FROM record_bodies WHERE record_id=(SELECT record_id FROM records WHERE key=?)",[body_bad.to_string()]).unwrap(),1);
        let unavailable = reader.read_records(&[body_bad]).unwrap();
        assert!(unavailable[0].is_none());
        let mut retired = input.context;
        retired.source_contract = "retired-source".into();
        c.execute(
            "UPDATE artifact_context SET context_json=?,context_hash=?",
            rusqlite::params![
                serde_json::to_string(&retired).unwrap(),
                retired.sha256().unwrap()
            ],
        )
        .unwrap();
        let reader = ReferenceReuseReader::new(&workspace, TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
        let (result, connections) = atlas_storage::diagnostics::count_workspace_connections(|| {
            reader.read_records(&[body_bad])
        })
        .unwrap();
        assert!(result.unwrap()[0].is_none());
        assert_eq!(connections, 0);
    }
    #[test]
    fn layout_batches_recheck_mutable_profile_storage_without_widening_record_reads() {
        let input = crate::foundation_tests::fixture(true);
        let (_directory, path) = crate::foundation_tests::build(&input);
        let workspace = initialize_workspace(&path).unwrap();
        let reader = ReferenceReuseReader::new(&workspace, TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
        let key = input.records[1].record.key();
        let keys = [key];
        assert!(!reader.read_layouts(&keys).unwrap()[0].is_empty());
        let c = rusqlite::Connection::open(&path).unwrap();
        let original: String = c
            .query_row(
                "SELECT identity_json FROM search_profiles WHERE profile_id=?",
                [input.context.search_profiles[0].profile_id.as_str()],
                |r| r.get(0),
            )
            .unwrap();
        let mut changed = input.context.search_profiles[0].clone();
        changed
            .revision
            .push_str("-changed-without-context-or-revision");
        assert_eq!(
            c.execute(
                "UPDATE search_profiles SET identity_json=? WHERE profile_id=?",
                rusqlite::params![
                    serde_json::to_string(&changed).unwrap(),
                    changed.profile_id.as_str()
                ]
            )
            .unwrap(),
            1
        );
        assert!(reader.read_records(&[key]).unwrap()[0].is_some());
        let error = reader.read_layouts(&keys).unwrap_err();
        assert!(
            error.to_string().contains("stored search profiles differ"),
            "{error}"
        );
        c.execute(
            "UPDATE search_profiles SET identity_json=? WHERE profile_id=?",
            rusqlite::params![original, changed.profile_id.as_str()],
        )
        .unwrap();
        assert!(!reader.read_layouts(&keys).unwrap()[0].is_empty());
    }
    #[test]
    fn layout_batches_admit_compatible_metadata_upgrades_before_open_validation() {
        let input = crate::foundation_tests::fixture(true);
        let (_directory, path) = crate::foundation_tests::build(&input);
        let workspace = initialize_workspace(&path).unwrap();
        let key = input.records[1].record.key();
        for expected in [
            crate::ReferenceUpgrade::QueryProjections,
            crate::ReferenceUpgrade::CoreProjections,
            crate::ReferenceUpgrade::SectionProjections,
        ] {
            let mut context = input.context.clone();
            match expected {
                crate::ReferenceUpgrade::QueryProjections => context.query_catalog_version -= 1,
                crate::ReferenceUpgrade::CoreProjections => context.core_sections_recipe.clear(),
                crate::ReferenceUpgrade::SectionProjections => {
                    context.selected_sections_recipe.clear()
                }
                _ => unreachable!(),
            }
            let c = rusqlite::Connection::open(&path).unwrap();
            c.execute(
                "UPDATE artifact_context SET context_json=?,context_hash=?",
                rusqlite::params![
                    serde_json::to_string(&context).unwrap(),
                    context.sha256().unwrap()
                ],
            )
            .unwrap();
            assert_eq!(crate::classify_reference_upgrade(&c).unwrap(), expected);
            let reader =
                ReferenceReuseReader::new(&workspace, TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
            assert!(
                !reader.read_layouts(&[key]).unwrap()[0].is_empty(),
                "{expected:?}"
            );
        }
    }
}
