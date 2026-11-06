//! Authentic pre-schema27 reference rows for initially unpublished fixtures.
use crate::{
    IndexBuildInput, IndexError,
    codec::{gzip, sha256},
    input::{SourceRelationshipDetails, SourceStoredRelationship},
    persistence::ProjectionRow,
    projections::{ProjectionIds, root_rows},
};
use atlas_domain::RecordKey;
use atlas_foundry_model::encode_snapshot;
use rusqlite::{Connection, params, types::Value};
use std::collections::{BTreeMap, BTreeSet};

fn nullable(value: Option<String>) -> Value {
    value.map(Value::Text).unwrap_or(Value::Null)
}

fn require_unpublished_reference(c: &Connection) -> Result<(), IndexError> {
    for table in [
        "artifact_context",
        "packs",
        "records",
        "projection_allocations",
        "record_bodies",
        "prepared_content",
        "relationship_occurrences",
        "developer_diagnostics",
        "verified_aliases",
        "remaster_pairs",
        "query_field_catalog",
        "semantic_units",
        "lexical_units",
        "search_profiles",
        "embedding_models",
        "embedding_inputs",
        "trait_definitions",
    ] {
        if c.prepare(&format!("SELECT 1 FROM {table} LIMIT 1"))?
            .exists([])?
        {
            return Err(IndexError::Invalid(format!(
                "historical fixture requires unpublished reference tables: {table} is populated"
            )));
        }
    }
    Ok(())
}

pub(crate) fn root_digest(
    source: &crate::SourceArtifactRecordInput,
    label: &str,
) -> Result<String, IndexError> {
    Ok(sha256(&serde_json::to_vec(&(
        encode_snapshot(source.record.source())?,
        &source.source_path,
        &source.content_hash,
        &source.preparation_context_hash,
        &source.content,
        &source.relationships,
        &source.diagnostics,
        label,
        "authored-query/v5",
    ))?))
}

pub(crate) fn records(
    c: &Connection,
    input: &IndexBuildInput,
) -> Result<BTreeMap<RecordKey, i64>, IndexError> {
    require_unpublished_reference(c)?;
    let mut context = input.context.clone();
    context.remaster_inputs_identity.clear();
    context.search_profiles = input.context.canonical_profiles()?;
    let json = serde_json::to_string(&context)?;
    c.execute("INSERT INTO artifact_context(singleton,format_version,context_hash,context_json) VALUES(1,?,?,?)",
        params![crate::ARTIFACT_FORMAT_VERSION, sha256(json.as_bytes()), json])?;
    for pack in &input.packs {
        let mut row = ProjectionRow::new("packs");
        row.text("pack_id", &pack.pack_id);
        row.text("label", &pack.label);
        row.insert(c)?;
    }
    let root_ids = input
        .records
        .iter()
        .map(|source| {
            let name =
                match atlas_record::source_record::SourceNodeView::from(source.record.source())
                    .name()
                {
                    atlas_record::source_record::SourceFieldView::Value(name) => {
                        Some(name.as_str())
                    }
                    _ => None,
                };
            Ok((
                source.record.key().clone(),
                crate::identity::retain_source(c, source.record.key(), name)?,
            ))
        })
        .collect::<Result<BTreeMap<_, _>, IndexError>>()?;
    let products = input
        .records
        .iter()
        .filter(|source| {
            !matches!(
                source.record.source(),
                atlas_foundry_model::FoundryDocumentSource::Macro(_)
            )
        })
        .map(|source| (source.record.key().clone(), root_ids[source.record.key()]))
        .collect::<BTreeMap<_, _>>();
    let labels = input
        .packs
        .iter()
        .map(|pack| (pack.pack_id.as_str(), pack.label.as_str()))
        .collect::<BTreeMap<_, _>>();
    let mut next = 0;
    for source in &input.records {
        let id = root_ids[source.record.key()];
        let label = labels
            .get(source.record.key().pack().as_str())
            .ok_or_else(|| IndexError::Invalid("missing pack label".into()))?;
        let seed = next;
        let mut ids = ProjectionIds::seeded(seed);
        let rows = root_rows(source, label, id, &mut ids)?;
        next = ids.end();
        c.execute(
            "INSERT INTO projection_allocations(record_id,seed,end_id,input_hash) VALUES(?,?,?,?)",
            params![id, seed, next, root_digest(source, label)?],
        )?;
        for row in rows {
            row.insert(c)?;
        }
        c.execute(
            "INSERT INTO record_bodies(record_id,codec_version,encoding,snapshot) VALUES(?,?,?,?)",
            params![
                id,
                atlas_foundry_model::SNAPSHOT_VERSION,
                "gzip6",
                gzip(&encode_snapshot(source.record.source())?)?
            ],
        )?;
    }
    for source in &input.records {
        let id = root_ids[source.record.key()];
        crate::writer::write_content(c, source, id, &root_ids, input.context.audience, true, true)?;
        for (ordinal, relationship) in source.relationships.iter().enumerate() {
            if relationship.locator.record != *source.record.key()
                || source
                    .record
                    .node_at(&relationship.locator.owners)
                    .is_none()
            {
                return Err(IndexError::Invalid(
                    "structured relationship locator disagrees with source".into(),
                ));
            }
            crate::writer::write_relationship(
                c,
                id,
                &SourceStoredRelationship {
                    locator: relationship.locator.clone(),
                    ordinal,
                    origin: "structured".into(),
                    kind: serde_json::from_str(&serde_json::to_string(&relationship.kind)?)?,
                    authored_target: relationship.authored_target.clone(),
                    occurrence_path: None,
                    resolution: relationship.resolution.clone(),
                    details: SourceRelationshipDetails {
                        availability: Some(relationship.availability.clone()),
                        reference_kind: None,
                        label: None,
                        audiences: Vec::new(),
                    },
                },
                &root_ids,
            )?;
        }
        for diagnostic in &source.diagnostics {
            crate::writer::write_diagnostic(c, id, diagnostic)?;
        }
        for diagnostic in crate::projections::projection_diagnostics(&source.record) {
            crate::writer::write_diagnostic(c, id, &diagnostic)?;
        }
    }
    for definition in crate::query::query_capabilities()?.fields {
        let mut row = ProjectionRow::new("query_field_catalog");
        row.text("field", &definition.id);
        row.text("definition_json", serde_json::to_string(&definition)?);
        row.insert(c)?;
    }
    Ok(products)
}

pub(crate) fn search_units(
    c: &Connection,
    input: &IndexBuildInput,
    ids: &BTreeMap<RecordKey, i64>,
) -> Result<(), IndexError> {
    lexical_units(c, &input.lexical_units, ids)?;
    for model in input.context.canonical_profiles()? {
        c.execute(
            "INSERT INTO search_profiles(profile_id,identity_json,dimensions) VALUES(?,?,?)",
            params![
                model.profile_id.as_str(),
                serde_json::to_string(&model)?,
                model.dimensions
            ],
        )?;
        c.execute_batch(&crate::schema::vector_ddl(&model)?)?;
    }
    let mut next = 0i64;
    for unit in &input.semantic_units {
        let model = input.context.search_profile(&unit.profile_id)?;
        crate::validation::check_vector(&unit.vector, model.dimensions)?;
        let id = crate::writer::product_id(ids, &unit.record)?;
        let table = crate::schema::vector_table(model)?;
        let location = crate::historical_locations::location_id(
            c,
            id,
            &serde_json::to_string(&unit.owners)?,
            unit.field.as_deref(),
            &serde_json::to_string(&unit.address)?,
            unit.chunk_ordinal,
        )?;
        let native = crate::embedding_values::publish_input(
            c,
            model,
            &unit.input_hash,
            unit.input_token_count,
            &unit.vector,
        )?;
        next = next
            .checked_add(1)
            .ok_or_else(|| IndexError::Invalid("semantic ID overflow".into()))?;
        let mut row = ProjectionRow::new("semantic_units");
        row.id("unit_id", next);
        row.id("record_id", id);
        row.text("profile_id", unit.profile_id.as_str());
        row.id("location_id", location);
        row.id("input_id", native);
        row.text(
            "unit_kind",
            if matches!(
                unit.address,
                atlas_domain::SourcePassageAddress::Identity {}
            ) {
                "identity"
            } else {
                "passage"
            },
        );
        row.insert(c)?;
        c.execute(
            &format!("INSERT INTO {table}(unit_id,embedding) VALUES(?,?)"),
            params![next, crate::read_units::vector_bytes(&unit.vector)],
        )?;
    }
    Ok(())
}

fn lexical_units(
    c: &Connection,
    units: &[crate::SourceLexicalUnitInput],
    ids: &BTreeMap<RecordKey, i64>,
) -> Result<(), IndexError> {
    let mut hashes = BTreeSet::new();
    for unit in units {
        let hash = sha256(&serde_json::to_vec(unit)?);
        if !hashes.insert(hash.clone()) {
            return Err(IndexError::Invalid("duplicate lexical unit".into()));
        }
        let mut row = ProjectionRow::new("lexical_units");
        row.text("input_hash", hash);
        row.id("record_id", crate::writer::product_id(ids, &unit.record)?);
        row.text("owners_json", serde_json::to_string(&unit.owners)?);
        row.push("field_path", nullable(unit.field.clone()));
        row.push(
            "section_json",
            nullable(
                unit.address
                    .as_ref()
                    .map(serde_json::to_string)
                    .transpose()?,
            ),
        );
        row.text("unit_kind", unit.kind.as_str());
        row.text("identity_terms", &unit.identity_terms);
        row.text("alias_terms", &unit.alias_terms);
        row.text("structured_terms", &unit.structured_terms);
        row.text("definition_terms", &unit.definition_terms);
        row.insert(c)?;
    }
    Ok(())
}
