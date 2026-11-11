//! Retained evidence coverage is independent from physical schema readiness.
use crate::{test_support::*, *};
use atlas_domain::{DefinitionKey, OwnedContentIdentity, OwnedContentLocator};
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
fn retained_traits_only_coverage_keeps_visible_contract_without_requiring_hidden_targets() {
    let input = fixture();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("legacy.sqlite");
    publish_input(&path, &input).unwrap();
    let reader = open_reader(&path).unwrap();
    assert_eq!(reader.reference_coverage(), ReferenceCoverage::TraitsOnly);
    let counts: (u64, u64) = reader
        .native()
        .unwrap()
        .query_row(
            "SELECT count(DISTINCT record_key), count(*) FROM trait_references WHERE trait_key=?1",
            [input.traits[0].key.to_string()],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    assert_eq!(counts, (1, 2));
    drop(reader);
    validate_artifact(&path).unwrap();
}
