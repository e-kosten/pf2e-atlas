//! Cache the approved bundled reference supplement without polling upstream.
use atlas_ingest::{BundledReferenceFile, ReferenceSupplementSelection};
use std::{
    collections::BTreeSet,
    io::Read,
    path::{Component, Path, PathBuf},
};

pub(crate) fn selection(
    cache_root: &Path,
    check: bool,
) -> Result<ReferenceSupplementSelection, String> {
    let identity = atlas_ingest::bundled_reference_supplement_id();
    let files = atlas_ingest::bundled_reference_supplement();
    let path = cache_root.join(&identity);
    validate_bundle(&identity, files)?;
    if path.exists() {
        validate_cache(&path, files)?;
        return Ok(ReferenceSupplementSelection::Directory { path });
    }
    if check {
        // The same approved bytes are available without changing persistent state.
        return Ok(ReferenceSupplementSelection::Bundled);
    }
    install_bundle(cache_root, &identity, files)
        .map(|path| ReferenceSupplementSelection::Directory { path })
}

fn validate_bundle(identity: &str, files: &[BundledReferenceFile]) -> Result<(), String> {
    if identity.len() != 64
        || !identity
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
        || files.is_empty()
    {
        return Err("approved reference supplement identity is invalid".into());
    }
    let mut names = BTreeSet::new();
    for file in files {
        let mut components = Path::new(file.name).components();
        if !matches!(components.next(), Some(Component::Normal(_)))
            || components.next().is_some()
            || !names.insert(file.name)
        {
            return Err("approved reference supplement file names are invalid".into());
        }
    }
    Ok(())
}

fn validate_cache(path: &Path, files: &[BundledReferenceFile]) -> Result<(), String> {
    for file in files {
        let local_path = path.join(file.name);
        let mut input = std::fs::File::open(&local_path).map_err(|error| {
            format!(
                "reference supplement {} is unavailable: {error}",
                local_path.display()
            )
        })?;
        // Bound reads even if a cache file changes between inspection and capture.
        let mut bytes = Vec::with_capacity(file.bytes.len());
        (&mut input)
            .take(file.bytes.len() as u64 + 1)
            .read_to_end(&mut bytes)
            .map_err(|error| {
                format!(
                    "cannot read reference supplement {}: {error}",
                    local_path.display()
                )
            })?;
        if bytes != file.bytes {
            return Err(format!(
                "reference supplement {} differs from the approved asset; preserve it and use a verified cache",
                local_path.display()
            ));
        }
    }
    Ok(())
}

fn install_bundle(
    cache_root: &Path,
    identity: &str,
    files: &[BundledReferenceFile],
) -> Result<PathBuf, String> {
    validate_bundle(identity, files)?;
    let destination = cache_root.join(identity);
    if destination.exists() {
        validate_cache(&destination, files)?;
        return Ok(destination);
    }
    std::fs::create_dir_all(cache_root).map_err(|error| {
        format!(
            "cannot create reference supplement cache {}: {error}",
            cache_root.display()
        )
    })?;
    let temporary = tempfile::Builder::new()
        .prefix(".reference-supplement-")
        .tempdir_in(cache_root)
        .map_err(|error| format!("cannot prepare reference supplement cache: {error}"))?;
    let prepared = temporary.path().join("asset");
    std::fs::create_dir(&prepared).map_err(|error| error.to_string())?;
    for file in files {
        std::fs::write(prepared.join(file.name), file.bytes).map_err(|error| {
            format!("cannot prepare reference supplement {}: {error}", file.name)
        })?;
    }
    validate_cache(&prepared, files)?;
    // A competing setup may have installed the same immutable asset. Never replace it.
    match std::fs::rename(&prepared, &destination) {
        Ok(()) => Ok(destination),
        Err(_) if destination.exists() => {
            validate_cache(&destination, files)?;
            Ok(destination)
        }
        Err(error) => Err(format!(
            "cannot publish reference supplement cache: {error}"
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn approved_bundle_is_available_offline_and_check_does_not_create_cache() {
        let temporary = tempfile::tempdir().unwrap();
        let cache = temporary.path().join("reference");
        assert!(matches!(
            selection(&cache, true).unwrap(),
            ReferenceSupplementSelection::Bundled
        ));
        assert!(!cache.exists());
        let ReferenceSupplementSelection::Directory { path } = selection(&cache, false).unwrap()
        else {
            panic!("normal setup must select the verified cache");
        };
        validate_cache(&path, atlas_ingest::bundled_reference_supplement()).unwrap();
        assert!(matches!(
            selection(&cache, true).unwrap(),
            ReferenceSupplementSelection::Directory { .. }
        ));
        assert_eq!(std::fs::read_dir(&cache).unwrap().count(), 1);
    }

    #[test]
    fn corrupt_or_missing_selected_cache_files_are_preserved_and_rejected() {
        for remove in [false, true] {
            let temporary = tempfile::tempdir().unwrap();
            let cache = temporary.path().join("reference");
            let ReferenceSupplementSelection::Directory { path } =
                selection(&cache, false).unwrap()
            else {
                panic!("expected cached selection");
            };
            let file = path.join(atlas_ingest::bundled_reference_supplement()[0].name);
            if remove {
                std::fs::remove_file(&file).unwrap();
            } else {
                std::fs::write(&file, "customized asset").unwrap();
            }
            for check in [true, false] {
                assert!(selection(&cache, check).is_err());
            }
            if remove {
                assert!(!file.exists());
            } else {
                assert_eq!(std::fs::read_to_string(&file).unwrap(), "customized asset");
            }
        }
    }

    #[test]
    fn invalid_bundle_never_creates_a_partial_cache() {
        let temporary = tempfile::tempdir().unwrap();
        let cache = temporary.path().join("reference");
        let bad = [BundledReferenceFile {
            name: "../escape",
            bytes: b"not an asset",
        }];
        assert!(install_bundle(&cache, &"0".repeat(64), &bad).is_err());
        assert!(!cache.exists());
        assert!(install_bundle(&cache, "../escape", &[]).is_err());
        assert!(!cache.exists());
    }
}
