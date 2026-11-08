//! Offline consolidation of filter and lookup membership from the retained artifact.
use crate::IndexError;
use rusqlite::Connection;

pub const TRAIT_MEMBERS_SCHEMA_SQL: &str = include_str!("trait_members_schema.sql");

pub fn backfill_trait_members(c: &Connection) -> atlas_storage::StorageResult<()> {
    backfill(c).map_err(|error| atlas_storage::StorageError::Migration(error.to_string()))
}

fn backfill(c: &Connection) -> Result<(), IndexError> {
    let mut keys = c.prepare("SELECT DISTINCT trait_key FROM trait_memberships")?;
    for key in keys.query_map([], |r| r.get::<_, String>(0))? {
        key?.parse::<atlas_domain::TraitKey>()
            .map_err(|e| IndexError::Invalid(format!("invalid legacy trait key: {e}")))?;
    }
    drop(keys);
    c.execute_batch(
        "INSERT INTO trait_members
           SELECT substr(trait_key,7),record_key,owners_json,1,source_label FROM trait_memberships;
         INSERT INTO trait_members
           SELECT t.value,r.key,'[]',0,NULL FROM record_traits t JOIN records r USING(record_id)
           WHERE true ON CONFLICT(value,record_key,owners_json) DO NOTHING;
         INSERT INTO trait_members
           SELECT t.value,r.key,json_array(json(i.owner_selector_json)),0,NULL
           FROM actor_item_traits t JOIN actor_items i ON i.id=t.item_id JOIN records r USING(record_id)
           WHERE true ON CONFLICT(value,record_key,owners_json) DO NOTHING;",
    )?;
    check_parity(
        c,
        "lookup",
        "SELECT trait_key,record_key,owners_json,source_label FROM trait_memberships",
        "SELECT 'trait:'||value,record_key,owners_json,source_label FROM trait_members WHERE lookup_member=1",
    )?;
    check_parity(
        c,
        "root filters",
        "SELECT record_id,value FROM record_traits",
        "SELECT r.record_id,m.value FROM trait_members m JOIN records r ON r.key=m.record_key WHERE m.owners_json='[]'",
    )?;
    check_parity(
        c,
        "embedded filters",
        "SELECT item_id,value FROM actor_item_traits",
        "SELECT i.id,m.value FROM trait_members m JOIN records r ON r.key=m.record_key
         JOIN actor_items i ON i.record_id=r.record_id
          AND m.owners_json=json_array(json(i.owner_selector_json))",
    )?;
    crate::component_digests::refresh_trait_migration(c)?;
    c.execute_batch(
        "DROP TABLE actor_item_traits;
         DROP TABLE record_traits;
         DROP TABLE trait_memberships;",
    )?;
    Ok(())
}

fn check_parity(c: &Connection, label: &str, old: &str, new: &str) -> Result<(), IndexError> {
    let diff: bool = c.query_row(
        &format!("SELECT EXISTS({old} EXCEPT {new}) OR EXISTS({new} EXCEPT {old})"),
        [],
        |row| row.get(0),
    )?;
    if diff {
        return Err(IndexError::Invalid(format!(
            "trait membership migration changed {label}"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{projections::root_rows, test_support::TEST_MIGRATIONS};

    #[test]
    fn upgrade_retains_filter_only_values_and_exact_lookup_owners() {
        atlas_sqlite_vec::register_sqlite_vec_auto_extension().unwrap();
        let directory = tempfile::tempdir().unwrap();
        let workspace =
            atlas_storage::Workspace::at(directory.path().join("traits.sqlite")).unwrap();
        workspace
            .migrate(&TEST_MIGRATIONS[..35], |_| Ok(()))
            .unwrap();
        let write = workspace.write(35).unwrap();
        let c = write.connection();
        let input = crate::foundation_tests::fixture(false);
        let source = crate::foundation_tests::record(
            "items",
            "Item",
            serde_json::json!({
                "_id":"aaaaaaaaaaaaaaaa","name":"Staff","type":"weapon",
                "system":{
                    "traits":{"value":["dragon","raw value"]},
                    "subitems":[{"type":"equipment","system":{"traits":{"value":["agile"]}}}]
                }
            }),
        );
        let key = source.record.key().to_string();
        c.execute("INSERT INTO packs VALUES('items','Items')", [])
            .unwrap();
        c.execute(
            "INSERT INTO record_identity(id,key,availability) VALUES(1,?,'available')",
            [&key],
        )
        .unwrap();
        for row in root_rows(&source, "Items", 1, &mut Default::default()).unwrap() {
            row.insert(c).unwrap();
        }
        let audience = input.context.audience;
        crate::historical_publication::filter_traits(c, 1, &source.record, audience).unwrap();
        let mut evidence = crate::traits::TraitEvidence::default();
        evidence.add_record(&source.record, audience).unwrap();
        crate::traits::write_historical_record_traits(c, source.record.key(), evidence).unwrap();
        write.commit().unwrap();

        workspace
            .migrate(&TEST_MIGRATIONS[..36], |_| Ok(()))
            .unwrap();
        let read = workspace.read(36).unwrap();
        let c = read.connection();
        let members = c
            .prepare("SELECT value,owners_json,lookup_member FROM trait_members ORDER BY value")
            .unwrap()
            .query_map([], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, bool>(2)?,
                ))
            })
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(members.len(), 3);
        assert!(members[0].1.contains("/system/subitems") && members[0].2);
        assert_eq!(members[1], ("dragon".into(), "[]".into(), true));
        assert_eq!(members[2], ("raw value".into(), "[]".into(), false));
        assert!(!c.prepare("SELECT 1 FROM sqlite_schema WHERE name IN ('record_traits','actor_item_traits','trait_memberships')")
            .unwrap().exists([]).unwrap());
    }
}
