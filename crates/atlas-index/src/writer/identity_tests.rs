//! Admission-derived identity facts keep metadata consumers off rich streams.
use super::*;
use crate::{test_support::*, *};
use atlas_record::source_record::select_record_search;
use std::cell::Cell;

struct Counted<'a> {
    source: &'a dyn PublicationSource,
    traversals: Cell<usize>,
    visits: Cell<usize>,
}
macro_rules! delegate_stream {
    ($method:ident, $row:ty) => {
        fn $method(
            &self,
            visitor: &mut dyn FnMut(&$row) -> Result<(), IndexError>,
        ) -> Result<(), IndexError> {
            self.source.$method(visitor)
        }
    };
}
impl PublicationSource for Counted<'_> {
    fn context(&self) -> &SourceArtifactBuildContext {
        self.source.context()
    }
    fn packs(&self) -> &[IndexBuildPack] {
        self.source.packs()
    }
    fn inventory(&self) -> &CompleteSourceInventory {
        self.source.inventory()
    }
    fn record_count(&self) -> usize {
        self.source.record_count()
    }
    fn semantic_count(&self) -> usize {
        self.source.semantic_count()
    }
    fn expected_instance(&self) -> Option<&ArtifactInstance> {
        self.source.expected_instance()
    }
    fn check_seal(&self) -> Result<(), IndexError> {
        self.source.check_seal()
    }
    fn records(
        &self,
        visitor: &mut dyn FnMut(PreparedRecordRef<'_>) -> Result<(), IndexError>,
    ) -> Result<(), IndexError> {
        self.traversals.set(self.traversals.get() + 1);
        self.source.records(&mut |record| {
            self.visits.set(self.visits.get() + 1);
            visitor(record)
        })
    }
    delegate_stream!(lexical, SourceLexicalUnitInput);
    delegate_stream!(semantic, SemanticOccurrence);
    delegate_stream!(aliases, SourceAliasInput);
    delegate_stream!(remaster_pairs, SourceRemasterPairInput);
    delegate_stream!(traits, TraitDefinition);
    delegate_stream!(trait_seeds, TraitSeedOccurrence);
}
fn fixture() -> IndexBuildInput {
    let mut input = crate::foundation_tests::fixture(false);
    input.records.reverse();
    input.records.insert(
        1,
        crate::foundation_tests::record(
            "items",
            "Item",
            serde_json::json!({"_id":"zzzzzzzzzzzzzzzz","type":"effect"}),
        ),
    );
    input.records.push(crate::foundation_tests::record(
        "items",
        "Item",
        serde_json::json!({"_id":"eeeeeeeeeeeeeeee","type":"effect","name":""}),
    ));
    input.context.source_file_count = input.records.len();
    input.lexical_units.clear();
    for source in &mut input.records {
        source.preparation_context_hash = input.context.preparation_sha256().unwrap();
        let label = &input
            .packs
            .iter()
            .find(|p| p.pack_id == source.record.key().pack().as_str())
            .unwrap()
            .label;
        let selection = select_record_search(
            &source.record,
            &source.content,
            input.context.audience,
            label,
            &input.context.used_trait_labels,
        )
        .unwrap();
        input
            .lexical_units
            .extend(project_lexical_units(&selection, &[]));
    }
    input
}
fn spool(
    input: IndexBuildInput,
    expected: ArtifactInstance,
    damage: Option<&str>,
) -> SealedPublicationSpool {
    let mut builder =
        PublicationSpool::new(input.context, input.packs, input.inventory, expected).unwrap();
    let count = input.records.len();
    for (ordinal, source) in input.records.into_iter().enumerate() {
        let label = &builder
            .packs
            .iter()
            .find(|p| p.pack_id == source.record.key().pack().as_str())
            .unwrap()
            .label;
        let selection = select_record_search(
            &source.record,
            &source.content,
            builder.context.audience,
            label,
            &builder.context.used_trait_labels,
        )
        .unwrap();
        let lexical = input
            .lexical_units
            .iter()
            .filter(|row| row.record == *source.record.key())
            .cloned()
            .collect();
        let mut snapshot = String::from_utf8(
            atlas_foundry_model::encode_snapshot(source.record.source()).unwrap(),
        )
        .unwrap();
        if ordinal + 1 == count {
            snapshot = match damage {
                Some("contract") => snapshot.replace(
                    atlas_foundry_model::SOURCE_CONTRACT_ID,
                    "retired-fixture-contract",
                ),
                Some("root") => snapshot.replace("eeeeeeeeeeeeeeee", "qqqqqqqqqqqqqqqq"),
                None => snapshot,
                Some(other) => panic!("unknown corruption {other}"),
            };
        }
        builder
            .records
            .push(&PreparedRecordFrame {
                pack: source.record.key().pack().as_str().into(),
                snapshot,
                source_path: source.source_path,
                content_hash: source.content_hash,
                preparation_context_hash: source.preparation_context_hash,
                content: source.content,
                relationships: source.relationships,
                diagnostics: source.diagnostics,
                selection,
                lexical,
            })
            .unwrap();
    }
    for row in input.lexical_units {
        builder.lexical.push(&row).unwrap();
    }
    builder.seal().unwrap()
}
fn origin(workspace: &atlas_storage::Workspace) -> ArtifactInstance {
    let read = workspace.read(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
    ArtifactInstance::new(
        read.workspace_key().unwrap(),
        read.reference_revision().unwrap(),
    )
}
fn publish(
    workspace: &atlas_storage::Workspace,
    source: &dyn PublicationSource,
) -> ArtifactValidationReport {
    let owner = workspace.update_owner().unwrap();
    let job =
        register_native_target(&owner, workspace, TEST_WORKSPACE_SCHEMA_VERSION, source).unwrap();
    let values = freeze_native_values(
        &owner,
        workspace,
        TEST_WORKSPACE_SCHEMA_VERSION,
        &job,
        source,
    )
    .unwrap();
    let maintenance = owner
        .maintenance(TEST_WORKSPACE_SCHEMA_VERSION, "checked identity test")
        .unwrap();
    SqliteIndexWriter::new(&maintenance)
        .write_job(source, &values, &job, &source.context().sha256().unwrap())
        .unwrap()
}
fn stable_rows(c: &Connection) -> std::collections::BTreeMap<String, Vec<Vec<Value>>> {
    let mut rows = reference_preservation_rows(c).unwrap();
    for operation in [
        "reference_update_jobs",
        "native_update_targets",
        "native_target_inputs",
    ] {
        rows.remove(operation);
    }
    rows
}
#[test]
fn production_publication_bounds_rich_visits_and_preserves_stream_order_optional_names_and_saved_rows()
 {
    let directory = tempfile::tempdir().unwrap();
    let workspace = initialize_workspace(directory.path().join("identities.sqlite")).unwrap();
    let mut previous = None;
    for iteration in 0..2 {
        let source = spool(fixture(), origin(&workspace), None);
        let counted = Counted {
            source: &source,
            traversals: Cell::new(0),
            visits: Cell::new(0),
        };
        let report = publish(&workspace, &counted);
        assert_eq!(report.roots, 6);
        eprintln!(
            "rich_publication iteration={iteration} traversals={} visits={} roots={}",
            counted.traversals.get(),
            counted.visits.get(),
            source.record_count()
        );
        assert!(
            counted.traversals.get() <= 4 && counted.visits.get() <= 4 * source.record_count(),
            "identity consumers must not rehydrate complete prepared records: traversals={} visits={}",
            counted.traversals.get(),
            counted.visits.get()
        );
        let read = workspace.read(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
        let c = read.connection();
        let identities = c
            .prepare("SELECT key,last_known_name FROM record_identity ORDER BY id")
            .unwrap()
            .query_map([], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, Option<String>>(1)?))
            })
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(
            identities,
            vec![
                (
                    "macros:dddddddddddddddd".into(),
                    Some("Hidden Dragon".into())
                ),
                ("items:zzzzzzzzzzzzzzzz".into(), None),
                ("items:cccccccccccccccc".into(), Some("Dragon Spell".into())),
                ("items:bbbbbbbbbbbbbbbb".into(), Some("Dragon Staff".into())),
                (
                    "actors:aaaaaaaaaaaaaaaa".into(),
                    Some("Dragon Guard".into())
                ),
                ("items:eeeeeeeeeeeeeeee".into(), Some("".into())),
            ]
        );
        assert_eq!(c.query_row("SELECT count(*) FROM core_roots WHERE record_id=(SELECT id FROM record_identity WHERE key='macros:dddddddddddddddd')",[],|r|r.get::<_,usize>(0)).unwrap(),0);
        if let Some(previous) = &previous {
            assert_eq!(&stable_rows(c), previous);
        }
        drop(read);
        if iteration == 0 {
            let write = workspace.write(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
            write.connection().execute_batch("INSERT INTO saved_lists(id,list_key,slug,name,created_at,updated_at) VALUES(1,'kept','kept','Keep note','now','now'); INSERT INTO saved_list_items(list_id,record_key,position,note,record_title_snapshot,added_at,updated_at) VALUES(1,'actors:aaaaaaaaaaaaaaaa',0,'Retain note','Dragon Guard','now','now')").unwrap();
            write.commit().unwrap();
            previous = Some(stable_rows(
                workspace
                    .read(TEST_WORKSPACE_SCHEMA_VERSION)
                    .unwrap()
                    .connection(),
            ));
        }
    }
}
#[test]
fn sealed_late_snapshot_contract_or_root_error_precedes_reference_mutation() {
    for (damage, expected_error) in [
        ("contract", "Unsupported source contract"),
        ("root", "selection belongs to another root"),
    ] {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("late-corruption.sqlite");
        let workspace = initialize_workspace(&path).unwrap();
        publish_input(&path, &fixture()).unwrap();
        let source = spool(fixture(), origin(&workspace), Some(damage));
        source.check_seal().unwrap();
        let owner = workspace.update_owner().unwrap();
        let job =
            register_native_target(&owner, &workspace, TEST_WORKSPACE_SCHEMA_VERSION, &source)
                .unwrap();
        let values = freeze_native_values(
            &owner,
            &workspace,
            TEST_WORKSPACE_SCHEMA_VERSION,
            &job,
            &source,
        )
        .unwrap();
        let maintenance = owner
            .maintenance(TEST_WORKSPACE_SCHEMA_VERSION, "late damaged input")
            .unwrap();
        maintenance.connection().execute_batch("CREATE TRIGGER reject_reference_mutation BEFORE INSERT ON artifact_context BEGIN SELECT RAISE(ABORT,'unexpected reference mutation'); END;").unwrap();
        let before = reference_preservation_rows(maintenance.connection()).unwrap();
        let error = SqliteIndexWriter::new(&maintenance)
            .write_job(&source, &values, &job, &source.context().sha256().unwrap())
            .unwrap_err();
        assert!(
            error.to_string().contains(expected_error),
            "{damage}: {error}"
        );
        assert_eq!(
            reference_preservation_rows(maintenance.connection()).unwrap(),
            before
        );
        assert_eq!(
            maintenance
                .connection()
                .query_row("SELECT reference_revision FROM workspace", [], |r| r
                    .get::<_, u64>(0))
                .unwrap(),
            1
        );
    }
}
