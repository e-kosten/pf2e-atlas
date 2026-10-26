use super::*;
use crate::source_content::{ContentReferenceResolution, ContentReferenceTarget};
use atlas_domain::{OwnedContentIdentity, OwnedContentLocator};
use atlas_foundry_model::{SourceContext, admit_document_source};
use serde_json::{Value, json};

fn record(pack: &str, kind: &str, value: Value) -> SourceBackedRecord {
    let source = admit_document_source(
        kind,
        SourceContext::new(pack, "fixture", "$"),
        &serde_json::to_vec(&value).unwrap(),
    )
    .unwrap();
    SourceBackedRecord::new(pack, source.model.unwrap()).unwrap()
}

#[test]
fn declared_grants_preserve_owned_attribution_and_remain_distinct_from_applied_grants() {
    let source = record(
        "actors",
        "Actor",
        json!({"_id":"aaaaaaaaaaaaaaaa","type":"npc","items":[
            {"_id":"bbbbbbbbbbbbbbbb","name":"Conditional ability","type":"action",
             "flags":{"pf2e":{"itemGrants":{"existing":{"id":"cccccccccccccccc"}}}},
             "system":{"rules":[
                {"key":"GrantItem","uuid":"Compendium.pf2e.effects.Item.dddddddddddddddd"},
                {"key":"GrantItem","uuid":"Compendium.pf2e.effects.Item.dddddddddddddddd",
                 "predicate":["self:trait:undead"],
                 "alterations":[{"mode":"override","property":"badge-value","value":2}]}
             ]}},
            {"_id":"cccccccccccccccc","name":"Already owned","type":"effect"}
        ]}),
    );
    let target = record(
        "effects",
        "Item",
        json!({"_id":"dddddddddddddddd","name":"Reference effect","type":"effect"}),
    );
    let mut index = SourceReferenceIndex::default();
    for r in [&source, &target] {
        index.insert_source(r.key(), r.source());
    }
    let relationships = resolve_source_relationships(&source, Some(&index));
    let declared: Vec<_> = relationships
        .iter()
        .filter(|r| r.kind == SourceRelationshipKind::DeclaredGrant)
        .collect();
    assert_eq!(declared.len(), 2);
    for (ordinal, occurrence) in declared.iter().enumerate() {
        assert_eq!(
            occurrence.locator.owners,
            vec![OwnedContentLocator {
                collection: "/items".into(),
                identity: OwnedContentIdentity::Stable("bbbbbbbbbbbbbbbb".into())
            }]
        );
        assert_eq!(
            occurrence.locator.field,
            format!("/system/rules/{ordinal}/uuid")
        );
        assert_eq!(
            occurrence.resolution,
            ContentReferenceResolution::Resolved(ContentReferenceTarget::Record {
                key: target.key().clone()
            })
        );
    }
    assert!(
        relationships
            .iter()
            .any(|r| r.kind == SourceRelationshipKind::ItemGrant
                && matches!(&r.resolution, ContentReferenceResolution::Resolved(
            ContentReferenceTarget::OwnedNode { owners, .. }
        ) if owners[0].identity == OwnedContentIdentity::Stable("cccccccccccccccc".into())))
    );
}

#[test]
fn declared_grants_omit_dynamic_ignored_and_rejected_rules_but_keep_static_unavailable_targets() {
    let source = record(
        "backgrounds",
        "Item",
        json!({"_id":"aaaaaaaaaaaaaaaa","type":"background","system":{"rules":[
            {"key":"GrantItem","uuid":"{item|flags.pf2e.rulesSelections.feat}"},
            {"key":"GrantItem","uuid":"Compendium.pf2e.feats-srd.Item.{actor|flags.choice}"},
            {"key":"GrantItem","uuid":"Compendium.pf2e.effects.Item.bbbbbbbbbbbbbbbb","ignored":true},
            {"key":"GrantItem","uuid":42},
            {"key":"GrantItem","uuid":"Compendium.pf2e.effects.Item.bbbbbbbbbbbbbbbb","predicate":42},
            {"key":"GrantItem","uuid":"bbbbbbbbbbbbbbbb"},
            {"key":"GrantItem","uuid":"Compendium.pf2e-animal-companions.AC-Features.Item.Missing"},
            {"key":"GrantItem","uuid":"Compendium.pf2e.effects.Item.Missing"},
            {"key":"ChoiceSet","choices":[{"value":"Compendium.pf2e.effects.Item.bbbbbbbbbbbbbbbb"}]}
        ]}}),
    );
    let relationships =
        resolve_source_relationships(&source, Some(&SourceReferenceIndex::default()));
    let declared: Vec<_> = relationships
        .iter()
        .filter(|r| r.kind == SourceRelationshipKind::DeclaredGrant)
        .collect();
    assert_eq!(
        declared
            .iter()
            .map(|r| r.locator.field.as_str())
            .collect::<Vec<_>>(),
        ["/system/rules/6/uuid", "/system/rules/7/uuid"]
    );
    assert!(
        declared
            .iter()
            .all(|r| r.resolution == ContentReferenceResolution::Unresolved)
    );
}

#[test]
fn declared_feature_lists_use_canonical_targets_and_escape_source_keys_across_abc_families() {
    let target = record(
        "feats-srd",
        "Item",
        json!({"_id":"bbbbbbbbbbbbbbbb","name":"Verified feature","type":"feat"}),
    );
    let mut index = SourceReferenceIndex::default();
    index.insert_source(target.key(), target.source());
    for family in ["ancestry", "background", "class"] {
        let source = record(
            "features",
            "Item",
            json!({"_id":"aaaaaaaaaaaaaaaa","type":family,"system":{"items":{
                "with/slash~":{"name":"An unrelated label","level":1,
                    "uuid":"Compendium.pf2e.feats-srd.Item.bbbbbbbbbbbbbbbb"},
                "dynamic":{"uuid":"Compendium.pf2e.feats-srd.Item.{actor|flags.feat}"},
                "invalid":{"uuid":42}
            }}}),
        );
        let relationships = resolve_source_relationships(&source, Some(&index));
        let declared: Vec<_> = relationships
            .iter()
            .filter(|r| r.kind == SourceRelationshipKind::DeclaredGrant)
            .collect();
        assert_eq!(declared.len(), 1, "{family}");
        assert_eq!(
            declared[0].locator.field,
            "/system/items/with~1slash~0/uuid"
        );
        assert_eq!(
            declared[0].resolution,
            ContentReferenceResolution::Resolved(ContentReferenceTarget::Record {
                key: target.key().clone()
            })
        );
    }
}
