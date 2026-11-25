//! Stat-block immunity links use the current reader's admitted records and traits.
use super::*;
use atlas_search::test_support::{
    ReferencePreparation, TraitDefinition, reference_inventory_identity,
};
use atlas_search::{AtlasRetrievalService, GetRecordRequest};

const SWARM: &str = "bestiary-ability-glossary-srd:baA0nSMhQyFyJIia";
const BLINDED: &str = "conditionitems:XgEqL1kFApUbl5Z2";
const RULE: &str = "pf2e:PF2E.NPC.Abilities.Glossary.SwarmMind";
const TOKENS: &[&str] = &[
    "fear-effects",
    "disease",
    "swarm-mind",
    "blinded",
    "stunned",
    "clumsy",
    "emotion",
    "mental",
    "poison",
    "time",
    "visual", // Its trait exists, but the source immunity entry is absent.
];

#[derive(Clone, Copy)]
enum Workspace {
    Complete,
    ForeignConditions,
    ConflictingSwarm,
    TraitsOnly,
}

fn immunity_fixture(scope: Workspace) -> crate::test_support::FixtureWorker {
    crate::test_support::fixture_worker_with_executor(
        crate::executor::RetrievalExecutor::from_test_fixture_factory(1, 16, move || {
            let mut rows: Vec<_> = TOKENS.iter().map(|token| json!({"type": token})).collect();
            rows[0]["exceptions"] = json!(["silver"]);
            let actors = [
                ("creatures", "npc"),
                ("hazards", "hazard"),
                ("armies", "army"),
                ("vehicles", "vehicle"),
                ("foreign", "npc"),
            ];
            let mut records: Vec<_> = actors
                .into_iter()
                .map(|(pack, family)| {
                    record(
                        pack,
                        "Actor",
                        json!({
                            "_id": "aaaaaaaaaaaaaaaa",
                            "type": family,
                            "name": pack,
                            "system": {"attributes": {"immunities": rows}}
                        }),
                    )
                })
                .collect();
            for (pack, id, family) in [
                ("conditionitems", "XgEqL1kFApUbl5Z2", "condition"),
                ("conditionitems", "dfCMdR4wnpbYNTix", "action"), // Stunned ID, wrong family.
                (
                    "bestiary-ability-glossary-srd",
                    "baA0nSMhQyFyJIia",
                    "action",
                ),
                (
                    "bestiary-ability-glossary-srd",
                    "bbbbbbbbbbbbbbbb",
                    "action",
                ),
            ] {
                records.push(record(
                    pack,
                    "Item",
                    json!({
                        "_id":id,"type":family,"name":"Localized target",
                        "system":{"description":{"value":"<p>Fixture description.</p>"}}
                    }),
                ));
            }
            let mut data = input(records, false);
            for token in [
                "fear", "disease", "emotion", "mental", "poison", "time", "visual",
            ] {
                let markup = if token == "emotion" {
                    "<p></p>"
                } else {
                    "<p>Readable trait.</p>"
                };
                let key = atlas_domain::TraitKey::new(token).unwrap();
                let prepared = atlas_record::source_content::prepare_reference_definition(
                    key.clone().into(),
                    markup,
                    data.context.audience,
                    None,
                    None,
                )
                .unwrap();
                data.traits.push(TraitDefinition {
                    key,
                    label: format!("Target {token}"),
                    label_key: None,
                    description_key: None,
                    description_html: Some(prepared.html.clone()),
                    authored_markup: Some(markup.into()),
                    prepared: Some(prepared),
                    catalogs: vec!["fixture".into()],
                    source_locations: vec![],
                    source_revision: None,
                    indexing_locale: "en".into(),
                    content_locale: Some("en".into()),
                });
            }
            if matches!(scope, Workspace::TraitsOnly) {
                return Ok(atlas_search::test_support::open_input_fixture(data)?);
            }
            let mut entries: Vec<_> = TOKENS
                .iter()
                .copied()
                .filter(|token| *token != "visual")
                .map(|token| metadata(N::ImmunityType, token, &format!("Immunity {token}"), None))
                .collect();
            let mut rule = metadata(N::Rule, RULE, "Swarm Mind rule", None);
            let target: atlas_domain::RecordKey = match scope {
                Workspace::ConflictingSwarm => "bestiary-ability-glossary-srd:bbbbbbbbbbbbbbbb",
                _ => SWARM,
            }
            .parse()
            .unwrap();
            rule.provenance.record_mapping = Some(ReferenceRecordMapping {
                target: target.clone(),
                package: "pf2e".into(),
                expected_document_kind: "Item".into(),
                expected_item_type: Some("action".into()),
                required_localization_key: None,
            });
            rule.availability = ReferenceAvailability::Available;
            rule.permission.disposition = ReferencePermissionDisposition::ExistingRecordPolicy;
            rule.backing = ReferenceBacking::Record { key: target };
            entries.push(rule);
            let (expected_entry_count, entry_keys_sha256) = reference_inventory_identity(
                entries
                    .iter()
                    .map(|entry| entry.key.clone())
                    .chain(data.traits.iter().map(|t| t.key.clone().into())),
            )?;
            let mut pack_systems =
                std::collections::BTreeMap::from([("foreign".into(), Some("sf2e".into()))]);
            if matches!(scope, Workspace::ForeignConditions) {
                pack_systems.insert("conditionitems".into(), Some("sf2e".into()));
            }
            data.context.reference_preparation = Some(ReferencePreparation {
                source_package: Some("pf2e".into()),
                pack_systems,
                recipe: "immunity-reading/v1".into(),
                input_hash: hash(b"immunity fixture"),
                coverage: ReferenceCoverage::Complete,
                expected_entry_count,
                entry_keys_sha256,
            });
            Ok(
                atlas_search::test_support::open_input_fixture_with_references(
                    data,
                    &entries,
                    &[],
                )?,
            )
        }),
    )
}

fn immunity_rows(presentation: &RecordPresentationView) -> &[IwrEntryView] {
    let rows = match &presentation.body {
        RecordBodyView::Creature(v) => &v.immunities,
        RecordBodyView::Hazard(v) => &v.actor.immunities,
        RecordBodyView::Army(v) => &v.immunities,
        RecordBodyView::Vehicle(v) => &v.defenses.immunities,
        _ => panic!("expected a stat block"),
    };
    rows.value.as_deref().unwrap()
}

#[test]
fn stat_block_immunities_keep_their_labels_and_open_the_same_verified_targets_in_each_reader() {
    let fixture = immunity_fixture(Workspace::Complete);
    for pack in ["creatures", "hazards", "armies", "vehicles"] {
        let detail = fixture
            .worker
            .record_detail(&format!("{pack}:aaaaaaaaaaaaaaaa"))
            .unwrap();
        let rows = immunity_rows(&detail.presentation);
        assert_eq!(rows.len(), TOKENS.len(), "{pack}");
        assert_eq!(
            rows[0].exceptions.value.as_deref(),
            Some(["silver".to_owned()].as_slice())
        );
        for (row, token) in rows.iter().zip(TOKENS.iter().copied()) {
            let value = row.damage_type.value.as_ref().unwrap();
            assert_eq!(value.value, token);
            if token == "visual" {
                assert_eq!(value.label, token);
            } else {
                assert_eq!(value.label, format!("Immunity {token}"));
            }
            match token {
                "fear-effects" | "disease" => {
                    let target = if token == "fear-effects" {
                        "trait:fear"
                    } else {
                        "trait:disease"
                    };
                    assert!(
                        matches!(&value.help, Some(ReferenceHelpView {
                        destination: ReferenceDestinationView::Definition { key }, ..
                    }) if key == target),
                        "{pack} {token}"
                    );
                }
                "swarm-mind" | "blinded" => {
                    let target = if token == "blinded" { BLINDED } else { SWARM };
                    assert!(
                        matches!(&value.help, Some(ReferenceHelpView {
                        definition_key: None, destination: ReferenceDestinationView::Record { navigation },
                    }) if navigation.record_key == target && navigation.owners.is_empty()),
                        "{pack} {token}"
                    );
                }
                _ => assert!(value.help.is_none(), "{pack} {token}"),
            }
        }
    }
}

fn unannotated_actor(retrieval: &AtlasRetrievalService, pack: &str) -> RecordPresentationView {
    let key = format!("{pack}:aaaaaaaaaaaaaaaa").parse().unwrap();
    let detail = retrieval
        .get_record(GetRecordRequest {
            record_key: &key,
            selected_content: &[],
        })
        .unwrap()
        .unwrap();
    super::super::record_presentation(
        &detail.source,
        detail.source.node_at(&[]).unwrap(),
        &crate::projection::localized_detail_summary_view(&detail, retrieval),
        &[],
        None,
        retrieval,
    )
    .unwrap()
}

#[test]
fn batch_shares_target_reads_while_checking_each_owners_pack() {
    let fixture = immunity_fixture(Workspace::Complete);
    fixture
        .worker
        .with_retrieval(&crate::ReadExecutionContext::uncancellable(), |retrieval| {
            let actor = unannotated_actor(retrieval, "creatures");
            let foreign = unannotated_actor(retrieval, "foreign");
            let mut roster = [actor.clone(), foreign, actor];
            retrieval.reset_read_metrics();
            super::super::reference_help::annotate_many(roster.iter_mut(), retrieval)?;
            let metrics = retrieval.read_metrics();
            assert_eq!(
                metrics.summary_batches, 1,
                "one batch for shared condition targets"
            );
            assert_eq!(metrics.source_body_decodes, 0);
            for (presentation, expected) in roster.iter().zip([true, false, true]) {
                for row in &immunity_rows(presentation)[..4] {
                    let value = row.damage_type.value.as_ref().unwrap();
                    assert_eq!(value.help.is_some(), expected, "{}", value.value);
                }
            }
            Ok(())
        })
        .unwrap();
}

#[test]
fn incomplete_or_conflicting_reference_evidence_withholds_the_affected_links() {
    for (scope, unlinked) in [
        (Workspace::TraitsOnly, &TOKENS[..4]),
        (Workspace::ForeignConditions, &["blinded"][..]),
        (Workspace::ConflictingSwarm, &["swarm-mind"][..]),
    ] {
        let fixture = immunity_fixture(scope);
        let detail = fixture
            .worker
            .record_detail("creatures:aaaaaaaaaaaaaaaa")
            .unwrap();
        let rows = immunity_rows(&detail.presentation);
        for token in unlinked {
            let value = rows
                .iter()
                .filter_map(|r| r.damage_type.value.as_ref())
                .find(|v| v.value == *token)
                .unwrap();
            assert!(value.help.is_none(), "{token}");
        }
    }
}
