use super::model::Scenario;
use atlas_domain::{RecordKey, ResultSort, SearchProfileId, SimilarityOptions};
use atlas_search::{
    AtlasRetrievalService, ConfigurableSimilarRequest, RecordScope, SearchPage,
    SimilarityEvaluationRequest, SimilarityEvaluationResult, SimilarityEvaluationRun,
    SimilarityEvaluationVariant,
};
use serde::Serialize;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::time::Instant;

pub(super) struct Case<'a> {
    pub scenario: &'a Scenario,
    pub profile: &'a SearchProfileId,
    pub generation: &'a str,
}
#[derive(Clone, Serialize)]
pub(super) struct Capture {
    pub representative_variant: String,
    pub options: SimilarityOptions,
    pub status: String,
    pub error: Option<Value>,
    pub returned_top20: Vec<RecordKey>,
    pub returned_scores: Vec<Value>,
    pub admitted_keys: Option<Vec<RecordKey>>,
    pub admission_stage: &'static str,
    pub recipe: Option<String>,
    pub samples: Vec<Value>,
    pub evidence_locations: Vec<Value>,
}
impl Capture {
    fn new(variant: &SimilarityEvaluationVariant) -> Self {
        Self {
            representative_variant: variant.id.clone(),
            options: variant.options.clone(),
            status: "unavailable".into(),
            error: None,
            returned_top20: vec![],
            returned_scores: vec![],
            admitted_keys: None,
            admission_stage: "complete_pre_rerank_pre_remaster_root_union_when_available",
            recipe: None,
            samples: vec![],
            evidence_locations: vec![],
        }
    }
}
pub(super) struct Captures {
    pub groups: Vec<Capture>,
    pub group_for_variant: BTreeMap<String, usize>,
}
fn group(options: &SimilarityOptions) -> String {
    match options {
        SimilarityOptions::Passages {
            candidate_parts, ..
        } => format!("passages-{candidate_parts:?}"),
        SimilarityOptions::ContentAverage {
            candidate_parts, ..
        } => format!("average-{candidate_parts:?}"),
        SimilarityOptions::Baseline {} => "baseline".into(),
    }
}
pub(super) fn capture(
    service: &AtlasRetrievalService,
    case: &Case<'_>,
    variant: &SimilarityEvaluationVariant,
    scope: RecordScope<'_>,
    prefer_remaster: bool,
    repetitions: u32,
) -> Result<Capture, String> {
    let scenario = case.scenario;
    let profile = case.profile;
    let generation = case.generation;
    verify_generation(service, generation)?;
    let mut report = Capture::new(variant);
    for sample in 0..repetitions {
        service.reset_read_metrics();
        let started = Instant::now();
        let outcome = service.configurable_similar(ConfigurableSimilarRequest {
            source_sections: None,
            content_scope: atlas_domain::SearchContentScope::All,
            seed: &scenario.seed,
            options: &variant.options,
            search_profile: Some(profile),
            filter: scenario.candidate_filter.as_ref(),
            scope,
            page: SearchPage::first(20).map_err(|e| e.to_string())?,
            sort: ResultSort::Relevance {},
            prefer_remaster,
            cancelled: None,
        });
        let elapsed = started.elapsed().as_secs_f64() * 1000.0;
        let metrics = service.read_metrics();
        let mut measurement = json!({"sample":sample,"elapsed_ms":elapsed,"instrumented_read_statements":metrics.sql_statements,"source_body_decodes":metrics.source_body_decodes,"prepared_content_batches":metrics.prepared_content_batches,"summary_batches":metrics.summary_batches});
        match outcome {
            Ok(Some(result)) => {
                if result.results.artifact_generation != generation {
                    return Err("artifact generation changed inside captured request".into());
                }
                report.status = "ok".into();
                report.returned_top20 = result
                    .results
                    .records
                    .iter()
                    .map(|r| r.record.key.clone())
                    .collect();
                report.returned_scores = result.results.records.iter().map(|r| json!({"record_key":r.record.key,"score":r.score,"semantic_similarity":r.semantic_similarity})).collect();
                report.admitted_keys = result.candidate_keys;
                report.recipe = Some(result.recipe);
                report.evidence_locations = result.results.records.iter().take(10).map(|r| json!({"record_key":r.record.key,"locations":r.matches.iter().map(|w| json!({"location":w.location,"label":w.label,"similarity":w.semantic_similarity})).collect::<Vec<_>>() })).collect();
                measurement["diagnostics"] =
                    serde_json::to_value(result.diagnostics).map_err(|e| e.to_string())?;
                measurement["status"] = json!("ok");
            }
            Ok(None) => {
                report.status = "seed_unavailable".into();
                measurement["status"] = json!(report.status);
                report.error = Some(
                    json!({"kind":"seed_unavailable","message":"Seed not available in this artifact"}),
                );
            }
            Err(error) => {
                report.status = "failed".into();
                report.error =
                    Some(json!({"kind":format!("{:?}",error.kind()),"message":error.to_string()}));
                measurement["status"] = json!("failed");
                measurement["error"] = report.error.clone().unwrap_or(Value::Null);
            }
        }
        report.samples.push(measurement);
        // A classified structural failure is not a fast successful timing sample.
        if report.status != "ok" {
            break;
        }
    }
    verify_generation(service, generation)?;
    Ok(report)
}
pub(super) fn captures(
    service: &AtlasRetrievalService,
    case: &Case<'_>,
    variants: &[SimilarityEvaluationVariant],
    scope: RecordScope<'_>,
    prefer_remaster: bool,
    repetitions: u32,
) -> Result<Captures, String> {
    let scenario = case.scenario;
    let profile = case.profile;
    let mut groups = Vec::new();
    let mut group_ids = BTreeMap::new();
    let mut group_for_variant = BTreeMap::new();
    for variant in variants {
        let key = group(&variant.options);
        let index = if let Some(index) = group_ids.get(&key) {
            *index
        } else {
            eprintln!(
                "{} / {} / {}: complete {} capture",
                scenario.id,
                profile,
                variant.id,
                if prefer_remaster {
                    "production"
                } else {
                    "controlled"
                }
            );
            let report = capture(service, case, variant, scope, prefer_remaster, repetitions)?;
            let index = groups.len();
            groups.push(report);
            group_ids.insert(key, index);
            index
        };
        group_for_variant.insert(variant.id.clone(), index);
    }
    Ok(Captures {
        groups,
        group_for_variant,
    })
}
pub(super) fn verify_generation(
    service: &AtlasRetrievalService,
    expected: &str,
) -> Result<(), String> {
    if service.artifact_generation().map_err(|e| e.to_string())? != expected {
        return Err("frozen artifact generation does not match the active read snapshot".into());
    }
    Ok(())
}
fn exact(
    service: &AtlasRetrievalService,
    scenario: &Scenario,
    profile: &SearchProfileId,
    variants: &[SimilarityEvaluationVariant],
    keys: &[RecordKey],
    generation: &str,
) -> Result<SimilarityEvaluationResult, String> {
    verify_generation(service, generation)?;
    let result = service
        .evaluate_similarity_candidates(SimilarityEvaluationRequest {
            seed: &scenario.seed,
            profile,
            variants,
            candidate_keys: keys,
            filter: scenario.candidate_filter.as_ref(),
            cancelled: None,
        })
        .map_err(|e| e.to_string())?;
    if result.artifact_generation != generation {
        return Err("exact scoring returned another generation".into());
    }
    verify_generation(service, generation)?;
    Ok(result)
}
fn ranked(
    service: &AtlasRetrievalService,
    run: &SimilarityEvaluationRun,
    allowed: Option<&BTreeSet<RecordKey>>,
    prefer_remaster: bool,
) -> Result<Vec<RecordKey>, String> {
    let mut eligible = run.clone();
    if let Some(allowed) = allowed {
        eligible.rows.retain(|r| allowed.contains(&r.record_key));
    }
    service
        .evaluation_ranking(&eligible, prefer_remaster)
        .map_err(|e| e.to_string())
}
pub(super) fn production(
    service: &AtlasRetrievalService,
    scenario: &Scenario,
    profile: &SearchProfileId,
    variants: &[SimilarityEvaluationVariant],
    repetitions: u32,
    generation: &str,
) -> Result<Value, String> {
    let captures = captures(
        service,
        &Case {
            scenario,
            profile,
            generation,
        },
        variants,
        RecordScope::All,
        true,
        repetitions,
    )?;
    let mut union = BTreeSet::new();
    for capture in &captures.groups {
        if capture.status == "ok" {
            union.extend(
                capture
                    .admitted_keys
                    .as_ref()
                    .unwrap_or(&capture.returned_top20)
                    .iter()
                    .cloned(),
            );
        }
    }
    let eligible = variants
        .iter()
        .filter(|v| captures.groups[captures.group_for_variant[&v.id]].status == "ok")
        .cloned()
        .collect::<Vec<_>>();
    let keys = union.into_iter().collect::<Vec<_>>();
    let mut derived = Vec::new();

    let matrix = if !keys.is_empty() && !eligible.is_empty() {
        Some(exact(
            service, scenario, profile, &eligible, &keys, generation,
        )?)
    } else {
        None
    };
    for variant in variants {
        let capture = &captures.groups[captures.group_for_variant[&variant.id]];
        let top = if capture.status != "ok" {
            vec![]
        } else if capture.representative_variant == variant.id {
            capture.returned_top20.clone()
        } else {
            let allowed = capture
                .admitted_keys
                .as_ref()
                .ok_or("reused decay capture lacks complete admitted keys")?
                .iter()
                .cloned()
                .collect::<BTreeSet<_>>();
            let run = matrix
                .as_ref()
                .and_then(|m| m.runs.iter().find(|r| r.id == variant.id))
                .ok_or("reused decay lacks exact score matrix")?;
            ranked(service, run, Some(&allowed), true)?
                .into_iter()
                .take(20)
                .collect()
        };
        derived.push(json!({"id":variant.id,"options":variant.options,"capture_group":captures.group_for_variant[&variant.id],"status":capture.status,"returned_top20":top,"ranking_source":if capture.representative_variant==variant.id {"actual_complete_product_request"} else {"exact_same_admitted_candidate_set_decay_reaggregation"},"fresh_product_latency_ms":if capture.representative_variant==variant.id {capture.samples.first().and_then(|v|v.get("elapsed_ms")).cloned()} else {None},"error":capture.error}));
    }
    let preparation=matrix.as_ref().map(|m|json!({"requested_candidates":m.requested_candidates,"eligible_candidates":m.eligible_candidates,"candidate_vectors":m.candidate_vectors,"baseline_vectors":m.baseline_vectors,"pairwise_comparisons":m.pairwise_comparisons,"seed_preparation_ms":m.seed_preparation_ms,"vector_read_ms":m.vector_read_ms,"baseline_read_and_scoring_ms":m.baseline_read_and_scoring_ms,"shared_maxima_ms":m.shared_maxima_ms,"total_ms":m.total_ms,"scope":"shared exact preparation over complete bounded candidate union; not full product latency"}));
    Ok(
        json!({"scope":"production_full_corpus","prefer_remaster":true,"capture_groups":captures.groups,"variants":derived,"candidate_score_preparation":preparation}),
    )
}
pub(super) fn oracle(
    service: &AtlasRetrievalService,
    scenario: &Scenario,
    profile: &SearchProfileId,
    variants: &[SimilarityEvaluationVariant],
    keys: &[RecordKey],
    generation: &str,
) -> Result<Value, String> {
    let captures = captures(
        service,
        &Case {
            scenario,
            profile,
            generation,
        },
        variants,
        RecordScope::Keys(keys),
        false,
        1,
    )?;
    let matrix = exact(service, scenario, profile, variants, keys, generation)?;
    let mut comparisons = Vec::new();
    for run in &matrix.runs {
        let ordered = ranked(service, run, None, false)?;
        let oracle_top = ordered.into_iter().take(10).collect::<Vec<_>>();
        let capture = &captures.groups[captures.group_for_variant[&run.id]];
        let admitted = capture
            .admitted_keys
            .as_ref()
            .map(|keys| keys.iter().cloned().collect::<BTreeSet<_>>());
        let actual_top = if capture.status != "ok" {
            vec![]
        } else if capture.representative_variant == run.id {
            capture.returned_top20.iter().take(10).cloned().collect()
        } else if let Some(allowed) = admitted.as_ref() {
            ranked(service, run, Some(allowed), false)?
                .into_iter()
                .take(10)
                .collect()
        } else {
            vec![]
        };
        let comparison = recall(
            &oracle_top,
            admitted.as_ref(),
            &actual_top,
            capture.status == "ok",
        );
        let boundary = oracle_top
            .last()
            .and_then(|k| run.rows.iter().find(|r| &r.record_key == k))
            .and_then(|r| r.score);
        let missed = oracle_top
            .iter()
            .filter(|k| admitted.as_ref().is_some_and(|set| !set.contains(*k)))
            .map(|k| {
                let score = run
                    .rows
                    .iter()
                    .find(|r| &r.record_key == k)
                    .and_then(|r| r.score);
                json!({"record_key":k,"score":score,"boundary_tie":score==boundary})
            })
            .collect::<Vec<_>>();
        comparisons.push(json!({"id":run.id,"oracle_top10":oracle_top,"bounded_returned_top10":actual_top,"capture_group":captures.group_for_variant[&run.id],"bounded_status":capture.status,"metrics":comparison,"oracle_boundary_score":boundary,"admission_misses":missed}));
    }
    Ok(
        json!({"scope":"controlled_explicit_corpus","prefer_remaster":false,"capture_groups":captures.groups,"exact":matrix,"comparisons":comparisons}),
    )
}
pub(super) fn recall(
    oracle: &[RecordKey],
    admitted: Option<&BTreeSet<RecordKey>>,
    returned: &[RecordKey],
    success: bool,
) -> Value {
    let denominator = oracle.len();
    let valid = denominator > 0 && success;
    let admission = if valid {
        admitted
            .map(|a| oracle.iter().filter(|k| a.contains(*k)).count() as f64 / denominator as f64)
    } else {
        None
    };
    let overlap = if valid {
        Some(oracle.iter().filter(|k| returned.contains(k)).count() as f64 / denominator as f64)
    } else {
        None
    };
    json!({"oracle_top_count":denominator,"candidate_admission_recall_at10":admission,"returned_top10_agreement":overlap,"admission_available":admitted.is_some(),"valid":valid})
}
pub(super) fn fixed_pool(
    service: &AtlasRetrievalService,
    case: &Case<'_>,
    variants: &[SimilarityEvaluationVariant],
    keys: &[RecordKey],
    profile_coverage: &[String],
    complete_profiles: bool,
) -> Result<Value, String> {
    let scenario = case.scenario;
    let profile = case.profile;
    let generation = case.generation;
    if keys.is_empty() {
        return Ok(json!({"status":"unavailable","reason":"no successful production captures"}));
    }
    let matrix = exact(service, scenario, profile, variants, keys, generation)?;
    let mut rankings = Vec::new();
    let mut blind_pool = BTreeSet::new();
    for run in &matrix.runs {
        let ranking = ranked(service, run, None, true)?;
        blind_pool.extend(ranking.iter().take(10).cloned());
        rankings.push(json!({"id":run.id,"ranking":ranking}));
    }
    Ok(
        json!({"scope":"same_scenario_cross_profile_fixed_pool","pool_source":"union of successful variant/profile bounded top20","keys":keys,"pool_profile_coverage":profile_coverage,"complete_manifest_profile_coverage":complete_profiles,"prefer_remaster":true,"exact":matrix,"rankings":rankings,"blind_judging_pool_keys":blind_pool}),
    )
}
