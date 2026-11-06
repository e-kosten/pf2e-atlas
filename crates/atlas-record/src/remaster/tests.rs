use super::*;
use crate::source_content::{
    ContentAudience, ContentVisibilityRule, FormulaContext, PreparedSourceContent,
    SourceContentLocator, prepare_source_content,
};
fn prepare(value: &str) -> PreparedSourceContent {
    prepare_source_content(
        SourceContentLocator {
            record: "journals:abcdefghijklmnop".parse().unwrap(),
            owners: vec![],
            field: "/text/content".into(),
        },
        value,
        ContentAudience {
            include_gm: false,
            include_owner: false,
            implicit_check_dc: ContentVisibilityRule::None,
        },
        ContentVisibilityRule::All,
        None,
        None,
        &FormulaContext::default(),
    )
    .unwrap()
}
#[test]
fn selected_statements_keep_removed_compound_grouped_and_unknown_meaning() {
    let content = prepare(
        "<table><tr><th>Old</th><th>Status</th><th>New</th></tr><tr><td>Old</td><td>Removed</td><td>Folded into @UUID[Compendium.pf2e.feats-srd.Item.New]</td></tr><tr><td>Group</td><td>Renamed, altered mechanics</td><td>@UUID[one]{A}, @UUID[two]{B}</td></tr><tr><td>Thing</td><td>Future status</td><td>Context</td></tr></table>",
    );
    let declarations = journal_declarations("Spells", &content).unwrap();
    assert_eq!(declarations.len(), 3);
    assert_eq!(declarations[0].kinds, vec![RemasterChangeKind::Removed]);
    assert_eq!(declarations[0].after[0].references.len(), 1);
    assert_eq!(
        declarations[1].kinds,
        vec![
            RemasterChangeKind::Renamed,
            RemasterChangeKind::MechanicsChanged
        ]
    );
    assert_eq!(declarations[1].after[0].references.len(), 2);
    assert_eq!(
        declarations[2].kinds,
        vec![RemasterChangeKind::Unclassified]
    );
    assert_eq!(
        declarations[2].authored_status.as_deref(),
        Some("Future status")
    );
}
#[test]
fn selected_hidden_fragments_cannot_contribute_statement_text_or_terms() {
    let content = prepare(
        "<table><tr><th>Old</th><th>New</th></tr><tr data-visibility='gm'><td>Secret</td><td>@UUID[secret]</td></tr><tr><td>Visible</td><td>@UUID[public]</td></tr></table>",
    );
    let declarations = journal_declarations("Equipment", &content).unwrap();
    assert_eq!(declarations.len(), 1);
    assert_eq!(declarations[0].before[0].terms, vec!["Visible"]);
    assert!(
        !serde_json::to_string(&declarations)
            .unwrap()
            .contains("Secret")
    );
}
#[test]
fn canonical_statements_do_not_take_current_reference_display_names_or_bindings() {
    let mut content = prepare(
        "<table><tr><th>Old</th><th>New</th></tr><tr><td>Old</td><td>@UUID[Compendium.pf2e.spells-srd.Item.New]</td></tr></table>",
    );
    let a = journal_declarations("Equipment", &content).unwrap();
    content.references[0].label.target_name = Some("Changed runtime label".into());
    let b = journal_declarations("Equipment", &content).unwrap();
    assert_eq!(a, b);
    assert_eq!(
        a[0].after[0].references[0].target,
        "Compendium.pf2e.spells-srd.Item.New"
    );
}
#[test]
fn name_groups_and_nonrecord_terms_retain_context() {
    let content = prepare(
        "<ul><li>Aasimar, Aphorite, Ganzi, and Tiefling are merged into @UUID[Compendium.pf2e.heritages.Item.Nephilim].</li></ul><h2>Languages</h2><ul><li>Abyssal is now Chthonian.</li></ul>",
    );
    let declarations = journal_declarations("Remaster Changes", &content).unwrap();
    assert_eq!(
        declarations[0].before[0].terms,
        vec!["Aasimar", "Aphorite", "Ganzi", "Tiefling"]
    );
    assert_eq!(declarations[1].context.namespace, "language");
    assert_eq!(declarations[1].after[0].terms, vec!["Chthonian"]);
}
#[test]
fn static_replacement_literal_retains_absent_identifiers_and_rejects_execution() {
    let bytes = br#"class M { #spells = [{ from: { _id: "abcdefghijklmnop", name: "Old" }, to: { _id: "ponmlkjihgfedcba", name: "New" }, }, ]; }"#.to_vec();
    let input = RemasterSourceInput::new(
        RemasterInputKind::Migration887,
        RemasterOrigin::Foundry,
        "revision".into(),
        "887.ts".into(),
        bytes,
    );
    let declarations = file_declarations(&input).unwrap();
    assert_eq!(
        declarations[0].before[0].references[0]
            .explicit_key
            .as_ref()
            .unwrap()
            .to_string(),
        "spells-srd:abcdefghijklmnop"
    );
    let bad = RemasterSourceInput::new(
        RemasterInputKind::Migration887,
        RemasterOrigin::Foundry,
        "revision".into(),
        "887.ts".into(),
        br#"#spells=[{from:{_id:makeId(),name:"Old"},to:{_id:"id",name:"New"}}];"#.to_vec(),
    );
    assert!(file_declarations(&bad).is_err());
    let mut tampered = input;
    tampered.bytes.push(b' ');
    assert!(file_declarations(&tampered).is_err());
}
#[test]
#[ignore = "requires the authenticated pinned PF2e source via ATLAS_PF2E_SOURCE"]
fn pinned_remaster_journal_and_static_inputs_reconcile_all_authored_forms() {
    let root = std::path::PathBuf::from(std::env::var_os("ATLAS_PF2E_SOURCE").expect("source"));
    let journal: serde_json::Value = serde_json::from_slice(
        &std::fs::read(root.join("packs/pf2e/journals/remaster-changes.json")).unwrap(),
    )
    .unwrap();
    let mut rows = 0;
    let mut removed = 0;
    let mut intro_groups = 0;
    let mut intro_names = 0;
    let mut term_groups = 0;
    let mut journal_total = 0;
    let mut discussion_count = 0;
    for page in journal["pages"].as_array().unwrap() {
        let declarations = journal_declarations(
            page["name"].as_str().unwrap(),
            &prepare(page["text"]["content"].as_str().unwrap()),
        )
        .unwrap();
        journal_total += declarations.len();
        for declaration in declarations {
            discussion_count +=
                usize::from(declaration.kinds.contains(&RemasterChangeKind::Discussion));
            if matches!(
                declaration.citations[0],
                RemasterCitation::Journal {
                    selector: RemasterSelector::ListItem { .. },
                    ..
                }
            ) {
                if page["name"] == "Remaster Changes" {
                    intro_groups += 1;
                    intro_names += declaration
                        .before
                        .iter()
                        .map(|s| s.terms.len())
                        .sum::<usize>();
                } else {
                    term_groups += 1;
                }
            }
            if matches!(
                declaration.citations[0],
                RemasterCitation::Journal {
                    selector: RemasterSelector::TableRow { .. },
                    ..
                }
            ) {
                rows += 1;
            }
            if declaration.kinds.contains(&RemasterChangeKind::Removed) {
                removed += 1;
            }
        }
    }
    assert_eq!(rows, 560);
    assert_eq!(removed, 54);
    assert_eq!((intro_groups, intro_names, term_groups), (6, 9, 21));
    assert_eq!((journal_total, discussion_count), (604, 17));
    for (name, kind, count) in [
        (
            "850-flat-footed-to-off-guard.ts",
            RemasterInputKind::Migration850,
            1,
        ),
        (
            "887-redirect-spell-links.ts",
            RemasterInputKind::Migration887,
            6,
        ),
    ] {
        let path = format!("src/module/migration/migrations/{name}");
        let input = RemasterSourceInput::new(
            kind,
            RemasterOrigin::Foundry,
            "28d9d156f252d084371d994dc359509b49eadf18".into(),
            path.clone(),
            std::fs::read(root.join(path)).unwrap(),
        );
        assert_eq!(file_declarations(&input).unwrap().len(), count);
    }
}

fn reviewed_input(before: &str, after: &str) -> RemasterSourceInput {
    RemasterSourceInput::new(
        RemasterInputKind::AtlasCorrespondence,
        RemasterOrigin::AtlasReviewed,
        "fixture".into(),
        format!("{before}-{after}.json"),
        serde_json::to_vec(&ReviewedCorrespondences {
            entries: vec![ReviewedCorrespondence {
                before: format!("actions:{before}").parse().unwrap(),
                after: format!("actions:{after}").parse().unwrap(),
                before_label: "Old".into(),
                after_label: "New".into(),
                namespace: "fixture".into(),
                rationale: "Explicit test assertion".into(),
                source_type: "action".into(),
            }],
        })
        .unwrap(),
    )
}
fn sources() -> Vec<RemasterSourceInput> {
    [
        ("aaaaaaaaaaaaaaaa", "bbbbbbbbbbbbbbbb"),
        ("aaaaaaaaaaaaaaaa", "cccccccccccccccc"),
        ("bbbbbbbbbbbbbbbb", "aaaaaaaaaaaaaaaa"),
    ]
    .into_iter()
    .map(|(a, b)| reviewed_input(a, b))
    .collect()
}
fn reviewed(before: &str, after: &str) -> (RemasterDeclaration, Vec<RemasterBinding>) {
    let source = reviewed_input(before, after);
    let declaration = file_declarations(&source).unwrap().remove(0);
    let bindings =
        bind_remaster_declarations(std::slice::from_ref(&declaration), &facts(), &[]).unwrap();
    (declaration, bindings)
}
fn facts() -> std::collections::BTreeMap<atlas_domain::RecordKey, RemasterRecordFacts> {
    ["aaaaaaaaaaaaaaaa", "bbbbbbbbbbbbbbbb", "cccccccccccccccc"]
        .into_iter()
        .map(|id| {
            (
                format!("actions:{id}").parse().unwrap(),
                RemasterRecordFacts {
                    name: Some(id.into()),
                    source_type: Some("action".into()),
                },
            )
        })
        .collect()
}
#[test]
fn preference_retains_repeated_support_but_rejects_conflicts_cycles_groups_and_removal() {
    let (a, ab) = reviewed("aaaaaaaaaaaaaaaa", "bbbbbbbbbbbbbbbb");
    let mut repeat = a.clone();
    repeat.context.qualifier = Some("Independent repeated citation".into());
    let repeat = repeat.seal().unwrap();
    let rb = bind_remaster_declarations(std::slice::from_ref(&repeat), &facts(), &[]).unwrap();
    let projection = derive_remaster_projection(
        &[a.clone(), repeat],
        &[ab.clone(), rb].concat(),
        &facts(),
        &sources(),
    );
    assert_eq!(projection.preferences.len(), 1);
    assert_eq!(projection.preferences[0].declarations.len(), 2);
    assert!(projection.diagnostics.is_empty());
    let (conflict, cb) = reviewed("aaaaaaaaaaaaaaaa", "cccccccccccccccc");
    assert!(
        derive_remaster_projection(
            &[a.clone(), conflict],
            &[ab.clone(), cb].concat(),
            &facts(),
            &sources()
        )
        .preferences
        .is_empty()
    );
    let (reverse, reverseb) = reviewed("bbbbbbbbbbbbbbbb", "aaaaaaaaaaaaaaaa");
    assert!(
        derive_remaster_projection(
            &[a.clone(), reverse],
            &[ab.clone(), reverseb].concat(),
            &facts(),
            &sources()
        )
        .preferences
        .is_empty()
    );
    for kind in [RemasterChangeKind::Removed, RemasterChangeKind::Merged] {
        let mut counter = a.clone();
        counter.kinds = vec![kind];
        let counter = counter.seal().unwrap();
        let bindings =
            bind_remaster_declarations(std::slice::from_ref(&counter), &facts(), &[]).unwrap();
        assert!(
            derive_remaster_projection(
                &[a.clone(), counter],
                &[ab.clone(), bindings].concat(),
                &facts(),
                &sources()
            )
            .preferences
            .is_empty()
        );
    }
    let mut grouped = a.clone();
    let second = grouped.after[0].references[0].clone();
    grouped.after[0].references.push(second);
    let grouped = grouped.seal().unwrap();
    let gb = bind_remaster_declarations(std::slice::from_ref(&grouped), &facts(), &[]).unwrap();
    assert!(
        derive_remaster_projection(&[a, grouped], &[ab, gb].concat(), &facts(), &sources())
            .preferences
            .is_empty()
    );
}
#[test]
fn resolver_veto_is_not_rebound_but_absent_literal_identity_is_retained() {
    let content = prepare(
        "<table><tr><th>Old</th><th>New</th></tr><tr><td>@UUID[Compendium.pf2e.actions.Item.aaaaaaaaaaaaaaaa]</td><td>@UUID[Compendium.pf2e.actions.Item.dddddddddddddddd]</td></tr></table>",
    );
    let declarations = journal_declarations("Class Features", &content).unwrap();
    let selected = RemasterSelectedField {
        locator: content.locator.clone(),
        authored_sha256: content.authored_markup_sha256.clone(),
        references: content.references.clone(),
    };
    let bindings = bind_remaster_declarations(&declarations, &facts(), &[selected]).unwrap();
    assert_eq!(
        bindings[0].resolution,
        RemasterBindingResolution::Unresolved
    );
    assert!(
        matches!(&bindings[1].resolution,RemasterBindingResolution::MissingExplicitKey{record_key} if record_key.to_string()=="actions:dddddddddddddddd")
    );
    assert!(
        derive_remaster_projection(&declarations, &bindings, &facts(), &sources())
            .preferences
            .is_empty()
    );
}

#[test]
fn unknown_reordered_headers_sections_and_statuses_never_grant_lookup_authority() {
    for (section, headers, status) in [
        (
            "Spells",
            "<th>New</th><th>Status</th><th>Old</th>",
            "Renamed",
        ),
        (
            "Unknown source section",
            "<th>Old</th><th>Status</th><th>New</th>",
            "Renamed",
        ),
        (
            "Spells",
            "<th>Old</th><th>Status</th><th>New</th>",
            "Renamed, Future status",
        ),
        (
            "Spells",
            "<th>Old</th><th>Status</th><th>New</th>",
            "Not renamed",
        ),
        ("Spells", "", "Renamed"),
    ] {
        let content = prepare(&format!(
            "<table><tr>{headers}</tr><tr><td>Authored old name</td><td>{status}</td><td>@UUID[Compendium.pf2e.actions.Item.bbbbbbbbbbbbbbbb]</td></tr></table>"
        ));
        let declarations = journal_declarations(section, &content).unwrap();
        assert_eq!(declarations.len(), 1);
        assert!(
            declarations[0]
                .kinds
                .contains(&RemasterChangeKind::Unclassified),
            "{section}/{headers}/{status}"
        );
        let binding = RemasterBinding {
            declaration: declarations[0].id.clone(),
            side: RemasterSideKind::After,
            group: 0,
            reference: 0,
            resolution: RemasterBindingResolution::Current {
                record_key: "actions:bbbbbbbbbbbbbbbb".parse().unwrap(),
                owners: vec![],
            },
        };
        let projection = derive_remaster_projection(&declarations, &[binding], &facts(), &[]);
        assert!(projection.aliases.is_empty());
        assert!(projection.preferences.is_empty());
    }
}
#[test]
fn conditional_adapter_withdraws_aliases_as_well_as_preference_when_current_facts_fail() {
    let source = reviewed_input("aaaaaaaaaaaaaaaa", "bbbbbbbbbbbbbbbb");
    let declarations = file_declarations(&source).unwrap();
    for defect in [
        "missing predecessor",
        "wrong predecessor family",
        "wrong successor family",
    ] {
        let mut records = facts();
        match defect {
            "missing predecessor" => {
                records.remove(&"actions:aaaaaaaaaaaaaaaa".parse().unwrap());
            }
            "wrong predecessor family" => {
                records
                    .get_mut(&"actions:aaaaaaaaaaaaaaaa".parse().unwrap())
                    .unwrap()
                    .source_type = Some("feat".into())
            }
            _ => {
                records
                    .get_mut(&"actions:bbbbbbbbbbbbbbbb".parse().unwrap())
                    .unwrap()
                    .source_type = Some("feat".into())
            }
        }
        let bindings = bind_remaster_declarations(&declarations, &records, &[]).unwrap();
        let projection = derive_remaster_projection(
            &declarations,
            &bindings,
            &records,
            std::slice::from_ref(&source),
        );
        assert!(projection.aliases.is_empty(), "{defect}");
        assert!(projection.preferences.is_empty(), "{defect}");
        assert!(
            projection
                .diagnostics
                .iter()
                .any(|d| d.reason == RemasterDerivationReason::AdapterConditions)
        );
    }
    let input=RemasterSourceInput::new(RemasterInputKind::Migration850,RemasterOrigin::Foundry,"fixture".into(),"850.ts".into(),br##"class M { #oldNamePattern = new RegExp(/\bFlat-Footed\b/g); #newName = "Off-Guard"; condition = "AJh5ex99aV6VTggg"; }"##.to_vec());
    let declarations = file_declarations(&input).unwrap();
    let records = std::collections::BTreeMap::from([(
        "conditionitems:AJh5ex99aV6VTggg".parse().unwrap(),
        RemasterRecordFacts {
            name: Some("Wrong family".into()),
            source_type: Some("feat".into()),
        },
    )]);
    let bindings = bind_remaster_declarations(&declarations, &records, &[]).unwrap();
    assert!(
        derive_remaster_projection(&declarations, &bindings, &records, &[input])
            .aliases
            .is_empty()
    );
}
#[test]
fn migration887_absent_predecessor_keeps_verified_spell_lookup_without_a_preference() {
    let input=RemasterSourceInput::new(RemasterInputKind::Migration887,RemasterOrigin::Foundry,"fixture".into(),"887.ts".into(),br#"class M { #spells=[{from:{_id:"aaaaaaaaaaaaaaaa",name:"Old spell"},to:{_id:"bbbbbbbbbbbbbbbb",name:"Current spell"}}]; }"#.to_vec());
    let declarations = file_declarations(&input).unwrap();
    let records = std::collections::BTreeMap::from([(
        "spells-srd:bbbbbbbbbbbbbbbb".parse().unwrap(),
        RemasterRecordFacts {
            name: Some("Current spell".into()),
            source_type: Some("spell".into()),
        },
    )]);
    let bindings = bind_remaster_declarations(&declarations, &records, &[]).unwrap();
    let projection = derive_remaster_projection(&declarations, &bindings, &records, &[input]);
    assert_eq!(projection.aliases[0].alias, "Old spell");
    assert!(projection.preferences.is_empty());
}

#[test]
fn table_header_paragraphs_are_not_rules_discussion() {
    let prepared = prepare(
        "<p>Selected source discussion.</p><table><tr><th><p>Old Name</p></th><th><p>New Name</p></th></tr><tr><td><p>Old</p></td><td><p>New</p></td></tr></table>",
    );
    let declarations = journal_declarations("Equipment", &prepared).unwrap();
    assert_eq!(declarations.len(), 2);
    let discussion = declarations
        .iter()
        .filter(|d| d.kinds.contains(&RemasterChangeKind::Discussion))
        .collect::<Vec<_>>();
    assert_eq!(discussion.len(), 1);
    assert_eq!(discussion[0].after[0].text, "Selected source discussion.");
}
