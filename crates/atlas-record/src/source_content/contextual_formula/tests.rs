use super::*;
use crate::source_content::*;

#[test]
fn symbolic_damage_preserves_compound_halving_and_rounding_operands() {
    let context = FormulaContext::default();
    for (formula, expected) in [
        (
            "(floor((@actor.level -1)/2)+1)d6[fire]",
            "a number of d6s of fire damage equal to (half (the creature’s level − 1)) (rounded down) + 1",
        ),
        (
            "floor(1 + @actor.level/2)d6[void]",
            "a number of d6s of void damage equal to (1 + (half the creature’s level)) (rounded down)",
        ),
        (
            "ceil(@actor.level/3 + @item.rank/2)d6[fire]",
            "a number of d6s of fire damage equal to ((the creature’s level ÷ 3) + (half spell rank)) (rounded up)",
        ),
    ] {
        assert_eq!(
            contextual_damage_text(formula, &context, false)
                .unwrap()
                .text,
            expected,
            "{formula}"
        );
    }
}

#[test]
fn known_values_resolve_only_the_bounded_arithmetic_subset() {
    let context = FormulaContext {
        actor_level: Some(7),
        item_rank: Some(5),
        item_is_spell: true,
        badge_value: Some(2),
        ..Default::default()
    };
    for (source, expected) in [
        ("(ceil(@actor.level / 2))d6[untyped]", "4d6"),
        ("(max(8,@item.rank*2))d6[mental]", "10d6"),
        ("(@item.level)d6[fire]", "5d6"),
        ("resolve(5+@item.badge.value)[fire]", "7"),
    ] {
        assert_eq!(
            resolved_contextual_damage(source, &context).unwrap()[0].formula,
            expected
        );
    }
    assert_eq!(
        resolved_contextual_roll("br", "ceil(@actor.level/2)d6 #rounds", &context)
            .unwrap()
            .formula,
        "4d6"
    );
    for invalid in [
        "1/0",
        "max()",
        "Math.ceil(3)",
        "@actor.unknown+1",
        "@actor.level;evil()",
        "1e999",
        "1+",
        &"(".repeat(40),
        &"1+".repeat(300),
    ] {
        assert!(
            resolve_contextual_formula(invalid, &context).is_none(),
            "{invalid}"
        );
    }
    for unsafe_pool in [
        "(201+@actor.level)d6[fire]",
        "(1d6-@actor.level)[fire]",
        "@item.rank[unknown]",
        "@actor.missing[fire]",
        "1d6[fire],@item.rank[unknown]",
    ] {
        assert!(
            resolved_contextual_damage(unsafe_pool, &context).is_none(),
            "{unsafe_pool}"
        );
    }
}

#[test]
fn missing_values_become_rules_without_guessing_defaults() {
    let context = FormulaContext {
        item_is_spell: true,
        condition_name: Some("Stupefied".into()),
        ..Default::default()
    };
    assert_eq!(
        contextual_damage_text("(ceil(@actor.level / 2))d6[untyped]", &context, false)
            .unwrap()
            .text,
        "a number of d6s of damage equal to (half the creature’s level) (rounded up)"
    );
    assert_eq!(
        contextual_damage_text("(max(8,@item.rank*2))d6[mental]", &context, false)
            .unwrap()
            .text,
        "2d6 mental damage per spell rank, with a minimum of 8d6"
    );
    assert_eq!(
        describe_contextual_formula("resolve(5+@item.badge.value)", &context).unwrap(),
        "5 + your stupefied value"
    );
    assert!(resolved_contextual_damage("@item.badge.value[fire]", &context).is_none());
    assert!(contextual_damage_text("@actor.flags.foo[fire]", &context, false).is_none());
}

#[test]
fn contextual_rendering_preserves_authored_labels_references_and_source_options() {
    let context = FormulaContext {
        condition_name: Some("Stupefied".into()),
        ..Default::default()
    };
    let prepared=prepare_source_content(SourceContentLocator{record:atlas_domain::RecordKey::parse("conditions:example").unwrap(),owners:vec![],field:"/description".into()},
        "<p>@Check[flat|showDC:all|dc:resolve(5+@item.badge.value)] with a DC equal to 5 + your stupefied value. @Damage[(ceil(@actor.level/2))d6[untyped]] damage. @Damage[@item.rank[fire]]{<strong>Authored @UUID[missing]{link}</strong>}</p>",
        ContentAudience{include_gm:true,include_owner:true,implicit_check_dc:ContentVisibilityRule::All},ContentVisibilityRule::All,None,None,&context).unwrap();
    assert_eq!(prepared.text.matches("your stupefied value").count(), 1);
    assert!(!prepared.text.contains("damage damage"));
    let rendered = render_contextual_interactions(
        &prepared.html,
        &prepared.interactions,
        &FormulaContext {
            actor_level: Some(7),
            badge_value: Some(2),
            item_rank: Some(5),
            ..context
        },
    );
    assert!(rendered.contains("DC 7 flat check"));
    assert!(!rendered.contains("your stupefied value"));
    assert!(rendered.contains(">4d6</span> damage"));
    assert!(rendered.contains("<strong>Authored"));
    assert!(rendered.contains("data-atlas-reference=\"0\""));
    assert!(
        matches!(&prepared.interactions[0].kind,ContentInteractionKind::Check{options,..} if options.get("dc").unwrap()=="resolve(5+@item.badge.value)")
    );
}

#[test]
fn literal_fallbacks_keep_existing_damage_semantics_and_short_label_nouns() {
    let context = FormulaContext {
        actor_level: Some(7),
        ..Default::default()
    };
    assert!(contextual_damage_text("(@actor.level-10)[fire]", &context, false).is_none());
    assert!(contextual_damage_text("1d6[fire],@actor.missing[cold]", &context, false).is_none());
    assert_eq!(
        contextual_damage_text("(4d8+10)[bludgeoning]", &context, false)
            .unwrap()
            .text,
        "4d8+10 bludgeoning damage"
    );
}

#[test]
fn absorbing_a_duplicate_never_removes_visible_authored_prose_when_the_macro_hides_its_dc() {
    let prepared=prepare_source_content(SourceContentLocator{record:atlas_domain::RecordKey::parse("conditions:example").unwrap(),owners:vec![],field:"/description".into()},
        "@Check[flat|showDC:gm|dc:resolve(5+@item.badge.value)] with a DC equal to 5 + your stupefied value.",
        ContentAudience{include_gm:false,include_owner:false,implicit_check_dc:ContentVisibilityRule::None},ContentVisibilityRule::All,None,None,&FormulaContext{condition_name:Some("Stupefied".into()),..Default::default()}).unwrap();
    assert_eq!(
        prepared.text,
        "flat check with a DC equal to 5 + your stupefied value."
    );
    assert!(
        matches!(&prepared.interactions[0].kind,ContentInteractionKind::Check{options,..} if !options.contains_key("dc"))
    );
}

#[test]
fn healing_uses_the_authored_hit_point_unit_without_losing_typed_or_mixed_labels() {
    let prepared=prepare_source_content(SourceContentLocator{record:atlas_domain::RecordKey::parse("actors:healing").unwrap(),owners:vec![],field:"/description".into()},
        "<p>Regains @Damage[(ceil(@actor.level/2))d6[healing]] Hit Points. @Damage[1d6[healing]] Hit Points. @Damage[1d6[vitality,healing]] Hit Points. @Damage[1d6[healing],1d6[fire]] Hit Points. @Damage[1d6[healing]]{Mending} Hit Points.</p>",
        ContentAudience{include_gm:true,include_owner:true,implicit_check_dc:ContentVisibilityRule::All},ContentVisibilityRule::All,None,None,&FormulaContext::default()).unwrap();
    assert!(
        prepared
            .text
            .contains("((half the creature’s level) (rounded up))d6 Hit Points")
    );
    let rendered = render_contextual_interactions(
        &prepared.html,
        &prepared.interactions,
        &FormulaContext {
            actor_level: Some(5),
            ..Default::default()
        },
    );
    let text = prepared_plain_text(&rendered).unwrap();
    assert!(text.contains("Regains 3d6 Hit Points. 1d6 Hit Points."));
    assert!(text.contains("1d6 vitality healing Hit Points"));
    assert!(text.contains("1d6 healing + 1d6 fire damage Hit Points"));
    assert!(text.contains("Mending Hit Points"));
    assert_eq!(
        resolved_contextual_damage(
            "(ceil(@actor.level/2))d6[healing]",
            &FormulaContext {
                actor_level: Some(5),
                ..Default::default()
            }
        )
        .unwrap()[0]
            .label,
        "healing"
    );
}
