use super::*;
use atlas_app_model::*;
use atlas_record::source_content::{OwnedContentIdentity, OwnedContentLocator};

fn known<T>(value: T) -> FactView<T> {
    FactView {
        state: QueryFieldState::Value,
        value: Some(value),
    }
}
fn missing<T>() -> FactView<T> {
    FactView {
        state: QueryFieldState::Missing,
        value: None,
    }
}
fn cleared<T>() -> FactView<T> {
    FactView {
        state: QueryFieldState::Null,
        value: None,
    }
}
fn no_changes() -> SpellChangeView {
    SpellChangeView {
        cast: missing(),
        range: missing(),
        target: missing(),
        area: missing(),
        defense: missing(),
        duration: missing(),
        sustained: missing(),
        traits: missing(),
        traditions: missing(),
        damage: missing(),
        heightening: missing(),
    }
}
fn numeric(value: i64) -> NumberFactView {
    NumberFactView {
        state: QueryFieldState::Value,
        value: Some(value.into()),
        adjustment: None,
    }
}
fn absent_number() -> NumberFactView {
    NumberFactView {
        state: QueryFieldState::NotApplicable,
        value: None,
        adjustment: None,
    }
}
fn summary(title: &str, family: &str) -> RecordSummaryView {
    RecordSummaryView {
        record_key: "creatures:caster".into(),
        title: title.into(),
        kind: family.into(),
        kind_label: family.into(),
        source_type: None,
        level_label: Some("Rank 2".into()),
        level_basis: None,
        rarity: Some("uncommon".into()),
        traits: vec![RecordBadgeView {
            kind: "trait".into(),
            label: "Fire".into(),
            value: "fire".into(),
        }],
        publication: Some("Local Bestiary".into()),
        pack: Some("Creatures".into()),
    }
}
fn address() -> RecordNavigationView {
    RecordNavigationView {
        record_key: "creatures:caster".into(),
        owners: vec![OwnedContentLocator {
            collection: "/items".into(),
            identity: OwnedContentIdentity::Stable("custom-spell".into()),
        }],
        field: None,
        passage: None,
        source_fingerprint: None,
    }
}
fn detail(body: RecordBodyView) -> RecordDetailView {
    RecordDetailView {
        record: summary("Local Caster", "creature"),
        presentation: RecordPresentationView {
            identity: summary("Custom Fireball", "spell"),
            content: vec![],
            owned: vec![],
            body,
        },
        selected: address(),
        relationships: vec![],
        relationships_truncated: false,
    }
}
fn damage_component() -> DamageComponentView {
    DamageComponentView {
        id: "local-damage".into(),
        formula: known("2d6+1".into()),
        damage_type: known("fire".into()),
        kinds: known(vec!["damage".into()]),
        category: known("persistent".into()),
        materials: known(vec![]),
        apply_modifier: known(false),
    }
}
fn spell() -> SpellPresentationView {
    SpellPresentationView {
        rank: numeric(2),
        traditions: known(vec!["arcane".into(), "primal".into()]),
        cast: known("2 actions".into()),
        requirements: known("A free hand".into()),
        cost: missing(),
        range: known("120 feet".into()),
        target: known("one creature".into()),
        area: known(SpellAreaView {
            shape: known("burst".into()),
            size: known("20 feet".into()),
            details: missing(),
        }),
        defense: known(SpellDefenseView {
            statistic: known("reflex".into()),
            basic: known(true),
            passive: missing(),
        }),
        duration: known("1 minute".into()),
        sustained: known(false),
        damage: known(vec![damage_component()]),
        heightening: known(SpellHeighteningView::Interval {
            interval: numeric(1),
            area: Box::new(absent_number()),
            damage: known(vec![HeighteningDamageView {
                id: "local-damage".into(),
                formula: known("1d6".into()),
            }]),
        }),
        forms: known(vec![]),
        casting_entry: known("entry-local".into()),
        authored_cast_rank: numeric(4),
    }
}
#[test]
fn selected_spell_preserves_parent_casting_damage_and_authored_heightening() {
    let record = detail(RecordBodyView::Spell(Box::new(spell())));
    let rendered = render_record(&record, TerminalDetail::Standard, 80).unwrap();
    for expected in [
        "Custom Fireball — spell Rank 2",
        "From Local Caster (creatures:caster)",
        "Rarity: uncommon",
        "Traits: Fire",
        "Source: Local Bestiary",
        "Traditions: arcane, primal",
        "Casting: 2 actions",
        "Requirements: A free hand",
        "Range: 120 feet",
        "Area: 20 feet burst",
        "Save: reflex",
        "Basic save: true",
        "Sustained: false",
        "2d6+1; fire",
        "category: persistent",
        "Authored cast rank: 4",
        "Authored heightening (+1)",
        "Damage [local-damage]: 1d6",
        "--owners '[{\"collection\":\"/items\",\"identity\":{\"Stable\":\"custom-spell\"}}]'",
    ] {
        assert!(
            rendered.contains(expected),
            "missing {expected:?} in {rendered}"
        );
    }
    let summary = render_record(&record, TerminalDetail::Summary, 80).unwrap();
    assert!(!rendered.contains("apply modifier: false"));
    assert!(!rendered.contains("materials: none"));
    assert!(summary.contains("Custom Fireball"));
    assert!(summary.contains("From Local Caster"));
    assert!(!summary.contains("Casting:"));
}

#[test]
fn fixed_heightening_and_forms_remain_authored_changes_with_no_formula_execution() {
    let mut spell = spell();
    let changes = SpellChangeView {
        cast: missing(),
        range: known("60 feet".into()),
        target: missing(),
        area: missing(),
        defense: missing(),
        duration: known("sustained up to 1 minute".into()),
        sustained: known(true),
        traits: known(vec!["polymorph".into()]),
        traditions: missing(),
        damage: known(vec![]),
        heightening: missing(),
    };
    spell.heightening = known(SpellHeighteningView::Fixed {
        levels: known(vec![SpellFixedLevelView {
            rank: 5,
            changes: known(changes.clone()),
        }]),
    });
    spell.forms = known(vec![SpellFormView {
        id: "blue".into(),
        name: known("Blue dragon".into()),
        sort: numeric(10),
        changes: known(changes),
    }]);
    let output = render_record(
        &detail(RecordBodyView::Spell(Box::new(spell))),
        TerminalDetail::Standard,
        80,
    )
    .unwrap();
    assert!(output.contains("Authored heightening (rank 5)"));
    assert!(output.contains("Authored form [blue]: Blue dragon"));
    assert!(output.contains("Damage: none"));
    assert!(output.contains("2d6+1"));
}

#[test]
fn unavailable_spell_parents_remain_unknown_and_empty_damage_remains_known() {
    let mut spell = spell();
    spell.defense.state = QueryFieldState::Invalid;
    spell.area.state = QueryFieldState::Null;
    spell.damage = known(vec![]);
    spell.heightening.state = QueryFieldState::Invalid;
    let output = render_record(
        &detail(RecordBodyView::Spell(Box::new(spell))),
        TerminalDetail::Standard,
        80,
    )
    .unwrap();
    assert!(output.contains("Defense: [invalid]"));
    assert!(output.contains("Area: [null]"));
    assert!(output.contains("Damage: none"));
    assert!(output.contains("Heightening: [invalid]"));
    assert!(!output.contains("Save: reflex"));
    assert!(!output.contains("Authored heightening"));
}

fn actor() -> ActorPresentationView {
    ActorPresentationView {
        level: numeric(7),
        armor_class: numeric(25),
        maximum_hp: numeric(80),
        perception: numeric(12),
        saves: ActorSavesView {
            fortitude: numeric(0),
            reflex: numeric(-2),
            will: NumberFactView {
                state: QueryFieldState::Invalid,
                value: None,
                adjustment: None,
            },
        },
        abilities: known(vec![]),
        skills: known(vec![ActorSkillView {
            key: "athletics".into(),
            label: "Athletics".into(),
            modifier: numeric(14),
            note: known("while climbing".into()),
            conditional: known(vec![ConditionalSkillView {
                label: known("against a grabbed creature".into()),
                modifier: numeric(16),
            }]),
        }]),
        movement: known(vec![ActorMovementView {
            movement_type: known("fly".into()),
            feet: numeric(40),
        }]),
        land_speed: numeric(25),
        immunities: known(vec![]),
        weaknesses: known(vec![]),
        resistances: known(vec![IwrEntryView {
            damage_type: known("all-damage".into()),
            magnitude: numeric(10),
            exceptions: known(vec![
                "force".into(),
                "ghost-touch".into(),
                "vitality".into(),
            ]),
            double_against: known(vec!["non-magical".into()]),
        }]),
        senses: known(vec![]),
        perception_details: missing(),
        languages: known(vec!["common".into()]),
        language_details: missing(),
        activities: known(vec![ActorActivityView {
            navigation: address(),
            title: "Jaws".into(),
            family: "melee".into(),
            kind: ActorActivityKindView::Strike,
            usage: Some("1 action".into()),
            traits: vec![],
            attack: numeric(15),
            lore_modifier: absent_number(),
            difficulty_class: absent_number(),
            casting_tradition: missing(),
            preparation: missing(),
            damage: known(vec![damage_component()]),
            casting_entry: None,
            association: missing(),
            notes: vec![RuntimeEffectNoteView {
                source: "local".into(),
                label: "Context".into(),
                reason: "authored use only".into(),
            }],
        }]),
        runtime: None,
    }
}
#[test]
fn actor_modifiers_distances_qualified_iwr_and_activity_context_are_visible() {
    let rendered = render_record(
        &detail(RecordBodyView::Creature(Box::new(actor()))),
        TerminalDetail::Standard,
        80,
    )
    .unwrap();
    for expected in [
        "Perception: +12",
        "Athletics: +14",
        "against a grabbed creature: +16",
        "Fortitude: +0",
        "Reflex: -2",
        "Will: [invalid]",
        "Speed: 25 feet",
        "Speed (fly): 40 feet",
        "all-damage 10 (except force, ghost-touch, vitality) (double against non-magical)",
        "Jaws — 1 action",
        "Attack: +15",
        "Context: authored use only",
        "Immunities: none",
    ] {
        assert!(
            rendered.contains(expected),
            "missing {expected:?} in {rendered}"
        );
    }
}

#[test]
fn selected_strike_retains_attack_damage_usage_and_parent_context() {
    let activity = actor().activities.value.as_mut().unwrap().remove(0);
    let mut record = detail(RecordBodyView::Activity(Box::new(activity)));
    record.presentation.identity.title = "Jaws".into();
    record.presentation.identity.kind_label = "Strike".into();
    let rendered = render_record(&record, TerminalDetail::Standard, 80).unwrap();
    for expected in [
        "Jaws — Strike",
        "From Local Caster",
        "Usage: 1 action",
        "Attack: +15",
        "2d6+1; fire",
        "Context: authored use only",
    ] {
        assert!(
            rendered.contains(expected),
            "missing {expected:?} in {rendered}"
        );
    }
}

#[test]
fn snapshot_local_reopen_command_preserves_generation_and_exact_field() {
    let mut record = detail(RecordBodyView::Content);
    record.selected.owners[0].identity = OwnedContentIdentity::SnapshotLocal { index: 42 };
    record.selected.source_fingerprint = Some("a".repeat(64));
    record.selected.field = Some("/system/description/value".into());
    let output = render_record(&record, TerminalDetail::Summary, 80).unwrap();
    assert!(output.contains("\"SnapshotLocal\":{\"index\":42}"));
    assert!(output.contains(&format!("--source-fingerprint '{}'", "a".repeat(64))));
    assert!(output.contains("--field '/system/description/value'"));
}
#[test]
fn hazard_uses_known_zero_and_dedicated_detection_structure_facts() {
    let record = detail(RecordBodyView::Hazard(Box::new(HazardPresentationView {
        actor: actor(),
        complex: known(false),
        stealth: numeric(0),
        hardness: numeric(10),
    })));
    let output = render_record(&record, TerminalDetail::Standard, 80).unwrap();
    assert!(output.contains("Complex: false"));
    assert!(output.contains("Stealth: +0"));
    assert!(output.contains("Hardness: 10"));
}

#[test]
fn effective_values_render_service_explanations_without_reapplying_modifiers() {
    let mut actor = actor();
    actor.armor_class = numeric(27);
    actor.armor_class.adjustment = Some(NumberAdjustmentView {
        authored: 25.into(),
        applied: vec![StatModifierView {
            source: "participant".into(),
            label: "Elite".into(),
            modifier_type: StatModifierTypeView::Adjustment,
            value: 2,
        }],
        suppressed: vec![],
        notes: vec![RuntimeEffectNoteView {
            source: "source".into(),
            label: "Context".into(),
            reason: "one supported adjustment".into(),
        }],
    });
    let output = render_record(
        &detail(RecordBodyView::Creature(Box::new(actor))),
        TerminalDetail::Standard,
        80,
    )
    .unwrap();
    assert!(output.contains("AC: 27 (authored 25; Elite +2; Context: one supported adjustment)"));
    assert!(!output.contains("AC: 29"));
}

#[test]
fn cosmetic_blanks_and_inapplicable_rows_are_omitted_without_hiding_known_zero() {
    let mut actor = actor();
    actor.perception = absent_number();
    actor.senses = FactView {
        state: QueryFieldState::NotApplicable,
        value: None,
    };
    actor.languages = FactView {
        state: QueryFieldState::NotApplicable,
        value: None,
    };
    actor.perception_details = known(String::new());
    actor.language_details = known("   ".into());
    let mut record = detail(RecordBodyView::Hazard(Box::new(HazardPresentationView {
        actor,
        complex: known(false),
        stealth: numeric(0),
        hardness: numeric(0),
    })));
    for body in [
        PreparedFieldBodyView::Html {
            html: "<p> </p>".into(),
            controls: vec![],
        },
        PreparedFieldBodyView::Plain {
            text: String::new(),
        },
    ] {
        record.presentation.content.push(PreparedContentFieldView {
            locator: atlas_record::source_content::SourceContentLocator {
                record: atlas_domain::RecordKey::parse("creatures:caster").unwrap(),
                owners: vec![],
                field: "/system/details/notes".into(),
            },
            role: "notes".into(),
            source_fingerprint: None,
            body,
        });
    }
    let output = render_record(&record, TerminalDetail::Standard, 80).unwrap();
    for unwanted in [
        "[not_applicable]",
        "Perception details:",
        "Language details:",
        "\nNotes\n",
    ] {
        assert!(!output.contains(unwanted), "{unwanted}: {output}");
    }
    for expected in [
        "Fortitude: +0",
        "Stealth: +0",
        "Hardness: 0",
        "Complex: false",
    ] {
        assert!(output.contains(expected), "{expected}: {output}");
    }
    let mut spell = spell();
    spell.requirements = known(String::new());
    spell.cost = known(String::new());
    spell.target = known(String::new());
    spell.duration = known(String::new());
    spell.damage.value.as_mut().unwrap()[0].category = cleared();
    let output = render_record(
        &detail(RecordBodyView::Spell(Box::new(spell))),
        TerminalDetail::Standard,
        80,
    )
    .unwrap();
    for unwanted in [
        "Requirements:",
        "Cost:",
        "Target:",
        "Duration:",
        "category:",
        "apply modifier: false",
        "materials: none",
    ] {
        assert!(!output.contains(unwanted), "{unwanted}: {output}");
    }
    assert!(output.contains("Casting: 2 actions"));
}

#[test]
fn sparse_forms_show_authored_changes_explicit_clears_and_heightening_formula() {
    let mut spell = spell();
    let mut changes = no_changes();
    changes.cast = known("2 actions".into());
    changes.defense = cleared();
    changes.damage = known(vec![DamageComponentView {
        id: "0".into(),
        formula: missing(),
        damage_type: missing(),
        kinds: known(vec!["damage".into()]),
        category: missing(),
        materials: missing(),
        apply_modifier: missing(),
    }]);
    changes.heightening = known(SpellHeighteningChangeView {
        kind: missing(),
        interval: absent_number(),
        area: absent_number(),
        damage: known(vec![HeighteningDamageView {
            id: "0".into(),
            formula: known("1d8+8".into()),
        }]),
        levels: missing(),
    });
    spell.forms = known(vec![SpellFormView {
        id: "two-actions".into(),
        name: missing(),
        sort: numeric(1),
        changes: known(changes),
    }]);
    let output = render_record(
        &detail(RecordBodyView::Spell(Box::new(spell))),
        TerminalDetail::Standard,
        80,
    )
    .unwrap();
    for expected in [
        "Authored form [two-actions]: 2 actions",
        "Authored changes to base",
        "Defense: cleared (null)",
        "Damage component changes [0]: kinds: damage",
        "Damage increment [0]: 1d8+8",
    ] {
        assert!(output.contains(expected), "{expected}: {output}");
    }
    assert!(!output.contains("[missing]"));
    assert!(!output.contains("Damage [0]:"));
}

#[test]
fn unavailable_collections_entries_and_lore_use_final_typed_contract() {
    let mut actor = actor();
    actor.skills.state = QueryFieldState::Invalid;
    actor.abilities = missing();
    actor.movement = FactView {
        state: QueryFieldState::Null,
        value: None,
    };
    let mut entry = actor.activities.value.as_ref().unwrap()[0].clone();
    entry.title = "Arcane Prepared Spells".into();
    entry.kind = ActorActivityKindView::CastingEntry;
    entry.attack = numeric(23);
    entry.difficulty_class = numeric(33);
    entry.casting_tradition = known("arcane".into());
    entry.preparation = known("prepared".into());
    entry.damage = FactView {
        state: QueryFieldState::NotApplicable,
        value: None,
    };
    let mut lore = entry.clone();
    lore.title = "River Lore".into();
    lore.kind = ActorActivityKindView::Lore;
    lore.attack = absent_number();
    lore.difficulty_class = absent_number();
    lore.casting_tradition = missing();
    lore.preparation = missing();
    lore.lore_modifier = numeric(10);
    lore.lore_modifier.adjustment = Some(NumberAdjustmentView {
        authored: 8.into(),
        applied: vec![StatModifierView {
            source: "participant".into(),
            label: "Elite".into(),
            modifier_type: StatModifierTypeView::Adjustment,
            value: 2,
        }],
        suppressed: vec![],
        notes: vec![],
    });
    actor.activities = known(vec![entry.clone(), lore]);
    let output = render_record(
        &detail(RecordBodyView::Creature(Box::new(actor))),
        TerminalDetail::Standard,
        80,
    )
    .unwrap();
    for expected in [
        "Skills: [invalid]",
        "Ability modifiers: [missing]",
        "Other speeds: [null]",
        "Spell attack: +23",
        "DC: 33",
        "Tradition: arcane",
        "Preparation: prepared",
        "Lore modifier: +10 (authored 8; Elite +2)",
    ] {
        assert!(output.contains(expected), "{expected}: {output}");
    }
    assert!(!output.contains("Athletics"));
    let output = render_record(
        &detail(RecordBodyView::Activity(Box::new(entry))),
        TerminalDetail::Standard,
        80,
    )
    .unwrap();
    assert!(output.contains("Spell attack: +23"));
    assert!(output.contains("DC: 33"));
}
