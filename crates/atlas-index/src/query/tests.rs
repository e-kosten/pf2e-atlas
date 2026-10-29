use super::*;
use atlas_domain::query::*;
use rusqlite::{Connection, params_from_iter, types::Value};
use serde_json::json;
fn available<T: std::fmt::Debug>(outcome: atlas_domain::QueryFacetOutcome<T>) -> T {
    match outcome {
        atlas_domain::QueryFacetOutcome::Available { value } => value,
        other => panic!("expected available discovery, got {other:?}"),
    }
}

fn database() -> Connection {
    let db = Connection::open_in_memory().unwrap();
    db.execute_batch(include_str!(
        "../../migrations/00000000000001_create_artifact/up.sql"
    ))
    .unwrap();
    db.execute_batch(crate::CONSUMABLE_FILTER_SCHEMA_SQL)
        .unwrap();
    db.execute_batch(crate::REFERENCE_NUMERIC_FILTER_SCHEMA_SQL)
        .unwrap();
    db.execute_batch(crate::PHYSICAL_BASE_TYPE_SCHEMA_SQL)
        .unwrap();
    db.execute("INSERT INTO packs VALUES('test','Test')", [])
        .unwrap();
    db
}
fn row(db: &Connection, table: &str, overrides: &[(&str, Value)]) {
    let mut stmt = db.prepare(&format!("PRAGMA table_info({table})")).unwrap();
    let columns = stmt
        .query_map([], |r| Ok((r.get::<_, String>(1)?, r.get::<_, i64>(3)?)))
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    let mut names = Vec::new();
    let mut values = Vec::new();
    for (name, not_null) in columns {
        if let Some((_, value)) = overrides.iter().find(|(n, _)| *n == name) {
            names.push(name);
            values.push(value.clone());
        } else if not_null != 0 {
            let value = if name.ends_with("_state") {
                Value::Text("not_applicable".into())
            } else {
                panic!("fixture needs {table}.{name}")
            };
            names.push(name);
            values.push(value);
        }
    }
    let slots = (1..=values.len())
        .map(|i| format!("?{i}"))
        .collect::<Vec<_>>()
        .join(",");
    db.execute(
        &format!("INSERT INTO {table} ({}) VALUES ({slots})", names.join(",")),
        params_from_iter(values),
    )
    .unwrap();
}
fn text(s: &str) -> Value {
    Value::Text(s.to_owned())
}
fn root(db: &Connection, id: i64, family: &str, rarity: &str) {
    db.execute(
        "INSERT INTO record_identity(id,key,availability) VALUES(?,?,'available')",
        rusqlite::params![id, format!("test:{id:016}")],
    )
    .unwrap();
    row(
        db,
        "records",
        &[
            ("record_id", id.into()),
            ("key", text(&format!("test:{id:016}"))),
            ("pack_id", text("test")),
            (
                "document_kind",
                text(if ["npc", "hazard"].contains(&family) {
                    "Actor"
                } else {
                    "Item"
                }),
            ),
            ("source_path", text("fixture.json")),
            ("content_hash", text("hash")),
            ("source_type_state", text("value")),
            ("source_type", text(family)),
            ("rarity_state", text("value")),
            ("rarity", text(rarity)),
            ("traits_state", text("value")),
            ("level_state", text("value")),
            ("level", Value::Integer(3)),
        ],
    );
}
fn result(db: &Connection, expression: &str) -> Vec<i64> {
    let compiled = compile_predicate(&parse_where(expression).unwrap()).unwrap();
    let mut s = db
        .prepare(&format!(
            "SELECT r.record_id FROM records r WHERE ({}) IS TRUE ORDER BY r.record_id",
            compiled.expression
        ))
        .unwrap();
    s.query_map(params_from_iter(compiled.parameters), |r| r.get(0))
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap()
}
#[test]
fn serde_request_shape_and_shared_ts_match() {
    let input = json!({"clause_id":"parent","kind":"exists","collection":"embedded.spells","scope_id":"spell-group","predicate":{"clause_id":"rank","kind":"compare","field":"spell.rank","op":"gte","value":3}});
    let p: QueryPredicate = serde_json::from_value(input.clone()).unwrap();
    assert_eq!(serde_json::to_value(p).unwrap(), input);
    for invalid in [
        json!({"kind":"metric","key":"hp.max","value":3}),
        json!({"kind":"compare","field":"level","op":"eq","value":3,"old_metric":"x"}),
    ] {
        assert!(serde_json::from_value::<QueryPredicate>(invalid).is_err());
    }
}
#[test]
fn cel_supported_examples_and_fail_closed_limits() {
    for expression in [
        "defenses.hp.maximum>=80",
        "source.pack.id in ['spells-srd','equipment-srd']",
        "'fire' in traits && rarity != 'common'",
        "hazard.complexity==false && hazard.hardness>=10",
        "spell.rank >= 3 && 'arcane' in spell.traditions",
        "embedded.spells.exists(i, i.source.type=='spell' && i.spell.rank>=3 && 'arcane' in i.spell.traditions)",
        "!embedded.spells.exists(i,i.availability['spell.rank']=='invalid')",
        "availability['defenses.hp.maximum']=='null'",
        "level >= -1",
        "level >= -9223372036854775808",
        "1 <= level",
    ] {
        assert!(
            parse_where(expression).is_ok(),
            "{expression}: {:?}",
            parse_where(expression)
        );
    }
    for expression in [
        "embedded.spells[0].spell.rank>=3",
        "embedded.spells.all(i,i.spell.rank>=3)",
        "embedded.spells.exists(i,i.spell.damage.exists(d,d.type=='fire'))",
        "level==spell.rank",
        "level+1>=3",
        "traits.matches('x')",
        "has(defenses.hp.maximum)",
        "defenses.hp.maximum==null",
        "availability[source.type]=='value'",
        "unknown_field==1",
        "rarity=='made-up'",
        "spell.traditions=='arcane'",
        "embedded.spells.exists(i,level>1)",
        "level==18446744073709551615u",
    ] {
        assert!(parse_where(expression).is_err(), "{expression}");
    }
    for expression in [
        format!("{}true{}", "(".repeat(2000), ")".repeat(2000)),
        format!("{}0{}", "[".repeat(2000), "]".repeat(2000)),
        format!("{}true", "!".repeat(2000)),
        format!("{}==0", "a.".repeat(2000)),
        "x".repeat(20000),
    ] {
        assert!(parse_where(&expression).is_err());
    }
    let q = parse_where(r#"publication.title=="x' OR 1=1 --""#).unwrap();
    let compiled = compile_predicate(&q).unwrap();
    assert!(!compiled.expression.contains("OR 1=1 --"));
    assert!(compiled.parameters.contains(&text("x' OR 1=1 --")));
    for expression in [
        format!("publication.title == '{}'", "([{{!".repeat(100)),
        format!("publication.title == r'{}'", "([{{!".repeat(100)),
        format!("publication.title == '''{}'''", "([{{!\n".repeat(100)),
        format!(
            "publication.title == R\"\"\"{}\"\"\"",
            "([{{!\n".repeat(100)
        ),
        format!("true // {}\n && true", "([{{!".repeat(100)),
        r#"publication.title == "escaped \" ([{""#.to_owned(),
    ] {
        assert!(
            parse_where(&expression).is_ok(),
            "{expression}: {:?}",
            parse_where(&expression)
        );
    }
}
#[test]
fn structured_limits_and_scope_attribution_reject_unsafe_requests() {
    let oversized = QueryPredicate::new(QueryExpression::Compare {
        field: "publication.title".to_owned(),
        op: QueryCompare::Eq,
        value: QueryLiteral::String("x".repeat(20_000)),
    });
    assert_eq!(validate_query(&oversized).unwrap_err().code, "query_limit");
    let mut deep = QueryPredicate::boolean(true);
    for _ in 0..40 {
        deep = QueryPredicate::new(QueryExpression::Not {
            predicate: Box::new(deep),
        });
    }
    assert_eq!(validate_query(&deep).unwrap_err().code, "query_limit");
    let wide = QueryPredicate::new(QueryExpression::AllOf {
        children: vec![QueryPredicate::boolean(true); 300],
    });
    assert_eq!(validate_query(&wide).unwrap_err().code, "query_limit");
    let listed = QueryPredicate::new(QueryExpression::In {
        field: "publication.title".to_owned(),
        values: vec![QueryLiteral::String("x".to_owned()); 129],
    });
    assert_eq!(validate_query(&listed).unwrap_err().code, "literal_list");
    let mut one = QueryPredicate::boolean(true);
    one.clause_id = Some("same".to_owned());
    assert_eq!(
        validate_query(&QueryPredicate::new(QueryExpression::AllOf {
            children: vec![one.clone(), one]
        }))
        .unwrap_err()
        .code,
        "clause_id"
    );
    let group = || {
        QueryPredicate::new(QueryExpression::Exists {
            collection: "embedded.spells".to_owned(),
            scope_id: "same".to_owned(),
            predicate: Box::new(QueryPredicate::boolean(true)),
        })
    };
    assert_eq!(
        validate_query(&QueryPredicate::new(QueryExpression::AllOf {
            children: vec![group(), group()]
        }))
        .unwrap_err()
        .code,
        "scope_id"
    );
    let mut unknown = QueryPredicate::new(QueryExpression::Compare {
        field: "invented.field".to_owned(),
        op: QueryCompare::Eq,
        value: QueryLiteral::Number(1.into()),
    });
    unknown.clause_id = Some("bad-clause".to_owned());
    let error = validate_query(&unknown).unwrap_err();
    assert_eq!(error.field.as_deref(), Some("invented.field"));
    assert_eq!(error.clause_id.as_deref(), Some("bad-clause"));
}

#[test]
fn known_empty_sets_and_macro_eligibility_do_not_infer_unknown_absence() {
    let db = database();
    root(&db, 1, "npc", "common");
    assert!(result(&db, "'fire' in traits").is_empty());
    assert_eq!(result(&db, "!('fire' in traits)"), vec![1]);
    for state in ["missing", "null", "invalid", "not_applicable"] {
        db.execute("UPDATE records SET traits_state=?1", [state])
            .unwrap();
        assert!(result(&db, "!('fire' in traits)").is_empty(), "{state}");
    }
    db.execute("UPDATE records SET document_kind='Macro'", [])
        .unwrap();
    assert!(result(&db, "true").is_empty());
    assert!(result(&db, "availability['traits']=='invalid' || true").is_empty());
}
#[test]
fn catalog_is_complete_unique_and_uses_generated_absent_choices() {
    let catalog = query_capabilities().unwrap();
    let mut ids = std::collections::HashSet::new();
    for field in &catalog.fields {
        assert!(ids.insert(&field.id), "{}", field.id);
        assert!(field.operators.contains(&"state".to_owned()));
        for example in &field.examples {
            assert!(
                parse_where(example).is_ok(),
                "{} {example}: {:?}",
                field.id,
                parse_where(example)
            );
        }
    }
    for id in [
        "level",
        "body.size",
        "physical.size",
        "ancestry.default_size",
        "movement.speeds[].feet",
        "defenses.resistances[].type",
        "spell.damage[].kinds",
        "embedded.spells[].spell.rank",
        "embedded.conditions[].condition.is_valued",
        "embedded.deities[].deity.domains.alternate",
        "effect.duration.hours",
    ] {
        assert!(ids.contains(&id.to_owned()), "{id}");
    }
    assert!(!ids.contains(&"embedded.spells[].spell.damage".to_owned()));
    assert!(
        catalog
            .fields
            .iter()
            .find(|f| f.id == "rarity")
            .unwrap()
            .choices
            .contains(&"unique".to_owned())
    );
    assert!(parse_where("rarity=='unique'").is_ok());
    assert!(parse_where("'never-before-seen-trait' in traits").is_ok());
}

#[test]
fn every_catalog_binding_executes_against_actual_schema() {
    let db = database();
    root(&db, 1, "npc", "common");
    row(
        &db,
        "actor_projection",
        &[("record_id", 1.into()), ("items_state", text("value"))],
    );
    for d in query_capabilities().unwrap().fields {
        let field = d.path.clone();
        let expression = match d.field_type {
            QueryFieldType::Number => QueryExpression::Compare {
                field,
                op: QueryCompare::Eq,
                value: QueryLiteral::Number(0.into()),
            },
            QueryFieldType::String => QueryExpression::Compare {
                field,
                op: QueryCompare::Eq,
                value: QueryLiteral::String(
                    d.choices.first().cloned().unwrap_or_else(|| "x".to_owned()),
                ),
            },
            QueryFieldType::Boolean => QueryExpression::Compare {
                field,
                op: QueryCompare::Eq,
                value: QueryLiteral::Boolean(false),
            },
            QueryFieldType::Set => QueryExpression::SetMatch {
                field,
                op: QuerySetMatch::Includes,
                values: vec![d.choices.first().cloned().unwrap_or_else(|| "x".to_owned())],
            },
            QueryFieldType::Collection => QueryExpression::Exists {
                collection: field,
                scope_id: "all-binding".to_owned(),
                predicate: Box::new(QueryPredicate::boolean(true)),
            },
        };
        let p = QueryPredicate::new(expression);
        let p = if let Some(scope) = &d.scope {
            QueryPredicate::new(QueryExpression::Exists {
                collection: scope.clone(),
                scope_id: "member-binding".to_owned(),
                predicate: Box::new(p),
            })
        } else {
            p
        };
        let query = validate_query(&p).unwrap();
        let compiled = compile_predicate(&query).unwrap();
        let sql = format!(
            "SELECT r.record_id FROM records r WHERE ({}) IS TRUE",
            compiled.expression
        );
        let mut statement = db
            .prepare(&sql)
            .unwrap_or_else(|e| panic!("{}: {e} {sql}", d.id));
        statement
            .query_map(params_from_iter(compiled.parameters), |r| {
                r.get::<_, i64>(0)
            })
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
    }
}
#[test]
fn scalar_unknown_truth_table_and_explicit_states() {
    let db = database();
    root(&db, 1, "npc", "common");
    for state in ["value", "missing", "null", "invalid", "not_applicable"] {
        db.execute(
            "UPDATE records SET level_state=?1,level=CASE WHEN ?1='value' THEN 3 END",
            [state],
        )
        .unwrap();
        for expression in ["level>=3", "!(level<3)", "level!=2"] {
            assert_eq!(
                result(&db, expression),
                if state == "value" { vec![1] } else { vec![] },
                "{state} {expression}"
            );
        }
        assert_eq!(
            result(&db, &format!("availability['level']=='{state}'")),
            vec![1]
        );
        assert_eq!(result(&db, "level>=3 || true"), vec![1]);
        assert_eq!(result(&db, "level>=3 && false"), Vec::<i64>::new());
    }
    db.execute("UPDATE records SET level_state='invalid',level=NULL", [])
        .unwrap();
    for (a, av) in [
        ("true", Some(true)),
        ("false", Some(false)),
        ("level>=3", None),
    ] {
        for (b, bv) in [
            ("true", Some(true)),
            ("false", Some(false)),
            ("level>=3", None),
        ] {
            for op in ["&&", "||"] {
                let expression = format!("({a}) {op} ({b})");
                let truth = |v: Option<bool>| match v {
                    Some(true) => QueryTruth::True,
                    Some(false) => QueryTruth::False,
                    None => QueryTruth::Unknown,
                };
                let expected = if op == "&&" {
                    truth(av).and(truth(bv))
                } else {
                    truth(av).or(truth(bv))
                };
                assert_eq!(
                    !result(&db, &expression).is_empty(),
                    expected == QueryTruth::True,
                    "{expression}"
                );
            }
        }
    }
}
#[test]
fn sql_exists_true_overrides_unknown_in_either_order_and_never_splits_children() {
    let db = database();
    root(&db, 1, "npc", "common");
    row(
        &db,
        "actor_projection",
        &[
            ("record_id", 1.into()),
            ("resistances_state", text("value")),
        ],
    );
    row(
        &db,
        "actor_iwr_entries",
        &[
            ("entry_id", 1.into()),
            ("record_id", 1.into()),
            ("kind", text("resistance")),
            ("original_index", 0.into()),
            ("type_state", text("value")),
            ("type", text("fire")),
            ("value_state", text("null")),
        ],
    );
    row(
        &db,
        "actor_iwr_entries",
        &[
            ("entry_id", 2.into()),
            ("record_id", 1.into()),
            ("kind", text("resistance")),
            ("original_index", 1.into()),
            ("type_state", text("value")),
            ("type", text("fire")),
            ("value_state", text("value")),
            ("value", 5.into()),
        ],
    );
    let yes = "defenses.resistances.exists(r,r.type=='fire' && r.value>=5)";
    assert_eq!(result(&db, yes), vec![1]);
    db.execute(
        "UPDATE actor_iwr_entries SET original_index=original_index+2",
        [],
    )
    .unwrap();
    db.execute(
        "UPDATE actor_iwr_entries SET original_index=100-original_index",
        [],
    )
    .unwrap();
    assert_eq!(result(&db, yes), vec![1]);
    db.execute(
        "UPDATE actor_iwr_entries SET type='cold' WHERE entry_id=2",
        [],
    )
    .unwrap();
    assert!(result(&db, yes).is_empty());
    assert!(result(&db, &format!("!({yes})")).is_empty());
    db.execute(
        "UPDATE actor_iwr_entries SET value_state='value',value=2 WHERE entry_id=1",
        [],
    )
    .unwrap();
    assert_eq!(result(&db, &format!("!({yes})")), vec![1]);
    db.execute("DELETE FROM actor_iwr_entries", []).unwrap();
    assert_eq!(result(&db, "!defenses.resistances.exists(r,true)"), vec![1]);
    for state in ["missing", "null", "invalid", "not_applicable"] {
        db.execute("UPDATE actor_projection SET resistances_state=?1", [state])
            .unwrap();
        assert!(result(&db, "defenses.resistances.exists(r,true)").is_empty());
        assert!(result(&db, "!defenses.resistances.exists(r,true)").is_empty());
    }
}
#[test]
fn numeric_boundaries_and_unit_gates_preserve_exact_domains() {
    let db = database();
    root(&db, 1, "npc", "common");
    db.execute("UPDATE records SET level=9223372036854775807", [])
        .unwrap();
    assert_eq!(result(&db, "level == 9223372036854775807"), vec![1]);
    assert!(parse_where("level==18446744073709551615u").is_err());
    root(&db, 2, "effect", "common");
    row(
        &db,
        "effect_projection",
        &[
            ("id", 1.into()),
            ("root_record_id", 2.into()),
            ("duration_unit_state", text("value")),
            ("duration_unit", text("hours")),
            ("duration_value_state", text("value")),
            ("duration_value", 3.into()),
        ],
    );
    assert_eq!(result(&db, "effect.duration.hours>=3"), vec![2]);
    assert!(result(&db, "!(effect.duration.rounds>=3)").is_empty());
    assert_eq!(
        result(
            &db,
            "availability['effect.duration.rounds']=='not_applicable'"
        ),
        vec![1, 2]
    );
}

#[test]
fn remaster_facets_restore_candidate_before_suppression_and_keep_absent_choices() {
    use crate::discovery::{QueryFacetContext, QueryValuesRequest, field_values};
    let db = database();
    root(&db, 1, "spell", "rare");
    root(&db, 2, "spell", "common");
    row(
        &db,
        "remaster_pairs",
        &[
            ("legacy_record_id", 1.into()),
            ("remaster_record_id", 2.into()),
            ("evidence_json", text("{}")),
        ],
    );
    let mut predicate = parse_where("rarity=='common'").unwrap().predicate().clone();
    predicate.clause_id = Some("rarity-choice".to_owned());
    let mut request = QueryValuesRequest {
        only_matching: false,
        field: "rarity".to_owned(),
        clause_id: Some("rarity-choice".to_owned()),
        query: validate_query(&predicate).unwrap(),
        context: QueryFacetContext {
            prefer_remaster: true,
            ..Default::default()
        },
        text: None,
        offset: 0,
        limit: 100,
    };
    let options = available(field_values(&db, &request).unwrap());
    let count = |options: &atlas_domain::query_discovery::QueryValueOptions, value: &str| {
        options
            .options
            .iter()
            .find(|o| o.value == QueryLiteral::String(value.to_owned()))
            .unwrap()
            .distinct_roots
    };
    assert_eq!(count(&options, "common"), 1);
    assert_eq!(count(&options, "rare"), 1);
    assert_eq!(count(&options, "unique"), 0);
    assert!(options.exhaustive);
    request.only_matching = true;
    request.limit = 1;
    let narrowed = available(field_values(&db, &request).unwrap());
    assert_eq!(narrowed.total_values, 2);
    assert!(
        narrowed
            .options
            .iter()
            .all(|option| option.distinct_roots > 0)
    );
    request.only_matching = false;
    request.offset = 100;
    request.limit = 1;
    let retained = available(field_values(&db, &request).unwrap());
    assert_eq!(retained.options.len(), 1);
    assert!(retained.options[0].selected);
    assert_eq!(
        retained.options[0].value,
        QueryLiteral::String("common".to_owned())
    );
    request.offset = 0;
    request.limit = 100;
    request.context.eligible_keys = Some(vec!["test:0000000000000001".parse().unwrap()]);
    request.context.bounded_candidates = true;
    let options = available(field_values(&db, &request).unwrap());
    assert_eq!(count(&options, "rare"), 1);
    assert_eq!(count(&options, "common"), 0);
    assert!(!options.exhaustive);
}

#[test]
fn explicit_value_counts_use_independent_counterfactual_root_sets_and_scopes() {
    use crate::discovery::{QueryFacetContext, QueryValueCountsRequest, field_value_counts};
    let db = database();
    // Roots 1 and 2 form a verified edition pair; 3 is another spell, 4 an NPC,
    // and 5 an excluded product family. An absent but valid rarity stays zero.
    for (id, family, rarity) in [
        (1, "spell", "rare"),
        (2, "spell", "common"),
        (3, "spell", "rare"),
        (4, "npc", "rare"),
        (5, "macro", "rare"),
    ] {
        root(&db, id, family, rarity);
    }
    row(
        &db,
        "remaster_pairs",
        &[
            ("legacy_record_id", 1.into()),
            ("remaster_record_id", 2.into()),
            ("evidence_json", text("{}")),
        ],
    );
    let predicate: QueryPredicate = serde_json::from_value(json!({"kind":"all_of","children":[{"kind":"compare","field":"source.type","op":"eq","value":"spell"},{"kind":"in","clause_id":"rarity","field":"rarity","values":["common","rare"]}]})).unwrap();
    let mut request = QueryValueCountsRequest {
        field: "rarity".into(),
        clause_id: Some("rarity".into()),
        query: validate_query(&predicate).unwrap(),
        context: QueryFacetContext {
            prefer_remaster: true,
            ..Default::default()
        },
        values: ["rare", "common", "unique", "rare"]
            .map(|v| QueryLiteral::String(v.into()))
            .to_vec(),
    };
    let counts = available(field_value_counts(&db, &request).unwrap());
    // Replacing the multi-selection with rare restores legacy 1 because remaster
    // 2 does not match rare. Count roots {1,3}; common has {2}; unique has {}.
    let expected_roots = [vec![1, 3], vec![2], vec![]];
    assert_eq!(
        counts
            .options
            .iter()
            .map(|o| o.distinct_roots)
            .collect::<Vec<_>>(),
        expected_roots.map(|roots| roots.len() as u64)
    );
    assert_eq!(counts.total_values, 3);
    assert!(counts.options[0].selected && counts.options[1].selected);
    request.context.eligible_keys = Some(vec![
        "test:0000000000000003".parse().unwrap(),
        "test:0000000000000004".parse().unwrap(),
    ]);
    assert_eq!(
        available(field_value_counts(&db, &request).unwrap())
            .options
            .iter()
            .map(|o| o.distinct_roots)
            .collect::<Vec<_>>(),
        vec![1, 0, 0]
    );
    request.context.eligible_keys = Some(vec![]);
    assert!(
        available(field_value_counts(&db, &request).unwrap())
            .options
            .iter()
            .all(|o| o.distinct_roots == 0)
    );
    request.clause_id = None;
    request.query = parse_where("true").unwrap();
    request.context.eligible_keys = None;
    assert_eq!(
        available(field_value_counts(&db, &request).unwrap()).options[0].distinct_roots,
        3
    );
}

#[test]
fn explicit_count_validation_precedes_optional_unavailability() {
    use crate::discovery::{QueryFacetContext, QueryValueCountsRequest, field_value_counts};
    let db = database();
    let predicate: QueryPredicate=serde_json::from_value(json!({"kind":"not","predicate":{"kind":"all_of","children":[{"kind":"compare","clause_id":"rarity","field":"rarity","op":"eq","value":"rare"}]}})).unwrap();
    let mut request = QueryValueCountsRequest {
        field: "rarity".into(),
        clause_id: Some("rarity".into()),
        query: validate_query(&predicate).unwrap(),
        context: QueryFacetContext::default(),
        values: vec![QueryLiteral::String("rare".into())],
    };
    assert!(matches!(
        field_value_counts(&db, &request).unwrap(),
        atlas_domain::QueryFacetOutcome::Unavailable { .. }
    ));
    request.values = vec![QueryLiteral::Boolean(true)];
    assert!(field_value_counts(&db, &request).is_err());
    request.values = vec![QueryLiteral::String("rare".into()); 129];
    assert!(
        matches!(field_value_counts(&db,&request),Err(crate::IndexError::Query(error)) if error.code=="discovery_limit")
    );
    request.values = vec![QueryLiteral::String("fire".into())];
    request.field = "traits".into();
    assert!(
        matches!(field_value_counts(&db,&request),Err(crate::IndexError::Query(error)) if error.code=="facet_target_field")
    );
}

#[test]
fn explicit_value_counts_preserve_other_fields_and_remove_same_field_selections() {
    use crate::discovery::{QueryFacetContext, QueryValueCountsRequest, field_value_counts};
    let db = database();
    for (id, family, rarity) in [
        (1, "spell", "common"),
        (2, "spell", "rare"),
        (3, "npc", "rare"),
        (4, "equipment", "unique"),
        (5, "spell", "rare"),
        (6, "macro", "rare"),
    ] {
        root(&db, id, family, rarity);
    }
    let predicate: QueryPredicate=serde_json::from_value(json!({"kind":"all_of","children":[{"kind":"in","clause_id":"target","field":"rarity","values":["common"]},{"kind":"any_of","children":[{"kind":"compare","field":"source.type","op":"eq","value":"spell"},{"kind":"compare","field":"source.type","op":"eq","value":"npc"}]},{"kind":"not","predicate":{"kind":"compare","field":"source.type","op":"eq","value":"npc"}}]})).unwrap();
    let mut request = QueryValueCountsRequest {
        field: "rarity".into(),
        clause_id: Some("target".into()),
        query: validate_query(&predicate).unwrap(),
        context: QueryFacetContext::default(),
        values: ["common", "rare", "unique"]
            .map(|value| QueryLiteral::String(value.into()))
            .to_vec(),
    };
    // The positive target is outside OR/NOT. Both siblings stay executable:
    // OR admits spells/NPCs, NOT removes NPCs, so rare roots are exactly {2,5}.
    let expected = [vec![1], vec![2, 5], vec![]];
    assert_eq!(
        available(field_value_counts(&db, &request).unwrap())
            .options
            .iter()
            .map(|option| option.distinct_roots)
            .collect::<Vec<_>>(),
        expected.map(|roots| roots.len() as u64)
    );
    let predicate: QueryPredicate=serde_json::from_value(json!({"kind":"all_of","children":[{"kind":"in","clause_id":"target","field":"rarity","values":["common"]},{"kind":"in","clause_id":"same-field-sibling","field":"rarity","values":["common","rare"]}]})).unwrap();
    request.query = validate_query(&predicate).unwrap();
    // Every rarity clause is self-excluded; other product eligibility still applies.
    let expected = [vec![1], vec![2, 3, 5], vec![4]];
    assert_eq!(
        available(field_value_counts(&db, &request).unwrap())
            .options
            .iter()
            .map(|option| option.distinct_roots)
            .collect::<Vec<_>>(),
        expected.map(|roots| roots.len() as u64)
    );
    request.clause_id = None;
    request.query = parse_where("true").unwrap();
    request.values.clear();
    let empty = available(field_value_counts(&db, &request).unwrap());
    assert!(empty.options.is_empty());
    assert_eq!(empty.total_values, 0);
}

#[test]
fn child_facets_preserve_remaining_same_child_conditions_and_count_roots_once() {
    use crate::discovery::{QueryFacetContext, QueryValuesRequest, field_values};
    let db = database();
    root(&db, 1, "npc", "common");
    root(&db, 2, "npc", "common");
    for id in [1, 2] {
        row(
            &db,
            "actor_projection",
            &[("record_id", id.into()), ("items_state", text("value"))],
        );
    }
    for (id, root, rank, tradition) in [
        (1, 1, 1, "arcane"),
        (2, 1, 5, "divine"),
        (3, 2, 5, "arcane"),
        (4, 2, 5, "arcane"),
    ] {
        row(
            &db,
            "actor_items",
            &[
                ("id", id.into()),
                ("record_id", root.into()),
                ("original_index", id.into()),
                ("owner_selector_json", text("[]")),
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
            "INSERT INTO spell_traditions VALUES(?1,?2)",
            rusqlite::params![id, tradition],
        )
        .unwrap();
    }
    let input = json!({"kind":"exists","collection":"embedded.spells","scope_id":"spell-scope","predicate":{"kind":"all_of","children":[{"clause_id":"rank-choice","kind":"compare","field":"spell.rank","op":"gte","value":3},{"clause_id":"tradition-choice","kind":"set_match","field":"spell.traditions","op":"includes","values":["arcane"]}]}});
    let query = validate_query(&serde_json::from_value(input).unwrap()).unwrap();
    let request = QueryValuesRequest {
        only_matching: false,
        field: "embedded.spells[].spell.traditions".to_owned(),
        clause_id: Some("tradition-choice".to_owned()),
        query,
        context: QueryFacetContext::default(),
        text: None,
        offset: 0,
        limit: 100,
    };
    let options = available(field_values(&db, &request).unwrap());
    let explicit = available(
        crate::discovery::field_value_counts(
            &db,
            &crate::QueryValueCountsRequest {
                field: request.field.clone(),
                clause_id: request.clause_id.clone(),
                query: request.query.clone(),
                context: request.context.clone(),
                values: ["arcane", "divine", "occult"]
                    .map(|v| QueryLiteral::String(v.into()))
                    .to_vec(),
            },
        )
        .unwrap(),
    );
    // Arcane has only parent 2 despite two matching child rows. Parent 1's arcane
    // child is below rank3; its rank5 divine child cannot witness arcane.
    assert_eq!(
        explicit
            .options
            .iter()
            .map(|o| o.distinct_roots)
            .collect::<Vec<_>>(),
        [vec![2], vec![1], vec![]].map(|roots| roots.len() as u64)
    );
    let numeric = available(
        crate::discovery::field_counts(
            &db,
            &crate::discovery::QueryCountsRequest {
                field: "embedded.spells[].spell.rank".to_owned(),
                clause_id: Some("rank-choice".to_owned()),
                query: request.query.clone(),
                context: request.context.clone(),
            },
        )
        .unwrap(),
    );
    assert_eq!(numeric.minimum, Some(1.into()));
    assert_eq!(numeric.maximum, Some(5.into()));
    assert_eq!(numeric.states[0].occurrences, 3);
    assert_eq!(numeric.states[0].distinct_roots, 2);
    for (tradition, count) in [("arcane", 1), ("divine", 1), ("occult", 0)] {
        assert_eq!(
            options
                .options
                .iter()
                .find(|o| o.value == QueryLiteral::String(tradition.to_owned()))
                .unwrap()
                .distinct_roots,
            count
        );
    }
    let other: QueryPredicate = serde_json::from_value(json!({"kind":"exists","collection":"embedded.spells","scope_id":"another-spell-scope","predicate":{"kind":"compare","field":"spell.rank","op":"lte","value":1}})).unwrap();
    let mut independent = request.clone();
    independent.query = validate_query(&QueryPredicate::new(QueryExpression::AllOf {
        children: vec![request.query.predicate().clone(), other],
    }))
    .unwrap();
    let independent = available(field_values(&db, &independent).unwrap());
    assert_eq!(
        independent
            .options
            .iter()
            .find(|o| o.value == QueryLiteral::String("divine".into()))
            .unwrap()
            .distinct_roots,
        1
    );
    assert_eq!(
        independent
            .options
            .iter()
            .find(|o| o.value == QueryLiteral::String("arcane".into()))
            .unwrap()
            .distinct_roots,
        0
    );
    let vocabulary = crate::discovery::field_vocabulary(
        &db,
        &crate::QueryVocabularyRequest {
            field: request.field.clone(),
            eligible_keys: Some(vec!["test:0000000000000001".parse().unwrap()]),
            text: None,
            offset: 0,
            limit: 10,
            selected_values: vec![],
        },
    )
    .unwrap();
    assert_eq!(vocabulary.total_values, 2);
    assert!(
        vocabulary
            .options
            .iter()
            .any(|o| o.value == QueryLiteral::String("arcane".into()))
    );
    assert!(
        vocabulary
            .options
            .iter()
            .any(|o| o.value == QueryLiteral::String("divine".into()))
    );
    let mut bad = request.clone();
    let mut p = request.query.predicate().clone();
    p = QueryPredicate::new(QueryExpression::Not {
        predicate: Box::new(p),
    });
    bad.query = validate_query(&p).unwrap();
    assert!(matches!(
        field_values(&db, &bad),
        Ok(atlas_domain::QueryFacetOutcome::Unavailable { .. })
    ));
}

#[test]
fn disjunctive_target_facets_are_explicitly_unavailable() {
    use crate::discovery::{QueryFacetContext, QueryValuesRequest, field_values};
    let db = database();
    root(&db, 1, "npc", "rare");
    let expression = json!({"kind":"any_of","children":[{"clause_id":"target","kind":"compare","field":"rarity","op":"eq","value":"rare"},{"kind":"boolean_constant","value":true}]});
    {
        let predicate: QueryPredicate = serde_json::from_value(expression).unwrap();
        let request = QueryValuesRequest {
            only_matching: false,
            field: "rarity".to_owned(),
            clause_id: Some("target".to_owned()),
            query: validate_query(&predicate).unwrap(),
            context: QueryFacetContext::default(),
            text: None,
            offset: 0,
            limit: 10,
        };
        assert!(matches!(
            field_values(&db, &request),
            Ok(atlas_domain::QueryFacetOutcome::Unavailable { .. })
        ));
    }
}

#[test]
fn state_discovery_partitions_roots_and_labels_child_occurrences() {
    use crate::discovery::{QueryCountsRequest, QueryFacetContext, field_counts};
    let db = database();
    for id in 1..=5 {
        root(&db, id, "npc", "common");
    }
    for (id, state) in (1..=5).zip(["value", "missing", "null", "invalid", "not_applicable"]) {
        db.execute("UPDATE records SET level_state=?1,level=CASE WHEN ?1='value' THEN 0 END WHERE record_id=?2",rusqlite::params![state,id]).unwrap();
    }
    let counts = available(
        field_counts(
            &db,
            &QueryCountsRequest {
                field: "level".to_owned(),
                clause_id: None,
                query: parse_where("true").unwrap(),
                context: QueryFacetContext::default(),
            },
        )
        .unwrap(),
    );
    assert_eq!(counts.counting_scope, "root");
    assert_eq!(counts.minimum, Some(0.into()));
    assert_eq!(counts.maximum, Some(0.into()));
    let mut states = counts
        .states
        .iter()
        .map(|state| {
            (
                state.state.as_str(),
                state.occurrences,
                state.distinct_roots,
            )
        })
        .collect::<Vec<_>>();
    states.sort();
    assert_eq!(
        states,
        ["invalid", "missing", "not_applicable", "null", "value"].map(|state| (state, 1, 1))
    );
}

#[test]
fn common_level_or_rank_preserves_spell_rank_and_all_unavailable_states() {
    let db = database();
    root(&db, 1, "npc", "common");
    root(&db, 2, "equipment", "common");
    for (id, state, rank) in [
        (3, "value", Some(7)),
        (4, "missing", None),
        (5, "null", None),
        (6, "invalid", None),
        (7, "value", Some(0)),
    ] {
        root(&db, id, "spell", "common");
        row(
            &db,
            "spell_projection",
            &[
                ("id", id.into()),
                ("root_record_id", id.into()),
                ("rank_state", text(state)),
                ("rank", rank.map(Value::Integer).unwrap_or(Value::Null)),
            ],
        );
    }
    root(&db, 8, "action", "common");
    assert_eq!(result(&db, "level_or_rank == 3"), vec![1, 2]);
    assert_eq!(result(&db, "level_or_rank == 7"), vec![3]);
    assert_eq!(result(&db, "level_or_rank == 0"), vec![7]);
    assert_eq!(result(&db, "!(level_or_rank == 3)"), vec![3, 7]);
    for (state, ids) in [
        ("missing", vec![4]),
        ("null", vec![5]),
        ("invalid", vec![6]),
        ("not_applicable", vec![8]),
    ] {
        assert_eq!(
            result(
                &db,
                &format!("availability[\"level_or_rank\"] == \"{state}\"")
            ),
            ids
        );
    }
    assert_eq!(result(&db, "level == 3"), vec![1, 2]);
    assert_eq!(result(&db, "spell.rank == 7"), vec![3]);
    assert_eq!(query_capabilities().unwrap().version, 8);
    let catalog = query_capabilities().unwrap();
    assert_eq!(
        catalog
            .fields
            .iter()
            .filter(|f| f.id == "level_or_rank")
            .count(),
        1
    );
    assert!(
        !catalog
            .fields
            .iter()
            .any(|f| f.id == "embedded.spells[].level_or_rank")
    );
}

#[test]
fn unsupported_facet_ancestry_does_not_hide_malformed_targets_or_limits() {
    use crate::discovery::{QueryFacetContext, QueryValuesRequest, field_values};
    let db = database();
    root(&db, 1, "npc", "common");
    let p: QueryPredicate = serde_json::from_value(json!({"kind":"not","predicate":{"kind":"all_of","children":[{"clause_id":"target","kind":"compare","field":"rarity","op":"eq","value":"rare"}]}})).unwrap();
    let mut request = QueryValuesRequest {
        only_matching: false,
        field: "rarity".into(),
        clause_id: Some("target".into()),
        query: validate_query(&p).unwrap(),
        context: QueryFacetContext::default(),
        text: None,
        offset: 0,
        limit: 10,
    };
    assert!(matches!(
        field_values(&db, &request),
        Ok(atlas_domain::QueryFacetOutcome::Unavailable {
            reason: atlas_domain::FacetUnavailableReason::NonConjunctiveContext
        })
    ));
    request.field = "traits".into();
    assert!(
        matches!(field_values(&db,&request), Err(crate::IndexError::Query(e)) if e.code == "facet_target_field")
    );
    request.field = "embedded.spells[].rarity".into();
    assert!(
        matches!(field_values(&db,&request), Err(crate::IndexError::Query(e)) if e.code == "facet_target_scope")
    );
    request.field = "rarity".into();
    request.clause_id = Some("absent".into());
    assert!(
        matches!(field_values(&db,&request), Err(crate::IndexError::Query(e)) if e.code == "clause_id")
    );
    request.clause_id = Some("target".into());
    request.limit = 0;
    assert!(
        matches!(field_values(&db,&request), Err(crate::IndexError::Query(e)) if e.code == "discovery_limit")
    );
    request.limit = 10;
    request.field = "level_or_rank".into();
    assert!(
        matches!(field_values(&db,&request), Err(crate::IndexError::Query(e)) if e.code == "discovery_type")
    );
    let group: QueryPredicate = serde_json::from_value(json!({"clause_id":"group","kind":"not","predicate":{"kind":"boolean_constant","value":true}})).unwrap();
    request.field = "rarity".into();
    request.clause_id = Some("group".into());
    request.query = validate_query(&group).unwrap();
    assert!(
        matches!(field_values(&db,&request), Err(crate::IndexError::Query(e)) if e.code == "facet_target_type")
    );
}

#[test]
fn count_free_vocabulary_is_scoped_paged_and_keeps_selections_separately() {
    use crate::discovery::{QueryVocabularyRequest, field_vocabulary};
    use atlas_domain::{QueryVocabularyBasis, RecordKey};
    let db = database();
    for id in 1..=3 {
        root(&db, id, "npc", "common");
    }
    db.execute(
        "UPDATE records SET document_kind='Macro' WHERE record_id=3",
        [],
    )
    .unwrap();
    for (id, trait_value) in [(1, "fire"), (1, "flying"), (2, "cold"), (3, "macro-only")] {
        db.execute(
            "INSERT INTO record_traits(record_id,value) VALUES(?1,?2)",
            rusqlite::params![id, trait_value],
        )
        .unwrap();
    }
    row(
        &db,
        "remaster_pairs",
        &[
            ("legacy_record_id", 1.into()),
            ("remaster_record_id", 2.into()),
            ("evidence_json", text("{}")),
        ],
    );
    let mut request = QueryVocabularyRequest {
        field: "traits".into(),
        eligible_keys: None,
        text: Some("f".into()),
        offset: 1,
        limit: 1,
        selected_values: vec![QueryLiteral::String("absent-selection".into())],
    };
    let page = field_vocabulary(&db, &request).unwrap();
    assert_eq!(page.total_values, 2);
    assert_eq!(page.options.len(), 1);
    assert_eq!(page.options[0].value, QueryLiteral::String("flying".into()));
    assert_eq!(page.selected_values, request.selected_values);
    assert_eq!(
        page.basis,
        QueryVocabularyBasis::ProductArtifactBeforeEditionPreference
    );
    let json = serde_json::to_value(&page).unwrap();
    assert!(json["options"][0].get("distinct_roots").is_none());
    request.text = None;
    request.offset = 0;
    request.eligible_keys = Some(vec![RecordKey::parse("test:0000000000000002").unwrap()]);
    let scoped = field_vocabulary(&db, &request).unwrap();
    assert_eq!(scoped.total_values, 1);
    assert_eq!(scoped.options[0].value, QueryLiteral::String("cold".into()));
    assert_eq!(
        scoped.basis,
        QueryVocabularyBasis::ProductKeyScopeBeforeEditionPreference
    );
    request.eligible_keys = Some(vec![]);
    let empty = field_vocabulary(&db, &request).unwrap();
    assert_eq!(empty.total_values, 0);
    assert!(empty.options.is_empty());
    assert_eq!(empty.selected_values, request.selected_values);
    request.field = "rarity".into();
    request.selected_values = vec![QueryLiteral::String("invented".into())];
    assert!(
        matches!(field_vocabulary(&db,&request),Err(crate::IndexError::Query(e)) if e.code=="invalid_choice")
    );
    request.field = "traits".into();
    request.selected_values = vec![QueryLiteral::Boolean(true)];
    assert!(field_vocabulary(&db, &request).is_err());
}

#[test]
fn root_facets_ignore_inclusions_and_exclusions_of_the_edited_field() {
    use crate::discovery::{QueryFacetContext, QueryValueCountsRequest, field_value_counts};
    let db = database();
    root(&db, 1, "spell", "common");
    root(&db, 2, "spell", "rare");
    root(&db, 3, "npc", "rare");
    for negative in [
        json!({"kind":"not","predicate":{"kind":"in","clause_id":"target","field":"rarity","values":["rare"]}}),
        json!({"kind":"compare","clause_id":"target","field":"rarity","op":"neq","value":"rare"}),
    ] {
        let predicate = serde_json::from_value(json!({"kind":"all_of","children":[negative,
          {"kind":"in","field":"rarity","values":["common"]},
          {"kind":"compare","field":"source.type","op":"eq","value":"spell"}]}))
        .unwrap();
        let result = available(
            field_value_counts(
                &db,
                &QueryValueCountsRequest {
                    field: "rarity".into(),
                    clause_id: Some("target".into()),
                    query: validate_query(&predicate).unwrap(),
                    context: QueryFacetContext::default(),
                    values: vec![
                        QueryLiteral::String("common".into()),
                        QueryLiteral::String("rare".into()),
                    ],
                },
            )
            .unwrap(),
        );
        assert_eq!(
            result
                .options
                .iter()
                .map(|option| option.distinct_roots)
                .collect::<Vec<_>>(),
            vec![1, 1]
        );
    }
}

#[test]
fn embedded_collection_membership_is_independent_of_parent_kind_and_traits() {
    let db = database();
    root(&db, 1, "npc", "common");
    root(&db, 2, "spell", "common");
    root(&db, 3, "npc", "common");
    root(&db, 4, "npc", "common");
    for id in [1, 3, 4] {
        row(
            &db,
            "actor_projection",
            &[
                ("record_id", id.into()),
                (
                    "items_state",
                    text(if id == 4 { "invalid" } else { "value" }),
                ),
            ],
        );
    }
    row(
        &db,
        "record_traits",
        &[("record_id", 2.into()), ("value", text("fire"))],
    );
    for (id, family, trait_value) in [(10, "spell", "fire"), (11, "weapon", "agile")] {
        row(
            &db,
            "actor_items",
            &[
                ("id", id.into()),
                ("record_id", 1.into()),
                ("original_index", id.into()),
                ("owner_selector_json", text("{}")),
                ("source_type_state", text("value")),
                ("source_type", text(family)),
                ("traits_state", text("value")),
            ],
        );
        row(
            &db,
            "actor_item_traits",
            &[("item_id", id.into()), ("value", text(trait_value))],
        );
    }
    db.execute("UPDATE records SET record_kind_state='value',record_kind=CASE WHEN source_type='npc' THEN 'creature' ELSE 'spell' END", []).unwrap();
    assert_eq!(
        result(
            &db,
            "kind == 'creature' && embedded.spells.exists(s, 'fire' in s.traits)"
        ),
        vec![1]
    );
    assert!(
        result(
            &db,
            "kind == 'spell' && embedded.spells.exists(s, 'fire' in s.traits)"
        )
        .is_empty()
    );
    assert_eq!(result(&db, "'fire' in traits"), vec![2]);
    assert_eq!(
        result(&db, "embedded.spells.exists(s, 'fire' in s.traits)"),
        vec![1]
    );
    assert!(result(&db, "embedded.equipment.exists(e, 'fire' in e.traits)").is_empty());
    assert_eq!(
        result(
            &db,
            "embedded.equipment.exists(e, e.kind == 'equipment' && e.source.type == 'weapon' && 'agile' in e.traits)"
        ),
        vec![1]
    );
    assert!(
        result(
            &db,
            "embedded.equipment.exists(e, 'agile' in e.traits && 'fire' in e.traits)"
        )
        .is_empty()
    );
    assert_eq!(result(&db, "!embedded.spells.exists(s, true)"), vec![3]);
    assert_eq!(
        result(&db, "availability['embedded.spells'] == 'invalid'"),
        vec![4]
    );
    assert_eq!(
        result(&db, "availability['embedded.spells'] == 'not_applicable'"),
        vec![2]
    );
}

#[test]
fn unit_price_gp_converts_fractional_copper_for_root_and_embedded_items() {
    let db = database();
    root(&db, 1, "equipment", "common");
    root(&db, 2, "npc", "common");
    row(
        &db,
        "actor_projection",
        &[("record_id", 2.into()), ("items_state", text("value"))],
    );
    row(
        &db,
        "actor_items",
        &[
            ("id", 10.into()),
            ("record_id", 2.into()),
            ("original_index", 0.into()),
            ("owner_selector_json", text("{}")),
            ("source_type_state", text("value")),
            ("source_type", text("equipment")),
        ],
    );
    for (id, root_id, item_id) in [
        (1, Value::Integer(1), Value::Null),
        (2, Value::Null, Value::Integer(10)),
    ] {
        row(
            &db,
            "physical_projection",
            &[
                ("id", id.into()),
                ("root_record_id", root_id),
                ("actor_item_id", item_id),
                ("price_per_item_cp_state", text("value")),
                ("price_per_item_cp", Value::Real(0.5)),
            ],
        );
    }
    assert_eq!(result(&db, "physical.unit_price_gp == 0.005"), vec![1]);
    assert_eq!(
        result(
            &db,
            "embedded.equipment.exists(e, e.physical.unit_price_gp == 0.005)"
        ),
        vec![2]
    );
    assert!(result(&db, "physical.unit_price_gp > 0.005").is_empty());
    db.execute(
        "UPDATE physical_projection SET price_per_item_cp_state='invalid',price_per_item_cp=NULL",
        [],
    )
    .unwrap();
    assert!(result(&db, "!(physical.unit_price_gp > 10)").is_empty());
}

#[test]
fn retired_filter_names_are_rejected_without_aliases() {
    for expression in [
        "actor.items.exists(i,true)",
        "actor.level > 1",
        "item.level > 1",
        "record.kind == 'spell'",
        "actor.speeds.exists(s,s.value>20)",
        "physical.price.per_item_cp > 100",
    ] {
        assert!(parse_where(expression).is_err(), "{expression}");
    }
    let catalog = query_capabilities().unwrap();
    let scopes = catalog
        .fields
        .iter()
        .filter(|f| f.scope.is_none() && f.path.starts_with("embedded."))
        .collect::<Vec<_>>();
    assert_eq!(scopes.len(), 17);
    assert!(
        catalog
            .fields
            .iter()
            .any(|f| f.id == "embedded.equipment[].kind")
    );
    assert!(
        !catalog
            .fields
            .iter()
            .any(|f| f.id == "embedded.spells[].level")
    );
    assert_eq!(
        catalog
            .fields
            .iter()
            .find(|f| f.id == "physical.unit_price_gp")
            .unwrap()
            .units
            .as_deref(),
        Some("gp")
    );
}

#[test]
fn unavailable_embedded_membership_does_not_become_empty() {
    let db = database();
    root(&db, 1, "npc", "common");
    row(
        &db,
        "actor_projection",
        &[("record_id", 1.into()), ("items_state", text("value"))],
    );
    row(
        &db,
        "actor_items",
        &[
            ("id", 10.into()),
            ("record_id", 1.into()),
            ("original_index", 0.into()),
            ("owner_selector_json", text("{}")),
            ("source_type_state", text("invalid")),
        ],
    );
    assert!(result(&db, "embedded.spells.exists(s,true)").is_empty());
    assert!(result(&db, "!embedded.spells.exists(s,true)").is_empty());
    row(
        &db,
        "actor_items",
        &[
            ("id", 11.into()),
            ("record_id", 1.into()),
            ("original_index", 1.into()),
            ("owner_selector_json", text("{}")),
            ("source_type_state", text("value")),
            ("source_type", text("spell")),
        ],
    );
    assert_eq!(result(&db, "embedded.spells.exists(s,true)"), vec![1]);
}

#[test]
fn consumable_damage_fields_share_root_and_embedded_semantics() {
    let db = database();
    root(&db, 1, "consumable", "common");
    root(&db, 2, "npc", "common");
    root(&db, 3, "consumable", "common");
    row(
        &db,
        "actor_projection",
        &[("record_id", 2.into()), ("items_state", text("value"))],
    );
    row(
        &db,
        "actor_items",
        &[
            ("id", 10.into()),
            ("record_id", 2.into()),
            ("original_index", 0.into()),
            ("owner_selector_json", text("{}")),
            ("source_type_state", text("value")),
            ("source_type", text("consumable")),
        ],
    );
    for (id, root_id, item_id) in [(1, 1.into(), Value::Null), (2, Value::Null, 10.into())] {
        row(
            &db,
            "physical_projection",
            &[
                ("id", id.into()),
                ("root_record_id", root_id),
                ("actor_item_id", item_id),
                ("consumable_damage_kind_state", text("value")),
                ("consumable_damage_kind", text("healing")),
                ("consumable_damage_type_state", text("value")),
                ("consumable_damage_type", text("vitality")),
            ],
        );
    }
    row(
        &db,
        "physical_projection",
        &[
            ("id", 3.into()),
            ("root_record_id", 3.into()),
            ("consumable_damage_kind_state", text("null")),
            ("consumable_damage_type_state", text("missing")),
        ],
    );
    assert_eq!(
        result(
            &db,
            "consumable.damage.kind == 'healing' && consumable.damage.type == 'vitality'"
        ),
        vec![1]
    );
    assert_eq!(
        result(
            &db,
            "embedded.equipment.exists(e, e.consumable.damage.kind == 'healing' && e.consumable.damage.type == 'vitality')"
        ),
        vec![2]
    );
    assert_eq!(
        result(&db, "!(consumable.damage.kind == 'damage')"),
        vec![1]
    );
    assert_eq!(
        result(&db, "availability['consumable.damage.kind'] == 'null'"),
        vec![3]
    );
}

#[test]
fn reference_numeric_filters_use_authored_root_and_embedded_values() {
    let db = database();
    let mut ids = crate::projections::ProjectionIds::default();
    let armor = json!({"_id":"armor00000000000","type":"armor","system":{"baseItem":"chain-mail","strength":3,"checkPenalty":-2,"speedPenalty":-5}});
    let fixtures = vec![
        (
            "Actor",
            json!({"_id":"npc0000000000000","type":"npc","system":{"abilities":{"str":{"mod":0}},"skills":{"athletics":{"base":20}}}}),
        ),
        (
            "Actor",
            json!({"_id":"char000000000000","type":"character","system":{"abilities":{"str":{"mod":5}},"skills":{"athletics":{"rank":3}}}}),
        ),
        ("Item", armor.clone()),
        (
            "Item",
            json!({"_id":"weapon0000000000","type":"weapon","system":{"damage":{"dice":2,"die":"d8"}}}),
        ),
        (
            "Item",
            json!({"_id":"shield0000000000","type":"shield","system":{"acBonus":2,"hp":{"max":25}}}),
        ),
        (
            "Actor",
            json!({"_id":"hazard0000000000","type":"hazard","system":{"attributes":{"stealth":{"value":-1}}}}),
        ),
        (
            "Actor",
            json!({"_id":"carrier000000000","type":"npc","items":[armor]}),
        ),
        (
            "Actor",
            json!({"_id":"null000000000000","type":"npc","system":{"skills":{"athletics":{"base":null}}}}),
        ),
        (
            "Actor",
            json!({"_id":"invalid000000000","type":"npc","system":{"skills":{"athletics":{"base":"bad"}}}}),
        ),
        (
            "Item",
            json!({"_id":"bomb000000000000","type":"consumable","system":{"baseItem":"alchemical-bomb"}}),
        ),
        (
            "Item",
            json!({"_id":"bombweapon000000","type":"weapon","system":{"baseItem":"alchemical-bomb"}}),
        ),
        (
            "Item",
            json!({"_id":"nullbase00000000","type":"consumable","system":{"baseItem":null}}),
        ),
        (
            "Item",
            json!({"_id":"badbase000000000","type":"consumable","system":{"baseItem":3}}),
        ),
    ];
    for (index, (document, raw)) in fixtures.into_iter().enumerate() {
        let input = crate::foundation_tests::record("test", document, raw);
        db.execute(
            "INSERT INTO record_identity(id,key,availability) VALUES(?,?,'available')",
            rusqlite::params![index as i64 + 1, input.record.key().to_string()],
        )
        .unwrap();
        for row in
            crate::projections::root_rows(&input, "Test", index as i64 + 1, &mut ids).unwrap()
        {
            row.insert(&db).unwrap();
        }
    }
    assert_eq!(result(&db, "skills.athletics.modifier >= 20"), vec![1]);
    assert_eq!(result(&db, "skills.athletics.rank == 3"), vec![2]);
    assert_eq!(result(&db, "abilities.str.modifier == 0"), vec![1]);
    assert_eq!(result(&db, "abilities.str.modifier == 5"), vec![2]);
    assert_eq!(
        result(
            &db,
            "armor.strength == 3 && armor.check_penalty == -2 && armor.speed_penalty == -5"
        ),
        vec![3]
    );
    assert_eq!(
        result(
            &db,
            "embedded.equipment.exists(i, i.armor.check_penalty == -2 && i.armor.strength == 3)"
        ),
        vec![7]
    );
    assert_eq!(
        result(
            &db,
            "weapon.damage.dice == 2 && weapon.damage.die_faces == 8"
        ),
        vec![4]
    );
    assert_eq!(
        result(&db, "shield.ac_bonus == 2 && shield.broken_threshold == 12"),
        vec![5]
    );
    assert_eq!(result(&db, "hazard.stealth == -1"), vec![6]);
    assert_eq!(
        result(&db, "physical.base_type == 'alchemical-bomb'"),
        vec![10, 11]
    );
    assert_eq!(result(&db, "physical.base_type == 'chain-mail'"), vec![3]);
    assert_eq!(
        result(
            &db,
            "embedded.equipment.exists(i, i.physical.base_type == 'chain-mail')"
        ),
        vec![7]
    );
    assert_eq!(
        result(&db, "availability['physical.base_type'] == 'null'"),
        vec![12]
    );
    assert_eq!(
        result(&db, "availability['physical.base_type'] == 'invalid'"),
        vec![13]
    );
    assert_eq!(
        result(&db, "availability['skills.athletics.modifier'] == 'null'"),
        vec![8]
    );
    assert_eq!(
        result(
            &db,
            "availability['skills.athletics.modifier'] == 'invalid'"
        ),
        vec![9]
    );
    assert_eq!(
        result(&db, "skills.athletics.modifier != 20"),
        Vec::<i64>::new()
    );
}
