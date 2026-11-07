//! Set members for reusable abilities declared by immediate owned actions.

// A malformed embedded item cannot establish absence of a capability.
pub(super) const PARENT: &str = "(SELECT p.record_id AS id, p.record_id AS root_record_id,
    NULL AS items,
    CASE WHEN NOT EXISTS (
        SELECT 1 FROM artifact_context a WHERE a.singleton=1
          AND json_extract(a.context_json,'$.reference_preparation.coverage')='complete'
          AND json_extract(a.context_json,'$.reference_preparation.source_package')='pf2e'
          AND NOT EXISTS (
            SELECT 1 FROM json_each(a.context_json,'$.reference_preparation.pack_systems') s
            WHERE s.key IN (r.pack_id,'bestiary-ability-glossary-srd') AND s.value IS NOT 'pf2e'
          )
    ) THEN 'missing' WHEN p.items_state='value' AND EXISTS (
        SELECT 1 FROM actor_items i WHERE i.record_id=p.record_id AND i.source_type_state!='value'
    ) THEN 'invalid' ELSE p.items_state END AS items_state
    FROM actor_projection p JOIN records r USING(record_id))";

// Own-body localization declarations take precedence over imported source identity.
// Anchor declarations at the root so correlated membership can use both index keys.
pub(super) const MEMBERS: &str = r"(
SELECT i.record_id, t.key AS value
FROM records r
CROSS JOIN definition_reference_occurrences d ON d.source_record_key=r.key
JOIN actor_items i ON i.record_id=r.record_id
  AND json_extract(d.evidence,'$.source.owners')=json_array(json(i.owner_selector_json))
JOIN records t ON t.key=d.target_record_key
WHERE r.source_type='npc' AND i.source_type='action' AND i.source_type_state='value'
  AND t.pack_id='bestiary-ability-glossary-srd' AND t.document_kind='Item'
  AND t.source_type_state='value' AND t.source_type='action'
  AND d.visible=1 AND json_extract(d.evidence,'$.source.field')='/system/description/value'
  AND json_extract(d.evidence,'$.evidence') GLOB 'PF2E.NPC.Abilities.Glossary.*'
  AND json_extract(d.evidence,'$.target.owners')='[]'
UNION ALL
SELECT i.record_id, t.key AS value
FROM actor_items i
JOIN records r ON r.record_id=i.record_id AND r.source_type='npc'
JOIN relationship_occurrences o ON o.record_id=i.record_id
  AND o.owners_json=json_array(json(i.owner_selector_json))
JOIN records t ON t.record_id=o.target_record_id
WHERE i.source_type='action' AND i.source_type_state='value'
  AND t.pack_id='bestiary-ability-glossary-srd' AND t.document_kind='Item'
  AND t.source_type_state='value' AND t.source_type='action'
  AND o.kind='CompendiumProvenance' AND o.origin='structured'
  AND o.resolution='resolved_record' AND coalesce(o.target_owners_json,'[]')='[]'
  AND NOT EXISTS (
    SELECT 1 FROM definition_reference_occurrences d
    WHERE d.source_record_key=r.key
      AND json_extract(d.evidence,'$.source.owners')=o.owners_json
      AND json_extract(d.evidence,'$.source.field')='/system/description/value'
      AND json_extract(d.evidence,'$.evidence') GLOB 'PF2E.NPC.Abilities.Glossary.*'
  )
  AND NOT EXISTS (
    SELECT 1 FROM relationship_occurrences other
    WHERE other.record_id=o.record_id AND other.owners_json=o.owners_json
      AND other.kind='CompendiumProvenance' AND other.origin='structured'
      AND (other.resolution!='resolved_record' OR other.target_record_id IS NOT o.target_record_id
           OR coalesce(other.target_owners_json,'[]')!='[]')
  )
)";

#[cfg(test)]
mod tests {
    use crate::query::tests::{database, result, root, row, text};
    use crate::{DefinitionReferenceInput, DefinitionReferenceSource, DefinitionReferenceTarget};
    use atlas_domain::{OwnedContentLocator, QueryLiteral, RecordKey};
    use rusqlite::params;
    use serde_json::json;

    #[test]
    fn action_membership_and_discovery_follow_owned_declarations() {
        let db = database();
        let pack = "bestiary-ability-glossary-srd";
        let grab = format!("{pack}:0000000000000010");
        let push = format!("{pack}:0000000000000011");
        let family = "bestiary-family-ability-glossary:0000000000000012";
        for pack in [pack, "bestiary-family-ability-glossary"] {
            db.execute("INSERT INTO packs VALUES(?,?)", [pack, pack])
                .unwrap();
        }
        for (id, key, name) in [
            (10, grab.as_str(), "Grab"),
            (11, push.as_str(), "Push"),
            (12, family, "Family ability"),
        ] {
            db.execute(
                "INSERT INTO record_identity(id,key,availability) VALUES(?,?,'available')",
                params![id, key],
            )
            .unwrap();
            row(
                &db,
                "records",
                &[
                    ("record_id", id.into()),
                    ("key", text(key)),
                    ("pack_id", text(key.split_once(':').unwrap().0)),
                    ("document_kind", text("Item")),
                    ("source_path", text("fixture.json")),
                    ("content_hash", text("hash")),
                    ("source_type_state", text("value")),
                    ("source_type", text("action")),
                    ("name_state", text("value")),
                    ("name", text(name)),
                    ("name_lookup_key", text(&name.to_lowercase())),
                ],
            );
        }
        let owners: Vec<OwnedContentLocator> = serde_json::from_value(json!([
            {"collection":"/items","identity":{"Stable":"AAAAAAAAAAAAAAAA"}}
        ]))
        .unwrap();
        let owners_json = serde_json::to_string(&owners).unwrap();
        let owner_json = serde_json::to_string(&owners[0]).unwrap();
        for id in 1..=9 {
            root(&db, id, if id == 7 { "character" } else { "npc" }, "common");
            row(
                &db,
                "actor_projection",
                &[
                    ("record_id", id.into()),
                    (
                        "items_state",
                        text(if id == 8 { "missing" } else { "value" }),
                    ),
                ],
            );
            if id != 8 {
                row(
                    &db,
                    "actor_items",
                    &[
                        ("id", id.into()),
                        ("record_id", id.into()),
                        ("original_index", 0.into()),
                        ("owner_selector_json", text(&owner_json)),
                        (
                            "source_type_state",
                            text(if id == 6 { "invalid" } else { "value" }),
                        ),
                        (
                            "source_type",
                            if id == 6 {
                                rusqlite::types::Value::Null
                            } else {
                                text("action")
                            },
                        ),
                    ],
                );
            }
        }
        for (id, key, label) in [
            (1, grab.as_str(), "Grab"),
            (1, push.as_str(), "Push"),
            (3, push.as_str(), "Push"),
            (4, family, "FamilyAbility"),
        ] {
            crate::references::insert_reference(
                &db,
                &DefinitionReferenceInput {
                    source: DefinitionReferenceSource::Record {
                        key: RecordKey::parse(&format!("test:{id:016}")).unwrap(),
                        owners: owners.clone(),
                        field: Some("/system/description/value".into()),
                    },
                    target: DefinitionReferenceTarget::Record {
                        key: RecordKey::parse(key).unwrap(),
                        owners: vec![],
                    },
                    evidence: format!("PF2E.NPC.Abilities.Glossary.{label}"),
                    occurrence_path: format!("/{label}"),
                    label: None,
                    visible: true,
                },
            )
            .unwrap();
        }
        for (id, target) in [
            (2, 10),
            (3, 10),
            (4, 10),
            (5, 10),
            (7, 10),
            (9, 10),
            (9, 11),
        ] {
            db.execute(
                "INSERT INTO relationship_occurrences(record_id,owners_json,field_path,ordinal,origin,kind,resolution,target_record_id) VALUES(?,?,'/_stats/compendiumSource',0,?,?,'resolved_record',?)",
                params![id, owners_json, if id == 5 { "content" } else { "structured" }, if id == 5 { "uuid" } else { "CompendiumProvenance" }, target],
            ).unwrap();
        }
        let has_grab = format!("'{grab}' in actions");
        assert!(result(&db, &format!("!({has_grab})")).is_empty());
        db.execute(
            "INSERT INTO artifact_context VALUES(1,3,printf('%064d',0),?)",
            [
                json!({"reference_preparation": {"coverage":"complete", "source_package":"pf2e"}})
                    .to_string(),
            ],
        )
        .unwrap();
        assert_eq!(result(&db, &has_grab), [1, 2]);
        assert_eq!(
            result(&db, &format!("{has_grab} && '{push}' in actions")),
            [1]
        );
        assert_eq!(result(&db, &format!("!({has_grab})")), [3, 4, 5, 9]);

        let values = crate::discovery::field_vocabulary(
            &db,
            &crate::QueryVocabularyRequest {
                field: "actions".into(),
                eligible_keys: None,
                text: Some("grab".into()),
                offset: 0,
                limit: 20,
                selected_values: vec![],
            },
        )
        .unwrap();
        assert_eq!(values.total_values, 1);
        assert_eq!(values.options[0].value, QueryLiteral::String(grab));

        for (pack_systems, applies) in [
            (json!({"test": "pf2e"}), true),
            (json!({"test": null}), false),
            (json!({"test": "dnd5e"}), false),
            (json!({"bestiary-ability-glossary-srd": null}), false),
        ] {
            db.execute(
                "UPDATE artifact_context SET context_json =
                 json_set(context_json, '$.reference_preparation.pack_systems', json(?))",
                [pack_systems.to_string()],
            )
            .unwrap();
            if applies {
                assert_eq!(result(&db, &has_grab), [1, 2]);
            } else {
                assert!(result(&db, &has_grab).is_empty());
                assert!(result(&db, &format!("!({has_grab})")).is_empty());
            }
        }
    }
}
