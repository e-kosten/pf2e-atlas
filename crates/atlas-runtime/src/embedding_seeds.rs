//! Release pins and acquisition policy for managed English setup.
use std::{
    io::{self, Read, Write},
    path::Path,
    time::Duration,
};

use atlas_index::{EmbeddingSeed, EmbeddingSeedFile, SourceSearchProfileIdentity};
use serde::{Deserialize, Serialize};

const RELEASE: &str = "https://github.com/e-kosten/pf2e-atlas/releases/download";

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EmbeddingSeedPin {
    pub tag: String,
    pub asset: String,
    pub file: EmbeddingSeedFile,
}

pub type EmbeddingSeedPins = std::collections::BTreeMap<String, EmbeddingSeedPin>;

pub fn parse_embedding_seed_pins(json: &str) -> Result<EmbeddingSeedPins, String> {
    let pins: EmbeddingSeedPins =
        serde_json::from_str(json).map_err(|error| format!("invalid release pins: {error}"))?;
    for pin in pins.values() {
        pin.file.validate().map_err(|error| error.to_string())?;
        release_url(&pin.tag, &pin.asset)?;
    }
    Ok(pins)
}

pub fn published_embedding_seed_pins() -> Result<EmbeddingSeedPins, String> {
    parse_embedding_seed_pins(include_str!("../embedding-seeds.json"))
}

pub(crate) type SeedSource<'a> = dyn FnMut(&SourceSearchProfileIdentity, &Path, bool) -> Result<Option<EmbeddingSeed>, String>
    + 'a;

#[cfg(feature = "seed-tools")]
pub struct LocalEmbeddingSeed {
    pub profile: atlas_domain::SearchProfileId,
    pub path: std::path::PathBuf,
    pub file: EmbeddingSeedFile,
}

#[cfg(feature = "seed-tools")]
pub(crate) fn open_local(
    files: &[LocalEmbeddingSeed],
    model: &SourceSearchProfileIdentity,
    scratch: &Path,
) -> Result<EmbeddingSeed, String> {
    let entry = files
        .iter()
        .find(|entry| entry.profile == model.profile_id)
        .ok_or_else(|| format!("no local asset supplied for {}", model.profile_id))?;
    let file = std::fs::File::open(&entry.path).map_err(|error| error.to_string())?;
    EmbeddingSeed::read(file, &entry.file, scratch).map_err(|error| error.to_string())
}

pub(crate) fn open_published(
    model: &SourceSearchProfileIdentity,
    scratch: &Path,
    offline: bool,
) -> Result<Option<EmbeddingSeed>, String> {
    if offline {
        return Err("the asset needs a download and setup is offline".into());
    }
    let pins = published_embedding_seed_pins()?;
    let pin = pins
        .get(model.profile_id.as_str())
        .ok_or_else(|| "this Atlas build has no published asset for the profile".to_owned())?;
    let mut file = tempfile::tempfile_in(scratch).map_err(|error| error.to_string())?;
    download_seed(pin, model.profile_id.as_str(), &mut file)?;
    tracing::info!(target:"atlas_progress", phase="verify_embedding_seed", profile=%model.profile_id,
        "Verifying precomputed embeddings for {}", model.profile_id);
    EmbeddingSeed::read(file, &pin.file, scratch)
        .map(Some)
        .map_err(|error| error.to_string())
}

fn download_seed(
    pin: &EmbeddingSeedPin,
    profile: &str,
    mut file: impl Write,
) -> Result<(), String> {
    pin.file.validate().map_err(|error| error.to_string())?;
    let url = release_url(&pin.tag, &pin.asset)?;
    tracing::info!(target:"atlas_progress", phase="fetch_embedding_seed", profile,
        "Downloading precomputed embeddings for {profile}");
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
    pin.file.validate().map_err(|error| error.to_string())?;
    let path = directory.join(format!("{}.atlas-seed.gz", pin.file.sha256));
    if !path.try_exists().map_err(result)? {
        let mut staged = tempfile::NamedTempFile::new_in(directory).map_err(result)?;
        download_seed(pin, profile.as_str(), staged.as_file_mut())?;
        check_seed_file(staged.as_file(), &pin.file)?;
        staged
            .persist_noclobber(&path)
            .map_err(|error| error.to_string())?;
    } else {
        check_seed_file(&std::fs::File::open(&path).map_err(result)?, &pin.file)?;
    }
    Ok(LocalEmbeddingSeed {
        profile,
        path,
        file: pin.file.clone(),
    })
}

#[cfg(feature = "seed-tools")]
fn check_seed_file(file: &std::fs::File, expected: &EmbeddingSeedFile) -> Result<(), String> {
    let measured = EmbeddingSeedFile::measure(file).map_err(|error| error.to_string())?;
    if measured.bytes != expected.bytes || measured.sha256 != expected.sha256 {
        return Err("download size or SHA-256 differs from the release pin".into());
    }
    Ok(())
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
