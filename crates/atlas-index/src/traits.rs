//! Persisted trait vocabulary and bounded, exact authored associations.
use crate::{IndexError, SqliteIndexReader};
use atlas_domain::OwnedContentLocator;
use atlas_domain::{RecordKey, SourceRecordSummary, TraitKey};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TraitDefinition {
    pub key: TraitKey,
    pub label: String,
    pub label_key: Option<String>,
    pub description_key: Option<String>,
    pub description_html: Option<String>,
    pub authored_markup: Option<String>,
    pub prepared: Option<atlas_record::source_content::PreparedReferenceDefinition>,
    pub catalogs: Vec<String>,
    pub source_locations: Vec<crate::ReferenceSourceLocation>,
    pub source_revision: Option<String>,
    pub indexing_locale: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_locale: Option<String>,
}
#[derive(Debug, Clone)]
pub struct TraitMembership {
    pub source_label: Option<String>,
    pub record: SourceRecordSummary,
    pub owners: Vec<OwnedContentLocator>,
}

fn write_trait_definition(
    c: &Connection,
    definition: &TraitDefinition,
    kept: &mut BTreeSet<String>,
) -> Result<(), IndexError> {
    let key = definition.key.to_string();
    if !kept.insert(key.clone()) {
        return Err(IndexError::Invalid("duplicate trait definition".into()));
    }
    let entry = crate::ReferenceEntry::from_trait(definition)?;
    crate::references::write_reference_entry(c, &entry, None)
}

pub(crate) fn write_record_trait_relations(
    c: &Connection,
    record: &RecordKey,
    evidence: TraitEvidence,
) -> Result<(), IndexError> {
    let current = read_record_memberships(c, record)?;
    for (value, record, owners, lookup_member, label) in evidence.memberships.difference(&current) {
        c.execute(
            "INSERT INTO trait_members VALUES(?,?,?,?,?)
             ON CONFLICT(value,record_key,owners_json) DO UPDATE SET
                 lookup_member=excluded.lookup_member,source_label=excluded.source_label",
            params![value, record, owners, lookup_member, label],
        )?;
    }
    for (value, record, owners, lookup_member, label) in current.difference(&evidence.memberships) {
        c.execute(
            "DELETE FROM trait_members WHERE value=? AND record_key=? AND owners_json=?
             AND lookup_member=? AND source_label IS ?",
            params![value, record, owners, lookup_member, label],
        )?;
    }
    Ok(())
}

fn read_record_memberships(
    c: &Connection,
    record: &RecordKey,
) -> Result<BTreeSet<Member>, IndexError> {
    Ok(c.prepare_cached("SELECT value,record_key,owners_json,lookup_member,source_label FROM trait_members WHERE record_key=?")?
        .query_map([record.to_string()], read_member)?
        .collect::<Result<_, _>>()?)
}

pub(crate) fn validate_definition_targets(
    c: &Connection,
    definition: &TraitDefinition,
) -> Result<(), IndexError> {
    use atlas_record::source_content::{ContentReferenceResolution, ContentReferenceTarget};

    let Some(prepared) = &definition.prepared else {
        return Ok(());
    };
    for occurrence in prepared
        .references
        .iter()
        .filter(|reference| reference.visible)
    {
        let ContentReferenceResolution::Resolved(target) = &occurrence.resolution else {
            continue;
        };
        let exists = match target {
            ContentReferenceTarget::Record { key } => c.query_row(
                "SELECT EXISTS(SELECT 1 FROM records WHERE key=? AND document_kind<>'Macro')",
                [key.to_string()],
                |row| row.get(0),
            )?,
            ContentReferenceTarget::OwnedNode { key, owners } => {
                !owners.is_empty()
                    && crate::read_product_source_record(c, key)?
                        .is_some_and(|record| record.node_at(owners).is_some())
            }
            ContentReferenceTarget::Url { .. } => true,
        };
        if !exists {
            return Err(IndexError::Invalid(format!(
                "resolved inline target of {} is absent",
                definition.key
            )));
        }
    }
    Ok(())
}

pub(crate) fn write_trait_definitions(
    c: &Connection,
    input: &crate::PublicationSpool,
) -> Result<BTreeSet<String>, IndexError> {
    let mut kept = BTreeSet::new();
    input.traits(&mut |definition| write_trait_definition(c, definition, &mut kept))?;
    Ok(kept)
}

pub(crate) fn prune_trait_definitions(
    c: &Connection,
    kept: &BTreeSet<String>,
) -> Result<(), IndexError> {
    let old = c
        .prepare("SELECT key FROM reference_entries WHERE key LIKE 'trait:%'")?
        .query_map([], |r| r.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    for key in old {
        if !kept.contains(&key) {
            c.execute("DELETE FROM reference_entries WHERE key=?", [key])?;
        }
    }
    Ok(())
}
impl SqliteIndexReader {
    pub fn trait_definition(&self, key: &TraitKey) -> Result<Option<TraitDefinition>, IndexError> {
        crate::references::read_entry(
            self.native()?,
            &atlas_domain::DefinitionKey::from(key.clone()),
        )?
        .map(|entry| entry.trait_view())
        .transpose()
    }

    pub fn trait_memberships(
        &self,
        key: &TraitKey,
        limit: u32,
        offset: u32,
    ) -> Result<(Vec<TraitMembership>, u64), IndexError> {
        if limit == 0 || limit > 100 {
            return Err(IndexError::Invalid("trait page size must be 1..100".into()));
        }
        let sql = "SELECT record_key,owners_json AS owners,source_label FROM trait_members WHERE value=?1 AND lookup_member=1";
        let total = self.native()?.query_row(
            &format!("SELECT count(*) FROM ({sql})"),
            [key.identifier()],
            |r| r.get(0),
        )?;
        let mut stmt = self.native()?.prepare(&format!(
            "SELECT record_key,owners,source_label FROM ({sql}) ORDER BY record_key,owners LIMIT ?2 OFFSET ?3"
        ))?;
        let rows = stmt
            .query_map(params![key.identifier(), limit, offset], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, Option<String>>(2)?,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        let rows = rows
            .into_iter()
            .map(|(record, owners, source_label)| {
                let record: RecordKey = record
                    .parse()
                    .map_err(|e| IndexError::Invalid(format!("invalid membership key: {e}")))?;
                Ok((record, owners, source_label))
            })
            .collect::<Result<Vec<_>, IndexError>>()?;
        let summaries = self.trait_page_summaries(rows.iter().map(|(key, _, _)| key))?;
        let mut entries = Vec::new();
        for (record, owners, source_label) in rows {
            if let Some(record) = summaries.get(&record) {
                let owners = if owners == "[]" {
                    Vec::new()
                } else {
                    serde_json::from_str(&owners)?
                };
                entries.push(TraitMembership {
                    record: record.clone(),
                    owners,
                    source_label,
                });
            }
        }
        Ok((entries, total))
    }
    fn trait_page_summaries<'a>(
        &self,
        keys: impl Iterator<Item = &'a RecordKey>,
    ) -> Result<BTreeMap<RecordKey, SourceRecordSummary>, IndexError> {
        let keys = keys
            .collect::<BTreeSet<_>>()
            .into_iter()
            .cloned()
            .collect::<Vec<_>>();
        Ok(self
            .read_summaries(&keys)?
            .records
            .into_iter()
            .map(|record| (record.key.clone(), record))
            .collect())
    }
}

type Member = (String, String, String, bool, Option<String>);

fn read_member(row: &rusqlite::Row<'_>) -> rusqlite::Result<Member> {
    Ok((
        row.get(0)?,
        row.get(1)?,
        row.get(2)?,
        row.get(3)?,
        row.get(4)?,
    ))
}

#[derive(Default)]
pub(crate) struct TraitEvidence {
    memberships: BTreeSet<Member>,
}

impl TraitEvidence {
    pub(crate) fn digest(&self) -> Result<String, IndexError> {
        Ok(crate::codec::sha256(&serde_json::to_vec(&(
            "trait-memberships/v1",
            &self.memberships,
        ))?))
    }

    pub(crate) fn validate_record(
        &self,
        c: &Connection,
        key: &RecordKey,
    ) -> Result<(), IndexError> {
        if read_record_memberships(c, key)? != self.memberships {
            return Err(IndexError::Invalid(format!(
                "trait memberships differ from retained snapshot for {key}"
            )));
        }
        Ok(())
    }

    pub(crate) fn add_record(
        &mut self,
        record: &atlas_record::source_record::SourceBackedRecord,
        audience: atlas_record::source_content::ContentAudience,
    ) -> Result<(), IndexError> {
        for membership in atlas_record::source_record::trait_record_memberships(record, audience) {
            self.memberships.insert((
                membership.identifier,
                record.key().to_string(),
                serde_json::to_string(&membership.owners)?,
                membership.lookup_member,
                if membership.owners.is_empty() {
                    None
                } else {
                    membership.source_label
                },
            ));
        }
        Ok(())
    }

    pub(crate) fn validate(self, c: &Connection) -> Result<(), IndexError> {
        let actual = c
            .prepare(
                "SELECT value,record_key,owners_json,lookup_member,source_label FROM trait_members",
            )?
            .query_map([], read_member)?
            .collect::<Result<BTreeSet<_>, _>>()?;
        if actual != self.memberships {
            return Err(IndexError::Invalid(
                "trait memberships differ from checked source".into(),
            ));
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    fn refresh_fixture(input: &mut crate::IndexBuildInput) {
        input.lexical_units.clear();
        let hash = input.context.preparation_sha256().unwrap();
        for source in &mut input.records {
            source.preparation_context_hash = hash.clone();
            let label = &input
                .packs
                .iter()
                .find(|p| p.pack_id == source.record.key().pack().as_str())
                .unwrap()
                .label;
            let selection = atlas_record::source_record::select_record_search(
                &source.record,
                &source.content,
                input.context.audience,
                label,
                &input.context.used_trait_labels,
            )
            .unwrap();
            input
                .lexical_units
                .extend(crate::project_lexical_units(&selection, &[]));
        }
    }
    fn definition(input: &crate::IndexBuildInput, id: &str) -> TraitDefinition {
        TraitDefinition {
            key: TraitKey::new(id).unwrap(),
            label: id.into(),
            label_key: None,
            description_key: None,
            description_html: None,
            authored_markup: None,
            prepared: None,
            catalogs: vec![],
            source_locations: vec![],
            source_revision: input.context.source_revision.clone(),
            indexing_locale: input.context.indexing_locale.clone(),
            content_locale: None,
        }
    }
    #[test]
    fn removed_roots_prune_memberships_without_touching_live_associations() {
        use atlas_record::source_content::prepare_reference_definition;
        use atlas_record::source_record::SourceReferenceIndex;
        fn input(keep_removed: bool) -> crate::IndexBuildInput {
            let mut input = crate::foundation_tests::fixture(false);
            for (index, id, name) in [
                (1, "bbbbbbbbbbbbbbbb", "Removed staff"),
                (2, "cccccccccccccccc", "Kept spell"),
            ] {
                input.records[index] = crate::foundation_tests::record(
                    "items",
                    "Item",
                    json!({"_id":id,"name":name,"type":"spell","system":{"traits":{"value":["fire"]},"description":{"value":"<p>@Localize[PF2E.TraitDragon]</p>"}}}),
                );
            }
            if !keep_removed {
                input.records.retain(|r| {
                    !matches!(
                        r.record.key().to_string().as_str(),
                        "actors:aaaaaaaaaaaaaaaa" | "items:bbbbbbbbbbbbbbbb"
                    )
                });
            }
            let mut resolver = SourceReferenceIndex::default();
            for source in &input.records {
                resolver.insert_source(source.record.key(), source.record.source());
            }
            let mut dragon = definition(&input, "dragon");
            let resolve = |kind: &atlas_record::source_content::ContentReferenceKind,
                           target: &str| {
                resolver.resolve_global_content_reference(kind, target)
            };
            let markup = "<p>@UUID[Compendium.pf2e.actors.Actor.aaaaaaaaaaaaaaaa]{Owner}</p>";
            let prepared = prepare_reference_definition(
                dragon.key.clone().into(),
                markup,
                input.context.audience,
                None,
                Some(&resolve),
            )
            .unwrap();
            dragon.authored_markup = Some(markup.into());
            dragon.description_html = Some(prepared.html.clone());
            dragon.description_key = Some("fixture.dragon.description".into());
            dragon.prepared = Some(prepared);
            input.traits.push(dragon);
            refresh_fixture(&mut input);
            input
        }
        let initial = input(true);
        let (_directory, path) = crate::foundation_tests::build(&initial);
        let c = Connection::open(&path).unwrap();
        c.execute_batch("INSERT INTO saved_lists(id,owner_id,list_key,slug,name,created_at,updated_at) VALUES(1,1,'kept','kept','Saved trait owner','now','now'); INSERT INTO saved_list_items(list_id,record_key,position,note,record_title_snapshot,added_at,updated_at) VALUES(1,'actors:aaaaaaaaaaaaaaaa',0,'Keep removed-owner note','Dragon Guard','now','now'); CREATE TABLE live_trait_write_audit(component TEXT NOT NULL);").unwrap();
        for (event, row) in [("INSERT", "new"), ("UPDATE", "new"), ("DELETE", "old")] {
            c.execute_batch(&format!("CREATE TRIGGER live_trait_members_{event} AFTER {event} ON trait_members WHEN {row}.record_key='items:cccccccccccccccc' BEGIN INSERT INTO live_trait_write_audit VALUES('trait_members'); END;")).unwrap();
        }
        let identity: i64 = c
            .query_row(
                "SELECT id FROM record_identity WHERE key='actors:aaaaaaaaaaaaaaaa'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        let revision: u64 = c
            .query_row("SELECT reference_revision FROM workspace", [], |r| r.get(0))
            .unwrap();
        let reduced = input(false);
        crate::test_support::publish_input(&path, &reduced).unwrap();
        assert_eq!(c.query_row("SELECT count(*) FROM trait_members WHERE record_key IN ('actors:aaaaaaaaaaaaaaaa','items:bbbbbbbbbbbbbbbb')",[],|r|r.get::<_,usize>(0)).unwrap(),0);
        assert_eq!(
            c.query_row(
                "SELECT availability FROM record_identity WHERE id=?",
                [identity],
                |r| r.get::<_, String>(0)
            )
            .unwrap(),
            "unavailable"
        );
        assert_eq!(
            c.query_row(
                "SELECT count(*) FROM trait_members WHERE record_key='items:cccccccccccccccc'",
                [],
                |r| r.get::<_, usize>(0)
            )
            .unwrap(),
            1
        );
        assert_eq!(
            c.query_row(
                "SELECT count(*) FROM reference_entries WHERE key='trait:dragon'",
                [],
                |r| r.get::<_, usize>(0)
            )
            .unwrap(),
            1
        );
        crate::test_support::publish_input(&path, &initial).unwrap();
        assert_eq!(
            c.query_row(
                "SELECT id FROM record_identity WHERE key='actors:aaaaaaaaaaaaaaaa'",
                [],
                |r| r.get::<_, i64>(0)
            )
            .unwrap(),
            identity
        );
        assert_eq!(
            c.query_row(
                "SELECT availability FROM record_identity WHERE id=?",
                [identity],
                |r| r.get::<_, String>(0)
            )
            .unwrap(),
            "available"
        );
        assert_eq!(
            c.query_row(
                "SELECT note FROM saved_list_items WHERE list_id=1",
                [],
                |r| r.get::<_, String>(0)
            )
            .unwrap(),
            "Keep removed-owner note"
        );
        assert_eq!(
            c.query_row("SELECT reference_revision FROM workspace", [], |r| r
                .get::<_, u64>(0))
                .unwrap(),
            revision + 2
        );
        assert_eq!(
            c.query_row("SELECT count(*) FROM live_trait_write_audit", [], |r| r
                .get::<_, usize>(
                0
            ))
            .unwrap(),
            0,
            "live associations were rewritten"
        );
        let plan = c
            .prepare("EXPLAIN QUERY PLAN SELECT value,record_key,owners_json,source_label FROM trait_members WHERE record_key=?")
            .unwrap()
            .query_map(["items:cccccccccccccccc"], |r| r.get::<_, String>(3))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert!(
            plan.iter()
                .any(|step| step.contains("SEARCH") && step.contains("record_key=?")),
            "root-key lookup must use a useful index: {plan:?}"
        );
        crate::validate_artifact(&path).unwrap();
    }
    #[test]
    fn duplicate_trait_definition_is_rejected_without_selecting_a_winner() {
        let mut input = crate::foundation_tests::fixture(false);
        let definition = definition(&input, "agile");
        input.traits.push(definition.clone());
        input.traits.push(definition);
        let directory = tempfile::tempdir().unwrap();
        let error =
            crate::test_support::publish_input(directory.path().join("duplicate.sqlite"), &input)
                .unwrap_err();
        assert!(
            error.to_string().contains("duplicate trait definition"),
            "{error}"
        );
    }
    #[test]
    fn visible_definition_link_rejects_a_removed_publication_target() {
        use atlas_record::source_content::{
            ContentReferenceTarget, ResolvedContentReference, prepare_reference_definition,
        };
        let mut input = crate::foundation_tests::fixture(false);
        let mut definition = definition(&input, "agile");
        let target = input.records[1].record.key().clone();
        let resolve = |_: &atlas_record::source_content::ContentReferenceKind, _: &str| {
            Some(ResolvedContentReference {
                target: ContentReferenceTarget::Record {
                    key: target.clone(),
                },
                display_name: Some("Dragon Staff".into()),
                item_family: Some("weapon".into()),
            })
        };
        let markup = "<p>@UUID[Compendium.pf2e.items.Item.bbbbbbbbbbbbbbbb]{Dragon Staff}</p>";
        let prepared = prepare_reference_definition(
            definition.key.clone().into(),
            markup,
            input.context.audience,
            None,
            Some(&resolve),
        )
        .unwrap();
        definition.authored_markup = Some(markup.into());
        definition.description_html = Some(prepared.html.clone());
        definition.prepared = Some(prepared);
        input.traits.push(definition);
        let (_directory, path) = crate::foundation_tests::build(&input);
        crate::validate_artifact(&path).unwrap();
        let mut reduced = input;
        reduced
            .records
            .retain(|record| record.record.key() != &target);
        refresh_fixture(&mut reduced);
        let error = crate::test_support::publish_input(&path, &reduced).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("resolved inline target of trait:agile is absent"),
            "{error}"
        );
        crate::validate_artifact(&path).unwrap();
        let reader = crate::test_support::open_reader(&path).unwrap();
        assert!(reader.read_summaries(&[target]).unwrap().records.len() == 1);
    }
    #[test]
    fn trait_unknown_policy_cannot_be_relabelled_during_projection_upgrade() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("workspace.sqlite");
        let mut input = crate::foundation_tests::fixture(false);
        crate::test_support::publish_input(&path, &input).unwrap();
        input.context.trait_policy = "trait-definition-centroid/v99".into();
        input.context.query_projection_version = "authored-query/v2".into();
        let json = serde_json::to_string(&input.context).unwrap();
        let workspace = crate::test_support::initialize_workspace(&path).unwrap();
        let write = workspace
            .write(crate::test_support::TEST_WORKSPACE_SCHEMA_VERSION)
            .unwrap();
        write
            .connection()
            .execute(
                "UPDATE artifact_context SET context_json=?1,context_hash=?2",
                params![json, crate::codec::sha256(json.as_bytes())],
            )
            .unwrap();
        assert!(crate::classify_reference_upgrade(write.connection()).is_err());
    }
    #[test]
    fn trait_memberships_preserve_nameless_spell_and_nested_subitem_owners() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("workspace.sqlite");
        let mut input = crate::foundation_tests::fixture(false);
        input.records[1] = crate::foundation_tests::record(
            "items",
            "Item",
            json!({"_id":"bbbbbbbbbbbbbbbb","name":"Dragon Staff","type":"weapon","system":{"traits":{"value":[]},"subitems":[{"_id":"eeeeeeeeeeeeeeee","name":"Attached Tool","type":"equipment","system":{"traits":{"value":["custom-tool"]},"subitems":[{"_id":"ffffffffffffffff","type":"consumable","system":{"spell":{"_id":"gggggggggggggggg","type":"spell","system":{"traits":{"value":["custom-spell"]}}}}}]}}]}}),
        );
        refresh_fixture(&mut input);
        crate::test_support::publish_input(&path, &input).unwrap();
        let reader = crate::test_support::open_reader(&path).unwrap();
        let (members, total) = reader
            .trait_memberships(&TraitKey::new("custom-tool").unwrap(), 20, 0)
            .unwrap();
        assert_eq!(total, 1);
        assert_eq!(members[0].owners.len(), 1);
        assert_eq!(members[0].source_label.as_deref(), Some("Attached Tool"));
        let (members, total) = reader
            .trait_memberships(&TraitKey::new("custom-spell").unwrap(), 20, 0)
            .unwrap();
        assert_eq!(total, 1);
        assert_eq!(members[0].owners.len(), 3);
        assert!(members[0].source_label.is_none());
    }

    use super::*;
    use serde_json::json;
    #[test]
    fn source_free_definitions_and_exact_root_embedded_memberships() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("workspace.sqlite");
        let mut input = crate::foundation_tests::fixture(false);
        let key = TraitKey::new("dragon").unwrap();
        input.traits.push(TraitDefinition {
            key: key.clone(),
            label: "Dragon".into(),
            label_key: None,
            description_key: None,
            description_html: None,
            authored_markup: None,
            prepared: None,
            catalogs: vec!["creatureTraits".into()],
            source_locations: vec![],
            source_revision: input.context.source_revision.clone(),
            indexing_locale: input.context.indexing_locale.clone(),
            content_locale: None,
        });
        input.records[0] = crate::foundation_tests::record(
            "actors",
            "Actor",
            json!({"_id":"aaaaaaaaaaaaaaaa","name":"Dragon Guard","type":"npc","system":{"details":{"level":{"value":9}},"traits":{"value":["dragon"]}},"items":[{"_id":"eeeeeeeeeeeeeeee","name":"Unknown Breath","type":"action","system":{"traits":{"value":["custom-unknown"]},"description":{"value":"Breath"}}}]}),
        );
        input.records[0].preparation_context_hash = input.context.preparation_sha256().unwrap();
        input.lexical_units.clear();
        for source in &input.records {
            let label = &input
                .packs
                .iter()
                .find(|p| p.pack_id == source.record.key().pack().as_str())
                .unwrap()
                .label;
            let selection = atlas_record::source_record::select_record_search(
                &source.record,
                &source.content,
                input.context.audience,
                label,
                &input.context.used_trait_labels,
            )
            .unwrap();
            input
                .lexical_units
                .extend(crate::project_lexical_units(&selection, &[]));
        }
        crate::test_support::publish_input(&path, &input).unwrap();
        let reader = crate::test_support::open_reader(&path).unwrap();
        assert_eq!(
            reader.trait_definition(&key).unwrap().unwrap().label,
            "Dragon"
        );
        let (root, total) = reader.trait_memberships(&key, 1, 0).unwrap();
        assert_eq!(total, 1);
        assert!(root[0].owners.is_empty());
        let unknown = TraitKey::new("custom-unknown").unwrap();
        assert!(reader.trait_definition(&unknown).unwrap().is_none());
        let (embedded, total) = reader.trait_memberships(&unknown, 1, 0).unwrap();
        assert_eq!(total, 1);
        assert_eq!(embedded[0].owners.len(), 1);
        assert_eq!(embedded[0].record.key, root[0].record.key);
        assert!(
            reader
                .trait_memberships(&unknown, 1, 1)
                .unwrap()
                .0
                .is_empty()
        );
    }
    #[test]
    fn legacy_context_without_trait_policy_requires_source_preparation() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("workspace.sqlite");
        let input = crate::foundation_tests::fixture(false);
        crate::test_support::publish_input(&path, &input).unwrap();
        let workspace = crate::test_support::initialize_workspace(&path).unwrap();
        let write = workspace
            .write(crate::test_support::TEST_WORKSPACE_SCHEMA_VERSION)
            .unwrap();
        let mut json = serde_json::to_value(&input.context).unwrap();
        json.as_object_mut().unwrap().remove("trait_policy");
        let json = serde_json::to_string(&json).unwrap();
        write
            .connection()
            .execute(
                "UPDATE artifact_context SET context_json=?1,context_hash=?2",
                params![json, crate::codec::sha256(json.as_bytes())],
            )
            .unwrap();
        assert_eq!(
            crate::classify_reference_upgrade(write.connection()).unwrap(),
            crate::ReferenceUpgrade::SourcePreparation
        );
    }
}
