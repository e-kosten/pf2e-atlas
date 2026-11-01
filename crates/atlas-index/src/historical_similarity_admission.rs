//! Released migration20 admission, frozen from main f76232c8.
//! Current context fields and input-owned numerical storage are not this contract.
use crate::codec::{checked_hash, sha256};
use crate::{IndexError, SourceArtifactBuildContext, SourceSearchProfileIdentity};
use rusqlite::{Connection, OptionalExtension};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};

// Keep the released decoder's defaults and unknown-field rejection. In particular,
// neither selected_sections_recipe nor admission existed in the published DTO.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Context {
    #[serde(default)]
    core_sections_recipe: String,
    #[serde(default)]
    trait_policy: String,
    format_version: u32,
    snapshot_version: u32,
    source_contract: String,
    source_revision: Option<String>,
    source_fingerprint: String,
    source_file_count: usize,
    indexing_locale: String,
    locale_catalog_sha256: String,
    english_catalog_sha256: String,
    used_trait_labels: BTreeMap<String, String>,
    localization_policy: String,
    audience: atlas_record::source_content::ContentAudience,
    content_interpretation_version: String,
    content_selection_version: String,
    asset_policy: String,
    relationship_policy: String,
    query_projection_version: String,
    query_catalog_version: u32,
    lookup_version: String,
    lexical_selection_version: String,
    fts_tokenizer: String,
    search_profiles: Vec<SourceSearchProfileIdentity>,
}
impl Context {
    // Private plumbing for the unchanged historical record decoder/projector.
    // New fields are absent; preparation_sha256 excludes both and is main-identical.
    fn into_record_context(self) -> SourceArtifactBuildContext {
        SourceArtifactBuildContext {
            selected_sections_recipe: String::new(),
            admission: None,
            core_sections_recipe: self.core_sections_recipe,
            trait_policy: self.trait_policy,
            format_version: self.format_version,
            snapshot_version: self.snapshot_version,
            source_contract: self.source_contract,
            source_revision: self.source_revision,
            source_fingerprint: self.source_fingerprint,
            source_file_count: self.source_file_count,
            indexing_locale: self.indexing_locale,
            locale_catalog_sha256: self.locale_catalog_sha256,
            english_catalog_sha256: self.english_catalog_sha256,
            used_trait_labels: self.used_trait_labels,
            localization_policy: self.localization_policy,
            audience: self.audience,
            content_interpretation_version: self.content_interpretation_version,
            content_selection_version: self.content_selection_version,
            asset_policy: self.asset_policy,
            relationship_policy: self.relationship_policy,
            query_projection_version: self.query_projection_version,
            query_catalog_version: self.query_catalog_version,
            lookup_version: self.lookup_version,
            lexical_selection_version: self.lexical_selection_version,
            fts_tokenizer: self.fts_tokenizer,
            search_profiles: self.search_profiles,
        }
    }
}

fn read_context(c: &Connection) -> Result<Option<SourceArtifactBuildContext>, IndexError> {
    let stored: Option<(u32, String, String)> = c.query_row(
        "SELECT format_version,context_hash,context_json FROM artifact_context WHERE singleton=1",
        [], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    ).optional()?;
    let Some((format, hash, json)) = stored else {
        return Ok(None);
    };
    if !checked_hash(&hash) || sha256(json.as_bytes()) != hash {
        return Err(IndexError::Invalid(
            "stored context checksum mismatch".into(),
        ));
    }
    let original: Context = serde_json::from_str(&json)?;
    if format != crate::ARTIFACT_FORMAT_VERSION || original.format_version != format {
        return Err(IndexError::Unsupported(
            "stored reference format is unsupported".into(),
        ));
    }
    Ok(Some(original.into_record_context()))
}

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

/// Only the immutable schema20 callback consumes this admission result.
pub(super) fn for_backfill(
    c: &Connection,
) -> Result<Option<SourceArtifactBuildContext>, IndexError> {
    let Some(original) = read_context(c)? else {
        return Ok(None);
    };
    if original.snapshot_version != atlas_foundry_model::SNAPSHOT_VERSION
        || original.source_contract != atlas_foundry_model::SOURCE_CONTRACT_ID
        || original.content_selection_version
            != atlas_record::source_record::SOURCE_CONTENT_SELECTION_VERSION
        || original.content_interpretation_version
            != atlas_record::source_content::CONTENT_INTERPRETATION_VERSION
    {
        return Ok(None);
    }
    // Preserve main's classifier order, including accelerators deferring open
    // validation. Do not insert current section policy or numerical-layout checks.
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
    let lexical_changed = older_policy(
        &target.lexical_selection_version,
        crate::input::LEXICAL_SELECTION_VERSION,
    )?;
    target.lexical_selection_version = crate::input::LEXICAL_SELECTION_VERSION.into();
    let core_changed = target.core_sections_recipe.is_empty()
        || older_policy(
            &target.core_sections_recipe,
            atlas_domain::CORE_SECTIONS_RECIPE,
        )?;
    target.core_sections_recipe = atlas_domain::CORE_SECTIONS_RECIPE.into();
    let mut source_changed = match target.trait_policy.as_str() {
        "" => true,
        "trait-definition-centroid/v1" => false,
        other => {
            return Err(IndexError::Unsupported(format!(
                "unsupported trait policy {other}"
            )));
        }
    };
    target.trait_policy = "trait-definition-centroid/v1".into();
    for (stored, current) in [
        (
            &mut target.lookup_version,
            crate::input::SOURCE_LOOKUP_VERSION,
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
    validate_context(&target)?;
    if source_changed {
        return Ok(None);
    }
    if !lexical_changed && !query_changed && !core_changed {
        validate_open(c, &original)?;
    }
    Ok(Some(original))
}

// This is main's context policy, excluding later PR-only section/admission fields.
fn validate_context(c: &SourceArtifactBuildContext) -> Result<(), IndexError> {
    use crate::input::*;
    if c.core_sections_recipe != atlas_domain::CORE_SECTIONS_RECIPE {
        return Err(IndexError::Unsupported("Core section projection requires an offline metadata update; run atlas setup --offline".into()));
    }
    if c.trait_policy != "trait-definition-centroid/v1" {
        return Err(IndexError::Unsupported("trait preparation policy".into()));
    }
    if c.format_version != ARTIFACT_FORMAT_VERSION
        || c.snapshot_version != atlas_foundry_model::SNAPSHOT_VERSION
        || c.source_contract != atlas_foundry_model::SOURCE_CONTRACT_ID
    {
        return Err(IndexError::Unsupported(
            "source/artifact contract identity".into(),
        ));
    }
    if c.query_projection_version != SOURCE_PROJECTION_VERSION
        || c.query_catalog_version != crate::query::query_capabilities()?.version
        || c.lookup_version != SOURCE_LOOKUP_VERSION
        || c.lexical_selection_version != LEXICAL_SELECTION_VERSION
        || c.fts_tokenizer != "unicode61 remove_diacritics 2"
        || c.content_interpretation_version
            != atlas_record::source_content::CONTENT_INTERPRETATION_VERSION
        || c.content_selection_version
            != atlas_record::source_record::SOURCE_CONTENT_SELECTION_VERSION
        || c.localization_policy != LOCALIZATION_POLICY_VERSION
        || c.asset_policy != ASSET_POLICY_VERSION
        || c.relationship_policy != RELATIONSHIP_POLICY_VERSION
    {
        return Err(IndexError::Unsupported(
            "projection/lookup/content/FTS policy identity".into(),
        ));
    }
    for hash in [
        &c.source_fingerprint,
        &c.locale_catalog_sha256,
        &c.english_catalog_sha256,
    ] {
        if !checked_hash(hash) {
            return Err(IndexError::Invalid(
                "build context digest must be lowercase SHA-256".into(),
            ));
        }
    }
    if c.indexing_locale.is_empty()
        || c.used_trait_labels
            .iter()
            .any(|(id, label)| id.trim().is_empty() || label.trim().is_empty())
        || c.content_selection_version.is_empty()
        || c.localization_policy.is_empty()
        || c.asset_policy.is_empty()
        || c.relationship_policy.is_empty()
    {
        return Err(IndexError::Invalid(
            "build context policy or locale is empty".into(),
        ));
    }
    for profile in c.canonical_profiles()? {
        let spec = atlas_embedding::search_profile(&profile.profile_id)
            .map_err(|error| IndexError::Unsupported(error.to_string()))?;
        if profile.model_id != spec.model_spec().model_id
            || profile.dimensions != spec.model_spec().dimensions
            || profile.token_limit == 0
            || profile.token_limit > 32768
            || profile.assets.is_empty()
            || profile
                .assets
                .iter()
                .any(|(path, hash)| path.is_empty() || !checked_hash(hash))
            || !checked_hash(&profile.model_sha256)
            || !checked_hash(&profile.tokenizer_sha256)
            || !checked_hash(&profile.segmentation_tokenizer_sha256)
            || [
                &profile.provider,
                &profile.revision,
                &profile.pooling,
                &profile.normalization,
                &profile.distance_metric,
                &profile.unit_policy_version,
                &profile.tokenizer_id,
                &profile.output_name,
                &profile.dtype,
                &profile.execution_policy,
                &profile.input_recipe,
            ]
            .iter()
            .any(|value| value.is_empty())
        {
            return Err(IndexError::Unsupported(
                "invalid stored search profile execution or recipe identity".into(),
            ));
        }
        let expected_assets = spec
            .model_spec()
            .required_assets
            .iter()
            .map(|(path, _)| *path)
            .collect::<BTreeSet<_>>();
        if profile
            .assets
            .keys()
            .map(String::as_str)
            .collect::<BTreeSet<_>>()
            != expected_assets
            || profile.assets.get("onnx/model.onnx") != Some(&profile.model_sha256)
            || profile.assets.get("tokenizer.json") != Some(&profile.tokenizer_sha256)
            || profile.dtype != "f32"
            || profile.normalization != "l2"
            || profile.distance_metric != "cosine"
        {
            return Err(IndexError::Invalid(
                "incomplete or inconsistent stored profile asset/vector contract".into(),
            ));
        }
    }
    Ok(())
}

fn validate_open(c: &Connection, context: &SourceArtifactBuildContext) -> Result<(), IndexError> {
    if context.search_profiles != context.canonical_profiles()? {
        return Err(IndexError::Invalid(
            "profile manifest must be canonical and duplicate-free".into(),
        ));
    }
    validate_context(context)?;
    let actual = c
        .prepare("SELECT field,definition_json FROM query_field_catalog ORDER BY field")?
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?
        .collect::<Result<BTreeMap<_, _>, _>>()?;
    // Catalog definitions are byte-identical to f76232c8's version8 owner.
    // Query compiler optimizations do not change these serialized definitions.
    let wanted = crate::query::query_capabilities()?
        .fields
        .into_iter()
        .map(|definition| Ok((definition.id.clone(), serde_json::to_string(&definition)?)))
        .collect::<Result<BTreeMap<_, _>, IndexError>>()?;
    if actual != wanted {
        return Err(IndexError::Unsupported(
            "filter catalog differs from executable bindings".into(),
        ));
    }
    let mut actual_profiles = Vec::new();
    let mut statement = c.prepare(
        "SELECT profile_id,identity_json,dimensions FROM search_profiles ORDER BY profile_id",
    )?;
    let mut rows = statement.query([])?;
    while let Some(row) = rows.next()? {
        let identity: SourceSearchProfileIdentity =
            serde_json::from_str(&row.get::<_, String>(1)?)?;
        if identity.profile_id.as_str() != row.get::<_, String>(0)?
            || identity.dimensions != row.get::<_, usize>(2)?
            || identity.dimensions == 0
        {
            return Err(IndexError::Invalid(
                "profile metadata identity mismatch".into(),
            ));
        }
        actual_profiles.push(identity);
    }
    if actual_profiles != context.canonical_profiles()? {
        return Err(IndexError::Invalid(
            "stored search profiles differ from complete manifest".into(),
        ));
    }
    let expected = context
        .search_profiles
        .iter()
        .map(crate::schema::vector_table)
        .collect::<Result<BTreeSet<_>, _>>()?;
    let actual = c.prepare("SELECT name FROM sqlite_master WHERE type='table' AND sql LIKE 'CREATE VIRTUAL TABLE%USING vec0%' ORDER BY name")?
        .query_map([], |row| row.get::<_, String>(0))?.collect::<Result<BTreeSet<_>, _>>()?;
    if actual != expected {
        return Err(IndexError::Invalid(
            "profile vector table set differs from manifest".into(),
        ));
    }
    if expected.is_empty()
        && c.query_row("SELECT count(*) FROM semantic_units", [], |row| {
            row.get::<_, usize>(0)
        })? != 0
    {
        return Err(IndexError::Invalid(
            "semantic storage exists without profiles".into(),
        ));
    }
    for model in &context.search_profiles {
        let table = crate::schema::vector_table(model)?;
        let sql: String = c.query_row(
            "SELECT sql FROM sqlite_master WHERE name=?",
            [&table],
            |row| row.get(0),
        )?;
        if sql != crate::schema::vector_ddl(model)? {
            return Err(IndexError::Invalid(
                "profile vector table dimensions or distance policy differs".into(),
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn raw_checksum_is_authenticated_before_decoding_unknown_fields() {
        let c = rusqlite::Connection::open_in_memory().unwrap();
        c.execute_batch("CREATE TABLE artifact_context(singleton INTEGER PRIMARY KEY,format_version INTEGER,context_hash TEXT,context_json TEXT);").unwrap();
        c.execute(
            "INSERT INTO artifact_context VALUES(1,3,printf('%064d',0),?)",
            [r#"{"selected_sections_recipe":"unknown/v999"}"#],
        )
        .unwrap();
        let error = super::for_backfill(&c).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("stored context checksum mismatch"),
            "{error}"
        );
    }

    #[test]
    fn released_catalog_output_remains_main_era() {
        assert_eq!(crate::query::QUERY_CAPABILITY_VERSION, 8);
        let fields = crate::query::query_capabilities()
            .unwrap()
            .fields
            .into_iter()
            .map(|definition| {
                (
                    definition.id.clone(),
                    serde_json::to_string(&definition).unwrap(),
                )
            })
            .collect::<std::collections::BTreeMap<_, _>>();
        let actual = crate::codec::sha256(&serde_json::to_vec(&fields).unwrap());
        // Operator derives once from the verified main-identical catalog owner,
        // records that derivation, then replaces this literal before acceptance.
        assert_eq!(
            actual,
            "ca89d908bb5f3dc846b4318c17c9b2ce86e13826878b311bc0f6ec85712473f6"
        );
    }
}
