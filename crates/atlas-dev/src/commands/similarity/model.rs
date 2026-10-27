use super::args::EvaluateArgs;
use atlas_domain::{
    QueryPredicate, RecordKey, SearchProfileId, SimilarityOptions, SimilarityPartSelector,
    SimilaritySeed,
};
use atlas_search::SimilarityEvaluationVariant;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

#[derive(Deserialize)]
pub(super) struct Manifest {
    pub version: u32,
    pub experiment_id: String,
    pub artifact_generation: String,
    pub source_revision: String,
    pub profiles: Vec<SearchProfileId>,
    pub repetitions: u32,
    pub variants: Vec<Variant>,
    pub scenarios: Vec<Scenario>,
    pub performance: Performance,
}
#[derive(Clone, Deserialize)]
pub(super) struct Variant {
    pub id: String,
    pub options: SimilarityOptions,
}
#[derive(Deserialize)]
pub(super) struct Scenario {
    pub id: String,
    pub split: String,
    pub stratum: String,
    pub seed: SimilaritySeed,
    pub selection: Selection,
    pub candidate_filter: Option<QueryPredicate>,
    pub quality_eligible: bool,
}
#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(super) enum Selection {
    All,
    Explicit {
        selectors: Vec<SimilarityPartSelector>,
        artifact_generation: String,
    },
}
#[derive(Deserialize)]
pub(super) struct Performance {
    pub scenario_ids: Vec<String>,
    pub profiles: Vec<SearchProfileId>,
    pub variant_ids: Vec<String>,
    pub warm_repetitions: u32,
    pub additional_cases: Vec<PerformanceCase>,
}
#[derive(Deserialize)]
pub(super) struct PerformanceCase {
    pub scenario_id: String,
    pub variant_id: String,
    pub profiles: Vec<SearchProfileId>,
}
#[derive(Deserialize)]
pub(super) struct Corpus {
    pub version: u32,
    pub artifact_generation: String,
    pub source_revision: String,
    pub records: Vec<CorpusRecord>,
}
#[derive(Deserialize)]
pub(super) struct CorpusRecord {
    pub record_key: RecordKey,
}
#[derive(Deserialize)]
struct Freeze {
    state: String,
    experiment_id: String,
    files: BTreeMap<String, String>,
}
pub(super) struct Inputs {
    pub manifest: Manifest,
    pub corpus: Corpus,
    pub manifest_sha256: String,
    pub corpus_sha256: String,
    pub freeze_sha256: String,
}
pub(super) fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn bytes(path: &Path) -> Result<Vec<u8>, String> {
    std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))
}
pub(super) fn load(options: &EvaluateArgs) -> Result<Inputs, String> {
    let manifest_bytes = bytes(&options.manifest)?;
    let corpus_bytes = bytes(&options.corpus)?;
    let manifest: Manifest =
        serde_json::from_slice(&manifest_bytes).map_err(|e| format!("manifest: {e}"))?;
    let corpus: Corpus =
        serde_json::from_slice(&corpus_bytes).map_err(|e| format!("corpus: {e}"))?;
    let freeze_path = options.freeze.clone().unwrap_or_else(|| {
        options
            .manifest
            .parent()
            .unwrap_or(Path::new("."))
            .join("freeze.json")
    });
    let freeze_bytes = bytes(&freeze_path)?;
    let freeze: Freeze =
        serde_json::from_slice(&freeze_bytes).map_err(|e| format!("freeze: {e}"))?;
    if freeze.state != "frozen-before-ranked-results"
        || freeze.experiment_id != manifest.experiment_id
    {
        return Err("input freeze identity/state mismatch".into());
    }
    for (name, expected) in &freeze.files {
        if Path::new(name).file_name().and_then(|x| x.to_str()) != Some(name) {
            return Err("freeze paths must be local leaf names".into());
        }
        let path = freeze_path.parent().unwrap_or(Path::new(".")).join(name);
        if hash(&bytes(&path)?) != *expected {
            return Err(format!("frozen input changed: {name}"));
        }
    }
    for (path, actual) in [
        (&options.manifest, hash(&manifest_bytes)),
        (&options.corpus, hash(&corpus_bytes)),
    ] {
        let name = path
            .file_name()
            .and_then(|x| x.to_str())
            .ok_or("invalid input filename")?;
        if freeze.files.get(name) != Some(&actual) {
            return Err(format!("input not bound by freeze: {name}"));
        }
    }
    if manifest.version != 1
        || corpus.version != 1
        || manifest.artifact_generation != corpus.artifact_generation
        || manifest.source_revision != corpus.source_revision
    {
        return Err("unsupported inputs or artifact/source identity mismatch".into());
    }
    if manifest.profiles.is_empty()
        || manifest.profiles.len() > 2
        || manifest
            .profiles
            .iter()
            .map(|p| p.as_str())
            .collect::<BTreeSet<_>>()
            .len()
            != manifest.profiles.len()
        || manifest
            .profiles
            .iter()
            .any(|p| !matches!(p.as_str(), "bge-base" | "embeddinggemma"))
    {
        return Err("manifest profiles must be unique stored BGE/Gemma profiles".into());
    }
    if manifest.scenarios.is_empty()
        || manifest.scenarios.len() > 64
        || manifest.variants.is_empty()
        || manifest.variants.len() > 64
        || !(1..=1000).contains(&manifest.repetitions)
    {
        return Err("manifest evaluation dimensions exceed limits".into());
    }
    if corpus.records.is_empty()
        || corpus.records.len() > 32768
        || corpus
            .records
            .iter()
            .map(|r| &r.record_key)
            .collect::<BTreeSet<_>>()
            .len()
            != corpus.records.len()
    {
        return Err("corpus keys must be nonempty, unique and at most 32768".into());
    }
    if manifest
        .scenarios
        .iter()
        .map(|s| &s.id)
        .collect::<BTreeSet<_>>()
        .len()
        != manifest.scenarios.len()
        || manifest
            .variants
            .iter()
            .map(|s| &s.id)
            .collect::<BTreeSet<_>>()
            .len()
            != manifest.variants.len()
    {
        return Err("duplicate scenario or variant IDs".into());
    }
    for id in manifest
        .scenarios
        .iter()
        .map(|s| &s.id)
        .chain(manifest.variants.iter().map(|v| &v.id))
    {
        if id.is_empty()
            || !id
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_')
        {
            return Err(format!("unsafe report ID: {id}"));
        }
    }
    validate_subset(
        &options.scenarios,
        manifest.scenarios.iter().map(|s| s.id.as_str()),
        "scenario",
    )?;
    validate_subset(
        &options.profiles,
        manifest.profiles.iter().map(|p| p.as_str()),
        "profile",
    )?;
    validate_subset(
        &options.variants,
        manifest.variants.iter().map(|v| v.id.as_str()),
        "variant",
    )?;
    if options.build_revision.len() != 40
        || !options
            .build_revision
            .bytes()
            .all(|x| x.is_ascii_hexdigit())
    {
        return Err("--build-revision requires a complete 40-character commit hash".into());
    }
    if !(1..=1000).contains(&manifest.performance.warm_repetitions) {
        return Err("invalid warm repetition count".into());
    }
    Ok(Inputs {
        manifest,
        corpus,
        manifest_sha256: hash(&manifest_bytes),
        corpus_sha256: hash(&corpus_bytes),
        freeze_sha256: hash(&freeze_bytes),
    })
}
fn validate_subset<'a>(
    requested: &[String],
    available: impl Iterator<Item = &'a str>,
    kind: &str,
) -> Result<(), String> {
    let allowed = available.collect::<BTreeSet<_>>();
    if requested.iter().collect::<BTreeSet<_>>().len() != requested.len() {
        return Err(format!("duplicate {kind} selection"));
    }
    for item in requested {
        if !allowed.contains(item.as_str()) {
            return Err(format!("unknown {kind}: {item}"));
        }
    }
    Ok(())
}
pub(super) fn variants(
    manifest: &Manifest,
    scenario: &Scenario,
    requested: &[String],
) -> Result<(Vec<SimilarityEvaluationVariant>, Vec<String>), String> {
    let mut variants = Vec::new();
    let mut skipped = Vec::new();
    let selected = match &scenario.selection {
        Selection::All => None,
        Selection::Explicit {
            selectors,
            artifact_generation,
        } => {
            if artifact_generation != &manifest.artifact_generation
                || selectors.is_empty()
                || selectors.iter().collect::<BTreeSet<_>>().len() != selectors.len()
            {
                return Err(format!("invalid exact selection for {}", scenario.id));
            }
            Some(selectors.clone())
        }
    };
    for variant in &manifest.variants {
        if !requested.is_empty() && !requested.contains(&variant.id) {
            continue;
        }
        let mut options = variant.options.clone();
        match &mut options {
            SimilarityOptions::Passages { parts, .. }
            | SimilarityOptions::ContentAverage { parts, .. } => *parts = selected.clone(),
            SimilarityOptions::Baseline {} => (),
        }
        if matches!(scenario.seed, SimilaritySeed::Trait { .. })
            && options.candidate_parts() == atlas_domain::SimilarityCandidateParts::Corresponding
        {
            skipped.push(variant.id.clone());
            continue;
        }
        options.validate(&scenario.seed)?;
        variants.push(SimilarityEvaluationVariant {
            id: variant.id.clone(),
            options,
        });
    }
    Ok((variants, skipped))
}
pub(super) fn report_path(dir: &Path, scenario: &str, profile: &str, phase: &str) -> PathBuf {
    dir.join(format!("{scenario}--{profile}--{phase}.json"))
}
