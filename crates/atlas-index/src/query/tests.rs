use super::*;
use atlas_domain::query::*;
use rusqlite::{Connection, params_from_iter, types::Value};
use serde_json::json;

fn database() -> Connection {
    let db = Connection::open_in_memory().unwrap();
    db.execute_batch(include_str!(
        "../../migrations/00000000000001_create_artifact/up.sql"
    ))
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
    let input = json!({"clause_id":"parent","kind":"exists","collection":"actor.items","scope_id":"spell-group","predicate":{"clause_id":"rank","kind":"compare","field":"spell.rank","op":"gte","value":3}});
    let p: QueryPredicate = serde_json::from_value(input.clone()).unwrap();
    assert_eq!(serde_json::to_value(p).unwrap(), input);
    for invalid in [
        json!({"kind":"metric","key":"hp.max","value":3}),
        json!({"kind":"compare","field":"actor.level","op":"eq","value":3,"old_metric":"x"}),
    ] {
        assert!(serde_json::from_value::<QueryPredicate>(invalid).is_err());
    }
}
#[test]
fn cel_supported_examples_and_fail_closed_limits() {
    for expression in [
        "actor.hp.maximum>=80",
        "source.pack.id in ['spells-srd','equipment-srd']",
        "'fire' in traits && rarity != 'common'",
        "hazard.complexity==false && hazard.hardness>=10",
        "spell.rank >= 3 && 'arcane' in spell.traditions",
        "actor.items.exists(i, i.source.type=='spell' && i.spell.rank>=3 && 'arcane' in i.spell.traditions)",
        "!actor.items.exists(i,i.availability['spell.rank']=='invalid')",
        "availability['actor.hp.maximum']=='null'",
        "actor.level >= -1",
        "actor.level >= -9223372036854775808",
        "1 <= actor.level",
    ] {
        assert!(
            parse_where(expression).is_ok(),
            "{expression}: {:?}",
            parse_where(expression)
        );
    }
    for expression in [
        "actor.items[0].spell.rank>=3",
        "actor.items.all(i,i.spell.rank>=3)",
        "actor.items.exists(i,i.spell.damage.exists(d,d.type=='fire'))",
        "actor.level==spell.rank",
        "actor.level+1>=3",
        "traits.matches('x')",
        "has(actor.hp.maximum)",
        "actor.hp.maximum==null",
        "availability[source.type]=='value'",
        "unknown_field==1",
        "rarity=='made-up'",
        "spell.traditions=='arcane'",
        "actor.items.exists(i,actor.level>1)",
        "actor.level==18446744073709551615u",
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
            collection: "actor.items".to_owned(),
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
        "actor.level",
        "item.level",
        "actor.size",
        "physical.size",
        "ancestry.size",
        "actor.speeds[].value",
        "actor.resistances[].type",
        "spell.damage[].kinds",
        "actor.items[].spell.rank",
        "actor.items[].condition.is_valued",
        "actor.items[].deity.domains.alternate",
        "effect.duration.hours",
    ] {
        assert!(ids.contains(&id.to_owned()), "{id}");
    }
    assert!(!ids.contains(&"actor.items[].spell.damage".to_owned()));
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
        for expression in ["actor.level>=3", "!(actor.level<3)", "actor.level!=2"] {
            assert_eq!(
                result(&db, expression),
                if state == "value" { vec![1] } else { vec![] },
                "{state} {expression}"
            );
        }
        assert_eq!(
            result(&db, &format!("availability['actor.level']=='{state}'")),
            vec![1]
        );
        assert_eq!(result(&db, "actor.level>=3 || true"), vec![1]);
        assert_eq!(result(&db, "actor.level>=3 && false"), Vec::<i64>::new());
    }
    db.execute("UPDATE records SET level_state='invalid',level=NULL", [])
        .unwrap();
    for (a, av) in [
        ("true", Some(true)),
        ("false", Some(false)),
        ("actor.level>=3", None),
    ] {
        for (b, bv) in [
            ("true", Some(true)),
            ("false", Some(false)),
            ("actor.level>=3", None),
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
    let yes = "actor.resistances.exists(r,r.type=='fire' && r.value>=5)";
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
    assert_eq!(result(&db, "!actor.resistances.exists(r,true)"), vec![1]);
    for state in ["missing", "null", "invalid", "not_applicable"] {
        db.execute("UPDATE actor_projection SET resistances_state=?1", [state])
            .unwrap();
        assert!(result(&db, "actor.resistances.exists(r,true)").is_empty());
        assert!(result(&db, "!actor.resistances.exists(r,true)").is_empty());
    }
}
#[test]
fn numeric_boundaries_and_unit_gates_preserve_exact_domains() {
    let db = database();
    root(&db, 1, "npc", "common");
    db.execute("UPDATE records SET level=9223372036854775807", [])
        .unwrap();
    assert_eq!(result(&db, "actor.level == 9223372036854775807"), vec![1]);
    assert!(parse_where("actor.level==18446744073709551615u").is_err());
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
    let options = field_values(&db, &request).unwrap();
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
    request.offset = 100;
    request.limit = 1;
    let retained = field_values(&db, &request).unwrap();
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
    let options = field_values(&db, &request).unwrap();
    assert_eq!(count(&options, "rare"), 1);
    assert_eq!(count(&options, "common"), 0);
    assert!(!options.exhaustive);
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
    let input = json!({"kind":"exists","collection":"actor.items","scope_id":"spell-scope","predicate":{"kind":"all_of","children":[{"clause_id":"rank-choice","kind":"compare","field":"spell.rank","op":"gte","value":3},{"clause_id":"tradition-choice","kind":"set_match","field":"spell.traditions","op":"includes","values":["arcane"]}]}});
    let query = validate_query(&serde_json::from_value(input).unwrap()).unwrap();
    let request = QueryValuesRequest {
        field: "actor.items[].spell.traditions".to_owned(),
        clause_id: Some("tradition-choice".to_owned()),
        query,
        context: QueryFacetContext::default(),
        text: None,
        offset: 0,
        limit: 100,
    };
    let options = field_values(&db, &request).unwrap();
    let numeric = crate::discovery::field_counts(
        &db,
        &crate::discovery::QueryCountsRequest {
            field: "actor.items[].spell.rank".to_owned(),
            clause_id: Some("rank-choice".to_owned()),
            query: request.query.clone(),
            context: request.context.clone(),
        },
    )
    .unwrap();
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
    let mut bad = request.clone();
    let mut p = request.query.predicate().clone();
    p = QueryPredicate::new(QueryExpression::Not {
        predicate: Box::new(p),
    });
    bad.query = validate_query(&p).unwrap();
    assert!(
        matches!(field_values(&db,&bad),Err(crate::IndexError::Query(e)) if e.code=="unsupported_facet_context")
    );
}

#[test]
fn exclusion_or_and_negated_scope_facets_fail_explicitly() {
    use crate::discovery::{QueryFacetContext, QueryValuesRequest, field_values};
    let db = database();
    root(&db, 1, "npc", "rare");
    for expression in [
        json!({"clause_id":"target","kind":"compare","field":"rarity","op":"neq","value":"rare"}),
        json!({"kind":"any_of","children":[{"clause_id":"target","kind":"compare","field":"rarity","op":"eq","value":"rare"},{"kind":"boolean_constant","value":true}]}),
        json!({"clause_id":"target","kind":"set_match","field":"traits","op":"excludes_any","values":["fire"]}),
    ] {
        let predicate: QueryPredicate = serde_json::from_value(expression).unwrap();
        let field = match &predicate.expression {
            QueryExpression::SetMatch { .. } => "traits",
            _ => "rarity",
        };
        let request = QueryValuesRequest {
            field: field.to_owned(),
            clause_id: Some("target".to_owned()),
            query: validate_query(&predicate).unwrap(),
            context: QueryFacetContext::default(),
            text: None,
            offset: 0,
            limit: 10,
        };
        assert!(
            matches!(field_values(&db,&request),Err(crate::IndexError::Query(e)) if e.code=="unsupported_facet_context")
        );
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
    let counts = field_counts(
        &db,
        &QueryCountsRequest {
            field: "actor.level".to_owned(),
            clause_id: None,
            query: parse_where("true").unwrap(),
            context: QueryFacetContext::default(),
        },
    )
    .unwrap();
    assert_eq!(counts.counting_scope, "root");
    assert_eq!(counts.minimum, Some(0.into()));
    assert_eq!(counts.maximum, Some(0.into()));
    assert!(
        counts
            .states
            .iter()
            .all(|s| s.occurrences == 1 && s.distinct_roots == 1)
    );
}
