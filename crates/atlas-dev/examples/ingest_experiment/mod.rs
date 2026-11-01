//! Controlled observations through existing owners, with no lifecycle SQL.
mod args;
mod observer;
use args::{Args, Stage};
use atlas_runtime::{
    AtlasPathMode, AtlasPathOverrides, AtlasRuntime, AtlasRuntimeOptions, WORKSPACE_SCHEMA_VERSION,
};
use clap::Parser;
use serde::Serialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
    path::Path,
    sync::Arc,
    time::{Duration, Instant},
};

type PublishedIdentity = atlas_index::ReferencePublicationExpectation;
fn publication(runtime: &AtlasRuntime) -> Result<PublishedIdentity, String> {
    let reader = runtime.open_index().map_err(|e| e.to_string())?;
    Ok(PublishedIdentity::from_reader(&reader))
}
fn file_hash(path: &Path) -> Result<String, String> {
    let mut file = File::open(path).map_err(|e| e.to_string())?;
    let mut buffer = [0; 65536];
    let mut hash = Sha256::new();
    loop {
        let read = file.read(&mut buffer).map_err(|e| e.to_string())?;
        if read == 0 {
            break;
        }
        hash.update(&buffer[..read]);
    }
    Ok(format!("{:x}", hash.finalize()))
}
fn write_json(path: &Path, value: &impl Serialize) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| e.to_string())?;
    serde_json::to_writer_pretty(&mut file, value).map_err(|e| e.to_string())?;
    writeln!(file).map_err(|e| e.to_string())?;
    file.sync_all().map_err(|e| e.to_string())?;
    #[cfg(unix)]
    if let Some(parent) = path.parent() {
        File::open(parent)
            .and_then(|f| f.sync_all())
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}
fn runtime(args: &Args) -> Result<AtlasRuntime, String> {
    AtlasRuntime::resolve(AtlasRuntimeOptions {
        default_search_profile: None,
        path_mode: AtlasPathMode::Global,
        overrides: AtlasPathOverrides {
            source_root: Some(args.source.clone()),
            embedding_cache_root: Some(args.models.clone()),
            index_path: Some(args.database.clone()),
        },
    })
    .map_err(|e| e.to_string())
}
fn app_service(args: &Args) -> Result<atlas_app_service::AtlasAppService, String> {
    atlas_app_service::AtlasAppService::start(atlas_app_service::AtlasAppServiceOptions {
        default_search_profile: None,
        path_mode: AtlasPathMode::Global,
        source_root: Some(args.source.clone()),
        embedding_cache_root: Some(args.models.clone()),
        index_path: Some(args.database.clone()),
        retrieval_mode: atlas_app_service::AppServiceRetrievalMode::OnDemandNoQueryInference,
    })
    .map_err(|e| e.to_string())
}
fn requested(args: &Args) -> Result<atlas_index::RequestedReferenceConfig, String> {
    Ok(atlas_index::RequestedReferenceConfig {
        source_root: args.source.clone(),
        manifest_path: None,
        indexing_locale: args.locale.clone(),
        search_profiles: args.profile_ids()?,
    })
}
fn build(args: &Args) -> Result<Value, String> {
    // Current schema/readiness was checked before entering the measured scope.
    // Persist the complete intended set through its existing owner, then release
    // the short configuration owner before the ordinary build acquires it.
    let workspace = atlas_storage::Workspace::at(&args.database).map_err(|e| e.to_string())?;
    let config = requested(args)?;
    {
        let _owner = workspace.update_owner().map_err(|e| e.to_string())?;
        let write = workspace
            .write(WORKSPACE_SCHEMA_VERSION)
            .map_err(|e| e.to_string())?;
        atlas_index::write_requested_reference_config(&write, &config)
            .map_err(|e| e.to_string())?;
        write.commit().map_err(|e| e.to_string())?;
    }
    let search_profiles = config
        .search_profiles
        .iter()
        .map(|id| {
            atlas_embedding::search_profile(id)
                .map(|spec| atlas_embedding::EmbeddingRuntimeConfig::new(spec.model, &args.models))
                .map_err(|e| e.to_string())
        })
        .collect::<Result<Vec<_>, _>>()?;
    let report = atlas_ingest::build_artifact(
        &workspace,
        WORKSPACE_SCHEMA_VERSION,
        atlas_ingest::BuildArtifactOptions {
            source_root: config.source_root,
            output_path: args.database.clone(),
            manifest_path: config.manifest_path,
            locale: config.indexing_locale,
            search_profiles,
            reuse_embeddings: true,
            reuse_passage_layouts: true,
            embedding_batch_size: 32,
        },
    )
    .map_err(|e| e.to_string())?;
    Ok(
        json!({"admission":report.admission,"profiles":report.profiles,"record_count":report.record_count,"semantic_unit_count":report.semantic_unit_count,"inferred_inputs":report.inferred_inputs,"reused_inputs":report.reused_inputs,"source_fingerprint":report.source_fingerprint,"build_duration_ms":report.build_duration_ms,"source_preparation":report.source_report}),
    )
}
fn roster(args: &Args) -> Result<Value, String> {
    let service = app_service(args)?;
    let encounter = args.encounter.as_deref().ok_or("roster encounter absent")?;
    let mut samples = Vec::new();
    for sample in 0..args.repetitions {
        let started = Instant::now();
        let view = service.encounter(encounter).map_err(|e| e.to_string())?;
        let elapsed = started.elapsed();
        let bytes = serde_json::to_vec(&view).map_err(|e| e.to_string())?;
        samples.push(json!({"sample":sample,"elapsed_ns":elapsed.as_nanos(),"response_sha256":format!("{:x}",Sha256::digest(&bytes))}));
        if sample == 0 {
            write_json(&args.output.join("roster.json"), &view)?;
        }
    }
    Ok(
        json!({"encounter":encounter,"samples":samples,"timing_scope":"AppService::encounter only; service startup and response serialization excluded; first sample context-cold, following samples reuse one on-demand service; uncontrolled OS caches"}),
    )
}
// Each AppService call opens and releases its own on-demand session. The two
// request intervals are independent; serializing evidence holds no read session.
fn observation_sample(
    service: &atlas_app_service::AtlasAppService,
    encounter: Option<&str>,
    started: &Instant,
    first_success: Option<&Path>,
) -> Result<(Value, bool), String> {
    let begin = started.elapsed();
    let request = Instant::now();
    let readiness = service.readiness();
    let elapsed = request.elapsed();
    let end = started.elapsed();
    let readiness = json!({
        "begin_ns":begin.as_nanos(),"end_ns":end.as_nanos(),
        "request_ns":elapsed.as_nanos(),"view":readiness,
    });
    let Some(encounter) = encounter else {
        return Ok((json!({"readiness":readiness}), false));
    };
    let begin = started.elapsed();
    let request = Instant::now();
    let result = service.encounter(encounter);
    let elapsed = request.elapsed();
    let end = started.elapsed();
    let success = result.is_ok();
    let outcome = match result {
        Ok(view) => {
            let bytes = serde_json::to_vec(&view).map_err(|e| e.to_string())?;
            // These fingerprints come from this composed DTO's existing owned
            // navigation links. Missing fingerprints remain missing evidence.
            let participants = view.participants.iter().filter_map(|participant| {
                let record = participant.record.as_ref()?;
                let fingerprints = participant.presentation.iter()
                    .flat_map(|presentation| &presentation.owned)
                    .filter_map(|link| link.navigation.source_fingerprint.as_deref())
                    .collect::<std::collections::BTreeSet<_>>();
                Some(json!({"participant_key":participant.participant_key,
                    "participant_revision":participant.revision,
                    "saved_record_key":participant.record_key,
                    "record_key":record.record_key,
                    "presentation_record_key":participant.presentation.as_ref().map(|p| &p.identity.record_key),
                    "source_fingerprints":fingerprints}))
            }).collect::<Vec<_>>();
            if let Some(path) = first_success {
                write_json(path, &view)?;
            }
            json!({"kind":"ok","artifact_generation":view.artifact_generation,
                "encounter_key":view.encounter.encounter_key,"encounter_revision":view.revision,
                "participant_count":view.participants.len(),"source_backed_participants":participants,
                "response_sha256":format!("{:x}",Sha256::digest(&bytes)),
                "first_response_evidence":first_success})
        }
        Err(error) => json!({"kind":"error","error":error.into_app_error()}),
    };
    Ok((
        json!({"readiness":readiness,"encounter":{
            "reference":encounter,"begin_ns":begin.as_nanos(),"end_ns":end.as_nanos(),
            "request_ns":elapsed.as_nanos(),"outcome":outcome,
        }}),
        success,
    ))
}
fn observe(args: &Args) -> Result<Value, String> {
    let service = app_service(args)?;
    let stop = args
        .stop_file
        .as_ref()
        .ok_or("observation stop file absent")?;
    let initial = service.readiness();
    write_json(&args.output.join("ready.json"), &initial)?;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(args.output.join("availability.jsonl"))
        .map_err(|e| e.to_string())?;
    let started = Instant::now();
    let mut observed = 0;
    let mut encounter_successes = 0;
    let first_response = args.output.join("encounter-first-success.json");
    for sample in 0..args.max_observations {
        if stop.exists() {
            break;
        }
        let (mut observation, encounter_succeeded) = observation_sample(
            &service,
            args.encounter.as_deref(),
            &started,
            (encounter_successes == 0).then_some(first_response.as_path()),
        )?;
        if encounter_succeeded {
            encounter_successes += 1;
        }
        observation["sample"] = json!(sample);
        serde_json::to_writer(&mut file, &observation).map_err(|e| e.to_string())?;
        writeln!(file)
            .and_then(|()| file.flush())
            .map_err(|e| e.to_string())?;
        observed += 1;
        std::thread::sleep(Duration::from_millis(args.interval_ms));
    }
    file.sync_all().map_err(|e| e.to_string())?;
    if !stop.exists() {
        return Err("observation ceiling reached before the explicit stop signal".into());
    }
    if !stop.is_file() || stop.is_symlink() || observed == 0 {
        return Err(
            "observation needs at least one request and a regular caller-created stop file".into(),
        );
    }
    Ok(
        json!({"observations":observed,"encounter":args.encounter,"encounter_successes":encounter_successes,
            "first_response_evidence":(encounter_successes > 0).then_some(first_response),
            "elapsed_ns":started.elapsed().as_nanos(),"interval_ms":args.interval_ms,
            "scope":"separate AppService readiness and optional composed encounter requests with independent intervals; not a shared snapshot or atomic pair; startup, serialization and evidence output excluded from each request; sessions released before sleep; sampled availability can perturb locks and never joins shipping CLI timing"}),
    )
}
pub(super) fn run() -> Result<(), String> {
    let args = Args::parse();
    args.check()?;
    let executable = std::env::current_exe().map_err(|e| e.to_string())?;
    let executable_sha256 = file_hash(&executable)?;
    let runtime = runtime(&args)?;
    let before = if matches!(args.stage, Stage::Upgrade | Stage::Recover) {
        None
    } else {
        Some(publication(&runtime)?)
    };
    let recovery = if args.stage == Stage::Recover {
        let path = args
            .interruption_marker
            .as_ref()
            .ok_or("interruption marker absent")?;
        let marker: observer::InterruptionMarker =
            serde_json::from_reader(File::open(path).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
        if marker.version != 1
            || marker.database != args.database
            || marker.declared_candidate != args.candidate
            || marker.executable_sha256 != executable_sha256
            || marker.phase != "validate_build_input"
        {
            return Err(
                "interruption marker does not bind this database/executable/candidate/phase".into(),
            );
        }
        Some(marker)
    } else {
        None
    };
    std::fs::create_dir(&args.output).map_err(|e| e.to_string())?;
    write_json(
        &args.output.join("input.json"),
        &json!({"database":args.database,"source":args.source,"models":args.models,"stage":format!("{:?}",args.stage),"profiles":args.profiles,"declared_candidate":args.candidate,"actual_executable":executable,"executable_sha256":executable_sha256,"before":before,"schema_target":WORKSPACE_SCHEMA_VERSION,"instrumentation":"developer-only scoped workspace SQL trace events; other threads/independent temporary SQLite connections excluded; instrumented wall time is not shipping CLI timing"}),
    )?;
    let interruption = if args.stage == Stage::Interrupt {
        let publication = before
            .clone()
            .ok_or("interruption needs an admitted current publication")?;
        Some((
            args.output.join("interruption.json"),
            observer::InterruptionMarker {
                version: 1,
                database: args.database.clone(),
                declared_candidate: args.candidate.clone(),
                executable_sha256: executable_sha256.clone(),
                publication,
                phase: "validate_build_input".into(),
            },
        ))
    } else {
        None
    };
    let events = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(args.output.join("events.jsonl"))
        .map_err(|e| e.to_string())?;
    let observer = Arc::new(observer::Observer::new(events, interruption));
    let started = Instant::now();
    let (result,sql)=atlas_storage::diagnostics::capture_sql_work(|| tracing::subscriber::with_default(Arc::clone(&observer),|| -> Result<Value,String> {
        match args.stage {
            Stage::Upgrade => { let changed=runtime.upgrade_workspace_if_present().map_err(|e|e.to_string())?;Ok(json!({"metadata_projection_update":changed,"operation":"runtime schema plus compatible metadata upgrade; no setup/source acquisition/inference"})) }
            Stage::Build|Stage::Interrupt => build(&args),
            Stage::Recover => {
                let workspace=atlas_storage::Workspace::at(&args.database).map_err(|e|e.to_string())?;
                let owner=workspace.update_owner().map_err(|e|e.to_string())?;
                let expected=recovery.as_ref().ok_or("recovery marker absent")?;
                atlas_index::recover_reference_publication_checked(&owner,WORKSPACE_SCHEMA_VERSION,&expected.publication).map_err(|e|e.to_string())?;
                let actual=publication(&runtime)?;
                if actual!=expected.publication { return Err("recovery did not preserve the marker's exact prior publication".into()); }
                drop(owner);
                Ok(json!({"recovered":true,"preserved_publication":actual}))
            }
            Stage::Roster => roster(&args),
            Stage::Observe => observe(&args),
        }
    })).map_err(|e|e.to_string())?;
    let elapsed = started.elapsed();
    let after = if result.is_ok() {
        Some(publication(&runtime)?)
    } else {
        None
    };
    let success = result.is_ok() && !observer.failed();
    write_json(
        &args.output.join("result.json"),
        &json!({"status":if success {"ok"} else {"error"},"elapsed_ns":elapsed.as_nanos(),"result":result.as_ref().ok(),"error":result.as_ref().err(),"capture_failed":observer.failed(),"after":after,"sql":{"workspace_connections":sql.workspace_connections,"statement_start_events":sql.statement_start_events,"subprogram_start_events":sql.subprogram_start_events,"statement_completion_events":sql.statement_completion_events,"scope":"new workspace connection attachments plus calling-thread execution in the explicit scope; retained attached connections can contribute in later scopes; includes rolled-back execution; triggers reported separately; no SQL retained; no cumulative VM aggregation"}}),
    )?;
    result?;
    if observer.failed() {
        return Err("event capture failed; retain evidence".into());
    }
    if args.stage == Stage::Interrupt {
        return Err("interruption event was not reached; no interruption success claimed".into());
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn real_lexical_build_reports_stage_files_and_only_changed_root_component_actions() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().canonicalize().unwrap();
        let source = root.join("source");
        std::fs::create_dir_all(source.join("packs/items")).unwrap();
        std::fs::create_dir_all(source.join("static/lang")).unwrap();
        std::fs::write(
            source.join("module.json"),
            r#"{"packs":[{"name":"items","label":"Items","type":"Item","path":"packs/items"}]}"#,
        )
        .unwrap();
        std::fs::write(source.join("static/lang/en.json"), "{}").unwrap();
        let document = source.join("packs/items/effect.json");
        let original = r#"{"_id":"aaaaaaaaaaaaaaaa","name":"Useful record","type":"effect","system":{"description":{"value":"<p>Original body.</p>"}}}"#;
        std::fs::write(&document, original).unwrap();
        let database = root.join("workspace.sqlite");
        atlas_runtime::initialize_workspace(&database).unwrap();
        let output = root.join("evidence");
        std::fs::create_dir(&output).unwrap();
        let models = root.join("models");
        std::fs::create_dir(&models).unwrap();
        let args = Args {
            task_root: root,
            database,
            source,
            models,
            output,
            candidate: "tiny lexical metric fixture".into(),
            stage: Stage::Build,
            profiles: Some("none".into()),
            locale: "en".into(),
            interruption_marker: None,
            encounter: None,
            repetitions: 1,
            stop_file: None,
            interval_ms: 25,
            max_observations: 1,
        };
        build(&args).unwrap();
        for changed in [false, true] {
            if changed {
                std::fs::write(&document, original.replace("Original body", "Changed body"))
                    .unwrap();
            }
            let path = args.output.join(if changed {
                "changed.jsonl"
            } else {
                "unchanged.jsonl"
            });
            let observer = Arc::new(observer::Observer::new(
                OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(&path)
                    .unwrap(),
                None,
            ));
            let report =
                tracing::subscriber::with_default(Arc::clone(&observer), || build(&args)).unwrap();
            assert_eq!(report["inferred_inputs"], 0);
            assert!(!observer.failed());
            let events = std::fs::read_to_string(path)
                .unwrap()
                .lines()
                .map(|line| serde_json::from_str::<Value>(line).unwrap())
                .collect::<Vec<_>>();
            for stage in [
                "source_preparation",
                "native_target_registration_and_pruning",
                "native_value_freeze",
                "final_source_freshness",
                "exclusive_publication",
                "physical_reclamation",
                "publication_exact_validation",
            ] {
                assert!(
                    events.iter().any(|event| event["fields"]["stage"] == stage),
                    "missing stage {stage}"
                );
            }
            let delta = &events
                .iter()
                .find(|event| event["fields"]["stage"] == "reference_delta")
                .unwrap()["fields"];
            assert_eq!(delta["query_root_replacements"], 0);
            assert_eq!(delta["relationship_root_replacements"], 0);
            assert_eq!(delta["snapshot_root_replacements"], u64::from(changed));
            assert_eq!(delta["content_root_replacements"], u64::from(changed));
            let files = &events
                .iter()
                .find(|event| event["fields"]["stage"] == "sealed_metadata_and_frozen_values")
                .unwrap()["fields"];
            assert!(files["file_bytes"].as_u64().unwrap() > 0);
            #[cfg(unix)]
            assert_eq!(files["allocated_bytes_known"], true);
        }
    }
    #[test]
    fn encounter_observations_hydrate_source_and_report_maintenance_then_recovery() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().canonicalize().unwrap();
        let source = root.join("source");
        std::fs::create_dir_all(source.join("packs/actors")).unwrap();
        std::fs::create_dir_all(source.join("static/lang")).unwrap();
        std::fs::write(source.join("module.json"),
            r#"{"packs":[{"name":"actors","label":"Actors","type":"Actor","path":"packs/actors"}]}"#).unwrap();
        std::fs::write(source.join("static/lang/en.json"), "{}").unwrap();
        std::fs::write(source.join("packs/actors/goblin.json"),
            r#"{"_id":"aaaaaaaaaaaaaaaa","type":"npc","name":"Fixture Goblin","system":{"traits":{"value":[]},"details":{"level":{"value":1},"publicNotes":"<p>Fixture creature.</p>"},"attributes":{"adjustment":null,"ac":{"value":16},"hp":{"max":20},"speed":{"value":25,"otherSpeeds":[]}},"saves":{"fortitude":{"value":5},"reflex":{"value":7},"will":{"value":3}},"perception":{"mod":5}},"items":[{"_id":"bbbbbbbbbbbbbbbb","type":"melee","name":"Claw","system":{"range":null,"traits":{"value":[]},"bonus":{"value":7},"damageRolls":{"main":{"damage":"1d6+2","damageType":"slashing"}},"description":{"value":"<p>Fixture claw.</p>"}}}]}"#).unwrap();
        let output = root.join("evidence");
        std::fs::create_dir(&output).unwrap();
        let models = root.join("models");
        std::fs::create_dir(&models).unwrap();
        let args = Args {
            task_root: root.clone(),
            database: root.join("workspace.sqlite"),
            source,
            models,
            output,
            candidate: "composed observer fixture".into(),
            stage: Stage::Observe,
            profiles: Some("none".into()),
            locale: "en".into(),
            interruption_marker: None,
            encounter: None,
            repetitions: 1,
            stop_file: None,
            interval_ms: 25,
            max_observations: 1,
        };
        atlas_runtime::initialize_workspace(&args.database).unwrap();
        build(&args).unwrap();
        let service = app_service(&args).unwrap();
        let encounter = service
            .create_encounter(
                serde_json::from_value(json!({
                    "name":"Observed encounter","description":null,"note":null,
                }))
                .unwrap(),
            )
            .unwrap()
            .encounter
            .encounter_key;
        service
            .add_encounter_record_participant(
                serde_json::from_value(json!({
                    "encounter_ref":encounter,"record_ref":"actors:aaaaaaaaaaaaaaaa",
                    "quantity":1,"participant_variant":null,"initiative":null,
                }))
                .unwrap(),
            )
            .unwrap();
        let started = Instant::now();
        let first = args.output.join("encounter-first-success.json");
        let (sample, success) =
            observation_sample(&service, Some(&encounter), &started, Some(&first)).unwrap();
        assert!(success);
        assert_eq!(sample["readiness"]["view"]["status"], "ready");
        let result = &sample["encounter"]["outcome"];
        assert_eq!(result["kind"], "ok");
        assert!(result["artifact_generation"].is_string());
        assert_eq!(result["encounter_key"], encounter);
        let participant = &result["source_backed_participants"][0];
        assert!(participant["participant_key"].is_string());
        assert_eq!(participant["record_key"], "actors:aaaaaaaaaaaaaaaa");
        assert_eq!(participant["saved_record_key"], participant["record_key"]);
        assert_eq!(
            participant["presentation_record_key"],
            participant["record_key"]
        );
        // Composed roster presentation does not expose these optional owned
        // links. Preserve missing fingerprint evidence instead of inventing it.
        assert!(
            participant["source_fingerprints"]
                .as_array()
                .unwrap()
                .is_empty()
        );
        let view = service.encounter(&encounter).unwrap();
        let hydrated = serde_json::to_value(&view).unwrap();
        assert_eq!(
            hydrated["participants"][0]["record"]["title"],
            "Fixture Goblin"
        );
        let body = &hydrated["participants"][0]["presentation"]["body"];
        assert_eq!(body["kind"], "creature");
        assert_eq!(body["value"]["level"]["value"], 1);
        assert_eq!(body["value"]["armor_class"]["value"], 16);
        assert_eq!(body["value"]["maximum_hp"]["value"], 20);
        assert_eq!(
            result["response_sha256"],
            format!("{:x}", Sha256::digest(serde_json::to_vec(&view).unwrap()))
        );
        assert_eq!(
            serde_json::from_reader::<_, Value>(File::open(&first).unwrap()).unwrap(),
            serde_json::to_value(&view).unwrap()
        );
        let (readiness_only, success) = observation_sample(&service, None, &started, None).unwrap();
        assert!(!success);
        assert!(readiness_only.get("encounter").is_none());
        let (missing, success) =
            observation_sample(&service, Some("missing-encounter"), &started, None).unwrap();
        assert!(!success);
        assert_eq!(
            missing["encounter"]["outcome"]["error"]["code"],
            "encounter_not_found"
        );

        // Acquiring maintenance proves no on-demand reader survived the sample.
        // Drop the exclusive admission while retaining its durable marker: reads
        // now report maintenance through the existing typed SetupRequired DTO.
        let workspace = atlas_storage::Workspace::at(&args.database).unwrap();
        let owner = workspace.update_owner().unwrap();
        drop(
            owner
                .maintenance(WORKSPACE_SCHEMA_VERSION, "observer fixture maintenance")
                .unwrap(),
        );
        let (blocked, success) =
            observation_sample(&service, Some(&encounter), &started, None).unwrap();
        assert!(!success);
        assert_eq!(blocked["readiness"]["view"]["status"], "blocked");
        assert_eq!(blocked["encounter"]["outcome"]["kind"], "error");
        assert_eq!(
            blocked["encounter"]["outcome"]["error"]["code"],
            "setup_required"
        );
        atlas_index::recover_reference_publication(&owner, WORKSPACE_SCHEMA_VERSION).unwrap();
        let (recovered, success) =
            observation_sample(&service, Some(&encounter), &started, None).unwrap();
        assert!(success);
        assert_eq!(
            recovered["encounter"]["outcome"]["response_sha256"],
            result["response_sha256"]
        );
        assert_eq!(
            recovered["encounter"]["outcome"]["artifact_generation"],
            result["artifact_generation"]
        );
        assert_eq!(std::fs::read_dir(&args.output).unwrap().count(), 1);
    }
    #[test]
    fn bound_markers_are_fresh_and_keep_exact_generation_fields() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("marker.json");
        let marker = observer::InterruptionMarker {
            version: 1,
            database: Path::new("/task/db").into(),
            declared_candidate: "candidate".into(),
            executable_sha256: "a".repeat(64),
            publication: serde_json::from_value(json!({"instance":{"workspace_key":"workspace","reference_revision":9},"context_sha256":"b".repeat(64)})).unwrap(),
            phase: "validate_build_input".into(),
        };
        write_json(&path, &marker).unwrap();
        assert!(write_json(&path, &marker).is_err());
        let read: observer::InterruptionMarker =
            serde_json::from_reader(File::open(path).unwrap()).unwrap();
        assert_eq!(read.publication, marker.publication);
        assert_eq!(read.database, marker.database);
        assert_eq!(read.executable_sha256, marker.executable_sha256);
    }
}
