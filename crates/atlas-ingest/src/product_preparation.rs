//! Bounded product preparation; rich developer inspection has a separate lifetime.
use crate::{
    BuildArtifactOptions, IngestError, SourceEnrichmentReport, SourceFingerprint,
    build::default_audience, index_build_input::DocumentAttribution,
    source::localization::LocalizationCatalog, source_fingerprint::SourceCapture,
};
use atlas_domain::{RecordKey, SourcePassageAddress, TraitKey};
use atlas_embedding::{PreparedNativeInput, TextEmbeddingTokenizer};
use atlas_foundry_model::{SourceAdmission, SourceContext, SourceValue, admit_document_source};
use atlas_index::{
    BuildFrameOffset, BuildFrameStream, IndexBuildPack, PreparedRecordFrame, PublicationSpool,
    ReferenceReuseReader, SourceArtifactDiagnostic,
};
use atlas_record::source_record::{
    SourceBackedRecord, SourceContentOutcome, SourceContentStatus, SourceNodeView,
    SourceReferenceContribution, SourceReferenceIndex, SourceRelationshipOccurrence,
    prepare_record_content, resolve_source_relationships, select_record_search,
    source_owned_document_count, source_record_key,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

fn index(error: atlas_index::IndexError) -> IngestError {
    IngestError::ArtifactWriteFailed(error.to_string())
}
fn embedding(error: atlas_embedding::EmbeddingError) -> IngestError {
    IngestError::DocumentEmbeddingFailed(error.to_string())
}
#[derive(Serialize, Deserialize)]
struct PreviousPreparation {
    hash: String,
    content: Vec<SourceContentOutcome>,
    relationships: Vec<SourceRelationshipOccurrence>,
}
#[derive(Serialize, Deserialize)]
struct AdmittedFrame {
    key: RecordKey,
    pack: String,
    label: String,
    path: String,
    hash: String,
    snapshot: String,
    diagnostics: Vec<atlas_foundry_model::SourceDiagnostic>,
    previous: Option<PreviousPreparation>,
}
struct Envelope {
    key: Option<RecordKey>,
    pack: String,
    kind: String,
    path: String,
    names: Vec<String>,
    reason: Option<String>,
    partial: bool,
    contribution: Option<SourceReferenceContribution>,
    labels: BTreeMap<String, String>,
    alias_name: Option<String>,
    family: Option<String>,
    journal: Option<BuildFrameOffset>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) enum RawOccurrence {
    Document {
        input: usize,
        attribution: DocumentAttribution,
    },
    Trait {
        input: usize,
        key: TraitKey,
        address: SourcePassageAddress,
    },
}
/// Rich source and prepared-record batches drain before the next batch is read.
const PREPARATION_REUSE_BATCH: usize = 32;

/// Simultaneously owned batch payloads at a read/processing boundary. These
/// counts exclude compact metadata and one record's processing temporaries.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct PreparationBatchOverlap {
    pub raw_sources: usize,
    pub retained_records: usize,
    pub admitted_records: usize,
    pub prepared_records: usize,
    pub layout_roots: usize,
}
impl PreparationBatchOverlap {
    fn total(self) -> usize {
        self.raw_sources
            + self.retained_records
            + self.admitted_records
            + self.prepared_records
            + self.layout_roots
    }
}

/// Observed work/byte counters, separate from process allocation measurements.
#[derive(Debug, Default)]
pub(crate) struct PreparationWork {
    pub selection_calls: usize,
    pub captured_document_reads: usize,
    pub admitted_record_reads: usize,
    pub alias_journal_reads: usize,
    pub alias_journal_fresh_preparations: usize,
    pub alias_journal_reused_preparations: usize,
    pub main_fresh_preparations: usize,
    pub largest_admitted_batch: usize,
    pub largest_content_overlap: PreparationBatchOverlap,
    pub reusable_record_read_calls: usize,
    pub reusable_record_read_batches: usize,
    pub reusable_layout_read_calls: usize,
    pub reusable_layout_read_batches: usize,
    pub largest_admission_batch_source_bytes: usize,
    pub largest_admission_overlap: PreparationBatchOverlap,
    pub largest_semantic_overlap: PreparationBatchOverlap,
    pub largest_source_bytes: usize,
    pub largest_admitted_frame_bytes: u64,
    pub largest_prepared_frame_bytes: u64,
}
pub(crate) struct PreparedSourceBuild {
    pub spool: PublicationSpool,
    pub native_inputs: BuildFrameStream<PreparedNativeInput>,
    pub occurrences: BuildFrameStream<RawOccurrence>,
    pub source_report: SourceEnrichmentReport,
    pub fingerprint: SourceFingerprint,
    pub context_shortened: usize,
    pub reused_sections: usize,
    pub document_units: usize,
    pub work: PreparationWork,
    // Field order closes every child stream before its fixture-owned directory.
    #[cfg(test)]
    _scratch: Option<tempfile::TempDir>,
}

#[cfg(test)]
impl PreparedSourceBuild {
    pub(crate) fn fixture_scratch_path(&self) -> &std::path::Path {
        self._scratch
            .as_ref()
            .expect("fixture owns its scratch")
            .path()
    }
}

fn envelope(raw: &SourceValue) -> (Vec<String>, Vec<String>) {
    let SourceValue::Object(raw) = raw else {
        return (Vec::new(), Vec::new());
    };
    let ids = raw
        .fields()
        .iter()
        .filter_map(|(name, value)| match (name.as_str(), value) {
            ("_id", SourceValue::String(value)) => Some(value.clone()),
            _ => None,
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let names = raw
        .fields()
        .iter()
        .filter_map(|(name, value)| match (name.as_str(), value) {
            ("name", SourceValue::String(value)) => Some(value.clone()),
            _ => None,
        })
        .collect();
    (ids, names)
}

pub(crate) fn prepare(
    options: &BuildArtifactOptions,
    reuse: &ReferenceReuseReader,
    scratch: &std::path::Path,
) -> Result<PreparedSourceBuild, IngestError> {
    let capture = SourceCapture::capture(&options.source_root, options.manifest_path.as_deref())?;
    tracing::debug!(target:"atlas_build_diagnostic",stage="source_capture",captured_files=capture.fingerprint.file_count,"Captured source inventory and manifest");
    capture.require_complete_packs()?;
    crate::source_compatibility::validate_parsed_manifest(
        &capture.manifest_path,
        &capture.manifest().manifest,
    )?;
    let catalog = LocalizationCatalog::load_captured(
        &options.source_root,
        &options.locale,
        options.manifest_path.as_deref(),
        Some(&capture),
    )?;
    let fingerprint = capture.fingerprint.clone();
    let mut context = crate::build::build_context(options, &catalog, &fingerprint)?;
    let preparation_hash = context.preparation_sha256().map_err(index)?;
    let reuse_preparation = reuse
        .preparation_matches(&preparation_hash)
        .map_err(index)?;
    let manifest = capture.manifest();
    let packs = manifest
        .manifest
        .packs
        .iter()
        .map(|p| IndexBuildPack {
            pack_id: p.name.clone(),
            label: p.label.clone(),
            document_kind: p.document_type.clone(),
            declared_path: p.path.clone(),
        })
        .collect::<Vec<_>>();
    let identities = reuse.identities().map_err(index)?;
    // Bind each captured physical path once, then admit that owned batch separately
    // for every declaration. Per-pack streams retain manifest order even when two
    // declarations share a directory and use different admission/reuse contexts.
    let mut bindings = BTreeMap::<std::path::PathBuf, Vec<usize>>::new();
    let mut admitted = BuildFrameStream::<AdmittedFrame>::new_in(scratch).map_err(index)?;
    let mut admitted_offsets = (0..packs.len()).map(|_| Vec::new()).collect::<Vec<_>>();
    let mut envelopes = (0..packs.len()).map(|_| Vec::new()).collect::<Vec<_>>();
    for (position, pack) in packs.iter().enumerate() {
        for path in capture.paths(&capture.pack(&pack.declared_path).0) {
            if path.extension().is_some_and(|s| s == "json")
                && path.file_name().is_none_or(|s| s != "_folders.json")
            {
                bindings.entry(path).or_default().push(position);
            }
        }
    }
    let bindings = bindings.into_iter().collect::<Vec<_>>();
    let mut id_counts = BTreeMap::new();
    let mut excluded = Vec::new();
    let mut report = SourceEnrichmentReport::default();
    let mut work = PreparationWork::default();
    for bindings in bindings.chunks(PREPARATION_REUSE_BATCH) {
        let mut owned = Vec::with_capacity(bindings.len());
        for (path, positions) in bindings {
            let path_relative =
                crate::source::discovery::relative_source_path(&options.source_root, path);
            let source = capture.read_verified(path)?;
            work.captured_document_reads += 1;
            work.largest_source_bytes = work.largest_source_bytes.max(source.bytes.len());
            owned.push((path_relative, source.bytes, source.sha256, positions));
        }
        tracing::trace!(target:"atlas_build_diagnostic",stage="captured_document_batch",source_files=owned.len(),"Owned checked document batch");
        work.largest_admission_batch_source_bytes = work
            .largest_admission_batch_source_bytes
            .max(owned.iter().map(|(_, bytes, _, _)| bytes.len()).sum());
        let positions = owned
            .iter()
            .flat_map(|(_, _, _, positions)| positions.iter().copied())
            .collect::<BTreeSet<_>>();
        for position in positions {
            let pack = &packs[position];
            let captured = owned
                .iter()
                .filter(|(_, _, _, positions)| positions.contains(&position))
                .map(|(path, bytes, hash, _)| {
                    let previous = identities
                        .get(&(pack.pack_id.clone(), path.clone()))
                        .filter(|old| {
                            old.content_hash == *hash && old.document_kind == pack.document_kind
                        })
                        .map(|old| old.key.clone());
                    (path.clone(), bytes.as_slice(), hash.clone(), previous)
                })
                .collect::<Vec<_>>();
            report.discovered_files += captured.len();
            let requests = captured
                .iter()
                .filter_map(|(_, _, _, key)| key.as_ref())
                .collect::<Vec<_>>();
            work.reusable_record_read_calls += requests.len();
            work.reusable_record_read_batches += usize::from(!requests.is_empty());
            let previous = if reuse_preparation {
                reuse
                    .read_records(&requests)
                    .map_err(index)?
                    .into_iter()
                    .map(|old| {
                        old.map(|old| {
                            (
                                old.record,
                                old.diagnostics,
                                Some(PreviousPreparation {
                                    hash: old.preparation_context_hash,
                                    content: old.content,
                                    relationships: old.relationships,
                                }),
                            )
                        })
                    })
                    .collect::<Vec<_>>()
            } else {
                reuse
                    .read_admissions(&requests)
                    .map_err(index)?
                    .into_iter()
                    .map(|old| old.map(|old| (old.record, old.diagnostics, None)))
                    .collect::<Vec<_>>()
            };
            let overlap = PreparationBatchOverlap {
                raw_sources: owned.len(),
                retained_records: previous.iter().filter(|p| p.is_some()).count(),
                ..Default::default()
            };
            if overlap.total() > work.largest_admission_overlap.total() {
                work.largest_admission_overlap = overlap;
            }
            let mut previous = previous.into_iter();
            for (path_relative, bytes, hash, previous_key) in captured {
                let previous = if previous_key.is_some() {
                    previous.next().flatten()
                } else {
                    None
                };
                let mut reused = false;
                let mut previous_preparation = None;
                let admission = if let Some((record, diagnostics, preparation)) =
                    previous.filter(|(_, diagnostics, _)| {
                        diagnostics.iter().all(|d| d.source_diagnostic.is_some())
                    }) {
                    let raw = atlas_foundry_model::parse_source_value(bytes);
                    match raw {
                        Ok(raw) => {
                            reused = true;
                            previous_preparation = preparation;
                            Ok(SourceAdmission {
                                raw,
                                model: Some(record.into_source()),
                                diagnostics: diagnostics
                                    .into_iter()
                                    .filter_map(|d| d.source_diagnostic)
                                    .collect(),
                            })
                        }
                        Err(error) => Err(error.to_string()),
                    }
                } else {
                    admit_document_source(
                        &pack.document_kind,
                        SourceContext::new(
                            format!("{}:{}", pack.pack_id, path_relative),
                            &path_relative,
                            "$",
                        ),
                        bytes,
                    )
                    .map_err(|error| error.to_string())
                };
                let admission = match admission {
                    Ok(value) => value,
                    Err(error) => {
                        report.quarantined_files += 1;
                        excluded.push(atlas_index::SourceExcludedInput {
                            pack: pack.pack_id.clone(),
                            path: path_relative,
                            reason: format!("Parse: {error}"),
                        });
                        continue;
                    }
                };
                report.retained_documents += 1;
                report.reused_admissions += usize::from(reused);
                report.admission_diagnostics += admission.diagnostics.len();
                let (raw_ids, names) = envelope(&admission.raw);
                for id in &raw_ids {
                    *id_counts
                        .entry((pack.pack_id.clone(), id.clone()))
                        .or_insert(0usize) += 1;
                }
                let partial = !admission.diagnostics.is_empty();
                let mut observation = Envelope {
                    key: None,
                    pack: pack.pack_id.clone(),
                    kind: pack.document_kind.clone(),
                    path: path_relative.clone(),
                    names,
                    reason: None,
                    partial,
                    contribution: None,
                    labels: BTreeMap::new(),
                    alias_name: None,
                    family: None,
                    journal: None,
                };
                if let Some(source) = admission.model {
                    report.typed_documents += 1;
                    report.partial_documents += usize::from(partial);
                    match source_record_key(&pack.pack_id, &source) {
                        Ok(key) => {
                            let record =
                                SourceBackedRecord::new(&pack.pack_id, source).map_err(|e| {
                                    IngestError::SourceSelectionFailed(format!("{:?}", e.reason))
                                })?;
                            observation.labels =
                                crate::source::localization::record_trait_labels(&record, &catalog);
                            observation.contribution =
                                Some(SourceReferenceIndex::contribution(&key, record.source()));
                            observation.key = Some(key.clone());
                            let snapshot = atlas_foundry_model::encode_snapshot(record.source())
                                .map_err(|e| IngestError::SourceSelectionFailed(e.to_string()))?;
                            let snapshot = String::from_utf8(snapshot)
                                .map_err(|e| IngestError::SourceSelectionFailed(e.to_string()))?;
                            let node = SourceNodeView::from(record.source());
                            if node.document_kind() != "Macro" {
                                observation.alias_name = node.name().value().cloned();
                            }
                            observation.family = node.source_type().value().map(str::to_owned);
                            let journal = node.document_kind() == "JournalEntry"
                                && node
                                    .name()
                                    .value()
                                    .is_some_and(|name| name == "Remaster Changes");
                            let offset = admitted
                                .push_indexed(&AdmittedFrame {
                                    key,
                                    pack: pack.pack_id.clone(),
                                    label: pack.label.clone(),
                                    path: path_relative,
                                    hash,
                                    snapshot,
                                    diagnostics: admission.diagnostics,
                                    previous: previous_preparation,
                                })
                                .map_err(index)?;
                            admitted_offsets[position].push(offset);
                            observation.journal = journal.then_some(offset);
                        }
                        Err(error) => observation.reason = Some(format!("{error:?}")),
                    }
                } else {
                    report.raw_only_documents += 1;
                    observation.reason = Some("UnsupportedRoot".into());
                }
                envelopes[position].push(observation);
            }
        }
    }
    let mut resolver = SourceReferenceIndex::default();
    let mut active = BTreeSet::new();
    let mut names = BTreeMap::new();
    let mut families = BTreeMap::new();
    let mut journals = Vec::new();
    let mut partial_records = 0usize;
    for mut observation in envelopes.into_iter().flatten() {
        if let Some(key) = &observation.key
            && id_counts
                .get(&(key.pack().as_str().to_owned(), key.id().as_str().to_owned()))
                .copied()
                .unwrap_or(0)
                > 1
        {
            observation.reason = Some("DuplicateId".into());
        }
        if observation.reason.is_none() {
            if let Some(key) = observation.key {
                if let Some(name) = observation.alias_name {
                    names.insert(key.clone(), name);
                }
                families.insert(key.clone(), observation.family);
                if let Some(offset) = observation.journal {
                    journals.push((key.clone(), offset));
                }
                active.insert(key);
                report.addressed_documents += 1;
                partial_records += usize::from(observation.partial);
                context.used_trait_labels.extend(observation.labels);
            }
            if let Some(contribution) = observation.contribution {
                resolver.extend_contribution(contribution);
            }
        } else {
            for name in observation.names {
                resolver.exclude_root_name(&observation.pack, &observation.kind, &name);
            }
            let reason = observation
                .reason
                .unwrap_or_else(|| "UnsupportedRoot".into());
            *report
                .identity_unavailable
                .entry(reason.clone())
                .or_default() += 1;
            excluded.push(atlas_index::SourceExcludedInput {
                pack: observation.pack,
                path: observation.path,
                reason,
            });
        }
    }
    if report.discovered_files > 0 && report.addressed_documents == 0 {
        return Err(IngestError::NoRecordsLoaded);
    }
    let order = packs
        .iter()
        .enumerate()
        .map(|(i, p)| (p.pack_id.as_str(), i))
        .collect::<BTreeMap<_, _>>();
    excluded.sort_by(|a, b| {
        (
            order.get(a.pack.as_str()),
            !a.reason.starts_with("Parse:"),
            &a.path,
        )
            .cmp(&(
                order.get(b.pack.as_str()),
                !b.reason.starts_with("Parse:"),
                &b.path,
            ))
    });
    context.admission = Some(atlas_index::SourceAdmissionSummary {
        discovered_files: report.discovered_files,
        admitted_records: report.addressed_documents,
        partial_records,
        quarantined_files: report.quarantined_files,
        excluded_documents: report.retained_documents - report.addressed_documents,
        exclusions: excluded,
    });
    let inventory = atlas_index::CompleteSourceInventory::checked(
        &context.source_fingerprint,
        packs.iter().map(|p| p.pack_id.clone()),
        packs.iter().map(|p| p.pack_id.clone()),
    )
    .map_err(index)?;
    let mut spool =
        PublicationSpool::new_in(scratch, context, packs, inventory, reuse.expected().clone())
            .map_err(index)?;
    // Alias journals need the complete resolver but ordinary roots need aliases
    // before lexical/identity selection. Read only those journals here, one rich
    // record at a time; their content is deliberately prepared again in the final
    // slice. Shared search selection still runs exactly once per active root.
    let mut admitted_reader = admitted.cursor().map_err(index)?;
    let (aliases, pairs) = crate::source_aliases::extract_prepared_aliases(
        &names,
        &families,
        &mut |visitor| {
            for (key, offset) in &journals {
                let mut frame = admitted_reader.read_at(*offset).map_err(index)?;
                if &frame.key != key {
                    return Err(index(atlas_index::IndexError::Invalid(
                        "journal frame identity differs".into(),
                    )));
                }
                let content = prepare_content(&mut frame, &catalog, &resolver, &preparation_hash)
                    .map_err(index)?;
                work.alias_journal_reads += 1;
                work.alias_journal_fresh_preparations += usize::from(!content.reused);
                work.alias_journal_reused_preparations += usize::from(content.reused);
                visitor(key, &content.content)?;
            }
            Ok(())
        },
        &options.source_root,
        &capture,
    )?;
    let mut by_record = BTreeMap::<RecordKey, Vec<String>>::new();
    for alias in aliases {
        by_record
            .entry(alias.record.clone())
            .or_default()
            .push(alias.alias.clone());
        spool.aliases.push(&alias).map_err(index)?;
    }
    for pair in pairs {
        spool.remaster_pairs.push(&pair).map_err(index)?;
    }
    let tokenizer = options
        .search_profiles
        .first()
        .map(|c| TextEmbeddingTokenizer::load_baseline(&c.cache_root))
        .transpose()
        .map_err(embedding)?;
    let mut native_inputs = BuildFrameStream::new_in(scratch).map_err(index)?;
    let mut occurrences = BuildFrameStream::new_in(scratch).map_err(index)?;
    let mut context_shortened = 0usize;
    let mut reused_sections = 0usize;
    let mut document_units = 0usize;
    let mut batch = Vec::with_capacity(PREPARATION_REUSE_BATCH);
    let mut consume_batch =
        |batch: &mut Vec<AdmittedFrame>| -> Result<(), atlas_index::IndexError> {
            work.largest_admitted_batch = work.largest_admitted_batch.max(batch.len());
            let count = batch.len();
            let mut prepared = Vec::with_capacity(count);
            for (position, mut frame) in batch.drain(..).enumerate() {
                let PreparedContent {
                    record,
                    content,
                    relationships,
                    reused,
                } = prepare_content(&mut frame, &catalog, &resolver, &preparation_hash)?;
                report.reused_preparations += usize::from(reused);
                work.main_fresh_preparations += usize::from(!reused);
                count_prepared(&mut report, &record, &content, relationships.len());
                work.selection_calls += 1;
                let selection = select_record_search(
                    &record,
                    &content,
                    default_audience(),
                    &frame.label,
                    &spool.context.used_trait_labels,
                )
                .map_err(|error| atlas_index::IndexError::Invalid(error.to_string()))?;
                prepared.push(PreparedRecordFrame {
                    pack: frame.pack,
                    snapshot: frame.snapshot,
                    source_path: frame.path,
                    content_hash: frame.hash,
                    preparation_context_hash: preparation_hash.clone(),
                    content,
                    relationships,
                    diagnostics: frame
                        .diagnostics
                        .into_iter()
                        .map(|d| SourceArtifactDiagnostic {
                            owners: None,
                            field: Some(d.json_path.clone()),
                            stage: "admission".into(),
                            code: "unparseable_source_field".into(),
                            message: d.to_string(),
                            source_diagnostic: Some(d),
                        })
                        .collect(),
                    selection,
                    lexical: Vec::new(),
                });
                let overlap = PreparationBatchOverlap {
                    admitted_records: count - position - 1,
                    prepared_records: prepared.len(),
                    ..Default::default()
                };
                if overlap.total() > work.largest_content_overlap.total() {
                    work.largest_content_overlap = overlap;
                }
            }
            let requests = if tokenizer.is_some() && options.reuse_passage_layouts {
                prepared
                    .iter()
                    .map(|frame| &frame.selection.root)
                    .collect::<Vec<_>>()
            } else {
                Vec::new()
            };
            work.reusable_layout_read_calls += requests.len();
            work.reusable_layout_read_batches += usize::from(!requests.is_empty());
            let layouts = if requests.is_empty() {
                prepared.iter().map(|_| Vec::new()).collect()
            } else {
                reuse.read_layouts(&requests)?
            };
            let overlap = PreparationBatchOverlap {
                prepared_records: prepared.len(),
                layout_roots: layouts.iter().filter(|r| !r.is_empty()).count(),
                ..Default::default()
            };
            if overlap.total() > work.largest_semantic_overlap.total() {
                work.largest_semantic_overlap = overlap;
            }
            for (mut frame, layouts) in prepared.into_iter().zip(layouts) {
                let aliases = by_record
                    .get(&frame.selection.root)
                    .map(Vec::as_slice)
                    .unwrap_or(&[]);
                frame.lexical = atlas_index::project_lexical_units(&frame.selection, aliases);
                for unit in &frame.lexical {
                    spool.lexical.push(unit)?;
                }
                frame
                    .selection
                    .identity_optional
                    .extend(aliases.iter().cloned());
                if let Some(tokenizer) = &tokenizer {
                    reused_sections += crate::index_build_input::semantic_units(
                        &frame.selection,
                        tokenizer,
                        &mut |attribution, input| {
                            let ordinal = native_inputs.len();
                            native_inputs
                                .push(&PreparedNativeInput::from(input))
                                .map_err(index)?;
                            occurrences
                                .push(&RawOccurrence::Document {
                                    input: ordinal,
                                    attribution,
                                })
                                .map_err(index)?;
                            document_units += 1;
                            Ok(())
                        },
                        &mut context_shortened,
                        &layouts,
                    )
                    .map_err(|e| atlas_index::IndexError::Invalid(e.to_string()))?;
                }
                spool.records.push(&frame)?;
            }
            Ok(())
        };
    let mut admitted_reads = 0;
    for (position, offsets) in admitted_offsets.iter().enumerate() {
        let pack = &manifest.manifest.packs[position];
        for offset in offsets {
            let frame = admitted_reader.read_at(*offset).map_err(index)?;
            admitted_reads += 1;
            if frame.pack != pack.name || frame.label != pack.label {
                return Err(index(atlas_index::IndexError::Invalid(
                    "admitted pack binding differs".into(),
                )));
            }
            if !active.contains(&frame.key) {
                continue;
            }
            batch.push(frame);
            if batch.len() == PREPARATION_REUSE_BATCH {
                consume_batch(&mut batch).map_err(index)?;
            }
        }
    }
    consume_batch(&mut batch).map_err(index)?;
    work.admitted_record_reads = admitted_reads;
    work.largest_admitted_frame_bytes = admitted.max_frame_bytes();
    let usage = admitted
        .file_usage()
        .map_err(index)?
        .combine(spool.file_usage().map_err(index)?)
        .map_err(index)?
        .combine(native_inputs.file_usage().map_err(index)?)
        .map_err(index)?
        .combine(occurrences.file_usage().map_err(index)?)
        .map_err(index)?;
    tracing::info!(target:"atlas_build_diagnostic",stage="admission_publication_overlap",file_bytes=usage.file_bytes,allocated_bytes=usage.allocated_bytes.unwrap_or(0),allocated_bytes_known=usage.allocated_bytes.is_some(),"Observed indexed admission and final publication files");
    drop(admitted_reader);
    drop(admitted);
    crate::traits::prepare_traits_stream(
        &catalog,
        &spool.context,
        tokenizer.as_ref(),
        &resolver,
        &mut |definition, inputs| {
            let key = definition.key.clone();
            spool.traits.push(&definition).map_err(index)?;
            for input in inputs {
                let ordinal = native_inputs.len();
                let address = input.address.clone();
                native_inputs
                    .push(&PreparedNativeInput::from(input))
                    .map_err(index)?;
                occurrences
                    .push(&RawOccurrence::Trait {
                        input: ordinal,
                        key: key.clone(),
                        address,
                    })
                    .map_err(index)?;
            }
            Ok(())
        },
    )?;
    work.largest_prepared_frame_bytes = spool.records.max_frame_bytes();
    tracing::info!(target:"atlas_build_diagnostic",phase="source_preparation",captured_document_reads=work.captured_document_reads,admitted_record_reads=work.admitted_record_reads,alias_journal_reads=work.alias_journal_reads,alias_journal_fresh_preparations=work.alias_journal_fresh_preparations,alias_journal_reused_preparations=work.alias_journal_reused_preparations,main_fresh_preparations=work.main_fresh_preparations,largest_admitted_batch=work.largest_admitted_batch,content_overlap_admitted_records=work.largest_content_overlap.admitted_records,content_overlap_prepared_records=work.largest_content_overlap.prepared_records,structural_reuse_record_batch=PREPARATION_REUSE_BATCH,reusable_record_read_batches=work.reusable_record_read_batches,reusable_layout_read_calls=work.reusable_layout_read_calls,reusable_layout_read_batches=work.reusable_layout_read_batches,largest_admission_batch_source_bytes=work.largest_admission_batch_source_bytes,admission_overlap_raw_sources=work.largest_admission_overlap.raw_sources,admission_overlap_retained_records=work.largest_admission_overlap.retained_records,semantic_overlap_prepared_records=work.largest_semantic_overlap.prepared_records,semantic_overlap_layout_roots=work.largest_semantic_overlap.layout_roots,selection_calls=work.selection_calls,reusable_record_read_calls=work.reusable_record_read_calls,largest_source_bytes=work.largest_source_bytes,largest_admitted_frame_bytes=work.largest_admitted_frame_bytes,largest_prepared_frame_bytes=work.largest_prepared_frame_bytes,record_frames=spool.records.len(),native_input_frames=native_inputs.len(),occurrence_frames=occurrences.len(),"Observed source preparation work and frame sizes");
    Ok(PreparedSourceBuild {
        spool,
        native_inputs,
        occurrences,
        source_report: report,
        fingerprint,
        context_shortened,
        reused_sections,
        document_units,
        work,
        #[cfg(test)]
        _scratch: None,
    })
}

struct PreparedContent {
    record: SourceBackedRecord,
    content: Vec<SourceContentOutcome>,
    relationships: Vec<SourceRelationshipOccurrence>,
    reused: bool,
}

fn prepare_content(
    frame: &mut AdmittedFrame,
    catalog: &LocalizationCatalog,
    resolver: &SourceReferenceIndex,
    preparation_hash: &str,
) -> Result<PreparedContent, atlas_index::IndexError> {
    let source = atlas_foundry_model::decode_snapshot(frame.snapshot.as_bytes())?;
    let record = SourceBackedRecord::new(&frame.pack, source)
        .map_err(|_| atlas_index::IndexError::Invalid("admitted identity differs".into()))?;
    if record.key() != &frame.key {
        return Err(atlas_index::IndexError::Invalid(
            "admitted snapshot root differs".into(),
        ));
    }
    let relationships = resolve_source_relationships(&record, Some(resolver));
    let (content, reused) = match frame.previous.take() {
        Some(previous)
            if previous.hash == preparation_hash
                && previous.relationships == relationships
                && crate::source_enrichment::references_unchanged(&previous.content, resolver) =>
        {
            (previous.content, true)
        }
        _ => (
            prepare_record_content(&record, default_audience(), Some(catalog), Some(resolver)),
            false,
        ),
    };
    Ok(PreparedContent {
        record,
        content,
        relationships,
        reused,
    })
}

fn count_prepared(
    report: &mut SourceEnrichmentReport,
    record: &SourceBackedRecord,
    content: &[SourceContentOutcome],
    relationships: usize,
) {
    report.owned_nodes += source_owned_document_count(record);
    report.relationships += relationships;
    for outcome in content {
        let state = match &outcome.status {
            SourceContentStatus::Prepared(p) => {
                report.references += p.references.len();
                for d in &p.diagnostics {
                    *report
                        .content_diagnostics
                        .entry(format!("{:?}", d.code))
                        .or_default() += 1;
                }
                "prepared"
            }
            SourceContentStatus::FormatUnavailable { .. } => "format_unavailable",
            SourceContentStatus::UnsupportedFormat { .. } => "unsupported_format",
            SourceContentStatus::PreparationFailed { .. } => "preparation_failed",
        };
        *report.content_outcomes.entry(state.into()).or_default() += 1;
    }
}

/// Tests exercise the product preparation path against a genuine temporary
/// workspace. Reuse is available only from its previously published artifact.
#[cfg(test)]
pub(crate) fn prepare_fixture(
    options: &BuildArtifactOptions,
) -> Result<PreparedSourceBuild, IngestError> {
    let workspace =
        atlas_index::test_support::initialize_workspace(&options.output_path).map_err(index)?;
    let reuse = ReferenceReuseReader::new(
        &workspace,
        atlas_index::test_support::TEST_WORKSPACE_SCHEMA_VERSION,
    )
    .map_err(index)?;
    // Preparation-only fixtures can survive a later publication to the same
    // workspace. Their directory is not an abandoned production attempt.
    let scratch =
        tempfile::tempdir().map_err(|e| IngestError::ArtifactWriteFailed(e.to_string()))?;
    let mut prepared = prepare(options, &reuse, scratch.path())?;
    prepared._scratch = Some(scratch);
    Ok(prepared)
}

#[cfg(test)]
mod tests;
