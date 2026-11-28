//! Main Search chooses quick controls from the committed Kind scope and catalog.
use atlas_app_model::{
    FilterEditorFieldView, FilterEditorView, SearchFilterToolbarView, SearchQuickFieldView,
};
use atlas_domain::{
    QueryCompare, QueryExpression, QueryFieldType, QueryLiteral, QueryPredicate,
    root_kind::{self, ROOT_KINDS},
};
use std::collections::{BTreeMap, BTreeSet};

const PRICE: &str = "physical.unit_price_gp";
const BULK: &str = "physical.bulk";
const RANK: &str = "spell.rank";
const LEVEL: &str = "level";
const LEVEL_OR_RANK: &str = "level_or_rank";
const RARITY: &str = "rarity";
const TRAITS: &str = "traits";
const TRADITION: &str = "spell.traditions";
const CAST_ACTIONS: &str = "spell.casting_form";
const CAST_TIME: &str = "spell.casting_time";
const MOVEMENT: &str = "movement.speeds";

fn specialists(kind: &str) -> &'static [&'static str] {
    match kind {
        "action" => &["action.type", "action.category"],
        "ammunition" | "container" => &[PRICE, BULK],
        "ancestry" => &["ancestry.default_boosts", "ancestry.vision"],
        "ancestry_feat" | "bonus_feat" | "class_feat" | "skill_feat" => &["action.type"],
        "armor" => &["armor.category", "armor.ac_bonus", PRICE],
        "army_war_action" => &["action.count"],
        "background" => &[
            "background.restricted_attribute_boosts",
            "background.skill_training",
        ],
        "character" => &["languages"],
        "class" => &["class.key_attributes"],
        "consumable" => &["consumable.category", PRICE, "physical.usage"],
        "creature" => &[MOVEMENT],
        "deity" => &["deity.domains.primary", "deity.favored_weapons"],
        "effect" => &["effect.duration.unit"],
        "general_equipment" => &[PRICE, "physical.usage", BULK],
        "hazard" => &["hazard.complexity", "hazard.disable_skills"],
        "heritage" => &["heritage.ancestry.slug", "heritage.versatile"],
        "ritual" => &[CAST_TIME],
        "shield" => &["shield.hardness", "shield.broken_threshold", PRICE],
        "spell" => &[TRADITION, CAST_ACTIONS],
        "treasure" => &[PRICE],
        "vehicle" => &["body.size"],
        "weapon" => &[
            "weapon.category",
            "weapon.damage.type",
            "weapon.group",
            PRICE,
        ],
        _ => &[],
    }
}

fn common_recommended(kind: &str, field: &str) -> bool {
    match field {
        RARITY => !matches!(
            kind,
            "ancestry_feature" | "bonus_feat" | "deity_curse" | "treasure"
        ),
        TRAITS => !matches!(
            kind,
            "army"
                | "background"
                | "condition"
                | "deity_curse"
                | "effect"
                | "kit"
                | "kingdom_feature"
                | "society_boon"
                | "treasure"
        ),
        _ => false,
    }
}

fn quick_view(field: &FilterEditorFieldView) -> SearchQuickFieldView {
    let label = match field.definition.id.as_str() {
        PRICE => "Unit price (gp)",
        RANK => "Rank",
        TRADITION => "Tradition",
        CAST_ACTIONS => "Cast actions",
        CAST_TIME => "Casting time",
        MOVEMENT => "Movement type",
        "background.restricted_attribute_boosts" => "Restricted boost",
        "background.skill_training" => "Skill training",
        "ancestry.default_boosts" => "Default boosts",
        "ancestry.vision" => "Vision",
        "deity.domains.primary" => "Primary domain",
        "weapon.damage.type" => "Damage type",
        _ => &field.label,
    };
    SearchQuickFieldView {
        field_id: field.definition.id.clone(),
        label: label.into(),
    }
}

fn family(kind: &str) -> Option<&'static str> {
    ROOT_KINDS
        .iter()
        .find(|choice| choice.id == kind)?
        .source_family
}

fn covers(field: &FilterEditorFieldView, kind: &str) -> bool {
    let usable = match field.definition.id.as_str() {
        TRADITION | CAST_ACTIONS => kind == "spell",
        CAST_TIME => kind == "ritual",
        LEVEL => !matches!(kind, "army_war_action" | "kingdom_activity"),
        _ => true,
    };
    usable
        && field.definition.scope.is_none()
        && (field.definition.family_types.is_empty()
            || family(kind).is_some_and(|family| {
                field
                    .definition
                    .family_types
                    .iter()
                    .any(|allowed| allowed == family)
            }))
}

fn mentions_root_kind(predicate: &QueryPredicate) -> bool {
    match &predicate.expression {
        QueryExpression::AllOf { children } | QueryExpression::AnyOf { children } => {
            children.iter().any(mentions_root_kind)
        }
        QueryExpression::Not { predicate } => mentions_root_kind(predicate),
        QueryExpression::Compare { field, .. }
        | QueryExpression::In { field, .. }
        | QueryExpression::SetMatch { field, .. }
        | QueryExpression::StateMatch { field, .. } => field == "kind",
        _ => false,
    }
}

fn conjuncts<'a>(predicate: &'a QueryPredicate, leaves: &mut Vec<&'a QueryPredicate>) {
    if let QueryExpression::AllOf { children } = &predicate.expression {
        for child in children {
            conjuncts(child, leaves);
        }
    } else {
        leaves.push(predicate);
    }
}

fn positive_kind_values(predicate: &QueryPredicate) -> Option<Vec<&str>> {
    match &predicate.expression {
        QueryExpression::Compare {
            field,
            op: QueryCompare::Eq,
            value: QueryLiteral::String(value),
        } if field == "kind" => Some(vec![value]),
        QueryExpression::In { field, values } if field == "kind" => values
            .iter()
            .map(|value| match value {
                QueryLiteral::String(value) => Some(value.as_str()),
                _ => None,
            })
            .collect(),
        _ => None,
    }
}

fn selector_includes(selector: &str, atom: &str) -> bool {
    selector == atom || root_kind::parent(atom) == Some(selector)
}

struct KindScope<'a> {
    atoms: Vec<&'a str>,
    named_parent: Option<&'a str>,
    kind_filter: QueryPredicate,
    has_refinements: bool,
}

fn kind_scope<'a>(filter: &'a QueryPredicate, journal_packs: &[&'a str]) -> Option<KindScope<'a>> {
    let mut leaves = Vec::new();
    conjuncts(filter, &mut leaves);
    let mut include = None;
    let mut exclude = Vec::new();
    let mut kind_leaves = Vec::new();
    let mut has_refinements = false;
    for leaf in leaves {
        if !mentions_root_kind(leaf) {
            has_refinements = true;
            continue;
        }
        if let Some(values) = positive_kind_values(leaf) {
            if include.replace(values).is_some() {
                return None;
            }
        } else {
            let values = match &leaf.expression {
                QueryExpression::Not { predicate } => positive_kind_values(predicate)?,
                QueryExpression::Compare {
                    field,
                    op: QueryCompare::Neq,
                    value: QueryLiteral::String(value),
                } if field == "kind" => vec![value.as_str()],
                _ => return None,
            };
            exclude.extend(values);
        }
        kind_leaves.push(leaf.clone());
    }

    let include = include?;
    let has_all_journals = include.contains(&"journal");
    let atoms: Vec<&str> = ROOT_KINDS
        .iter()
        .filter(|choice| {
            choice.source_family.is_some()
                || choice.id == "roll_table"
                || (choice.id == "journal" && journal_packs.is_empty())
        })
        .map(|choice| choice.id)
        .chain(
            include
                .iter()
                .copied()
                .filter(|id| !has_all_journals && root_kind::journal_pack(id).is_some()),
        )
        .chain(journal_packs.iter().copied().filter(|_| has_all_journals))
        .filter(|atom| include.iter().any(|id| selector_includes(id, atom)))
        .filter(|atom| {
            !exclude.iter().any(|id| {
                selector_includes(id, atom)
                    || (*atom == "kingdom_and_warfare"
                        && root_kind::is_campaign_feature_selector(id))
            })
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    // An explicitly named parent can recommend a control that serves most of
    // its content. Independently selected kinds always require full coverage.
    let named_parent = include.into_iter().find(|id| {
        matches!(
            *id,
            "all_equipment" | "all_spells" | "character_advancement" | "kingdom_and_warfare"
        ) && atoms.len() > 1
            && atoms.iter().all(|atom| selector_includes(id, atom))
    });
    let kind_filter = if !has_refinements {
        filter.clone()
    } else if kind_leaves.len() == 1 {
        kind_leaves.pop()?
    } else {
        QueryPredicate::new(QueryExpression::AllOf {
            children: kind_leaves,
        })
    };
    Some(KindScope {
        atoms,
        named_parent,
        kind_filter,
        has_refinements,
    })
}

fn quick_specialists(
    scope: &KindScope<'_>,
    fields: &BTreeMap<&str, &FilterEditorFieldView>,
) -> Vec<&'static str> {
    let named_fields: &[&str] = match scope.named_parent {
        Some("all_equipment") => &[PRICE, BULK],
        Some("all_spells") => &[TRADITION, CAST_ACTIONS],
        _ => &[],
    };
    if !named_fields.is_empty() {
        return named_fields
            .iter()
            .copied()
            .filter(|id| {
                fields
                    .get(id)
                    .is_some_and(|field| scope.atoms.iter().any(|kind| covers(field, kind)))
            })
            .collect();
    }

    // Preserve shortlists for a single kind. For a mixed equipment scope,
    // also nominate shared physical controls before applying the coverage gate.
    let mut scores = BTreeMap::<&str, (usize, usize)>::new();
    for kind in &scope.atoms {
        let primary = specialists(kind);
        let shared = [PRICE, BULK].into_iter().filter(|id| {
            scope.atoms.len() > 1
                && !primary.contains(id)
                && fields.get(id).is_some_and(|field| covers(field, kind))
        });
        for (priority, id) in primary.iter().copied().chain(shared).enumerate() {
            if fields
                .get(id)
                .is_some_and(|field| scope.atoms.iter().all(|kind| covers(field, kind)))
            {
                let (mentions, best) = scores.entry(id).or_insert((0, priority));
                *mentions += 1;
                *best = (*best).min(priority);
            }
        }
    }

    let mut ranked: Vec<_> = scores.into_iter().collect();
    ranked.sort_by(|(a, (count_a, priority_a)), (b, (count_b, priority_b))| {
        count_b
            .cmp(count_a)
            .then(priority_a.cmp(priority_b))
            .then((*a != PRICE).cmp(&(*b != PRICE)))
            .then(a.cmp(b))
    });
    let limit = match scope.atoms.as_slice() {
        ["weapon"] => 4,
        ["armor" | "consumable" | "general_equipment" | "shield"] => 3,
        _ => 2,
    };
    ranked.into_iter().take(limit).map(|(id, _)| id).collect()
}

fn active_field(predicate: &QueryPredicate, id: &str) -> bool {
    match &predicate.expression {
        QueryExpression::AllOf { children } => children.iter().any(|child| active_field(child, id)),
        QueryExpression::Not { predicate } => active_field(predicate, id),
        QueryExpression::Compare { field, .. }
        | QueryExpression::In { field, .. }
        | QueryExpression::SetMatch { field, .. } => field == id,
        QueryExpression::Exists { collection, .. } => collection == id,
        _ => false,
    }
}

pub(crate) fn resolve(
    editor: &FilterEditorView,
    filter: Option<&QueryPredicate>,
) -> SearchFilterToolbarView {
    let fields: BTreeMap<_, _> = editor
        .groups
        .iter()
        .flat_map(|group| &group.fields)
        .filter(|field| {
            field.definition.scope.is_none()
                && (field.definition.field_type != QueryFieldType::Collection
                    || (field.definition.id == MOVEMENT && field.collection_control.is_some()))
        })
        .map(|field| (field.definition.id.as_str(), field))
        .collect();

    let journal_packs: Vec<_> = fields
        .get("kind")
        .into_iter()
        .flat_map(|field| &field.choices)
        .filter_map(|choice| match &choice.value {
            QueryLiteral::String(id) if root_kind::journal_pack(id).is_some() => Some(id.as_str()),
            _ => None,
        })
        .collect();
    let scope = filter
        .and_then(|filter| kind_scope(filter, &journal_packs))
        .filter(|scope| !scope.atoms.is_empty());
    let toolbar_ids: Vec<_> = if let Some(scope) = scope.as_ref() {
        let can_show = |id| {
            fields
                .get(id)
                .is_some_and(|field| scope.atoms.iter().any(|kind| covers(field, kind)))
        };
        let numeric = match (can_show(LEVEL), can_show(RANK)) {
            (true, true) => Some(LEVEL_OR_RANK),
            (true, false) => Some(LEVEL),
            (false, true) => Some(RANK),
            (false, false) => None,
        };
        ["kind"]
            .into_iter()
            .chain(numeric)
            .chain(quick_specialists(scope, &fields))
            .chain([RARITY, TRAITS].into_iter().filter(|id| {
                scope.named_parent == Some("all_equipment") && can_show(id)
                    || fields.get(id).is_some_and(|field| {
                        let parent_recommends = scope.named_parent == Some("character_advancement")
                            || (scope.named_parent == Some("kingdom_and_warfare") && *id == TRAITS);
                        scope.atoms.iter().all(|kind| {
                            covers(field, kind)
                                && (parent_recommends || common_recommended(kind, id))
                        })
                    })
            }))
            .collect()
    } else {
        editor.common_field_ids.iter().map(String::as_str).collect()
    };

    let toolbar_fields = toolbar_ids
        .iter()
        .filter_map(|id| fields.get(id).map(|field| quick_view(field)))
        .collect();
    let retained_fields = [LEVEL, RANK, LEVEL_OR_RANK, RARITY, TRAITS, PRICE, BULK]
        .into_iter()
        .chain(
            ROOT_KINDS
                .iter()
                .flat_map(|choice| specialists(choice.id).iter().copied()),
        )
        .collect::<BTreeSet<_>>()
        .into_iter()
        .filter(|id| {
            !toolbar_ids.contains(id) && filter.is_some_and(|filter| active_field(filter, id))
        })
        .filter_map(|id| fields.get(id).map(|field| quick_view(field)))
        .collect();
    let has_refinements = scope.as_ref().is_some_and(|scope| scope.has_refinements);
    SearchFilterToolbarView {
        fields: toolbar_fields,
        retained_fields,
        kind_scope: scope.map(|scope| scope.kind_filter),
        has_refinements,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    fn view(filter: Value) -> SearchFilterToolbarView {
        let filter = serde_json::from_value(filter).unwrap();
        let editor = crate::discovery::filter_editor_view(
            atlas_search::discover_filter_fields().unwrap(),
            &[],
            "fixture".into(),
            vec![],
        )
        .unwrap();
        resolve(&editor, Some(&filter))
    }

    fn kind(values: &[&str]) -> Value {
        json!({"kind":"in","field":"kind","values":values})
    }

    fn ids(toolbar: &SearchFilterToolbarView) -> Vec<&str> {
        toolbar
            .fields
            .iter()
            .map(|field| field.field_id.as_str())
            .collect()
    }

    #[test]
    fn approved_shortlists_and_named_parents() {
        for (selection, expected) in [
            (
                &["background"][..],
                vec![
                    "kind",
                    "background.restricted_attribute_boosts",
                    "background.skill_training",
                    RARITY,
                ],
            ),
            (
                &["weapon"],
                vec![
                    "kind",
                    LEVEL,
                    "weapon.category",
                    "weapon.damage.type",
                    "weapon.group",
                    PRICE,
                    RARITY,
                    TRAITS,
                ],
            ),
            (
                &["armor", "weapon"],
                vec!["kind", LEVEL, PRICE, BULK, RARITY, TRAITS],
            ),
            (&["weapon", "armor", "kit"], vec!["kind", LEVEL]),
            (
                &["all_equipment", "armor"],
                vec!["kind", LEVEL, PRICE, BULK, RARITY, TRAITS],
            ),
            (&["all_equipment", "background"], vec!["kind", LEVEL]),
            (
                &["all_spells"],
                vec!["kind", RANK, TRADITION, CAST_ACTIONS, RARITY, TRAITS],
            ),
            (&["spell", "ritual"], vec!["kind", RANK, RARITY, TRAITS]),
            (&["ritual"], vec!["kind", RANK, CAST_TIME, RARITY, TRAITS]),
            (&["spell", "background"], vec!["kind", RANK, RARITY]),
            (&["character_creation"], vec!["kind", RARITY]),
            (
                &["character_advancement"],
                vec!["kind", LEVEL, "action.type", RARITY, TRAITS],
            ),
            (&["kingdom_and_warfare"], vec!["kind", LEVEL, TRAITS]),
            (&["journal_pack/journals"], vec!["kind"]),
            (&["creature"], vec!["kind", LEVEL, MOVEMENT, RARITY, TRAITS]),
            (&["condition"], vec!["kind"]),
        ] {
            assert_eq!(ids(&view(kind(selection))), expected, "{selection:?}");
        }
    }

    #[test]
    fn retains_exact_numeric_criteria_and_returns_only_kind_for_clear() {
        let selection = kind(&["spell", "general_feat"]);
        let toolbar = view(json!({"kind":"all_of","children":[selection,
            {"kind":"compare","field":"spell.rank","op":"eq","value":2}]}));
        assert!(ids(&toolbar).contains(&LEVEL_OR_RANK));
        assert_eq!(toolbar.retained_fields[0].field_id, RANK);
        assert_eq!(toolbar.retained_fields[0].label, "Rank");
        assert!(toolbar.has_refinements);
        assert_eq!(serde_json::to_value(toolbar.kind_scope).unwrap(), selection);
    }

    #[test]
    fn distinguishes_calendar_activities_and_unknown_group_only_roots() {
        assert_eq!(
            ids(&view(kind(&["kingdom_activity"]))),
            vec!["kind", TRAITS]
        );
        let without = |id| {
            json!({"kind":"all_of","children":[
            kind(&["kingdom_and_warfare"]),{"kind":"not","predicate":kind(&[id])}]})
        };
        let army_excluded: QueryPredicate = serde_json::from_value(without("army")).unwrap();
        let tactic_excluded: QueryPredicate =
            serde_json::from_value(without("army_tactic")).unwrap();
        assert!(
            kind_scope(&army_excluded, &[])
                .unwrap()
                .atoms
                .contains(&"kingdom_and_warfare")
        );
        assert!(
            !kind_scope(&tactic_excluded, &[])
                .unwrap()
                .atoms
                .contains(&"kingdom_and_warfare")
        );
    }

    #[test]
    fn only_ordinary_root_kind_conjunctions_adapt() {
        let toolbar =
            view(json!({"kind":"any_of","children":[kind(&["spell"]),kind(&["background"])]}));
        assert_eq!(ids(&toolbar), vec!["kind", LEVEL_OR_RANK, RARITY, TRAITS]);
        assert!(toolbar.kind_scope.is_none());

        let exclusion = json!({"kind":"all_of","children":[kind(&["all_spells"]),
            {"kind":"not","predicate":kind(&["spell"])}]});
        let toolbar = view(exclusion.clone());
        assert_eq!(ids(&toolbar), vec!["kind", RANK, CAST_TIME, RARITY, TRAITS]);
        assert_eq!(serde_json::to_value(toolbar.kind_scope).unwrap(), exclusion);
        assert!(!toolbar.has_refinements);
        let empty = view(json!({"kind":"all_of","children":[kind(&["spell"]),
            {"kind":"not","predicate":kind(&["all_spells"])}]}));
        assert!(empty.kind_scope.is_none());
        assert_eq!(ids(&empty), vec!["kind", LEVEL_OR_RANK, RARITY, TRAITS]);
    }

    #[test]
    fn all_journal_packs_can_be_excluded_without_counting_search_results() {
        let filter = serde_json::from_value(json!({"kind":"all_of","children":[
            kind(&["all_equipment", "journal"]),
            {"kind":"not","predicate":kind(&["journal_pack/journals", "journal_pack/criticaldeck"])}
        ]}))
        .unwrap();
        let toolbar = kind_scope(
            &filter,
            &["journal_pack/journals", "journal_pack/criticaldeck"],
        )
        .unwrap();
        assert_eq!(toolbar.named_parent, Some("all_equipment"));
    }

    #[test]
    fn every_static_selector_has_a_valid_toolbar() {
        for choice in ROOT_KINDS {
            let toolbar = view(kind(&[choice.id]));
            let fields = ids(&toolbar);
            assert_eq!(fields.first(), Some(&"kind"), "{}", choice.id);
            assert_eq!(BTreeSet::from_iter(fields.iter()).len(), fields.len());
        }
    }
}
