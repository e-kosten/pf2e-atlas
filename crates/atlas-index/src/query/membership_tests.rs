use super::{
    compiler::compile_predicate,
    tests::{database, result, root, row, text},
    {ValidatedQuery, parse_where, validate_query},
};
use atlas_domain::{QueryCompare, QueryExpression, QueryLiteral, QueryPredicate, QuerySetMatch};
use rusqlite::{Connection, params_from_iter, types::Value};

fn general_result(db: &Connection, query: &ValidatedQuery) -> Vec<i64> {
    let compiled = compile_predicate(query).unwrap();
    let mut statement = db
        .prepare(&format!(
            "SELECT r.record_id FROM records r WHERE ({}) IS TRUE ORDER BY r.record_id",
            compiled.expression
        ))
        .unwrap();
    statement
        .query_map(params_from_iter(compiled.parameters), |row| row.get(0))
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap()
}

fn eligible_result(
    db: &Connection,
    query: &ValidatedQuery,
    keys: Option<&[atlas_domain::RecordKey]>,
) -> Vec<i64> {
    let (sql, parameters) = crate::read_units::eligible_sql(query, keys).unwrap();
    db.prepare(&format!("{sql} ORDER BY r.record_id"))
        .unwrap()
        .query_map(params_from_iter(parameters), |row| row.get(0))
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap()
}

fn states() -> Connection {
    let db = database();
    for (id, state, ac) in [
        (1, "value", Some(26)),
        (2, "value", Some(0)),
        (3, "missing", None),
        (4, "null", None),
        (5, "invalid", None),
        (6, "not_applicable", None),
        (7, "value", Some(25)),
    ] {
        root(&db, id, "npc", "common");
        row(
            &db,
            "actor_projection",
            &[
                ("record_id", id.into()),
                ("armor_class_state", text(state)),
                ("armor_class", ac.map(Value::Integer).unwrap_or(Value::Null)),
            ],
        );
        if (3..=6).contains(&id) {
            db.execute(
                "UPDATE records SET traits_state=?1 WHERE record_id=?2",
                rusqlite::params![state, id],
            )
            .unwrap();
        }
    }
    root(&db, 8, "weapon", "common");
    row(
        &db,
        "actor_projection",
        &[
            ("record_id", 8.into()),
            ("armor_class_state", text("value")),
            ("armor_class", 100.into()),
        ],
    );
    root(&db, 9, "npc", "common");
    db.execute(
        "UPDATE records SET document_kind='Macro' WHERE record_id=9",
        [],
    )
    .unwrap();
    row(
        &db,
        "actor_projection",
        &[
            ("record_id", 9.into()),
            ("armor_class_state", text("value")),
            ("armor_class", 100.into()),
        ],
    );
    for (id, value) in [(1, "fire"), (7, "fire"), (7, "water"), (8, "fire")] {
        db.execute(
            "INSERT INTO record_traits VALUES(?,?)",
            rusqlite::params![id, value],
        )
        .unwrap();
    }
    db
}

#[test]
fn positive_membership_retains_unknown_zero_family_and_macro_semantics() {
    let db = states();
    for (expression, expected) in [
        ("defenses.ac >= 25", vec![1, 7]),
        ("source.type in ['weapon']", vec![8]),
        ("defenses.ac == 0", vec![2]),
        ("!(defenses.ac >= 25)", vec![2]),
        ("!(!(defenses.ac >= 25))", vec![1, 7]),
        ("defenses.ac >= 25 && level >= 3", vec![1, 7]),
        (
            "defenses.ac >= 25 || availability['defenses.ac']=='missing'",
            vec![1, 3, 7],
        ),
        ("defenses.ac >= 25 && (false || level==3)", vec![1, 7]),
        ("defenses.ac > 0 && false", vec![]),
        ("defenses.ac > 0 && !('fire' in traits)", vec![]),
        ("'fire' in traits", vec![1, 7, 8]),
        ("'fire' in traits && 'water' in traits", vec![7]),
        ("!('fire' in traits)", vec![2]),
    ] {
        let query = parse_where(expression).unwrap();
        assert_eq!(eligible_result(&db, &query, None), expected, "{expression}");
        assert_eq!(
            general_result(&db, &query),
            expected,
            "general {expression}"
        );
    }
    let query = parse_where("defenses.ac>=25 && 'fire' in traits").unwrap();
    let keys = ["test:0000000000000007".parse().unwrap()];
    assert_eq!(eligible_result(&db, &query, Some(&keys)), vec![7]);
    assert!(eligible_result(&db, &query, Some(&[])).is_empty());
}

#[test]
fn positive_set_membership_uses_known_parents_and_distinct_root_sets() {
    let db = states();
    for (op, values, expected) in [
        (QuerySetMatch::Includes, vec!["fire"], vec![1, 7, 8]),
        (
            QuerySetMatch::IncludesAny,
            vec!["fire", "water"],
            vec![1, 7, 8],
        ),
        (QuerySetMatch::IncludesAll, vec!["fire", "water"], vec![7]),
        (
            QuerySetMatch::IncludesAll,
            vec!["fire", "fire"],
            vec![1, 7, 8],
        ),
        (QuerySetMatch::ExcludesAny, vec!["fire", "water"], vec![2]),
    ] {
        let predicate = QueryPredicate::new(QueryExpression::SetMatch {
            field: "traits".into(),
            op,
            values: values.into_iter().map(str::to_owned).collect(),
        });
        let query = validate_query(&predicate).unwrap();
        assert_eq!(eligible_result(&db, &query, None), expected, "{op:?}");
        assert_eq!(general_result(&db, &query), expected, "general {op:?}");
    }
    for op in [QuerySetMatch::IncludesAny, QuerySetMatch::IncludesAll] {
        let predicate = QueryPredicate::new(QueryExpression::SetMatch {
            field: "traits".into(),
            op,
            values: vec![],
        });
        assert!(validate_query(&predicate).is_err());
    }
}

#[test]
fn indexed_membership_keeps_fractional_prices_and_extreme_numeric_affinity() {
    let db = database();
    for (ordinal, value) in [
        Value::Integer(i64::MIN),
        Value::Integer(-1),
        Value::Integer(0),
        Value::Real(0.5),
        Value::Integer(1),
        Value::Integer(9_007_199_254_740_991),
        Value::Integer(9_007_199_254_740_993),
        Value::Integer(i64::MAX),
    ]
    .into_iter()
    .enumerate()
    {
        let id = ordinal as i64 + 1;
        root(&db, id, "equipment", "common");
        row(
            &db,
            "physical_projection",
            &[
                ("id", id.into()),
                ("root_record_id", id.into()),
                ("price_per_item_cp_state", text("value")),
                ("price_per_item_cp", value),
            ],
        );
    }
    assert_eq!(result(&db, "physical.unit_price_gp == 0.005"), vec![4]);
    let steps = plan(&db, "physical.unit_price_gp>=10000");
    assert!(
        steps.iter().any(|step| step.contains("physical_price")),
        "{steps:?}"
    );
    assert!(
        !steps.iter().any(|step| step.contains("CORRELATED")),
        "{steps:?}"
    );
    for op in [
        QueryCompare::Eq,
        QueryCompare::Neq,
        QueryCompare::Lt,
        QueryCompare::Lte,
        QueryCompare::Gt,
        QueryCompare::Gte,
    ] {
        for value in [
            serde_json::json!(-1),
            serde_json::json!(0),
            serde_json::json!(0.005),
            serde_json::json!(90_071_992_547_409.92),
            serde_json::json!(i64::MAX),
        ] {
            let query = validate_query(&QueryPredicate::new(QueryExpression::Compare {
                field: "physical.unit_price_gp".into(),
                op,
                value: QueryLiteral::Number(value.as_number().unwrap().clone()),
            }))
            .unwrap();
            assert_eq!(
                eligible_result(&db, &query, None),
                general_result(&db, &query),
                "{op:?} {value}"
            );
        }
    }
}

fn plan(db: &Connection, expression: &str) -> Vec<String> {
    let (sql, parameters) =
        crate::read_units::eligible_sql(&parse_where(expression).unwrap(), None).unwrap();
    db.prepare(&format!("EXPLAIN QUERY PLAN {sql}"))
        .unwrap()
        .query_map(params_from_iter(parameters), |row| row.get(3))
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap()
}

#[test]
fn production_membership_exposes_existing_scalar_and_trait_indexes() {
    let db = states();
    for (expression, index) in [
        ("defenses.ac>=25", "actor_ac"),
        ("level>=3", "records_level"),
        ("'fire' in traits", "record_traits_value"),
    ] {
        let steps = plan(&db, expression);
        assert!(
            steps.iter().any(|step| step.contains(index)),
            "{expression}: {steps:?}"
        );
        assert!(
            !steps.iter().any(|step| step.contains("CORRELATED")),
            "{expression}: {steps:?}"
        );
    }
}

#[test]
fn conjunction_mixes_indexed_leaves_with_general_same_child_exists() {
    let db = database();
    for id in [1, 2] {
        root(&db, id, "npc", "common");
        row(
            &db,
            "actor_projection",
            &[
                ("record_id", id.into()),
                ("armor_class_state", text("value")),
                ("armor_class", 25.into()),
                ("items_state", text("value")),
            ],
        );
    }
    for (id, record, rank, tradition) in [
        (10, 1, 3, "divine"),
        (11, 1, 1, "arcane"),
        (12, 2, 3, "arcane"),
    ] {
        row(
            &db,
            "actor_items",
            &[
                ("id", id.into()),
                ("record_id", record.into()),
                ("original_index", (id - 10).into()),
                ("owner_selector_json", text("{}")),
                ("source_type_state", text("value")),
                ("source_type", text("spell")),
            ],
        );
        row(
            &db,
            "spell_projection",
            &[
                ("id", id.into()),
                ("actor_item_id", id.into()),
                ("rank_state", text("value")),
                ("rank", rank.into()),
                ("traditions_state", text("value")),
            ],
        );
        db.execute(
            "INSERT INTO spell_traditions VALUES(?,?)",
            rusqlite::params![id, tradition],
        )
        .unwrap();
    }
    let expression = "defenses.ac>=25 && embedded.spells.exists(s,s.spell.rank>=3 && 'arcane' in s.spell.traditions)";
    let query = parse_where(expression).unwrap();
    assert_eq!(eligible_result(&db, &query, None), vec![2]);
    assert_eq!(general_result(&db, &query), vec![2]);
    let steps = plan(&db, expression);
    assert!(
        steps.iter().any(|step| step.contains("actor_ac")),
        "{steps:?}"
    );
    let negated = parse_where("defenses.ac>=25 && !embedded.spells.exists(s,s.spell.rank>=3 && 'arcane' in s.spell.traditions)").unwrap();
    assert_eq!(eligible_result(&db, &negated, None), vec![1]);
    assert_eq!(general_result(&db, &negated), vec![1]);
}

#[test]
fn null_root_owners_from_embedded_only_matches_preserve_accepted_sets() {
    let db = database();
    root(&db, 1, "equipment", "common");
    root(&db, 2, "npc", "common");
    root(&db, 3, "spell", "common");
    row(
        &db,
        "actor_projection",
        &[("record_id", 2.into()), ("items_state", text("value"))],
    );
    for (id, family) in [(100, "equipment"), (101, "spell")] {
        row(
            &db,
            "actor_items",
            &[
                ("id", id.into()),
                ("record_id", 2.into()),
                ("original_index", (id - 100).into()),
                ("owner_selector_json", text("{}")),
                ("source_type_state", text("value")),
                ("source_type", text(family)),
            ],
        );
    }
    for (id, root_id, item_id, price) in [
        (200, Value::Integer(1), Value::Null, 1),
        (201, Value::Null, Value::Integer(100), 1_000_000),
    ] {
        row(
            &db,
            "physical_projection",
            &[
                ("id", id.into()),
                ("root_record_id", root_id),
                ("actor_item_id", item_id),
                ("price_per_item_cp_state", text("value")),
                ("price_per_item_cp", price.into()),
            ],
        );
    }
    for (id, root_id, item_id, tradition) in [
        (300, Value::Integer(3), Value::Null, "divine"),
        (301, Value::Null, Value::Integer(101), "arcane"),
    ] {
        row(
            &db,
            "spell_projection",
            &[
                ("id", id.into()),
                ("root_record_id", root_id),
                ("actor_item_id", item_id),
                ("traditions_state", text("value")),
            ],
        );
        db.execute(
            "INSERT INTO spell_traditions VALUES(?,?)",
            rusqlite::params![id, tradition],
        )
        .unwrap();
    }
    for (expression, expected) in [
        ("physical.unit_price_gp>=1000", vec![]),
        ("physical.unit_price_gp>=1000 && level>=0", vec![]),
        ("!(physical.unit_price_gp>=1000)", vec![1]),
        (
            "physical.unit_price_gp>=1000 || source.type=='npc'",
            vec![2],
        ),
        ("'arcane' in spell.traditions", vec![]),
        (
            "'arcane' in spell.traditions && 'divine' in spell.traditions",
            vec![],
        ),
        ("!('arcane' in spell.traditions)", vec![3]),
        ("'divine' in spell.traditions", vec![3]),
        (
            "embedded.equipment.exists(e,e.physical.unit_price_gp>=1000)",
            vec![2],
        ),
        (
            "embedded.spells.exists(s,'arcane' in s.spell.traditions)",
            vec![2],
        ),
    ] {
        assert_eq!(result(&db, expression), expected, "{expression}");
    }
}
