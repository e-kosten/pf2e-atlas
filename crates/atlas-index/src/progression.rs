//! Source-backed progression expressions and their specifically scoped record edges.

use crate::{IndexError, SourceRelationshipDetails, SourceStoredRelationship, SqliteIndexReader};
use atlas_domain::RecordKey;
use atlas_record::progression::{PrerequisiteClause, PrerequisiteExpression, ProgressionRecord};
use atlas_record::source_content::{ContentReferenceResolution, ContentReferenceTarget};
use atlas_record::source_record::{SourceBackedRecord, SourceContentOutcome, SourceContentStatus};
use rusqlite::{Connection, OptionalExtension, params};
use std::collections::{BTreeMap, BTreeSet};

pub const PROGRESSION_RECIPE: &str = "record-progression/v2";
pub(crate) const PREREQUISITE_KIND: &str = "Prerequisite";
pub(crate) const ORDINARY_RELATIONSHIP: &str = "o.kind<>'Prerequisite'";

pub struct ProgressionPage<T> {
    pub entries: Vec<T>,
    pub has_more: bool,
}

pub struct PrerequisiteDependent {
    pub record_key: RecordKey,
    pub leaf_paths: Vec<String>,
}

// Filter real ancestry feat roots by exact published trait membership before paging.
const ANCESTRY_FEAT_MATCH: &str = "r.source_type='feat' AND EXISTS(
    SELECT 1 FROM ability_projection ability
    WHERE ability.root_record_id=r.record_id AND ability.category='ancestry'
)";
const CURRENT_SYSTEM_PACK: &str = "NOT EXISTS(
    SELECT 1 FROM json_each(?4) scope WHERE scope.key=r.pack_id AND scope.value IS NOT 'pf2e'
)";
const HERITAGE_ACCESS_MATCH: &str = "EXISTS(
    SELECT 1 FROM json_each(pool.ancestry_access_json) access
    JOIN trait_members tm ON tm.value=json_extract(access.value,'$.trait_key')
    WHERE tm.record_key=r.key AND tm.owners_json='[]' AND tm.lookup_member=1
)";

fn indexed_key(raw: String) -> Result<RecordKey, IndexError> {
    raw.parse()
        .map_err(|error| IndexError::Invalid(format!("progression record key: {error}")))
}

fn query_limit(limit: usize) -> Result<usize, IndexError> {
    if !(1..=100).contains(&limit) {
        return Err(IndexError::Invalid(
            "progression page must contain 1..=100 records".into(),
        ));
    }
    Ok(limit + 1)
}

fn page<T>(mut entries: Vec<T>, limit: usize) -> ProgressionPage<T> {
    let has_more = entries.len() > limit;
    entries.truncate(limit);
    ProgressionPage { entries, has_more }
}

impl SqliteIndexReader {
    pub fn read_progression(
        &self,
        key: &RecordKey,
    ) -> Result<Option<ProgressionRecord>, IndexError> {
        self.count_statement();
        read_persisted(
            self.native()?,
            "SELECT p.requirements_json,p.ancestry_access_json
             FROM record_progression p JOIN records r USING(record_id) WHERE r.key=?",
            [key.to_string()],
        )
    }

    pub fn has_progression(&self, key: &RecordKey) -> Result<bool, IndexError> {
        if !crate::reference_pack_applies(
            self.context().reference_preparation.as_ref(),
            "pf2e",
            key.pack().as_str(),
        ) {
            return Ok(false);
        }
        self.count_statement();
        Ok(self.native()?.query_row(
            &format!(
                "SELECT EXISTS(
                SELECT 1 FROM records r WHERE r.key=?1 AND (
                    EXISTS(SELECT 1 FROM record_progression p WHERE p.record_id=r.record_id)
                    OR EXISTS(SELECT 1 FROM relationship_occurrences o
                        WHERE o.target_record_id=r.record_id AND o.kind='Prerequisite')
                    OR (({ANCESTRY_FEAT_MATCH}) AND EXISTS(
                        SELECT 1 FROM record_progression pool
                        WHERE pool.ancestry_access_json IS NOT NULL AND {HERITAGE_ACCESS_MATCH}
                    ))
                )
            )"
            ),
            [key.to_string()],
            |row| row.get(0),
        )?)
    }

    pub fn read_prerequisite_dependents(
        &self,
        key: &RecordKey,
        offset: usize,
        limit: usize,
    ) -> Result<ProgressionPage<PrerequisiteDependent>, IndexError> {
        let requested = query_limit(limit)?;
        self.count_statement();
        let mut statement = self.native()?.prepare_cached(
            "SELECT r.key, json_group_array(o.occurrence_path ORDER BY o.ordinal)
            FROM records target
            JOIN relationship_occurrences o ON o.target_record_id=target.record_id
            JOIN records r ON r.record_id=o.record_id
            WHERE target.key=?1 AND o.kind='Prerequisite'
            GROUP BY r.record_id
            ORDER BY r.level IS NULL, r.level, r.name_lookup_key, r.key
            LIMIT ?2 OFFSET ?3",
        )?;
        let rows = statement.query_map(params![key.to_string(), requested, offset], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;
        let entries = rows
            .map(|row| {
                let (key, paths) = row?;
                Ok(PrerequisiteDependent {
                    record_key: indexed_key(key)?,
                    leaf_paths: serde_json::from_str(&paths)?,
                })
            })
            .collect::<Result<Vec<_>, IndexError>>()?;
        Ok(page(entries, limit))
    }

    pub fn read_ancestry_feat_pool(
        &self,
        trait_key: &str,
        offset: usize,
        limit: usize,
    ) -> Result<ProgressionPage<RecordKey>, IndexError> {
        self.read_progression_keys(
            &format!("SELECT r.key
                FROM trait_members tm JOIN records r ON r.key=tm.record_key
                WHERE tm.value=?1 AND tm.owners_json='[]' AND tm.lookup_member=1 AND {ANCESTRY_FEAT_MATCH} AND {CURRENT_SYSTEM_PACK}
                ORDER BY r.level IS NULL, r.level, r.name_lookup_key, r.key LIMIT ?2 OFFSET ?3"),
            trait_key, offset, limit,
        )
    }

    pub fn read_ancestry_feat_access(
        &self,
        key: &RecordKey,
        offset: usize,
        limit: usize,
    ) -> Result<ProgressionPage<RecordKey>, IndexError> {
        self.read_progression_keys(
            &format!(
                "SELECT heritage.key FROM record_progression pool
                JOIN records heritage ON heritage.record_id=pool.record_id
                JOIN records r ON r.key=?1
                WHERE pool.ancestry_access_json IS NOT NULL AND {ANCESTRY_FEAT_MATCH} AND {HERITAGE_ACCESS_MATCH} AND {CURRENT_SYSTEM_PACK}
                ORDER BY heritage.name_lookup_key, heritage.key LIMIT ?2 OFFSET ?3"
            ),
            &key.to_string(),
            offset,
            limit,
        )
    }

    fn read_progression_keys(
        &self,
        sql: &str,
        input: &str,
        offset: usize,
        limit: usize,
    ) -> Result<ProgressionPage<RecordKey>, IndexError> {
        let requested = query_limit(limit)?;
        let Some(preparation) = self
            .context()
            .reference_preparation
            .as_ref()
            .filter(|p| p.source_package.as_deref() == Some("pf2e"))
        else {
            return Ok(page(Vec::new(), limit));
        };
        let packs = serde_json::to_string(&preparation.pack_systems)?;
        self.count_statement();
        let mut statement = self.native()?.prepare_cached(sql)?;
        let rows =
            statement.query_map(params![input, requested, offset, packs], |row| row.get(0))?;
        let entries = rows
            .map(|row| indexed_key(row?))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(page(entries, limit))
    }
}

/// Project these rows from the expression once. The exact owner and leaf locator
/// also provide the expected occurrence identities for artifact validation.
fn projected_edges(
    key: &RecordKey,
    progression: Option<&ProgressionRecord>,
) -> Result<Vec<SourceStoredRelationship>, IndexError> {
    let mut paths = BTreeSet::new();
    progression
        .into_iter()
        .flat_map(ProgressionRecord::record_targets)
        .enumerate()
        .map(|(ordinal, target)| {
            if target.locator.record != *key
                || !target.locator.owners.is_empty()
                || target.target_key == key
                || !paths.insert(target.leaf_path.clone())
            {
                return Err(IndexError::Invalid(
                    "progression occurrence has an invalid owner, target, or leaf path".into(),
                ));
            }
            Ok(SourceStoredRelationship {
                locator: target.locator.clone(),
                ordinal,
                origin: "structured".into(),
                kind: PREREQUISITE_KIND.into(),
                authored_target: None,
                occurrence_path: Some(target.leaf_path),
                resolution: ContentReferenceResolution::Resolved(ContentReferenceTarget::Record {
                    key: target.target_key.clone(),
                }),
                details: SourceRelationshipDetails::default(),
            })
        })
        .collect()
}

pub(crate) fn stored_roots(c: &Connection) -> Result<BTreeSet<i64>, IndexError> {
    let mut statement = c.prepare(
        "SELECT record_id FROM record_progression
         UNION SELECT record_id FROM relationship_occurrences WHERE kind='Prerequisite'",
    )?;
    Ok(statement
        .query_map([], |row| row.get(0))?
        .collect::<Result<_, _>>()?)
}

const READ_PROGRESSION_BY_ID: &str = "SELECT requirements_json,ancestry_access_json
    FROM record_progression WHERE record_id=?";

fn read_persisted(
    c: &Connection,
    sql: &str,
    params: impl rusqlite::Params,
) -> Result<Option<ProgressionRecord>, IndexError> {
    let columns = c
        .prepare_cached(sql)?
        .query_row(params, |row| {
            Ok((
                row.get::<_, Option<String>>(0)?,
                row.get::<_, Option<String>>(1)?,
            ))
        })
        .optional()?;
    columns
        .map(|(requirements, ancestry_access)| {
            Ok(ProgressionRecord {
                requirements: requirements
                    .as_deref()
                    .map(serde_json::from_str)
                    .transpose()?
                    .unwrap_or_default(),
                ancestry_access: ancestry_access
                    .as_deref()
                    .map(serde_json::from_str)
                    .transpose()?
                    .unwrap_or_default(),
            })
        })
        .transpose()
}

fn read_edges(c: &Connection, id: i64) -> Result<Vec<SourceStoredRelationship>, IndexError> {
    let mut statement = c.prepare_cached(&format!(
        "{} WHERE o.record_id=? AND o.kind='Prerequisite' ORDER BY o.ordinal",
        crate::read_content::RELATIONSHIP_SELECT
    ))?;
    Ok(statement
        .query_map([id], crate::read_content::relationship_row)?
        .collect::<Result<_, _>>()?)
}

pub(crate) fn replace(
    c: &Connection,
    ids: &BTreeMap<RecordKey, i64>,
    id: i64,
    key: &RecordKey,
    had_progression: bool,
    progression: Option<&ProgressionRecord>,
) -> Result<(), IndexError> {
    if progression.is_none() && !had_progression {
        return Ok(());
    }
    let edges = projected_edges(key, progression)?;
    if had_progression {
        if read_persisted(c, READ_PROGRESSION_BY_ID, [id])?.as_ref() == progression
            && read_edges(c, id)? == edges
        {
            return Ok(());
        }
        c.execute(
            "DELETE FROM relationship_occurrences WHERE record_id=? AND kind='Prerequisite'",
            [id],
        )?;
    }
    if let Some(progression) = progression {
        let requirements = (!progression.requirements.is_empty())
            .then(|| serde_json::to_string(&progression.requirements))
            .transpose()?;
        let ancestry_access = (!progression.ancestry_access.is_empty())
            .then(|| serde_json::to_string(&progression.ancestry_access))
            .transpose()?;
        c.execute(
            "INSERT INTO record_progression(record_id,requirements_json,ancestry_access_json) VALUES(?1,?2,?3)
             ON CONFLICT(record_id) DO UPDATE SET requirements_json=excluded.requirements_json,
                 ancestry_access_json=excluded.ancestry_access_json
             WHERE requirements_json IS NOT excluded.requirements_json
                OR ancestry_access_json IS NOT excluded.ancestry_access_json",
            params![id, requirements, ancestry_access],
        )?;
    } else {
        c.execute("DELETE FROM record_progression WHERE record_id=?", [id])?;
    }
    for edge in &edges {
        crate::writer::write_relationship(c, id, edge, ids)?;
    }
    Ok(())
}

// The published resolver intentionally omits excluded source envelopes. An
// abstention from source preparation remains an abstention during standalone
// validation. It cannot justify publishing a new or differently resolved edge.
fn validates_expression(
    stored: &PrerequisiteExpression,
    replayed: &PrerequisiteExpression,
) -> bool {
    use PrerequisiteExpression::{AllOf, AnyOf, Unresolved};
    match (stored, replayed) {
        (AllOf { children }, AllOf { children: expected })
        | (AnyOf { children }, AnyOf { children: expected }) => {
            children.len() == expected.len()
                && children
                    .iter()
                    .zip(expected)
                    .all(|(a, b)| validates_expression(a, b))
        }
        (Unresolved { text }, PrerequisiteExpression::Record { label, .. }) => text == label,
        _ => stored == replayed,
    }
}

fn contains_record(expression: &PrerequisiteExpression) -> bool {
    match expression {
        PrerequisiteExpression::Record { .. } => true,
        PrerequisiteExpression::AllOf { children } | PrerequisiteExpression::AnyOf { children } => {
            children.iter().any(contains_record)
        }
        _ => false,
    }
}

fn validates_clause(
    stored: &PrerequisiteClause,
    replayed: &PrerequisiteClause,
    content: &[SourceContentOutcome],
) -> bool {
    if stored.locator != replayed.locator {
        return false;
    }
    if let PrerequisiteExpression::Unresolved { text } = &stored.expression
        && contains_record(&replayed.expression)
    {
        // A single excluded name can prevent an entire compound clause from
        // becoming a group at ingest. Retain exactly the canonical fallback.
        return content
            .iter()
            .find_map(|outcome| match &outcome.status {
                SourceContentStatus::Prepared(prepared) if prepared.locator == stored.locator => {
                    atlas_record::progression::clause_text(prepared)
                }
                _ => None,
            })
            .is_some_and(|visible| text == visible.trim());
    }
    validates_expression(&stored.expression, &replayed.expression)
}

pub(crate) fn validate(
    c: &Connection,
    id: i64,
    record: &SourceBackedRecord,
    expected: Option<&ProgressionRecord>,
    content: &[SourceContentOutcome],
) -> Result<usize, IndexError> {
    let stored = read_persisted(c, READ_PROGRESSION_BY_ID, [id])?;
    let matches = match (stored.as_ref(), expected) {
        (None, None) => true,
        (Some(stored), Some(expected)) => {
            stored.ancestry_access == expected.ancestry_access
                && stored.requirements.len() == expected.requirements.len()
                && stored
                    .requirements
                    .iter()
                    .zip(&expected.requirements)
                    .all(|(a, b)| validates_clause(a, b, content))
        }
        _ => false,
    };
    let edges = read_edges(c, id)?;
    if !matches || projected_edges(record.key(), stored.as_ref())? != edges {
        return Err(IndexError::Invalid(
            "published progression expression or edge evidence differs".into(),
        ));
    }
    Ok(edges.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::foundation_tests::{build, fixture, record, synchronize_fixture_terms};
    use crate::{SourceRelationshipDirection, SourceRelationshipRequest};
    use atlas_domain::{QueryRecordSelection, QueryRelationship, QueryRelationshipDirection};
    use serde_json::json;

    #[test]
    fn compound_abstentions_keep_exact_text_when_standalone_replay_recovers_links() {
        use atlas_record::progression::{ProgressionIndex, prepare};
        for clause in ["Alpha and Beta.", "Alpha, Beta, or Gamma."] {
            let owner = record(
                "items",
                "Item",
                json!({
                    "_id": "dependentxxxxxxx", "name": "Compound", "type": "feat", "system": {
                        "category": "skill", "prerequisites": {"value": [{"value": clause}]}
                    }
                }),
            );
            let mut index = ProgressionIndex::default();
            for (id, name) in [
                ("aaaaaaaaaaaaaaaa", "Alpha"),
                ("bbbbbbbbbbbbbbbb", "Beta"),
                ("cccccccccccccccc", "Gamma"),
            ] {
                let candidate = record(
                    "items",
                    "Item",
                    json!({
                        "_id": id, "name": name, "type": "feat", "system": {"category": "skill"}
                    }),
                );
                index.insert(&candidate.record, &candidate.source_path);
            }
            let replayed =
                prepare(&owner.record, &owner.source_path, &index, &owner.content).unwrap();
            assert!(replayed.record_targets().len() >= 2, "{clause}");
            index.exclude_name("Alpha");
            let mut stored =
                prepare(&owner.record, &owner.source_path, &index, &owner.content).unwrap();
            assert_eq!(
                stored.requirements[0].expression,
                PrerequisiteExpression::Unresolved {
                    text: clause.into()
                }
            );
            assert!(validates_clause(
                &stored.requirements[0],
                &replayed.requirements[0],
                &owner.content
            ));
            stored.requirements[0].expression = PrerequisiteExpression::Unresolved {
                text: clause.trim_end_matches('.').into(),
            };
            assert!(!validates_clause(
                &stored.requirements[0],
                &replayed.requirements[0],
                &owner.content
            ));
        }
    }

    #[test]
    fn missing_progression_recipe_requires_source_preparation_with_passage_reuse() {
        let input = fixture(true);
        let (_dir, path) = build(&input);
        let c = Connection::open(&path).unwrap();
        let mut old = input.context.clone();
        old.progression_recipe.clear();
        let old_json = serde_json::to_string(&old).unwrap();
        c.execute(
            "UPDATE artifact_context SET context_hash=?1, context_json=?2",
            params![crate::codec::sha256(old_json.as_bytes()), old_json],
        )
        .unwrap();
        assert_eq!(
            crate::classify_reference_upgrade(&c).unwrap(),
            crate::ReferenceUpgrade::SourcePreparation,
        );
        assert!(crate::test_support::open_reader(&path).is_err());
        let workspace = atlas_storage::Workspace::at(&path).unwrap();
        let read = workspace
            .read(crate::test_support::TEST_WORKSPACE_SCHEMA_VERSION)
            .unwrap();
        assert!(
            !crate::read_reusable_passage_layouts(&read)
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn progression_reuses_record_edges_without_entering_ordinary_graph() {
        let mut input = fixture(false);
        let (expected_entry_count, entry_keys_sha256) =
            crate::reference_inventory_identity(std::iter::empty()).unwrap();
        input.context.reference_preparation = Some(crate::ReferencePreparation {
            source_package: Some("pf2e".into()),
            pack_systems: BTreeMap::new(),
            recipe: "fixture-reference/v1".into(),
            input_hash: crate::codec::sha256(b"fixture references"),
            coverage: crate::ReferenceCoverage::TraitsOnly,
            expected_entry_count,
            entry_keys_sha256,
        });
        let preparation_hash = input.context.preparation_sha256().unwrap();
        for prepared in &mut input.records {
            prepared
                .preparation_context_hash
                .clone_from(&preparation_hash);
        }
        for (id, name, level, prerequisites) in [
            ("prerequisitexxxx", "Battle Medicine", 1, json!([])),
            (
                "dependentxxxxxxx",
                "Medic Dedication",
                2,
                json!([
                    {"value": "Battle Medicine"},
                    {"value": "trained in Medicine"},
                ]),
            ),
        ] {
            let mut prepared = record(
                "items",
                "Item",
                json!({
                    "_id": id, "name": name, "type": "feat", "system": {
                        "category": "skill", "level": {"value": level},
                        "prerequisites": {"value": prerequisites}
                    }
                }),
            );
            prepared.preparation_context_hash = input.context.preparation_sha256().unwrap();
            input.records.push(prepared);
        }
        input.context.source_file_count = input.records.len();
        synchronize_fixture_terms(&mut input);
        let (_dir, path) = build(&input);
        crate::validate_artifact(&path).unwrap();
        let target = "items:prerequisitexxxx".parse().unwrap();
        let dependant: RecordKey = "items:dependentxxxxxxx".parse().unwrap();
        let reader = crate::test_support::open_reader(&path).unwrap();
        assert!(reader.has_progression(&target).unwrap());
        let progression = reader.read_progression(&dependant).unwrap().unwrap();
        assert_eq!(progression.requirements.len(), 2);
        let page = reader.read_prerequisite_dependents(&target, 0, 1).unwrap();
        assert!(!page.has_more);
        assert_eq!(page.entries[0].record_key, dependant);
        assert_eq!(
            page.entries[0].leaf_paths,
            vec![progression.record_targets()[0].leaf_path.clone()]
        );
        let query = QueryRelationship {
            direction: QueryRelationshipDirection::Incoming,
            selection: QueryRecordSelection {
                record_key: target.clone(),
                owners: vec![],
                field: None,
                passage: None,
                source_fingerprint: None,
                spell_rank: None,
            },
            include_contents: true,
        };
        assert!(!reader.has_related_records(&query).unwrap());
        let ordinary = reader
            .read_relationships(&SourceRelationshipRequest {
                record: target,
                owners: None,
                direction: SourceRelationshipDirection::Incoming,
                fields: vec![],
                limit: 1,
            })
            .unwrap();
        assert!(ordinary.occurrences.is_empty());
        drop(reader);

        let c = Connection::open(&path).unwrap();
        let occurrence: i64 = c
            .query_row(
                "SELECT occurrence_id FROM relationship_occurrences WHERE kind='Prerequisite'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let last = input.records.len() - 1;
        let mut linked_source = record(
            "items",
            "Item",
            json!({
                "_id": "dependentxxxxxxx", "name": "Medic Dedication", "type": "feat",
                "system": {
                    "category": "skill", "level": {"value": 2},
                    "prerequisites": {"value": [
                        {"value": "Battle Medicine"}, {"value": "trained in Medicine"}
                    ]},
                    "description": {"value": "<p>@UUID[Compendium.pf2e.items.Item.bbbbbbbbbbbbbbbb]{Dragon Staff}</p>"}
                }
            }),
        );
        let mut resolver = atlas_record::source_record::SourceReferenceIndex::default();
        for source in &input.records {
            resolver.insert_source(source.record.key(), source.record.source());
        }
        linked_source.content = atlas_record::source_record::prepare_record_content(
            &linked_source.record,
            input.context.audience,
            None,
            Some(&resolver),
        );
        linked_source.preparation_context_hash = preparation_hash;
        input.records[last] = linked_source;
        synchronize_fixture_terms(&mut input);
        crate::test_support::publish_input(&path, &input).unwrap();
        let stored: (usize, i64) = c
            .query_row(
                "SELECT (SELECT count(*) FROM relationship_occurrences WHERE origin='content'),
                    (SELECT occurrence_id FROM relationship_occurrences WHERE kind='Prerequisite')",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(stored, (1, occurrence));
        crate::validate_artifact(&path).unwrap();

        c.execute("UPDATE relationship_occurrences SET occurrence_path='/wrong' WHERE kind='Prerequisite'", []).unwrap();
        let error = crate::validate_artifact(&path).unwrap_err();
        assert!(
            error.to_string().contains("progression expression or edge"),
            "{error}"
        );
    }
}
