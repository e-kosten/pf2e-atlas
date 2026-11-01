//! One streaming reference writer for prepared files and genuine fixture sources.
use crate::{
    IndexError, NativeValueSource, PreparedRecordRef, PublicationSource, SourceRelationshipDetails,
    SourceStoredRelationship, component_digests::ComponentDigests, persistence::ProjectionRow,
    projections::root_rows,
};
use atlas_record::source_record::SourceNodeView;
use rusqlite::{Connection, OptionalExtension, params};
use std::collections::{BTreeMap, BTreeSet};

/// Compare the actual retained record, stored reference resolutions and aliases
/// under its prior compatible policies. No current resolver supplies old evidence.
fn previous(
    c: &Connection,
    id: i64,
    context: &crate::SourceArtifactBuildContext,
) -> Result<Option<ComponentDigests>, IndexError> {
    if let Some(digests) = ComponentDigests::read(c, id)? {
        return Ok(Some(digests));
    }
    let mut digests = crate::component_digests::actual(c, context, id)?;
    let (key, label): (String, String) = c.query_row(
        "SELECT r.key,p.label FROM records r JOIN packs p USING(pack_id) WHERE record_id=?",
        [id],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;
    let key = key
        .parse()
        .map_err(|e| IndexError::Invalid(format!("prior component key: {e}")))?;
    let source = crate::persisted_inputs::read_attested_persisted_input(c, context, &key)?
        .ok_or_else(|| IndexError::Invalid("prior component source disappeared".into()))?;
    if context.query_projection_version == crate::SOURCE_PROJECTION_VERSION
        && context.query_catalog_version == crate::query::QUERY_CAPABILITY_VERSION
    {
        let seed: i64 = c.query_row(
            "SELECT seed FROM projection_allocations WHERE record_id=?",
            [id],
            |r| r.get(0),
        )?;
        for row in root_rows(
            &source,
            &label,
            id,
            &mut crate::projections::ProjectionIds::seeded(seed),
        )? {
            let sql = format!(
                "SELECT count(*) FROM {} WHERE {}",
                row.table,
                row.columns
                    .iter()
                    .zip(&row.values)
                    .map(|(column, value)| match value {
                        rusqlite::types::Value::Integer(_) =>
                            format!("{column} IS ? AND typeof({column})='integer'"),
                        rusqlite::types::Value::Real(_) =>
                            format!("{column} IS ? AND typeof({column})='real'"),
                        _ => format!("{column} IS ?"),
                    })
                    .collect::<Vec<_>>()
                    .join(" AND ")
            );
            if c.query_row(&sql, rusqlite::params_from_iter(&row.values), |r| {
                r.get::<_, usize>(0)
            })? != 1
            {
                return Err(IndexError::Invalid(
                    "prior typed projection differs from checked source".into(),
                ));
            }
        }
    } else {
        digests.query = "prior-incompatible-query-policy".into();
    }
    if context.selected_sections_recipe == crate::SELECTED_SECTIONS_RECIPE
        && context.lexical_selection_version == crate::LEXICAL_SELECTION_VERSION
        && context.core_sections_recipe == atlas_domain::CORE_SECTIONS_RECIPE
    {
        crate::validation_units::validate_selection(
            c,
            id,
            &source.record,
            context,
            &label,
            &source.content,
        )?;
    } else {
        digests.sections = "prior-incompatible-section-policy".into();
        digests.lexical = "prior-incompatible-lexical-policy".into();
        digests.core = "prior-incompatible-core-policy".into();
    }
    Ok(Some(digests))
}

pub(crate) fn write(
    c: &Connection,
    input: &dyn PublicationSource,
    values: &dyn NativeValueSource,
) -> Result<(), IndexError> {
    let started = std::time::Instant::now();
    let affected_before = c.total_changes();
    let mut query_root_replacements = 0u64;
    let mut snapshot_root_replacements = 0u64;
    let mut content_root_replacements = 0u64;
    let mut relationship_root_replacements = 0u64;
    let old_context = crate::read_stored_reference_context(c)?;
    let reusable_context = !matches!(
        crate::classify_reference_upgrade(c)?,
        crate::ReferenceUpgrade::SourcePreparation | crate::ReferenceUpgrade::SourceReplacement
    );
    let mut prior = BTreeMap::new();
    input.records(&mut |frame| {
        let key = frame.source.record.key();
        let id: Option<i64> = c
            .query_row(
                "SELECT record_id FROM records WHERE key=?",
                [key.to_string()],
                |r| r.get(0),
            )
            .optional()?;
        let value = match id {
            Some(id) if reusable_context => match &old_context {
                Some(context) => previous(c, id, context)?,
                None => None,
            },
            _ => None,
        };
        prior.insert(key.clone(), value);
        Ok(())
    })?;
    let mut canonical = input.context().clone();
    canonical.search_profiles = canonical.canonical_profiles()?;
    let json = serde_json::to_string(&canonical)?;
    c.execute("INSERT INTO artifact_context VALUES(1,?,?,?) ON CONFLICT(singleton) DO UPDATE SET format_version=excluded.format_version,context_hash=excluded.context_hash,context_json=excluded.context_json WHERE context_hash IS NOT excluded.context_hash",params![crate::ARTIFACT_FORMAT_VERSION,crate::codec::sha256(json.as_bytes()),json])?;
    for pack in input.packs() {
        let mut row = ProjectionRow::new("packs");
        row.text("pack_id", &pack.pack_id);
        row.text("label", &pack.label);
        crate::publication_delta::upsert(c, &row, "pack_id")?;
    }
    let labels = input
        .packs()
        .iter()
        .map(|p| (p.pack_id.as_str(), p.label.as_str()))
        .collect::<BTreeMap<_, _>>();
    let mut ids = BTreeMap::new();
    let mut products = BTreeMap::new();
    let mut live = BTreeSet::new();
    input.records(&mut |frame| {
        let source = frame.source;
        let name = SourceNodeView::from(source.record.source())
            .name()
            .value()
            .cloned();
        let id = crate::identity::retain_source(c, source.record.key(), name.as_deref())?;
        ids.insert(source.record.key().clone(), id);
        live.insert(id);
        if SourceNodeView::from(source.record.source()).document_kind() != "Macro" {
            products.insert(source.record.key().clone(), id);
        }
        Ok(())
    })?;
    let mut next_digests = crate::BuildFrameStream::<(i64, ComponentDigests)>::new()?;
    let mut sections = BTreeSet::new();
    input.records(&mut |frame|{
        let source=frame.source;let id=ids[source.record.key()];
        let label=labels[source.record.key().pack().as_str()];
        let old=prior.remove(source.record.key()).flatten();
        let desired=ComponentDigests::project(PreparedRecordRef{source,snapshot:frame.snapshot,selection:frame.selection,lexical:frame.lexical},label,input.context().audience)?;
        let query_changed=old.as_ref().is_none_or(|old|old.query!=desired.query) || old_context.as_ref().is_some_and(|old|old.query_projection_version!=input.context().query_projection_version || old.query_catalog_version!=input.context().query_catalog_version);
        let snapshot_changed=old.as_ref().is_none_or(|old|old.snapshot!=desired.snapshot);
        let content_changed=old.as_ref().is_none_or(|old|old.content!=desired.content);
        let relationships_changed=old.as_ref().is_none_or(|old|old.relationships!=desired.relationships);
        let digest=crate::publication_delta::root_digest_encoded(source,label,frame.snapshot)?;
        if query_changed {
            query_root_replacements+=1;
            crate::publication_delta::clear_query(c,id)?;
            let mut allocation=crate::publication_delta::allocate(c,source,label,id,&digest)?;
            for row in root_rows(source,label,id,&mut allocation)?{if row.table=="records"{crate::publication_delta::upsert(c,&row,"record_id")?;}else{row.insert(c)?;}}
        }else{
            c.execute("UPDATE records SET source_path=?,content_hash=? WHERE record_id=? AND (source_path IS NOT ?1 OR content_hash IS NOT ?2)",params![source.source_path,source.content_hash,id])?;
            c.execute("UPDATE projection_allocations SET input_hash=? WHERE record_id=? AND input_hash IS NOT ?1",params![digest,id])?;
        }
        if snapshot_changed {
            snapshot_root_replacements+=1;
            let bytes=crate::codec::gzip(frame.snapshot)?;
            c.execute("INSERT INTO record_bodies VALUES(?,?,?,?) ON CONFLICT(record_id) DO UPDATE SET codec_version=excluded.codec_version,encoding=excluded.encoding,snapshot=excluded.snapshot WHERE codec_version IS NOT excluded.codec_version OR encoding IS NOT excluded.encoding OR snapshot IS NOT excluded.snapshot",params![id,atlas_foundry_model::SNAPSHOT_VERSION,"gzip6",bytes])?;
            c.execute("DELETE FROM developer_diagnostics WHERE record_id=? AND stage<>'preparation'",[id])?;
            for diagnostic in &source.diagnostics{crate::writer::write_diagnostic(c,id,diagnostic)?;}
        }else if query_changed{c.execute("DELETE FROM developer_diagnostics WHERE record_id=? AND stage='projection'",[id])?;}
        if snapshot_changed || query_changed{for diagnostic in crate::projections::projection_diagnostics(&source.record){crate::writer::write_diagnostic(c,id,&diagnostic)?;}}
        if content_changed {content_root_replacements+=1;c.execute("DELETE FROM prepared_content WHERE record_id=?",[id])?;c.execute("DELETE FROM developer_diagnostics WHERE record_id=? AND stage='preparation'",[id])?;}
        if relationships_changed {relationship_root_replacements+=1;c.execute("DELETE FROM relationship_occurrences WHERE record_id=?",[id])?;}
        if content_changed || relationships_changed{crate::writer::write_content(c,source,id,&ids,input.context().audience,content_changed,relationships_changed)?;}
        if relationships_changed {for (ordinal,relationship) in source.relationships.iter().enumerate(){
            if relationship.locator.record!=*source.record.key() || source.record.node_at(&relationship.locator.owners).is_none(){return Err(IndexError::Invalid("structured relationship owner differs".into()));}
            crate::writer::write_relationship(c,id,&SourceStoredRelationship{locator:relationship.locator.clone(),ordinal,origin:"structured".into(),kind:serde_json::from_str(&serde_json::to_string(&relationship.kind)?)?,authored_target:relationship.authored_target.clone(),occurrence_path:None,resolution:relationship.resolution.clone(),details:SourceRelationshipDetails{availability:Some(relationship.availability.clone()),reference_kind:None,label:None,audiences:Vec::new()}},&ids)?;
        }}
        for section in crate::selected_sections::project(&source.record,frame.selection,&source.content_hash,&source.preparation_context_hash)?{sections.insert(crate::selected_sections::upsert(c,id,&section)?);}
        next_digests.push(&(id,desired))?;
        Ok(())
    })?;
    crate::publication_delta::aliases(c, input, &products)?;
    let definitions = crate::query::query_capabilities()?.fields;
    let existing = c
        .prepare("SELECT field FROM query_field_catalog")?
        .query_map([], |r| r.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    for field in existing {
        if !definitions.iter().any(|d| d.id == field) {
            c.execute("DELETE FROM query_field_catalog WHERE field=?", [field])?;
        }
    }
    for definition in definitions {
        let mut row = ProjectionRow::new("query_field_catalog");
        row.text("field", &definition.id);
        row.text("definition_json", serde_json::to_string(&definition)?);
        crate::publication_delta::upsert(c, &row, "field")?;
    }
    crate::publication_delta::search_units(c, input, &products, values)?;
    c.execute("DELETE FROM semantic_locations WHERE NOT EXISTS(SELECT 1 FROM semantic_units u WHERE u.location_id=semantic_locations.location_id)",[])?;
    let old_sections = c
        .prepare("SELECT section_id FROM selected_sections")?
        .query_map([], |r| r.get::<_, i64>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    for id in old_sections {
        if !sections.contains(&id) {
            c.execute("DELETE FROM selected_sections WHERE section_id=?", [id])?;
        }
    }
    crate::publication_delta::remove_absent_roots(c, &live)?;
    crate::traits::write_traits_stream(c, input, values)?;
    input.records(&mut |frame| {
        let id = ids[frame.source.record.key()];
        if products.contains_key(frame.source.record.key()) {
            let membership =
                crate::core_projection::membership(c, id, &frame.source.record, frame.selection)?;
            crate::core_projection::write_membership(c, id, &membership)?;
        }
        Ok(())
    })?;
    next_digests.visit(&mut |(id, digest)| digest.write(c, id))?;
    c.execute(
        "DELETE FROM packs WHERE pack_id NOT IN (SELECT pack_id FROM records)",
        [],
    )?;
    c.cache_flush()?;
    let usage = next_digests.file_usage()?;
    tracing::info!(target:"atlas_build_diagnostic",stage="reference_delta",elapsed_ms=started.elapsed().as_secs_f64()*1000.,sqlite_affected_rows=c.total_changes()-affected_before,query_root_replacements,snapshot_root_replacements,content_root_replacements,relationship_root_replacements,digest_spool_file_bytes=usage.file_bytes,digest_spool_allocated_bytes=usage.allocated_bytes.unwrap_or(0),allocated_bytes_known=usage.allocated_bytes.is_some(),"Observed reference reconciliation work");
    Ok(())
}
