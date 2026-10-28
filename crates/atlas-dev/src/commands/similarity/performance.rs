use super::{
    args::EvaluateArgs,
    execution::{Case, capture},
    model::{Manifest, Scenario},
    open_service,
};
use atlas_domain::SearchProfileId;
use atlas_search::{RecordScope, SimilarityEvaluationVariant};
use serde_json::{Value, json};

pub(super) fn declared(
    manifest: &Manifest,
    scenario: &Scenario,
    profile: &SearchProfileId,
    variant: &SimilarityEvaluationVariant,
) -> bool {
    let p = &manifest.performance;
    (p.scenario_ids.contains(&scenario.id)
        && p.profiles.contains(profile)
        && p.variant_ids.contains(&variant.id))
        || p.additional_cases.iter().any(|c| {
            c.scenario_id == scenario.id
                && c.variant_id == variant.id
                && c.profiles.contains(profile)
        })
}
pub(super) fn run(
    options: &EvaluateArgs,
    manifest: &Manifest,
    scenario: &Scenario,
    profile: &SearchProfileId,
    variants: &[SimilarityEvaluationVariant],
    process_cold: bool,
) -> Result<Value, String> {
    let mut cases = Vec::new();
    for variant in variants
        .iter()
        .filter(|v| declared(manifest, scenario, profile, v))
    {
        eprintln!(
            "{} / {} / {}: fresh context + warm performance requests",
            scenario.id, profile, variant.id
        );
        let service = open_service(options, profile, manifest)?;
        let cold = capture(
            &service,
            &Case {
                scenario,
                profile,
                generation: &manifest.artifact_generation,
            },
            variant,
            RecordScope::All,
            true,
            1,
        )?;
        let count = options
            .repetitions
            .unwrap_or(manifest.performance.warm_repetitions);
        let warm = if cold.status == "ok" {
            Some(capture(
                &service,
                &Case {
                    scenario,
                    profile,
                    generation: &manifest.artifact_generation,
                },
                variant,
                RecordScope::All,
                true,
                count,
            )?)
        } else {
            None
        };
        let samples = warm
            .as_ref()
            .filter(|w| w.status == "ok" && w.samples.len() == count as usize)
            .map(|w| {
                w.samples
                    .iter()
                    .filter_map(|s| s["elapsed_ms"].as_f64())
                    .collect::<Vec<_>>()
            });
        let stats=samples.as_ref().map(|v|json!({"samples":v.len(),"satisfies_manifest_repetition_count":v.len()==manifest.performance.warm_repetitions as usize,"p50_ms":percentile(v,0.5),"p95_ms":percentile(v,0.95),"convention":"nearest-rank, ceil(p*N)-1"}));
        cases.push(json!({"variant_id":variant.id,"options":variant.options,"cold_definition":"first complete request in a fresh application read context; uncontrolled OS page cache","satisfies_manifest_process_cold":process_cold,"process_cold_note":if process_cold {"single declared performance cell is the first complete product request in this process"}else{"context-cold only; use one scenario/profile/variant per separate performance invocation to satisfy the frozen fresh-process definition"},"warm_definition":"complete product pipeline repeated in the same service/context as the cold request","requested_warm_samples":count,"cold":cold,"warm":warm,"warm_statistics":stats,"process_id":std::process::id(),"peak_rss_bytes":null,"memory_measurement":"external process peak-RSS receipt required; runner does not infer peak from samples","cancellation_concurrency":"not measured by this single-request timing cell"}));
    }
    Ok(
        json!({"scope":"declared_full_production_performance_subset","cases":cases,"empty_reason":"non-declared scenarios/variants are intentionally absent"}),
    )
}
pub(super) fn percentile(samples: &[f64], p: f64) -> Option<f64> {
    if samples.is_empty() {
        return None;
    }
    let mut values = samples.to_vec();
    values.sort_by(f64::total_cmp);
    values
        .get(((p * values.len() as f64).ceil() as usize).saturating_sub(1))
        .copied()
}
