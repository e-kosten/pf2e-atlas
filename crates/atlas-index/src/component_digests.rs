//! Closed dependency identities for record-owned reference components.
use crate::{IndexError, PreparedRecordRef, codec::sha256};
use rusqlite::{Connection, OptionalExtension, params, types::Value};
use serde::{Deserialize, Serialize};

pub const COMPONENT_DIGEST_SCHEMA_SQL: &str = include_str!("component_digests.sql");
const RECIPE: &str = "closed-reference-components/v1";
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ComponentDigests {
    pub snapshot: String,
    pub query: String,
    pub content: String,
    pub relationships: String,
    pub sections: String,
    pub lexical: String,
    pub trait_associations: String,
    pub core: String,
}
fn hash(value: &impl Serialize) -> Result<String, IndexError> {
    Ok(sha256(&serde_json::to_vec(value)?))
}
#[derive(Serialize)]
enum Scalar<'a> {
    Null,
    Integer(i64),
    Real(u64),
    Text(&'a str),
    Blob(&'a [u8]),
}
fn scalar(value: &Value) -> Scalar<'_> {
    match value {
        Value::Null => Scalar::Null,
        Value::Integer(n) => Scalar::Integer(*n),
        Value::Real(n) => Scalar::Real(n.to_bits()),
        Value::Text(s) => Scalar::Text(s),
        Value::Blob(b) => Scalar::Blob(b),
    }
}
impl ComponentDigests {
    pub(crate) fn project(
        record: PreparedRecordRef<'_>,
        label: &str,
        audience: atlas_record::source_content::ContentAudience,
    ) -> Result<Self, IndexError> {
        let source = record.source;
        let mut ids = crate::projections::ProjectionIds::default();
        let rows = crate::projections::root_rows(source, label, 0, &mut ids)?;
        let query = rows
            .iter()
            .map(|row| {
                (
                    row.table,
                    row.columns
                        .iter()
                        .zip(&row.values)
                        .filter(|(column, _)| {
                            row.table != "records"
                                || !matches!(column.as_str(), "source_path" | "content_hash")
                        })
                        .map(|(column, value)| (column.as_str(), scalar(value)))
                        .collect::<Vec<_>>(),
                )
            })
            .collect::<Vec<_>>();
        let sections = crate::selected_sections::project(
            &source.record,
            record.selection,
            &source.content_hash,
            &source.preparation_context_hash,
        )?;
        let sections = sections
            .iter()
            .map(|s| (&s.part, &s.address))
            .collect::<Vec<_>>();
        let lexical = record
            .lexical
            .iter()
            .map(hash)
            .collect::<Result<std::collections::BTreeSet<_>, _>>()?;
        let mut evidence = crate::traits::TraitEvidence::default();
        evidence.add_record(&source.record, audience)?;
        let core =
            atlas_record::source_record::core_search_selection(&source.record, record.selection)
                .map_err(|e| IndexError::Invalid(e.to_string()))?;
        let retained = RetainedPayloadDigests::project(source, record.snapshot)?;
        Ok(Self {
            snapshot: retained.snapshot,
            query: hash(&(crate::SOURCE_PROJECTION_VERSION, query))?,
            content: retained.content,
            relationships: retained.relationships,
            sections: hash(&sections)?,
            lexical: hash(&lexical)?,
            trait_associations: hash(&evidence)?,
            core: hash(&(atlas_domain::CORE_SECTIONS_RECIPE, core))?,
        })
    }
    pub(crate) fn read(c: &Connection, id: i64) -> Result<Option<Self>, IndexError> {
        let row: Option<(String, String)> = c
            .query_row(
                "SELECT recipe,digests_json FROM reference_component_digests WHERE record_id=?",
                [id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()?;
        row.map(|(recipe, json)| {
            if recipe != RECIPE {
                return Err(IndexError::Invalid(
                    "component digest recipe differs".into(),
                ));
            }
            let value: Self = serde_json::from_str(&json)?;
            value.validate()?;
            Ok(value)
        })
        .transpose()
    }
    fn validate(&self) -> Result<(), IndexError> {
        for value in [
            &self.snapshot,
            &self.query,
            &self.content,
            &self.relationships,
            &self.sections,
            &self.lexical,
            &self.trait_associations,
            &self.core,
        ] {
            if !crate::codec::checked_hash(value) {
                return Err(IndexError::Invalid("component digest is invalid".into()));
            }
        }
        Ok(())
    }
    pub(crate) fn write(&self, c: &Connection, id: i64) -> Result<(), IndexError> {
        self.validate()?;
        c.execute("INSERT INTO reference_component_digests VALUES(?,?,?) ON CONFLICT(record_id) DO UPDATE SET recipe=excluded.recipe,digests_json=excluded.digests_json WHERE recipe IS NOT excluded.recipe OR digests_json IS NOT excluded.digests_json",params![id,RECIPE,serde_json::to_string(self)?])?;
        Ok(())
    }
}

/// Stable payload identities are valid under stored preparation policy. They do
/// not depend on current Query/Lexical/Core recipes, and do not select prose.
struct RetainedPayloadDigests {
    snapshot: String,
    content: String,
    relationships: String,
}
impl RetainedPayloadDigests {
    fn project(
        source: &crate::SourceArtifactRecordInput,
        snapshot: &[u8],
    ) -> Result<Self, IndexError> {
        let references = source
            .content
            .iter()
            .filter_map(|outcome| match &outcome.status {
                atlas_record::source_record::SourceContentStatus::Prepared(p) => {
                    Some((&p.locator, &p.references))
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        Ok(Self {
            snapshot: hash(&(
                RECIPE,
                snapshot,
                &source.source_path,
                &source.content_hash,
                &source.diagnostics,
            ))?,
            content: hash(&(&source.preparation_context_hash, &source.content))?,
            relationships: hash(&(&source.relationships, references))?,
        })
    }
}
/// Verify before retained bytes can become new preparation or maintenance input.
/// Missing entries remain historical unknowns; present proofs cannot be restamped
/// from codec-valid corrupted payloads. This has no recursive validation/read.
pub(crate) fn validate_retained_payload(
    c: &Connection,
    id: i64,
    source: &crate::SourceArtifactRecordInput,
    snapshot: &[u8],
) -> Result<(), IndexError> {
    let Some(stored) = ComponentDigests::read(c, id)? else {
        return Ok(());
    };
    let actual = RetainedPayloadDigests::project(source, snapshot)?;
    for (component, expected, actual) in [
        ("snapshot", stored.snapshot, actual.snapshot),
        ("content", stored.content, actual.content),
        ("relationships", stored.relationships, actual.relationships),
    ] {
        if expected != actual {
            return Err(IndexError::Invalid(format!(
                "retained {component} payload differs from its component attestation"
            )));
        }
    }
    Ok(())
}

/// Reconstruct each attested component from the exact publishing transaction.
/// Rich state is released between roots. Existing semantic validation must still
/// verify the actual projections; these hashes never replace it.
pub(crate) fn actual(
    c: &Connection,
    context: &crate::SourceArtifactBuildContext,
    id: i64,
) -> Result<ComponentDigests, IndexError> {
    let (key, label): (String, String) = c.query_row(
        "SELECT r.key,p.label FROM records r JOIN packs p USING(pack_id) WHERE record_id=?",
        [id],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;
    let key = key
        .parse()
        .map_err(|e| IndexError::Invalid(format!("component record key: {e}")))?;
    let source = crate::persisted_inputs::read_attested_persisted_input(c, context, &key)?
        .ok_or_else(|| IndexError::Invalid("component source is absent".into()))?;
    let selection = atlas_record::source_record::select_record_search(
        &source.record,
        &source.content,
        context.audience,
        &label,
        &context.used_trait_labels,
    )
    .map_err(|e| IndexError::Invalid(e.to_string()))?;
    let aliases = c
        .prepare("SELECT alias FROM verified_aliases WHERE record_id=? ORDER BY alias")?
        .query_map([id], |r| r.get(0))?
        .collect::<Result<Vec<String>, _>>()?;
    let lexical = crate::project_lexical_units(&selection, &aliases);
    let snapshot = atlas_foundry_model::encode_snapshot(source.record.source())?;
    ComponentDigests::project(
        PreparedRecordRef {
            source: &source,
            snapshot: &snapshot,
            selection: &selection,
            lexical: &lexical,
        },
        &label,
        context.audience,
    )
}
pub(crate) fn validate_present(
    c: &Connection,
    context: &crate::SourceArtifactBuildContext,
) -> Result<(), IndexError> {
    visit_ids(c, &mut |id| {
        let stored = ComponentDigests::read(c, id)?
            .ok_or_else(|| IndexError::Invalid("component digest disappeared".into()))?;
        if stored != actual(c, context, id)? {
            return Err(IndexError::Invalid(format!(
                "component digest differs from published record {id}"
            )));
        }
        Ok(())
    })
}
/// Metadata-only upgrades refresh existing attestations after rewriting their
/// actual dependent projections. Missing metadata stays explicitly unknown.
pub(crate) fn refresh_present(
    c: &Connection,
    context: &crate::SourceArtifactBuildContext,
) -> Result<(), IndexError> {
    visit_ids(c, &mut |id| actual(c, context, id)?.write(c, id))
}
fn visit_ids(
    c: &Connection,
    visitor: &mut dyn FnMut(i64) -> Result<(), IndexError>,
) -> Result<(), IndexError> {
    let mut after = 0_i64;
    loop {
        let ids=c.prepare("SELECT record_id FROM reference_component_digests WHERE record_id>? ORDER BY record_id LIMIT 128")?.query_map([after],|r|r.get::<_,i64>(0))?.collect::<Result<Vec<_>,_>>()?;
        if ids.is_empty() {
            return Ok(());
        }
        for id in ids {
            after = id;
            visitor(id)?;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn audit(c: &Connection) {
        c.execute_batch(
            "CREATE TABLE component_write_audit(component TEXT NOT NULL,record_id INTEGER)",
        )
        .unwrap();
        for (table, key) in [
            ("records", "record_id"),
            ("record_traits", "record_id"),
            ("actor_projection", "record_id"),
            ("physical_projection", "root_record_id"),
            ("spell_projection", "root_record_id"),
            ("record_bodies", "record_id"),
            ("prepared_content", "record_id"),
            ("relationship_occurrences", "record_id"),
            ("selected_sections", "record_id"),
            ("lexical_units", "record_id"),
            ("semantic_units", "record_id"),
            ("core_roots", "record_id"),
            ("core_lexical_units", "record_id"),
            ("core_semantic_locations", "record_id"),
            ("reference_component_digests", "record_id"),
        ] {
            for (event, row) in [("INSERT", "new"), ("UPDATE", "new"), ("DELETE", "old")] {
                c.execute_batch(&format!("CREATE TRIGGER component_audit_{table}_{event} AFTER {event} ON {table} BEGIN INSERT INTO component_write_audit VALUES('{table}',{row}.{key}); END;")).unwrap();
            }
        }
        for table in [
            "trait_definitions",
            "trait_vectors",
            "trait_memberships",
            "trait_references",
        ] {
            for event in ["INSERT", "UPDATE", "DELETE"] {
                c.execute_batch(&format!("CREATE TRIGGER component_audit_{table}_{event} AFTER {event} ON {table} BEGIN INSERT INTO component_write_audit VALUES('{table}',NULL); END;")).unwrap();
            }
        }
    }
    #[test]
    fn unchanged_publication_preserves_actual_component_rows() {
        let input = crate::foundation_tests::fixture(true);
        let (_dir, path) = crate::foundation_tests::build(&input);
        let c = Connection::open(&path).unwrap();
        audit(&c);
        crate::test_support::publish_input(&path, &input).unwrap();
        let writes: usize = c
            .query_row("SELECT count(*) FROM component_write_audit", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(
            writes, 0,
            "unchanged publication rewrote an owned component"
        );
        crate::validate_artifact(&path).unwrap();
    }
    #[test]
    fn one_body_change_preserves_unrelated_roots_and_query_rows() {
        let input = crate::foundation_tests::fixture(false);
        let (_dir, path) = crate::foundation_tests::build(&input);
        let c = Connection::open(&path).unwrap();
        audit(&c);
        let changed = crate::foundation_tests::fixture_with_description(
            false,
            "<h2>Flame</h2><p>Changed independent body.</p>",
        );
        crate::test_support::publish_input(&path, &changed).unwrap();
        let rows=c.prepare("SELECT DISTINCT component,record_id FROM component_write_audit ORDER BY component,record_id").unwrap().query_map([],|r|Ok((r.get::<_,String>(0)?,r.get::<_,Option<i64>>(1)?))).unwrap().collect::<Result<Vec<_>,_>>().unwrap();
        let id: i64 = c
            .query_row(
                "SELECT record_id FROM records WHERE key='items:bbbbbbbbbbbbbbbb'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert!(!rows.is_empty());
        assert!(
            rows.iter().all(|(component, root)| root == &Some(id)
                && !matches!(
                    component.as_str(),
                    "record_traits"
                        | "actor_projection"
                        | "physical_projection"
                        | "spell_projection"
                )),
            "unrelated/query writes: {rows:?}"
        );
        crate::validate_artifact(&path).unwrap();
    }
    #[test]
    fn each_present_component_hash_is_independently_checked() {
        let input = crate::foundation_tests::fixture(false);
        let (_dir, path) = crate::foundation_tests::build(&input);
        let c = Connection::open(&path).unwrap();
        let (id, json): (i64, String) = c
            .query_row(
                "SELECT record_id,digests_json FROM reference_component_digests LIMIT 1",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        let original: serde_json::Value = serde_json::from_str(&json).unwrap();
        for field in [
            "snapshot",
            "query",
            "content",
            "relationships",
            "sections",
            "lexical",
            "trait_associations",
            "core",
        ] {
            let mut damaged = original.clone();
            damaged[field] = serde_json::json!("0".repeat(64));
            c.execute(
                "UPDATE reference_component_digests SET digests_json=? WHERE record_id=?",
                params![damaged.to_string(), id],
            )
            .unwrap();
            let error = crate::validate_artifact(&path).unwrap_err();
            assert!(
                error.to_string().contains("component digest differs")
                    || error.to_string().contains("component attestation"),
                "{field}: {error}"
            );
        }
        c.execute(
            "UPDATE reference_component_digests SET digests_json=? WHERE record_id=?",
            params![json, id],
        )
        .unwrap();
        crate::validate_artifact(&path).unwrap();
    }
    #[test]
    fn absent_digest_never_attests_corrupt_retained_preparation() {
        let input = crate::foundation_tests::fixture(false);
        let (_dir, path) = crate::foundation_tests::build(&input);
        let c = Connection::open(&path).unwrap();
        c.execute("DELETE FROM reference_component_digests", [])
            .unwrap();
        c.execute(
            "UPDATE prepared_content SET preparation_context_hash=?",
            ["0".repeat(64)],
        )
        .unwrap();
        assert!(crate::test_support::publish_input(&path, &input).is_err());
        let count: usize = c
            .query_row(
                "SELECT count(*) FROM reference_component_digests",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(
            count, 0,
            "failed publication stamped desired digests over corrupt retained state"
        );
    }
    #[test]
    fn populated_schema25_upgrade_compares_actual_prior_rows_without_rewriting_them() {
        use crate::test_support::*;
        let input = crate::foundation_tests::fixture(false);
        atlas_sqlite_vec::register_sqlite_vec_auto_extension().unwrap();
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("schema25.sqlite");
        let workspace = atlas_storage::Workspace::at(&path).unwrap();
        workspace
            .migrate(&TEST_MIGRATIONS[..24], |_| Ok(()))
            .unwrap();
        let owner = workspace.update_owner().unwrap();
        let maintenance = owner
            .maintenance(24, "populating actual schema24 source fixture")
            .unwrap();
        publish_pre_section_fixture(&maintenance, &input).unwrap();
        drop(maintenance);
        drop(owner);
        workspace
            .migrate(&TEST_MIGRATIONS[..25], |_| Ok(()))
            .unwrap();
        // The physical section migration has rebuilt actual selected rows. This
        // fixture activates that independently versioned marker at schema25.
        let write = workspace.write(25).unwrap();
        write
            .connection()
            .execute(
                "UPDATE artifact_context SET context_json=?,context_hash=?",
                params![
                    serde_json::to_string(&input.context).unwrap(),
                    input.context.sha256().unwrap()
                ],
            )
            .unwrap();
        write.commit().unwrap();
        workspace.migrate(TEST_MIGRATIONS, |_| Ok(())).unwrap();
        let c = Connection::open(&path).unwrap();
        assert_eq!(
            c.query_row(
                "SELECT count(*) FROM reference_component_digests",
                [],
                |r| r.get::<_, usize>(0)
            )
            .unwrap(),
            0
        );
        audit(&c);
        crate::test_support::publish_input(&path, &input).unwrap();
        let writes:usize=c.query_row("SELECT count(*) FROM component_write_audit WHERE component<>'reference_component_digests'",[],|r|r.get(0)).unwrap();
        assert_eq!(
            writes, 0,
            "first post26 publication rewrote unchanged actual components"
        );
        crate::validate_reference(&c).unwrap();
    }
    #[test]
    fn admission_diagnostic_only_change_updates_published_evidence() {
        let mut input = crate::foundation_tests::fixture(false);
        let (_dir, path) = crate::foundation_tests::build(&input);
        input.records[0]
            .diagnostics
            .push(crate::SourceArtifactDiagnostic {
                source_diagnostic: None,
                owners: None,
                field: Some("/system/example".into()),
                stage: "admission".into(),
                code: "fixture_diagnostic".into(),
                message: "Changed retained admission evidence".into(),
            });
        crate::test_support::publish_input(&path, &input).unwrap();
        let c = Connection::open(&path).unwrap();
        let context = crate::read_stored_reference_context(&c).unwrap().unwrap();
        let restored = crate::persisted_inputs::read_attested_persisted_input(
            &c,
            &context,
            input.records[0].record.key(),
        )
        .unwrap()
        .unwrap();
        assert_eq!(restored.diagnostics, input.records[0].diagnostics);
        crate::validate_reference(&c).unwrap();
    }
}

#[cfg(test)]
mod retained_payload_tests {
    use super::*;
    #[test]
    fn query_maintenance_rejects_retained_cache_corruption_without_restamping_proofs() {
        let input = crate::foundation_tests::fixture(false);
        let (_dir, path) = crate::foundation_tests::build(&input);
        let c = Connection::open(&path).unwrap();
        c.execute_batch("INSERT INTO saved_lists(id,list_key,slug,name,created_at,updated_at) VALUES(1,'kept','kept','Kept maintenance notes','now','now')").unwrap();
        let mut context = input.context.clone();
        context.query_projection_version = "authored-query/v4".into();
        let json = serde_json::to_string(&context).unwrap();
        c.execute(
            "UPDATE artifact_context SET context_json=?,context_hash=?",
            params![json, context.sha256().unwrap()],
        )
        .unwrap();
        crate::test_support::alter_retained_html_for_fixture(
            &c,
            input.records[1].record.key(),
            "/system/description/value",
            "Dragon breath.",
            "Changed cached breath.",
        )
        .unwrap();
        let revision: u64 = c
            .query_row("SELECT reference_revision FROM workspace", [], |r| r.get(0))
            .unwrap();
        let proofs = c
            .prepare(
                "SELECT record_id,digests_json FROM reference_component_digests ORDER BY record_id",
            )
            .unwrap()
            .query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        let workspace = atlas_storage::Workspace::at(&path).unwrap();
        let error = crate::ensure_reference_projections(
            &workspace,
            crate::test_support::TEST_WORKSPACE_SCHEMA_VERSION,
        )
        .unwrap_err();
        assert!(
            error.to_string().contains("component attestation"),
            "{error}"
        );
        assert_eq!(
            c.query_row("SELECT reference_revision FROM workspace", [], |r| r
                .get::<_, u64>(0))
                .unwrap(),
            revision
        );
        assert_eq!(
            c.query_row("SELECT context_json FROM artifact_context", [], |r| r
                .get::<_, String>(0))
                .unwrap(),
            json
        );
        let after = c
            .prepare(
                "SELECT record_id,digests_json FROM reference_component_digests ORDER BY record_id",
            )
            .unwrap()
            .query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(after, proofs);
        assert_eq!(
            c.query_row("SELECT name FROM saved_lists WHERE id=1", [], |r| r
                .get::<_, String>(0))
                .unwrap(),
            "Kept maintenance notes"
        );
    }
    #[test]
    fn historical_pre_attestation_entry_rejects_current_schema_before_decoding() {
        let input = crate::foundation_tests::fixture(false);
        let (_dir, path) = crate::foundation_tests::build(&input);
        let c = Connection::open(path).unwrap();
        c.execute("UPDATE record_bodies SET snapshot=x'00'", [])
            .unwrap();
        let error = crate::persisted_inputs::read_pre_component_digest_migration_input(
            &c,
            &input.context,
            input.records[0].record.key(),
        )
        .unwrap_err();
        assert!(
            error
                .to_string()
                .contains("restricted to historical migration25"),
            "{error}"
        );
    }
}

#[cfg(test)]
mod historical_payload_tests {
    use super::*;
    fn bodies(c: &Connection) -> Vec<(String, String)> {
        c.prepare("SELECT r.key,b.snapshot FROM records r JOIN record_bodies b USING(record_id) ORDER BY r.key").unwrap().query_map([],|r|Ok((r.get(0)?,crate::codec::sha256(&r.get::<_,Vec<u8>>(1)?)))).unwrap().collect::<Result<_,_>>().unwrap()
    }
    fn vectors(c: &Connection, table: &str) -> Vec<(i64, String)> {
        c.prepare(&format!(
            "SELECT unit_id,embedding FROM {table} ORDER BY unit_id"
        ))
        .unwrap()
        .query_map([], |r| {
            Ok((r.get(0)?, crate::codec::sha256(&r.get::<_, Vec<u8>>(1)?)))
        })
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap()
    }
    #[test]
    fn populated_pre20_upgrade_preserves_payloads_through_frozen20_and_guarded25() {
        use crate::test_support::*;
        let input = crate::foundation_tests::fixture(true);
        atlas_sqlite_vec::register_sqlite_vec_auto_extension().unwrap();
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("pre20.sqlite");
        let workspace = atlas_storage::Workspace::at(&path).unwrap();
        workspace
            .migrate(&TEST_MIGRATIONS[..19], |_| Ok(()))
            .unwrap();
        let owner = workspace.update_owner().unwrap();
        let maintenance = owner
            .maintenance(19, "actual populated pre20 fixture")
            .unwrap();
        publish_pre_core_fixture(&maintenance, &input).unwrap();
        drop(maintenance);
        drop(owner);
        let table = crate::schema::vector_table(&input.context.search_profiles[0]).unwrap();
        let c = Connection::open(&path).unwrap();
        c.execute_batch("INSERT INTO saved_lists(id,list_key,slug,name,created_at,updated_at) VALUES(1,'kept','kept','Kept pre20 state','now','now')").unwrap();
        let old_bodies = bodies(&c);
        let old_vectors = vectors(&c, &table);
        assert!(!old_bodies.is_empty());
        assert!(!old_vectors.is_empty());
        workspace.migrate(TEST_MIGRATIONS, |_| Ok(())).unwrap();
        assert!(
            crate::ensure_reference_projections(&workspace, TEST_WORKSPACE_SCHEMA_VERSION).unwrap()
        );
        assert_eq!(bodies(&c), old_bodies);
        assert_eq!(vectors(&c, &table), old_vectors);
        assert_eq!(
            c.query_row("SELECT name FROM saved_lists WHERE id=1", [], |r| r
                .get::<_, String>(0))
                .unwrap(),
            "Kept pre20 state"
        );
        assert_eq!(
            c.query_row(
                "SELECT count(*) FROM reference_component_digests",
                [],
                |r| r.get::<_, usize>(0)
            )
            .unwrap(),
            0
        );
        crate::validate_artifact(&path).unwrap();
    }
    #[test]
    fn current_reader_does_not_treat_missing_attestation_storage_as_historical() {
        let input = crate::foundation_tests::fixture(false);
        let (_dir, path) = crate::foundation_tests::build(&input);
        let c = Connection::open(path).unwrap();
        c.execute_batch("DROP TABLE reference_component_digests")
            .unwrap();
        let error = crate::persisted_inputs::read_attested_persisted_input(
            &c,
            &input.context,
            input.records[0].record.key(),
        )
        .unwrap_err();
        assert!(
            error
                .to_string()
                .contains("no such table: reference_component_digests"),
            "{error}"
        );
        c.execute("UPDATE record_bodies SET snapshot=x'00'", [])
            .unwrap();
        let error = crate::persisted_inputs::read_persisted_input(
            &c,
            &input.context,
            input.records[0].record.key(),
        )
        .unwrap_err();
        assert!(
            error
                .to_string()
                .contains("restricted to historical migration20"),
            "{error}"
        );
    }
}
