//! Bounded developer evidence, not a product command or persisted receipt.
//! Usage: cargo run -p atlas-ingest --example source_enrichment_probe -- SOURCE_ROOT LOCALE_JSON
use atlas_foundry_model::{decode_snapshot, encode_snapshot};
use atlas_ingest::{
    EnrichedDocumentOutcome, SourceEnrichmentContext, enrich_loaded_source, load_foundry_documents,
};
use atlas_record::{
    source_content::{
        ContentAudience, ContentReferenceResolution, ContentVisibilityRule, LocalizationResolver,
    },
    source_record::{
        SourceContentStatus, SourceFieldView, SourceQueryView, SourceReferenceIndex,
        enrich_source_record,
    },
};
use scraper::{Html, Selector};
use serde::Serialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    error::Error,
    fs::File,
    time::{Duration, Instant},
};

struct Locale(BTreeMap<String, String>);
impl LocalizationResolver for Locale {
    fn localized_value(&self, key: &str) -> Option<&str> {
        self.0.get(key).map(String::as_str)
    }
}
fn flatten(value: &Value, path: &str, out: &mut BTreeMap<String, String>) {
    match value {
        Value::String(text) => {
            out.insert(path.into(), text.clone());
        }
        Value::Object(map) => {
            for (key, value) in map {
                flatten(
                    value,
                    &if path.is_empty() {
                        key.clone()
                    } else {
                        format!("{path}.{key}")
                    },
                    out,
                );
            }
        }
        _ => {}
    }
}
fn compare<T: Serialize>(
    field: &str,
    view: SourceFieldView<'_, T>,
    raw: Option<&Value>,
) -> Result<usize, Box<dyn Error>> {
    if let SourceFieldView::Value(value) = view {
        let projected = serde_json::to_value(value)?;
        if raw != Some(&projected) {
            return Err(format!("projection mismatch {field}: {projected} versus {raw:?}").into());
        }
        Ok(1)
    } else {
        Ok(0)
    }
}
fn markers(
    prepared: &atlas_record::source_content::PreparedSourceContent,
) -> Result<(), Box<dyn Error>> {
    let fragment = Html::parse_fragment(&prepared.html);
    for (attribute, count) in [
        ("data-atlas-reference", prepared.references.len()),
        ("data-atlas-interaction", prepared.interactions.len()),
    ] {
        let mut seen = vec![0usize; count];
        let selector =
            Selector::parse(&format!("[{attribute}]")).map_err(|error| error.to_string())?;
        for element in fragment.select(&selector) {
            let ordinal = element
                .value()
                .attr(attribute)
                .ok_or("missing marker")?
                .parse::<usize>()?;
            let slot = seen.get_mut(ordinal).ok_or("orphan marker")?;
            *slot += 1;
        }
        for (ordinal, hits) in seen.iter().enumerate() {
            let expected =
                attribute == "data-atlas-interaction" || prepared.references[ordinal].visible;
            if *hits != usize::from(expected) {
                return Err(format!(
                    "marker mismatch {attribute}/{ordinal} in {:?}: {hits} hits",
                    prepared.locator
                )
                .into());
            }
        }
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.len() != 2 {
        return Err("usage: source_enrichment_probe SOURCE_ROOT LOCALE_JSON".into());
    }
    let mut strings = BTreeMap::new();
    flatten(
        &serde_json::from_reader::<_, Value>(File::open(&args[1])?)?,
        "",
        &mut strings,
    );
    let locale = Locale(strings);
    // Explicit source-inspection policy; this is not the product's default.
    let audience = ContentAudience {
        include_gm: false,
        include_owner: false,
        implicit_check_dc: ContentVisibilityRule::Gm,
    };
    eprintln!("Loading typed source");
    let start = Instant::now();
    let loaded = load_foundry_documents(&args[0], None)?;
    let loading = start.elapsed();
    let mut originals = BTreeMap::new();
    let mut source_bytes = 0usize;
    eprintln!("Capturing checked source identities");
    for pack in &loaded.packs {
        for document in &pack.documents {
            source_bytes += document.bytes.len();
            if let Some(model) = &document.admission.model {
                originals.insert(
                    document.provenance.source_path.clone(),
                    Sha256::digest(encode_snapshot(model)?),
                );
            }
        }
    }
    eprintln!("Enriching every retained source document");
    let start = Instant::now();
    let enriched = enrich_loaded_source(
        loaded,
        SourceEnrichmentContext {
            audience,
            localization: Some(&locale),
        },
    );
    let preparation = start.elapsed();
    let report = enriched.report();
    if report.discovered_files != report.retained_documents + report.quarantined_files {
        return Err("source accounting mismatch".into());
    }
    let mut snapshot_bytes = 0usize;
    let mut enrichment_bytes = 0usize;
    let mut html_bytes = 0usize;
    let mut text_bytes = 0usize;
    let mut encode = Duration::ZERO;
    let mut decode = Duration::ZERO;
    let mut compared = 0usize;
    let mut prepared_fields = 0usize;
    let mut projections = 0usize;
    let mut diagnostic_examples = BTreeMap::<String, Vec<Value>>::new();
    let mut reference_resolutions = BTreeMap::<&str, usize>::new();
    let mut largest = Vec::new();
    let mut resolver = SourceReferenceIndex::default();
    eprintln!("Checking snapshots, projections and emitted markers");
    for pack in &enriched.packs {
        for document in &pack.documents {
            let model = match &document.outcome {
                EnrichedDocumentOutcome::Addressed(record) => Some(&record.source),
                EnrichedDocumentOutcome::Unavailable { source, .. } => source.as_ref(),
            };
            if let Some(model) = model {
                let start = Instant::now();
                let snapshot = encode_snapshot(model)?;
                encode += start.elapsed();
                snapshot_bytes += snapshot.len();
                if originals.get(&document.provenance.source_path)
                    != Some(&Sha256::digest(&snapshot))
                {
                    return Err(
                        format!("source changed: {}", document.provenance.source_path).into(),
                    );
                }
                let start = Instant::now();
                let decoded = decode_snapshot(&snapshot)?;
                decode += start.elapsed();
                if &decoded != model {
                    return Err("checked snapshot equality mismatch".into());
                }
                compared += 1;
            }
            if let EnrichedDocumentOutcome::Addressed(record) = &document.outcome {
                resolver.insert_source(&record.key, &record.source);
                let sidecar = serde_json::to_vec(&record.enrichment)?;
                enrichment_bytes += sidecar.len();
                let query = SourceQueryView::new(&record.source, &pack.name, &pack.label);
                let raw: Value = serde_json::from_slice(&document.bytes)?;
                if query.source.document_kind() != pack.document_type {
                    return Err("source kind mismatch".into());
                }
                projections += compare("source.type", query.source.source_type(), raw.get("type"))?;
                let publication = if pack.document_type == "Actor" {
                    "/system/details/publication"
                } else {
                    "/system/publication"
                };
                projections += compare(
                    "publication.title",
                    query.publication_title(),
                    raw.pointer(&format!("{publication}/title")),
                )?;
                projections += compare(
                    "publication.remaster",
                    query.publication_remaster(),
                    raw.pointer(&format!("{publication}/remaster")),
                )?;
                projections += compare(
                    "traits",
                    query.traits(),
                    raw.pointer("/system/traits/value"),
                )?;
                projections += compare(
                    "rarity",
                    query.rarity(),
                    raw.pointer("/system/traits/rarity"),
                )?;
                projections += compare(
                    "actor.level",
                    query.actor().level(),
                    raw.pointer("/system/details/level/value"),
                )?;
                projections += compare(
                    "actor.armor_class",
                    query.actor().armor_class(),
                    raw.pointer("/system/attributes/ac/value"),
                )?;
                projections += compare(
                    "actor.hp.maximum",
                    query.actor().hp_maximum(),
                    raw.pointer("/system/attributes/hp/max"),
                )?;
                projections += compare(
                    "hazard.hardness",
                    query.actor().hazard_hardness(),
                    raw.pointer("/system/attributes/hardness"),
                )?;
                projections += compare(
                    "hazard.complexity",
                    query.actor().hazard_complexity(),
                    raw.pointer("/system/details/isComplex"),
                )?;
                if let atlas_record::source_record::SourceNodeView::Item(item) = query.source {
                    projections += compare(
                        "spell.rank",
                        item.spell_rank(),
                        raw.pointer("/system/level/value"),
                    )?;
                    projections += compare(
                        "spell.traditions",
                        item.spell_traditions(),
                        raw.pointer("/system/traits/traditions"),
                    )?;
                }
                let mut record_html = 0;
                let mut record_text = 0;
                for content in &record.enrichment.content {
                    if let SourceContentStatus::Prepared(prepared) = &content.status {
                        markers(prepared)?;
                        for reference in &prepared.references {
                            let state = match reference.resolution {
                                ContentReferenceResolution::Resolved(_) => "resolved",
                                ContentReferenceResolution::UnverifiedUrl { .. } => {
                                    "unverified_url"
                                }
                                ContentReferenceResolution::Unresolved => "unresolved",
                                ContentReferenceResolution::Blocked => "blocked",
                            };
                            *reference_resolutions.entry(state).or_default() += 1;
                        }
                        prepared_fields += 1;
                        record_html += prepared.html.len();
                        record_text += prepared.text.len();
                        for diagnostic in &prepared.diagnostics {
                            let samples = diagnostic_examples
                                .entry(format!("{:?}", diagnostic.code))
                                .or_default();
                            if samples.len() < 3 {
                                samples.push(json!({"source_path":document.provenance.source_path,"locator":content.locator,"detail":diagnostic.detail}));
                            }
                        }
                    }
                }
                html_bytes += record_html;
                text_bytes += record_text;
                largest.push((
                    document.bytes.len(),
                    document.provenance.source_path.as_str(),
                    record,
                    sidecar.len(),
                    record_html,
                    record_text,
                ));
            }
        }
    }
    largest.sort_by_key(|entry| std::cmp::Reverse(entry.0));
    largest.truncate(5);
    let mut warm = Vec::new();
    for (bytes, path, record, sidecar, html, text) in largest {
        let mut repeats = Vec::new();
        for _ in 0..3 {
            let source = record.source.clone();
            let start = Instant::now();
            let prepared = enrich_source_record(
                record.key.clone(),
                source,
                audience,
                Some(&locale),
                Some(&resolver),
            );
            let elapsed = start.elapsed();
            if prepared.enrichment != record.enrichment {
                return Err("warm preparation mismatch".into());
            }
            repeats.push(elapsed.as_secs_f64() * 1000.0);
        }
        warm.push(json!({"source_path":path,"source_bytes":bytes,"snapshot_bytes":encode_snapshot(&record.source)?.len(),"enrichment_bytes":sidecar,"prepared_html_bytes":html,"prepared_text_bytes":text,"warm_preparation_ms":repeats}));
    }
    println!(
        "{}",
        serde_json::to_string_pretty(
            &json!({"source_contract":atlas_foundry_model::SOURCE_CONTRACT_ID,"source_root":args[0],"audience":{"include_gm":false,"include_owner":false,"implicit_check_dc":"gm"},"report":report,"reference_resolutions":reference_resolutions,"diagnostic_examples":diagnostic_examples,"checked_source_roundtrips":compared,"verified_projection_values":projections,"verified_prepared_fields":prepared_fields,"bytes":{"authored":source_bytes,"checked_snapshots":snapshot_bytes,"enrichment_including_prepared_content":enrichment_bytes,"prepared_html":html_bytes,"prepared_text":text_bytes},"seconds":{"load_and_admit":loading.as_secs_f64(),"identity_index_and_enrichment":preparation.as_secs_f64(),"snapshot_encode":encode.as_secs_f64(),"snapshot_decode":decode.as_secs_f64()},"largest_roots":warm})
        )?
    );
    if enriched
        .report()
        .content_outcomes
        .get("preparation_failed")
        .copied()
        .unwrap_or(0)
        > 0
    {
        return Err("content preparation failures retained; inspect outcomes".into());
    }
    Ok(())
}
