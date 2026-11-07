//! Retained evidence coverage is independent from physical schema readiness.
use crate::{test_support::*, *};
use atlas_domain::{DefinitionKey, OwnedContentIdentity, OwnedContentLocator, SearchProfileId};
use atlas_record::source_content::{ContentReferenceTarget, ResolvedContentReference};

fn fixture() -> IndexBuildInput {
    let mut input = crate::foundation_tests::fixture_with_description(
        true,
        "<p>@Localize[PF2E.TraitDragon]</p><span data-visibility=\"none\">@Localize[PF2E.TraitDragon]</span>",
    );
    let mut resolver = atlas_record::source_record::SourceReferenceIndex::default();
    for source in &input.records {
        resolver.insert_source(source.record.key(), source.record.source());
    }
    let resolve = |kind: &atlas_record::source_content::ContentReferenceKind, authored: &str| {
        resolver
            .resolve_global_content_reference(kind, authored)
            .or_else(|| {
                let target = if authored.ends_with("eeeeeeeeeeeeeeee") {
                    ContentReferenceTarget::Record {
                        key: "items:eeeeeeeeeeeeeeee".parse().unwrap(),
                    }
                } else if authored.ends_with("ffffffffffffffff") {
                    ContentReferenceTarget::OwnedNode {
                        key: "actors:aaaaaaaaaaaaaaaa".parse().unwrap(),
                        owners: vec![OwnedContentLocator {
                            collection: "items".into(),
                            identity: OwnedContentIdentity::Stable("ffffffffffffffff".into()),
                        }],
                    }
                } else {
                    return None;
                };
                Some(ResolvedContentReference {
                    target,
                    display_name: Some("Retained stale destination".into()),
                    item_family: Some("action".into()),
                })
            })
    };
    let markup = concat!(
        r#"<p>@UUID[Compendium.pf2e.items.Item.bbbbbbbbbbbbbbbb]{Visible staff}</p>"#,
        r#"<span data-visibility="none">@UUID[Compendium.pf2e.items.Item.cccccccccccccccc]{Hidden existing spell} "#,
        r#"@UUID[Compendium.pf2e.items.Item.eeeeeeeeeeeeeeee]{Hidden missing root} "#,
        r#"@UUID[Compendium.pf2e.actors.Actor.aaaaaaaaaaaaaaaa.Item.ffffffffffffffff]{Hidden stale owner}</span>"#,
    );
    let key: DefinitionKey = "trait:dragon".parse().unwrap();
    let prepared = atlas_record::source_content::prepare_reference_definition(
        key.clone(),
        markup,
        input.context.audience,
        None,
        Some(&resolve),
    )
    .unwrap();
    assert_eq!(prepared.references.iter().filter(|r| r.visible).count(), 1);
    assert_eq!(prepared.references.len(), 4);
    let text = format!("Dragon\n{}", prepared.text);
    let text_hash = crate::codec::sha256(text.as_bytes());
    let mut vector = vec![0.0; 768];
    vector[3] = 1.0;
    input.trait_vectors.push(TraitVectorInput {
        key: key.trait_key().unwrap(),
        profile_id: SearchProfileId::new("bge-base"),
        input_hash: crate::codec::sha256(
            format!("trait-definition-centroid/v1:{text_hash}").as_bytes(),
        ),
        vector: vector.clone(),
        chunks: vec![TraitSeedChunk {
            input: text.clone(),
            input_hash: text_hash.clone(),
            token_count: 2,
            address: atlas_domain::SourcePassageAddress::PlainSection {
                source_text_sha256: text_hash,
                selection_version: "trait-definition-centroid/v1".into(),
                section_ordinal: 0,
                label: Some("Dragon".into()),
                chunk_bytes: atlas_domain::SourceByteRange {
                    start: 0,
                    end: text.len(),
                },
            },
            vector,
        }],
    });
    input.traits.push(TraitDefinition {
        key: key.trait_key().unwrap(),
        label: "Dragon".into(),
        label_key: Some("PF2E.TraitDragon".into()),
        description_key: None,
        description_html: Some(prepared.html.clone()),
        authored_markup: Some(markup.into()),
        prepared: Some(prepared),
        catalogs: vec!["creatureTraits".into()],
        source_locations: vec![],
        source_revision: input.context.source_revision.clone(),
        indexing_locale: input.context.indexing_locale.clone(),
        content_locale: None,
    });
    input
}

#[test]
fn retained_traits_only_migration_keeps_visible_contract_without_requiring_hidden_targets() {
    let input = fixture();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("legacy.sqlite");
    let workspace = atlas_storage::Workspace::at(&path).unwrap();
    atlas_sqlite_vec::register_sqlite_vec_auto_extension().unwrap();
    workspace
        .migrate(&TEST_MIGRATIONS[..24], |_| Ok(()))
        .unwrap();
    {
        let owner = workspace.update_owner().unwrap();
        let maintenance = owner
            .maintenance(24, "retained hidden trait evidence")
            .unwrap();
        publish_pre_section_fixture(&maintenance, &input).unwrap();
    }
    workspace
        .migrate(&TEST_MIGRATIONS[..29], |_| Ok(()))
        .unwrap();
    workspace
        .migrate(&TEST_MIGRATIONS[..32], |_| Ok(()))
        .unwrap();
    stage_pre_glossary_catalog(&workspace, &input.context.remaster_inputs_identity).unwrap();
    workspace.migrate(TEST_MIGRATIONS, |_| Ok(())).unwrap();
    ensure_reference_projections(&workspace, TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
    let reader = open_reader(&path).unwrap();
    assert_eq!(reader.reference_coverage(), ReferenceCoverage::TraitsOnly);
    assert_eq!(
        reader
            .trait_references(&input.traits[0].key, 20, 0)
            .unwrap()
            .1,
        1
    );
    assert_eq!(
        reader
            .trait_reference_entries(&input.traits[0].key, 20, 0)
            .unwrap()
            .1,
        2
    );
    drop(reader);
    validate_artifact(&path).unwrap();
}
