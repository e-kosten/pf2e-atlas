use std::path::Path;
use std::time::{Duration, Instant};

use lbug::{Connection, Database, SystemConfig};
use rusqlite::Connection as SqliteConnection;

struct GraphValueCase {
    name: &'static str,
    question: &'static str,
    read: &'static str,
    ladybug_query: &'static str,
    sqlite_query: &'static str,
    comparison_note: &'static str,
}

struct QueryOutput {
    rows: Vec<String>,
    compile_ms: Option<f64>,
    execute_ms: f64,
}

pub(crate) fn run_graph_value_eval(
    sqlite_path: &Path,
    ladybug_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let sqlite = SqliteConnection::open(sqlite_path)?;
    let database = Database::new(ladybug_path, SystemConfig::default())?;
    let ladybug = Connection::new(&database)?;

    println!("Graph product-value evaluation");
    println!("==============================");
    println!("sqlite:  {}", sqlite_path.display());
    println!("ladybug: {}", ladybug_path.display());
    println!();
    println!("This spike-only harness compares graph-shaped questions against Ladybug");
    println!("Cypher and equivalent SQLite SQL where the current SQLite artifact has");
    println!("the data. It is meant to test product value, not product CLI code.");

    for case in graph_value_cases() {
        print_case(case, &ladybug, &sqlite)?;
    }
    print_multihop_discovery(&ladybug)?;

    println!();
    println!("Decision read");
    println!("-------------");
    println!("Relationship overlap, mechanic neighborhoods, variants, remaster links,");
    println!("and broad backlinks are graph-shaped but SQL-expressible with normal join");
    println!("tables. The strongest Ladybug idea was evidence-level reference");
    println!("provenance; the SQLite artifact now has reference_occurrences so that");
    println!("same-content reference questions can be evaluated without duplicating");
    println!("content text or adopting a general graph database.");

    Ok(())
}

fn print_multihop_discovery(ladybug: &Connection<'_>) -> Result<(), Box<dyn std::error::Error>> {
    println!();
    println!("Multi-hop discovery");
    println!("-------------------");
    println!("These are Ladybug probes over Record -> EvidenceUnit -> Record paths.");
    println!("The comparable SQLite shape is reference_occurrences self-joined by");
    println!("record_key/content_key, with content hydrated from records or");
    println!("record_content only when snippets are needed.");

    print_ladybug_discovery(
        ladybug,
        "highest-support same-evidence rule pairs",
        "MATCH (source:Record)-[:HAS_EVIDENCE_UNIT]->(evidence:EvidenceUnit)-[:EVIDENCE_REFERENCES]->(left:Record),
               (evidence)-[:EVIDENCE_REFERENCES]->(right:Record)
         WHERE source.is_default_visible
           AND left.record_key < right.record_key
           AND left.record_family = 'rule'
           AND right.record_family = 'rule'
           AND evidence.source_kind <> 'embedded_item_description'
         WITH left, right, count(DISTINCT evidence) AS evidence_units, count(DISTINCT source) AS records
         WHERE evidence_units >= 3
         RETURN left.record_key, left.name, right.record_key, right.name, evidence_units, records
         ORDER BY evidence_units DESC, records DESC, left.name, right.name
         LIMIT 20;",
        "Finds mechanic pairs repeatedly co-located inside the same authored evidence unit. This is useful for surfacing rule interactions, but broad condition pairs can still be noisy.",
    )?;

    print_ladybug_discovery(
        ladybug,
        "named ability sections that bridge two mechanics",
        "MATCH (source:Record)-[:HAS_EVIDENCE_UNIT]->(evidence:EvidenceUnit)-[:EVIDENCE_REFERENCES]->(left:Record),
               (evidence)-[:EVIDENCE_REFERENCES]->(right:Record)
         WHERE source.is_default_visible
           AND left.record_key < right.record_key
           AND evidence.label IS NOT NULL
           AND evidence.label <> ''
           AND evidence.label <> 'Description'
           AND evidence.source_kind <> 'embedded_item_description'
           AND left.record_family = 'rule'
           AND right.record_family = 'rule'
         RETURN source.record_key, source.name, source.record_family, evidence.label, evidence.source_kind,
                left.name, right.name
         ORDER BY source.record_family, source.name, evidence.ordinal, left.name, right.name
         LIMIT 40;",
        "Good product shape: a UI can say not just that a record touches two mechanics, but which named action/ability/effect section creates the interaction.",
    )?;

    print_ladybug_discovery(
        ladybug,
        "Demoralize plus Frightened same-evidence examples",
        "MATCH (source:Record)-[:HAS_EVIDENCE_UNIT]->(evidence:EvidenceUnit)-[:EVIDENCE_REFERENCES]->(:Record {record_key: 'actionspf2e:2u915NdUyQan6uKF'}),
               (evidence)-[:EVIDENCE_REFERENCES]->(:Record {record_key: 'conditionitems:TBSHQspnbcqxsmjL'})
         WHERE source.is_default_visible
         RETURN source.record_key, source.name, source.record_family, evidence.label, evidence.source_kind, evidence.search_text
         ORDER BY source.record_family, source.name, evidence.ordinal
         LIMIT 20;",
        "Concrete user-facing interaction: records whose exact text section connects Demoralize to Frightened. Current SQLite can overmatch at record level here.",
    )?;

    print_ladybug_discovery(
        ladybug,
        "Treat Wounds plus immunity same-evidence examples",
        "MATCH (source:Record)-[:HAS_EVIDENCE_UNIT]->(evidence:EvidenceUnit)-[:EVIDENCE_REFERENCES]->(:Record {record_key: 'actionspf2e:1kGNdIIhuglAjIp9'}),
               (evidence)-[:EVIDENCE_REFERENCES]->(target:Record)
         WHERE source.is_default_visible
           AND target.name CONTAINS 'Immunity'
         RETURN source.record_key, source.name, source.record_family, evidence.label, evidence.source_kind, target.name, evidence.search_text
         ORDER BY source.record_family, source.name, evidence.ordinal
         LIMIT 20;",
        "Tests whether evidence traversal can find healing cooldown/immunity interactions at the section level. If this returns coherent rows, it is a strong browse/explainability surface.",
    )?;

    print_ladybug_discovery(
        ladybug,
        "Frightened same-section expansion with evidence labels",
        "MATCH (source:Record)-[:HAS_EVIDENCE_UNIT]->(evidence:EvidenceUnit)-[:EVIDENCE_REFERENCES]->(:Record {record_key: 'conditionitems:TBSHQspnbcqxsmjL'}),
               (evidence)-[:EVIDENCE_REFERENCES]->(other:Record)
         WHERE source.is_default_visible
           AND other.record_key <> 'conditionitems:TBSHQspnbcqxsmjL'
           AND evidence.label IS NOT NULL
           AND evidence.label <> ''
           AND other.record_family = 'rule'
         RETURN source.record_key, source.name, source.record_family, evidence.label, other.name
         ORDER BY source.record_family, source.name, evidence.ordinal, other.name
         LIMIT 40;",
        "This is richer than a backlink list: every row has a concrete evidence label explaining why Frightened is connected to another mechanic.",
    )?;

    Ok(())
}

fn print_ladybug_discovery(
    connection: &Connection<'_>,
    label: &str,
    query: &str,
    read: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!();
    println!("### {label}");
    println!("read: {read}");
    let output = run_ladybug_query(connection, query)?;
    print_output("ladybug", &output);
    Ok(())
}

fn graph_value_cases() -> &'static [GraphValueCase] {
    &[
        GraphValueCase {
            name: "same-evidence mechanic bridge",
            question: "Which specific ability/text unit connects Demoralize and Frightened?",
            read: "Best candidate for graph-specific product value: it returns evidence-level context, not just matching records.",
            ladybug_query: "MATCH (record:Record)-[:HAS_EVIDENCE_UNIT]->(evidence:EvidenceUnit)-[:EVIDENCE_REFERENCES]->(:Record {record_key: 'actionspf2e:2u915NdUyQan6uKF'}),
                                  (evidence)-[:EVIDENCE_REFERENCES]->(:Record {record_key: 'conditionitems:TBSHQspnbcqxsmjL'})
                           WHERE record.is_default_visible
                           RETURN record.record_key, record.name, record.record_family, evidence.label, evidence.source_kind, evidence.unit_kind
                           ORDER BY record.record_family, record.name, evidence.ordinal
                           LIMIT 20;",
            sqlite_query: "SELECT r.record_key, r.name, r.record_family, demoralize.content_key, demoralize.display_text, frightened.display_text
                           FROM records r
                           JOIN reference_occurrences demoralize
                             ON demoralize.record_key = r.record_key
                            AND demoralize.target_record_key = 'actionspf2e:2u915NdUyQan6uKF'
                           JOIN reference_occurrences frightened
                             ON frightened.record_key = r.record_key
                            AND frightened.content_key = demoralize.content_key
                            AND frightened.target_record_key = 'conditionitems:TBSHQspnbcqxsmjL'
                           WHERE r.is_default_visible = 1
                           ORDER BY r.record_family, r.name
                           LIMIT 20;",
            comparison_note: "With reference_occurrences, SQLite can now prove both references came from the same logical content unit without storing duplicate content text.",
        },
        GraphValueCase {
            name: "relationship-ranked more-like-this",
            question: "Which records are mechanically similar to Fear by shared traits and shared referenced rules?",
            read: "Useful, but this is mostly a relationship-overlap score; SQL can express it cleanly with trait/reference join tables.",
            ladybug_query: "MATCH (seed:Record {record_key: 'spells-srd:4koZzrnMXhhosn0D'})-[:HAS_TRAIT]->(trait:Trait)<-[:HAS_TRAIT]-(other:Record),
                                  (seed)-[:REFERENCES]->(target:Record)<-[:REFERENCES]-(other)
                           WHERE other.record_key <> seed.record_key AND other.is_default_visible
                           WITH other, count(DISTINCT trait) AS shared_traits, count(DISTINCT target) AS shared_references
                           RETURN other.record_key, other.name, other.record_family, shared_traits, shared_references, shared_traits * 2 + shared_references * 5 AS graph_score
                           ORDER BY graph_score DESC, shared_references DESC, shared_traits DESC, other.name
                           LIMIT 20;",
            sqlite_query: "WITH
                             seed_traits AS (
                               SELECT trait FROM record_traits WHERE record_key = 'spells-srd:4koZzrnMXhhosn0D'
                             ),
                             seed_refs AS (
                               SELECT to_record_key FROM reference_edges WHERE from_record_key = 'spells-srd:4koZzrnMXhhosn0D'
                             ),
                             trait_counts AS (
                               SELECT rt.record_key, count(DISTINCT rt.trait) AS shared_traits
                               FROM record_traits rt
                               JOIN seed_traits st ON st.trait = rt.trait
                               WHERE rt.record_key <> 'spells-srd:4koZzrnMXhhosn0D'
                               GROUP BY rt.record_key
                             ),
                             ref_counts AS (
                               SELECT edge.from_record_key AS record_key, count(DISTINCT edge.to_record_key) AS shared_references
                               FROM reference_edges edge
                               JOIN seed_refs sr ON sr.to_record_key = edge.to_record_key
                               WHERE edge.from_record_key <> 'spells-srd:4koZzrnMXhhosn0D'
                               GROUP BY edge.from_record_key
                             )
                           SELECT r.record_key, r.name, r.record_family, tc.shared_traits, rc.shared_references,
                                  tc.shared_traits * 2 + rc.shared_references * 5 AS graph_score
                           FROM records r
                           JOIN trait_counts tc ON tc.record_key = r.record_key
                           JOIN ref_counts rc ON rc.record_key = r.record_key
                           WHERE r.is_default_visible = 1
                           ORDER BY graph_score DESC, rc.shared_references DESC, tc.shared_traits DESC, r.name
                           LIMIT 20;",
            comparison_note: "If this is valuable, it argues for a relationship-overlap feature, not necessarily a graph database.",
        },
        GraphValueCase {
            name: "mechanic neighborhood",
            question: "What other rule records are commonly co-referenced by records that reference Frightened?",
            read: "Good for browse/ecology pages, but it is a count over two reference-edge joins.",
            ladybug_query: "MATCH (source:Record)-[:REFERENCES]->(:Record {record_key: 'conditionitems:TBSHQspnbcqxsmjL'}),
                                  (source)-[:REFERENCES]->(other:Record)
                           WHERE source.is_default_visible
                             AND other.record_key <> 'conditionitems:TBSHQspnbcqxsmjL'
                             AND other.record_family = 'rule'
                           WITH other, count(DISTINCT source) AS support
                           RETURN other.record_key, other.name, support
                           ORDER BY support DESC, other.name
                           LIMIT 20;",
            sqlite_query: "SELECT other.record_key, other.name, count(DISTINCT source.record_key) AS support
                           FROM reference_edges frightened
                           JOIN records source ON source.record_key = frightened.from_record_key
                           JOIN reference_edges neighbor ON neighbor.from_record_key = source.record_key
                           JOIN records other ON other.record_key = neighbor.to_record_key
                           WHERE frightened.to_record_key = 'conditionitems:TBSHQspnbcqxsmjL'
                             AND source.is_default_visible = 1
                             AND other.record_key <> 'conditionitems:TBSHQspnbcqxsmjL'
                             AND other.record_family = 'rule'
                           GROUP BY other.record_key, other.name
                           ORDER BY support DESC, other.name
                           LIMIT 20;",
            comparison_note: "This is graph-shaped navigation, but not a Ladybug-only capability.",
        },
        GraphValueCase {
            name: "trait plus mechanic bridge",
            question: "Which fire-trait records reference persistent damage?",
            read: "Useful structured graph intersection, but simple SQL over record_traits and reference_edges.",
            ladybug_query: "MATCH (record:Record)-[:HAS_TRAIT]->(:Trait {name: 'fire'}),
                                  (record)-[edge:REFERENCES]->(target:Record {record_key: 'conditionitems:lDVqvLKA6eF3Df60'})
                           WHERE record.is_default_visible
                           RETURN record.record_key, record.name, record.record_family, edge.source_kind
                           ORDER BY record.record_family, record.name
                           LIMIT 20;",
            sqlite_query: "SELECT r.record_key, r.name, r.record_family, edge.source_kind
                           FROM records r
                           JOIN record_traits rt ON rt.record_key = r.record_key AND rt.trait = 'fire'
                           JOIN reference_edges edge
                             ON edge.from_record_key = r.record_key
                            AND edge.to_record_key = 'conditionitems:lDVqvLKA6eF3Df60'
                           WHERE r.is_default_visible = 1
                           ORDER BY r.record_family, r.name
                           LIMIT 20;",
            comparison_note: "This is a strong product query, but the relational model handles it directly.",
        },
        GraphValueCase {
            name: "variant progression",
            question: "What records are in the Dread Ampoule variant family?",
            read: "Good product surface, but variants are already materialized and easy for either backend.",
            ladybug_query: "MATCH (record:Record)-[rel:IN_VARIANT_GROUP]->(variant_group:VariantGroup {base_name: 'Dread Ampoule'})
                           WHERE record.is_default_visible
                           RETURN record.record_key, record.name, record.record_family, record.level, rel.variant_label
                           ORDER BY record.level, record.name;",
            sqlite_query: "SELECT record_key, name, record_family, level, variant_label
                           FROM records
                           WHERE is_default_visible = 1 AND variant_group_key = (
                             SELECT variant_group_key FROM records
                             WHERE variant_base_name = 'Dread Ampoule'
                             LIMIT 1
                           )
                           ORDER BY level, name;",
            comparison_note: "This is not enough by itself to justify Ladybug; SQLite already stores the grouping fields.",
        },
    ]
}

fn print_case(
    case: &GraphValueCase,
    ladybug: &Connection<'_>,
    sqlite: &SqliteConnection,
) -> Result<(), Box<dyn std::error::Error>> {
    println!();
    println!("## {}", case.name);
    println!("question: {}", case.question);
    println!("read: {}", case.read);
    println!();

    let ladybug_output = run_ladybug_query(ladybug, case.ladybug_query)?;
    print_output("ladybug", &ladybug_output);

    let sqlite_output = run_sqlite_query(sqlite, case.sqlite_query)?;
    print_output("sqlite", &sqlite_output);

    println!("comparison: {}", case.comparison_note);
    Ok(())
}

fn run_ladybug_query(
    connection: &Connection<'_>,
    query: &str,
) -> Result<QueryOutput, Box<dyn std::error::Error>> {
    let mut result = connection.query(query)?;
    let mut rows = Vec::new();
    for row in &mut result {
        rows.push(
            row.iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(" | "),
        );
    }
    Ok(QueryOutput {
        rows,
        compile_ms: Some(result.get_compiling_time()),
        execute_ms: result.get_execution_time(),
    })
}

fn run_sqlite_query(
    connection: &SqliteConnection,
    query: &str,
) -> Result<QueryOutput, Box<dyn std::error::Error>> {
    let started = Instant::now();
    let mut statement = connection.prepare(query)?;
    let column_count = statement.column_count();
    let mut rows = statement.query([])?;
    let mut output = Vec::new();
    while let Some(row) = rows.next()? {
        let mut values = Vec::with_capacity(column_count);
        for index in 0..column_count {
            values.push(sqlite_value(row, index)?);
        }
        output.push(values.join(" | "));
    }
    Ok(QueryOutput {
        rows: output,
        compile_ms: None,
        execute_ms: duration_ms(started.elapsed()),
    })
}

fn print_output(label: &str, output: &QueryOutput) {
    println!("-- {label}");
    for row in output.rows.iter().take(20) {
        println!("{row}");
    }
    if output.rows.is_empty() {
        println!("(no rows)");
    }
    match output.compile_ms {
        Some(compile_ms) => println!(
            "timing: compile={compile_ms:.3}ms execute={:.3}ms rows={}",
            output.execute_ms,
            output.rows.len()
        ),
        None => println!(
            "timing: execute={:.3}ms rows={}",
            output.execute_ms,
            output.rows.len()
        ),
    }
    println!();
}

fn sqlite_value(row: &rusqlite::Row<'_>, index: usize) -> rusqlite::Result<String> {
    let value = row.get_ref(index)?;
    Ok(match value {
        rusqlite::types::ValueRef::Null => "NULL".to_string(),
        rusqlite::types::ValueRef::Integer(value) => value.to_string(),
        rusqlite::types::ValueRef::Real(value) => value.to_string(),
        rusqlite::types::ValueRef::Text(value) => String::from_utf8_lossy(value).into_owned(),
        rusqlite::types::ValueRef::Blob(value) => format!("<{} bytes>", value.len()),
    })
}

fn duration_ms(duration: Duration) -> f64 {
    duration.as_secs_f64() * 1000.0
}
