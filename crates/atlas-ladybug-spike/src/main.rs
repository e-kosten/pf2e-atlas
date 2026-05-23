use std::fs;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;

use arrow_array::{
    ArrayRef, BooleanArray, FixedSizeListArray, Float32Array, Int64Array, RecordBatch, StringArray,
};
use arrow_schema::{DataType, Field, Schema};
use lbug::{Connection, Database, SystemConfig, Value};
use parquet::arrow::ArrowWriter;
use serde_json::Value as JsonValue;

mod atlas_cli;
mod cli_parity;
mod search_eval;

use atlas_cli::summarize_json;
use cli_parity::run_cli_parity;
use search_eval::run_search_eval;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let mode = args.next().unwrap_or_else(|| "in-memory".to_string());
    if mode == "bulk-parquet" {
        return run_bulk_parquet_probe();
    }
    if mode == "query-artifact" {
        let path = args
            .next()
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(".cache/ladybug-spike/bulk-scratch.lbug"));
        return run_artifact_query_probe(&path);
    }
    if mode == "graph-opportunities" {
        let path = args
            .next()
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(".cache/ladybug-spike/bulk-scratch.lbug"));
        return run_graph_opportunity_probe(&path);
    }
    if mode == "graph-relevance" {
        let path = args
            .next()
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(".cache/ladybug-spike/bulk-scratch.lbug"));
        return run_graph_relevance_probe(&path);
    }
    if mode == "graph-product-eval" {
        let path = args
            .next()
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(".cache/ladybug-spike/evidence-unit-scratch.lbug"));
        return run_graph_product_eval_probe(&path);
    }
    if mode == "baseline-parity" {
        let path = args
            .next()
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(".cache/ladybug-spike/evidence-unit-scratch.lbug"));
        return run_baseline_parity_probe(&path);
    }
    if mode == "hydration-audit" {
        let sqlite_path = args
            .next()
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(".cache/ladybug-spike/parity-noemb.sqlite"));
        let ladybug_path = args
            .next()
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(".cache/ladybug-spike/parity-noemb.lbug"));
        let atlas_bin = args
            .next()
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("target/debug/atlas"));
        return run_hydration_audit(&sqlite_path, &ladybug_path, &atlas_bin);
    }
    if mode == "vector-filter-parity" {
        let path = args
            .next()
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(".cache/ladybug-spike/with-embeddings.lbug"));
        return run_vector_filter_parity_probe(&path);
    }
    if mode == "search-eval" {
        let sqlite_path = args
            .next()
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(".cache/ladybug-spike/with-embeddings.sqlite"));
        let ladybug_path = args
            .next()
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(".cache/ladybug-spike/with-embeddings.lbug"));
        let atlas_bin = args
            .next()
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("target/debug/atlas"));
        return run_search_eval(&sqlite_path, &ladybug_path, &atlas_bin);
    }
    if mode == "cli-parity" {
        let sqlite_path = args
            .next()
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(".cache/ladybug-spike/with-embeddings.sqlite"));
        let ladybug_path = args
            .next()
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(".cache/ladybug-spike/with-embeddings.lbug"));
        let atlas_bin = args
            .next()
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("target/debug/atlas"));
        return run_cli_parity(&sqlite_path, &ladybug_path, &atlas_bin);
    }
    if mode == "fts-projection" {
        let path = args
            .next()
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(".cache/ladybug-spike/with-embeddings.lbug"));
        return run_fts_projection_probe(&path);
    }
    if mode == "fts-rrf-risk" {
        let path = args
            .next()
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(".cache/ladybug-spike/metric-metadata.lbug"));
        return run_fts_rrf_risk_probe(&path);
    }
    if let Some(mode) = OnDiskMode::parse(&mode) {
        return run_on_disk_probe(mode);
    }
    if let Some(mode) = TheoryMode::parse(&mode) {
        return run_theory_probe(mode);
    }

    let database = Database::in_memory(SystemConfig::default())?;
    let connection = Connection::new(&database)?;

    let extensions = load_extensions(&connection);
    create_schema(&connection)?;
    insert_fixture(&connection)?;
    let indexes = create_indexes(&connection, &extensions);

    println!("LadybugDB PF2e graph/search probe");
    println!("==================================");
    report_query(
        &connection,
        "record key lookup",
        "MATCH (r:Record {record_key: 'spell:fear'}) RETURN r.title, r.family, r.rank;",
    )?;
    report_query(
        &connection,
        "structured filter count",
        "MATCH (r:Record) WHERE r.family = 'spell' AND r.rank <= 3 AND r.is_default_visible RETURN count(r);",
    )?;
    report_query(
        &connection,
        "facet count under partial filter",
        "MATCH (r:Record)-[:HAS_TRAIT]->(t:Trait) WHERE r.family = 'spell' AND r.is_default_visible RETURN t.name, count(r) ORDER BY t.name;",
    )?;
    report_query(
        &connection,
        "graph neighborhood for frightened",
        "MATCH (r:Record)-[:REFERS_TO|APPLIES_CONCEPT]->(target) WHERE target.record_key = 'rule:frightened' OR target.name = 'frightened' RETURN r.record_key, r.title ORDER BY r.record_key;",
    )?;
    report_query(
        &connection,
        "path explanation",
        "MATCH p = (r:Record {record_key: 'spell:fear'})-[:REFERS_TO|APPLIES_CONCEPT]->(target) WHERE target.record_key = 'rule:frightened' OR target.name = 'frightened' RETURN p;",
    )?;
    if indexes.fts {
        report_query(
            &connection,
            "fts query with structured filter",
            "CALL QUERY_FTS_INDEX('Record', 'record_fts', 'frightened condition', top := 10) WITH node AS r, score WHERE r.family = 'spell' AND r.is_default_visible RETURN r.record_key, r.title, score ORDER BY score DESC;",
        )?;
    } else {
        println!();
        println!("-- fts query with structured filter");
        println!("skipped: FTS extension or index unavailable");
    }

    if indexes.vector {
        report_query(
            &connection,
            "vector query over all embedding units",
            "CALL QUERY_VECTOR_INDEX('EmbeddingUnit', 'unit_hnsw', CAST([0.98, 0.01, 0.01, 0.00], 'FLOAT[4]'), 4, efs := 50) WITH node AS u, distance MATCH (r:Record)-[:HAS_EMBEDDING]->(u) RETURN r.record_key, u.unit_key, distance ORDER BY distance;",
        )?;

        if extensions.algo {
            connection.query("CALL PROJECT_GRAPH('filtered_spell_units', {'EmbeddingUnit': 'n.parent_family = \\'spell\\' AND n.parent_rank <= 3 AND n.parent_default_visible = true'}, []);")?;
            report_query(
                &connection,
                "vector query over projected filtered graph",
                "CALL QUERY_VECTOR_INDEX('filtered_spell_units', 'unit_hnsw', CAST([0.98, 0.01, 0.01, 0.00], 'FLOAT[4]'), 4, efs := 50) WITH node AS u, distance MATCH (r:Record)-[:HAS_EMBEDDING]->(u) RETURN r.record_key, u.unit_key, distance ORDER BY distance;",
            )?;
        } else {
            println!();
            println!("-- vector query over projected filtered graph");
            println!("skipped: ALGO extension unavailable");
        }
    } else {
        println!();
        println!("-- vector query over all embedding units");
        println!("skipped: VECTOR extension or index unavailable");
        println!();
        println!("-- vector query over projected filtered graph");
        println!("skipped: VECTOR extension or index unavailable");
    }

    Ok(())
}

fn run_fts_projection_probe(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let database = Database::new(path, SystemConfig::default())?;
    let connection = Connection::new(&database)?;
    load_extension(&connection, "FTS");

    let _ = connection.query("CALL DROP_PROJECTED_GRAPH('eligible_spell_search_docs');");
    let projection = "MATCH (record:Record)-[:HAS_SEARCH_DOCUMENT]->(doc:SearchDocument) \
                      WHERE record.is_default_visible AND record.record_family = 'spell' \
                      RETURN doc";
    let create_query = format!(
        "CALL PROJECT_GRAPH_CYPHER('eligible_spell_search_docs', {});",
        cypher_string_literal(projection)
    );
    println!("-- create projected SearchDocument graph");
    match connection.query(&create_query) {
        Ok(mut rows) => {
            for row in &mut rows {
                println!("{row:?}");
            }
        }
        Err(error) => {
            println!("projection failed: {error}");
            return Ok(());
        }
    }

    println!("-- query FTS index on projected SearchDocument graph");
    match connection.query(
        "CALL QUERY_FTS_INDEX('eligible_spell_search_docs', 'search_document_fts', 'fire damage', top := 10)
         WITH node AS doc, score
         MATCH (record:Record)-[:HAS_SEARCH_DOCUMENT]->(doc)
         RETURN record.record_key, record.name, record.record_family, score
         ORDER BY score DESC
         LIMIT 10;",
    ) {
        Ok(mut rows) => for row in &mut rows {
            println!("{row:?}");
        },
        Err(error) => println!("projected FTS failed: {error}"),
    }

    println!("-- materialized filtered SearchDocument table");
    let _ = connection.query(
        "CALL DROP_FTS_INDEX('EligibleSpellSearchDocument', 'eligible_spell_search_document_fts');",
    );
    let _ = connection.query("DROP TABLE EligibleSpellSearchDocument;");
    match connection.query(
        "CREATE NODE TABLE EligibleSpellSearchDocument AS
         MATCH (record:Record)-[:HAS_SEARCH_DOCUMENT]->(doc:SearchDocument)
         WHERE record.is_default_visible AND record.record_family = 'spell'
         RETURN doc.search_doc_key AS search_doc_key,
                doc.record_key AS record_key,
                doc.title AS title,
                doc.aliases AS aliases,
                doc.traits AS traits,
                doc.taxonomy_terms AS taxonomy_terms,
                doc.constraint_terms AS constraint_terms,
                doc.mechanic_terms AS mechanic_terms,
                doc.source_terms AS source_terms,
                doc.metric_terms AS metric_terms,
                doc.headings AS headings,
                doc.body AS body,
                doc.facts AS facts,
                doc.reference_terms AS reference_terms,
                doc.embedded_content AS embedded_content;",
    ) {
        Ok(mut rows) => {
            for row in &mut rows {
                println!("{row:?}");
            }
        }
        Err(error) => {
            println!("materialized table failed: {error}");
            return Ok(());
        }
    }
    match connection.query(
        "CALL CREATE_FTS_INDEX(
           'EligibleSpellSearchDocument',
           'eligible_spell_search_document_fts',
           ['title', 'aliases', 'traits', 'taxonomy_terms', 'constraint_terms',
            'mechanic_terms', 'source_terms', 'metric_terms', 'headings', 'body',
            'facts', 'reference_terms', 'embedded_content'],
           stemmer := 'porter'
         );",
    ) {
        Ok(mut rows) => {
            for row in &mut rows {
                println!("{row:?}");
            }
        }
        Err(error) => {
            println!("materialized FTS index failed: {error}");
            return Ok(());
        }
    }
    println!("-- query FTS index on materialized filtered table");
    match connection.query(
        "CALL QUERY_FTS_INDEX('EligibleSpellSearchDocument', 'eligible_spell_search_document_fts', 'fire damage', top := 10)
         WITH node AS doc, score
         RETURN doc.record_key, doc.title, score
         ORDER BY score DESC
         LIMIT 10;",
    ) {
        Ok(mut rows) => for row in &mut rows {
            println!("{row:?}");
        },
        Err(error) => println!("materialized FTS failed: {error}"),
    }

    Ok(())
}

#[derive(Clone, Copy, Debug)]
struct OnDiskMode {
    fts: bool,
    vector: bool,
}

impl OnDiskMode {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "disk-fts" => Some(Self {
                fts: true,
                vector: false,
            }),
            "disk-vector" => Some(Self {
                fts: false,
                vector: true,
            }),
            "disk-both" => Some(Self {
                fts: true,
                vector: true,
            }),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum TheoryMode {
    SearchDocumentFts,
    SearchDocumentEmptyFts,
    Vector384,
    ManyVector384,
    EmptyVector,
    EmptyBoth,
    IngestShapeIndexes,
}

impl TheoryMode {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "theory-searchdoc-fts" => Some(Self::SearchDocumentFts),
            "theory-searchdoc-empty-fts" => Some(Self::SearchDocumentEmptyFts),
            "theory-vector-384" => Some(Self::Vector384),
            "theory-many-vector-384" => Some(Self::ManyVector384),
            "theory-empty-vector" => Some(Self::EmptyVector),
            "theory-empty-both" => Some(Self::EmptyBoth),
            "theory-ingest-shape-indexes" => Some(Self::IngestShapeIndexes),
            _ => None,
        }
    }
}

fn run_theory_probe(mode: TheoryMode) -> Result<(), Box<dyn std::error::Error>> {
    let path = PathBuf::from("target/ladybug-spike/theory.lbug");
    remove_ladybug_files(&path)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    {
        let database = Database::new(&path, SystemConfig::default())?;
        let connection = Connection::new(&database)?;
        load_extensions(&connection);
        match mode {
            TheoryMode::SearchDocumentFts => build_search_document_fts(&connection, false)?,
            TheoryMode::SearchDocumentEmptyFts => build_search_document_fts(&connection, true)?,
            TheoryMode::Vector384 => build_vector_384(&connection, 4)?,
            TheoryMode::ManyVector384 => build_vector_384(&connection, 128)?,
            TheoryMode::EmptyVector => build_vector_384(&connection, 0)?,
            TheoryMode::EmptyBoth => {
                build_search_document_fts(&connection, false)?;
                build_vector_384(&connection, 0)?;
            }
            TheoryMode::IngestShapeIndexes => build_ingest_shape_indexes(&connection)?,
        }
        connection.query("CHECKPOINT;")?;
    }

    println!("reopening {}", path.display());
    {
        let database = Database::new(&path, SystemConfig::default())?;
        let connection = Connection::new(&database)?;
        load_extensions(&connection);
        match mode {
            TheoryMode::SearchDocumentFts | TheoryMode::SearchDocumentEmptyFts => {
                report_query(
                    &connection,
                    "search document FTS after reopen",
                    "CALL QUERY_FTS_INDEX('SearchDocument', 'search_document_fts', 'frightened condition', top := 10) WITH node AS d, score RETURN d.search_doc_key, score ORDER BY score DESC;",
                )?;
            }
            TheoryMode::Vector384 | TheoryMode::ManyVector384 | TheoryMode::EmptyVector => {
                report_query(
                    &connection,
                    "384-d vector query after reopen",
                    &format!(
                        "CALL QUERY_VECTOR_INDEX('EmbeddingUnit', 'embedding_hnsw', CAST({}, 'FLOAT[384]'), 4, efs := 50) WITH node AS u, distance RETURN u.embedding_unit_key, distance ORDER BY distance;",
                        vector_literal(384, 0)
                    ),
                )?;
            }
            TheoryMode::EmptyBoth => {
                report_query(
                    &connection,
                    "empty-both FTS after reopen",
                    "CALL QUERY_FTS_INDEX('SearchDocument', 'search_document_fts', 'frightened condition', top := 10) WITH node AS d, score RETURN d.search_doc_key, score ORDER BY score DESC;",
                )?;
                report_query(
                    &connection,
                    "empty-both vector query after reopen",
                    &format!(
                        "CALL QUERY_VECTOR_INDEX('EmbeddingUnit', 'embedding_hnsw', CAST({}, 'FLOAT[384]'), 4, efs := 50) WITH node AS u, distance RETURN u.embedding_unit_key, distance ORDER BY distance;",
                        vector_literal(384, 0)
                    ),
                )?;
            }
            TheoryMode::IngestShapeIndexes => {
                report_query(
                    &connection,
                    "ingest-shaped FTS after reopen",
                    "CALL QUERY_FTS_INDEX('SearchDocument', 'search_document_fts', 'frightened condition', top := 10) WITH node AS d, score RETURN d.search_doc_key, score ORDER BY score DESC;",
                )?;
                report_query(
                    &connection,
                    "ingest-shaped vector after reopen",
                    "CALL QUERY_VECTOR_INDEX('EmbeddingUnit', 'embedding_hnsw', CAST([0.98, 0.01, 0.01, 0.00], 'FLOAT[4]'), 4, efs := 50) WITH node AS u, distance RETURN u.embedding_unit_key, distance ORDER BY distance;",
                )?;
            }
        }
    }
    remove_ladybug_files(&path)?;
    Ok(())
}

fn build_search_document_fts(
    connection: &Connection<'_>,
    empty_heavy: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    connection.query(
        "CREATE NODE TABLE SearchDocument(
            search_doc_key STRING,
            record_key STRING,
            title STRING,
            aliases STRING,
            traits STRING,
            taxonomy_terms STRING,
            constraint_terms STRING,
            mechanic_terms STRING,
            source_terms STRING,
            metric_terms STRING,
            headings STRING,
            body STRING,
            facts STRING,
            reference_terms STRING,
            embedded_content STRING,
            PRIMARY KEY(search_doc_key)
        );",
    )?;
    if empty_heavy {
        for index in 0..32 {
            connection.query(&format!(
                "CREATE (:SearchDocument {{
                    search_doc_key: 'doc:{index}',
                    record_key: 'record:{index}',
                    title: '{}',
                    aliases: '',
                    traits: '{}',
                    taxonomy_terms: '',
                    constraint_terms: '',
                    mechanic_terms: '',
                    source_terms: '',
                    metric_terms: '',
                    headings: '',
                    body: '{}',
                    facts: '',
                    reference_terms: '',
                    embedded_content: ''
                }});",
                if index == 0 { "Fear" } else { "" },
                if index % 2 == 0 { "fear mental" } else { "" },
                if index == 0 {
                    "frightened condition penalty"
                } else {
                    ""
                },
            ))?;
        }
    } else {
        for statement in [
            "CREATE (:SearchDocument {search_doc_key: 'spell:fear#fts', record_key: 'spell:fear', title: 'Fear', aliases: 'Fear spell', traits: 'fear mental', taxonomy_terms: 'spell rank 1', constraint_terms: '', mechanic_terms: 'frightened condition', source_terms: 'Player Core', metric_terms: '', headings: 'Effect', body: 'The target becomes frightened and may flee.', facts: 'rank 1 mental fear', reference_terms: 'frightened condition', embedded_content: ''});",
            "CREATE (:SearchDocument {search_doc_key: 'rule:frightened#fts', record_key: 'rule:frightened', title: 'Frightened', aliases: '', traits: 'condition fear mental', taxonomy_terms: 'rule condition', constraint_terms: '', mechanic_terms: 'penalty checks DCs', source_terms: 'Player Core', metric_terms: '', headings: '', body: 'Frightened is a condition that penalizes checks and DCs.', facts: '', reference_terms: '', embedded_content: ''});",
        ] {
            connection.query(statement)?;
        }
    }
    connection.query(
        "CALL CREATE_FTS_INDEX(
            'SearchDocument',
            'search_document_fts',
            [
                'title', 'aliases', 'traits', 'taxonomy_terms', 'constraint_terms',
                'mechanic_terms', 'source_terms', 'metric_terms', 'headings', 'body',
                'facts', 'reference_terms', 'embedded_content'
            ],
            stemmer := 'porter'
        );",
    )?;
    Ok(())
}

fn build_vector_384(
    connection: &Connection<'_>,
    rows: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    connection.query(
        "CREATE NODE TABLE EmbeddingUnit(
            embedding_unit_key STRING,
            record_key STRING,
            unit_kind STRING,
            embedding FLOAT[384],
            PRIMARY KEY(embedding_unit_key)
        );",
    )?;
    for index in 0..rows {
        connection.query(&format!(
            "CREATE (:EmbeddingUnit {{
                embedding_unit_key: 'unit:{index}',
                record_key: 'record:{index}',
                unit_kind: 'parent',
                embedding: CAST({}, 'FLOAT[384]')
            }});",
            vector_literal(384, index),
        ))?;
    }
    connection.query(
        "CALL CREATE_VECTOR_INDEX('EmbeddingUnit', 'embedding_hnsw', 'embedding', metric := 'cosine');",
    )?;
    Ok(())
}

fn build_ingest_shape_indexes(
    connection: &Connection<'_>,
) -> Result<(), Box<dyn std::error::Error>> {
    connection.query("CREATE NODE TABLE Record(record_key STRING, name STRING, record_family STRING, level INT64, rarity STRING, is_default_visible BOOL, PRIMARY KEY(record_key));")?;
    connection.query("CREATE NODE TABLE SearchDocument(search_doc_key STRING, record_key STRING, title STRING, aliases STRING, traits STRING, taxonomy_terms STRING, constraint_terms STRING, mechanic_terms STRING, source_terms STRING, metric_terms STRING, headings STRING, body STRING, facts STRING, reference_terms STRING, embedded_content STRING, PRIMARY KEY(search_doc_key));")?;
    connection.query("CREATE NODE TABLE EmbeddingUnit(embedding_unit_key STRING, record_key STRING, unit_kind STRING, label STRING, ordinal INT64, semantic_input_hash STRING, dimensions INT64, embedding FLOAT[4], PRIMARY KEY(embedding_unit_key));")?;
    connection.query("CREATE NODE TABLE Trait(name STRING, PRIMARY KEY(name));")?;
    connection.query("CREATE NODE TABLE Metric(metric_key_id STRING, metric_domain STRING, metric_key STRING, value_type STRING, PRIMARY KEY(metric_key_id));")?;
    connection.query("CREATE REL TABLE HAS_SEARCH_DOCUMENT(FROM Record TO SearchDocument);")?;
    connection.query("CREATE REL TABLE HAS_EMBEDDING_UNIT(FROM Record TO EmbeddingUnit);")?;
    connection.query("CREATE REL TABLE HAS_TRAIT(FROM Record TO Trait);")?;
    connection.query("CREATE REL TABLE HAS_METRIC(FROM Record TO Metric, number_value DOUBLE, text_value STRING, bool_value BOOL);")?;
    for statement in [
        "CREATE (:Record {record_key: 'spell:fear', name: 'Fear', record_family: 'spell', level: 1, rarity: 'common', is_default_visible: true});",
        "CREATE (:Record {record_key: 'rule:frightened', name: 'Frightened', record_family: 'rule', level: 0, rarity: 'common', is_default_visible: true});",
        "CREATE (:SearchDocument {search_doc_key: 'spell:fear#fts', record_key: 'spell:fear', title: 'Fear', aliases: '', traits: 'fear mental', taxonomy_terms: 'spell', constraint_terms: '', mechanic_terms: 'frightened condition', source_terms: 'Player Core', metric_terms: '', headings: '', body: 'The target becomes frightened.', facts: '', reference_terms: 'frightened', embedded_content: ''});",
        "CREATE (:SearchDocument {search_doc_key: 'rule:frightened#fts', record_key: 'rule:frightened', title: 'Frightened', aliases: '', traits: 'condition fear mental', taxonomy_terms: 'rule', constraint_terms: '', mechanic_terms: 'penalty checks DCs', source_terms: 'Player Core', metric_terms: '', headings: '', body: 'Frightened is a condition.', facts: '', reference_terms: '', embedded_content: ''});",
        "CREATE (:EmbeddingUnit {embedding_unit_key: 'spell:fear#parent', record_key: 'spell:fear', unit_kind: 'parent', label: '', ordinal: 0, semantic_input_hash: 'hash1', dimensions: 4, embedding: CAST([0.95, 0.05, 0.0, 0.0], 'FLOAT[4]')});",
        "CREATE (:EmbeddingUnit {embedding_unit_key: 'rule:frightened#parent', record_key: 'rule:frightened', unit_kind: 'parent', label: '', ordinal: 0, semantic_input_hash: 'hash2', dimensions: 4, embedding: CAST([1.0, 0.0, 0.0, 0.0], 'FLOAT[4]')});",
        "CREATE (:Trait {name: 'fear'});",
        "CREATE (:Metric {metric_key_id: 'actor:ac.value', metric_domain: 'actor', metric_key: 'ac.value', value_type: 'number'});",
        "MATCH (r:Record {record_key: 'spell:fear'}), (d:SearchDocument {search_doc_key: 'spell:fear#fts'}) CREATE (r)-[:HAS_SEARCH_DOCUMENT]->(d);",
        "MATCH (r:Record {record_key: 'rule:frightened'}), (d:SearchDocument {search_doc_key: 'rule:frightened#fts'}) CREATE (r)-[:HAS_SEARCH_DOCUMENT]->(d);",
        "MATCH (r:Record), (u:EmbeddingUnit) WHERE r.record_key = u.record_key CREATE (r)-[:HAS_EMBEDDING_UNIT]->(u);",
        "MATCH (r:Record {record_key: 'spell:fear'}), (t:Trait {name: 'fear'}) CREATE (r)-[:HAS_TRAIT]->(t);",
        "MATCH (r:Record {record_key: 'spell:fear'}), (m:Metric {metric_key_id: 'actor:ac.value'}) CREATE (r)-[:HAS_METRIC {number_value: 20.0, text_value: null, bool_value: null}]->(m);",
    ] {
        connection.query(statement)?;
    }
    connection.query(
        "CALL CREATE_FTS_INDEX(
            'SearchDocument',
            'search_document_fts',
            [
                'title', 'aliases', 'traits', 'taxonomy_terms', 'constraint_terms',
                'mechanic_terms', 'source_terms', 'metric_terms', 'headings', 'body',
                'facts', 'reference_terms', 'embedded_content'
            ],
            stemmer := 'porter'
        );",
    )?;
    connection.query(
        "CALL CREATE_VECTOR_INDEX('EmbeddingUnit', 'embedding_hnsw', 'embedding', metric := 'cosine');",
    )?;
    Ok(())
}

fn run_bulk_parquet_probe() -> Result<(), Box<dyn std::error::Error>> {
    let db_path = PathBuf::from("target/ladybug-spike/bulk-parquet.lbug");
    let staging_dir = PathBuf::from("target/ladybug-spike/bulk-parquet-staging");
    remove_ladybug_files(&db_path)?;
    if staging_dir.exists() {
        fs::remove_dir_all(&staging_dir)?;
    }
    fs::create_dir_all(&staging_dir)?;

    write_bulk_parquet_files(&staging_dir)?;

    {
        let database = Database::new(&db_path, SystemConfig::default())?;
        let connection = Connection::new(&database)?;
        let extensions = load_extensions(&connection);
        create_bulk_parquet_schema(&connection)?;

        for (table, file_name) in [
            ("Record", "record.parquet"),
            ("SearchDocument", "search_document.parquet"),
            ("EmbeddingUnit", "embedding_unit.parquet"),
            ("ContentUnit", "content_unit.parquet"),
            ("Metric", "metric.parquet"),
            ("HAS_SEARCH_DOCUMENT", "has_search_document.parquet"),
            ("HAS_EMBEDDING_UNIT", "has_embedding_unit.parquet"),
            ("HAS_CONTENT_UNIT", "has_content_unit.parquet"),
            ("HAS_METRIC", "has_metric.parquet"),
            ("REFERENCES", "references.parquet"),
        ] {
            copy_from_parquet(&connection, table, &staging_dir.join(file_name))?;
        }

        if extensions.fts {
            connection.query(
                "CALL CREATE_FTS_INDEX(
                    'SearchDocument',
                    'search_document_fts',
                    ['title', 'traits', 'body', 'reference_terms'],
                    stemmer := 'porter'
                );",
            )?;
        }
        if extensions.vector {
            connection.query(
                "CALL CREATE_VECTOR_INDEX('EmbeddingUnit', 'embedding_hnsw', 'embedding', metric := 'cosine');",
            )?;
        }
        connection.query("CHECKPOINT;")?;
    }

    println!("reopening {}", db_path.display());
    {
        let database = Database::new(&db_path, SystemConfig::default())?;
        let connection = Connection::new(&database)?;
        let extensions = load_extensions(&connection);
        report_query(
            &connection,
            "bulk parquet record count",
            "MATCH (r:Record) RETURN count(r);",
        )?;
        report_query(
            &connection,
            "bulk parquet content relationship count",
            "MATCH (:Record)-[rel:HAS_CONTENT_UNIT]->(:ContentUnit) RETURN count(rel);",
        )?;
        report_query(
            &connection,
            "bulk parquet metric relationship payload",
            "MATCH (r:Record)-[hm:HAS_METRIC]->(m:Metric) RETURN r.record_key, m.metric_key, hm.number_value, hm.text_value, hm.bool_value ORDER BY r.record_key;",
        )?;
        report_query(
            &connection,
            "bulk parquet reference edges",
            "MATCH (from:Record)-[ref:REFERENCES]->(to:Record) RETURN from.record_key, to.record_key, ref.source_kind, ref.visibility ORDER BY from.record_key;",
        )?;
        if extensions.fts {
            report_query(
                &connection,
                "bulk parquet FTS",
                "CALL QUERY_FTS_INDEX('SearchDocument', 'search_document_fts', 'frightened condition', top := 10) WITH node AS d, score RETURN d.search_doc_key, score ORDER BY score DESC;",
            )?;
        }
        if extensions.vector {
            report_query(
                &connection,
                "bulk parquet vector",
                "CALL QUERY_VECTOR_INDEX('EmbeddingUnit', 'embedding_hnsw', CAST([1.0, 0.0, 0.0, 0.0], 'FLOAT[4]'), 4, efs := 50) WITH node AS u, distance RETURN u.embedding_unit_key, distance ORDER BY distance;",
            )?;
        }
    }

    fs::remove_dir_all(&staging_dir)?;
    remove_ladybug_files(&db_path)?;
    Ok(())
}

fn run_artifact_query_probe(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let database = Database::new(path, SystemConfig::default())?;
    let connection = Connection::new(&database)?;
    let extensions = load_extensions(&connection);

    println!("LadybugDB full artifact query probe");
    println!("===================================");
    println!("artifact: {}", path.display());

    report_query(
        &connection,
        "record count",
        "MATCH (r:Record) RETURN count(r);",
    )?;
    report_query(
        &connection,
        "default-visible count",
        "MATCH (r:Record) WHERE r.is_default_visible RETURN count(r);",
    )?;
    report_query(
        &connection,
        "record key lookup: Fear",
        "MATCH (r:Record {record_key: 'spells-srd:4koZzrnMXhhosn0D'}) RETURN r.record_key, r.name, r.record_family, r.level, r.rarity;",
    )?;
    report_query(
        &connection,
        "structured filter count: common spells rank <= 3",
        "MATCH (r:Record) WHERE r.record_family = 'spell' AND r.level <= 3 AND r.rarity = 'common' AND r.is_default_visible RETURN count(r);",
    )?;
    report_query(
        &connection,
        "facet count under partial filter: traits for common spells rank <= 3",
        "MATCH (r:Record)-[:HAS_TRAIT]->(t:Trait)
         WHERE r.record_family = 'spell' AND r.level <= 3 AND r.rarity = 'common' AND r.is_default_visible
         RETURN t.name, count(r) ORDER BY count(r) DESC, t.name LIMIT 20;",
    )?;
    report_query(
        &connection,
        "metric range filter: AC >= 30",
        "MATCH (r:Record)-[hm:HAS_METRIC]->(m:Metric {metric_key: 'ac.value'})
         WHERE hm.number_value >= 30 AND r.is_default_visible
         RETURN count(r);",
    )?;
    report_query(
        &connection,
        "metric value sample: disable deception dc max",
        "MATCH (r:Record)-[hm:HAS_METRIC]->(m:Metric {metric_key: 'disable.deception.dc.max'})
         RETURN r.record_key, r.name, hm.number_value ORDER BY hm.number_value DESC, r.name LIMIT 10;",
    )?;
    report_query(
        &connection,
        "graph out-neighborhood: Fear references",
        "MATCH (from:Record {record_key: 'spells-srd:4koZzrnMXhhosn0D'})-[ref:REFERENCES]->(to:Record)
         RETURN to.record_key, to.name, ref.source_kind, ref.visibility ORDER BY to.name LIMIT 20;",
    )?;
    report_query(
        &connection,
        "graph backlinks: references to Frightened",
        "MATCH (from:Record)-[ref:REFERENCES]->(to:Record {record_key: 'conditionitems:TBSHQspnbcqxsmjL'})
         WHERE from.is_default_visible
         RETURN from.record_key, from.name, from.record_family, ref.source_kind ORDER BY from.name LIMIT 20;",
    )?;

    if extensions.fts {
        report_query(
            &connection,
            "FTS: frightened condition",
            "CALL QUERY_FTS_INDEX('SearchDocument', 'search_document_fts', 'frightened condition', top := 10)
             WITH node AS d, score
             MATCH (r:Record)-[:HAS_SEARCH_DOCUMENT]->(d)
             RETURN r.record_key, r.name, r.record_family, score ORDER BY score DESC;",
        )?;
        report_query(
            &connection,
            "FTS with structured filter: fire spell rank <= 3",
            "CALL QUERY_FTS_INDEX('SearchDocument', 'search_document_fts', 'fire damage', top := 50)
             WITH node AS d, score
             MATCH (r:Record)-[:HAS_SEARCH_DOCUMENT]->(d)
             WHERE r.record_family = 'spell' AND r.level <= 3 AND r.is_default_visible
             RETURN r.record_key, r.name, r.level, score ORDER BY score DESC LIMIT 10;",
        )?;
    } else {
        println!();
        println!("-- FTS queries");
        println!("skipped: FTS extension unavailable");
    }

    if extensions.vector {
        report_query(
            &connection,
            "vector self-query: copied legacy embedding",
            &format!(
                "CALL QUERY_VECTOR_INDEX('EmbeddingUnit', 'embedding_hnsw', CAST({}, 'FLOAT[384]'), 10, efs := 50)
                 WITH node AS u, distance
                 MATCH (r:Record)-[:HAS_EMBEDDING_UNIT]->(u)
                 RETURN r.record_key, r.name, u.unit_kind, distance ORDER BY distance LIMIT 10;",
                vector_literal(384, 0)
            ),
        )?;
        report_query(
            &connection,
            "vector with structured filter: creatures only",
            &format!(
                "CALL QUERY_VECTOR_INDEX('EmbeddingUnit', 'embedding_hnsw', CAST({}, 'FLOAT[384]'), 100, efs := 50)
                 WITH node AS u, distance
                 MATCH (r:Record)-[:HAS_EMBEDDING_UNIT]->(u)
                 WHERE r.record_family = 'creature' AND r.is_default_visible
                 RETURN r.record_key, r.name, distance ORDER BY distance LIMIT 10;",
                vector_literal(384, 0)
            ),
        )?;
    } else {
        println!();
        println!("-- vector queries");
        println!("skipped: VECTOR extension unavailable");
    }

    Ok(())
}

fn run_graph_opportunity_probe(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let database = Database::new(path, SystemConfig::default())?;
    let connection = Connection::new(&database)?;

    println!("LadybugDB graph opportunity probe");
    println!("==================================");
    println!("artifact: {}", path.display());

    report_query(
        &connection,
        "reference hubs: most-referenced default-visible records",
        "MATCH (source:Record)-[ref:REFERENCES]->(target:Record)
         WHERE source.is_default_visible AND target.is_default_visible
         RETURN target.record_key, target.name, target.record_family, count(source)
         ORDER BY count(source) DESC, target.name
         LIMIT 25;",
    )?;
    report_query(
        &connection,
        "Frightened impact by source family and edge kind",
        "MATCH (source:Record)-[ref:REFERENCES]->(target:Record {record_key: 'conditionitems:TBSHQspnbcqxsmjL'})
         WHERE source.is_default_visible
         RETURN source.record_family, ref.source_kind, count(source)
         ORDER BY count(source) DESC, source.record_family, ref.source_kind;",
    )?;
    report_query(
        &connection,
        "records that connect Demoralize and Frightened",
        "MATCH (source:Record)-[:REFERENCES]->(demoralize:Record {record_key: 'actionspf2e:2u915NdUyQan6uKF'}),
               (source)-[fear_ref:REFERENCES]->(frightened:Record {record_key: 'conditionitems:TBSHQspnbcqxsmjL'})
         WHERE source.is_default_visible
         RETURN source.record_key, source.name, source.record_family, fear_ref.source_kind
         ORDER BY source.record_family, source.name
         LIMIT 40;",
    )?;
    report_query(
        &connection,
        "conditions co-referenced with Frightened",
        "MATCH (source:Record)-[:REFERENCES]->(:Record {record_key: 'conditionitems:TBSHQspnbcqxsmjL'}),
               (source)-[:REFERENCES]->(other:Record)
         WHERE source.is_default_visible
           AND other.record_key <> 'conditionitems:TBSHQspnbcqxsmjL'
           AND other.record_family = 'rule'
         RETURN other.record_key, other.name, count(source)
         ORDER BY count(source) DESC, other.name
         LIMIT 30;",
    )?;
    report_query(
        &connection,
        "records bridging Fire trait and Persistent Damage references",
        "MATCH (source:Record)-[:HAS_TRAIT]->(:Trait {name: 'fire'}),
               (source)-[ref:REFERENCES]->(target:Record {record_key: 'conditionitems:lDVqvLKA6eF3Df60'})
         WHERE source.is_default_visible
         RETURN source.record_key, source.name, source.record_family, ref.source_kind
         ORDER BY source.record_family, source.name
         LIMIT 40;",
    )?;
    report_query(
        &connection,
        "shared-reference similarity for Fear",
        "MATCH (fear:Record {record_key: 'spells-srd:4koZzrnMXhhosn0D'})-[:REFERENCES]->(target:Record),
               (other:Record)-[:REFERENCES]->(target)
         WHERE other.record_key <> fear.record_key AND other.is_default_visible
         RETURN other.record_key, other.name, other.record_family, count(target)
         ORDER BY count(target) DESC, other.name
         LIMIT 30;",
    )?;
    report_query(
        &connection,
        "shared-trait and shared-reference overlap for Fear",
        "MATCH (fear:Record {record_key: 'spells-srd:4koZzrnMXhhosn0D'})-[:HAS_TRAIT]->(trait:Trait),
               (other:Record)-[:HAS_TRAIT]->(trait),
               (fear)-[:REFERENCES]->(target:Record),
               (other)-[:REFERENCES]->(target)
         WHERE other.record_key <> fear.record_key AND other.is_default_visible
         RETURN other.record_key, other.name, other.record_family, count(DISTINCT trait), count(DISTINCT target)
         ORDER BY count(DISTINCT target) DESC, count(DISTINCT trait) DESC, other.name
         LIMIT 30;",
    )?;
    report_query(
        &connection,
        "source-kind mix for embedded-item graph edges",
        "MATCH (source:Record)-[ref:REFERENCES]->(target:Record)
         WHERE ref.source_kind = 'embedded_item_description'
         RETURN source.record_family, target.record_family, count(ref)
         ORDER BY count(ref) DESC, source.record_family, target.record_family
         LIMIT 25;",
    )?;

    Ok(())
}

fn run_graph_relevance_probe(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let database = Database::new(path, SystemConfig::default())?;
    let connection = Connection::new(&database)?;

    println!("LadybugDB graph relevance probe");
    println!("===============================");
    println!("artifact: {}", path.display());

    report_query(
        &connection,
        "product question: records mechanically similar to Fear",
        "MATCH (seed:Record {record_key: 'spells-srd:4koZzrnMXhhosn0D'})-[:HAS_TRAIT]->(trait:Trait)<-[:HAS_TRAIT]-(other:Record),
               (seed)-[:REFERENCES]->(target:Record)<-[:REFERENCES]-(other)
         WHERE other.record_key <> seed.record_key AND other.is_default_visible
         WITH other, count(DISTINCT trait) AS shared_traits, count(DISTINCT target) AS shared_targets
         RETURN other.record_key, other.name, other.record_family, shared_traits, shared_targets, shared_traits * 2 + shared_targets * 5 AS graph_score
         ORDER BY graph_score DESC, shared_targets DESC, shared_traits DESC, other.name
         LIMIT 25;",
    )?;
    report_query(
        &connection,
        "explain Fear similarity: shared traits",
        "MATCH (seed:Record {record_key: 'spells-srd:4koZzrnMXhhosn0D'})-[:HAS_TRAIT]->(trait:Trait)<-[:HAS_TRAIT]-(other:Record),
               (seed)-[:REFERENCES]->(:Record)<-[:REFERENCES]-(other)
         WHERE other.record_key <> seed.record_key AND other.is_default_visible
         RETURN DISTINCT other.name, other.record_family, trait.name
         ORDER BY other.name, trait.name
         LIMIT 40;",
    )?;
    report_query(
        &connection,
        "explain Fear similarity: shared referenced rules",
        "MATCH (seed:Record {record_key: 'spells-srd:4koZzrnMXhhosn0D'})-[:REFERENCES]->(target:Record)<-[:REFERENCES]-(other:Record),
               (seed)-[:HAS_TRAIT]->(:Trait)<-[:HAS_TRAIT]-(other)
         WHERE other.record_key <> seed.record_key AND other.is_default_visible
         RETURN DISTINCT other.name, other.record_family, target.name, target.record_family
         ORDER BY other.name, target.name
         LIMIT 40;",
    )?;
    report_query(
        &connection,
        "product question: records bridging Demoralize and Frightened",
        "MATCH (source:Record)-[demoralize_ref:REFERENCES]->(:Record {record_key: 'actionspf2e:2u915NdUyQan6uKF'}),
               (source)-[frightened_ref:REFERENCES]->(:Record {record_key: 'conditionitems:TBSHQspnbcqxsmjL'})
         WHERE source.is_default_visible
         RETURN source.record_key, source.name, source.record_family, demoralize_ref.source_kind, frightened_ref.source_kind
         ORDER BY source.record_family, source.name
         LIMIT 50;",
    )?;
    report_query(
        &connection,
        "product question: conditions most associated with Frightened",
        "MATCH (source:Record)-[:REFERENCES]->(:Record {record_key: 'conditionitems:TBSHQspnbcqxsmjL'}),
               (source)-[:REFERENCES]->(other:Record)
         WHERE source.is_default_visible
           AND other.record_key <> 'conditionitems:TBSHQspnbcqxsmjL'
           AND other.record_family = 'rule'
         WITH other, count(DISTINCT source) AS shared_sources
         RETURN other.record_key, other.name, shared_sources
         ORDER BY shared_sources DESC, other.name
         LIMIT 25;",
    )?;
    report_query(
        &connection,
        "product question: Frightened impact by record family",
        "MATCH (source:Record)-[ref:REFERENCES]->(:Record {record_key: 'conditionitems:TBSHQspnbcqxsmjL'})
         WHERE source.is_default_visible
         WITH source.record_family AS family, ref.source_kind AS source_kind, count(source) AS references
         RETURN family, source_kind, references
         ORDER BY references DESC, family, source_kind
         LIMIT 25;",
    )?;
    report_query(
        &connection,
        "product question: spell-like fear mechanics using trait plus condition reference",
        "MATCH (record:Record)-[:HAS_TRAIT]->(:Trait {name: 'fear'}),
               (record)-[ref:REFERENCES]->(target:Record {record_key: 'conditionitems:TBSHQspnbcqxsmjL'})
         WHERE record.is_default_visible
         RETURN record.record_key, record.name, record.record_family, record.level, ref.source_kind
         ORDER BY record.record_family, record.level, record.name
         LIMIT 50;",
    )?;
    report_query(
        &connection,
        "product question: variant/progression families with broad level spread",
        "MATCH (record:Record)-[rel:IN_VARIANT_GROUP]->(variant_group:VariantGroup)
         WHERE record.is_default_visible AND record.level >= 0
         WITH variant_group, count(record) AS variants, min(record.level) AS min_level, max(record.level) AS max_level
         RETURN variant_group.variant_group_key, variant_group.base_name, variants, min_level, max_level, max_level - min_level AS level_spread
         ORDER BY level_spread DESC, variants DESC, variant_group.base_name
         LIMIT 30;",
    )?;
    report_query(
        &connection,
        "product question: variant family members as navigable progression",
        "MATCH (record:Record)-[rel:IN_VARIANT_GROUP]->(variant_group:VariantGroup {base_name: 'Dread Ampoule'})
         WHERE record.is_default_visible
         RETURN record.record_key, record.name, record.record_family, record.level, rel.variant_label, rel.variant_axes_json
         ORDER BY record.level, record.name;",
    )?;
    report_query(
        &connection,
        "product question: remaster pairs with shared referenced mechanics",
        "MATCH (legacy:Record)-[:REMASTERED_BY]->(remaster:Record),
               (legacy)-[:REFERENCES]->(target:Record)<-[:REFERENCES]-(remaster)
         RETURN legacy.record_key, legacy.name, remaster.record_key, remaster.name, count(DISTINCT target) AS shared_references
         ORDER BY shared_references DESC, legacy.name
         LIMIT 25;",
    )?;
    report_query(
        &connection,
        "product question: ambiguous aliases as resolver UX targets",
        "MATCH (record:Record)-[:HAS_ALIAS]->(alias:Alias)
         WITH alias.normalized_alias AS normalized_alias, count(DISTINCT record) AS records
         WHERE records > 1
         RETURN normalized_alias, records
         ORDER BY records DESC, normalized_alias
         LIMIT 25;",
    )?;
    report_query(
        &connection,
        "product question: alias ambiguity detail",
        "MATCH (record:Record)-[:HAS_ALIAS]->(alias:Alias)
         WHERE alias.normalized_alias = 'shield'
         RETURN record.record_key, record.name, record.record_family, alias.alias_text, alias.source_kind
         ORDER BY record.record_family, record.name
         LIMIT 40;",
    )?;
    report_query(
        &connection,
        "product question: source books that concentrate Frightened mechanics",
        "MATCH (record:Record)-[:PUBLISHED_IN]->(publication:Publication),
               (record)-[ref:REFERENCES]->(:Record {record_key: 'conditionitems:TBSHQspnbcqxsmjL'})
         WHERE record.is_default_visible
         RETURN publication.title, publication.family, count(record) AS records, count(ref) AS references
         ORDER BY references DESC, records DESC, publication.title
         LIMIT 25;",
    )?;
    report_query(
        &connection,
        "product question: mechanic ecology around creatures with Frightened abilities",
        "MATCH (creature:Record)-[:REFERENCES]->(:Record {record_key: 'conditionitems:TBSHQspnbcqxsmjL'}),
               (creature)-[:HAS_TRAIT]->(trait:Trait),
               (creature)-[metric_rel:HAS_METRIC]->(metric:Metric {metric_key: 'ac.value'})
         WHERE creature.is_default_visible AND creature.record_family = 'creature'
         WITH trait.name AS trait_name, count(creature) AS creatures, min(metric_rel.number_value) AS min_ac, max(metric_rel.number_value) AS max_ac
         RETURN trait_name, creatures, min_ac, max_ac
         ORDER BY creatures DESC, trait_name
         LIMIT 25;",
    )?;
    report_query(
        &connection,
        "product question: records connected through content-unit evidence",
        "MATCH (record:Record)-[:HAS_CONTENT_UNIT]->(content:ContentUnit),
               (record)-[ref:REFERENCES]->(target:Record {record_key: 'conditionitems:TBSHQspnbcqxsmjL'})
         WHERE record.is_default_visible
         RETURN record.record_key, record.name, record.record_family, content.source_kind, content.label, ref.source_kind
         ORDER BY record.record_family, record.name
         LIMIT 40;",
    )?;
    report_query(
        &connection,
        "product question: direct content-unit evidence for Frightened",
        "MATCH (record:Record)-[:HAS_CONTENT_UNIT]->(content:ContentUnit)-[evidence:CONTENT_REFERENCES]->(target:Record {record_key: 'conditionitems:TBSHQspnbcqxsmjL'})
         WHERE record.is_default_visible
         RETURN record.record_key, record.name, record.record_family, content.label, content.source_kind, evidence.display_text, evidence.source_kind
         ORDER BY record.record_family, record.name, content.label
         LIMIT 60;",
    )?;
    report_query(
        &connection,
        "product question: evidence-unit matches for Frightened",
        "MATCH (record:Record)-[:HAS_EVIDENCE_UNIT]->(evidence:EvidenceUnit)-[edge:EVIDENCE_REFERENCES]->(:Record {record_key: 'conditionitems:TBSHQspnbcqxsmjL'})
         WHERE record.is_default_visible
         RETURN record.record_key, record.name, record.record_family, evidence.label, evidence.source_kind, evidence.unit_kind, edge.display_text
         ORDER BY record.record_family, record.name, evidence.ordinal
         LIMIT 80;",
    )?;
    report_query(
        &connection,
        "product question: evidence-unit source mix for Frightened",
        "MATCH (record:Record)-[:HAS_EVIDENCE_UNIT]->(evidence:EvidenceUnit)-[:EVIDENCE_REFERENCES]->(:Record {record_key: 'conditionitems:TBSHQspnbcqxsmjL'})
         WHERE record.is_default_visible
         RETURN record.record_family, evidence.source_kind, evidence.unit_kind, count(DISTINCT evidence) AS evidence_units, count(DISTINCT record) AS records
         ORDER BY evidence_units DESC, records DESC, record.record_family, evidence.source_kind, evidence.unit_kind
         LIMIT 30;",
    )?;
    report_query(
        &connection,
        "product question: Demoralize plus Frightened through same evidence unit",
        "MATCH (record:Record)-[:HAS_EVIDENCE_UNIT]->(evidence:EvidenceUnit)-[:EVIDENCE_REFERENCES]->(:Record {record_key: 'actionspf2e:2u915NdUyQan6uKF'}),
               (evidence)-[:EVIDENCE_REFERENCES]->(:Record {record_key: 'conditionitems:TBSHQspnbcqxsmjL'})
         WHERE record.is_default_visible
         RETURN record.record_key, record.name, record.record_family, evidence.label, evidence.source_kind, evidence.unit_kind
         ORDER BY record.record_family, record.name, evidence.ordinal
         LIMIT 80;",
    )?;
    report_query(
        &connection,
        "product question: evidence units with optional embeddings",
        "MATCH (record:Record)-[:HAS_EVIDENCE_UNIT]->(evidence:EvidenceUnit)
         OPTIONAL MATCH (evidence)-[:HAS_EVIDENCE_EMBEDDING]->(embedding:EmbeddingUnit)
         WITH evidence.source_kind AS source_kind, evidence.unit_kind AS unit_kind, count(evidence) AS evidence_units, count(embedding) AS embedded_units
         RETURN source_kind, unit_kind, evidence_units, embedded_units
         ORDER BY evidence_units DESC, source_kind, unit_kind
         LIMIT 30;",
    )?;
    report_query(
        &connection,
        "product question: content-unit evidence counts for Frightened",
        "MATCH (record:Record)-[:HAS_CONTENT_UNIT]->(content:ContentUnit)-[evidence:CONTENT_REFERENCES]->(:Record {record_key: 'conditionitems:TBSHQspnbcqxsmjL'})
         WHERE record.is_default_visible
         RETURN record.record_family, content.source_kind, count(DISTINCT content) AS evidence_units, count(DISTINCT record) AS records
         ORDER BY evidence_units DESC, records DESC, record.record_family, content.source_kind
         LIMIT 25;",
    )?;
    report_query(
        &connection,
        "product question: Demoralize plus Frightened through same content unit",
        "MATCH (record:Record)-[:HAS_CONTENT_UNIT]->(content:ContentUnit)-[:CONTENT_REFERENCES]->(:Record {record_key: 'actionspf2e:2u915NdUyQan6uKF'}),
               (content)-[:CONTENT_REFERENCES]->(:Record {record_key: 'conditionitems:TBSHQspnbcqxsmjL'})
         WHERE record.is_default_visible
         RETURN record.record_key, record.name, record.record_family, content.label, content.source_kind
         ORDER BY record.record_family, record.name, content.label
         LIMIT 60;",
    )?;

    Ok(())
}

fn run_graph_product_eval_probe(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let database = Database::new(path, SystemConfig::default())?;
    let connection = Connection::new(&database)?;

    println!("LadybugDB graph product evaluation");
    println!("==================================");
    println!("artifact: {}", path.display());
    println!();
    println!("Scale: strong = graph is doing product-shaped work that would be awkward to");
    println!("       recreate relationally; mixed = useful but noisy or SQL-plausible;");
    println!("       weak = graph adds little beyond current SQLite-shaped behavior.");

    report_evaluation_query(
        &connection,
        "more-like-this by relationship evidence",
        "strong",
        "Tests whether graph ranking can combine traits plus shared referenced rules and explain why records are similar to Fear.",
        "A graph has a real advantage if the top rows are thematically close fear effects and can be explained through shared paths.",
        "MATCH (seed:Record {record_key: 'spells-srd:4koZzrnMXhhosn0D'})-[:HAS_TRAIT]->(trait:Trait)<-[:HAS_TRAIT]-(other:Record),
               (seed)-[:REFERENCES]->(target:Record)<-[:REFERENCES]-(other)
         WHERE other.record_key <> seed.record_key AND other.is_default_visible
         WITH other, count(DISTINCT trait) AS shared_traits, count(DISTINCT target) AS shared_targets
         RETURN other.record_key, other.name, other.record_family, shared_traits, shared_targets, shared_traits * 2 + shared_targets * 5 AS graph_score
         ORDER BY graph_score DESC, shared_targets DESC, shared_traits DESC, other.name
         LIMIT 15;",
    )?;
    report_evaluation_query(
        &connection,
        "same-evidence mechanic bridge",
        "strong",
        "Tests whether the graph can answer 'which specific ability/text unit connects Demoralize and Frightened?' instead of only returning broad records.",
        "This is the clearest evidence-unit product win: the UI can show the exact ability or description section that bridges two mechanics.",
        "MATCH (record:Record)-[:HAS_EVIDENCE_UNIT]->(evidence:EvidenceUnit)-[:EVIDENCE_REFERENCES]->(:Record {record_key: 'actionspf2e:2u915NdUyQan6uKF'}),
               (evidence)-[:EVIDENCE_REFERENCES]->(:Record {record_key: 'conditionitems:TBSHQspnbcqxsmjL'})
         WHERE record.is_default_visible
         RETURN record.record_key, record.name, record.record_family, evidence.label, evidence.source_kind, evidence.unit_kind
         ORDER BY record.record_family, record.name, evidence.ordinal
         LIMIT 25;",
    )?;
    report_evaluation_query(
        &connection,
        "mechanic impact map",
        "mixed",
        "Tests whether a mechanic page can summarize where Frightened appears by record family and evidence source.",
        "Useful for product navigation and source impact, but much of this is still count/facet-shaped and SQLite can express it.",
        "MATCH (record:Record)-[:HAS_EVIDENCE_UNIT]->(evidence:EvidenceUnit)-[:EVIDENCE_REFERENCES]->(:Record {record_key: 'conditionitems:TBSHQspnbcqxsmjL'})
         WHERE record.is_default_visible
         RETURN record.record_family, evidence.source_kind, evidence.unit_kind, count(DISTINCT evidence) AS evidence_units, count(DISTINCT record) AS records
         ORDER BY evidence_units DESC, records DESC, record.record_family, evidence.source_kind, evidence.unit_kind
         LIMIT 20;",
    )?;
    report_evaluation_query(
        &connection,
        "mechanic neighborhood",
        "mixed",
        "Tests whether graph co-reference can build 'mechanics related to Frightened' pages.",
        "The shape is promising, but common references and broad rule families need weighting or explicit Concept nodes before this is product-ready.",
        "MATCH (source:Record)-[:REFERENCES]->(:Record {record_key: 'conditionitems:TBSHQspnbcqxsmjL'}),
               (source)-[:REFERENCES]->(other:Record)
         WHERE source.is_default_visible
           AND other.record_key <> 'conditionitems:TBSHQspnbcqxsmjL'
           AND other.record_family = 'rule'
         WITH other, count(DISTINCT source) AS shared_sources
         RETURN other.record_key, other.name, shared_sources
         ORDER BY shared_sources DESC, other.name
         LIMIT 15;",
    )?;
    report_evaluation_query(
        &connection,
        "variant progression navigation",
        "strong",
        "Tests whether graph relationships make item/creature/spell families naturally navigable.",
        "This is a good graph product shape because the user wants to move along a relationship, not run a search query from scratch.",
        "MATCH (record:Record)-[rel:IN_VARIANT_GROUP]->(variant_group:VariantGroup {base_name: 'Dread Ampoule'})
         WHERE record.is_default_visible
         RETURN record.record_key, record.name, record.record_family, record.level, rel.variant_label, rel.variant_axes_json
         ORDER BY record.level, record.name;",
    )?;
    report_evaluation_query(
        &connection,
        "remaster relationship comparison",
        "mixed",
        "Tests whether legacy/remaster links become richer when combined with shared referenced mechanics.",
        "Promising as a product surface, but the current model needs better diff-specific facts to be more than relationship context.",
        "MATCH (legacy:Record)-[:REMASTERED_BY]->(remaster:Record),
               (legacy)-[:REFERENCES]->(target:Record)<-[:REFERENCES]-(remaster)
         RETURN legacy.record_key, legacy.name, remaster.record_key, remaster.name, count(DISTINCT target) AS shared_references
         ORDER BY shared_references DESC, legacy.name
         LIMIT 15;",
    )?;
    report_evaluation_query(
        &connection,
        "alias ambiguity resolver candidates",
        "mixed",
        "Tests whether aliases can drive lookup disambiguation and explain why a name resolved.",
        "The graph shape is useful, but current aliases need deduplication and better resolver-specific ranking.",
        "MATCH (record:Record)-[:HAS_ALIAS]->(alias:Alias)
         WITH alias.normalized_alias AS normalized_alias, count(DISTINCT record) AS records
         WHERE records > 1
         RETURN normalized_alias, records
         ORDER BY records DESC, normalized_alias
         LIMIT 15;",
    )?;
    report_evaluation_query(
        &connection,
        "creature mechanic ecology",
        "mixed",
        "Tests whether a graph can explain which creature traits and metrics cluster around a mechanic.",
        "Useful for encounter/design exploration, but noisy until broad traits are weighted or filtered.",
        "MATCH (creature:Record)-[:REFERENCES]->(:Record {record_key: 'conditionitems:TBSHQspnbcqxsmjL'}),
               (creature)-[:HAS_TRAIT]->(trait:Trait),
               (creature)-[metric_rel:HAS_METRIC]->(metric:Metric {metric_key: 'ac.value'})
         WHERE creature.is_default_visible AND creature.record_family = 'creature'
         WITH trait.name AS trait_name, count(creature) AS creatures, min(metric_rel.number_value) AS min_ac, max(metric_rel.number_value) AS max_ac
         RETURN trait_name, creatures, min_ac, max_ac
         ORDER BY creatures DESC, trait_name
         LIMIT 15;",
    )?;
    report_evaluation_query(
        &connection,
        "plain structured count",
        "weak",
        "Tests a baseline SQLite-shaped question: how many visible low-level spells are there?",
        "This is not graph-novel. It is important product behavior, but SQLite is already an excellent fit for it.",
        "MATCH (record:Record)
         WHERE record.is_default_visible AND record.record_family = 'spell' AND record.level >= 0 AND record.level <= 3
         RETURN count(record);",
    )?;

    println!();
    println!("Overall read");
    println!("------------");
    println!("Strong graph cases: relationship-ranked more-like-this, same-evidence");
    println!("mechanic bridges, and progression navigation.");
    println!("Mixed graph cases: mechanic pages, remaster context, alias UX, and");
    println!("ecology views. They are promising but need weighting, concepts, or");
    println!("better product-specific facts.");
    println!("Weak graph cases: direct lookup, ordinary structured counts, and flat");
    println!("facets. SQLite remains a better fit for those baseline operations.");
    println!();
    println!("Current conclusion: the graph is product-novel as an explanation and");
    println!("relationship-relevance layer. It has not yet proven that it should");
    println!("replace SQLite for the whole core search artifact.");

    Ok(())
}

#[derive(Debug, Clone, Copy)]
struct FtsRiskCase {
    label: &'static str,
    query: &'static str,
}

fn run_fts_rrf_risk_probe(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let database = Database::new(path, SystemConfig::default())?;
    let connection = Connection::new(&database)?;
    if !load_extension(&connection, "FTS") {
        println!("status: blocked");
        println!("reason: FTS extension unavailable");
        return Ok(());
    }

    println!("LadybugDB FTS RRF risk probe");
    println!("============================");
    println!("artifact: {}", path.display());
    println!(
        "read: broad OR candidates can add weak RRF votes; strict/min-token variants test mitigations."
    );

    let cases = [
        FtsRiskCase {
            label: "title-ish exact action",
            query: "treat wounds",
        },
        FtsRiskCase {
            label: "mechanic phrase",
            query: "frightened condition",
        },
        FtsRiskCase {
            label: "common combat terms",
            query: "fire damage",
        },
        FtsRiskCase {
            label: "metric/source-ish terms",
            query: "armor class",
        },
        FtsRiskCase {
            label: "mixed conceptual natural language",
            query: "low level fear spell",
        },
    ];

    for case in cases {
        report_fts_risk_case(&connection, case);
    }

    Ok(())
}

fn report_fts_risk_case(connection: &Connection<'_>, case: FtsRiskCase) {
    println!();
    println!("## {}: {:?}", case.label, case.query);
    let tokens = fts_probe_tokens(case.query);
    println!("tokens: {}", tokens.join(", "));
    for (variant, query) in [
        ("broad_or", fts_probe_or_query(&tokens)),
        ("strict_and", fts_probe_and_query(&tokens)),
    ] {
        if query.is_empty() {
            println!("{variant}: skipped");
            continue;
        }
        println!();
        println!("-- {variant}: {query}");
        match fts_probe_hits(connection, &query, &tokens, 200) {
            Ok(hits) => print_fts_probe_summary(&hits, tokens.len()),
            Err(error) => println!("error: {error}"),
        }
    }
}

#[derive(Debug)]
struct FtsProbeHit {
    record_key: String,
    name: String,
    family: String,
    score: f64,
    matched_tokens: usize,
}

fn fts_probe_hits(
    connection: &Connection<'_>,
    query: &str,
    tokens: &[String],
    top: usize,
) -> Result<Vec<FtsProbeHit>, Box<dyn std::error::Error>> {
    let sql = format!(
        "CALL QUERY_FTS_INDEX('SearchDocument', 'search_document_fts', {}, top := {top})
         WITH node AS doc, score
         MATCH (record:Record)-[:HAS_SEARCH_DOCUMENT]->(doc)
         RETURN record.record_key, record.name, record.record_family, score,
                doc.title, doc.aliases, doc.traits, doc.taxonomy_terms,
                doc.constraint_terms, doc.mechanic_terms, doc.source_terms,
                doc.metric_terms, doc.headings, doc.body, doc.facts,
                doc.reference_terms, doc.embedded_content
         ORDER BY score DESC
         LIMIT {top};",
        cypher_string_literal(query)
    );
    let mut rows = connection.query(&sql)?;
    let mut hits = Vec::new();
    for row in &mut rows {
        let text = (4..=16)
            .filter_map(|index| value_string(&row, index))
            .collect::<Vec<_>>()
            .join(" ");
        hits.push(FtsProbeHit {
            record_key: value_string(&row, 0).unwrap_or_default(),
            name: value_string(&row, 1).unwrap_or_default(),
            family: value_string(&row, 2).unwrap_or_default(),
            score: value_f64(&row, 3).unwrap_or_default(),
            matched_tokens: count_matched_tokens(tokens, &text),
        });
    }
    Ok(hits)
}

fn print_fts_probe_summary(hits: &[FtsProbeHit], token_count: usize) {
    if hits.is_empty() {
        println!("hits: 0");
        return;
    }
    let single_token_top_10 = hits
        .iter()
        .take(10)
        .filter(|hit| hit.matched_tokens <= 1 && token_count > 1)
        .count();
    let single_token_top_50 = hits
        .iter()
        .take(50)
        .filter(|hit| hit.matched_tokens <= 1 && token_count > 1)
        .count();
    let full_coverage_top_50 = hits
        .iter()
        .take(50)
        .filter(|hit| hit.matched_tokens == token_count)
        .count();
    println!(
        "hits={} single_token_top10={} single_token_top50={} full_coverage_top50={}",
        hits.len(),
        single_token_top_10,
        single_token_top_50,
        full_coverage_top_50
    );
    println!("top:");
    for (index, hit) in hits.iter().take(10).enumerate() {
        println!(
            "  {:>2}. {} | {} | {} | score={:.3} | token_coverage={}/{}",
            index + 1,
            hit.record_key,
            hit.name,
            hit.family,
            hit.score,
            hit.matched_tokens,
            token_count
        );
    }
}

fn fts_probe_tokens(query: &str) -> Vec<String> {
    query
        .split(|character: char| !character.is_ascii_alphanumeric())
        .map(|token| token.to_ascii_lowercase())
        .filter(|token| !token.is_empty())
        .collect()
}

fn fts_probe_or_query(tokens: &[String]) -> String {
    tokens
        .iter()
        .map(|token| format!("\"{token}\""))
        .collect::<Vec<_>>()
        .join(" OR ")
}

fn fts_probe_and_query(tokens: &[String]) -> String {
    tokens
        .iter()
        .map(|token| format!("\"{token}\""))
        .collect::<Vec<_>>()
        .join(" ")
}

fn count_matched_tokens(tokens: &[String], text: &str) -> usize {
    let text_tokens = fts_probe_tokens(text);
    tokens
        .iter()
        .filter(|token| text_tokens.iter().any(|text_token| text_token == *token))
        .count()
}

fn value_string(row: &[Value], index: usize) -> Option<String> {
    row.get(index).and_then(|value| match value {
        Value::String(value) => Some(value.clone()),
        Value::Int64(value) => Some(value.to_string()),
        Value::Double(value) => Some(value.to_string()),
        Value::Bool(value) => Some(value.to_string()),
        Value::Null(_) => None,
        _ => Some(value.to_string()),
    })
}

fn value_f64(row: &[Value], index: usize) -> Option<f64> {
    row.get(index).and_then(|value| match value {
        Value::Double(value) => Some(*value),
        Value::Int64(value) => Some(*value as f64),
        Value::String(value) => value.parse().ok(),
        _ => None,
    })
}

#[derive(Debug, Clone, Copy)]
struct HydrationAuditCase {
    label: &'static str,
    key: &'static str,
}

fn run_hydration_audit(
    sqlite_path: &Path,
    ladybug_path: &Path,
    atlas_bin: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let cases = [
        HydrationAuditCase {
            label: "spell",
            key: "spells-srd:4koZzrnMXhhosn0D",
        },
        HydrationAuditCase {
            label: "feat",
            key: "feats-srd:muMOxZyduEFv8UT6",
        },
        HydrationAuditCase {
            label: "equipment",
            key: "equipment-srd:s1vB3HdXjMigYAnY",
        },
        HydrationAuditCase {
            label: "creature",
            key: "pfs-season-5-bestiary:u74d5wQNBJLVHB7m",
        },
        HydrationAuditCase {
            label: "hazard",
            key: "pfs-season-4-bestiary:Vy5P6hdCiVLeRWTa",
        },
        HydrationAuditCase {
            label: "rule",
            key: "actionspf2e:1kGNdIIhuglAjIp9",
        },
        HydrationAuditCase {
            label: "affliction",
            key: "derived-afflictions:fe0c64b7",
        },
    ];

    println!("LadybugDB hydration audit");
    println!("=========================");
    println!("sqlite: {}", sqlite_path.display());
    println!("ladybug: {}", ladybug_path.display());
    println!("atlas: {}", atlas_bin.display());
    println!("detail: full");

    let mut passed = 0usize;
    let mut mismatched = 0usize;
    let mut failed = 0usize;

    for case in cases {
        println!();
        println!("## {} ({})", case.label, case.key);
        let sqlite = record_get_json(atlas_bin, sqlite_path, ladybug_path, "sqlite", case.key);
        let ladybug = record_get_json(atlas_bin, sqlite_path, ladybug_path, "ladybug", case.key);
        match (sqlite, ladybug) {
            (Ok(sqlite), Ok(ladybug)) => {
                let sqlite_record = envelope_record(&sqlite)?;
                let ladybug_record = envelope_record(&ladybug)?;
                let mut differences = Vec::new();
                collect_json_differences(
                    "$.data.record",
                    sqlite_record,
                    ladybug_record,
                    &mut differences,
                    40,
                );
                if differences.is_empty() {
                    passed += 1;
                    println!("status: parity-clean");
                } else {
                    mismatched += 1;
                    println!("status: mismatch");
                    println!("differences: {}", differences.len());
                    for difference in differences.iter().take(20) {
                        println!("  - {difference}");
                    }
                    if differences.len() > 20 {
                        println!("  - ... {} more", differences.len() - 20);
                    }
                }
            }
            (sqlite, ladybug) => {
                failed += 1;
                println!("status: failed");
                if let Err(error) = sqlite {
                    println!("sqlite error: {error}");
                }
                if let Err(error) = ladybug {
                    println!("ladybug error: {error}");
                }
            }
        }
    }

    println!();
    println!("Summary");
    println!("-------");
    println!("passed: {passed}");
    println!("mismatched: {mismatched}");
    println!("failed: {failed}");
    Ok(())
}

#[derive(Debug, Clone, Copy)]
struct VectorFilterCase {
    label: &'static str,
    projection_name: &'static str,
    projection_cypher: &'static str,
}

fn run_vector_filter_parity_probe(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let database = Database::new(path, SystemConfig::default())?;
    let connection = Connection::new(&database)?;

    println!("LadybugDB vector filter parity probe");
    println!("====================================");
    println!("artifact: {}", path.display());

    if !prepare_baseline_vector(&connection) {
        println!("status: blocked");
        println!("reason: VECTOR extension/index is unavailable or the artifact has no embeddings");
        return Ok(());
    }

    let cases = [
        VectorFilterCase {
            label: "family + level + trait",
            projection_name: "eligible_vector_spell_low_fire",
            projection_cypher: "MATCH (record:Record)-[:HAS_EMBEDDING_UNIT]->(embedding:EmbeddingUnit), \
                               (record)-[:HAS_TRAIT]->(trait:Trait) \
                               WHERE record.is_default_visible \
                                 AND record.record_family = 'spell' \
                                 AND record.level >= 1 \
                                 AND record.level <= 3 \
                                 AND trait.name = 'fire' \
                               RETURN embedding",
        },
        VectorFilterCase {
            label: "family + metric range",
            projection_name: "eligible_vector_creature_ac_30_35",
            projection_cypher: "MATCH (record:Record)-[:HAS_EMBEDDING_UNIT]->(embedding:EmbeddingUnit), \
                               (record)-[metric_rel:HAS_METRIC]->(metric:Metric) \
                               WHERE record.is_default_visible \
                                 AND record.record_family = 'creature' \
                                 AND metric.metric_key = 'ac.value' \
                                 AND metric_rel.number_value >= 30 \
                                 AND metric_rel.number_value <= 35 \
                               RETURN embedding",
        },
        VectorFilterCase {
            label: "publication/remaster filter",
            projection_name: "eligible_vector_core_remaster_spells",
            projection_cypher: "MATCH (record:Record)-[:HAS_EMBEDDING_UNIT]->(embedding:EmbeddingUnit), \
                               (record)-[:PUBLISHED_IN]->(publication:Publication) \
                               WHERE record.is_default_visible \
                                 AND record.record_family = 'spell' \
                                 AND publication.family = 'core' \
                                 AND publication.remaster = true \
                               RETURN embedding",
        },
        VectorFilterCase {
            label: "reference-derived filter",
            projection_name: "eligible_vector_references_frightened",
            projection_cypher: "MATCH (record:Record)-[:HAS_EMBEDDING_UNIT]->(embedding:EmbeddingUnit), \
                               (record)-[:REFERENCES]->(:Record {record_key: 'conditionitems:TBSHQspnbcqxsmjL'}) \
                               WHERE record.is_default_visible \
                                 AND record.record_family IN ['spell', 'feat', 'equipment', 'creature'] \
                               RETURN embedding",
        },
    ];

    for case in cases {
        report_vector_filter_case(&connection, case);
    }

    Ok(())
}

fn report_vector_filter_case(connection: &Connection<'_>, case: VectorFilterCase) {
    println!();
    println!("## {}", case.label);
    let _ = connection.query(&format!(
        "CALL DROP_PROJECTED_GRAPH('{}');",
        case.projection_name
    ));
    let create_query = format!(
        "CALL PROJECT_GRAPH_CYPHER('{}', {});",
        case.projection_name,
        cypher_string_literal(case.projection_cypher)
    );
    match connection.query(&create_query) {
        Ok(mut result) => {
            let mut rows = Vec::new();
            for row in &mut result {
                rows.push(format_row(&row));
            }
            println!(
                "projection timing: compile={:.3}ms execute={:.3}ms",
                result.get_compiling_time(),
                result.get_execution_time()
            );
            for row in rows {
                println!("projection: {row}");
            }
        }
        Err(error) => {
            println!("status: projection failed");
            println!("error: {error}");
            return;
        }
    }

    let query = format!(
        "CALL QUERY_VECTOR_INDEX('{}', 'embedding_hnsw', CAST({}, 'FLOAT[384]'), 10, efs := 50)
         WITH node AS embedding, distance
         MATCH (record:Record)-[:HAS_EMBEDDING_UNIT]->(embedding)
         RETURN record.record_key, record.name, record.record_family, embedding.unit_kind, distance
         ORDER BY distance
         LIMIT 10;",
        case.projection_name,
        vector_literal(384, 0)
    );
    report_parity_query(
        connection,
        "prefiltered vector top-k",
        "parity-test",
        "The vector query runs against the projected eligible EmbeddingUnit graph, so top-k is evaluated after structural filtering.",
        &query,
    );
}

fn record_get_json(
    atlas_bin: &Path,
    sqlite_path: &Path,
    ladybug_path: &Path,
    backend: &str,
    key: &str,
) -> Result<JsonValue, Box<dyn std::error::Error>> {
    let mut command = Command::new(atlas_bin);
    command
        .arg("record")
        .arg("get")
        .arg(key)
        .arg("--index")
        .arg(sqlite_path)
        .arg("--index-backend")
        .arg(backend)
        .arg("--detail")
        .arg("full")
        .arg("--json");
    if backend == "ladybug" {
        command.arg("--ladybug-index").arg(ladybug_path);
    }

    let output = command.output()?;
    if !output.status.success() {
        return Err(format!(
            "atlas record get failed with status {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }

    let json = serde_json::from_slice(&output.stdout)?;
    Ok(json)
}

fn envelope_record(json: &JsonValue) -> Result<&JsonValue, Box<dyn std::error::Error>> {
    match json.pointer("/status").and_then(JsonValue::as_str) {
        Some("ok") => json
            .pointer("/data/record")
            .ok_or_else(|| "JSON envelope did not contain /data/record".into()),
        Some(status) => Err(format!("JSON envelope status was {status:?}").into()),
        None => Err("JSON envelope did not contain string /status".into()),
    }
}

fn collect_json_differences(
    path: &str,
    left: &JsonValue,
    right: &JsonValue,
    differences: &mut Vec<String>,
    max: usize,
) {
    if differences.len() >= max || left == right {
        return;
    }

    match (left, right) {
        (JsonValue::Object(left_object), JsonValue::Object(right_object)) => {
            let mut keys = std::collections::BTreeSet::new();
            keys.extend(left_object.keys());
            keys.extend(right_object.keys());
            for key in keys {
                if differences.len() >= max {
                    return;
                }
                let child_path = format!("{path}.{key}");
                match (left_object.get(key), right_object.get(key)) {
                    (Some(left_value), Some(right_value)) => collect_json_differences(
                        &child_path,
                        left_value,
                        right_value,
                        differences,
                        max,
                    ),
                    (Some(left_value), None) => differences.push(format!(
                        "{child_path}: sqlite={} ladybug=<missing>",
                        summarize_json(left_value)
                    )),
                    (None, Some(right_value)) => differences.push(format!(
                        "{child_path}: sqlite=<missing> ladybug={}",
                        summarize_json(right_value)
                    )),
                    (None, None) => {}
                }
            }
        }
        (JsonValue::Array(left_array), JsonValue::Array(right_array)) => {
            if left_array.len() != right_array.len() {
                differences.push(format!(
                    "{path}.length: sqlite={} ladybug={}",
                    left_array.len(),
                    right_array.len()
                ));
                return;
            }
            for (index, (left_value, right_value)) in
                left_array.iter().zip(right_array.iter()).enumerate()
            {
                if differences.len() >= max {
                    return;
                }
                collect_json_differences(
                    &format!("{path}[{index}]"),
                    left_value,
                    right_value,
                    differences,
                    max,
                );
            }
        }
        _ => differences.push(format!(
            "{path}: sqlite={} ladybug={}",
            summarize_json(left),
            summarize_json(right)
        )),
    }
}

fn run_baseline_parity_probe(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let database = Database::new(path, SystemConfig::default())?;
    let connection = Connection::new(&database)?;

    println!("LadybugDB baseline parity probe");
    println!("===============================");
    println!("artifact: {}", path.display());
    println!();
    println!("Classification:");
    println!("  parity-clean = Ladybug expresses the current SQLite-shaped need naturally.");
    println!("  parity-awkward = possible, but the query/model is more awkward than SQLite.");
    println!("  parity-blocked = not proven with this artifact or currently failing.");
    println!(
        "  potentially-better = preserves baseline behavior while improving evidence/provenance shape."
    );

    report_parity_query(
        &connection,
        "direct record lookup by recordKey",
        "parity-clean",
        "Equivalent to direct SQLite record-key lookup.",
        "MATCH (record:Record {record_key: 'spells-srd:4koZzrnMXhhosn0D'})
         RETURN record.record_key, record.name, record.record_family, record.level, record.rarity;",
    );
    report_parity_query(
        &connection,
        "structured count: visible common low-rank spells",
        "parity-clean",
        "Scalar structured filters over Record properties are straightforward.",
        "MATCH (record:Record)
         WHERE record.is_default_visible
           AND record.record_family = 'spell'
           AND record.level >= 0
           AND record.level <= 3
           AND record.rarity = 'common'
         RETURN count(record);",
    );
    report_parity_query(
        &connection,
        "facet counts under partial filter",
        "parity-clean",
        "Facet counts over relationship-backed traits work with the same eligible-record pattern.",
        "MATCH (record:Record)-[:HAS_TRAIT]->(trait:Trait)
         WHERE record.is_default_visible
           AND record.record_family = 'spell'
           AND record.level >= 0
           AND record.level <= 3
           AND record.rarity = 'common'
         RETURN trait.name, count(DISTINCT record) AS records
         ORDER BY records DESC, trait.name
         LIMIT 20;",
    );
    report_parity_query(
        &connection,
        "relationship-backed structured filter",
        "parity-clean",
        "Multi-value filters such as traits are naturally represented as graph relationships.",
        "MATCH (record:Record)-[:HAS_TRAIT]->(:Trait {name: 'fear'})
         WHERE record.is_default_visible
           AND record.record_family IN ['spell', 'feat', 'equipment']
           AND record.level >= 0
           AND record.level <= 6
         RETURN record.record_key, record.name, record.record_family, record.level
         ORDER BY record.record_family, record.level, record.name
         LIMIT 25;",
    );
    report_parity_query(
        &connection,
        "open-ended metric range filter",
        "parity-clean",
        "Metric fact nodes preserve the SQLite side-table style for arbitrary numeric filters.",
        "MATCH (record:Record)-[metric_rel:HAS_METRIC]->(metric:Metric {metric_key: 'ac.value'})
         WHERE record.is_default_visible
           AND record.record_family = 'creature'
           AND metric_rel.number_value >= 30
           AND metric_rel.number_value <= 35
         RETURN count(DISTINCT record), min(metric_rel.number_value), max(metric_rel.number_value);",
    );
    report_parity_query(
        &connection,
        "evidence/reference search with structured filters",
        "potentially-better",
        "This keeps structured filters on Record while using evidence edges for provenance.",
        "MATCH (record:Record)-[:HAS_EVIDENCE_UNIT]->(evidence:EvidenceUnit)-[edge:EVIDENCE_REFERENCES]->(:Record {record_key: 'conditionitems:TBSHQspnbcqxsmjL'})
         WHERE record.is_default_visible
           AND record.record_family = 'feat'
           AND record.level >= 0
           AND record.level <= 6
         RETURN record.record_key, record.name, record.level, evidence.label, evidence.source_kind, edge.display_text
         ORDER BY record.level, record.name
         LIMIT 25;",
    );
    report_parity_query(
        &connection,
        "collapse evidence hits back to distinct records",
        "potentially-better",
        "Evidence-level hits can collapse to records while retaining hit counts for ranking/explanation.",
        "MATCH (record:Record)-[:HAS_EVIDENCE_UNIT]->(evidence:EvidenceUnit)-[:EVIDENCE_REFERENCES]->(:Record {record_key: 'conditionitems:TBSHQspnbcqxsmjL'})
         WHERE record.is_default_visible
           AND record.record_family IN ['spell', 'feat', 'equipment']
         RETURN record.record_key, record.name, record.record_family, count(DISTINCT evidence) AS evidence_hits
         ORDER BY evidence_hits DESC, record.record_family, record.name
         LIMIT 25;",
    );
    report_parity_query(
        &connection,
        "facet counts under evidence/reference filter",
        "potentially-better",
        "Facet counts can be computed from the same evidence-filtered eligible set.",
        "MATCH (record:Record)-[:HAS_EVIDENCE_UNIT]->(:EvidenceUnit)-[:EVIDENCE_REFERENCES]->(:Record {record_key: 'conditionitems:TBSHQspnbcqxsmjL'}),
               (record)-[:HAS_TRAIT]->(trait:Trait)
         WHERE record.is_default_visible
           AND record.record_family IN ['spell', 'feat', 'equipment']
         RETURN trait.name, count(DISTINCT record) AS records
         ORDER BY records DESC, trait.name
         LIMIT 20;",
    );

    let fts_ready = prepare_baseline_fts(&connection);
    if fts_ready {
        report_parity_query(
            &connection,
            "SearchDocument FTS with structured filters",
            "parity-clean",
            "FTS candidates can be joined back to Record and filtered there.",
            "CALL QUERY_FTS_INDEX('SearchDocument', 'search_document_fts', 'frightened condition', top := 50)
             WITH node AS doc, score
             MATCH (record:Record)-[:HAS_SEARCH_DOCUMENT]->(doc)
             WHERE record.is_default_visible
               AND record.record_family = 'spell'
               AND record.level >= 0
               AND record.level <= 3
             RETURN record.record_key, record.name, record.level, score
             ORDER BY score DESC
             LIMIT 15;",
        );
        report_parity_query(
            &connection,
            "EvidenceUnit FTS with structured filters and record collapse",
            "potentially-better",
            "Evidence FTS can filter through parent Record and return match provenance.",
            "CALL QUERY_FTS_INDEX('EvidenceUnit', 'evidence_unit_fts', 'frightened demoralize', top := 80)
             WITH node AS evidence, score
             MATCH (record:Record)-[:HAS_EVIDENCE_UNIT]->(evidence)
             WHERE record.is_default_visible
               AND record.record_family IN ['feat', 'equipment', 'spell', 'creature']
             RETURN record.record_key, record.name, record.record_family, evidence.label, evidence.source_kind, score
             ORDER BY score DESC
             LIMIT 20;",
        );
    } else {
        report_parity_blocked(
            "FTS with structured filters",
            "parity-blocked",
            "Could not load/create/query FTS indexes for this artifact. This is an artifact/setup gap to resolve before judging Ladybug FTS parity.",
        );
    }

    report_parity_query(
        &connection,
        "semantic/vector readiness",
        "parity-clean-or-blocked-by-artifact",
        "Counts embedding rows. A no-embedding artifact should return zero; a with-embeddings artifact should return real rows.",
        "MATCH (record:Record)-[:HAS_EMBEDDING_UNIT]->(embedding:EmbeddingUnit)
         RETURN count(DISTINCT record), count(embedding), min(embedding.dimensions), max(embedding.dimensions);",
    );
    report_parity_query(
        &connection,
        "evidence-to-embedding linkage readiness",
        "potentially-better-or-blocked-by-artifact",
        "Counts EvidenceUnit rows that can point at the exact EmbeddingUnit used for semantic provenance.",
        "MATCH (evidence:EvidenceUnit)-[:HAS_EVIDENCE_EMBEDDING]->(embedding:EmbeddingUnit)
         RETURN count(DISTINCT evidence), count(DISTINCT embedding);",
    );

    let vector_ready = prepare_baseline_vector(&connection);
    if vector_ready {
        report_parity_query(
            &connection,
            "vector search with record collapse",
            "parity-clean",
            "Vector candidates can collapse back to parent records through HAS_EMBEDDING_UNIT.",
            &format!(
                "CALL QUERY_VECTOR_INDEX('EmbeddingUnit', 'embedding_hnsw', CAST({}, 'FLOAT[384]'), 25, efs := 50)
                 WITH node AS embedding, distance
                 MATCH (record:Record)-[:HAS_EMBEDDING_UNIT]->(embedding)
                 WHERE record.is_default_visible
                 RETURN record.record_key, record.name, record.record_family, embedding.unit_kind, distance
                 ORDER BY distance
                 LIMIT 15;",
                vector_literal(384, 0)
            ),
        );
        report_parity_query(
            &connection,
            "vector search with structured filters",
            "parity-awkward",
            "This proves vector-result filtering works, but it is currently top-k then Record filter. The next proof point is prefiltered vector search over an eligible-record graph/projection.",
            &format!(
                "CALL QUERY_VECTOR_INDEX('EmbeddingUnit', 'embedding_hnsw', CAST({}, 'FLOAT[384]'), 100, efs := 50)
                 WITH node AS embedding, distance
                 MATCH (record:Record)-[:HAS_EMBEDDING_UNIT]->(embedding)
                 WHERE record.is_default_visible
                   AND record.record_family = 'creature'
                 RETURN record.record_key, record.name, record.record_family, embedding.unit_kind, distance
                 ORDER BY distance
                 LIMIT 15;",
                vector_literal(384, 0)
            ),
        );
        let projected_graph_ready = prepare_prefiltered_vector_projection(&connection);
        if projected_graph_ready {
            report_parity_query(
                &connection,
                "prefiltered vector search through projected graph",
                "parity-clean",
                "PROJECT_GRAPH_CYPHER returns only eligible EmbeddingUnit nodes before QUERY_VECTOR_INDEX runs, preserving structured-filter-first semantics.",
                &format!(
                    "CALL QUERY_VECTOR_INDEX('eligible_creature_embeddings', 'embedding_hnsw', CAST({}, 'FLOAT[384]'), 15, efs := 50)
                     WITH node AS embedding, distance
                     MATCH (record:Record)-[:HAS_EMBEDDING_UNIT]->(embedding)
                     RETURN record.record_key, record.name, record.record_family, embedding.unit_kind, distance
                     ORDER BY distance
                     LIMIT 15;",
                    vector_literal(384, 0)
                ),
            );
        } else {
            report_parity_blocked(
                "prefiltered vector search through projected graph",
                "parity-blocked",
                "Could not create/query a PROJECT_GRAPH_CYPHER projection over eligible EmbeddingUnit nodes.",
            );
        }
    } else {
        report_parity_blocked(
            "vector search with structured filters",
            "parity-blocked",
            "Could not load/create/query the vector index for this artifact. If the artifact was built with --no-embeddings, this is expected.",
        );
    }

    println!();
    println!("Current parity read");
    println!("-------------------");
    println!("Clean: record-key lookup, scalar structured filters, relationship-backed");
    println!("filters, metric range filters, and facet counts are natural in Ladybug.");
    println!("Potentially better: reference/evidence search can share Record-level");
    println!("structured filters while preserving evidence-level explanations.");
    println!("Semantic/vector: embedding rows and vector index/query mechanics are now");
    println!("checked when the artifact contains embeddings. Post-filtered vector search");
    println!("is marked awkward; PROJECT_GRAPH_CYPHER prefiltered vector search is the");
    println!("critical parity-clean proof when available.");

    Ok(())
}

fn create_bulk_parquet_schema(
    connection: &Connection<'_>,
) -> Result<(), Box<dyn std::error::Error>> {
    for statement in [
        "CREATE NODE TABLE Record(record_key STRING, name STRING, record_family STRING, level INT64, rarity STRING, is_default_visible BOOL, PRIMARY KEY(record_key));",
        "CREATE NODE TABLE SearchDocument(search_doc_key STRING, record_key STRING, title STRING, traits STRING, body STRING, reference_terms STRING, PRIMARY KEY(search_doc_key));",
        "CREATE NODE TABLE EmbeddingUnit(embedding_unit_key STRING, record_key STRING, unit_kind STRING, label STRING, ordinal INT64, dimensions INT64, embedding FLOAT[4], PRIMARY KEY(embedding_unit_key));",
        "CREATE NODE TABLE ContentUnit(content_unit_key STRING, record_key STRING, ordinal INT64, source_kind STRING, visibility STRING, contributes_to_search BOOL, content_json STRING, PRIMARY KEY(content_unit_key));",
        "CREATE NODE TABLE Metric(metric_key_id STRING, metric_domain STRING, metric_key STRING, value_type STRING, PRIMARY KEY(metric_key_id));",
        "CREATE REL TABLE HAS_SEARCH_DOCUMENT(FROM Record TO SearchDocument);",
        "CREATE REL TABLE HAS_EMBEDDING_UNIT(FROM Record TO EmbeddingUnit);",
        "CREATE REL TABLE HAS_CONTENT_UNIT(FROM Record TO ContentUnit);",
        "CREATE REL TABLE HAS_METRIC(FROM Record TO Metric, number_value DOUBLE, text_value STRING, bool_value BOOL);",
        "CREATE REL TABLE REFERENCES(FROM Record TO Record, display_text STRING, reference_text STRING, source_kind STRING, visibility STRING);",
    ] {
        connection.query(statement)?;
    }
    Ok(())
}

fn write_bulk_parquet_files(staging_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    write_record_parquet(staging_dir)?;
    write_search_document_parquet(staging_dir)?;
    write_embedding_unit_parquet(staging_dir)?;
    write_content_unit_parquet(staging_dir)?;
    write_metric_parquet(staging_dir)?;
    write_has_search_document_parquet(staging_dir)?;
    write_has_embedding_unit_parquet(staging_dir)?;
    write_has_content_unit_parquet(staging_dir)?;
    write_has_metric_parquet(staging_dir)?;
    write_references_parquet(staging_dir)?;
    Ok(())
}

fn write_record_parquet(staging_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    write_parquet(
        &staging_dir.join("record.parquet"),
        vec![
            field("record_key", DataType::Utf8, false),
            field("name", DataType::Utf8, false),
            field("record_family", DataType::Utf8, false),
            field("level", DataType::Int64, true),
            field("rarity", DataType::Utf8, true),
            field("is_default_visible", DataType::Boolean, false),
        ],
        vec![
            strings(["spell:fear", "rule:frightened", "feat:intimidating-glare"]),
            strings(["Fear", "Frightened", "Intimidating Glare"]),
            strings(["spell", "rule", "feat"]),
            nullable_ints([Some(1), None, Some(1)]),
            nullable_strings([Some("common"), None, Some("common")]),
            bools([true, true, true]),
        ],
    )
}

fn write_search_document_parquet(staging_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    write_parquet(
        &staging_dir.join("search_document.parquet"),
        vec![
            field("search_doc_key", DataType::Utf8, false),
            field("record_key", DataType::Utf8, false),
            field("title", DataType::Utf8, false),
            field("traits", DataType::Utf8, false),
            field("body", DataType::Utf8, false),
            field("reference_terms", DataType::Utf8, false),
        ],
        vec![
            strings([
                "spell:fear#fts",
                "rule:frightened#fts",
                "feat:intimidating-glare#fts",
            ]),
            strings(["spell:fear", "rule:frightened", "feat:intimidating-glare"]),
            strings(["Fear", "Frightened", "Intimidating Glare"]),
            strings([
                "fear mental emotion",
                "condition fear mental",
                "auditory fear skill",
            ]),
            strings([
                "The target becomes frightened and may flee.",
                "Frightened is a condition that penalizes checks and DCs.",
                "Demoralize without speaking by glaring at a creature.",
            ]),
            strings(["frightened condition", "", "frightened demoralize"]),
        ],
    )
}

fn write_embedding_unit_parquet(staging_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    write_parquet(
        &staging_dir.join("embedding_unit.parquet"),
        vec![
            field("embedding_unit_key", DataType::Utf8, false),
            field("record_key", DataType::Utf8, false),
            field("unit_kind", DataType::Utf8, false),
            field("label", DataType::Utf8, true),
            field("ordinal", DataType::Int64, false),
            field("dimensions", DataType::Int64, false),
            Field::new(
                "embedding",
                DataType::FixedSizeList(Arc::new(field("item", DataType::Float32, false)), 4),
                false,
            ),
        ],
        vec![
            strings([
                "spell:fear#parent",
                "rule:frightened#parent",
                "feat:intimidating-glare#parent",
            ]),
            strings(["spell:fear", "rule:frightened", "feat:intimidating-glare"]),
            strings(["parent", "parent", "parent"]),
            nullable_strings([None, Some("condition"), None]),
            ints([0, 0, 0]),
            ints([4, 4, 4]),
            fixed_f32_lists([
                [0.95, 0.05, 0.0, 0.0],
                [1.0, 0.0, 0.0, 0.0],
                [0.90, 0.03, 0.02, 0.0],
            ])?,
        ],
    )
}

fn write_content_unit_parquet(staging_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    write_parquet(
        &staging_dir.join("content_unit.parquet"),
        vec![
            field("content_unit_key", DataType::Utf8, false),
            field("record_key", DataType::Utf8, false),
            field("ordinal", DataType::Int64, false),
            field("source_kind", DataType::Utf8, false),
            field("visibility", DataType::Utf8, false),
            field("contributes_to_search", DataType::Boolean, false),
            field("content_json", DataType::Utf8, false),
        ],
        vec![
            strings(["spell:fear#content#0", "feat:intimidating-glare#content#0"]),
            strings(["spell:fear", "feat:intimidating-glare"]),
            ints([0, 0]),
            strings(["primary", "primary"]),
            strings(["public", "public"]),
            bools([true, true]),
            strings([
                r#"{"blocks":[{"kind":"paragraph","text":"The target becomes frightened."}]}"#,
                r#"{"blocks":[{"kind":"paragraph","text":"You can Demoralize silently."}]}"#,
            ]),
        ],
    )
}

fn write_metric_parquet(staging_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    write_parquet(
        &staging_dir.join("metric.parquet"),
        vec![
            field("metric_key_id", DataType::Utf8, false),
            field("metric_domain", DataType::Utf8, false),
            field("metric_key", DataType::Utf8, false),
            field("value_type", DataType::Utf8, false),
        ],
        vec![
            strings(["actor:ac.value", "actor:disable.deception.dc.max"]),
            strings(["actor", "actor"]),
            strings(["ac.value", "disable.deception.dc.max"]),
            strings(["number", "number"]),
        ],
    )
}

fn write_has_search_document_parquet(staging_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    write_parquet(
        &staging_dir.join("has_search_document.parquet"),
        vec![
            field("from", DataType::Utf8, false),
            field("to", DataType::Utf8, false),
        ],
        vec![
            strings(["spell:fear", "rule:frightened", "feat:intimidating-glare"]),
            strings([
                "spell:fear#fts",
                "rule:frightened#fts",
                "feat:intimidating-glare#fts",
            ]),
        ],
    )
}

fn write_has_embedding_unit_parquet(staging_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    write_parquet(
        &staging_dir.join("has_embedding_unit.parquet"),
        vec![
            field("from", DataType::Utf8, false),
            field("to", DataType::Utf8, false),
        ],
        vec![
            strings(["spell:fear", "rule:frightened", "feat:intimidating-glare"]),
            strings([
                "spell:fear#parent",
                "rule:frightened#parent",
                "feat:intimidating-glare#parent",
            ]),
        ],
    )
}

fn write_has_content_unit_parquet(staging_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    write_parquet(
        &staging_dir.join("has_content_unit.parquet"),
        vec![
            field("from", DataType::Utf8, false),
            field("to", DataType::Utf8, false),
        ],
        vec![
            strings(["spell:fear", "feat:intimidating-glare"]),
            strings(["spell:fear#content#0", "feat:intimidating-glare#content#0"]),
        ],
    )
}

fn write_has_metric_parquet(staging_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    write_parquet(
        &staging_dir.join("has_metric.parquet"),
        vec![
            field("from", DataType::Utf8, false),
            field("to", DataType::Utf8, false),
            field("number_value", DataType::Float64, true),
            field("text_value", DataType::Utf8, true),
            field("bool_value", DataType::Boolean, true),
        ],
        vec![
            strings(["spell:fear", "rule:frightened"]),
            strings(["actor:disable.deception.dc.max", "actor:ac.value"]),
            nullable_floats([Some(24.0), Some(18.0)]),
            nullable_strings([None, None]),
            nullable_bools([None, None]),
        ],
    )
}

fn write_references_parquet(staging_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    write_parquet(
        &staging_dir.join("references.parquet"),
        vec![
            field("from", DataType::Utf8, false),
            field("to", DataType::Utf8, false),
            field("display_text", DataType::Utf8, true),
            field("reference_text", DataType::Utf8, false),
            field("source_kind", DataType::Utf8, false),
            field("visibility", DataType::Utf8, false),
        ],
        vec![
            strings(["spell:fear", "feat:intimidating-glare"]),
            strings(["rule:frightened", "rule:frightened"]),
            nullable_strings([Some("frightened"), Some("frightened")]),
            strings(["@UUID[rule:frightened]", "@UUID[rule:frightened]"]),
            strings(["inline", "semantic"]),
            strings(["default", "default"]),
        ],
    )
}

fn copy_from_parquet(
    connection: &Connection<'_>,
    table: &str,
    path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("COPY {table} FROM {}", path.display());
    connection.query(&format!(
        "COPY {table} FROM {};",
        cypher_string_literal(&path.to_string_lossy())
    ))?;
    Ok(())
}

fn write_parquet(
    path: &Path,
    fields: Vec<Field>,
    arrays: Vec<ArrayRef>,
) -> Result<(), Box<dyn std::error::Error>> {
    let schema = Arc::new(Schema::new(fields));
    let batch = RecordBatch::try_new(schema.clone(), arrays)?;
    let file = File::create(path)?;
    let mut writer = ArrowWriter::try_new(file, schema, None)?;
    writer.write(&batch)?;
    writer.close()?;
    Ok(())
}

fn field(name: &str, data_type: DataType, nullable: bool) -> Field {
    Field::new(name, data_type, nullable)
}

fn strings<const N: usize>(values: [&str; N]) -> ArrayRef {
    Arc::new(StringArray::from_iter_values(values))
}

fn nullable_strings<const N: usize>(values: [Option<&str>; N]) -> ArrayRef {
    Arc::new(StringArray::from(values.to_vec()))
}

fn ints<const N: usize>(values: [i64; N]) -> ArrayRef {
    Arc::new(Int64Array::from(values.to_vec()))
}

fn nullable_ints<const N: usize>(values: [Option<i64>; N]) -> ArrayRef {
    Arc::new(Int64Array::from(values.to_vec()))
}

fn bools<const N: usize>(values: [bool; N]) -> ArrayRef {
    Arc::new(BooleanArray::from(values.to_vec()))
}

fn nullable_bools<const N: usize>(values: [Option<bool>; N]) -> ArrayRef {
    Arc::new(BooleanArray::from(values.to_vec()))
}

fn nullable_floats<const N: usize>(values: [Option<f64>; N]) -> ArrayRef {
    Arc::new(arrow_array::Float64Array::from(values.to_vec()))
}

fn fixed_f32_lists<const ROWS: usize>(
    rows: [[f32; 4]; ROWS],
) -> Result<ArrayRef, Box<dyn std::error::Error>> {
    let values = rows
        .into_iter()
        .flat_map(|row| row.into_iter())
        .collect::<Vec<_>>();
    let values: ArrayRef = Arc::new(Float32Array::from(values));
    let field = Arc::new(field("item", DataType::Float32, false));
    Ok(Arc::new(FixedSizeListArray::try_new(
        field, 4, values, None,
    )?))
}

fn cypher_string_literal(value: &str) -> String {
    format!("'{}'", value.replace('\\', "\\\\").replace('\'', "\\'"))
}

fn vector_literal(dimensions: usize, seed: usize) -> String {
    let values = (0..dimensions)
        .map(|index| {
            let value = if index == seed % dimensions { 1.0 } else { 0.0 };
            format!("{value:.1}")
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("[{values}]")
}

fn run_on_disk_probe(mode: OnDiskMode) -> Result<(), Box<dyn std::error::Error>> {
    let path = PathBuf::from("target/ladybug-spike/toy.lbug");
    remove_ladybug_files(&path)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    {
        let database = Database::new(&path, SystemConfig::default())?;
        let connection = Connection::new(&database)?;
        let extensions = load_extensions(&connection);
        create_schema(&connection)?;
        insert_fixture(&connection)?;
        if mode.fts && extensions.fts {
            connection.query(
                "CALL CREATE_FTS_INDEX('Record', 'record_fts', ['search_title', 'search_traits', 'search_body'], stemmer := 'porter');",
            )?;
        }
        if mode.vector && extensions.vector {
            connection.query(
                "CALL CREATE_VECTOR_INDEX('EmbeddingUnit', 'unit_hnsw', 'embedding', metric := 'l2');",
            )?;
        }
        connection.query("CHECKPOINT;")?;
    }

    println!("reopening {}", path.display());
    {
        let database = Database::new(&path, SystemConfig::default())?;
        let connection = Connection::new(&database)?;
        let extensions = load_extensions(&connection);
        report_query(
            &connection,
            "record count after reopen",
            "MATCH (r:Record) RETURN count(r);",
        )?;
        if mode.fts && extensions.fts {
            report_query(
                &connection,
                "fts query after reopen",
                "CALL QUERY_FTS_INDEX('Record', 'record_fts', 'frightened condition', top := 10) WITH node AS r, score RETURN r.record_key, score ORDER BY score DESC;",
            )?;
        }
        if mode.vector && extensions.vector {
            report_query(
                &connection,
                "vector query after reopen",
                "CALL QUERY_VECTOR_INDEX('EmbeddingUnit', 'unit_hnsw', CAST([0.98, 0.01, 0.01, 0.00], 'FLOAT[4]'), 4, efs := 50) WITH node AS u, distance RETURN u.unit_key, distance ORDER BY distance;",
            )?;
        }
    }
    remove_ladybug_files(&path)?;
    Ok(())
}

#[derive(Clone, Copy, Debug)]
struct ExtensionStatus {
    fts: bool,
    vector: bool,
    algo: bool,
}

#[derive(Clone, Copy, Debug)]
struct IndexStatus {
    fts: bool,
    vector: bool,
}

fn load_extensions(connection: &Connection<'_>) -> ExtensionStatus {
    ExtensionStatus {
        fts: load_extension(connection, "FTS"),
        vector: load_extension(connection, "VECTOR"),
        algo: load_extension(connection, "ALGO"),
    }
}

fn load_extension(connection: &Connection<'_>, extension: &str) -> bool {
    let load_query = format!("LOAD EXTENSION {extension};");
    if connection.query(&load_query).is_ok() {
        println!("loaded extension {extension}");
        return true;
    }

    let install_query = format!("INSTALL {extension};");
    match connection.query(&install_query) {
        Ok(_) => match connection.query(&load_query) {
            Ok(_) => {
                println!("installed and loaded extension {extension}");
                true
            }
            Err(error) => {
                println!("installed but could not load extension {extension}: {error}");
                false
            }
        },
        Err(error) => {
            println!("extension {extension} unavailable: {error}");
            false
        }
    }
}

fn create_schema(connection: &Connection<'_>) -> Result<(), Box<dyn std::error::Error>> {
    for statement in [
        "CREATE NODE TABLE Record(record_key STRING, title STRING, family STRING, rank INT64, source STRING, rarity STRING, is_default_visible BOOL, search_title STRING, search_traits STRING, search_body STRING, PRIMARY KEY(record_key));",
        "CREATE NODE TABLE Trait(name STRING, PRIMARY KEY(name));",
        "CREATE NODE TABLE Concept(name STRING, PRIMARY KEY(name));",
        "CREATE NODE TABLE EmbeddingUnit(unit_key STRING, parent_record_key STRING, parent_family STRING, parent_rank INT64, parent_default_visible BOOL, unit_kind STRING, text STRING, embedding FLOAT[4], PRIMARY KEY(unit_key));",
        "CREATE REL TABLE HAS_TRAIT(FROM Record TO Trait);",
        "CREATE REL TABLE REFERS_TO(FROM Record TO Record, visibility STRING, source_kind STRING);",
        "CREATE REL TABLE APPLIES_CONCEPT(FROM Record TO Concept);",
        "CREATE REL TABLE HAS_EMBEDDING(FROM Record TO EmbeddingUnit);",
    ] {
        connection.query(statement)?;
    }

    Ok(())
}

fn insert_fixture(connection: &Connection<'_>) -> Result<(), Box<dyn std::error::Error>> {
    for statement in [
        "CREATE (:Trait {name: 'fear'});",
        "CREATE (:Trait {name: 'mental'});",
        "CREATE (:Trait {name: 'illusion'});",
        "CREATE (:Trait {name: 'auditory'});",
        "CREATE (:Concept {name: 'frightened'});",
        "CREATE (:Record {record_key: 'rule:frightened', title: 'Frightened', family: 'rule', rank: 0, source: 'Player Core', rarity: 'common', is_default_visible: true, search_title: 'Frightened', search_traits: 'condition fear mental', search_body: 'Frightened is a condition that penalizes checks and DCs.'});",
        "CREATE (:Record {record_key: 'spell:fear', title: 'Fear', family: 'spell', rank: 1, source: 'Player Core', rarity: 'common', is_default_visible: true, search_title: 'Fear', search_traits: 'fear mental emotion', search_body: 'The target becomes frightened and may flee from the caster.'});",
        "CREATE (:Record {record_key: 'spell:phantasmal-calamity', title: 'Phantasmal Calamity', family: 'spell', rank: 6, source: 'Player Core', rarity: 'common', is_default_visible: true, search_title: 'Phantasmal Calamity', search_traits: 'illusion mental', search_body: 'A vision overwhelms creatures and can leave them frightened.'});",
        "CREATE (:Record {record_key: 'feat:intimidating-glare', title: 'Intimidating Glare', family: 'feat', rank: 1, source: 'Player Core', rarity: 'common', is_default_visible: true, search_title: 'Intimidating Glare', search_traits: 'auditory fear skill', search_body: 'Demoralize without speaking by glaring at a creature.'});",
        "CREATE (:EmbeddingUnit {unit_key: 'rule:frightened#body', parent_record_key: 'rule:frightened', parent_family: 'rule', parent_rank: 0, parent_default_visible: true, unit_kind: 'body', text: 'frightened condition penalty checks DCs', embedding: CAST([1.0, 0.0, 0.0, 0.0], 'FLOAT[4]')});",
        "CREATE (:EmbeddingUnit {unit_key: 'spell:fear#effect', parent_record_key: 'spell:fear', parent_family: 'spell', parent_rank: 1, parent_default_visible: true, unit_kind: 'effect', text: 'spell causes frightened fear condition', embedding: CAST([0.95, 0.05, 0.0, 0.0], 'FLOAT[4]')});",
        "CREATE (:EmbeddingUnit {unit_key: 'spell:phantasmal-calamity#effect', parent_record_key: 'spell:phantasmal-calamity', parent_family: 'spell', parent_rank: 6, parent_default_visible: true, unit_kind: 'effect', text: 'mental illusion damage frightened vision', embedding: CAST([0.82, 0.12, 0.04, 0.0], 'FLOAT[4]')});",
        "CREATE (:EmbeddingUnit {unit_key: 'feat:intimidating-glare#effect', parent_record_key: 'feat:intimidating-glare', parent_family: 'feat', parent_rank: 1, parent_default_visible: true, unit_kind: 'effect', text: 'demoralize fear glare frightened', embedding: CAST([0.90, 0.03, 0.02, 0.0], 'FLOAT[4]')});",
    ] {
        connection.query(statement)?;
    }

    for statement in [
        "MATCH (r:Record {record_key: 'spell:fear'}), (t:Trait {name: 'fear'}) CREATE (r)-[:HAS_TRAIT]->(t);",
        "MATCH (r:Record {record_key: 'spell:fear'}), (t:Trait {name: 'mental'}) CREATE (r)-[:HAS_TRAIT]->(t);",
        "MATCH (r:Record {record_key: 'spell:phantasmal-calamity'}), (t:Trait {name: 'illusion'}) CREATE (r)-[:HAS_TRAIT]->(t);",
        "MATCH (r:Record {record_key: 'spell:phantasmal-calamity'}), (t:Trait {name: 'mental'}) CREATE (r)-[:HAS_TRAIT]->(t);",
        "MATCH (r:Record {record_key: 'feat:intimidating-glare'}), (t:Trait {name: 'auditory'}) CREATE (r)-[:HAS_TRAIT]->(t);",
        "MATCH (r:Record {record_key: 'feat:intimidating-glare'}), (t:Trait {name: 'fear'}) CREATE (r)-[:HAS_TRAIT]->(t);",
        "MATCH (r:Record {record_key: 'spell:fear'}), (target:Record {record_key: 'rule:frightened'}) CREATE (r)-[:REFERS_TO {visibility: 'default', source_kind: 'inline'}]->(target);",
        "MATCH (r:Record {record_key: 'spell:phantasmal-calamity'}), (target:Record {record_key: 'rule:frightened'}) CREATE (r)-[:REFERS_TO {visibility: 'default', source_kind: 'inline'}]->(target);",
        "MATCH (r:Record {record_key: 'feat:intimidating-glare'}), (target:Record {record_key: 'rule:frightened'}) CREATE (r)-[:REFERS_TO {visibility: 'default', source_kind: 'semantic'}]->(target);",
        "MATCH (r:Record {record_key: 'spell:fear'}), (c:Concept {name: 'frightened'}) CREATE (r)-[:APPLIES_CONCEPT]->(c);",
        "MATCH (r:Record {record_key: 'spell:phantasmal-calamity'}), (c:Concept {name: 'frightened'}) CREATE (r)-[:APPLIES_CONCEPT]->(c);",
        "MATCH (r:Record), (u:EmbeddingUnit) WHERE r.record_key = u.parent_record_key CREATE (r)-[:HAS_EMBEDDING]->(u);",
    ] {
        connection.query(statement)?;
    }

    Ok(())
}

fn create_indexes(connection: &Connection<'_>, extensions: &ExtensionStatus) -> IndexStatus {
    let fts = extensions.fts
        && connection
            .query(
        "CALL CREATE_FTS_INDEX('Record', 'record_fts', ['search_title', 'search_traits', 'search_body'], stemmer := 'porter');",
            )
            .map(|_| true)
            .unwrap_or_else(|error| {
                println!("could not create FTS index: {error}");
                false
            });
    let vector = extensions.vector
        && connection
            .query("CALL CREATE_VECTOR_INDEX('EmbeddingUnit', 'unit_hnsw', 'embedding', metric := 'l2');")
            .map(|_| true)
            .unwrap_or_else(|error| {
                println!("could not create vector index: {error}");
                false
            });
    IndexStatus { fts, vector }
}

fn report_query(
    connection: &Connection<'_>,
    label: &str,
    query: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!();
    println!("-- {label}");
    let mut result = connection.query(query)?;
    for row in &mut result {
        println!("{}", format_row(&row));
    }
    println!(
        "timing: compile={:.3}ms execute={:.3}ms",
        result.get_compiling_time(),
        result.get_execution_time()
    );
    Ok(())
}

fn report_evaluation_query(
    connection: &Connection<'_>,
    label: &str,
    current_read: &str,
    product_question: &str,
    graph_value_test: &str,
    query: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!();
    println!("## {label}");
    println!("current read: {current_read}");
    println!("product question: {product_question}");
    println!("graph value test: {graph_value_test}");
    report_query(connection, "query result", query)
}

fn report_parity_query(
    connection: &Connection<'_>,
    label: &str,
    classification: &str,
    note: &str,
    query: &str,
) {
    println!();
    println!("## {label}");
    println!("classification: {classification}");
    println!("note: {note}");
    println!();
    println!("-- query result");
    match connection.query(query) {
        Ok(mut result) => {
            for row in &mut result {
                println!("{}", format_row(&row));
            }
            println!(
                "timing: compile={:.3}ms execute={:.3}ms",
                result.get_compiling_time(),
                result.get_execution_time()
            );
        }
        Err(error) => {
            println!("query failed: {error}");
        }
    }
}

fn report_parity_blocked(label: &str, classification: &str, note: &str) {
    println!();
    println!("## {label}");
    println!("classification: {classification}");
    println!("note: {note}");
}

fn prepare_baseline_fts(connection: &Connection<'_>) -> bool {
    if !load_extension(connection, "FTS") {
        return false;
    }

    let _ = connection.query(
        "CALL CREATE_FTS_INDEX(
            'SearchDocument',
            'search_document_fts',
            [
                'title', 'aliases', 'traits', 'taxonomy_terms', 'constraint_terms',
                'mechanic_terms', 'source_terms', 'metric_terms', 'headings', 'body',
                'facts', 'reference_terms', 'embedded_content'
            ],
            stemmer := 'porter'
        );",
    );
    let _ = connection.query(
        "CALL CREATE_FTS_INDEX(
            'EvidenceUnit',
            'evidence_unit_fts',
            ['label', 'search_text'],
            stemmer := 'porter'
        );",
    );

    connection
        .query(
            "CALL QUERY_FTS_INDEX('SearchDocument', 'search_document_fts', 'frightened', top := 1)
             WITH node AS doc, score
             RETURN doc.search_doc_key, score
             LIMIT 1;",
        )
        .is_ok()
        && connection
            .query(
                "CALL QUERY_FTS_INDEX('EvidenceUnit', 'evidence_unit_fts', 'frightened', top := 1)
                 WITH node AS evidence, score
                 RETURN evidence.evidence_unit_key, score
                 LIMIT 1;",
            )
            .is_ok()
}

fn prepare_baseline_vector(connection: &Connection<'_>) -> bool {
    if !load_extension(connection, "VECTOR") {
        return false;
    }

    let _ = connection.query(
        "CALL CREATE_VECTOR_INDEX(
            'EmbeddingUnit',
            'embedding_hnsw',
            'embedding',
            metric := 'cosine'
        );",
    );

    connection
        .query(&format!(
            "CALL QUERY_VECTOR_INDEX('EmbeddingUnit', 'embedding_hnsw', CAST({}, 'FLOAT[384]'), 1, efs := 50)
             WITH node AS embedding, distance
             RETURN embedding.embedding_unit_key, distance
             LIMIT 1;",
            vector_literal(384, 0)
        ))
        .is_ok()
}

fn prepare_prefiltered_vector_projection(connection: &Connection<'_>) -> bool {
    let _ = connection.query("CALL DROP_PROJECTED_GRAPH('eligible_creature_embeddings');");

    let projection = "MATCH (record:Record)-[:HAS_EMBEDDING_UNIT]->(embedding:EmbeddingUnit) \
                      WHERE record.is_default_visible AND record.record_family = 'creature' \
                      RETURN embedding";
    let create_query = format!(
        "CALL PROJECT_GRAPH_CYPHER('eligible_creature_embeddings', {});",
        cypher_string_literal(projection)
    );

    match connection.query(&create_query) {
        Ok(mut result) => for _ in &mut result {},
        Err(error) => {
            println!("could not create PROJECT_GRAPH_CYPHER projection: {error}");
            return false;
        }
    }

    connection
        .query(&format!(
            "CALL QUERY_VECTOR_INDEX('eligible_creature_embeddings', 'embedding_hnsw', CAST({}, 'FLOAT[384]'), 1, efs := 50)
             WITH node AS embedding, distance
             MATCH (record:Record)-[:HAS_EMBEDDING_UNIT]->(embedding)
             WHERE record.record_family = 'creature'
             RETURN record.record_key, distance
             LIMIT 1;",
            vector_literal(384, 0)
        ))
        .is_ok()
}

fn format_row(row: &[Value]) -> String {
    row.iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(" | ")
}

fn remove_ladybug_files(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    for suffix in ["", ".wal", ".wal.checkpoint", ".shadow", ".tmp"] {
        let candidate = path_with_suffix(path, suffix);
        match fs::remove_file(&candidate) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(Box::new(error)),
        }
    }
    Ok(())
}

fn path_with_suffix(path: &Path, suffix: &str) -> PathBuf {
    let mut value = path.as_os_str().to_os_string();
    value.push(suffix);
    PathBuf::from(value)
}
