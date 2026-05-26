use graphqlite::Graph;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::time::Instant;

pub(crate) fn run_graphqlite_smoke_probe() -> Result<(), Box<dyn std::error::Error>> {
    let graph = Graph::open_in_memory()?;
    let id_map = graph.insert_nodes_bulk([
        (
            "record:spells-srd:4koZzrnMXhhosn0D",
            vec![
                ("record_key", "spells-srd:4koZzrnMXhhosn0D"),
                ("name", "Fear"),
                ("family", "spell"),
            ],
            "Record",
        ),
        (
            "trait:fear",
            vec![("value", "fear"), ("field", "traits")],
            "Trait",
        ),
        (
            "record:conditionitems:TBSHQspnbcqxsmjL",
            vec![
                ("record_key", "conditionitems:TBSHQspnbcqxsmjL"),
                ("name", "Frightened"),
                ("family", "rule"),
            ],
            "Record",
        ),
    ])?;
    let edges_inserted = graph.insert_edges_bulk(
        [
            (
                "record:spells-srd:4koZzrnMXhhosn0D",
                "trait:fear",
                vec![("source", "record_traits")],
                "HAS_TRAIT",
            ),
            (
                "record:spells-srd:4koZzrnMXhhosn0D",
                "record:conditionitems:TBSHQspnbcqxsmjL",
                vec![("source", "reference_edges")],
                "REFERENCES",
            ),
        ],
        &id_map,
    )?;

    let rows = graph.query(
        "MATCH (record:Record {name: 'Fear'})-[:HAS_TRAIT]->(trait:Trait)
         RETURN record.record_key, trait.value",
    )?;
    let first = rows
        .get(0)
        .ok_or("GraphQLite smoke query returned no rows")?;
    let record_key: String = first.get("record.record_key")?;
    let trait_value: String = first.get("trait.value")?;
    let stats = graph.stats()?;

    println!("GraphQLite smoke probe");
    println!("======================");
    println!("nodes_inserted: {}", id_map.len());
    println!("edges_inserted: {edges_inserted}");
    println!("graph_nodes: {}", stats.node_count);
    println!("graph_edges: {}", stats.edge_count);
    println!("query_result: {record_key} HAS_TRAIT {trait_value}");

    Ok(())
}

pub(crate) fn run_graphqlite_read_patterns(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let graph = Graph::open(path)?;
    let stats = graph.stats()?;
    let connection = graph.connection().sqlite_connection();

    println!("GraphQLite read-pattern probe");
    println!("=============================");
    println!("artifact: {}", path.display());
    println!(
        "graph_stats: {} nodes, {} edges",
        stats.node_count, stats.edge_count
    );
    println!();

    print_projection_metadata(connection)?;
    print_graphqlite_tables(connection)?;
    run_cypher_trait_probe(&graph)?;
    run_cypher_reference_probe(&graph)?;
    run_raw_sql_trait_join_probe(connection)?;
    run_raw_sql_reference_join_probe(connection)?;
    run_raw_sql_content_join_probe(connection)?;

    Ok(())
}

pub(crate) fn run_graphqlite_product_patterns(
    path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let graph = Graph::open(path)?;
    let stats = graph.stats()?;

    println!("GraphQLite product-pattern probe");
    println!("=================================");
    println!("artifact: {}", path.display());
    println!(
        "graph_stats: {} nodes, {} edges",
        stats.node_count, stats.edge_count
    );
    println!();

    run_named_query(
        &graph,
        "spells with fear trait that reference frightened",
        "MATCH (spell:Record)-[:HAS_TRAIT]->(:Trait {value: 'fear'})
         MATCH (spell)-[edge:REFERENCES]->(target:Record {record_key: 'conditionitems:TBSHQspnbcqxsmjL'})
         WHERE spell.family = 'spell'
         RETURN spell.record_key, spell.name, edge.display_text
         LIMIT 12",
        &["spell.record_key", "spell.name", "edge.display_text"],
    )?;

    run_named_query(
        &graph,
        "records that reference frightened and also have a fear trait",
        "MATCH (record:Record)-[:HAS_TRAIT]->(:Trait {value: 'fear'})
         MATCH (record)-[edge:REFERENCES]->(target:Record {record_key: 'conditionitems:TBSHQspnbcqxsmjL'})
         RETURN record.record_key, record.name, record.family, edge.default_graph
         LIMIT 12",
        &[
            "record.record_key",
            "record.name",
            "record.family",
            "edge.default_graph",
        ],
    )?;

    run_named_query(
        &graph,
        "records sharing a target referenced by Fear",
        "MATCH (seed:Record {record_key: 'spells-srd:4koZzrnMXhhosn0D'})-[:REFERENCES]->(target:Record)
         MATCH (other:Record)-[:REFERENCES]->(target)
         WHERE other.record_key <> seed.record_key
         RETURN target.record_key, target.name, other.record_key, other.name, other.family
         LIMIT 12",
        &[
            "target.record_key",
            "target.name",
            "other.record_key",
            "other.name",
            "other.family",
        ],
    )?;

    run_named_query(
        &graph,
        "records sharing traits with Fear",
        "MATCH (seed:Record {record_key: 'spells-srd:4koZzrnMXhhosn0D'})-[:HAS_TRAIT]->(trait:Trait)
         MATCH (other:Record)-[:HAS_TRAIT]->(trait)
         WHERE other.record_key <> seed.record_key
         RETURN trait.value, other.record_key, other.name, other.family
         LIMIT 16",
        &[
            "trait.value",
            "other.record_key",
            "other.name",
            "other.family",
        ],
    )?;

    run_named_query(
        &graph,
        "spell similarity by shared non-broad traits with Fear",
        "MATCH (seed:Record {record_key: 'spells-srd:4koZzrnMXhhosn0D'})-[:HAS_TRAIT]->(trait:Trait)
         MATCH (other:Record)-[:HAS_TRAIT]->(trait)
         WHERE other.record_key <> seed.record_key
           AND other.family = seed.family
           AND trait.value <> 'concentrate'
           AND trait.value <> 'manipulate'
         RETURN other.record_key, other.name, count(trait) AS shared_trait_count
         ORDER BY shared_trait_count DESC, other.name
         LIMIT 16",
        &["other.record_key", "other.name", "shared_trait_count"],
    )?;

    run_named_query(
        &graph,
        "spell similarity by shared referenced mechanics with Fear",
        "MATCH (seed:Record {record_key: 'spells-srd:4koZzrnMXhhosn0D'})-[:REFERENCES]->(target:Record)
         MATCH (other:Record)-[:REFERENCES]->(target)
         WHERE other.record_key <> seed.record_key
           AND other.family = seed.family
         RETURN other.record_key, other.name, count(target) AS shared_reference_count
         ORDER BY shared_reference_count DESC, other.name
         LIMIT 16",
        &["other.record_key", "other.name", "shared_reference_count"],
    )?;

    run_scored_trait_similarity(&graph)?;
    run_scored_reference_similarity(&graph)?;

    run_named_query(
        &graph,
        "variant group neighbors",
        "MATCH (record:Record)-[:VARIANT_OF]->(group:VariantGroup)<-[:VARIANT_OF]-(sibling:Record)
         WHERE record.record_key = 'spells-srd:4koZzrnMXhhosn0D'
           AND sibling.record_key <> record.record_key
         RETURN group.key, group.base_name, sibling.record_key, sibling.name, sibling.family
         LIMIT 12",
        &[
            "group.key",
            "group.base_name",
            "sibling.record_key",
            "sibling.name",
            "sibling.family",
        ],
    )?;

    run_named_query(
        &graph,
        "creature content units with embedded gear/effects",
        "MATCH (creature:Record)-[:HAS_CONTENT_UNIT]->(content:ContentUnit)
         WHERE creature.family = 'creature'
           AND content.label <> ''
         RETURN creature.record_key, creature.name, content.source_kind, content.label
         LIMIT 16",
        &[
            "creature.record_key",
            "creature.name",
            "content.source_kind",
            "content.label",
        ],
    )?;

    run_named_query(
        &graph,
        "content units that mention canonical records",
        "MATCH (record:Record)-[:HAS_CONTENT_UNIT]->(content:ContentUnit)-[mention:MENTIONS]->(target:Record)
         RETURN record.record_key, record.name, content.source_kind, content.label, target.record_key, target.name, mention.display_text
         LIMIT 16",
        &[
            "record.record_key",
            "record.name",
            "content.source_kind",
            "content.label",
            "target.record_key",
            "target.name",
            "mention.display_text",
        ],
    )?;

    run_named_query(
        &graph,
        "same-evidence mechanic bridge",
        "MATCH (record:Record)-[:HAS_EVIDENCE_UNIT]->(evidence:EvidenceUnit)-[:EVIDENCE_REFERENCES]->(:Record {record_key: 'actionspf2e:2u915NdUyQan6uKF'})
         MATCH (evidence)-[:EVIDENCE_REFERENCES]->(:Record {record_key: 'conditionitems:TBSHQspnbcqxsmjL'})
         WHERE record.default_visible = true
         RETURN record.record_key, record.name, record.family, evidence.label, evidence.source_kind, evidence.unit_kind
         LIMIT 16",
        &[
            "record.record_key",
            "record.name",
            "record.family",
            "evidence.label",
            "evidence.source_kind",
            "evidence.unit_kind",
        ],
    )?;

    run_named_query(
        &graph,
        "mechanic impact map rows for Frightened",
        "MATCH (record:Record)-[:HAS_EVIDENCE_UNIT]->(evidence:EvidenceUnit)-[:EVIDENCE_REFERENCES]->(:Record {record_key: 'conditionitems:TBSHQspnbcqxsmjL'})
         WHERE record.default_visible = true
         RETURN record.family, evidence.source_kind, evidence.unit_kind, record.record_key, evidence.evidence_unit_key
         LIMIT 16",
        &[
            "record.family",
            "evidence.source_kind",
            "evidence.unit_kind",
            "record.record_key",
            "evidence.evidence_unit_key",
        ],
    )?;

    run_named_query(
        &graph,
        "records sharing taxonomy families with Fear",
        "MATCH (seed:Record {record_key: 'spells-srd:4koZzrnMXhhosn0D'})-[:HAS_TAXONOMY_FAMILY]->(family:TaxonomyFamily)
         MATCH (other:Record)-[:HAS_TAXONOMY_FAMILY]->(family)
         WHERE other.record_key <> seed.record_key
         RETURN family.value, other.record_key, other.name, other.family
         LIMIT 16",
        &[
            "family.value",
            "other.record_key",
            "other.name",
            "other.family",
        ],
    )?;

    run_named_query(
        &graph,
        "sample records with taxonomy families",
        "MATCH (record:Record)-[:HAS_TAXONOMY_FAMILY]->(family:TaxonomyFamily)
         RETURN family.value, record.record_key, record.name, record.family
         LIMIT 16",
        &[
            "family.value",
            "record.record_key",
            "record.name",
            "record.family",
        ],
    )?;

    run_named_query(
        &graph,
        "publication neighbors for Fear",
        "MATCH (seed:Record {record_key: 'spells-srd:4koZzrnMXhhosn0D'})-[:PUBLISHED_IN]->(publication:Publication)<-[:PUBLISHED_IN]-(other:Record)
         WHERE other.record_key <> seed.record_key
           AND other.family = seed.family
         RETURN publication.title, publication.family, publication.remaster, other.record_key, other.name
         LIMIT 16",
        &[
            "publication.title",
            "publication.family",
            "publication.remaster",
            "other.record_key",
            "other.name",
        ],
    )?;

    run_named_query(
        &graph,
        "pack neighbors for Fear",
        "MATCH (seed:Record {record_key: 'spells-srd:4koZzrnMXhhosn0D'})-[:IN_PACK]->(pack:Pack)<-[:IN_PACK]-(other:Record)
         WHERE other.record_key <> seed.record_key
           AND other.family = seed.family
         RETURN pack.name, pack.label, other.record_key, other.name
         LIMIT 16",
        &["pack.name", "pack.label", "other.record_key", "other.name"],
    )?;

    run_named_query(
        &graph,
        "sample variant axes",
        "MATCH (record:Record)-[:HAS_VARIANT_AXIS]->(axis:VariantAxis)
         RETURN axis.value, record.record_key, record.name, record.family
         LIMIT 16",
        &[
            "axis.value",
            "record.record_key",
            "record.name",
            "record.family",
        ],
    )?;

    run_named_query(
        &graph,
        "variant progression navigation: Dread Ampoule",
        "MATCH (record:Record)-[:VARIANT_OF]->(group:VariantGroup {base_name: 'Dread Ampoule'})
         RETURN record.record_key, record.name, record.family, group.base_name
         LIMIT 16",
        &[
            "record.record_key",
            "record.name",
            "record.family",
            "group.base_name",
        ],
    )?;

    run_named_query(
        &graph,
        "remaster records sharing references",
        "MATCH (legacy:Record)-[:REMASTERED_BY]->(remaster:Record)
         MATCH (legacy)-[:REFERENCES]->(target:Record)<-[:REFERENCES]-(remaster)
         RETURN legacy.record_key, legacy.name, remaster.record_key, remaster.name, target.record_key
         LIMIT 16",
        &[
            "legacy.record_key",
            "legacy.name",
            "remaster.record_key",
            "remaster.name",
            "target.record_key",
        ],
    )?;

    Ok(())
}

fn run_scored_trait_similarity(graph: &Graph) -> Result<(), Box<dyn std::error::Error>> {
    let started = Instant::now();
    let rows = graph.query(
        "MATCH (seed:Record {record_key: 'spells-srd:4koZzrnMXhhosn0D'})-[:HAS_TRAIT]->(trait:Trait)
         MATCH (other:Record)-[:HAS_TRAIT]->(trait)
         WHERE other.record_key <> seed.record_key
           AND other.family = seed.family
           AND trait.value <> 'concentrate'
           AND trait.value <> 'manipulate'
         RETURN other.record_key, other.name, trait.value
         LIMIT 5000",
    )?;
    let mut candidates: BTreeMap<String, SimilarityCandidate> = BTreeMap::new();
    for row in rows.iter() {
        let record_key: String = row.get("other.record_key")?;
        let name: String = row.get("other.name")?;
        let trait_value: String = row.get("trait.value")?;
        candidates
            .entry(record_key.clone())
            .or_insert_with(|| SimilarityCandidate::new(record_key, name))
            .evidence
            .insert(trait_value);
    }
    let mut ranked: Vec<_> = candidates.into_values().collect();
    ranked.sort_by(|left, right| {
        right
            .evidence
            .len()
            .cmp(&left.evidence.len())
            .then_with(|| left.name.cmp(&right.name))
            .then_with(|| left.record_key.cmp(&right.record_key))
    });
    println!(
        "Rust-scored spell similarity by shared non-broad traits with Fear ({} candidates from {} evidence rows in {}):",
        ranked.len(),
        rows.len(),
        format_duration(started.elapsed())
    );
    for candidate in ranked.iter().take(16) {
        println!(
            "  {} | {} | {} shared: {}",
            candidate.record_key,
            candidate.name,
            candidate.evidence.len(),
            candidate
                .evidence
                .iter()
                .cloned()
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    println!();
    Ok(())
}

fn run_scored_reference_similarity(graph: &Graph) -> Result<(), Box<dyn std::error::Error>> {
    let started = Instant::now();
    let rows = graph.query(
        "MATCH (seed:Record {record_key: 'spells-srd:4koZzrnMXhhosn0D'})-[:REFERENCES]->(target:Record)
         MATCH (other:Record)-[:REFERENCES]->(target)
         WHERE other.record_key <> seed.record_key
           AND other.family = seed.family
         RETURN other.record_key, other.name, target.record_key
         LIMIT 5000",
    )?;
    let mut candidates: BTreeMap<String, SimilarityCandidate> = BTreeMap::new();
    for row in rows.iter() {
        let record_key: String = row.get("other.record_key")?;
        let name: String = row.get("other.name")?;
        let target_key: String = row.get("target.record_key")?;
        candidates
            .entry(record_key.clone())
            .or_insert_with(|| SimilarityCandidate::new(record_key, name))
            .evidence
            .insert(target_key);
    }
    let mut ranked: Vec<_> = candidates.into_values().collect();
    ranked.sort_by(|left, right| {
        right
            .evidence
            .len()
            .cmp(&left.evidence.len())
            .then_with(|| left.name.cmp(&right.name))
            .then_with(|| left.record_key.cmp(&right.record_key))
    });
    println!(
        "Rust-scored spell similarity by shared referenced mechanics with Fear ({} candidates from {} evidence rows in {}):",
        ranked.len(),
        rows.len(),
        format_duration(started.elapsed())
    );
    for candidate in ranked.iter().take(16) {
        println!(
            "  {} | {} | {} shared references",
            candidate.record_key,
            candidate.name,
            candidate.evidence.len()
        );
    }
    println!();
    Ok(())
}

struct SimilarityCandidate {
    record_key: String,
    name: String,
    evidence: BTreeSet<String>,
}

impl SimilarityCandidate {
    fn new(record_key: String, name: String) -> Self {
        Self {
            record_key,
            name,
            evidence: BTreeSet::new(),
        }
    }
}

fn run_named_query(
    graph: &Graph,
    label: &str,
    query: &str,
    columns: &[&str],
) -> Result<(), Box<dyn std::error::Error>> {
    let started = Instant::now();
    let rows = graph.query(query)?;
    println!(
        "{label} ({} rows in {}):",
        rows.len(),
        format_duration(started.elapsed())
    );
    for row in rows.iter().take(16) {
        let values = columns
            .iter()
            .map(|column| display_cell(row, column))
            .collect::<Result<Vec<_>, _>>()?;
        println!("  {}", values.join(" | "));
    }
    println!();
    Ok(())
}

fn display_cell(row: &graphqlite::Row, column: &str) -> Result<String, Box<dyn std::error::Error>> {
    if let Ok(value) = row.get::<String>(column) {
        return Ok(value);
    }
    if let Ok(value) = row.get::<bool>(column) {
        return Ok(value.to_string());
    }
    if let Ok(value) = row.get::<i64>(column) {
        return Ok(value.to_string());
    }
    if let Ok(value) = row.get::<f64>(column) {
        return Ok(value.to_string());
    }
    Ok("<unprintable>".to_string())
}

fn print_projection_metadata(
    connection: &rusqlite::Connection,
) -> Result<(), Box<dyn std::error::Error>> {
    let started = Instant::now();
    let mut statement = connection.prepare(
        "SELECT key, value
         FROM artifact_metadata
         WHERE key LIKE 'graphqlite_%'
         ORDER BY key",
    )?;
    let rows = statement
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    println!(
        "metadata rows ({} in {}):",
        rows.len(),
        format_duration(started.elapsed())
    );
    for (key, value) in rows {
        println!("  {key}: {value}");
    }
    println!();
    Ok(())
}

fn print_graphqlite_tables(
    connection: &rusqlite::Connection,
) -> Result<(), Box<dyn std::error::Error>> {
    let started = Instant::now();
    let mut statement = connection.prepare(
        "SELECT name
         FROM sqlite_master
         WHERE type = 'table'
           AND name IN (
             'nodes',
             'edges',
             'node_labels',
             'property_keys',
             'node_props_text',
             'node_props_int',
             'node_props_real',
             'node_props_bool',
             'edge_props_text',
             'edge_props_int',
             'edge_props_real',
             'edge_props_bool'
           )
         ORDER BY name",
    )?;
    let tables = statement
        .query_map([], |row| row.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    println!(
        "raw GraphQLite tables visible to SQLite ({} in {}):",
        tables.len(),
        format_duration(started.elapsed())
    );
    for table in tables {
        println!("  {table}");
    }
    println!();
    Ok(())
}

fn run_cypher_trait_probe(graph: &Graph) -> Result<(), Box<dyn std::error::Error>> {
    let started = Instant::now();
    let rows = graph.query(
        "MATCH (record:Record)-[:HAS_TRAIT]->(trait:Trait {value: 'fear'})
         WHERE record.family = 'spell'
         RETURN record.record_key, record.name
         LIMIT 8",
    )?;
    println!(
        "cypher trait lookup ({} rows in {}):",
        rows.len(),
        format_duration(started.elapsed())
    );
    for row in rows.iter().take(8) {
        let record_key: String = row.get("record.record_key")?;
        let name: String = row.get("record.name")?;
        println!("  {record_key} | {name}");
    }
    println!();
    Ok(())
}

fn run_cypher_reference_probe(graph: &Graph) -> Result<(), Box<dyn std::error::Error>> {
    let started = Instant::now();
    let rows = graph.query(
        "MATCH (record:Record)-[edge:REFERENCES]->(target:Record)
         WHERE target.record_key = 'conditionitems:TBSHQspnbcqxsmjL'
         RETURN record.record_key, record.name, edge.default_graph
         LIMIT 8",
    )?;
    println!(
        "cypher reference lookup ({} rows in {}):",
        rows.len(),
        format_duration(started.elapsed())
    );
    for row in rows.iter().take(8) {
        let record_key: String = row.get("record.record_key")?;
        let name: String = row.get("record.name")?;
        let default_graph: bool = row.get("edge.default_graph")?;
        println!("  {record_key} | {name} | default_graph={default_graph}");
    }
    println!();
    Ok(())
}

fn run_raw_sql_trait_join_probe(
    connection: &rusqlite::Connection,
) -> Result<(), Box<dyn std::error::Error>> {
    let started = Instant::now();
    let mut statement = connection.prepare(
        "SELECT records.record_key, records.name, trait_value.value
         FROM records
         JOIN property_keys record_id_key ON record_id_key.key = 'id'
         JOIN node_props_text record_id
           ON record_id.key_id = record_id_key.id
          AND record_id.value = 'record:' || records.record_key
         JOIN node_labels record_label
           ON record_label.node_id = record_id.node_id
          AND record_label.label = 'Record'
         JOIN edges trait_edge
           ON trait_edge.source_id = record_id.node_id
          AND trait_edge.type = 'HAS_TRAIT'
         JOIN node_labels trait_label
           ON trait_label.node_id = trait_edge.target_id
          AND trait_label.label = 'Trait'
         JOIN property_keys trait_value_key ON trait_value_key.key = 'value'
         JOIN node_props_text trait_value
           ON trait_value.node_id = trait_edge.target_id
          AND trait_value.key_id = trait_value_key.id
         WHERE records.record_family = 'spell'
           AND records.level <= 3
           AND trait_value.value = 'fear'
         ORDER BY records.normalized_name, records.record_key
         LIMIT 8",
    )?;
    let rows = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    println!(
        "raw SQLite join: records + GraphQLite HAS_TRAIT ({} rows in {}):",
        rows.len(),
        format_duration(started.elapsed())
    );
    for (record_key, name, trait_value) in rows {
        println!("  {record_key} | {name} | trait={trait_value}");
    }
    println!();
    Ok(())
}

fn run_raw_sql_reference_join_probe(
    connection: &rusqlite::Connection,
) -> Result<(), Box<dyn std::error::Error>> {
    let started = Instant::now();
    let mut statement = connection.prepare(
        "SELECT source_records.record_key, source_records.name, edge_default.value
         FROM records AS target_records
         JOIN property_keys id_key ON id_key.key = 'id'
         JOIN node_props_text target_node
           ON target_node.key_id = id_key.id
          AND target_node.value = 'record:' || target_records.record_key
         JOIN edges reference_edge
           ON reference_edge.target_id = target_node.node_id
          AND reference_edge.type = 'REFERENCES'
         JOIN node_props_text source_node
           ON source_node.key_id = id_key.id
          AND source_node.node_id = reference_edge.source_id
         JOIN records AS source_records
           ON source_node.value = 'record:' || source_records.record_key
         JOIN property_keys default_key ON default_key.key = 'default_graph'
         JOIN edge_props_bool edge_default
           ON edge_default.edge_id = reference_edge.id
          AND edge_default.key_id = default_key.id
         WHERE target_records.record_key = 'conditionitems:TBSHQspnbcqxsmjL'
           AND edge_default.value = 1
         ORDER BY source_records.normalized_name, source_records.record_key
         LIMIT 8",
    )?;
    let rows = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    println!(
        "raw SQLite join: records + GraphQLite REFERENCES backlinks ({} rows in {}):",
        rows.len(),
        format_duration(started.elapsed())
    );
    for (record_key, name, default_graph) in rows {
        println!(
            "  {record_key} | {name} | default_graph={}",
            default_graph != 0
        );
    }
    println!();
    Ok(())
}

fn run_raw_sql_content_join_probe(
    connection: &rusqlite::Connection,
) -> Result<(), Box<dyn std::error::Error>> {
    let started = Instant::now();
    let mut statement = connection.prepare(
        "SELECT records.record_key, records.name, content_label.value
         FROM records
         JOIN property_keys record_id_key ON record_id_key.key = 'id'
         JOIN node_props_text record_node
           ON record_node.key_id = record_id_key.id
          AND record_node.value = 'record:' || records.record_key
         JOIN edges content_edge
           ON content_edge.source_id = record_node.node_id
          AND content_edge.type = 'HAS_CONTENT_UNIT'
         JOIN property_keys content_label_key ON content_label_key.key = 'label'
         JOIN node_props_text content_label
           ON content_label.node_id = content_edge.target_id
          AND content_label.key_id = content_label_key.id
         WHERE records.record_family = 'creature'
           AND content_label.value <> ''
         ORDER BY records.normalized_name, records.record_key, content_label.value
         LIMIT 8",
    )?;
    let rows = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    println!(
        "raw SQLite join: records + GraphQLite content units ({} rows in {}):",
        rows.len(),
        format_duration(started.elapsed())
    );
    for (record_key, name, label) in rows {
        println!("  {record_key} | {name} | content={label}");
    }
    println!();
    Ok(())
}

fn format_duration(duration: std::time::Duration) -> String {
    if duration.as_secs() > 0 {
        format!("{}.{:03}s", duration.as_secs(), duration.subsec_millis())
    } else {
        format!("{}ms", duration.as_millis())
    }
}
