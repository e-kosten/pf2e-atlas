use super::*;
use crate::source_coverage::consumable_receipt_view as view;

const READER: &str = "source::dto::ConsumableSource::exact_leaf";

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ConsumableArtifactReceiptOperations {
    pub sqlite_builds: usize,
    pub requested_reads: usize,
    pub source_baselines: usize,
    pub compatible_mutations: usize,
    pub provenance_mutations: usize,
    pub discriminator_rejections: usize,
}

struct Pipeline {
    loaded: crate::records::LoadedSourceRecord,
    input: atlas_index::IndexBuildInput,
    hydrated: atlas_record::RetrievedRecord,
}

pub(super) fn registered(identity: &SourceLeafIdentity, reader: Option<&str>) -> bool {
    reader == Some(READER)
        && matches!(
            identity.type_id.as_str(),
            "item--consumable--top-level--root--root--root"
                | "item--consumable--embedded--actor--npc--actor-items"
                | "item--consumable--embedded--actor--character--actor-items"
                | "item--consumable--embedded--actor--hazard--actor-items"
        )
}

/// Share baseline and compatible mutation artifacts by exact source file. Each
/// leaf still reads its own typed destinations and rejects an unchanged value.
pub fn capture_registered_consumable_source_leaf_receipts(
    ledger: &SourceLeafCoverageLedger,
    repository: &Path,
) -> Result<(Vec<SourceLeafReceipt>, ConsumableArtifactReceiptOperations), CoverageContractError> {
    if let Some(failure) = lint_source_leaf_ledger(ledger).into_iter().next() {
        return Err(error(failure.code, failure.message));
    }
    let mut groups = BTreeMap::<&str, Vec<(usize, usize)>>::new();
    for (leaf_index, leaf) in ledger.leaves.iter().enumerate() {
        if !registered(&ledger.identity_for(leaf), leaf.reader.reader_id.as_deref()) {
            return Err(error(
                CoverageFailureCode::ReaderNotObserved,
                "unregistered H5 accessor",
            ));
        }
        for (fixture_index, fixture) in leaf.fixtures.iter().enumerate() {
            groups
                .entry(&fixture.source_path)
                .or_default()
                .push((leaf_index, fixture_index));
        }
    }
    let mut operations = ConsumableArtifactReceiptOperations::default();
    let mut receipts = Vec::new();
    for entries in groups.values() {
        let (leaf_index, fixture_index) = entries[0];
        let fixture = ResolvedFixture::load(
            &ledger.leaves[leaf_index].fixtures[fixture_index],
            repository,
            &ledger.selector,
        )?;
        let baseline = pipeline(&fixture, &fixture.raw, &mut operations)?;
        operations.source_baselines += 1;
        let mut mutated = fixture.raw.clone();
        let mut provenance_raw = fixture.raw.clone();
        let mut has_provenance = false;
        let mut mutated_paths = BTreeMap::<Option<u32>, Vec<Vec<SelectedPathStep>>>::new();
        for &(leaf_index, fixture_index) in entries {
            let leaf = &ledger.leaves[leaf_index];
            if leaf.normalized_path == "$.type" {
                continue;
            }
            let contract = &leaf.fixtures[fixture_index];
            let ordinal = item_ordinal(&fixture.raw, &contract.record_key)?;
            let provenance =
                leaf.disposition == crate::source_coverage::SourceLeafDisposition::ProvenanceOnly;
            has_provenance |= provenance;
            let root = item_root_mut(
                if provenance {
                    &mut provenance_raw
                } else {
                    &mut mutated
                },
                ordinal,
            )?;
            let selected = select_pattern_leaf_matching(
                item_root(&fixture.raw, ordinal)?,
                &leaf.normalized_path,
                |value| {
                    digest_serializable(value).is_ok_and(|digest| digest == contract.excerpt_digest)
                },
            )?;
            mutate_h5_leaf(
                root,
                &selected,
                &leaf.normalized_path,
                mutated_paths.entry(ordinal).or_default(),
            )?;
        }
        let mutation = pipeline(&fixture, &mutated, &mut operations)?;
        operations.compatible_mutations += 1;
        let provenance_mutation = if has_provenance {
            let result = pipeline(&fixture, &provenance_raw, &mut operations)?;
            operations.provenance_mutations += 1;
            Some(result)
        } else {
            None
        };
        for &(leaf_index, fixture_index) in entries {
            let leaf = &ledger.leaves[leaf_index];
            let fixture =
                ResolvedFixture::load(&leaf.fixtures[fixture_index], repository, &ledger.selector)?;
            let ordinal = item_ordinal(&fixture.raw, &fixture.reference.record_key)?;
            let selected = select_pattern_leaf_matching(
                item_root(&fixture.raw, ordinal)?,
                &leaf.normalized_path,
                |value| {
                    digest_serializable(value)
                        .is_ok_and(|digest| digest == fixture.reference.excerpt_digest)
                },
            )?;
            let excerpt = match &selected.source {
                SourcePresence::Value(value) => value.value.clone().unwrap_or(Value::Null),
                SourcePresence::Null => Value::Null,
                SourcePresence::Missing => serde_json::json!({"state":"missing"}),
            };
            validate_hazard_excerpt(&fixture, &excerpt)?;
            if leaf.normalized_path == "$.type" {
                let mut rejected_raw = fixture.raw.clone();
                let root = item_root_mut(&mut rejected_raw, ordinal)?;
                mutate_selected_pattern_leaf(root, &selected, &leaf.normalized_path)?;
                // Exercise admission with a novel discriminator; no artifact is
                // built from a rejected source document.
                let source = parse_serialized_source_object(
                    &serde_json::to_vec(root)
                        .map_err(|e| error(CoverageFailureCode::ReaderNotObserved, e))?,
                )
                .map_err(|e| error(CoverageFailureCode::ReaderNotObserved, e))?;
                let parsed = crate::source::dto::parse_item_source_from_serialized(
                    pinned_source_version_metadata(),
                    SourceIdentity::new("receipt", "receipt"),
                    None,
                    root.clone(),
                    &source,
                );
                if parsed.is_ok() {
                    return Err(error(
                        CoverageFailureCode::ReaderNotObserved,
                        "H5 type mutation was admitted",
                    ));
                }
                operations.discriminator_rejections += 1;
                receipts.push(capture_leaf(
                    ledger, leaf_index, fixture, ordinal, &baseline, None,
                )?);
            } else {
                receipts.push(capture_leaf(
                    ledger,
                    leaf_index,
                    fixture,
                    ordinal,
                    &baseline,
                    if leaf.disposition
                        == crate::source_coverage::SourceLeafDisposition::ProvenanceOnly
                    {
                        provenance_mutation.as_ref()
                    } else {
                        Some(&mutation)
                    },
                )?);
            }
        }
    }
    Ok((receipts, operations))
}

fn item_ordinal(raw: &Value, key: &str) -> Result<Option<u32>, CoverageContractError> {
    let Some((_, id)) = key.rsplit_once("#item:") else {
        return Ok(None);
    };
    raw.get("items")
        .and_then(Value::as_array)
        .and_then(|items| {
            items
                .iter()
                .position(|item| item.get("_id").and_then(Value::as_str) == Some(id))
        })
        .map(|ordinal| Some(ordinal as u32))
        .ok_or_else(|| {
            error(
                CoverageFailureCode::FixtureNotSourceGrounded,
                "missing exact H5 fixture item",
            )
        })
}

fn item_root(raw: &Value, ordinal: Option<u32>) -> Result<&Value, CoverageContractError> {
    match ordinal {
        None => Ok(raw),
        Some(ordinal) => raw
            .get("items")
            .and_then(|items| items.get(ordinal as usize))
            .ok_or_else(|| {
                error(
                    CoverageFailureCode::ReaderNotObserved,
                    "missing H5 item ordinal",
                )
            }),
    }
}
fn item_root_mut(
    raw: &mut Value,
    ordinal: Option<u32>,
) -> Result<&mut Value, CoverageContractError> {
    match ordinal {
        None => Ok(raw),
        Some(ordinal) => raw
            .get_mut("items")
            .and_then(|items| items.get_mut(ordinal as usize))
            .ok_or_else(|| {
                error(
                    CoverageFailureCode::ReaderNotObserved,
                    "missing H5 item ordinal",
                )
            }),
    }
}

fn pipeline(
    fixture: &ResolvedFixture,
    raw: &Value,
    operations: &mut ConsumableArtifactReceiptOperations,
) -> Result<Pipeline, CoverageContractError> {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| error(CoverageFailureCode::ReaderNotObserved, e))?
        .as_nanos();
    let root =
        std::env::temp_dir().join(format!("atlas-h5-receipt-{}-{nonce}", std::process::id()));
    std::fs::create_dir_all(root.join("packs/fixture"))
        .map_err(|e| error(CoverageFailureCode::ReaderNotObserved, e))?;
    let result =
        (|| {
            let pack = fixture
                .reference
                .record_key
                .split_once(':')
                .map(|(pack, _)| pack)
                .ok_or_else(|| {
                    error(
                        CoverageFailureCode::FixtureNotSourceGrounded,
                        "invalid fixture key",
                    )
                })?;
            let class = if fixture.reference.record_key.contains("#item:") {
                "Actor"
            } else {
                "Item"
            };
            std::fs::write(root.join("module.json"), serde_json::to_vec(&serde_json::json!({
            "packs": [{"name":pack,"label":"H5 receipt","type":class,"path":"packs/fixture"}]
        })).map_err(|e| error(CoverageFailureCode::ReaderNotObserved, e))?)
            .map_err(|e| error(CoverageFailureCode::ReaderNotObserved, e))?;
            std::fs::write(
                root.join("packs/fixture/record.json"),
                serde_json::to_vec(raw)
                    .map_err(|e| error(CoverageFailureCode::ReaderNotObserved, e))?,
            )
            .map_err(|e| error(CoverageFailureCode::ReaderNotObserved, e))?;
            let source = crate::source_pipeline::load_foundry_source(&root, None)
                .map_err(|e| error(CoverageFailureCode::ReaderNotObserved, e))?;
            let expected_key = format!(
                "{pack}:{}",
                raw.get("_id").and_then(Value::as_str).ok_or_else(|| error(
                    CoverageFailureCode::ReaderNotObserved,
                    "missing source ID"
                ))?
            );
            let loaded = source
                .records
                .iter()
                .find(|record| record.record.identity.key.to_string() == expected_key)
                .cloned()
                .ok_or_else(|| {
                    error(
                        CoverageFailureCode::ReaderNotObserved,
                        "H5 fixture did not load",
                    )
                })?;
            let key = loaded.record.identity.key.clone();
            let input = index_build_input(source);
            let hydrated = persist_and_hydrate_spell(&input, &key)?;
            operations.sqlite_builds += 1;
            operations.requested_reads += 1;
            Ok(Pipeline {
                loaded,
                input,
                hydrated,
            })
        })();
    let _ = std::fs::remove_dir_all(&root);
    result
}

fn constructed(
    pipeline: &Pipeline,
    post: bool,
) -> Result<atlas_record::RetrievedRecord, CoverageContractError> {
    Ok(if post {
        atlas_record::RetrievedRecord {
            record: pipeline
                .input
                .records
                .iter()
                .find(|record| record.identity.key == pipeline.loaded.record.identity.key)
                .ok_or_else(|| {
                    error(
                        FinalOwnerStage::PostProjection.mismatch_code(),
                        "build input lost exact record",
                    )
                })?
                .clone(),
            body: pipeline
                .input
                .canonical_bodies
                .iter()
                .find(|body| body.record_key() == &pipeline.loaded.record.identity.key)
                .cloned(),
            spell_children: pipeline.input.canonical_spell_children.clone(),
            consumable_occurrences: pipeline
                .input
                .consumable_occurrence_sets
                .iter()
                .find(|set| {
                    set.entities.first().is_some_and(|entity| {
                        entity.owner_record_key == pipeline.loaded.record.identity.key
                    })
                })
                .cloned()
                .unwrap_or_default(),
        }
    } else {
        atlas_record::RetrievedRecord {
            record: pipeline.loaded.record.clone(),
            body: pipeline.loaded.facts.canonical_body.clone(),
            spell_children: pipeline.loaded.facts.canonical_spell_children.clone(),
            consumable_occurrences: pipeline.loaded.facts.consumable_occurrences.clone(),
        }
    })
}

fn source_item(
    pipeline: &Pipeline,
    ordinal: Option<u32>,
) -> Result<&crate::source::dto::FullItemSource, CoverageContractError> {
    let source = if let Some(ordinal) = ordinal {
        pipeline
            .loaded
            .facts
            .consumable_occurrence_candidates
            .iter()
            .find(|candidate| candidate.authored_order == ordinal)
            .map(|candidate| &candidate.source)
    } else {
        pipeline.loaded.facts.item_source.as_ref()
    };
    source.map(|source| source.source.source()).ok_or_else(|| {
        error(
            CoverageFailureCode::ReaderNotObserved,
            "missing selected typed H5 DTO",
        )
    })
}

fn stage_leaf(
    pipeline: &Pipeline,
    ordinal: Option<u32>,
    path: &str,
    stage: FinalOwnerStage,
    selected: &SelectedPatternLeaf,
) -> Result<SourcePresence<SourceLeafValue>, CoverageContractError> {
    if matches!(
        path,
        "$.system.description.value" | "$.system.description.gm"
    ) {
        return content_leaf(pipeline, ordinal, path, stage);
    }
    let value = match stage {
        FinalOwnerStage::SourceDto => view::source_item(source_item(pipeline, ordinal)?)?,
        FinalOwnerStage::Canonical => {
            view::canonical_item(&constructed(pipeline, false)?, ordinal)?
        }
        FinalOwnerStage::PostProjection => {
            view::canonical_item(&constructed(pipeline, true)?, ordinal)?
        }
        FinalOwnerStage::ArtifactHydration | FinalOwnerStage::DurableProvenance => {
            view::canonical_item(&pipeline.hydrated, ordinal)?
        }
        FinalOwnerStage::PublicSurface => public_item(&pipeline.hydrated, ordinal)?,
    };
    if stage != FinalOwnerStage::SourceDto {
        // Compare complete typed collections as well as the selected scalar:
        // an unchanged first member cannot hide lost/reordered siblings.
        let source = view::source_item(source_item(pipeline, ordinal)?)?;
        if value.get("system") != source.get("system") {
            return Err(error(
                stage.mismatch_code(),
                "H5 typed system lost value, member order, or multiplicity",
            ));
        }
    }
    let mut selected = selected.clone();
    let mut selected_value = &value;
    for step in &selected.steps {
        selected_value = match step {
            SelectedPathStep::Key(key) => selected_value.get(key),
            SelectedPathStep::Index(index) => selected_value.get(*index),
        }
        .ok_or_else(|| error(stage.mismatch_code(), "typed H5 owner lost selected member"))?;
    }
    let metadata = match &selected.source {
        SourcePresence::Value(value) => (
            value.member_kind,
            value.member_identity.clone(),
            value.ordinal,
        ),
        _ => (None, None, None),
    };
    selected.source = if selected_value.is_null() {
        SourcePresence::Null
    } else {
        source_leaf_value(selected_value.clone(), metadata.0, metadata.1, metadata.2)?
    };
    if let SourcePresence::Value(leaf) = &mut selected.source {
        if path == "$.system.bulk.value" {
            // Source DTO and canonical exact decimals have a string carrier;
            // the declared source shape remains numeric.
            leaf.json_type = SourceJsonType::Number;
            leaf.stable_digest = Some(digest_serializable(&leaf.value)?);
            leaf.value = None;
        }
        if path.starts_with("$.system.rules[]") {
            let index = selected
                .steps
                .iter()
                .find_map(|step| match step {
                    SelectedPathStep::Index(index) => Some(*index),
                    _ => None,
                })
                .ok_or_else(|| error(stage.mismatch_code(), "missing rule ordinal"))?;
            let evidence = value
                .pointer("/system/__h5_rule_evidence")
                .and_then(Value::as_array)
                .and_then(|rules| rules.get(index))
                .ok_or_else(|| {
                    error(
                        stage.mismatch_code(),
                        "missing typed unsupported rule evidence",
                    )
                })?;
            leaf.unsupported = Some(TypedUnsupportedValue {
                value: evidence.clone(),
                reason: evidence["reason"]
                    .as_str()
                    .ok_or_else(|| error(stage.mismatch_code(), "missing rule reason"))?
                    .to_string(),
            });
        }
    }
    Ok(selected.source)
}

fn public_item(
    record: &atlas_record::RetrievedRecord,
    ordinal: Option<u32>,
) -> Result<Value, CoverageContractError> {
    let public = record_json(
        record,
        RecordJsonOptions {
            detail: DetailLevel::Full,
            include_source_json: false,
        },
    )
    .map_err(|e| error(CoverageFailureCode::PublicSurfaceMismatch, e))?;
    let mut root = serde_json::Map::new();
    let system = match public.presentation {
        RecordPresentationJson::Consumable { body, .. } if ordinal.is_none() => {
            let key = RecordKey::parse(&public.base.key)
                .map_err(|e| error(CoverageFailureCode::PublicSurfaceMismatch, e))?;
            root.insert(
                "_id".to_string(),
                Value::String(key.id().as_str().to_string()),
            );
            root.insert("name".to_string(), Value::String(public.base.name));
            view::public_system(&body.definition, &body.source_state)?
        }
        RecordPresentationJson::Creature { consumables, .. }
        | RecordPresentationJson::Hazard { consumables, .. } => {
            let occurrence = consumables
                .iter()
                .find(|occurrence| Some(occurrence.authored_order) == ordinal)
                .ok_or_else(|| {
                    error(
                        CoverageFailureCode::PublicSurfaceMismatch,
                        "missing public H5 occurrence",
                    )
                })?;
            let atlas_record::ConsumableOccurrenceTargetJson::ParentOwned { definition, .. } =
                &occurrence.target
            else {
                return Err(error(
                    CoverageFailureCode::PublicSurfaceMismatch,
                    "unexpected public resolved receipt",
                ));
            };
            let id = match &occurrence.source_id {
                atlas_record::ConsumableFactJson::Known(value) => Value::String(value.clone()),
                _ => {
                    return Err(error(
                        CoverageFailureCode::PublicSurfaceMismatch,
                        "missing public occurrence source identity",
                    ));
                }
            };
            root.insert("_id".to_string(), id);
            root.insert("name".to_string(), Value::String(occurrence.name.clone()));
            view::public_system(definition, &occurrence.source_state)?
        }
        _ => {
            return Err(error(
                CoverageFailureCode::PublicSurfaceMismatch,
                "no H5 public surface for selected context",
            ));
        }
    };
    root.insert("type".to_string(), Value::String("consumable".to_string()));
    root.insert("system".to_string(), system);
    Ok(Value::Object(root))
}

fn content_leaf(
    pipeline: &Pipeline,
    ordinal: Option<u32>,
    path: &str,
    stage: FinalOwnerStage,
) -> Result<SourcePresence<SourceLeafValue>, CoverageContractError> {
    let rich_stage = match stage {
        FinalOwnerStage::SourceDto => MappedSpellStage::Source,
        FinalOwnerStage::Canonical => MappedSpellStage::Canonical,
        FinalOwnerStage::PostProjection => MappedSpellStage::PostProjection,
        _ => MappedSpellStage::ArtifactHydration,
    };
    let gm = path.ends_with(".gm");
    let kind = match (ordinal, gm) {
        (None, false) => ContentSourceKind::Description,
        (None, true) => ContentSourceKind::GmNotes,
        (Some(_), false) => ContentSourceKind::EmbeddedItemDescription,
        (Some(_), true) => ContentSourceKind::EmbeddedGmDescription,
    };
    if stage == FinalOwnerStage::SourceDto {
        let document = pipeline
            .loaded
            .facts
            .source_facts
            .content_sources
            .iter()
            .find(|source| {
                source.source_kind == kind
                    && source.authored_ordinal_or_range.as_deref()
                        == ordinal.map(|value| value.to_string()).as_deref()
            })
            .ok_or_else(|| {
                error(
                    CoverageFailureCode::ReaderNotObserved,
                    "missing source-owned H5 content",
                )
            })?;
        return rich_document_leaf(&document.document, rich_stage);
    }
    let record = match stage {
        FinalOwnerStage::Canonical => constructed(pipeline, false)?,
        FinalOwnerStage::PostProjection => constructed(pipeline, true)?,
        _ => pipeline.hydrated.clone(),
    };
    let documents = if let Some(ordinal) = ordinal {
        &record
            .consumable_occurrences
            .occurrences
            .iter()
            .find(|occurrence| occurrence.authored_order == ordinal)
            .ok_or_else(|| error(stage.mismatch_code(), "missing content occurrence"))?
            .authored_content
            .documents
    } else {
        &record
            .body
            .as_ref()
            .and_then(RecordBody::as_consumable)
            .ok_or_else(|| error(stage.mismatch_code(), "missing content body"))?
            .content
            .documents
    };
    let document = documents
        .iter()
        .find(|document| document.source_kind == kind)
        .ok_or_else(|| error(stage.mismatch_code(), "missing exact content kind"))?;
    let text = atlas_record::render_plain_text(&document.document);
    if stage == FinalOwnerStage::PublicSurface {
        let public = record_json(
            &record,
            RecordJsonOptions {
                detail: DetailLevel::Full,
                include_source_json: false,
            },
        )
        .map_err(|e| error(stage.mismatch_code(), e))?;
        let content = match public.presentation {
            RecordPresentationJson::Consumable { body, .. } => body.content,
            RecordPresentationJson::Creature { consumables, .. }
            | RecordPresentationJson::Hazard { consumables, .. } => {
                consumables
                    .into_iter()
                    .find(|occurrence| Some(occurrence.authored_order) == ordinal)
                    .ok_or_else(|| {
                        error(stage.mismatch_code(), "missing public content occurrence")
                    })?
                    .content
            }
            _ => {
                return Err(error(
                    stage.mismatch_code(),
                    "missing public content family",
                ));
            }
        };
        if !content.iter().any(|content| {
            content.content_key == document.id.content_key.as_str()
                && content.content_hash == document.content_hash.as_str()
                && content.text == text
        }) {
            return Err(error(
                stage.mismatch_code(),
                "public H5 content identity/hash/text mismatch",
            ));
        }
    }
    rich_document_leaf(&document.document, rich_stage)
}

fn capture_leaf(
    ledger: &SourceLeafCoverageLedger,
    leaf_index: usize,
    fixture: ResolvedFixture,
    ordinal: Option<u32>,
    baseline: &Pipeline,
    mutation: Option<&Pipeline>,
) -> Result<SourceLeafReceipt, CoverageContractError> {
    let leaf = &ledger.leaves[leaf_index];
    let selected = select_pattern_leaf_matching(
        item_root(&fixture.raw, ordinal)?,
        &leaf.normalized_path,
        |value| {
            digest_serializable(value)
                .is_ok_and(|digest| digest == fixture.reference.excerpt_digest)
        },
    )?;
    let source = stage_leaf(
        baseline,
        ordinal,
        &leaf.normalized_path,
        FinalOwnerStage::SourceDto,
        &selected,
    )?;
    let source_mutation = mutation
        .map(|mutation| {
            stage_leaf(
                mutation,
                ordinal,
                &leaf.normalized_path,
                FinalOwnerStage::SourceDto,
                &selected,
            )
        })
        .transpose()?
        .unwrap_or(SourcePresence::Missing);
    let mut observations = Vec::new();
    for owner in &leaf.final_owners {
        let actual = stage_leaf(
            baseline,
            ordinal,
            &leaf.normalized_path,
            owner.stage,
            &selected,
        )?;
        let changed = mutation
            .map(|mutation| {
                stage_leaf(
                    mutation,
                    ordinal,
                    &leaf.normalized_path,
                    owner.stage,
                    &selected,
                )
            })
            .transpose()?
            .unwrap_or(SourcePresence::Missing);
        if actual != source || changed != source_mutation {
            return Err(error(
                owner.stage.mismatch_code(),
                format!("{} exact typed owner changed meaning", leaf.normalized_path),
            ));
        }
        observations.push(presence_stage(
            owner.stage,
            &owner.destination,
            "sealed::H5TypedLeaf",
            actual,
            &changed,
        )?);
    }
    let mut receipt = sealed_receipt(
        ledger.identity_for(leaf),
        fixture.reference,
        source,
        source_mutation,
        READER,
        "sealed::H5TypedLeaf",
        observations,
    )?;
    if leaf.disposition == crate::source_coverage::SourceLeafDisposition::ProvenanceOnly {
        let mutation = mutation.ok_or_else(|| {
            error(
                CoverageFailureCode::ProvenanceNotDurable,
                "missing provenance mutation",
            )
        })?;
        let before = atlas_record::build_search_fts_projection(
            &baseline.hydrated.record,
            &[],
            baseline.hydrated.body.as_ref(),
        );
        let after = atlas_record::build_search_fts_projection(
            &mutation.hydrated.record,
            &[],
            mutation.hydrated.body.as_ref(),
        );
        if before != after {
            return Err(error(
                CoverageFailureCode::ProvenanceNotDurable,
                "H5 provenance mutation changed FTS semantics",
            ));
        }
        receipt.reader.purpose = SourceAccessorPurpose::ProvenanceReader;
        receipt.semantic_output = Some(SemanticOutputObservation {
            accessor_binding: "build_search_fts_projection: isolated provenance mutation"
                .to_string(),
            observed: false,
            mutation_observed: true,
        });
        receipt.evidence_digest = evidence_digest(
            &receipt.identity,
            &receipt.fixture,
            &receipt.source,
            &receipt.reader,
            &receipt.observations,
            &receipt.semantic_output,
        )?;
    }
    Ok(receipt)
}

// Container-valued rule leaves and their descendant declarations can overlap.
// Mutate each concrete primitive once so overlapping booleans cannot cancel.
fn mutate_h5_leaf(
    root: &mut Value,
    selected: &SelectedPatternLeaf,
    normalized_path: &str,
    seen: &mut Vec<Vec<SelectedPathStep>>,
) -> Result<(), CoverageContractError> {
    let mut value = &*root;
    for step in &selected.steps {
        value = match step {
            SelectedPathStep::Key(key) => value.get(key),
            SelectedPathStep::Index(index) => value.get(*index),
        }
        .ok_or_else(|| {
            error(
                CoverageFailureCode::ReaderNotObserved,
                "H5 mutation path disappeared",
            )
        })?;
    }
    let children: Vec<_> = match value {
        Value::Object(values) => values.keys().cloned().map(SelectedPathStep::Key).collect(),
        Value::Array(values) => (0..values.len()).map(SelectedPathStep::Index).collect(),
        _ => Vec::new(),
    };
    if !children.is_empty() {
        for step in children {
            let mut child = selected.clone();
            child.steps.push(step);
            mutate_h5_leaf(root, &child, normalized_path, seen)?;
        }
        return Ok(());
    }
    if seen.contains(&selected.steps) {
        return Ok(());
    }
    mutate_selected_pattern_leaf(root, selected, normalized_path)?;
    seen.push(selected.steps.clone());
    Ok(())
}

pub(super) fn capture_one(
    identity: SourceLeafIdentity,
    fixture: ResolvedFixture,
    ledger: &SourceLeafCoverageLedger,
) -> Result<SourceLeafReceipt, CoverageContractError> {
    let leaf_index = ledger
        .leaves
        .iter()
        .position(|leaf| leaf.normalized_path == identity.normalized_path)
        .ok_or_else(|| error(CoverageFailureCode::ReaderNotObserved, "unknown H5 leaf"))?;
    let ordinal = item_ordinal(&fixture.raw, &fixture.reference.record_key)?;
    let selected = select_pattern_leaf_matching(
        item_root(&fixture.raw, ordinal)?,
        &identity.normalized_path,
        |value| {
            digest_serializable(value)
                .is_ok_and(|digest| digest == fixture.reference.excerpt_digest)
        },
    )?;
    let excerpt = match &selected.source {
        SourcePresence::Value(value) => value.value.clone().unwrap_or(Value::Null),
        SourcePresence::Null => Value::Null,
        SourcePresence::Missing => serde_json::json!({"state":"missing"}),
    };
    validate_hazard_excerpt(&fixture, &excerpt)?;
    let mut operations = ConsumableArtifactReceiptOperations::default();
    let baseline = pipeline(&fixture, &fixture.raw, &mut operations)?;
    let mut raw = fixture.raw.clone();
    mutate_h5_leaf(
        item_root_mut(&mut raw, ordinal)?,
        &selected,
        &identity.normalized_path,
        &mut Vec::new(),
    )?;
    if identity.normalized_path == "$.type" {
        if pipeline(&fixture, &raw, &mut operations).is_ok() {
            return Err(error(
                CoverageFailureCode::ReaderNotObserved,
                "H5 discriminator mutation admitted",
            ));
        }
        capture_leaf(ledger, leaf_index, fixture, ordinal, &baseline, None)
    } else {
        let mutation = pipeline(&fixture, &raw, &mut operations)?;
        capture_leaf(
            ledger,
            leaf_index,
            fixture,
            ordinal,
            &baseline,
            Some(&mutation),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overlapping_rule_container_and_leaf_mutations_do_not_cancel() {
        let mut raw =
            serde_json::json!({"system":{"rules":[{"value":{"enabled":false,"tags":["a","b"]}}]}});
        let original = raw.clone();
        let mut seen = Vec::new();
        for path in [
            "$.system.rules[].value",
            "$.system.rules[].value.enabled",
            "$.system.rules[].value.tags[]",
        ] {
            let selected = select_pattern_leaf(&raw, path).unwrap();
            mutate_h5_leaf(&mut raw, &selected, path, &mut seen).unwrap();
        }
        assert_eq!(
            raw.pointer("/system/rules/0/value/enabled"),
            Some(&Value::Bool(true))
        );
        assert_eq!(
            raw.pointer("/system/rules/0/value/tags")
                .unwrap()
                .as_array()
                .unwrap()
                .len(),
            2
        );
        assert_ne!(raw, original);
    }

    #[test]
    fn h5_receipt_rejects_missing_canonical_projected_and_hydrated_owners() {
        let repository = PathBuf::from(
            std::env::var_os("PF2E_SOURCE_REPOSITORY")
                .expect("TEST PREREQUISITE: accepted pinned PF2E repository"),
        );
        let ledger = parse_source_leaf_ledger(include_str!(
            "../../../../../contracts/source-leaf-coverage/v1/item-consumable-standalone.yaml"
        ))
        .unwrap();
        let fixture =
            ResolvedFixture::load(&ledger.leaves[0].fixtures[0], &repository, &ledger.selector)
                .unwrap();
        let mut actual = pipeline(
            &fixture,
            &fixture.raw,
            &mut ConsumableArtifactReceiptOperations::default(),
        )
        .unwrap();
        let selected = select_pattern_leaf(&fixture.raw, "$._id").unwrap();
        for stage in [
            FinalOwnerStage::Canonical,
            FinalOwnerStage::PostProjection,
            FinalOwnerStage::ArtifactHydration,
            FinalOwnerStage::PublicSurface,
        ] {
            stage_leaf(&actual, None, "$._id", stage, &selected).expect("baseline typed stage");
        }
        let canonical = actual.loaded.facts.canonical_body.take();
        assert!(
            stage_leaf(
                &actual,
                None,
                "$._id",
                FinalOwnerStage::Canonical,
                &selected
            )
            .is_err()
        );
        actual.loaded.facts.canonical_body = canonical;
        let projected_records = std::mem::take(&mut actual.input.records);
        assert!(
            stage_leaf(
                &actual,
                None,
                "$._id",
                FinalOwnerStage::PostProjection,
                &selected
            )
            .is_err()
        );
        actual.input.records = projected_records;
        let projected = std::mem::take(&mut actual.input.canonical_bodies);
        assert!(
            stage_leaf(
                &actual,
                None,
                "$._id",
                FinalOwnerStage::PostProjection,
                &selected
            )
            .is_err()
        );
        actual.input.canonical_bodies = projected;
        let hydrated = actual.hydrated.body.take();
        assert!(
            stage_leaf(
                &actual,
                None,
                "$._id",
                FinalOwnerStage::ArtifactHydration,
                &selected
            )
            .is_err()
        );
        assert!(
            stage_leaf(
                &actual,
                None,
                "$._id",
                FinalOwnerStage::PublicSurface,
                &selected
            )
            .is_err()
        );
        actual.hydrated.body = hydrated;
        let Some(RecordBody::Consumable(body)) = actual.hydrated.body.as_mut() else {
            panic!("consumable")
        };
        body.definition.traits =
            FactValue::Value(atlas_record::ConsumableSourceValue::Known(vec![
                "receipt-corruption".to_string(),
                "receipt-corruption".to_string(),
            ]));
        assert!(
            stage_leaf(
                &actual,
                None,
                "$._id",
                FinalOwnerStage::ArtifactHydration,
                &selected
            )
            .is_err(),
            "full collection comparison must catch corruption outside the selected scalar"
        );
    }
}
