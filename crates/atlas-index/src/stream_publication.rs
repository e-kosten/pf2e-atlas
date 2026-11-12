//! One streaming reference writer for prepared files and genuine fixture sources.
use crate::{
    IndexError, PreparedRecordRef, PublicationSpool, SourceRelationshipDetails,
    SourceStoredRelationship, component_digests::ComponentDigests, persistence::ProjectionRow,
};
use rusqlite::{Connection, OptionalExtension, params};
use std::collections::{BTreeMap, BTreeSet};

#[derive(serde::Serialize, serde::Deserialize)]
struct RootUpdate {
    id: i64,
    digests: Option<ComponentDigests>,
    core: Option<atlas_record::source_record::CoreSearchSelection>,
    refresh_core: bool,
}

pub(crate) fn write(
    c: &Connection,
    input: &PublicationSpool,
    values: &crate::prepared_publication::PublicationValues<'_, '_>,
    reusable_context: bool,
    scratch: &std::path::Path,
) -> Result<(), IndexError> {
    let started = std::time::Instant::now();
    let affected_before = c.total_changes();
    let mut query_root_replacements = 0u64;
    let mut snapshot_root_replacements = 0u64;
    let mut content_root_replacements = 0u64;
    let mut relationship_root_replacements = 0u64;
    let old_context = crate::read_stored_reference_context(c)?;
    let identities = input.identities();
    let mut prior = BTreeMap::new();
    for identity in identities {
        let key = &identity.key;
        let id: Option<i64> = c
            .query_row(
                "SELECT record_id FROM records WHERE key=?",
                [key.to_string()],
                |r| r.get(0),
            )
            .optional()?;
        let value = match id {
            Some(id) if reusable_context => ComponentDigests::read(c, id)?,
            _ => None,
        };
        prior.insert(key.clone(), value);
    }
    let mut canonical = input.context.clone();
    canonical.search_profiles = canonical.canonical_profiles()?;
    let json = serde_json::to_string(&canonical)?;
    c.execute(
        "INSERT INTO artifact_context(singleton,format_version,context_hash,context_json,query_catalog_sha256) VALUES(1,?,?,?,?)
         ON CONFLICT(singleton) DO UPDATE SET format_version=excluded.format_version,
             context_hash=excluded.context_hash,context_json=excluded.context_json,query_catalog_sha256=excluded.query_catalog_sha256
         WHERE context_hash IS NOT excluded.context_hash OR query_catalog_sha256 IS NOT excluded.query_catalog_sha256",
        params![crate::ARTIFACT_FORMAT_VERSION,crate::codec::sha256(json.as_bytes()),json,crate::catalog_fingerprint::current()?.as_slice()],
    )?;
    for pack in &input.packs {
        let mut row = ProjectionRow::new("packs");
        row.text("pack_id", &pack.pack_id);
        row.text("label", &pack.label);
        crate::publication_delta::upsert(c, &row, "pack_id")?;
    }
    let labels = input
        .packs
        .iter()
        .map(|p| (p.pack_id.as_str(), p.label.as_str()))
        .collect::<BTreeMap<_, _>>();
    let mut ids = BTreeMap::new();
    let mut products = BTreeMap::new();
    let mut live = BTreeSet::new();
    for identity in identities {
        let id = crate::identity::retain_source(c, &identity.key, identity.name.as_deref())?;
        ids.insert(identity.key.clone(), id);
        live.insert(id);
        if identity.is_product {
            products.insert(identity.key.clone(), id);
        }
    }
    crate::references::write_notices(c, input)?;
    let previous_traits = c
        .prepare("SELECT key FROM reference_entries WHERE key LIKE 'trait:%'")?
        .query_map([], |row| row.get::<_, String>(0))?
        .collect::<Result<BTreeSet<_>, _>>()?;
    let kept_traits = crate::traits::write_trait_definitions(c, input)?;
    let trait_vocabulary_changed = previous_traits != kept_traits;
    let mut next_roots = crate::BuildFrameStream::<RootUpdate>::new_in(scratch)?;
    let mut sections_to_prune = BTreeMap::<i64, BTreeSet<i64>>::new();
    input.records(&mut |frame|{
        let source=frame.source;
        let id=crate::writer::product_id(&ids,source.record.key())?;
        let label=labels.get(source.record.key().pack().as_str()).copied().ok_or_else(||IndexError::Invalid("record pack is absent".into()))?;
        let old=prior.remove(source.record.key()).flatten();
        let derived=crate::component_digests::DerivedComponents::project(PreparedRecordRef{source,snapshot:frame.snapshot,selection:frame.selection,lexical:frame.lexical},label,input.context.audience)?;
        let desired=derived.digests;
        let query_changed=old.as_ref().is_none_or(|old|old.query!=desired.query) || old_context.as_ref().is_some_and(|old|old.query_projection_version!=input.context.query_projection_version || old.query_catalog_version!=input.context.query_catalog_version);
        let snapshot_changed=old.as_ref().is_none_or(|old|old.snapshot!=desired.snapshot);
        let content_changed=old.as_ref().is_none_or(|old|old.content!=desired.content);
        let relationships_changed=old.as_ref().is_none_or(|old|old.relationships!=desired.relationships);
        let sections_changed=old.as_ref().is_none_or(|old|old.sections!=desired.sections)
            || old_context.as_ref().is_some_and(|old|old.selected_sections_recipe!=input.context.selected_sections_recipe);
        let traits_changed=trait_vocabulary_changed || old.as_ref().is_none_or(|old|old.trait_associations!=desired.trait_associations);
        let refresh_core=sections_changed || old.as_ref().is_none_or(|old|old.core!=desired.core)
            || old_context.as_ref().is_some_and(|old|old.core_sections_recipe!=input.context.core_sections_recipe);
        if query_changed {
            query_root_replacements+=1;
            crate::publication_delta::replace_query_rows(c,id,derived.allocation_footprint,derived.query_rows)?;
        }else{
            c.execute("UPDATE records SET source_path=?,content_hash=? WHERE record_id=? AND (source_path IS NOT ?1 OR content_hash IS NOT ?2)",params![source.source_path,source.content_hash,id])?;
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
        if content_changed || relationships_changed{crate::writer::write_content(c,source,id,&ids,input.context.audience,content_changed,relationships_changed)?;}
        if relationships_changed {for (ordinal,relationship) in source.relationships.iter().enumerate(){
            if relationship.locator.record!=*source.record.key() || source.record.node_at(&relationship.locator.owners).is_none(){return Err(IndexError::Invalid("structured relationship owner differs".into()));}
            crate::writer::write_relationship(c,id,&SourceStoredRelationship{locator:relationship.locator.clone(),ordinal,origin:"structured".into(),kind:serde_json::from_str(&serde_json::to_string(&relationship.kind)?)?,authored_target:relationship.authored_target.clone(),occurrence_path:None,resolution:relationship.resolution.clone(),details:SourceRelationshipDetails{availability:Some(relationship.availability.clone()),reference_kind:None,label:None,audiences:Vec::new()}},&ids)?;
        }}
        if sections_changed {
            let kept=sections_to_prune.entry(id).or_default();
            for section in derived.sections {kept.insert(crate::selected_sections::upsert(c,id,&section)?);}
        }
        if traits_changed {crate::traits::write_record_trait_relations(c,source.record.key(),derived.traits)?;}
        let changed=old.as_ref()!=Some(&desired);
        next_roots.push(&RootUpdate {
            id,
            digests: changed.then_some(desired),
            core: products.contains_key(source.record.key()).then_some(derived.core),
            refresh_core,
        })?;
        Ok(())
    })?;
    crate::publication_delta::aliases(c, input, &products)?;
    crate::verified_duplicates::publish(c, input, &products)?;
    crate::verified_duplicates::validate(c, input.context.verified_duplicate_sha256.as_deref())?;
    crate::remaster::publish(c, input, &products)?;
    let changed_search = crate::publication_delta::search_units(c, input, &products, values)?;
    c.execute("DELETE FROM semantic_locations WHERE NOT EXISTS(SELECT 1 FROM semantic_units u WHERE u.location_id=semantic_locations.location_id)",[])?;
    for (record, kept) in sections_to_prune {
        let old = c
            .prepare("SELECT section_id FROM selected_sections WHERE record_id=?")?
            .query_map([record], |row| row.get::<_, i64>(0))?
            .collect::<Result<Vec<_>, _>>()?;
        for section in old {
            if !kept.contains(&section) {
                c.execute(
                    "DELETE FROM selected_sections WHERE section_id=?",
                    [section],
                )?;
            }
        }
    }
    crate::publication_delta::remove_absent_roots(c, &live)?;
    let kept_entries = crate::references::write_entries(c, input)?;
    crate::references::validate_inventory(
        input.context.reference_preparation.as_ref(),
        kept_traits.iter().chain(&kept_entries).cloned(),
    )?;
    crate::owned_abilities::write_evidence(c, input)?;
    crate::traits::prune_trait_definitions(c, &kept_traits)?;
    next_roots.visit(&mut |root| {
        if let Some(core) = root.core
            && (root.refresh_core || changed_search.contains(&root.id))
        {
            let membership = crate::core_projection::membership_from_selection(c, root.id, &core)?;
            crate::core_projection::write_membership(c, root.id, &membership)?;
        }
        if let Some(digests) = root.digests {
            digests.write(c, root.id)?;
        }
        Ok(())
    })?;
    c.execute(
        "DELETE FROM packs WHERE pack_id NOT IN (SELECT pack_id FROM records)",
        [],
    )?;
    c.cache_flush()?;
    let usage = next_roots.file_usage()?;
    tracing::info!(target:"atlas_build_diagnostic",stage="reference_delta",elapsed_ms=started.elapsed().as_secs_f64()*1000.,sqlite_affected_rows=c.total_changes()-affected_before,query_root_replacements,snapshot_root_replacements,content_root_replacements,relationship_root_replacements,digest_spool_file_bytes=usage.file_bytes,digest_spool_allocated_bytes=usage.allocated_bytes.unwrap_or(0),allocated_bytes_known=usage.allocated_bytes.is_some(),"Observed reference reconciliation work");
    Ok(())
}
