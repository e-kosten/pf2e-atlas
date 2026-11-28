use super::*;
use crate::source_content::{ContentAudience, ContentVisibilityRule};
use crate::source_record::{SourceReferenceIndex, prepare_record_content};
use atlas_foundry_model::{SourceContext, admit_document_source};
use serde_json::{Value, json};

fn record(pack: &str, source: Value) -> SourceBackedRecord {
    let body = serde_json::to_vec(&source).unwrap();
    let admission = admit_document_source(
        "Item",
        SourceContext::new(pack, "progression fixture", "$"),
        &body,
    )
    .unwrap();
    SourceBackedRecord::new(pack, admission.model.unwrap()).unwrap()
}

fn feat(pack: &str, id: &str, name: &str, clauses: &[&str]) -> SourceBackedRecord {
    let clauses: Vec<_> = clauses.iter().map(|text| json!({"value": text})).collect();
    record(
        pack,
        json!({"_id": id, "type": "feat", "name": name, "system": {
            "prerequisites": {"value": clauses}, "category": "class", "traits": {"value": []}
        }}),
    )
}

fn progression(
    record: &SourceBackedRecord,
    index: &ProgressionIndex,
    resolver: &SourceReferenceIndex,
) -> Option<ProgressionRecord> {
    progression_at(
        record,
        "packs/pf2e/feats/archetype/medic/example.json",
        index,
        resolver,
    )
}

fn progression_at(
    record: &SourceBackedRecord,
    path: &str,
    index: &ProgressionIndex,
    resolver: &SourceReferenceIndex,
) -> Option<ProgressionRecord> {
    let content = prepare_record_content(
        record,
        ContentAudience {
            include_gm: false,
            include_owner: false,
            implicit_check_dc: ContentVisibilityRule::None,
        },
        None,
        Some(resolver),
    );
    prepare(record, path, index, &content)
}

#[test]
fn shared_archetype_alternatives_resolve_the_written_dedication_qualifier() {
    let dedication = |id, title| {
        record(
            "feats-srd",
            json!({"_id":id,"type":"feat","name":title,
                "system":{"category":"class","traits":{"value":["archetype","dedication"]}}
            }),
        )
    };
    let acrobat = dedication("AcrobatDedic0001", "Acrobat Dedication");
    let gladiator = dedication("GladiatDedic0001", "Gladiator Dedication");
    let sentinel = dedication("SentinelDedi0001", "Sentinel Dedication");
    let background = record(
        "backgrounds",
        json!({"_id":"AcrobatBackg0001","type":"background","name":"Acrobat","system":{}}),
    );
    let guardian = record(
        "classes",
        json!({"_id":"GuardianClas0001","type":"class","name":"Guardian","system":{}}),
    );
    let guardian_dedication = dedication("GuardianDedi0001", "Guardian Dedication");
    let mut index = ProgressionIndex::default();
    for candidate in [
        &acrobat,
        &gladiator,
        &sentinel,
        &background,
        &guardian,
        &guardian_dedication,
    ] {
        index.insert(candidate, "fixture.json");
    }
    let resolver = SourceReferenceIndex::default();
    let raw = "Acrobat or Gladiator Dedication";
    let fight = feat(
        "feats-srd",
        "Choreography0001",
        "Fight Choreography",
        &[raw],
    );
    let path = "packs/pf2e/feats/archetype/shared-archetype-feats/fight-choreography.json";
    let result = progression_at(&fight, path, &index, &resolver).unwrap();
    assert_eq!(
        result
            .record_targets()
            .iter()
            .map(|target| target.target_key)
            .collect::<Vec<_>>(),
        [acrobat.key(), gladiator.key()]
    );
    let PrerequisiteExpression::AnyOf { children } = &result.requirements[0].expression else {
        panic!("expected alternative dedications");
    };
    let PrerequisiteExpression::Record {
        text_range: Some(range),
        ..
    } = children[0]
    else {
        panic!("expected exact shorthand span");
    };
    assert_eq!(&raw[range.start..range.end], "Acrobat");

    let class_feat = feat(
        "feats-srd",
        "MightyBulwrk0001",
        "Mighty Bulwark",
        &["Guardian or Sentinel Dedication"],
    );
    let result = progression_at(
        &class_feat,
        "packs/pf2e/feats/class/guardian/mighty-bulwark.json",
        &index,
        &resolver,
    )
    .unwrap();
    assert_eq!(result.record_targets()[0].target_key, guardian.key());

    index.exclude_name("Acrobat Dedication");
    let result = progression_at(&fight, path, &index, &resolver).unwrap();
    assert_eq!(
        result.requirements[0].expression,
        PrerequisiteExpression::Unresolved {
            text: raw.to_owned()
        }
    );
}

#[test]
fn bare_capabilities_and_ancestry_variants_do_not_bind_to_one_record() {
    let names = [
        "Animal Companion",
        "Familiar",
        "Scurry!",
        "Scurry! (Gremlin)",
    ];
    let mut index = ProgressionIndex::default();
    for (i, name) in names.into_iter().enumerate() {
        index.insert(
            &feat("feats-srd", &format!("Alternative{i:05}"), name, &[]),
            "fixture.json",
        );
    }
    let source = feat(
        "feats-srd",
        "OtherFeat0000001",
        "Example",
        &[
            "Animal Companion or Familiar",
            "Animal Companion feat",
            "Scurry!",
            "Scurry! (Gremlin)",
        ],
    );
    let result = progression(&source, &index, &SourceReferenceIndex::default()).unwrap();
    assert_eq!(result.record_targets().len(), 1);
    assert_eq!(
        result.record_targets()[0].target_key.to_string(),
        "feats-srd:Alternative00003"
    );
}

#[test]
fn alternative_spans_keep_the_exact_record_and_rank_structure() {
    let medicine = feat("feats-srd", "MedicineFeat0001", "Battle Medicine", &[]);
    let heritage = record(
        "heritages",
        json!({"_id": "BuddingCentaur01", "type": "heritage", "name": "Budding Speaker Centaur", "system": {}}),
    );
    let raw = "  Budding Speaker Centaur heritage or Battle Medicine.  ";
    let source = feat(
        "feats-srd",
        "DoctorFeat000001",
        "Speaker's Defense",
        &[
            raw,
            "trained in Medicine and Diplomacy",
            "Charisma +2 or the ability to cast spells",
        ],
    );
    let mut index = ProgressionIndex::default();
    index.insert(&medicine, "packs/pf2e/feats/skill/battle-medicine.json");
    index.insert(
        &heritage,
        "packs/pf2e/heritages/centaur/budding-speaker-centaur.json",
    );
    let result = progression(&source, &index, &SourceReferenceIndex::default()).unwrap();
    let targets = result.record_targets();
    assert_eq!(targets.len(), 2);
    assert_eq!(targets[0].target_key, heritage.key());
    assert_eq!(targets[1].target_key, medicine.key());
    assert_eq!(
        targets[1].leaf_path,
        "/requirements/0/expression/children/1"
    );
    let PrerequisiteExpression::AnyOf { children } = &result.requirements[0].expression else {
        panic!("expected alternatives");
    };
    let PrerequisiteExpression::Record {
        text_range: Some(range),
        ..
    } = children[1]
    else {
        panic!("expected exact source range");
    };
    assert_eq!(&raw[range.start..range.end], "Battle Medicine");
    assert!(matches!(
        result.requirements[1].expression,
        PrerequisiteExpression::AllOf { ref children } if children.len() == 2
    ));
    assert!(matches!(
        result.requirements[2].expression,
        PrerequisiteExpression::AnyOf { ref children }
            if matches!(children[0], PrerequisiteExpression::AttributeMinimum { minimum: 2, .. })
                && matches!(children[1], PrerequisiteExpression::Unresolved { .. })
    ));
}

#[test]
fn excluded_and_ambiguous_titles_remain_unresolved() {
    let medicine = feat("feats-srd", "MedicineFeat0001", "Battle Medicine", &[]);
    let duplicate = feat("feats-srd", "MedicineFeat0002", "Battle Medicine", &[]);
    let source = feat(
        "feats-srd",
        "DoctorFeat000001",
        "Medic Dedication",
        &["Battle Medicine"],
    );
    let mut index = ProgressionIndex::default();
    let resolver = SourceReferenceIndex::default();
    index.insert(&medicine, "packs/pf2e/feats/skill/battle-medicine.json");
    assert_eq!(
        progression(&source, &index, &resolver)
            .unwrap()
            .record_targets()
            .len(),
        1
    );
    index.insert(
        &duplicate,
        "packs/pf2e/feats/skill/battle-medicine-legacy.json",
    );
    assert!(
        progression(&source, &index, &resolver)
            .unwrap()
            .record_targets()
            .is_empty()
    );
    let mut index = ProgressionIndex::default();
    index.insert(&medicine, "packs/pf2e/feats/skill/battle-medicine.json");
    index.exclude_name("Battle Medicine");
    assert!(
        progression(&source, &index, &resolver)
            .unwrap()
            .record_targets()
            .is_empty()
    );
}

#[test]
fn explicit_foundry_reference_retains_identity_despite_title_collision() {
    let medicine = feat("feats-srd", "MedicineFeat0001", "Battle Medicine", &[]);
    let duplicate = feat("feats-srd", "MedicineFeat0002", "Battle Medicine", &[]);
    let source = feat(
        "feats-srd",
        "DoctorFeat000001",
        "Medic Dedication",
        &["@UUID[Compendium.pf2e.feats-srd.Item.MedicineFeat0001]{Medicine feat}"],
    );
    let mut index = ProgressionIndex::default();
    let mut resolver = SourceReferenceIndex::default();
    for target in [&medicine, &duplicate] {
        index.insert(target, "packs/pf2e/feats/skill/battle-medicine.json");
        resolver.insert_source(target.key(), target.source());
    }
    let result = progression(&source, &index, &resolver).unwrap();
    assert_eq!(result.record_targets()[0].target_key, medicine.key());
    assert!(matches!(
        result.requirements[0].expression,
        PrerequisiteExpression::Record {
            evidence: PrerequisiteEvidence::AuthoredReference,
            text_range: None,
            ..
        }
    ));
}

#[test]
fn heritage_access_excludes_ignored_conditional_substituted_and_wrong_operation_rules() {
    let path = "system.details.ancestry.countsAs";
    let source = record(
        "heritages",
        json!({"_id":"BuddingCentaur01","name":"Aiuvarin","type":"heritage",
        "system":{"rules":[
            {"key":"ActiveEffectLike","mode":"add","path":path,"value":"elf"},
            {"key":"ActiveEffectLike","mode":"add","path":path,"value":"aiuvarin"},
            {"key":"ActiveEffectLike","mode":"add","path":path,"value":"gnome","ignored":true},
            {"key":"ActiveEffectLike","mode":"add","path":path,"value":"halfling","predicate":["has:feat"]},
            {"key":"ActiveEffectLike","mode":"override","path":path,"value":"human"},
            {"key":"ActiveEffectLike","mode":"add","path":path,"value":"{level}"},
            {"key":"ActiveEffectLike","mode":"add","path":"system.details.ancestry.other","value":"dwarf"}
        ]}}),
    );
    let result = progression(
        &source,
        &ProgressionIndex::default(),
        &SourceReferenceIndex::default(),
    )
    .unwrap();
    assert_eq!(
        result
            .ancestry_access
            .iter()
            .map(|access| access.trait_key.as_str())
            .collect::<Vec<_>>(),
        ["elf", "aiuvarin"]
    );
}

#[test]
fn shared_attribute_minimum_applies_to_every_named_alternative() {
    let source = feat(
        "feats-srd",
        "MagusDedication1",
        "Magus Dedication",
        &["Strength or Dexterity +2", "Intelligence or Wisdom +2"],
    );
    let result = progression(
        &source,
        &ProgressionIndex::default(),
        &SourceReferenceIndex::default(),
    )
    .unwrap();
    for (clause, (first, second)) in result.requirements.iter().zip([
        (Attribute::Strength, Attribute::Dexterity),
        (Attribute::Intelligence, Attribute::Wisdom),
    ]) {
        assert_eq!(
            clause.expression,
            PrerequisiteExpression::AnyOf {
                children: [first, second]
                    .into_iter()
                    .map(|attribute| PrerequisiteExpression::AttributeMinimum {
                        attribute,
                        minimum: 2,
                    })
                    .collect(),
            },
        );
    }
}

#[test]
fn ambiguous_shared_qualifiers_do_not_create_record_edges() {
    let medium = feat("classfeatures", "MediumFeature001", "Medium", &[]);
    let patron = feat("classfeatures", "PatronFeature001", "Patron", &[]);
    let source = feat(
        "feats-srd",
        "AmbiguousFeat001",
        "Ambiguous prerequisite fixture",
        &[
            "Medium or Small size",
            "you follow a good-aligned deity or patron",
        ],
    );
    let mut index = ProgressionIndex::default();
    index.insert(&medium, "packs/pf2e/class-features/medium.json");
    index.insert(&patron, "packs/pf2e/class-features/patron.json");
    let result = progression(&source, &index, &SourceReferenceIndex::default()).unwrap();
    assert!(result.record_targets().is_empty());
    assert!(
        result.requirements.iter().all(|clause| {
            matches!(clause.expression, PrerequisiteExpression::Unresolved { .. })
        })
    );
}
