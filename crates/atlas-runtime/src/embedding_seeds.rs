//! Release pins and acquisition policy for managed English setup.
use std::{
    io::{self, Read, Write},
    path::Path,
    time::Duration,
};

use atlas_index::{EmbeddingSeed, EmbeddingSeedFile, SourceSearchProfileIdentity};
use atlas_ingest::SeedPart;
use serde::{Deserialize, Serialize};

const RELEASE: &str = "https://github.com/e-kosten/pf2e-atlas/releases/download";

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EmbeddingSeedPin {
    pub tag: String,
    pub asset: String,
    pub file: EmbeddingSeedFile,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EmbeddingSeedDeltaPin {
    pub base_sha256: String,
    pub asset: EmbeddingSeedPin,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EmbeddingSeedProfilePins {
    pub base: EmbeddingSeedPin,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delta: Option<EmbeddingSeedDeltaPin>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EmbeddingSeedPins {
    pub version: u32,
    pub profiles: std::collections::BTreeMap<String, EmbeddingSeedProfilePins>,
}

pub fn parse_embedding_seed_pins(json: &str) -> Result<EmbeddingSeedPins, String> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Manifest {
        Current(EmbeddingSeedPins),
        Historical(std::collections::BTreeMap<String, EmbeddingSeedPin>),
    }

    let manifest =
        serde_json::from_str(json).map_err(|error| format!("invalid release pins: {error}"))?;
    let pins = match manifest {
        Manifest::Current(pins) if pins.version == 2 => pins,
        Manifest::Current(_) => return Err("unsupported release pin manifest version".into()),
        Manifest::Historical(profiles) => EmbeddingSeedPins {
            version: 2,
            profiles: profiles
                .into_iter()
                .map(|(profile, base)| (profile, EmbeddingSeedProfilePins { base, delta: None }))
                .collect(),
        },
    };
    for (profile, pins) in &pins.profiles {
        validate_pin(&pins.base)?;
        if let Some(delta) = &pins.delta {
            validate_pin(&delta.asset)?;
            validate_delta_binding(
                profile,
                &pins.base.file,
                &delta.base_sha256,
                &delta.asset.file,
            )?;
        }
    }
    Ok(pins)
}

fn validate_pin(pin: &EmbeddingSeedPin) -> Result<(), String> {
    pin.file.validate().map_err(|error| error.to_string())?;
    release_url(&pin.tag, &pin.asset)?;
    Ok(())
}

fn validate_delta_binding(
    profile: &str,
    base: &EmbeddingSeedFile,
    expected_base: &str,
    delta: &EmbeddingSeedFile,
) -> Result<(), String> {
    if base.sha256 != expected_base || base.sha256 == delta.sha256 {
        return Err(format!(
            "{profile}: embedding seed delta does not identify a distinct asset paired with its exact base"
        ));
    }
    Ok(())
}

pub fn published_embedding_seed_pins() -> Result<EmbeddingSeedPins, String> {
    parse_embedding_seed_pins(include_str!("../embedding-seeds.json"))
}

pub(crate) type SeedSource<'a> = dyn FnMut(
        &SourceSearchProfileIdentity,
        &Path,
        bool,
        SeedPart,
    ) -> Result<Option<EmbeddingSeed>, String>
    + 'a;

#[cfg(feature = "seed-tools")]
pub struct LocalEmbeddingSeed {
    pub profile: atlas_domain::SearchProfileId,
    pub path: std::path::PathBuf,
    pub file: EmbeddingSeedFile,
    pub base_sha256: Option<String>,
}

#[cfg(feature = "seed-tools")]
impl LocalEmbeddingSeed {
    pub fn open(&self, scratch: &Path) -> Result<EmbeddingSeed, String> {
        let file = std::fs::File::open(&self.path).map_err(|error| error.to_string())?;
        EmbeddingSeed::read(file, &self.file, scratch).map_err(|error| error.to_string())
    }
}

#[cfg(feature = "seed-tools")]
pub(crate) fn open_local(
    files: &[LocalEmbeddingSeed],
    model: &SourceSearchProfileIdentity,
    scratch: &Path,
    part: SeedPart,
) -> Result<Option<EmbeddingSeed>, String> {
    let mut matching = files
        .iter()
        .filter(|entry| entry.profile == model.profile_id);
    let base = matching
        .clone()
        .find(|entry| entry.base_sha256.is_none())
        .ok_or_else(|| format!("no local base asset supplied for {}", model.profile_id))?;
    match part {
        SeedPart::Base => base.open(scratch).map(Some),
        SeedPart::Delta => matching
            .find(|entry| entry.base_sha256.is_some())
            .map(|delta| {
                validate_delta_binding(
                    model.profile_id.as_str(),
                    &base.file,
                    delta.base_sha256.as_deref().unwrap_or_default(),
                    &delta.file,
                )?;
                delta.open(scratch)
            })
            .transpose(),
    }
}

pub(crate) fn open_published(
    model: &SourceSearchProfileIdentity,
    scratch: &Path,
    offline: bool,
    part: SeedPart,
) -> Result<Option<EmbeddingSeed>, String> {
    let pins = published_embedding_seed_pins()?;
    let pins = pins
        .profiles
        .get(model.profile_id.as_str())
        .ok_or_else(|| "this Atlas build has no published asset for the profile".to_owned())?;
    let pin = match part {
        SeedPart::Base => &pins.base,
        SeedPart::Delta => match &pins.delta {
            Some(delta) => &delta.asset,
            None => return Ok(None),
        },
    };
    if offline {
        return Err("the asset needs a download and setup is offline".into());
    }
    let mut file = tempfile::tempfile_in(scratch).map_err(|error| error.to_string())?;
    download_seed(pin, &model.profile_id, &mut file)?;
    let label = profile_label(&model.profile_id);
    EmbeddingSeed::read_with_progress(file, &pin.file, scratch, label)
        .map(Some)
        .map_err(|error| error.to_string())
}

fn profile_label(profile: &atlas_domain::SearchProfileId) -> &str {
    atlas_embedding::search_profile(profile).map_or(profile.as_str(), |spec| spec.label)
}

fn download_seed(
    pin: &EmbeddingSeedPin,
    profile: &atlas_domain::SearchProfileId,
    mut file: impl Write,
) -> Result<(), String> {
    pin.file.validate().map_err(|error| error.to_string())?;
    let url = release_url(&pin.tag, &pin.asset)?;
    tracing::info!(target:"atlas_progress", phase="fetch_embedding_seed", profile=%profile,
        "Downloading precomputed embeddings for {}", profile_label(profile));
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .https_only(true)
        .max_redirects(4)
        .timeout_global(Some(Duration::from_secs(3600)))
        .build()
        .into();
    let mut response = agent
        .get(&url)
        .header("Accept-Encoding", "identity")
        .call()
        .map_err(|error| error.to_string())?;
    let mut reader = response.body_mut().as_reader().take(pin.file.bytes + 1);
    if io::copy(&mut reader, &mut file).map_err(|error| error.to_string())? != pin.file.bytes {
        return Err("download size differs from the release pin".into());
    }
    Ok(())
}

#[cfg(feature = "seed-tools")]
pub fn download_embedding_seed(
    profile: atlas_domain::SearchProfileId,
    pin: &EmbeddingSeedPin,
    directory: &Path,
) -> Result<LocalEmbeddingSeed, String> {
    let result = |error: std::io::Error| error.to_string();
    let verify = |file| pin.file.verify(file).map_err(|error| error.to_string());
    pin.file.validate().map_err(|error| error.to_string())?;
    let path = directory.join(format!("{}.atlas-seed.gz", pin.file.sha256));
    if !path.try_exists().map_err(result)? {
        let mut staged = tempfile::NamedTempFile::new_in(directory).map_err(result)?;
        download_seed(pin, &profile, staged.as_file_mut())?;
        verify(staged.as_file())?;
        staged
            .persist_noclobber(&path)
            .map_err(|error| error.to_string())?;
    } else {
        verify(&std::fs::File::open(&path).map_err(result)?)?;
    }
    Ok(LocalEmbeddingSeed {
        profile,
        path,
        file: pin.file.clone(),
        base_sha256: None,
    })
}

fn safe_component(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
        && !matches!(value, "." | "..")
}

fn release_url(tag: &str, asset: &str) -> Result<String, String> {
    if tag.len() > 128 || !tag.split('/').all(safe_component) || !safe_component(asset) {
        return Err("invalid release tag or asset name".into());
    }
    // GitHub's download route treats the tag as one path segment.
    let tag = tag.replace('/', "%2F");
    Ok(format!("{RELEASE}/{tag}/{asset}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn historical_pins_normalize_and_deltas_bind_the_exact_base() {
        use serde_json::json;

        let digest = |digit: &str| digit.repeat(64);
        let base = json!({"tag": "embedding-seed/bge-base/v1", "asset": "base.gz",
            "file": {"bytes": 1, "sha256": digest("a")}});
        let historical = json!({"bge-base": &base});
        let mut current = json!({"version": 2, "profiles": {"bge-base": {"base": &base}}});
        assert_eq!(
            parse_embedding_seed_pins(&historical.to_string()).unwrap(),
            parse_embedding_seed_pins(&current.to_string()).unwrap()
        );
        let mut delta = json!({"base_sha256": digest("a"), "asset": {
            "tag": "embedding-seed/bge-base/delta/v1", "asset": "delta.gz",
            "file": {"bytes": 1, "sha256": digest("b")}
        }});
        current["profiles"]["bge-base"]["delta"] = delta.clone();
        assert!(parse_embedding_seed_pins(&current.to_string()).is_ok());
        delta["base_sha256"] = digest("c").into();
        current["profiles"]["bge-base"]["delta"] = delta;
        assert!(parse_embedding_seed_pins(&current.to_string()).is_err());
    }

    #[test]
    fn release_download_encodes_the_tag_as_one_path_segment() {
        assert_eq!(
            release_url(
                "embedding-seed/bge-base/v1",
                "bge-base-seed-v1.atlas-seed.gz"
            )
            .unwrap(),
            format!("{RELEASE}/embedding-seed%2Fbge-base%2Fv1/bge-base-seed-v1.atlas-seed.gz"),
        );
        assert_eq!(
            release_url("legacy-v1", "seed.gz").unwrap(),
            format!("{RELEASE}/legacy-v1/seed.gz"),
        );
    }

    #[test]
    fn release_download_rejects_unsafe_tag_segments_and_asset_names() {
        for tag in [
            "",
            "/leading",
            "trailing/",
            "two//parts",
            "prefix/../v1",
            "prefix/./v1",
            "prefix%2Fv1",
            "v1?x",
        ] {
            assert!(release_url(tag, "seed.gz").is_err(), "{tag}");
        }
        assert!(release_url("embedding-seed/bge-base/v1", "dir/seed.gz").is_err());
    }
}
