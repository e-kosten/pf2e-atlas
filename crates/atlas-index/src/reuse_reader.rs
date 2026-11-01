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

/// Each request is independent; repeated keys retain their original ordinal and flags.
#[derive(Clone, Copy, Debug)]
pub struct SourceReuseRequest<'a> {
    pub key: &'a RecordKey,
    pub record: bool,
    pub layouts: bool,
}

#[derive(Debug)]
pub struct ReusedSourceRecord {
    pub key: RecordKey,
    pub record: Option<SourceArtifactRecordInput>,
    pub layouts: Vec<SourcePassageLayout>,
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
    /// Read only requested retained payloads in one current snapshot. All returned
    /// values are owned; the session closes before the caller processes the batch.
    pub fn read_batch(
        &self,
        requests: &[SourceReuseRequest<'_>],
    ) -> Result<Vec<ReusedSourceRecord>, IndexError> {
        if requests.len() > SOURCE_REUSE_BATCH_LIMIT {
            return Err(IndexError::Invalid(
                "source reuse batch exceeds 128 records".into(),
            ));
        }
        let mut output = requests
            .iter()
            .map(|request| ReusedSourceRecord {
                key: request.key.clone(),
                record: None,
                layouts: Vec::new(),
            })
            .collect::<Vec<_>>();
        let Some(context) = &self.context else {
            return Ok(output);
        };
        if !requests
            .iter()
            .any(|request| request.record || request.layouts)
        {
            return Ok(output);
        }
        let session = self.read()?;
        let layouts = if requests.iter().any(|request| request.layouts) {
            crate::passage_layouts::PassageLayoutReader::new(session.connection(), context)?
        } else {
            None
        };
        for (request, result) in requests.iter().zip(&mut output) {
            if request.record {
                result.record = crate::persisted_inputs::read_attested_persisted_input(
                    session.connection(),
                    context,
                    request.key,
                )?;
            }
            if request.layouts
                && let Some(layouts) = &layouts
            {
                result.layouts = layouts.read(request.key)?;
            }
        }
        Ok(output)
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
        assert!(
            reader
                .read_batch(&[SourceReuseRequest {
                    key,
                    record: true,
                    layouts: false
                }])
                .unwrap()[0]
                .record
                .is_some()
        );
        assert!(
            reader
                .read_batch(&[SourceReuseRequest {
                    key,
                    record: false,
                    layouts: true
                }])
                .is_ok()
        );
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
                .read_batch(&[SourceReuseRequest {
                    key,
                    record: true,
                    layouts: false
                }])
                .unwrap_err()
                .to_string()
                .contains("reference context changed")
        );
        assert!(
            reader
                .read_batch(&[SourceReuseRequest {
                    key,
                    record: false,
                    layouts: true
                }])
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
        assert!(
            reader
                .read_batch(&[SourceReuseRequest {
                    key,
                    record: true,
                    layouts: false
                }])
                .unwrap()[0]
                .record
                .is_some()
        );
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
                .read_batch(&[SourceReuseRequest {
                    key,
                    record: true,
                    layouts: false
                }])
                .unwrap_err()
                .to_string()
                .contains("reference revision changed")
        );
        assert!(
            reader
                .read_batch(&[SourceReuseRequest {
                    key,
                    record: false,
                    layouts: true
                }])
                .unwrap_err()
                .to_string()
                .contains("reference revision changed")
        );
    }
    #[test]
    fn owned_batches_preserve_repeated_keys_flags_subsets_and_strict_io_bounds() {
        use atlas_storage::diagnostics::capture_sql_work;
        let input = crate::foundation_tests::fixture(true);
        let (_directory, path) = crate::foundation_tests::build(&input);
        let workspace = initialize_workspace(&path).unwrap();
        let mut reader =
            ReferenceReuseReader::new(&workspace, TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
        let key = input.records[1].record.key();
        let absent: RecordKey = "items:0000000000000000".parse().unwrap();
        let requests = [
            SourceReuseRequest {
                key,
                record: true,
                layouts: false,
            },
            SourceReuseRequest {
                key,
                record: false,
                layouts: true,
            },
            SourceReuseRequest {
                key: &absent,
                record: true,
                layouts: true,
            },
            SourceReuseRequest {
                key,
                record: true,
                layouts: true,
            },
            SourceReuseRequest {
                key,
                record: false,
                layouts: false,
            },
        ];
        let (result, metrics) = capture_sql_work(|| reader.read_batch(&requests)).unwrap();
        let result = result.unwrap();
        assert_eq!(metrics.workspace_connections, 1);
        assert_eq!(
            result.iter().map(|r| &r.key).collect::<Vec<_>>(),
            requests.iter().map(|r| r.key).collect::<Vec<_>>()
        );
        assert!(result[0].record.is_some() && result[0].layouts.is_empty());
        assert!(result[1].record.is_none() && !result[1].layouts.is_empty());
        assert!(result[2].record.is_none() && result[2].layouts.is_empty());
        assert!(result[3].record.is_some() && !result[3].layouts.is_empty());
        assert!(result[4].record.is_none() && result[4].layouts.is_empty());
        assert_eq!(
            result[0].record.as_ref().unwrap().content,
            result[3].record.as_ref().unwrap().content
        );
        for count in [1, 32, 128] {
            let requests = vec![requests[3]; count];
            let (result, metrics) = capture_sql_work(|| reader.read_batch(&requests)).unwrap();
            assert_eq!(result.unwrap().len(), count);
            assert_eq!(metrics.workspace_connections, 1);
        }
        for requests in [vec![], vec![requests[4]; 128]] {
            let (result, metrics) = capture_sql_work(|| reader.read_batch(&requests)).unwrap();
            assert_eq!(result.unwrap().len(), requests.len());
            assert_eq!(metrics.workspace_connections, 0);
        }
        // Guard before compatibility as well as before I/O, including no-flag requests.
        reader.context = None;
        for request in requests {
            let (result, metrics) =
                capture_sql_work(|| reader.read_batch(&[request; 129])).unwrap();
            assert!(result.unwrap_err().to_string().contains("exceeds 128"));
            assert_eq!(metrics.workspace_connections, 0);
        }
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
        let result = reader
            .read_batch(&[
                SourceReuseRequest {
                    key: body_bad,
                    record: false,
                    layouts: true,
                },
                SourceReuseRequest {
                    key: layout_bad,
                    record: true,
                    layouts: false,
                },
            ])
            .unwrap();
        assert!(!result[0].layouts.is_empty() && result[0].record.is_none());
        assert!(result[1].record.is_some() && result[1].layouts.is_empty());
        assert!(
            reader
                .read_batch(&[SourceReuseRequest {
                    key: body_bad,
                    record: true,
                    layouts: false
                }])
                .is_err()
        );
        assert!(
            reader
                .read_batch(&[SourceReuseRequest {
                    key: layout_bad,
                    record: false,
                    layouts: true
                }])
                .is_err()
        );
        assert_eq!(c.execute("DELETE FROM record_bodies WHERE record_id=(SELECT record_id FROM records WHERE key=?)",[body_bad.to_string()]).unwrap(),1);
        let unavailable = reader
            .read_batch(&[SourceReuseRequest {
                key: body_bad,
                record: true,
                layouts: false,
            }])
            .unwrap();
        assert!(unavailable[0].record.is_none());
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
        let (result, metrics) = atlas_storage::diagnostics::capture_sql_work(|| {
            reader.read_batch(&[SourceReuseRequest {
                key: body_bad,
                record: true,
                layouts: true,
            }])
        })
        .unwrap();
        assert!(result.unwrap()[0].record.is_none());
        assert_eq!(metrics.workspace_connections, 0);
    }
    #[test]
    fn layout_batches_recheck_mutable_profile_storage_without_widening_record_reads() {
        let input = crate::foundation_tests::fixture(true);
        let (_directory, path) = crate::foundation_tests::build(&input);
        let workspace = initialize_workspace(&path).unwrap();
        let reader = ReferenceReuseReader::new(&workspace, TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
        let key = input.records[1].record.key();
        let request = SourceReuseRequest {
            key,
            record: false,
            layouts: true,
        };
        assert!(!reader.read_batch(&[request]).unwrap()[0].layouts.is_empty());
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
        assert!(
            reader
                .read_batch(&[SourceReuseRequest {
                    key,
                    record: true,
                    layouts: false
                }])
                .unwrap()[0]
                .record
                .is_some()
        );
        let error = reader.read_batch(&[request]).unwrap_err();
        assert!(
            error.to_string().contains("stored search profiles differ"),
            "{error}"
        );
        c.execute(
            "UPDATE search_profiles SET identity_json=? WHERE profile_id=?",
            rusqlite::params![original, changed.profile_id.as_str()],
        )
        .unwrap();
        assert!(!reader.read_batch(&[request]).unwrap()[0].layouts.is_empty());
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
                !reader
                    .read_batch(&[SourceReuseRequest {
                        key,
                        record: false,
                        layouts: true
                    }])
                    .unwrap()[0]
                    .layouts
                    .is_empty(),
                "{expected:?}"
            );
        }
    }
}
