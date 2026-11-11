//! Release pins and acquisition policy for managed English setup.
use std::{
    io::{self, Read},
    path::Path,
    time::Duration,
};

use atlas_index::{EmbeddingSeed, EmbeddingSeedFile, SourceSearchProfileIdentity};
use serde::Deserialize;

const RELEASE: &str = "https://github.com/e-kosten/pf2e-atlas/releases/download";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Pin {
    tag: String,
    asset: String,
    file: EmbeddingSeedFile,
}

pub(crate) type SeedSource<'a> =
    dyn FnMut(&SourceSearchProfileIdentity, &Path, bool) -> Result<EmbeddingSeed, String> + 'a;

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
    EmbeddingSeed::open(file, &entry.file, model, scratch).map_err(|error| error.to_string())
}

pub(crate) fn open_published(
    model: &SourceSearchProfileIdentity,
    scratch: &Path,
    offline: bool,
) -> Result<EmbeddingSeed, String> {
    if offline {
        return Err("the asset needs a download and setup is offline".into());
    }
    let pins: std::collections::BTreeMap<String, Pin> =
        serde_json::from_str(include_str!("../embedding-seeds.json"))
            .map_err(|error| format!("invalid release pins: {error}"))?;
    let pin = pins
        .get(model.profile_id.as_str())
        .ok_or_else(|| "this Atlas build has no published asset for the profile".to_owned())?;
    pin.file.validate().map_err(|error| error.to_string())?;
    let url = release_url(&pin.tag, &pin.asset)?;
    tracing::info!(target:"atlas_progress", phase="fetch_embedding_seed", profile=%model.profile_id,
        "Downloading precomputed embeddings for {}", model.profile_id);
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
    let mut file = tempfile::tempfile_in(scratch).map_err(|error| error.to_string())?;
    if io::copy(&mut reader, &mut file).map_err(|error| error.to_string())? != pin.file.bytes {
        return Err("download size differs from the release pin".into());
    }
    tracing::info!(target:"atlas_progress", phase="verify_embedding_seed", profile=%model.profile_id,
        "Verifying precomputed embeddings for {}", model.profile_id);
    EmbeddingSeed::open(file, &pin.file, model, scratch).map_err(|error| error.to_string())
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
