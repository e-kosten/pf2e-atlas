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
    BuildFrameStream, IndexBuildPack, PreparedRecordFrame, PublicationSpool, ReferenceReuseReader,
    SourceArtifactDiagnostic,
};
use atlas_record::source_record::{
    SourceBackedRecord, SourceContentOutcome, SourceContentStatus, SourceNodeView,
    SourceReferenceContribution, SourceReferenceIndex, SourceRelationshipOccurrence,
    prepare_record_content, resolve_source_relationships, select_record_search,
    source_owned_document_count, source_record_key,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
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
    snapshot: Vec<u8>,
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
/// Observed work/byte counters, separate from the API's structural record window.
#[derive(Debug, Default)]
pub(crate) struct PreparationWork {
    pub selection_calls: usize,
    pub reusable_record_read_calls: usize,
    pub largest_source_bytes: usize,
    pub largest_admitted_frame_bytes: u64,
    pub largest_prepared_frame_bytes: u64,
    pub largest_encoded_prepared_frame_bytes: u64,
}
pub(crate) struct PreparedSourceBuild {
    pub spool: PublicationSpool,
    pub native_inputs: atlas_index::SealedBuildFrameStream<PreparedNativeInput>,
    pub occurrences: atlas_index::SealedBuildFrameStream<RawOccurrence>,
    pub source_report: SourceEnrichmentReport,
    pub fingerprint: SourceFingerprint,
    pub catalog: LocalizationCatalog,
    pub context_shortened: usize,
    pub reused_sections: usize,
    pub document_units: usize,
    pub work: PreparationWork,
}

impl PreparedSourceBuild {
    pub(crate) fn check_execution_streams(&self) -> Result<(), IngestError> {
        let binding = (
            &self.spool.context,
            self.spool.expected.workspace_key(),
            self.spool.expected.reference_revision(),
            self.native_inputs.len(),
            self.occurrences.len(),
        );
        self.native_inputs.check_binding(&binding).map_err(index)?;
        self.occurrences.check_binding(&binding).map_err(index)
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
) -> Result<PreparedSourceBuild, IngestError> {
    crate::validate_source_compatibility(&options.source_root, options.manifest_path.as_deref())?;
    let capture = SourceCapture::capture(&options.source_root, options.manifest_path.as_deref())?;
    capture.require_complete_packs()?;
    crate::source_compatibility::validate_manifest(
        &capture.manifest_path,
        &capture.read(&capture.manifest_path)?,
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
    let manifest =
        crate::source::discovery::parse_manifest_bytes(&capture.read(&capture.manifest_path)?)?;
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
    let mut admitted = BuildFrameStream::<AdmittedFrame>::new().map_err(index)?;
    let mut envelopes = Vec::new();
    let mut id_counts = BTreeMap::new();
    let mut excluded = Vec::new();
    let mut report = SourceEnrichmentReport::default();
    let mut work = PreparationWork::default();
    for pack in &packs {
        let directory = &capture.pack(&pack.declared_path).0;
        for path in capture.paths(directory).into_iter().filter(|p| {
            p.extension().is_some_and(|s| s == "json")
                && p.file_name().is_none_or(|s| s != "_folders.json")
        }) {
            report.discovered_files += 1;
            let path_relative =
                crate::source::discovery::relative_source_path(&options.source_root, &path);
            let bytes = capture.read(&path)?;
            work.largest_source_bytes = work.largest_source_bytes.max(bytes.len());
            let hash = format!("{:x}", Sha256::digest(&bytes));
            let previous = match identities.get(&(pack.pack_id.clone(), path_relative.clone())) {
                Some(old)
                    if old.content_hash == hash && old.document_kind == pack.document_kind =>
                {
                    work.reusable_record_read_calls += 1;
                    reuse.record(&old.key).map_err(index)?
                }
                _ => None,
            };
            let mut reused = false;
            let mut previous_preparation = None;
            let admission = if let Some(old) = previous.filter(|old| {
                old.diagnostics
                    .iter()
                    .all(|d| d.source_diagnostic.is_some())
            }) {
                let raw = atlas_foundry_model::parse_source_value(&bytes);
                match raw {
                    Ok(raw) => {
                        reused = true;
                        previous_preparation = Some(PreviousPreparation {
                            hash: old.preparation_context_hash,
                            content: old.content,
                            relationships: old.relationships,
                        });
                        Ok(SourceAdmission {
                            raw,
                            model: Some(old.record.into_source()),
                            diagnostics: old
                                .diagnostics
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
                    &bytes,
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
                        admitted
                            .push(&AdmittedFrame {
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
                    }
                    Err(error) => observation.reason = Some(format!("{error:?}")),
                }
            } else {
                report.raw_only_documents += 1;
                observation.reason = Some("UnsupportedRoot".into());
            }
            envelopes.push(observation);
        }
    }
    capture.verify_reads()?;
    let mut resolver = SourceReferenceIndex::default();
    let mut active = BTreeSet::new();
    let mut partial_records = 0usize;
    for mut observation in envelopes {
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
    let mut spool = PublicationSpool::new(context, packs, inventory, reuse.expected().clone())
        .map_err(index)?;
    let mut prepared = BuildFrameStream::<PreparedRecordFrame>::new().map_err(index)?;
    let mut names = BTreeMap::new();
    let mut families = BTreeMap::new();
    let mut journal_ordinals = BTreeSet::new();
    admitted
        .visit(&mut |frame| {
            if !active.contains(&frame.key) {
                return Ok(());
            }
            let source = atlas_foundry_model::decode_snapshot(&frame.snapshot)?;
            let record = SourceBackedRecord::new(&frame.pack, source).map_err(|_| {
                atlas_index::IndexError::Invalid("admitted identity differs".into())
            })?;
            let relationships = resolve_source_relationships(&record, Some(&resolver));
            let content = match frame.previous {
                Some(previous)
                    if previous.hash == preparation_hash
                        && previous.relationships == relationships
                        && crate::source_enrichment::references_unchanged(
                            &previous.content,
                            &resolver,
                        ) =>
                {
                    report.reused_preparations += 1;
                    previous.content
                }
                _ => prepare_record_content(
                    &record,
                    default_audience(),
                    Some(&catalog),
                    Some(&resolver),
                ),
            };
            count_prepared(&mut report, &record, &content, relationships.len());
            work.selection_calls += 1;
            let selection = select_record_search(
                &record,
                &content,
                default_audience(),
                &frame.label,
                &spool.context.used_trait_labels,
            )
            .map_err(|e| atlas_index::IndexError::Invalid(e.to_string()))?;
            let node = SourceNodeView::from(record.source());
            if node.document_kind() != "Macro"
                && let Some(name) = node.name().value()
            {
                names.insert(frame.key.clone(), name.clone());
            }
            families.insert(
                frame.key.clone(),
                node.source_type().value().map(str::to_owned),
            );
            if node.document_kind() == "JournalEntry"
                && node
                    .name()
                    .value()
                    .is_some_and(|name| name == "Remaster Changes")
            {
                journal_ordinals.insert(prepared.len());
            }
            prepared.push(&PreparedRecordFrame {
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
            })?;
            Ok(())
        })
        .map_err(index)?;
    work.largest_admitted_frame_bytes = admitted.max_decoded_frame_bytes();
    let usage = admitted
        .file_usage()
        .map_err(index)?
        .combine(prepared.file_usage().map_err(index)?)
        .map_err(index)?
        .combine(spool.file_usage().map_err(index)?)
        .map_err(index)?;
    tracing::info!(target:"atlas_build_diagnostic",stage="admitted_prepared_overlap",file_bytes=usage.file_bytes,allocated_bytes=usage.allocated_bytes.unwrap_or(0),allocated_bytes_known=usage.allocated_bytes.is_some(),"Observed live metadata spool files");
    drop(admitted);
    let (aliases, pairs) = crate::source_aliases::extract_prepared_aliases(
        &names,
        &families,
        &mut |visitor| {
            prepared
                .visit_selected(&journal_ordinals, &mut |frame| {
                    visitor(&frame.selection.root, &frame.content)
                        .map_err(|e| atlas_index::IndexError::Invalid(e.to_string()))
                })
                .map_err(index)
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
    let mut native_inputs = BuildFrameStream::new().map_err(index)?;
    let mut occurrences = BuildFrameStream::new().map_err(index)?;
    let mut context_shortened = 0usize;
    let mut reused_sections = 0usize;
    let mut document_units = 0usize;
    prepared
        .visit(&mut |mut frame| {
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
                let layouts = if options.reuse_passage_layouts {
                    reuse.layouts(&frame.selection.root)?
                } else {
                    Vec::new()
                };
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
            Ok(())
        })
        .map_err(index)?;
    let usage = prepared
        .file_usage()
        .map_err(index)?
        .combine(spool.file_usage().map_err(index)?)
        .map_err(index)?
        .combine(native_inputs.file_usage().map_err(index)?)
        .map_err(index)?
        .combine(occurrences.file_usage().map_err(index)?)
        .map_err(index)?;
    tracing::info!(target:"atlas_build_diagnostic",stage="prepared_publication_overlap",file_bytes=usage.file_bytes,allocated_bytes=usage.allocated_bytes.unwrap_or(0),allocated_bytes_known=usage.allocated_bytes.is_some(),"Observed live metadata spool files");
    drop(prepared);
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
    capture.verify_reads()?;
    work.largest_prepared_frame_bytes = spool.records.max_decoded_frame_bytes();
    work.largest_encoded_prepared_frame_bytes = spool.records.max_encoded_frame_bytes();
    tracing::info!(target:"atlas_build_diagnostic",phase="source_preparation",structural_preparation_record_window=1,selection_calls=work.selection_calls,reusable_record_read_calls=work.reusable_record_read_calls,largest_source_bytes=work.largest_source_bytes,largest_admitted_frame_bytes=work.largest_admitted_frame_bytes,largest_prepared_frame_bytes=work.largest_prepared_frame_bytes,largest_encoded_prepared_frame_bytes=work.largest_encoded_prepared_frame_bytes,record_frames=spool.records.len(),native_input_frames=native_inputs.len(),occurrence_frames=occurrences.len(),"Observed source preparation work and frame sizes");
    let binding = (
        &spool.context,
        spool.expected.workspace_key(),
        spool.expected.reference_revision(),
        native_inputs.len(),
        occurrences.len(),
    );
    let native_inputs = native_inputs.seal(&binding).map_err(index)?;
    let occurrences = occurrences.seal(&binding).map_err(index)?;
    Ok(PreparedSourceBuild {
        spool,
        native_inputs,
        occurrences,
        source_report: report,
        fingerprint,
        catalog,
        context_shortened,
        reused_sections,
        document_units,
        work,
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
    prepare(options, &reuse)
}

#[cfg(test)]
mod tests;
