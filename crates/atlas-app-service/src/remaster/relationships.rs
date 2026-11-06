//! Consolidated product relationships; authored declarations remain the evidence owner.
use atlas_app_model::{
    RecordNavigationView, RemasterChangeKindView as Kind, RemasterDeclarationView,
    RemasterEndpointResolutionView as Resolution, RemasterInlinePartView as Part,
    RemasterRelationshipView, RemasterSideView,
};
use atlas_domain::OwnedContentLocator;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone)]
pub(super) struct Scope {
    pub namespace: String,
    pub family: Option<String>,
    pub qualifier: Option<String>,
    pub atlas: bool,
    pub body_truncated: bool,
}
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Address {
    key: String,
    owners: Vec<OwnedContentLocator>,
}
impl From<&RecordNavigationView> for Address {
    fn from(value: &RecordNavigationView) -> Self {
        Self {
            key: value.record_key.clone(),
            owners: value.owners.clone(),
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum Direction {
    Incoming,
    Outgoing,
    Same,
    Term,
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum Meaning {
    Succession,
    Merger,
    Removed,
    Mechanics,
    Discussion,
    Other,
}
#[derive(PartialEq, Eq)]
struct GroupKey {
    namespace: String,
    family: Option<String>,
    qualifier: Option<String>,
    anchor: Option<Address>,
    counterpart: Option<Address>,
    direction: Direction,
    meaning: Meaning,
    isolation: Option<String>,
}
struct Group {
    key: GroupKey,
    indices: Vec<usize>,
    generic: bool,
}
#[derive(Clone, PartialEq, Eq)]
enum OperandKey {
    Current(Address),
    Absent(String),
    Name(String),
}
#[derive(Clone)]
struct Operand {
    key: OperandKey,
    label: String,
    navigation: Option<RecordNavigationView>,
    source: Option<String>,
}

fn meaning(value: &RemasterDeclarationView) -> Meaning {
    let kinds = &value.kinds;
    if kinds.contains(&Kind::Unclassified) {
        Meaning::Other
    } else if kinds.contains(&Kind::Discussion) {
        Meaning::Discussion
    } else if kinds.contains(&Kind::Removed) {
        Meaning::Removed
    } else if kinds.contains(&Kind::Merged) {
        Meaning::Merger
    } else if kinds.contains(&Kind::Renamed) || kinds.contains(&Kind::Replaced) {
        Meaning::Succession
    } else if kinds.contains(&Kind::MechanicsChanged) {
        Meaning::Mechanics
    } else {
        Meaning::Other
    }
}
fn matches(sides: &[RemasterSideView], selected: &RecordNavigationView) -> bool {
    sides
        .iter()
        .flat_map(|side| &side.endpoints)
        .any(|endpoint| match &endpoint.resolution {
            Resolution::Current { navigation, .. } => {
                Address::from(navigation) == Address::from(selected)
            }
            Resolution::Absent { record_key, .. } => {
                record_key == &selected.record_key && selected.owners.is_empty()
            }
            _ => false,
        })
}
fn single_current(sides: &[RemasterSideView]) -> Option<(&RecordNavigationView, Option<String>)> {
    if sides.len() != 1 || sides[0].endpoints.len() != 1 {
        return None;
    }
    match &sides[0].endpoints[0].resolution {
        Resolution::Current { record, navigation } => {
            Some((navigation, record.source_type.clone()))
        }
        _ => None,
    }
}
fn selected_family(
    value: &RemasterDeclarationView,
    selected: &RecordNavigationView,
) -> Option<String> {
    value
        .before
        .iter()
        .chain(&value.after)
        .flat_map(|side| &side.endpoints)
        .find_map(|endpoint| match &endpoint.resolution {
            Resolution::Current { record, navigation }
                if Address::from(navigation) == Address::from(selected) =>
            {
                record.source_type.clone()
            }
            _ => None,
        })
}
fn has_ambiguity(value: &RemasterDeclarationView) -> bool {
    value
        .before
        .iter()
        .chain(&value.after)
        .flat_map(|side| &side.endpoints)
        .any(|endpoint| matches!(endpoint.resolution, Resolution::Ambiguous { .. }))
}
fn grouped(sides: &[RemasterSideView]) -> bool {
    sides.len() > 1
        || sides
            .iter()
            .any(|side| side.endpoints.len().max(side.terms.len()) > 1)
}
fn anchor_is_single(sides: &[RemasterSideView], selected: &RecordNavigationView) -> bool {
    sides.len() == 1 && sides[0].endpoints.len() == 1 && matches(sides, selected)
}
fn group_key(
    value: &RemasterDeclarationView,
    scope: &Scope,
    selected: Option<&RecordNavigationView>,
) -> (GroupKey, bool) {
    let (direction, anchor, family, generic, opposite) = if let Some(selected) = selected {
        let before = matches(&value.before, selected);
        let after = matches(&value.after, selected);
        if before && after {
            (
                Direction::Same,
                Some(Address::from(selected)),
                selected_family(value, selected),
                false,
                &value.before,
            )
        } else if after {
            (
                Direction::Incoming,
                Some(Address::from(selected)),
                selected_family(value, selected)
                    .or_else(|| single_current(&value.before).and_then(|(_, family)| family)),
                !anchor_is_single(&value.after, selected),
                &value.before,
            )
        } else if before {
            (
                Direction::Outgoing,
                Some(Address::from(selected)),
                selected_family(value, selected)
                    .or_else(|| single_current(&value.after).and_then(|(_, family)| family)),
                !anchor_is_single(&value.before, selected),
                &value.after,
            )
        } else {
            (Direction::Term, None, None, true, &value.before)
        }
    } else if let Some((anchor, family)) = single_current(&value.after) {
        (
            Direction::Term,
            Some(Address::from(anchor)),
            family,
            false,
            &value.before,
        )
    } else {
        (Direction::Term, None, None, true, &value.before)
    };
    let generic = generic || opposite.len() > 1;
    let counterpart = if direction == Direction::Outgoing
        && opposite.len() == 1
        && opposite[0].endpoints.len() == 1
    {
        match &opposite[0].endpoints[0].resolution {
            Resolution::Current { navigation, .. } => Some(Address::from(navigation)),
            Resolution::Absent { record_key, .. } => Some(Address {
                key: record_key.clone(),
                owners: vec![],
            }),
            _ => None,
        }
    } else {
        None
    };
    let meaning = meaning(value);
    let mergeable = matches!(meaning, Meaning::Succession | Meaning::Merger)
        && anchor.is_some()
        && (direction != Direction::Outgoing || counterpart.is_some())
        && !generic
        && !grouped(opposite)
        && !has_ambiguity(value)
        && !scope.body_truncated;
    (
        GroupKey {
            namespace: scope.namespace.clone(),
            family: scope.family.clone().or(family),
            qualifier: scope.qualifier.clone(),
            anchor,
            counterpart,
            direction,
            meaning,
            isolation: (!mergeable).then(|| value.id.clone()),
        },
        generic,
    )
}
fn normalized(value: &str) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}
fn operand(
    endpoint: &atlas_app_model::RemasterEndpointView,
    selected: Option<&RecordNavigationView>,
    generic: bool,
) -> Operand {
    match &endpoint.resolution {
        Resolution::Current { record, navigation } => {
            if selected.is_some_and(|selected| Address::from(selected) == Address::from(navigation))
            {
                let label = if generic {
                    "this entry".to_owned()
                } else {
                    endpoint.label.clone()
                };
                Operand {
                    key: OperandKey::Name(normalized(&label)),
                    label,
                    navigation: None,
                    source: None,
                }
            } else {
                Operand {
                    key: OperandKey::Current(Address::from(navigation)),
                    label: record.title.clone(),
                    navigation: Some(navigation.clone()),
                    source: record.publication.clone().or(record.pack.clone()),
                }
            }
        }
        Resolution::Absent { record_key, .. } => Operand {
            key: OperandKey::Absent(record_key.clone()),
            label: endpoint.label.clone(),
            navigation: None,
            source: None,
        },
        _ => Operand {
            key: OperandKey::Name(normalized(&endpoint.label)),
            label: endpoint.label.clone(),
            navigation: None,
            source: None,
        },
    }
}
fn side_has_content(side: &RemasterSideView) -> bool {
    !side.endpoints.is_empty()
        || !side.terms.is_empty()
        || !matches!(side.text.trim(), "" | "—" | "-")
}
fn operands(
    sides: &[RemasterSideView],
    selected: Option<&RecordNavigationView>,
    generic: bool,
) -> Vec<Operand> {
    sides
        .iter()
        .filter(|side| side_has_content(side))
        .flat_map(|side| {
            if !side.endpoints.is_empty() {
                side.endpoints
                    .iter()
                    .map(|endpoint| operand(endpoint, selected, generic))
                    .collect()
            } else if !side.terms.is_empty() {
                side.terms
                    .iter()
                    .map(|label| Operand {
                        key: OperandKey::Name(normalized(label)),
                        label: label.clone(),
                        navigation: None,
                        source: None,
                    })
                    .collect()
            } else if !side.text.trim().is_empty() {
                vec![Operand {
                    key: OperandKey::Name(normalized(&side.text)),
                    label: side.text.clone(),
                    navigation: None,
                    source: None,
                }]
            } else {
                vec![]
            }
        })
        .collect()
}
fn deduplicate(values: Vec<Operand>) -> Vec<Operand> {
    let mut absences = BTreeMap::<String, BTreeSet<String>>::new();
    let mut current = BTreeMap::<String, BTreeSet<Address>>::new();
    for value in &values {
        match &value.key {
            OperandKey::Absent(key) => {
                absences
                    .entry(normalized(&value.label))
                    .or_default()
                    .insert(key.clone());
            }
            OperandKey::Current(key) => {
                current
                    .entry(normalized(&value.label))
                    .or_default()
                    .insert(key.clone());
            }
            _ => {}
        }
    }
    let mut result = Vec::new();
    for mut value in values {
        let name = normalized(&value.label);
        let conflicting_absence = absences.get(&name).is_some_and(|keys| keys.len() > 1);
        let current_name = current.contains_key(&name);
        match &value.key {
            OperandKey::Absent(key) if conflicting_absence || current_name => {
                value.label = format!("{} ({key})", value.label);
            }
            OperandKey::Absent(_) => value.key = OperandKey::Name(name),
            OperandKey::Name(_) if conflicting_absence || current_name => {
                value.label = format!("{} (unidentified)", value.label)
            }
            _ => {}
        }
        if !result.iter().any(|prior: &Operand| prior.key == value.key) {
            result.push(value);
        }
    }
    result
}
fn qualified(mut values: Vec<Operand>) -> Vec<Operand> {
    let mut counts = BTreeMap::<String, usize>::new();
    for value in &values {
        if value.navigation.is_some() {
            *counts.entry(normalized(&value.label)).or_default() += 1;
        }
    }
    for value in &mut values {
        if let Some(navigation) = &value.navigation
            && counts
                .get(&normalized(&value.label))
                .is_some_and(|count| *count > 1)
        {
            value.label = format!(
                "{} — {}",
                value.label,
                value.source.as_deref().unwrap_or(&navigation.record_key)
            );
        }
    }
    let mut qualified_counts = BTreeMap::<String, usize>::new();
    for value in &values {
        if value.navigation.is_some() {
            *qualified_counts.entry(value.label.clone()).or_default() += 1;
        }
    }
    for value in &mut values {
        if let Some(navigation) = &value.navigation
            && qualified_counts
                .get(&value.label)
                .is_some_and(|count| *count > 1)
        {
            value.label = format!("{} ({})", value.label, address_hint(navigation));
        }
    }
    values
}
fn address_hint(navigation: &RecordNavigationView) -> String {
    let mut hint = navigation.record_key.clone();
    for owner in &navigation.owners {
        let identity = match &owner.identity {
            atlas_domain::OwnedContentIdentity::Stable(id) => id.clone(),
            atlas_domain::OwnedContentIdentity::SnapshotLocal { index } => {
                format!("position {index}")
            }
        };
        hint.push_str(&format!(
            " {}:{identity}",
            owner.collection.trim_start_matches('/')
        ));
    }
    hint
}
fn destination_labels(values: &[RemasterDeclarationView]) -> BTreeMap<Address, String> {
    let mut destinations = BTreeMap::new();
    for endpoint in values
        .iter()
        .flat_map(|value| value.before.iter().chain(&value.after))
        .flat_map(|side| &side.endpoints)
    {
        if let Resolution::Current { navigation, .. } = &endpoint.resolution {
            destinations
                .entry(Address::from(navigation))
                .or_insert_with(|| operand(endpoint, None, false));
        }
    }
    qualified(destinations.into_values().collect())
        .into_iter()
        .filter_map(|value| match value.key {
            OperandKey::Current(address) => Some((address, value.label)),
            _ => None,
        })
        .collect()
}
fn push_text(parts: &mut Vec<Part>, value: &str) {
    if !value.is_empty() {
        parts.push(Part::Text {
            text: super::text(value),
        });
    }
}
fn append_list(
    parts: &mut Vec<Part>,
    values: Vec<Operand>,
    labels: &BTreeMap<Address, String>,
    remaining: &mut usize,
    truncated: &mut bool,
) {
    let values = deduplicate(values)
        .into_iter()
        .map(|mut value| {
            if let OperandKey::Current(address) = &value.key
                && let Some(label) = labels.get(address)
            {
                value.label = label.clone();
            }
            value
        })
        .collect::<Vec<_>>();
    let count = values.len().min(*remaining);
    *truncated |= count < values.len();
    *remaining -= count;
    for (index, value) in values.into_iter().take(count).enumerate() {
        if index > 0 {
            push_text(
                parts,
                if index + 1 == count {
                    if count > 2 { ", and " } else { " and " }
                } else {
                    ", "
                },
            );
        }
        match value.navigation {
            Some(navigation) => parts.push(Part::Reference {
                label: super::text(&value.label),
                navigation,
            }),
            None => push_text(parts, &value.label),
        }
        *truncated |= value.label.chars().count() > 400;
    }
}
fn append_expression(
    parts: &mut Vec<Part>,
    sides: &[RemasterSideView],
    selected: Option<&RecordNavigationView>,
    labels: &BTreeMap<Address, String>,
    remaining: &mut usize,
    truncated: &mut bool,
) {
    for (index, side) in sides
        .iter()
        .filter(|side| side_has_content(side))
        .enumerate()
    {
        if *remaining == 0 {
            *truncated = true;
            break;
        }
        if index > 0 {
            push_text(parts, "; ");
        }
        let grouped = side.endpoints.len().max(side.terms.len()) > 1;
        if grouped {
            push_text(parts, "(");
        }
        append_list(
            parts,
            operands(std::slice::from_ref(side), selected, true),
            labels,
            remaining,
            truncated,
        );
        if grouped {
            push_text(parts, ")");
        }
    }
}
fn render(
    group: &Group,
    values: &[RemasterDeclarationView],
    scopes: &BTreeMap<String, Scope>,
    selected: Option<&RecordNavigationView>,
    labels: &BTreeMap<Address, String>,
) -> RemasterRelationshipView {
    let first = &values[group.indices[0]];
    let scope = &scopes[&first.id];
    let declarations = group
        .indices
        .iter()
        .map(|index| &values[*index])
        .collect::<Vec<_>>();
    let replaced = declarations
        .iter()
        .any(|value| value.kinds.contains(&Kind::Replaced));
    let mechanics = declarations
        .iter()
        .any(|value| value.kinds.contains(&Kind::MechanicsChanged));
    let opposite = declarations
        .iter()
        .flat_map(|value| match group.key.direction {
            Direction::Incoming | Direction::Same | Direction::Term => {
                operands(&value.before, selected, false)
            }
            Direction::Outgoing => operands(&value.after, selected, false),
        })
        .collect::<Vec<_>>();
    let old_count = deduplicate(opposite.clone()).len();
    let mut label = match (group.key.meaning, group.key.direction) {
        (Meaning::Succession, Direction::Same) => {
            if old_count > 1 {
                "Former names"
            } else {
                "Former name"
            }
        }
        (Meaning::Succession, Direction::Incoming) => {
            if replaced || old_count > 1 {
                "Replaces"
            } else {
                "Former name"
            }
        }
        (Meaning::Succession, Direction::Outgoing) => {
            if replaced {
                "Replaced by"
            } else {
                "Renamed to"
            }
        }
        (Meaning::Succession, Direction::Term) => "Replacement",
        (Meaning::Merger, Direction::Incoming | Direction::Same) => "Merged from",
        (Meaning::Merger, Direction::Outgoing) => "Merged into",
        (Meaning::Merger, Direction::Term) => "Merger",
        (Meaning::Removed, Direction::Incoming) => "Related removed entry",
        (Meaning::Removed, Direction::Outgoing) if !first.after.is_empty() => {
            "Removed in the remaster; related entry"
        }
        (Meaning::Removed, _) => "Removed in the remaster",
        (Meaning::Mechanics, _) => "Mechanics changed",
        (Meaning::Discussion, _) => "Remaster rules context",
        (Meaning::Other, _) => "Remaster relationship",
    }
    .to_owned();
    if group.generic {
        label = match group.key.meaning {
            Meaning::Succession => {
                if replaced {
                    "Replacement"
                } else {
                    "Name change"
                }
            }
            Meaning::Merger => "Merger",
            Meaning::Removed => "Removed in the remaster",
            Meaning::Mechanics => "Mechanics changed",
            Meaning::Discussion => "Remaster rules context",
            Meaning::Other => "Remaster relationship",
        }
        .into();
    }
    if let Some(qualifier) = &scope.qualifier
        && !scope.atlas
        && !qualifier.trim().is_empty()
    {
        label = format!("{label} ({})", super::text(qualifier));
    }
    let mut parts = Vec::new();
    let mut remaining = 20;
    let mut truncated = declarations.iter().any(|value| value.truncated);
    if group.generic {
        append_expression(
            &mut parts,
            &first.before,
            selected,
            labels,
            &mut remaining,
            &mut truncated,
        );
        if first.before.iter().any(side_has_content) && first.after.iter().any(side_has_content) {
            push_text(&mut parts, " → ");
        }
        append_expression(
            &mut parts,
            &first.after,
            selected,
            labels,
            &mut remaining,
            &mut truncated,
        );
    } else if group.key.meaning == Meaning::Discussion {
        append_expression(
            &mut parts,
            &first.after,
            selected,
            labels,
            &mut remaining,
            &mut truncated,
        );
    } else {
        append_list(&mut parts, opposite, labels, &mut remaining, &mut truncated);
        if group.key.direction == Direction::Term && first.after.iter().any(side_has_content) {
            if !parts.is_empty() {
                push_text(&mut parts, " → ");
            }
            append_expression(
                &mut parts,
                &first.after,
                None,
                labels,
                &mut remaining,
                &mut truncated,
            );
        }
    }
    if mechanics && group.key.meaning != Meaning::Mechanics {
        push_text(&mut parts, " (mechanics changed)");
    }
    let (label, label_shortened) = super::shorten(&label);
    truncated |= label_shortened;
    RemasterRelationshipView {
        id: format!("relationship-{}", first.id),
        label,
        parts,
        declaration_ids: declarations.iter().map(|value| value.id.clone()).collect(),
        explanation_links: vec![],
        truncated,
    }
}
pub(super) fn relationships(
    values: &[RemasterDeclarationView],
    scopes: &BTreeMap<String, Scope>,
    selected: Option<&RecordNavigationView>,
) -> Vec<RemasterRelationshipView> {
    let mut groups = Vec::<Group>::new();
    for (index, value) in values.iter().enumerate() {
        let (key, generic) = group_key(value, &scopes[&value.id], selected);
        if let Some(group) = groups.iter_mut().find(|group| group.key == key) {
            group.indices.push(index);
        } else {
            groups.push(Group {
                key,
                indices: vec![index],
                generic,
            });
        }
    }
    let labels = destination_labels(values);
    groups
        .iter()
        .map(|group| render(group, values, scopes, selected, &labels))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    #[derive(Deserialize)]
    struct Case {
        selection: RecordNavigationView,
        declarations: Vec<RemasterDeclarationView>,
    }
    fn case(name: &str) -> Case {
        let value: serde_json::Value =
            serde_json::from_str(include_str!("fixtures/current-relationships.json")).unwrap();
        serde_json::from_value(value["cases"][name].clone()).unwrap()
    }
    fn scopes(
        values: &[RemasterDeclarationView],
        namespace: &str,
        family: Option<&str>,
    ) -> BTreeMap<String, Scope> {
        values
            .iter()
            .map(|value| {
                (
                    value.id.clone(),
                    Scope {
                        namespace: namespace.into(),
                        family: family.map(str::to_owned),
                        qualifier: None,
                        atlas: value
                            .origins
                            .contains(&atlas_app_model::RemasterCitationOriginView::AtlasReviewed),
                        body_truncated: value
                            .before
                            .iter()
                            .chain(&value.after)
                            .any(|side| side.truncated),
                    },
                )
            })
            .collect()
    }
    fn words(value: &RemasterRelationshipView) -> String {
        value
            .parts
            .iter()
            .map(|part| match part {
                Part::Text { text } => text.as_str(),
                Part::Reference { label, .. } => label.as_str(),
            })
            .collect()
    }
    fn refs(value: &RemasterRelationshipView) -> Vec<(&str, &RecordNavigationView)> {
        value
            .parts
            .iter()
            .filter_map(|part| match part {
                Part::Reference { label, navigation } => Some((label.as_str(), navigation)),
                _ => None,
            })
            .collect()
    }
    #[test]
    fn remaster_actual_cleanse_five_clauses_become_one_relationship_with_three_plain_names() {
        let case = self::case("cleanse");
        assert_eq!(case.declarations.len(), 5);
        let mut scope = scopes(&case.declarations, "spell", Some("spell"));
        // Journal context does not declare a family; its exact selected spell does.
        for value in &case.declarations {
            if value.kinds == [Kind::Renamed] {
                scope.get_mut(&value.id).unwrap().family = None;
            }
        }
        let grouped = relationships(&case.declarations, &scope, Some(&case.selection));
        assert_eq!(grouped.len(), 1);
        assert_eq!(grouped[0].label, "Replaces");
        assert_eq!(grouped[0].declaration_ids.len(), 5);
        assert_eq!(
            words(&grouped[0]),
            "Remove Curse, Remove Disease, and Neutralize Poison"
        );
        assert!(refs(&grouped[0]).is_empty());
        assert_eq!(
            case.declarations
                .iter()
                .map(|value| value.citations.len())
                .sum::<usize>(),
            5
        );
        assert!(
            case.declarations
                .iter()
                .any(|value| value.context[0] == "Spell replacement")
        );
        assert!(
            case.declarations
                .iter()
                .any(|value| value.context[0] == "Spells")
        );
    }
    #[test]
    fn remaster_actual_nephilim_preserves_collective_merger_and_all_four_missing_names_once() {
        let case = self::case("nephilim");
        let grouped = relationships(
            &case.declarations,
            &scopes(&case.declarations, "rules", None),
            Some(&case.selection),
        );
        assert_eq!(grouped.len(), 1);
        assert_eq!(grouped[0].label, "Merged from");
        assert_eq!(words(&grouped[0]), "Aasimar, Aphorite, Ganzi, and Tiefling");
        assert!(refs(&grouped[0]).is_empty());
        assert_eq!(
            grouped[0].declaration_ids,
            case.declarations
                .iter()
                .map(|value| value.id.clone())
                .collect::<Vec<_>>()
        );
        assert_eq!(case.declarations[0].before[0].terms.len(), 4);
        assert!(!case.declarations[0].citations.is_empty());
    }
    #[test]
    fn remaster_actual_glossary_versions_link_each_actual_destination_once_without_main_provenance()
    {
        for (name, expected) in [
            ("legacy", "Reactive Strike"),
            ("reactive", "Attack of Opportunity"),
        ] {
            let case = self::case(name);
            let grouped = relationships(
                &case.declarations,
                &scopes(&case.declarations, "creature_ability", Some("action")),
                Some(&case.selection),
            );
            assert_eq!(grouped.len(), 1);
            assert_eq!(refs(&grouped[0]).len(), 1);
            assert_eq!(refs(&grouped[0])[0].0, expected);
            assert_eq!(words(&grouped[0]), expected);
            assert!(!grouped[0].label.contains("Atlas"));
            assert_eq!(
                case.declarations[0].citations[0].attribution,
                "Atlas mapping"
            );
        }
    }
    #[test]
    fn remaster_class_scopes_opposite_directions_and_ambiguity_remain_separate() {
        let case = self::case("cleanse");
        let mut values = vec![case.declarations[0].clone(), case.declarations[0].clone()];
        values[1].id = "other-class".into();
        let mut scope = scopes(&values, "class_feature", Some("spell"));
        scope.get_mut(&values[0].id).unwrap().qualifier = Some("Wizard".into());
        scope.get_mut(&values[1].id).unwrap().qualifier = Some("Cleric".into());
        let grouped = relationships(&values, &scope, Some(&case.selection));
        assert_eq!(grouped.len(), 2);
        assert!(grouped[0].label.contains("Wizard"));
        assert!(grouped[1].label.contains("Cleric"));
        let mut values = vec![case.declarations[0].clone(), case.declarations[0].clone()];
        values[1].id = "opposite".into();
        let second = &mut values[1];
        std::mem::swap(&mut second.before, &mut second.after);
        assert_eq!(
            relationships(
                &values,
                &scopes(&values, "spell", Some("spell")),
                Some(&case.selection)
            )
            .len(),
            2
        );
        values[1] = case.declarations[0].clone();
        values[1].id = "ambiguous".into();
        values[1].before[0].endpoints[0].resolution = Resolution::Ambiguous {
            candidates: vec![],
            truncated: false,
            message: "No identity selected".into(),
        };
        let grouped = relationships(
            &values,
            &scopes(&values, "spell", Some("spell")),
            Some(&case.selection),
        );
        assert_eq!(grouped.len(), 2);
        assert!(grouped.iter().all(|value| refs(value).is_empty()));
    }
    #[test]
    fn remaster_same_key_rename_uses_distinct_authored_former_names_as_plain_text() {
        let case = self::case("reactive");
        let mut value = case.declarations[0].clone();
        value.kinds = vec![Kind::Renamed];
        value.before = value.after.clone();
        value.before[0].endpoints[0].label = "Flat-Footed".into();
        value.before[0].terms = vec!["Flat-Footed".into()];
        let mut other = value.clone();
        other.id = "another-former-name".into();
        other.before[0].endpoints[0].label = "Off-Balance".into();
        other.before[0].terms = vec!["Off-Balance".into()];
        let values = vec![value, other];
        let grouped = relationships(
            &values,
            &scopes(&values, "condition", Some("condition")),
            Some(&case.selection),
        );
        assert_eq!(grouped.len(), 1);
        assert_eq!(grouped[0].label, "Former names");
        assert_eq!(words(&grouped[0]), "Flat-Footed and Off-Balance");
        assert!(refs(&grouped[0]).is_empty());
    }
    #[test]
    fn remaster_current_titles_are_honest_and_source_context_only_disambiguates_same_titles() {
        let case = self::case("reactive");
        let mut value = case.declarations[0].clone();
        value.before[0].endpoints[0].label = "Authored historical caption".into();
        if let Resolution::Current { record, .. } = &mut value.before[0].endpoints[0].resolution {
            record.title = "Reactive Strike".into();
        }
        let values = vec![value];
        let grouped = relationships(
            &values,
            &scopes(&values, "creature_ability", Some("action")),
            Some(&case.selection),
        );
        assert_eq!(
            refs(&grouped[0])[0].0,
            "Reactive Strike — Pathfinder Bestiary"
        );
        assert!(!words(&grouped[0]).contains("historical caption"));
    }
    #[test]
    fn remaster_nary_groups_and_explicit_term_transitions_never_become_cartesian_pairs() {
        let case = self::case("reactive");
        let mut value = case.declarations[0].clone();
        let mut old = value.before[0].endpoints[0].clone();
        let mut new = value.after[0].endpoints[0].clone();
        for (endpoint, key, title) in [
            (&mut old, "items:dddddddddddddddd", "Other old entry"),
            (&mut new, "items:eeeeeeeeeeeeeeee", "Other new entry"),
        ] {
            if let Resolution::Current { record, navigation } = &mut endpoint.resolution {
                record.record_key = key.into();
                record.title = title.into();
                navigation.record_key = key.into();
            }
            endpoint.label = title.into();
        }
        value.before[0].endpoints.push(old);
        value.after[0].endpoints.push(new);
        let values = vec![value];
        let grouped = relationships(
            &values,
            &scopes(&values, "creature_ability", Some("action")),
            Some(&case.selection),
        );
        assert_eq!(grouped.len(), 1);
        assert_eq!(words(&grouped[0]).matches('→').count(), 1);
        assert_eq!(words(&grouped[0]).matches('(').count(), 2);
        assert!(words(&grouped[0]).contains("this entry"));
        assert_eq!(grouped[0].declaration_ids.len(), 1);
        let case = self::case("cleanse");
        let grouped = relationships(
            &case.declarations,
            &scopes(&case.declarations, "spell", Some("spell")),
            None,
        );
        assert_eq!(grouped.len(), 1);
        assert_eq!(refs(&grouped[0]).len(), 1);
        assert_eq!(
            words(&grouped[0]),
            "Remove Curse, Remove Disease, and Neutralize Poison → Cleanse Affliction"
        );
    }
    #[test]
    fn remaster_name_cap_keeps_every_support_and_declares_the_cut() {
        let case = self::case("cleanse");
        let values = (0..21)
            .map(|index| {
                let mut value = case.declarations[0].clone();
                value.id = format!("support-{index}");
                value.before[0].endpoints[0].label = format!("Former name {index}");
                value.before[0].terms = vec![format!("Former name {index}")];
                value.before[0].endpoints[0].resolution = Resolution::Absent {
                    record_key: format!("spells-srd:{index:016}"),
                    message: "Unavailable".into(),
                };
                value
            })
            .collect::<Vec<_>>();
        let grouped = relationships(
            &values,
            &scopes(&values, "spell", Some("spell")),
            Some(&case.selection),
        );
        assert_eq!(grouped.len(), 1);
        assert!(grouped[0].truncated);
        assert_eq!(grouped[0].declaration_ids.len(), 21);
        assert!(!words(&grouped[0]).contains("Former name 20"));
        assert!(words(&grouped[0]).contains("Former name 19"));
    }
    #[test]
    fn remaster_outgoing_conflicting_targets_do_not_become_a_collective_replacement() {
        let case = self::case("legacy");
        let first = case.declarations[0].clone();
        let mut second = first.clone();
        second.id = "conflicting-successor".into();
        if let Resolution::Current { record, navigation } =
            &mut second.after[0].endpoints[0].resolution
        {
            record.record_key = "items:dddddddddddddddd".into();
            navigation.record_key = record.record_key.clone();
            record.title = "Different successor".into();
        }
        let values = vec![first.clone(), second.clone()];
        let grouped = relationships(
            &values,
            &scopes(&values, "creature_ability", Some("action")),
            Some(&case.selection),
        );
        assert_eq!(grouped.len(), 2);
        assert!(
            grouped
                .iter()
                .all(|value| value.declaration_ids.len() == 1 && refs(value).len() == 1)
        );
        assert!(!words(&grouped[0]).contains(" and "));
        assert!(!words(&grouped[1]).contains(" and "));
        if let Resolution::Current { record, .. } = &mut second.after[0].endpoints[0].resolution {
            record.title = "Reactive Strike".into();
        }
        let values = vec![first.clone(), second];
        let grouped = relationships(
            &values,
            &scopes(&values, "creature_ability", Some("action")),
            Some(&case.selection),
        );
        assert_eq!(grouped.len(), 2);
        assert_ne!(
            refs(&grouped[0])[0].1.record_key,
            refs(&grouped[1])[0].1.record_key
        );
        assert_ne!(
            refs(&grouped[0])[0].0,
            refs(&grouped[1])[0].0,
            "same-title separate claims need distinct captions"
        );
        let mut corroboration = first.clone();
        corroboration.id = "corroborating-source".into();
        let values = vec![first, corroboration];
        let grouped = relationships(
            &values,
            &scopes(&values, "creature_ability", Some("action")),
            Some(&case.selection),
        );
        assert_eq!(grouped.len(), 1);
        assert_eq!(refs(&grouped[0]).len(), 1);
        assert_eq!(grouped[0].declaration_ids.len(), 2);
    }
    #[test]
    fn remaster_consolidation_preserves_one_mechanics_note_and_all_raw_kinds() {
        let case = self::case("cleanse");
        let mut values = case.declarations;
        values[0].kinds.push(Kind::MechanicsChanged);
        values[1].kinds.push(Kind::MechanicsChanged);
        let grouped = relationships(
            &values,
            &scopes(&values, "spell", Some("spell")),
            Some(&case.selection),
        );
        assert_eq!(grouped.len(), 1);
        assert_eq!(words(&grouped[0]).matches("mechanics changed").count(), 1);
        assert_eq!(grouped[0].declaration_ids.len(), 5);
        assert!(values[0].kinds.contains(&Kind::MechanicsChanged));
    }
    #[test]
    fn remaster_same_title_sibling_owned_destinations_have_distinct_exact_context() {
        let case = self::case("legacy");
        let mut values = vec![case.declarations[0].clone(), case.declarations[0].clone()];
        values[1].id = "second-owned-target".into();
        for (index, value) in values.iter_mut().enumerate() {
            if let Resolution::Current { record, navigation } =
                &mut value.after[0].endpoints[0].resolution
            {
                record.record_key = "actors:cccccccccccccccc".into();
                navigation.record_key = record.record_key.clone();
                navigation.owners = vec![OwnedContentLocator {
                    collection: "/items".into(),
                    identity: atlas_domain::OwnedContentIdentity::Stable(format!(
                        "owned{index:011}"
                    )),
                }];
            }
        }
        let grouped = relationships(
            &values,
            &scopes(&values, "creature_ability", Some("action")),
            Some(&case.selection),
        );
        assert_eq!(grouped.len(), 2);
        let first = refs(&grouped[0])[0];
        let second = refs(&grouped[1])[0];
        assert_ne!(first.0, second.0);
        assert_eq!(first.1.record_key, second.1.record_key);
        assert_ne!(first.1.owners, second.1.owners);
        assert!(first.0.contains("owned00000000000"));
        assert!(second.0.contains("owned00000000001"));
    }
    #[test]
    fn remaster_no_seed_removed_and_discussion_keep_known_kind_without_identity_inference() {
        let case = self::case("cleanse");
        let mut removed = case.declarations[0].clone();
        removed.kinds = vec![Kind::Removed];
        removed.after.clear();
        let values = vec![removed];
        let grouped = relationships(&values, &scopes(&values, "spell", Some("spell")), None);
        assert_eq!(grouped[0].label, "Removed in the remaster");
        assert_eq!(words(&grouped[0]), "Remove Curse");
        assert!(refs(&grouped[0]).is_empty());
        let mut with_placeholder = values[0].clone();
        with_placeholder.after = vec![RemasterSideView {
            text: "—".into(),
            terms: vec![],
            endpoints: vec![],
            truncated: false,
        }];
        let placeholder_values = vec![with_placeholder];
        let placeholder = relationships(
            &placeholder_values,
            &scopes(&placeholder_values, "spell", Some("spell")),
            None,
        );
        assert_eq!(words(&placeholder[0]), "Remove Curse");
        let mut discussion = values[0].clone();
        discussion.id = "rules-discussion".into();
        discussion.kinds = vec![Kind::Discussion];
        discussion.before.clear();
        discussion.after = vec![RemasterSideView {
            text: "Spell school changes".into(),
            terms: vec![],
            endpoints: vec![atlas_app_model::RemasterEndpointView {
                label: "Spell school changes".into(),
                context: None,
                resolution: Resolution::SourceText,
            }],
            truncated: false,
        }];
        let values = vec![discussion];
        let grouped = relationships(&values, &scopes(&values, "rules", None), None);
        assert_eq!(grouped[0].label, "Remaster rules context");
        assert_eq!(words(&grouped[0]), "Spell school changes");
        assert!(refs(&grouped[0]).is_empty());
        let mut mechanics = values[0].clone();
        mechanics.kinds = vec![Kind::MechanicsChanged];
        let values = vec![mechanics];
        assert_eq!(
            relationships(&values, &scopes(&values, "rules", None), None)[0].label,
            "Mechanics changed"
        );
    }
}
