//! Synthetic vectors prove eligibility and attribution, not semantic quality.
use crate::*;
use atlas_domain::{
    CORE_SECTIONS_RECIPE, CoreFallbackReason, CoreSectionRepresentation, QueryPredicate,
    SearchContentScope, SearchProfileId, SimilarityComparisonKey, SimilaritySeed, SourceByteRange,
    SourcePassageAddress,
};
use atlas_record::source_record::{SOURCE_CONTENT_SELECTION_VERSION, select_record_search};
use serde_json::{Value, json};
use std::collections::BTreeMap;

fn basis(first: f32, second: f32) -> Vec<f32> {
    let mut vector = vec![0.; 768];
    vector[0] = first;
    vector[1] = second;
    vector
}

pub(super) fn fixture(values: Vec<(&str, &str, Value)>, both: bool) -> IndexBuildInput {
    let mut input = crate::foundation_tests::fixture(true);
    input.records = values
        .into_iter()
        .map(|(pack, kind, value)| crate::foundation_tests::record(pack, kind, value))
        .collect();
    input.packs = vec![
        IndexBuildPack {
            pack_id: "actors".into(),
            label: "Actors".into(),
            document_kind: "Actor".into(),
            declared_path: "packs/actors".into(),
        },
        IndexBuildPack {
            pack_id: "items".into(),
            label: "Items".into(),
            document_kind: "Item".into(),
            declared_path: "packs/items".into(),
        },
        IndexBuildPack {
            pack_id: "journals".into(),
            label: "Journals".into(),
            document_kind: "JournalEntry".into(),
            declared_path: "packs/journals".into(),
        },
    ];
    input.inventory = CompleteSourceInventory::checked(
        &input.context.source_fingerprint,
        input.packs.iter().map(|p| p.pack_id.clone()),
        input.packs.iter().map(|p| p.pack_id.clone()),
    )
    .unwrap();
    input.context.source_file_count = input.records.len();
    if both {
        input
            .context
            .search_profiles
            .push(SourceSearchProfileIdentity::from_spec(
                atlas_embedding::SEARCH_PROFILES[1],
            ));
        input
            .context
            .search_profiles
            .sort_by(|a, b| a.profile_id.cmp(&b.profile_id));
    }
    input.lexical_units.clear();
    input.semantic_units.clear();
    let preparation = input.context.preparation_sha256().unwrap();
    for source in &mut input.records {
        source.preparation_context_hash = preparation.clone();
        let label = input
            .packs
            .iter()
            .find(|pack| pack.pack_id == source.record.key().pack().as_str())
            .unwrap()
            .label
            .clone();
        let selected = select_record_search(
            &source.record,
            &source.content,
            input.context.audience,
            &label,
            &BTreeMap::new(),
        )
        .unwrap();
        input
            .lexical_units
            .extend(project_lexical_units(&selected, &[]));
        for profile in &input.context.search_profiles {
            input.semantic_units.push(SourceSemanticUnitInput {
                profile_id: profile.profile_id.clone(),
                record: source.record.key().clone(),
                owners: vec![],
                field: None,
                address: SourcePassageAddress::Identity {},
                chunk_ordinal: 0,
                input_token_count: 8,
                input_hash: crate::codec::sha256(
                    format!("{}:identity", source.record.key()).as_bytes(),
                ),
                vector: basis(0., 1.),
            });
            for field in &selected.fields {
                for section in &field.sections {
                    let range = SourceByteRange {
                        start: 0,
                        end: section.text.len(),
                    };
                    let address = if let Some(hash) = &field.prepared_html_sha256 {
                        SourcePassageAddress::HtmlSection {
                            prepared_html_sha256: hash.clone(),
                            canonical_text_sha256: crate::codec::sha256(section.text.as_bytes()),
                            selection_version: SOURCE_CONTENT_SELECTION_VERSION.into(),
                            section_ordinal: section.section_ordinal,
                            label: section.label.clone(),
                            chunk_bytes: range,
                        }
                    } else {
                        SourcePassageAddress::PlainSection {
                            source_text_sha256: crate::codec::sha256(section.text.as_bytes()),
                            selection_version: SOURCE_CONTENT_SELECTION_VERSION.into(),
                            section_ordinal: section.section_ordinal,
                            label: section.label.clone(),
                            chunk_bytes: range,
                        }
                    };
                    let vector = if field.locator.owners.is_empty() {
                        basis(0.8, 0.6)
                    } else {
                        basis(1., 0.)
                    };
                    input.semantic_units.push(SourceSemanticUnitInput {
                        profile_id: profile.profile_id.clone(),
                        record: source.record.key().clone(),
                        owners: field.locator.owners.clone(),
                        field: Some(field.locator.field.clone()),
                        address,
                        chunk_ordinal: 0,
                        input_token_count: 8,
                        input_hash: crate::codec::sha256(
                            &serde_json::to_vec(&(
                                source.record.key(),
                                &field.locator,
                                section.section_ordinal,
                            ))
                            .unwrap(),
                        ),
                        vector,
                    });
                }
            }
        }
    }
    input
}

pub(super) fn nested_fixture(both: bool) -> IndexBuildInput {
    fixture(
        vec![
            (
                "actors",
                "Actor",
                json!({"_id":"aaaaaaaaaaaaaaaa","name":"Source creature","type":"npc","system":{"details":{"publicNotes":"<h2>Origin</h2><p>Core creature bodyword.</p>","privateNotes":"<h2>Secret</h2><p>Private text.</p>","blurb":"Core short blurb"}},"items":[{"_id":"cccccccccccccccc","name":"Needle","type":"consumable","system":{"description":{"value":"<h2>Needle</h2><p>Excluded carried content.</p>"}}}]}),
            ),
            (
                "items",
                "Item",
                json!({"_id":"bbbbbbbbbbbbbbbb","name":"Source item","type":"equipment","system":{"description":{"value":"<h2>Needle one two three four five six seven eight nine ten eleven twelve</h2><p>Allowed item bodyword.</p>"}}}),
            ),
        ],
        both,
    )
}

#[test]
fn core_masks_units_before_both_lexical_and_knn_limits_without_expanding_terms() {
    let input = nested_fixture(true);
    let (_directory, path) = crate::foundation_tests::build(&input);
    let reader = crate::test_support::open_reader(&path).unwrap();
    let all = validate_query(&QueryPredicate::boolean(true)).unwrap();
    let a = input.records[0].record.key();
    let b = input.records[1].record.key();
    let lexical_all = reader
        .lexical_candidates("needle", &all, None, SearchContentScope::All, 1)
        .unwrap();
    let lexical_core = reader
        .lexical_candidates("needle", &all, None, SearchContentScope::Core, 1)
        .unwrap();
    assert_eq!(&lexical_all[0].location.record, a);
    assert_eq!(&lexical_core[0].location.record, b);
    let roots = reader
        .lexical_root_candidates("needle", &all, None, SearchContentScope::Core)
        .unwrap();
    assert_eq!(
        roots.iter().map(|hit| &hit.record).collect::<Vec<_>>(),
        vec![b]
    );
    assert!(
        reader
            .lexical_root_candidates("bodyword", &all, None, SearchContentScope::Core)
            .unwrap()
            .is_empty()
    );
    assert!(
        reader
            .lexical_candidates(
                "source needle",
                &all,
                Some(std::slice::from_ref(a)),
                SearchContentScope::Core,
                100
            )
            .unwrap()
            .is_empty()
    );
    for profile in &input.context.search_profiles {
        let hits = reader
            .vector_candidates(crate::SourceVectorCandidateRequest {
                profile: &profile.profile_id,
                vector: &basis(1., 0.),
                query: &all,
                keys: None,
                exclude: None,
                content_scope: SearchContentScope::All,
                limit: 1,
            })
            .unwrap();
        assert_eq!(&hits[0].location.record, a);
        assert!(!hits[0].location.owners.is_empty());
        let core = reader
            .vector_candidates(crate::SourceVectorCandidateRequest {
                profile: &profile.profile_id,
                vector: &basis(1., 0.),
                query: &all,
                keys: None,
                exclude: None,
                content_scope: SearchContentScope::Core,
                limit: 1,
            })
            .unwrap();
        assert!(core[0].location.owners.is_empty());
        assert!(matches!(
            core[0].location.address,
            Some(SourcePassageAddress::HtmlSection { .. })
                | Some(SourcePassageAddress::PlainSection { .. })
        ));
    }
    assert_eq!(reader.read_metrics().source_body_decodes, 0);
}

#[test]
fn inventory_and_units_distinguish_identity_and_exact_core_prose() {
    let input = nested_fixture(false);
    let (_directory, path) = crate::foundation_tests::build(&input);
    let reader = crate::test_support::open_reader(&path).unwrap();
    let profile = SearchProfileId::new("bge-base");
    let key = input.records[0].record.key();
    let inventory = reader
        .similarity_parts(
            &SimilaritySeed::Record {
                record_key: key.clone(),
            },
            Some(&profile),
        )
        .unwrap();
    assert_eq!(
        inventory
            .parts
            .iter()
            .filter(|component| component.is_identity())
            .count(),
        1
    );
    assert_eq!(
        inventory.core.representation,
        CoreSectionRepresentation::IdentityAndProse
    );
    assert_eq!(inventory.core.recipe, CORE_SECTIONS_RECIPE);
    let units = reader
        .similarity_units(
            &profile,
            std::slice::from_ref(key),
            SearchContentScope::Core,
            true,
            100,
        )
        .unwrap();
    assert_eq!(
        units
            .iter()
            .filter(|unit| unit.component.is_identity())
            .count(),
        1
    );
    assert!(
        units
            .iter()
            .all(|unit| unit.location.as_ref().unwrap().owners.is_empty())
    );
    assert!(
        units
            .iter()
            .filter_map(|unit| unit.component.part())
            .all(|part| part.field != "/system/details/privateNotes")
    );
    assert_eq!(units.len(), 3); // One identity, one public description and one plain blurb.
    let identity = inventory
        .parts
        .iter()
        .find(|component| component.is_identity())
        .unwrap();
    let exact = reader
        .selected_similarity_units(
            &profile,
            std::slice::from_ref(key),
            Some(std::slice::from_ref(identity.selector())),
            SearchContentScope::All,
            true,
            1,
        )
        .unwrap();
    assert_eq!(exact.len(), 1);
    assert!(matches!(
        exact[0].address,
        SourcePassageAddress::Identity {}
    ));
    assert!(exact[0].location.as_ref().unwrap().field.is_none());
    let legacy = reader
        .similarity_units(
            &profile,
            std::slice::from_ref(key),
            SearchContentScope::All,
            false,
            100,
        )
        .unwrap();
    assert!(legacy.iter().all(|unit| !unit.component.is_identity()));
    assert!(
        legacy
            .iter()
            .any(|unit| !unit.location.as_ref().unwrap().owners.is_empty())
    );
}

#[test]
fn identity_correspondence_and_scoped_baseline_use_the_same_destination_mask() {
    let input = nested_fixture(false);
    let (_directory, path) = crate::foundation_tests::build(&input);
    let reader = crate::test_support::open_reader(&path).unwrap();
    let profile = SearchProfileId::new("bge-base");
    let all = validate_query(&QueryPredicate::boolean(true)).unwrap();
    let hits = reader
        .similarity_candidates(SourceSimilarityCandidateRequest {
            profile: &profile,
            vector: &basis(0., 1.),
            query: &all,
            keys: None,
            exclude: None,
            comparison_key: Some(SimilarityComparisonKey::Identity),
            content_scope: SearchContentScope::All,
            include_identity: true,
            limit: 1,
        })
        .unwrap();
    assert_eq!(hits.len(), 1);
    assert!(matches!(
        hits[0].location.address,
        Some(SourcePassageAddress::Identity {})
    ));
    let legacy = reader
        .similarity_candidates(SourceSimilarityCandidateRequest {
            profile: &profile,
            vector: &basis(0., 1.),
            query: &all,
            keys: None,
            exclude: None,
            comparison_key: None,
            content_scope: SearchContentScope::All,
            include_identity: false,
            limit: 100,
        })
        .unwrap();
    assert!(legacy.iter().all(|hit| !matches!(
        hit.location.address,
        Some(SourcePassageAddress::Identity {})
    )));
    let key = input.records[0].record.key();
    let baseline_all = reader
        .baseline_similarity_scores(
            &profile,
            std::slice::from_ref(key),
            &basis(1., 0.),
            SearchContentScope::All,
        )
        .unwrap();
    let baseline_core = reader
        .baseline_similarity_scores(
            &profile,
            std::slice::from_ref(key),
            &basis(1., 0.),
            SearchContentScope::Core,
        )
        .unwrap();
    assert!((baseline_all[0].1 - 1.).abs() < 1e-6);
    assert!((baseline_core[0].1 - 0.8).abs() < 1e-6);
    assert_eq!(baseline_core[0].2, 3);
}

#[test]
fn no_core_prose_and_unmodelled_kind_keep_distinct_identity_only_outcomes() {
    let input = fixture(
        vec![
            (
                "actors",
                "Actor",
                json!({"_id":"aaaaaaaaaaaaaaaa","name":"Sparse creature","type":"npc"}),
            ),
            (
                "journals",
                "JournalEntry",
                json!({"_id":"bbbbbbbbbbbbbbbb","name":"Many-page journal","pages":[]}),
            ),
        ],
        false,
    );
    let (_directory, path) = crate::foundation_tests::build(&input);
    let reader = crate::test_support::open_reader(&path).unwrap();
    let profile = SearchProfileId::new("bge-base");
    for (index, reason) in [
        (0, CoreFallbackReason::NoCoreProse),
        (1, CoreFallbackReason::UnmodelledKind),
    ] {
        let key = input.records[index].record.key();
        let core = reader.core_selection(key, Some(&profile)).unwrap();
        assert_eq!(core.representation, CoreSectionRepresentation::IdentityOnly);
        assert_eq!(core.fallback_reason, Some(reason));
        assert_eq!(core.selectors.len(), 1);
        let units = reader
            .selected_similarity_units(
                &profile,
                std::slice::from_ref(key),
                Some(&core.selectors),
                SearchContentScope::Core,
                true,
                1,
            )
            .unwrap();
        assert_eq!(units.len(), 1);
        assert!(units[0].component.is_identity());
    }
}

fn table_values(c: &rusqlite::Connection) -> BTreeMap<String, Vec<Vec<rusqlite::types::Value>>> {
    let tables=c.prepare("SELECT name FROM sqlite_schema WHERE type='table' AND name NOT LIKE 'core_%' AND name NOT IN ('artifact_context','workspace','schema_migrations') ORDER BY name").unwrap().query_map([],|row|row.get::<_,String>(0)).unwrap().collect::<Result<Vec<_>,_>>().unwrap();
    tables
        .into_iter()
        .map(|table| {
            let quoted = format!("\"{}\"", table.replace('"', "\"\""));
            let columns = c
                .prepare(&format!("PRAGMA table_info({quoted})"))
                .unwrap()
                .query_map([], |row| row.get::<_, String>(1))
                .unwrap()
                .collect::<Result<Vec<_>, _>>()
                .unwrap();
            let order = (1..=columns.len())
                .map(|i| i.to_string())
                .collect::<Vec<_>>()
                .join(",");
            let mut statement = c
                .prepare(&format!("SELECT * FROM {quoted} ORDER BY {order}"))
                .unwrap();
            let rows = statement
                .query_map([], |row| {
                    (0..columns.len())
                        .map(|i| row.get(i))
                        .collect::<Result<Vec<rusqlite::types::Value>, _>>()
                })
                .unwrap()
                .collect::<Result<Vec<_>, _>>()
                .unwrap();
            (table, rows)
        })
        .collect()
}

#[test]
fn offline_core_upgrade_preserves_every_existing_table_and_publishes_one_generation() {
    use atlas_local_state::{LocalStateWrite, NewSavedList};
    let input = nested_fixture(true);
    let (_directory, path) = crate::foundation_tests::build(&input);
    let workspace = atlas_storage::Workspace::at(&path).unwrap();
    let write = workspace
        .write(crate::test_support::TEST_WORKSPACE_SCHEMA_VERSION)
        .unwrap();
    LocalStateWrite::new(&write)
        .saved_lists()
        .create(NewSavedList {
            slug: "kept".into(),
            name: "Kept".into(),
            description: Some("Retained note".into()),
            tags: vec![],
        })
        .unwrap();
    let revision = write.reference_revision().unwrap();
    let mut context = input.context.clone();
    context.core_sections_recipe.clear();
    let json = serde_json::to_string(&context).unwrap();
    write
        .connection()
        .execute(
            "UPDATE artifact_context SET context_hash=?,context_json=?",
            rusqlite::params![crate::codec::sha256(json.as_bytes()), json],
        )
        .unwrap();
    write.connection().execute_batch("DROP TABLE core_semantic_locations; DROP TABLE core_lexical_units; DROP TABLE core_parts; DROP TABLE core_roots; DELETE FROM schema_migrations WHERE version=21; PRAGMA user_version=20;").unwrap();
    let before = table_values(write.connection());
    write.commit().unwrap();
    let upgraded = crate::test_support::initialize_workspace(&path).unwrap();
    let read = upgraded
        .read(crate::test_support::TEST_WORKSPACE_SCHEMA_VERSION)
        .unwrap();
    assert_eq!(read.reference_revision().unwrap(), revision);
    assert_eq!(
        classify_reference_upgrade(read.connection()).unwrap(),
        ReferenceUpgrade::CoreProjections
    );
    drop(read);
    assert!(
        ensure_reference_projections(
            &upgraded,
            crate::test_support::TEST_WORKSPACE_SCHEMA_VERSION
        )
        .unwrap()
    );
    let read = upgraded
        .read(crate::test_support::TEST_WORKSPACE_SCHEMA_VERSION)
        .unwrap();
    assert_eq!(read.reference_revision().unwrap(), revision + 1);
    assert_eq!(table_values(read.connection()), before);
    let context_after = read_stored_reference_context(read.connection())
        .unwrap()
        .unwrap();
    assert_eq!(
        context_after.preparation_sha256().unwrap(),
        context.preparation_sha256().unwrap()
    );
    assert_eq!(context_after.core_sections_recipe, CORE_SECTIONS_RECIPE);
    drop(read);
    assert!(
        !ensure_reference_projections(
            &upgraded,
            crate::test_support::TEST_WORKSPACE_SCHEMA_VERSION
        )
        .unwrap()
    );
    let reader = crate::test_support::open_reader(&path).unwrap();
    assert_eq!(
        reader
            .core_selection(
                input.records[0].record.key(),
                Some(&SearchProfileId::new("bge-base"))
            )
            .unwrap()
            .representation,
        CoreSectionRepresentation::IdentityAndProse
    );
}

#[test]
fn an_allowed_owned_name_remains_core_when_its_description_is_empty() {
    let input = fixture(
        vec![(
            "actors",
            "Actor",
            json!({"_id":"aaaaaaaaaaaaaaaa","name":"Foundational PC","type":"character","items":[{"_id":"bbbbbbbbbbbbbbbb","name":"Pyromancer","type":"class","system":{"description":{"value":""}}},{"_id":"cccccccccccccccc","name":"Incidental","type":"feat","system":{"description":{"value":"<h2>Incidental ability</h2><p>Not a foundational document.</p>"}}}]}),
        )],
        false,
    );
    let (_directory, path) = crate::foundation_tests::build(&input);
    let reader = crate::test_support::open_reader(&path).unwrap();
    let all = validate_query(&QueryPredicate::boolean(true)).unwrap();
    let core = reader
        .core_selection(
            input.records[0].record.key(),
            Some(&SearchProfileId::new("bge-base")),
        )
        .unwrap();
    assert_eq!(core.representation, CoreSectionRepresentation::IdentityOnly);
    assert_eq!(
        reader
            .lexical_root_candidates("pyromancer", &all, None, SearchContentScope::Core)
            .unwrap()
            .len(),
        1
    );
    assert!(
        reader
            .lexical_root_candidates("incidental", &all, None, SearchContentScope::Core)
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        reader
            .lexical_root_candidates("incidental", &all, None, SearchContentScope::All)
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn trait_core_inventory_retains_its_single_label_only_representation() {
    let mut input = nested_fixture(false);
    let key: atlas_domain::TraitKey = "trait:core-label".parse().unwrap();
    let label = "Core label";
    let text = format!("{label}\n");
    let hash = crate::codec::sha256(text.as_bytes());
    let vector = basis(1., 0.);
    input.traits.push(TraitDefinition {
        key: key.clone(),
        label: label.into(),
        label_key: None,
        description_key: None,
        description_html: None,
        authored_markup: None,
        prepared: None,
        catalogs: vec![],
        source_locations: vec![],
        source_revision: None,
        indexing_locale: "en".into(),
    });
    input.trait_vectors.push(TraitVectorInput {
        key: key.clone(),
        profile_id: SearchProfileId::new("bge-base"),
        input_hash: crate::codec::sha256(format!("trait-definition-centroid/v1:{hash}").as_bytes()),
        vector: vector.clone(),
        chunks: vec![TraitSeedChunk {
            input: text.clone(),
            input_hash: hash,
            token_count: 2,
            address: SourcePassageAddress::PlainSection {
                source_text_sha256: crate::codec::sha256(text.as_bytes()),
                selection_version: "trait-definition-centroid/v1".into(),
                section_ordinal: 0,
                label: Some(label.into()),
                chunk_bytes: SourceByteRange {
                    start: 0,
                    end: text.len(),
                },
            },
            vector,
        }],
    });
    let (_directory, path) = crate::foundation_tests::build(&input);
    let reader = crate::test_support::open_reader(&path).unwrap();
    let profile = SearchProfileId::new("bge-base");
    let inventory = reader
        .similarity_parts(
            &SimilaritySeed::Trait {
                trait_key: key.clone(),
            },
            Some(&profile),
        )
        .unwrap();
    assert_eq!(
        inventory.core.representation,
        CoreSectionRepresentation::TraitLabelOnly
    );
    assert_eq!(inventory.parts.len(), 1);
    assert!(!inventory.parts[0].is_identity());
    assert!(inventory.parts[0].part().unwrap().label_only);
    let units = reader.trait_similarity_units(&profile, &key, 1).unwrap();
    assert_eq!(units.len(), 1);
    assert!(!units[0].component.is_identity());
}

#[test]
fn required_identity_vector_absence_is_a_readiness_failure() {
    let input = nested_fixture(false);
    let (_directory, path) = crate::foundation_tests::build(&input);
    let key = input.records[0].record.key();
    let c = rusqlite::Connection::open(&path).unwrap();
    c.execute("DELETE FROM semantic_units WHERE unit_kind='identity' AND record_id=(SELECT record_id FROM records WHERE key=?)",[key.to_string()]).unwrap();
    drop(c);
    let reader = crate::test_support::open_reader(&path).unwrap();
    let profile = SearchProfileId::new("bge-base");
    let inventory = reader
        .similarity_parts(
            &SimilaritySeed::Record {
                record_key: key.clone(),
            },
            Some(&profile),
        )
        .unwrap();
    assert!(
        !inventory
            .parts
            .iter()
            .find(|component| component.is_identity())
            .unwrap()
            .has_vectors()
    );
    let error = reader
        .similarity_units(
            &profile,
            std::slice::from_ref(key),
            SearchContentScope::Core,
            true,
            100,
        )
        .unwrap_err();
    assert!(matches!(error, IndexError::Unavailable(_)));
}

#[test]
fn metadata_publication_failure_rolls_back_core_and_keeps_the_old_generation() {
    let input = nested_fixture(false);
    let (_directory, path) = crate::foundation_tests::build(&input);
    let workspace = atlas_storage::Workspace::at(&path).unwrap();
    let write = workspace
        .write(crate::test_support::TEST_WORKSPACE_SCHEMA_VERSION)
        .unwrap();
    let before = table_values(write.connection());
    let revision = write.reference_revision().unwrap();
    let mut context = input.context.clone();
    context.core_sections_recipe.clear();
    write
        .connection()
        .execute(
            "UPDATE artifact_context SET context_hash=?,context_json=?",
            rusqlite::params![
                context.sha256().unwrap(),
                serde_json::to_string(&context).unwrap()
            ],
        )
        .unwrap();
    write.connection().execute_batch("DELETE FROM core_roots; CREATE TRIGGER reject_core_insert BEFORE INSERT ON core_parts BEGIN SELECT RAISE(ABORT,'core publication fault'); END;").unwrap();
    write.commit().unwrap();
    let error = ensure_reference_projections(
        &workspace,
        crate::test_support::TEST_WORKSPACE_SCHEMA_VERSION,
    )
    .unwrap_err();
    assert!(error.to_string().contains("core publication fault"));
    // Explicit fault inspection is not an ordinary product read while closed.
    let c = rusqlite::Connection::open(&path).unwrap();
    assert_eq!(table_values(&c), before);
    assert_eq!(
        c.query_row("SELECT reference_revision FROM workspace", [], |row| row
            .get::<_, u64>(0))
            .unwrap(),
        revision
    );
    assert_eq!(
        c.query_row("SELECT count(*) FROM core_roots", [], |row| row
            .get::<_, usize>(0))
            .unwrap(),
        0
    );
    assert_eq!(read_stored_reference_context(&c).unwrap().unwrap(), context);
    assert!(
        c.query_row("SELECT maintenance IS NOT NULL FROM workspace", [], |row| {
            row.get::<_, bool>(0)
        })
        .unwrap()
    );
    c.execute_batch("DROP TRIGGER reject_core_insert;").unwrap();
    drop(c);
    {
        let owner = workspace.update_owner().unwrap();
        assert!(
            recover_reference_publication(
                &owner,
                crate::test_support::TEST_WORKSPACE_SCHEMA_VERSION
            )
            .unwrap()
        );
    }
    assert!(
        ensure_reference_projections(
            &workspace,
            crate::test_support::TEST_WORKSPACE_SCHEMA_VERSION
        )
        .unwrap()
    );
    let read = workspace
        .read(crate::test_support::TEST_WORKSPACE_SCHEMA_VERSION)
        .unwrap();
    assert_eq!(read.reference_revision().unwrap(), revision + 1);
}

#[test]
fn lexical_backfill_refreshes_current_core_atomically_without_rewriting_other_data() {
    let input = nested_fixture(false);
    let (_directory, path) = crate::foundation_tests::build(&input);
    let workspace = atlas_storage::Workspace::at(&path).unwrap();
    let schema = crate::test_support::TEST_WORKSPACE_SCHEMA_VERSION;
    let write = workspace.write(schema).unwrap();
    let revision = write.reference_revision().unwrap();
    let mut context = input.context.clone();
    assert_eq!(context.core_sections_recipe, CORE_SECTIONS_RECIPE);
    context.lexical_selection_version = "curated-lexical/v1".into();
    write
        .connection()
        .execute(
            "UPDATE artifact_context SET context_hash=?,context_json=?",
            rusqlite::params![
                context.sha256().unwrap(),
                serde_json::to_string(&context).unwrap()
            ],
        )
        .unwrap();
    write.connection().execute(
        "INSERT INTO verified_aliases(record_id,alias,alias_lookup_key,evidence_json) SELECT record_id,'Snow Keeper','snow keeper','\"source redirect\"' FROM records WHERE key=?",
        [input.records[0].record.key().to_string()],
    ).unwrap();
    let before = table_values(write.connection());
    let core_values = |c: &rusqlite::Connection| {
        [
            "core_roots",
            "core_parts",
            "core_lexical_units",
            "core_semantic_locations",
        ]
        .into_iter()
        .map(|table| {
            let mut statement = c
                .prepare(&format!("SELECT * FROM {table} ORDER BY 1"))
                .unwrap();
            let columns = statement.column_count();
            let rows = statement
                .query_map([], |row| {
                    (0..columns)
                        .map(|column| row.get::<_, rusqlite::types::Value>(column))
                        .collect::<rusqlite::Result<Vec<_>>>()
                })
                .unwrap()
                .collect::<Result<Vec<_>, _>>()
                .unwrap();
            (table, rows)
        })
        .collect::<BTreeMap<_, _>>()
    };
    let core_before = core_values(write.connection());
    assert!(core_before.values().all(|rows| !rows.is_empty()));
    write.connection().execute_batch("CREATE TRIGGER reject_core_insert BEFORE INSERT ON core_parts BEGIN SELECT RAISE(ABORT,'core publication fault'); END;").unwrap();
    write.commit().unwrap();

    let error = ensure_reference_projections(&workspace, schema).unwrap_err();
    assert!(error.to_string().contains("core publication fault"));
    // The Core refresh is part of the lexical publication, not a later repair.
    let c = rusqlite::Connection::open(&path).unwrap();
    assert_eq!(table_values(&c), before);
    assert_eq!(core_values(&c), core_before);
    assert_eq!(read_stored_reference_context(&c).unwrap().unwrap(), context);
    assert_eq!(
        c.query_row("SELECT reference_revision FROM workspace", [], |row| row
            .get::<_, u64>(0))
            .unwrap(),
        revision
    );
    c.execute_batch("DROP TRIGGER reject_core_insert;").unwrap();
    drop(c);
    {
        let owner = workspace.update_owner().unwrap();
        assert!(recover_reference_publication(&owner, schema).unwrap());
    }
    assert!(ensure_reference_projections(&workspace, schema).unwrap());
    let read = workspace.read(schema).unwrap();
    assert_eq!(read.reference_revision().unwrap(), revision + 1);
    let alias_members: usize = read.connection().query_row(
        "SELECT count(*) FROM core_lexical_units cl JOIN lexical_units lu USING(unit_id) WHERE lu.unit_kind='root_alias' AND lu.alias_terms='Snow Keeper'",
        [], |row| row.get(0),
    ).unwrap();
    assert_eq!(alias_members, 1);
    let nonlexical = |values: BTreeMap<String, Vec<Vec<rusqlite::types::Value>>>| {
        values
            .into_iter()
            .filter(|(table, _)| !table.starts_with("lexical_"))
            .collect::<BTreeMap<_, _>>()
    };
    assert_eq!(
        nonlexical(table_values(read.connection())),
        nonlexical(before)
    );
    assert_eq!(
        read_stored_reference_context(read.connection())
            .unwrap()
            .unwrap()
            .core_sections_recipe,
        CORE_SECTIONS_RECIPE
    );
    drop(read);
    assert!(!ensure_reference_projections(&workspace, schema).unwrap());
}

#[test]
fn core_upgrade_classification_preserves_preparation_precedence_and_rejects_newer_policies() {
    for (core, preparation, expected) in [
        (
            "",
            atlas_record::source_content::CONTENT_INTERPRETATION_VERSION,
            Some(ReferenceUpgrade::CoreProjections),
        ),
        (
            "",
            "foundry-content/v21",
            Some(ReferenceUpgrade::SourcePreparation),
        ),
        (
            "core-sections/v2",
            atlas_record::source_content::CONTENT_INTERPRETATION_VERSION,
            None,
        ),
        (
            "unknown-core/v1",
            atlas_record::source_content::CONTENT_INTERPRETATION_VERSION,
            None,
        ),
    ] {
        // No bodies, units or Core tables: classification must not inspect them.
        let c = rusqlite::Connection::open_in_memory().unwrap();
        c.execute_batch("CREATE TABLE artifact_context(singleton INTEGER PRIMARY KEY,format_version INTEGER,context_hash TEXT,context_json TEXT);").unwrap();
        let mut context = crate::foundation_tests::fixture(false).context;
        context.core_sections_recipe = core.into();
        context.content_interpretation_version = preparation.into();
        c.execute(
            "INSERT INTO artifact_context VALUES(1,?,?,?)",
            rusqlite::params![
                context.format_version,
                context.sha256().unwrap(),
                serde_json::to_string(&context).unwrap()
            ],
        )
        .unwrap();
        match expected {
            Some(expected) => assert_eq!(classify_reference_upgrade(&c).unwrap(), expected),
            None => assert!(matches!(
                classify_reference_upgrade(&c),
                Err(IndexError::Unsupported(_))
            )),
        }
    }
}
