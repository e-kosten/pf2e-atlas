//! Keep the emitted shape of shared DTOs while documenting the wider Serde input.
//! Only components reached in both directions need a second name.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::{Value, json};

const INPUT_LOCATIONS: [&str; 2] = ["requestBody", "parameters"];
const COMPONENT_PREFIX: &str = "#/components/schemas/";
type Components = serde_json::Map<String, Value>;

struct InputFields {
    component: &'static str,
    optional: &'static [&'static str],
    nullable: &'static [&'static str],
}

const INPUT_FIELDS: &[InputFields] = &[
    InputFields {
        component: "ConditionDurationView",
        optional: &["timing"],
        nullable: &["remaining_rounds"],
    },
    InputFields {
        component: "ConditionExtraActionView",
        optional: &["guidance"],
        nullable: &[],
    },
    InputFields {
        component: "ConditionOriginView",
        optional: &[
            "participant_key",
            "record_key",
            "activity_id",
            "artifact_generation",
        ],
        nullable: &[],
    },
    InputFields {
        component: "ContentRollEncounterContext",
        optional: &["condition_id"],
        nullable: &[],
    },
    InputFields {
        component: "EncounterDifficultyReferenceView",
        optional: &["party_level", "party_size"],
        nullable: &[],
    },
    InputFields {
        component: "HpEventContextView",
        optional: &[
            "attack_or_spell_damage",
            "base_death_threshold",
            "critical",
            "enemy_source",
            "keep_current_initiative",
            "keep_upright",
            "lethality",
            "natural_rest_healing",
            "responsible_participant_key",
        ],
        nullable: &[],
    },
    InputFields {
        component: "PersistentDraftContext",
        optional: &["artifact_generation", "boundary_key", "record_key"],
        nullable: &[],
    },
    InputFields {
        component: "PersistentRecoveryDraft",
        optional: &["dc_override", "roll"],
        nullable: &[],
    },
    InputFields {
        component: "QueryPredicate",
        optional: &[],
        nullable: &["clause_id"],
    },
    InputFields {
        component: "QueryRecordSelection",
        optional: &["embedded_path", "field", "passage", "source_fingerprint"],
        nullable: &["spell_rank"],
    },
    InputFields {
        component: "RecordNavigationView",
        optional: &["field", "passage", "source_fingerprint"],
        nullable: &["spell_rank"],
    },
    InputFields {
        component: "ContentRollOffer",
        optional: &["authored_command"],
        nullable: &["spell_rank", "encounter"],
    },
    InputFields {
        component: "SimilarityOptions",
        optional: &["candidate_parts", "decay", "parts"],
        nullable: &[],
    },
    InputFields {
        component: "SourcePassageAddress",
        optional: &["label"],
        nullable: &[],
    },
];

pub(super) fn describe_accepted_inputs(document: &mut Value) -> Result<(), String> {
    let paths = document["paths"]
        .as_object()
        .ok_or("missing OpenAPI paths")?;
    let operations = |locations: &[&str]| {
        paths
            .values()
            .filter_map(Value::as_object)
            .flat_map(Components::values)
            .flat_map(|operation| {
                locations
                    .iter()
                    .flat_map(|location| references(&operation[*location]))
            })
            .collect::<BTreeSet<_>>()
    };
    let request_roots = operations(&INPUT_LOCATIONS);
    let response_roots = operations(&["responses"]);
    let schemas = document["components"]["schemas"]
        .as_object_mut()
        .ok_or("missing OpenAPI schemas")?;
    let requests = reachable(schemas, request_roots)?;
    let responses = reachable(schemas, response_roots)?;
    let mut affected: BTreeSet<_> = INPUT_FIELDS
        .iter()
        .map(|fields| fields.component.to_owned())
        .collect();

    // Reverse reachability includes recursive components without recursively
    // expanding their definitions.
    loop {
        let parents: Vec<_> = requests
            .difference(&affected)
            .filter(|name| {
                references(&schemas[*name])
                    .iter()
                    .any(|child| affected.contains(child))
            })
            .cloned()
            .collect();
        if parents.is_empty() {
            break;
        }
        affected.extend(parents);
    }

    let input_names: BTreeMap<_, _> = affected
        .intersection(&requests)
        .filter(|name| responses.contains(*name))
        .map(|name| {
            let input = format!("{name}InputSchema");
            if schemas.contains_key(&input) {
                return Err(format!("duplicate schema {input}"));
            }
            Ok((name.clone(), input))
        })
        .collect::<Result<_, _>>()?;
    for name in affected.intersection(&requests) {
        let mut schema = schemas[name].clone();
        if let Some(fields) = INPUT_FIELDS.iter().find(|fields| fields.component == name) {
            loosen(&mut schema, fields);
        }
        rewrite_references(&mut schema, &input_names);
        schemas.insert(input_names.get(name).unwrap_or(name).clone(), schema);
    }

    let paths = document["paths"]
        .as_object_mut()
        .ok_or("missing OpenAPI paths")?;
    for operation in paths
        .values_mut()
        .filter_map(Value::as_object_mut)
        .flat_map(Components::values_mut)
    {
        for location in INPUT_LOCATIONS {
            if let Some(input) = operation.get_mut(location) {
                rewrite_references(input, &input_names);
            }
        }
    }
    Ok(())
}

fn loosen(schema: &mut Value, fields: &InputFields) {
    if let Some(required) = schema.get_mut("required").and_then(Value::as_array_mut) {
        required.retain(|field| {
            field
                .as_str()
                .is_none_or(|name| !fields.optional.contains(&name))
        });
    }
    if let Some(properties) = schema.get_mut("properties").and_then(Value::as_object_mut) {
        for name in fields.nullable {
            if let Some(property) = properties.get_mut(*name) {
                *property = json!({"anyOf": [property.take(), {"type": "null"}]});
            }
        }
    }
    for keyword in ["oneOf", "allOf"] {
        if let Some(branches) = schema.get_mut(keyword).and_then(Value::as_array_mut) {
            for branch in branches {
                loosen(branch, fields);
            }
        }
    }
}

fn reachable(schemas: &Components, roots: BTreeSet<String>) -> Result<BTreeSet<String>, String> {
    let mut pending: Vec<_> = roots.into_iter().collect();
    let mut seen = BTreeSet::new();
    while let Some(name) = pending.pop() {
        if seen.insert(name.clone()) {
            let schema = schemas
                .get(&name)
                .ok_or_else(|| format!("missing schema {name}"))?;
            pending.extend(references(schema));
        }
    }
    Ok(seen)
}

fn references(node: &Value) -> Vec<String> {
    let mut found = Vec::new();
    match node {
        Value::Object(object) => {
            if let Some(name) = object
                .get("$ref")
                .and_then(Value::as_str)
                .and_then(|path| path.strip_prefix(COMPONENT_PREFIX))
            {
                found.push(name.to_owned());
            }
            found.extend(object.values().flat_map(references));
        }
        Value::Array(items) => found.extend(items.iter().flat_map(references)),
        _ => {}
    }
    found
}

fn rewrite_references(node: &mut Value, names: &BTreeMap<String, String>) {
    match node {
        Value::Object(object) => {
            if let Some(reference) = object.get_mut("$ref")
                && let Some(name) = reference
                    .as_str()
                    .and_then(|path| path.strip_prefix(COMPONENT_PREFIX))
                && let Some(input) = names.get(name)
            {
                *reference = Value::String(format!("{COMPONENT_PREFIX}{input}"));
            }
            for child in object.values_mut() {
                rewrite_references(child, names);
            }
        }
        Value::Array(items) => {
            for child in items {
                rewrite_references(child, names);
            }
        }
        _ => {}
    }
}
