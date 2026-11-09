//! Acquire and verify the immutable source tree paired with the executable.
use atlas_foundry_model::generated::{REFERENCE_SOURCE_COMMIT, REFERENCE_SOURCE_REPOSITORY};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::{self, Read},
    path::{Path, PathBuf},
    time::Duration,
};

const MAX_ARCHIVE: u64 = 512 * 1024 * 1024;
const MAX_TREE: u64 = 1024 * 1024 * 1024;
const MAX_ENTRIES: usize = 100_000;
const PIN: &str = include_str!("../../../dev-tools/source-contracts/source-pin.json");

pub(crate) fn revision(
    managed: bool,
    custom_manifest: bool,
) -> atlas_ingest::SourceRevision<'static> {
    use atlas_ingest::SourceRevision;
    match (managed, custom_manifest) {
        (true, false) => SourceRevision::Verified(REFERENCE_SOURCE_COMMIT),
        (true, true) => SourceRevision::Unspecified,
        (false, _) => SourceRevision::DetectGit,
    }
}

pub(crate) fn validate(path: &Path) -> Result<(), String> {
    let expected = expected_digest()?;
    let mut tree = Tree::default();
    scan(path, "", &mut tree).map_err(|error| error.to_string())?;
    compare(tree, &expected)
}

fn expected_digest() -> Result<String, String> {
    #[derive(serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Pin {
        repository: String,
        commit: String,
        archive_tree_sha256: String,
    }
    let pin: Pin = serde_json::from_str(PIN).map_err(|error| error.to_string())?;
    if pin.commit != REFERENCE_SOURCE_COMMIT || pin.repository != REFERENCE_SOURCE_REPOSITORY {
        return Err("managed source archive pin differs from the generated model".into());
    }
    Ok(pin.archive_tree_sha256)
}

fn compare(tree: Tree, expected: &str) -> Result<(), String> {
    if tree.digest() != expected {
        return Err("managed Foundry source differs from the pinned archive; preserve local edits and use a clean cache or an explicit --source directory".into());
    }
    Ok(())
}

pub(crate) fn acquire(path: &Path) -> Result<(), String> {
    if path.exists() {
        return validate(path);
    }
    let expected = expected_digest()?;
    let repository = REFERENCE_SOURCE_REPOSITORY
        .strip_prefix("https://github.com/")
        .and_then(|value| value.strip_suffix(".git"))
        .ok_or("managed Foundry source repository must be hosted on GitHub")?;
    let url = format!("https://codeload.github.com/{repository}/tar.gz/{REFERENCE_SOURCE_COMMIT}");
    let parent = path
        .parent()
        .ok_or("source cache has no parent directory")?;
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    let temporary = tempfile::Builder::new()
        .prefix(".pf2e-fetch-")
        .tempdir_in(parent)
        .map_err(|error| error.to_string())?;
    tracing::info!(target:"atlas_progress", phase="fetch_source", revision=REFERENCE_SOURCE_COMMIT, "Downloading Foundry source");
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .max_redirects(0)
        .timeout_global(Some(Duration::from_secs(600)))
        .build()
        .into();
    let mut response = agent.get(url).call().map_err(|error| error.to_string())?;
    let bytes = response
        .body_mut()
        .with_config()
        .limit(MAX_ARCHIVE)
        .reader();
    let mut archive = flate2::read::GzDecoder::new(bytes).take(MAX_TREE);
    tracing::info!(target:"atlas_progress", phase="extract_source", "Extracting and verifying Foundry source");
    let root = format!("pf2e-{REFERENCE_SOURCE_COMMIT}");
    let tree = extract(&mut archive, &root, temporary.path()).map_err(|error| error.to_string())?;
    io::copy(&mut archive, &mut io::sink()).map_err(|error| error.to_string())?;
    if archive.limit() == 0 {
        return Err(invalid_archive().to_string());
    }
    compare(tree, &expected)?;
    match fs::rename(temporary.path(), path) {
        Ok(()) => Ok(()),
        Err(_) if path.exists() => validate(path),
        Err(error) => Err(format!("cannot publish pinned source cache: {error}")),
    }
}

#[derive(Default)]
struct Tree {
    files: BTreeMap<String, [u8; 32]>,
    case_insensitive_names: BTreeSet<String>,
    bytes: u64,
}

impl Tree {
    fn add(&mut self, name: String, size: u64, reader: &mut impl Read) -> io::Result<()> {
        self.bytes = self.bytes.checked_add(size).ok_or_else(invalid_archive)?;
        if self.files.len() >= MAX_ENTRIES
            || self.bytes > MAX_TREE
            || !self.case_insensitive_names.insert(name.to_lowercase())
        {
            return Err(invalid_archive());
        }
        let mut hash = Sha256::new();
        if io::copy(reader, &mut hash)? != size {
            return Err(invalid_archive());
        }
        self.files.insert(name, hash.finalize().into());
        Ok(())
    }

    fn digest(self) -> String {
        let mut hash = Sha256::new();
        hash.update(b"atlas-managed-source-v1\0");
        for (name, file_hash) in self.files {
            hash.update((name.len() as u64).to_be_bytes());
            hash.update(name.as_bytes());
            hash.update(file_hash);
        }
        format!("{:x}", hash.finalize())
    }
}

fn scan(directory: &Path, prefix: &str, tree: &mut Tree) -> io::Result<()> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let kind = entry.file_type()?;
        let component = entry
            .file_name()
            .into_string()
            .map_err(|_| invalid_archive())?;
        if prefix.is_empty() && component == ".git" && kind.is_dir() {
            continue;
        }
        if !valid_component(&component) {
            return Err(invalid_archive());
        }
        let name = format!("{prefix}{component}");
        if kind.is_dir() {
            scan(&entry.path(), &format!("{name}/"), tree)?;
        } else if kind.is_file() {
            let mut file = fs::File::open(entry.path())?;
            tree.add(name, file.metadata()?.len(), &mut file)?;
        } else {
            return Err(invalid_archive());
        }
    }
    Ok(())
}

fn valid_component(name: &str) -> bool {
    let base = name
        .split('.')
        .next()
        .unwrap_or_default()
        .to_ascii_uppercase();
    !name.is_empty()
        && name != ".git"
        && !name.ends_with([' ', '.'])
        && !name
            .chars()
            .any(|value| value.is_control() || "\\/:*?\"<>|".contains(value))
        && !matches!(base.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        && !(base.len() == 4
            && (base.starts_with("COM") || base.starts_with("LPT"))
            && matches!(base.as_bytes()[3], b'1'..=b'9'))
}

fn extract(reader: impl Read, root: &str, target: &Path) -> io::Result<Tree> {
    let mut tree = Tree::default();
    for (index, entry) in tar::Archive::new(reader).entries()?.enumerate() {
        if index >= MAX_ENTRIES {
            return Err(invalid_archive());
        }
        let mut entry = entry?;
        let kind = entry.header().entry_type();
        // Git archive emits one small global PAX record for commit metadata.
        // The pinned tree digest authenticates the actual payload independently.
        if index == 0 && kind.is_pax_global_extensions() && entry.size() <= 4096 {
            continue;
        }
        let bytes = entry.path_bytes();
        let name = std::str::from_utf8(&bytes).map_err(|_| invalid_archive())?;
        let name = if kind.is_dir() {
            name.trim_end_matches('/')
        } else {
            name
        };
        let mut parts = name.split('/');
        if parts.next() != Some(root) {
            return Err(invalid_archive());
        }
        let mut path = PathBuf::from(target);
        for part in parts {
            if !valid_component(part) {
                return Err(invalid_archive());
            }
            path.push(part);
        }
        if kind.is_dir() {
            continue;
        }
        if !kind.is_file() || path == target {
            return Err(invalid_archive());
        }
        let relative = name
            .strip_prefix(root)
            .and_then(|name| name.strip_prefix('/'))
            .ok_or_else(invalid_archive)?
            .to_owned();
        let size = entry.size();
        fs::create_dir_all(path.parent().ok_or_else(invalid_archive)?)?;
        let mut output = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)?;
        tree.add(relative, size, &mut CopyTo::new(&mut entry, &mut output))?;
    }
    Ok(tree)
}

struct CopyTo<'a, R, W> {
    reader: &'a mut R,
    writer: &'a mut W,
}

impl<'a, R, W> CopyTo<'a, R, W> {
    fn new(reader: &'a mut R, writer: &'a mut W) -> Self {
        Self { reader, writer }
    }
}

impl<R: Read, W: io::Write> Read for CopyTo<'_, R, W> {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        let count = self.reader.read(buffer)?;
        self.writer.write_all(&buffer[..count])?;
        Ok(count)
    }
}

fn invalid_archive() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        "unsafe or oversized managed Foundry source archive",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_and_verifies_the_same_file_inventory_as_an_existing_cache() {
        let mut archive = tar::Builder::new(Vec::new());
        let contents = b"reference bytes";
        let mut header = tar::Header::new_gnu();
        header.set_mode(0o644);
        header.set_size(contents.len() as u64);
        archive
            .append_data(&mut header, "pf2e-pin/packs/record.json", &contents[..])
            .unwrap();
        let bytes = archive.into_inner().unwrap();
        let directory = tempfile::tempdir().unwrap();
        let expected = extract(bytes.as_slice(), "pf2e-pin", directory.path())
            .unwrap()
            .digest();
        fs::create_dir(directory.path().join(".git")).unwrap();
        let mut actual = Tree::default();
        scan(directory.path(), "", &mut actual).unwrap();
        assert!(compare(actual, &expected).is_ok());
        fs::write(directory.path().join("packs/record.json"), "different").unwrap();
        let mut changed = Tree::default();
        scan(directory.path(), "", &mut changed).unwrap();
        assert!(compare(changed, &expected).is_err());
    }

    #[test]
    fn rejects_unsafe_paths_before_creating_files() {
        for path in [
            "../record.json",
            "C:/record.json",
            "packs\\record.json",
            "NUL.txt",
            ".git",
        ] {
            let mut archive = tar::Builder::new(Vec::new());
            let mut header = tar::Header::new_gnu();
            header.set_mode(0o644);
            header.set_size(0);
            let name = format!("pf2e-pin/{path}");
            header.as_mut_bytes()[..name.len()].copy_from_slice(name.as_bytes());
            header.set_cksum();
            archive.append(&header, io::empty()).unwrap();
            let directory = tempfile::tempdir().unwrap();
            let bytes = archive.into_inner().unwrap();
            assert!(
                extract(bytes.as_slice(), "pf2e-pin", directory.path()).is_err(),
                "{path}"
            );
            assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 0);
        }
    }
}
