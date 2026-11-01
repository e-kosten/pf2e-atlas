//! Opt-in fixture-feature census/projection pilot over an explicitly supplied copy.
use crate::{
    IndexError,
    core_projection::{membership, write_membership},
};
use atlas_record::source_record::select_record_search;
use rusqlite::Connection;
use serde_json::{Value, json};
use std::{collections::BTreeMap, path::Path, time::Instant};

#[derive(Default, serde::Serialize)]
struct FamilyCensus {
    records: usize,
    identity_only: usize,
    identity_and_prose: usize,
    core_owners: usize,
    core_fields: usize,
    core_sections: usize,
    core_lexical_units: usize,
    core_semantic_locations: usize,
    selected_fields: usize,
    fields_by_visibility: BTreeMap<String, usize>,
    core_fields_by_visibility: BTreeMap<String, usize>,
    declared_prose_fields: usize,
    declared_fields_by_visibility: BTreeMap<String, usize>,
    selected_plain_fields: usize,
    core_plain_fields: usize,
    unavailable_visibility: usize,
}

fn file_bytes(path: &Path) -> u64 {
    std::fs::metadata(path)
        .map(|metadata| metadata.len())
        .unwrap_or(0)
}
fn temporary_bytes() -> u64 {
    std::fs::read_dir(std::env::temp_dir())
        .map(|entries| {
            entries
                .filter_map(Result::ok)
                .filter_map(|entry| entry.metadata().ok())
                .filter(|metadata| metadata.is_file())
                .map(|metadata| metadata.len())
                .sum()
        })
        .unwrap_or(0)
}

pub(crate) fn run(path: &Path) -> Result<Value, IndexError> {
    atlas_sqlite_vec::register_sqlite_vec_auto_extension()
        .map_err(|error| IndexError::Operation(error.to_string()))?;
    let started = Instant::now();
    let c = Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE)?;
    c.execute_batch("PRAGMA foreign_keys=ON; PRAGMA cache_size=-8192; BEGIN IMMEDIATE;")?;
    let report = (|| -> Result<Value, IndexError> {
        let context = crate::read_stored_reference_context(&c)?.ok_or_else(|| {
            IndexError::Invalid("pilot requires retained reference metadata".into())
        })?;
        if context.snapshot_version != atlas_foundry_model::SNAPSHOT_VERSION
            || context.source_contract != atlas_foundry_model::SOURCE_CONTRACT_ID
            || context.content_selection_version
                != atlas_record::source_record::SOURCE_CONTENT_SELECTION_VERSION
        {
            return Err(IndexError::Unsupported(
                "pilot requires compatible checked source and section contracts".into(),
            ));
        }
        let version: u32 = c.query_row("PRAGMA user_version", [], |row| row.get(0))?;
        let revision: u64 = c.query_row(
            "SELECT reference_revision FROM workspace WHERE id=1",
            [],
            |row| row.get(0),
        )?;
        let before_pages: u64 = c.query_row("PRAGMA page_count", [], |row| row.get(0))?;
        let before_free_pages: u64 = c.query_row("PRAGMA freelist_count", [], |row| row.get(0))?;
        let page_size: u64 = c.query_row("PRAGMA page_size", [], |row| row.get(0))?;
        let temp_store: u64 = c.query_row("PRAGMA temp_store", [], |row| row.get(0))?;
        let wal = path.with_file_name(format!(
            "{}-wal",
            path.file_name()
                .and_then(|name| name.to_str())
                .ok_or_else(|| IndexError::Invalid("invalid pilot path".into()))?
        ));
        let before_wal = file_bytes(&wal);
        let before_temp = temporary_bytes();
        if context.selected_sections_recipe != crate::SELECTED_SECTIONS_RECIPE {
            return Err(IndexError::Unsupported(
                "pilot requires the current canonical section projection".into(),
            ));
        }
        c.execute_batch("DELETE FROM core_parts; DELETE FROM core_lexical_units; DELETE FROM core_semantic_locations; DELETE FROM core_roots;")?;
        let mut families = BTreeMap::<String, FamilyCensus>::new();
        for family in [
            "npc",
            "hazard",
            "spell",
            "rule",
            "equipment",
            "character_option",
            "deity",
            "campaign_feature",
            "vehicle",
            "army",
            "character",
            "familiar",
            "unmodelled",
            "trait",
        ] {
            families.insert(family.into(), FamilyCensus::default());
        }
        let mut after = 0_i64;
        let mut records = 0_usize;
        let mut read_ms = 0_f64;
        let mut selection_and_membership_ms = 0_f64;
        let mut write_ms = 0_f64;
        let mut peak_wal = before_wal;
        let mut peak_temp = before_temp;
        let mut max_selected_fields = 0_usize;
        loop {
            let roots=c.prepare("SELECT r.record_id,r.key,p.label FROM records r JOIN packs p USING(pack_id) WHERE r.record_id>? AND r.document_kind<>'Macro' ORDER BY r.record_id LIMIT 128")?.query_map([after],|row|Ok((row.get::<_,i64>(0)?,row.get::<_,String>(1)?,row.get::<_,String>(2)?)))?.collect::<Result<Vec<_>,_>>()?;
            if roots.is_empty() {
                break;
            }
            for (id, key, label) in roots {
                after = id;
                let key = key
                    .parse()
                    .map_err(|error| IndexError::Invalid(format!("invalid pilot key:{error}")))?;
                let clock = Instant::now();
                let source = crate::persisted_inputs::read_persisted_input(&c, &context, &key)?
                    .ok_or_else(|| IndexError::Invalid("pilot lost a checked record".into()))?;
                read_ms += clock.elapsed().as_secs_f64() * 1000.;
                let clock = Instant::now();
                let selection = select_record_search(
                    &source.record,
                    &source.content,
                    context.audience,
                    &label,
                    &context.used_trait_labels,
                )
                .map_err(|error| IndexError::Invalid(error.to_string()))?;
                let core =
                    atlas_record::source_record::core_search_selection(&source.record, &selection)
                        .map_err(|error| IndexError::Invalid(error.to_string()))?;
                let projected = membership(&c, id, &source.record, &selection)?;
                selection_and_membership_ms += clock.elapsed().as_secs_f64() * 1000.;
                let census = families.entry(projected.family.clone()).or_default();
                census.records += 1;
                census.identity_only += usize::from(projected.parts.is_empty());
                census.identity_and_prose += usize::from(!projected.parts.is_empty());
                census.core_owners += core.owners.len();
                census.core_fields += core.fields.len();
                census.core_sections += projected.parts.len();
                census.core_lexical_units += projected.lexical_units.len();
                census.core_semantic_locations += projected.semantic_locations.len();
                census.selected_fields += selection.fields.len();
                max_selected_fields = max_selected_fields.max(selection.fields.len());
                let mut declared = Vec::new();
                source.record.visit_content_selections(|owners, field| {
                    if field.role == atlas_record::source_record::SourceContentRole::Name {
                        return;
                    }
                    let visibility = field
                        .visibility
                        .value()
                        .map(|value| format!("{value:?}"))
                        .unwrap_or_else(|| format!("{:?}", field.visibility.availability()));
                    let known_visibility = matches!(
                        field.visibility.availability(),
                        atlas_record::source_record::FieldAvailability::Value
                    );
                    let plain = matches!(
                        field.format.value(),
                        Some(atlas_record::source_record::SourceContentFormat::Plain)
                    );
                    declared.push((
                        atlas_record::source_content::SourceContentLocator {
                            record: source.record.key().clone(),
                            owners: owners.to_vec(),
                            field: field.field.into_owned(),
                        },
                        visibility,
                        known_visibility,
                        plain,
                    ));
                });
                for (locator, visibility, known_visibility, plain) in declared {
                    census.declared_prose_fields += 1;
                    *census
                        .declared_fields_by_visibility
                        .entry(visibility.clone())
                        .or_default() += 1;
                    if selection
                        .fields
                        .iter()
                        .any(|field| field.locator == locator)
                    {
                        *census
                            .fields_by_visibility
                            .entry(visibility.clone())
                            .or_default() += 1;
                        census.selected_plain_fields += usize::from(plain);
                    }
                    if core.fields.contains(&locator) {
                        *census
                            .core_fields_by_visibility
                            .entry(visibility)
                            .or_default() += 1;
                        census.core_plain_fields += usize::from(plain);
                    }
                    if !known_visibility {
                        census.unavailable_visibility += 1;
                    }
                }
                let clock = Instant::now();
                write_membership(&c, id, &projected)?;
                write_ms += clock.elapsed().as_secs_f64() * 1000.;
                records += 1;
            }
            peak_wal = peak_wal.max(file_bytes(&wal));
            peak_temp = peak_temp.max(temporary_bytes());
            if records.is_multiple_of(1024) {
                eprintln!(
                    "Core policy pilot: {records} records; {:.1}s",
                    started.elapsed().as_secs_f64()
                );
            }
        }
        let after_pages: u64 = c.query_row("PRAGMA page_count", [], |row| row.get(0))?;
        let after_free_pages: u64 = c.query_row("PRAGMA freelist_count", [], |row| row.get(0))?;
        let allocation = match c.prepare("SELECT d.name,sum(d.pgsize) FROM dbstat d JOIN sqlite_schema s ON s.name=d.name WHERE s.tbl_name IN ('core_roots','core_parts','core_lexical_units','core_semantic_locations') GROUP BY d.name ORDER BY d.name") {
            Ok(mut statement) => {
                let rows=statement.query_map([],|row|Ok((row.get::<_,String>(0)?,row.get::<_,u64>(1)?)))?.collect::<Result<BTreeMap<_,_>,_>>()?;
                json!({"per_relation_bytes":rows,"total_bytes":rows.values().sum::<u64>(),"includes_implicit_indexes":true})
            },
            Err(error) => json!({"unavailable":error.to_string()}),
        };
        let mut rows = BTreeMap::<String, u64>::new();
        for table in [
            "core_roots",
            "core_parts",
            "core_lexical_units",
            "core_semantic_locations",
        ] {
            rows.insert(
                table.into(),
                c.query_row(&format!("SELECT count(*) FROM {table}"), [], |row| {
                    row.get(0)
                })?,
            );
        }
        let mut profiles = BTreeMap::<String, Value>::new();
        for profile in &context.search_profiles {
            let identities:u64=c.query_row("SELECT count(*) FROM semantic_units su JOIN records r USING(record_id) WHERE su.profile_id=? AND su.unit_kind='identity' AND r.document_kind<>'Macro'",[profile.profile_id.as_str()],|row|row.get(0))?;
            let core_units:u64=c.query_row("SELECT count(*) FROM semantic_units su JOIN core_semantic_locations cl USING(location_id) WHERE su.profile_id=?",[profile.profile_id.as_str()],|row|row.get(0))?;
            let missing_core_parts:u64=c.query_row("SELECT count(*) FROM core_parts cp JOIN selected_sections sp USING(section_id) WHERE NOT EXISTS(SELECT 1 FROM semantic_units su JOIN semantic_locations sl USING(location_id) WHERE sl.section_id=sp.section_id AND su.profile_id=?)",[profile.profile_id.as_str()],|row|row.get(0))?;
            let traits: u64 = c.query_row(
                "SELECT count(*) FROM trait_vectors WHERE profile_id=?",
                [profile.profile_id.as_str()],
                |row| row.get(0),
            )?;
            profiles.insert(profile.profile_id.to_string(),json!({"root_identity_vectors":identities,"roots_without_identity":records.saturating_sub(identities as usize),"core_semantic_units":core_units,"core_sections_without_vectors":missing_core_parts,"trait_vectors":traits,"dimensions":profile.dimensions,"validity":"native values were validated by the retained publication; this pilot counts presence and does not rerun complete numerical validation"}));
        }
        let (trait_definitions,trait_prose):(u64,u64)=c.query_row("SELECT count(*),coalesce(sum(json_type(definition_json,'$.prepared')='object'),0) FROM trait_definitions",[],|row|Ok((row.get(0)?,row.get(1)?)))?;
        let trait_census = json!({"definitions":trait_definitions,"definition_prose":trait_prose,"label_only":trait_definitions-trait_prose,"separate_record_identity_components":0,"family_record_count_is_zero_because":"Traits are a separate reference entity, not records; definitions and profile presence are counted here."});
        Ok(
            json!({"date":"2026-10-27","kind":"rollback_only_metadata_projection_pilot","database":path,"schema_version":version,"reference_revision":revision,"source_contract":context.source_contract,"preparation_recipe":context.content_interpretation_version,"core_recipe":atlas_domain::CORE_SECTIONS_RECIPE,"artifact_audience":context.audience,"source_checkout_access":false,"model_access":false,"inference":false,"records":records,"families":families,"traits":trait_census,"profiles":profiles,"rows":rows,"read_checked_ms":read_ms,"selection_policy_membership_ms":selection_and_membership_ms,"write_projection_ms":write_ms,"duration_ms":started.elapsed().as_secs_f64()*1000.,"before_pages":before_pages,"after_pages_in_transaction":after_pages,"before_free_pages":before_free_pages,"after_free_pages_in_transaction":after_free_pages,"core_allocation":allocation,"page_size":page_size,"logical_database_growth_bytes":after_pages.saturating_sub(before_pages)*page_size,"database_file_bytes":file_bytes(path),"wal_before_bytes":before_wal,"wal_peak_observed_bytes":peak_wal,"temp_store":temp_store,"cache_size_kib":8192,"temporary_directory":std::env::temp_dir(),"temp_before_bytes":before_temp,"temp_peak_observed_bytes":peak_temp,"maximum_selected_fields_per_root":max_selected_fields,"sampling":"WAL and scoped temporary directory sampled after each batch of at most 128 roots; peaks are observed, not continuous OS maxima.","rolled_back":true,"quality_evaluation":false}),
        )
    })();
    c.execute_batch("ROLLBACK;")?;
    report
}
