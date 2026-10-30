//! Reuse published source addresses without storing a second model-input corpus.
use crate::{IndexError, embedding_values::hash_bytes};
use atlas_domain::{OwnedContentLocator, RecordKey};
use atlas_embedding::PassageLayout;
use atlas_storage::ReadSession;
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct SourcePassageLayout {
    pub owners: Vec<OwnedContentLocator>,
    pub field: String,
    pub chunk_ordinal: usize,
    pub passage: PassageLayout,
}

/// Admit shared segmentation only; model-native token counts and vectors are not
/// reused here. Current selected text must independently match every input hash.
pub fn read_reusable_passage_layouts(
    session: &ReadSession,
) -> Result<BTreeMap<RecordKey, Vec<SourcePassageLayout>>, IndexError> {
    let c = session.connection();
    let Some(context) = crate::read_stored_reference_context(c)? else {
        return Ok(BTreeMap::new());
    };
    let Some(profile) = context.search_profiles.iter().find(|profile| {
        profile.unit_policy_version == atlas_embedding::EMBEDDING_UNIT_POLICY_VERSION
            && profile.segmentation_tokenizer_sha256
                == atlas_embedding::BASELINE_SEGMENTATION_TOKENIZER_SHA256
            && profile.input_recipe == atlas_embedding::BASELINE_INPUT_RECIPE
    }) else {
        return Ok(BTreeMap::new());
    };
    let model_identity = serde_json::to_string(profile)?;
    let mut statement = c.prepare("SELECT r.key,u.record_id,l.record_id,l.location_hash,l.owners_json,l.field_path,l.section_json,l.chunk_ordinal,lower(hex(i.input_hash)),m.identity_json=?2 FROM semantic_units u JOIN records r ON r.record_id=u.record_id JOIN semantic_locations l USING(location_id) JOIN embedding_inputs i USING(input_id) JOIN embedding_models m USING(model_id) WHERE u.profile_id=?1 AND u.unit_kind='passage' ORDER BY u.record_id,l.location_id")?;
    let mut rows = statement.query([profile.profile_id.as_str(), model_identity.as_str()])?;
    let mut output: BTreeMap<RecordKey, Vec<SourcePassageLayout>> = BTreeMap::new();
    while let Some(row) = rows.next()? {
        let key: String = row.get(0)?;
        let record: i64 = row.get(1)?;
        let located_record: i64 = row.get(2)?;
        let checksum: Vec<u8> = row.get(3)?;
        let owners: String = row.get(4)?;
        let field: Option<String> = row.get(5)?;
        let address: String = row.get(6)?;
        let ordinal: usize = row.get(7)?;
        if !row.get::<_, bool>(9)? {
            return Err(IndexError::Invalid(
                "published passage model/profile mismatch".into(),
            ));
        }
        if record != located_record
            || checksum
                != hash_bytes(&serde_json::to_vec(&(
                    record, &owners, &field, &address, ordinal,
                ))?)
        {
            return Err(IndexError::Invalid(
                "published passage location checksum mismatch".into(),
            ));
        }
        let input_sha256: String = row.get(8)?;
        if !crate::codec::checked_hash(&input_sha256) {
            return Err(IndexError::Invalid(
                "published passage input hash is invalid".into(),
            ));
        }
        let address: atlas_domain::SourcePassageAddress = serde_json::from_str(&address)?;
        if matches!(address, atlas_domain::SourcePassageAddress::Identity {}) {
            return Err(IndexError::Invalid(
                "published passage has an identity address".into(),
            ));
        }
        output
            .entry(crate::read_units::parse_key(key)?)
            .or_default()
            .push(SourcePassageLayout {
                owners: serde_json::from_str(&owners)?,
                field: field.ok_or_else(|| {
                    IndexError::Invalid("published passage field is absent".into())
                })?,
                chunk_ordinal: ordinal,
                passage: PassageLayout {
                    address,
                    input_sha256,
                },
            });
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{TEST_WORKSPACE_SCHEMA_VERSION, initialize_workspace};

    #[test]
    fn published_layouts_are_shared_across_profiles_and_have_no_native_values() {
        let input = crate::profile_tests::both();
        let (_dir, path) = crate::foundation_tests::build(&input);
        let workspace = initialize_workspace(&path).unwrap();
        let read = workspace.read(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
        let layouts = read_reusable_passage_layouts(&read).unwrap();
        assert_eq!(
            layouts.values().map(Vec::len).sum::<usize>(),
            input
                .semantic_units
                .iter()
                .filter(|unit| unit.profile_id.as_str() == "bge-base" && unit.field.is_some())
                .count()
        );
        assert!(!layouts.is_empty());
        for unit in input
            .semantic_units
            .iter()
            .filter(|unit| unit.profile_id.as_str() == "bge-base" && unit.field.is_some())
        {
            assert!(
                layouts[&unit.record]
                    .iter()
                    .any(|layout| layout.passage.address == unit.address
                        && layout.passage.input_sha256 == unit.input_hash
                        && layout.owners == unit.owners
                        && Some(layout.field.as_str()) == unit.field.as_deref())
            );
        }
    }

    #[test]
    fn unsupported_segmentation_recipe_is_a_miss_and_corrupt_locations_are_errors() {
        let input = crate::foundation_tests::fixture(true);
        let (_dir, path) = crate::foundation_tests::build(&input);
        let workspace = initialize_workspace(&path).unwrap();
        let c = rusqlite::Connection::open(&path).unwrap();
        for dependency in [
            "unit_policy_version",
            "segmentation_tokenizer_sha256",
            "input_recipe",
        ] {
            let mut context = serde_json::to_value(&input.context).unwrap();
            context["search_profiles"][0][dependency] = serde_json::json!("prior-recipe");
            let context: crate::SourceArtifactBuildContext =
                serde_json::from_value(context).unwrap();
            c.execute(
                "UPDATE artifact_context SET context_json=?1,context_hash=?2",
                rusqlite::params![
                    serde_json::to_string(&context).unwrap(),
                    context.sha256().unwrap()
                ],
            )
            .unwrap();
            let read = workspace.read(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
            assert!(read_reusable_passage_layouts(&read).unwrap().is_empty());
            drop(read);
        }
        c.execute(
            "UPDATE artifact_context SET context_json=?1,context_hash=?2",
            rusqlite::params![
                serde_json::to_string(&input.context).unwrap(),
                input.context.sha256().unwrap()
            ],
        )
        .unwrap();
        c.execute("UPDATE semantic_locations SET location_hash=zeroblob(32) WHERE location_id=(SELECT min(location_id) FROM semantic_units WHERE unit_kind='passage')", []).unwrap();
        let read = workspace.read(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
        assert!(
            read_reusable_passage_layouts(&read)
                .unwrap_err()
                .to_string()
                .contains("checksum")
        );
    }

    #[test]
    fn contradictory_published_model_and_passage_kind_are_errors() {
        for corruption in ["model", "identity address"] {
            let input = crate::foundation_tests::fixture(true);
            let (_dir, path) = crate::foundation_tests::build(&input);
            let workspace = initialize_workspace(&path).unwrap();
            let c = rusqlite::Connection::open(&path).unwrap();
            if corruption == "model" {
                let mut model = input.context.search_profiles[0].clone();
                model.revision.push_str("-contradiction");
                c.execute(
                    "UPDATE embedding_models SET identity_json=?",
                    [serde_json::to_string(&model).unwrap()],
                )
                .unwrap();
            } else {
                let (id, record, owners, field, ordinal): (i64,i64,String,Option<String>,usize) = c.query_row("SELECT location_id,record_id,owners_json,field_path,chunk_ordinal FROM semantic_locations WHERE location_id=(SELECT min(location_id) FROM semantic_units WHERE unit_kind='passage')", [], |row| Ok((row.get(0)?,row.get(1)?,row.get(2)?,row.get(3)?,row.get(4)?))).unwrap();
                let address =
                    serde_json::to_string(&atlas_domain::SourcePassageAddress::Identity {})
                        .unwrap();
                let hash = hash_bytes(
                    &serde_json::to_vec(&(record, &owners, &field, &address, ordinal)).unwrap(),
                );
                c.execute("UPDATE semantic_locations SET section_json=?1,location_hash=?2 WHERE location_id=?3", rusqlite::params![address,hash,id]).unwrap();
            }
            let read = workspace.read(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
            assert!(
                read_reusable_passage_layouts(&read).is_err(),
                "{corruption}"
            );
        }
    }
}
