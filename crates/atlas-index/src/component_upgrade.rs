//! Rebuild query accelerators from retained DTOs without re-ingest or inference.
use crate::{
    IndexError, SourceArtifactBuildContext,
    codec::{checked_hash, sha256},
    persistence::ProjectionRow,
};
use atlas_storage::Workspace;
use rusqlite::{Connection, OptionalExtension};

/// Read authenticated component metadata without requiring the current query
/// catalog. Updaters use this before selecting their supported rebuild path.
pub fn read_stored_reference_context(
    c: &Connection,
) -> Result<Option<SourceArtifactBuildContext>, IndexError> {
    let stored: Option<(u32, String, String)> = c
        .query_row(
            "SELECT format_version,context_hash,context_json FROM artifact_context WHERE singleton=1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .optional()?;
    let Some((format, hash, json)) = stored else {
        return Ok(None);
    };
    if !checked_hash(&hash) || sha256(json.as_bytes()) != hash {
        return Err(IndexError::Invalid(
            "stored context checksum mismatch".into(),
        ));
    }
    let original: SourceArtifactBuildContext = serde_json::from_str(&json)?;
    if format != crate::ARTIFACT_FORMAT_VERSION || original.format_version != format {
        return Err(IndexError::Unsupported(
            "stored reference format is unsupported".into(),
        ));
    }
    Ok(Some(original))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReferenceUpgrade {
    Current,
    QueryProjections,
    SourcePreparation,
}

/// Known policy namespaces are versioned independently. Older positive versions
/// can be regenerated from compatible source; unknown/newer contracts cannot.
fn older_policy(stored: &str, current: &str) -> Result<bool, IndexError> {
    if stored == current {
        return Ok(false);
    }
    let parse = |value: &str| -> Option<(String, u32)> {
        let (namespace, version) = value.rsplit_once("/v")?;
        Some((namespace.to_owned(), version.parse().ok()?))
    };
    match (parse(stored), parse(current)) {
        (Some((namespace, version)), Some((wanted, latest)))
            if namespace == wanted && version > 0 && version < latest =>
        {
            Ok(true)
        }
        _ => Err(IndexError::Unsupported(format!(
            "unsupported stored policy {stored}; expected {current}"
        ))),
    }
}

pub fn classify_reference_upgrade(c: &Connection) -> Result<ReferenceUpgrade, IndexError> {
    Ok(upgrade_plan(c)?.0)
}

fn upgrade_plan(
    c: &Connection,
) -> Result<(ReferenceUpgrade, Option<SourceArtifactBuildContext>), IndexError> {
    let Some(original) = read_stored_reference_context(c)? else {
        return Ok((ReferenceUpgrade::Current, None));
    };
    if original.snapshot_version != atlas_foundry_model::SNAPSHOT_VERSION
        || original.source_contract != atlas_foundry_model::SOURCE_CONTRACT_ID
    {
        return Err(IndexError::Unsupported(
            "stored source snapshot contract is unsupported".into(),
        ));
    }
    let current_catalog = crate::query::QUERY_CAPABILITY_VERSION;
    if original.query_catalog_version == 0 || original.query_catalog_version > current_catalog {
        return Err(IndexError::Unsupported(
            "filter catalog version is not supported by this executable".into(),
        ));
    }
    let query_changed = older_policy(
        &original.query_projection_version,
        crate::input::SOURCE_PROJECTION_VERSION,
    )? || original.query_catalog_version != current_catalog;
    let mut target = original.clone();
    target.query_catalog_version = current_catalog;
    target.query_projection_version = crate::input::SOURCE_PROJECTION_VERSION.into();
    let mut source_changed = false;
    for (stored, current) in [
        (
            &mut target.lookup_version,
            crate::input::SOURCE_LOOKUP_VERSION,
        ),
        (
            &mut target.lexical_selection_version,
            crate::input::LEXICAL_SELECTION_VERSION,
        ),
        (
            &mut target.content_interpretation_version,
            atlas_record::source_content::CONTENT_INTERPRETATION_VERSION,
        ),
        (
            &mut target.content_selection_version,
            atlas_record::source_record::SOURCE_CONTENT_SELECTION_VERSION,
        ),
        (
            &mut target.localization_policy,
            crate::input::LOCALIZATION_POLICY_VERSION,
        ),
        (&mut target.asset_policy, crate::input::ASSET_POLICY_VERSION),
        (
            &mut target.relationship_policy,
            crate::input::RELATIONSHIP_POLICY_VERSION,
        ),
    ] {
        source_changed |= older_policy(stored, current)?;
        *stored = current.into();
    }
    // The target must still satisfy every non-upgradable source/model/FTS
    // contract. No current-reader checks are bypassed for product consumers.
    crate::writer::validate_context(&target)?;
    if source_changed {
        return Ok((ReferenceUpgrade::SourcePreparation, None));
    }
    if query_changed {
        return Ok((ReferenceUpgrade::QueryProjections, Some(target)));
    }
    crate::validation::validate_open(c)?;
    Ok((ReferenceUpgrade::Current, None))
}

fn upgrade_context(c: &Connection) -> Result<Option<SourceArtifactBuildContext>, IndexError> {
    Ok(upgrade_plan(c)?.1)
}

/// Called by runtime before admitting product sessions. A newer executable may
/// regenerate filter rows from supported stored DTOs, never from user state.
pub fn ensure_query_projections(workspace: &Workspace, schema: u32) -> Result<bool, IndexError> {
    // Fast path permits concurrent ordinary operations when no work is needed.
    match workspace.read(schema) {
        Ok(read) => {
            if upgrade_context(read.connection())?.is_none() {
                return Ok(false);
            }
        }
        Err(atlas_storage::StorageError::Maintenance(_)) => {}
        Err(error) => return Err(error.into()),
    }
    let owner = workspace.update_owner()?;
    let maintenance = owner.maintenance(schema, "updating query projections")?;
    let target = upgrade_context(maintenance.connection())?;
    let Some(target) = target else {
        drop(maintenance);
        crate::recover_reference_publication(&owner, schema)?;
        return Ok(false);
    };
    maintenance.publish(|c| {
        let records = crate::read_persisted_inputs(c, &target)?;
        for source in records {
            let (id, label): (i64, String) = c.query_row(
                "SELECT r.record_id,p.label FROM records r JOIN packs p USING(pack_id) WHERE r.key=?", [source.record.key().to_string()],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )?;
            for table in ["record_traits", "actor_projection"] {
                c.execute(&format!("DELETE FROM {table} WHERE record_id=?"), [id])?;
            }
            for table in ["spell_projection", "physical_projection", "ability_projection", "heritage_projection", "effect_projection", "condition_projection", "deity_projection"] {
                c.execute(&format!("DELETE FROM {table} WHERE root_record_id=?"), [id])?;
            }
            let digest = crate::publication_delta::root_digest(&source, &label)?;
            let mut ids = crate::publication_delta::allocate(c, &source, &label, id, &digest)?;
            for row in crate::projections::root_rows(&source, &label, id, &mut ids)? {
                if row.table == "records" { crate::publication_delta::upsert(c, &row, "record_id")?; }
                else { row.insert(c)?; }
            }
            c.execute("DELETE FROM developer_diagnostics WHERE record_id=? AND stage='projection'", [id])?;
            for diagnostic in crate::projections::projection_diagnostics(&source.record) {
                crate::writer::write_diagnostic(c, id, &diagnostic)?;
            }
        }
        c.execute("DELETE FROM query_field_catalog", [])?;
        for definition in crate::query::query_capabilities()?.fields {
            let mut row = ProjectionRow::new("query_field_catalog");
            row.text("field", &definition.id);
            row.text("definition_json", serde_json::to_string(&definition)?);
            row.insert(c)?;
        }
        let json = serde_json::to_string(&target)?;
        c.execute("UPDATE artifact_context SET context_hash=?,context_json=? WHERE singleton=1", rusqlite::params![sha256(json.as_bytes()), json])?;
        crate::validate_reference(c)?;
        Ok::<_, IndexError>(true)
    })
}
