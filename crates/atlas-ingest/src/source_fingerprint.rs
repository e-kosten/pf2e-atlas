//! Content-based source freshness, independent of Git cleanliness or timestamps.
use crate::{
    IngestError,
    source::discovery::{
        default_manifest_path, parse_manifest_bytes, relative_source_path, resolve_pack_path,
    },
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    process::Command,
};

pub const SOURCE_FINGERPRINT_VERSION: &str = "source-content-sha256/v2";
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceFingerprint {
    pub kind: String,
    pub value: String,
    pub file_count: usize,
}
/// Include every file in every declared pack, including excluded, invalid and
/// unaddressable documents and pack definitions. Hash paths and exact bytes.
pub fn compute_source_fingerprint(
    source_root: &Path,
    manifest_path: Option<&Path>,
) -> Result<SourceFingerprint, IngestError> {
    Ok(SourceCapture::capture(source_root, manifest_path)?.fingerprint)
}

/// Exact inventory for one preparation pass. Only hashes and path bindings are retained.
/// Discovery uses this inventory and every consumed byte is checked before parsing;
/// a final inventory comparison alone cannot detect transient (ABA) source edits.
pub(crate) struct SourceCapture {
    pub(crate) fingerprint: SourceFingerprint,
    pub(crate) manifest_path: PathBuf,
    files: BTreeMap<PathBuf, Vec<u8>>,
    packs: BTreeMap<String, (PathBuf, bool)>,
    // The ordinary loader quarantines unreadable files. Capture failures instead
    // invalidate the entire build, even if a later read sees restored bytes.
    failed: std::sync::atomic::AtomicBool,
}
impl SourceCapture {
    pub(crate) fn capture(
        source_root: &Path,
        manifest_path: Option<&Path>,
    ) -> Result<Self, IngestError> {
        let manifest_path = manifest_path
            .map(Path::to_owned)
            .unwrap_or_else(|| default_manifest_path(source_root));
        let manifest_bytes =
            fs::read(&manifest_path).map_err(|e| IngestError::SourceUnavailable(e.to_string()))?;
        let parsed = parse_manifest_bytes(&manifest_bytes)?;
        let mut files = BTreeSet::from([manifest_path.clone()]);
        let mut packs = BTreeMap::new();
        for pack in &parsed.manifest.packs {
            let path = resolve_pack_path(source_root, pack);
            let present = path.is_dir();
            packs.insert(pack.path.clone(), (path.clone(), present));
            if present {
                collect_files(&path, &mut files)?;
            }
        }
        for directory in ["static/lang", "src/module/migration/migrations"] {
            let path = source_root.join(directory);
            if path.is_dir() {
                collect_files(&path, &mut files)?;
            }
        }
        let mut hasher = Sha256::new();
        frame(&mut hasher, SOURCE_FINGERPRINT_VERSION.as_bytes());
        // Definitions remain in the manifest bytes; explicit resolved bindings make
        // a fallback directory remap distinguishable without storing a second copy.
        for pack in &parsed.manifest.packs {
            for value in [&pack.name, &pack.label, &pack.document_type, &pack.path] {
                frame(&mut hasher, value.as_bytes());
            }
            let (path, present) = &packs[&pack.path];
            frame(
                &mut hasher,
                relative_source_path(source_root, path).as_bytes(),
            );
            frame(&mut hasher, if *present { b"present" } else { b"missing" });
        }
        let mut hashes = BTreeMap::new();
        for path in &files {
            let bytes = if path == &manifest_path {
                Ok(manifest_bytes.clone())
            } else {
                fs::read(path)
            }
            .map_err(|error| {
                IngestError::SourceUnavailable(format!("{}: {error}", path.display()))
            })?;
            frame(
                &mut hasher,
                relative_source_path(source_root, path).as_bytes(),
            );
            let digest = Sha256::digest(&bytes).to_vec();
            hashes.insert(path.clone(), digest.clone());
            frame(&mut hasher, &digest);
            frame(&mut hasher, &(bytes.len() as u64).to_le_bytes());
        }
        let fingerprint = SourceFingerprint {
            kind: SOURCE_FINGERPRINT_VERSION.into(),
            value: format!("{:x}", hasher.finalize()),
            file_count: files.len(),
        };
        Ok(Self {
            fingerprint,
            manifest_path,
            files: hashes,
            packs,
            failed: false.into(),
        })
    }
    pub(crate) fn read(&self, path: &Path) -> Result<Vec<u8>, IngestError> {
        let result = fs::read(path)
            .map_err(|e| IngestError::SourceUnavailable(format!("{}: {e}", path.display())))
            .and_then(|bytes| {
                if self
                    .files
                    .get(path)
                    .is_some_and(|hash| *hash == Sha256::digest(&bytes).as_slice())
                {
                    Ok(bytes)
                } else {
                    Err(IngestError::SourceUnavailable(format!(
                        "source changed after capture: {}",
                        path.display()
                    )))
                }
            });
        if result.is_err() {
            self.failed
                .store(true, std::sync::atomic::Ordering::Relaxed);
        }
        result
    }
    pub(crate) fn verify_reads(&self) -> Result<(), IngestError> {
        if self.failed.load(std::sync::atomic::Ordering::Relaxed) {
            Err(IngestError::SourceUnavailable(
                "captured source read failed; retry build".into(),
            ))
        } else {
            Ok(())
        }
    }
    pub(crate) fn paths(&self, directory: &Path) -> Vec<PathBuf> {
        self.files
            .keys()
            .filter(|p| p.starts_with(directory))
            .cloned()
            .collect()
    }
    pub(crate) fn pack(&self, declared_path: &str) -> &(PathBuf, bool) {
        &self.packs[declared_path]
    }
}
fn frame(hasher: &mut Sha256, value: &[u8]) {
    hasher.update((value.len() as u64).to_le_bytes());
    hasher.update(value);
}
fn collect_files(path: &Path, output: &mut BTreeSet<PathBuf>) -> Result<(), IngestError> {
    for entry in fs::read_dir(path).map_err(|e| IngestError::SourceUnavailable(e.to_string()))? {
        let entry = entry.map_err(|e| IngestError::SourceUnavailable(e.to_string()))?;
        let kind = entry
            .file_type()
            .map_err(|e| IngestError::SourceUnavailable(e.to_string()))?;
        if kind.is_symlink() {
            return Err(IngestError::SourceUnavailable(format!(
                "source pack symlink is unsupported: {}",
                entry.path().display()
            )));
        }
        if kind.is_dir() {
            collect_files(&entry.path(), output)?;
        } else if kind.is_file() {
            output.insert(entry.path());
        }
    }
    Ok(())
}
/// Revision is provenance only: the content fingerprint is always authoritative.
pub fn source_git_commit_if_clean(source_root: &Path) -> Result<String, IngestError> {
    // An exported directory has no Git revision. Avoid spawning a process that
    // searches ancestor repositories only to reject that unrelated checkout.
    if !source_root.join(".git").exists() {
        return Err(IngestError::SourceUnavailable(
            "source root is not a Git checkout root".into(),
        ));
    }
    let top = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(source_root)
        .output()
        .map_err(|error| IngestError::SourceUnavailable(error.to_string()))?;
    let top = String::from_utf8(top.stdout)
        .map_err(|error| IngestError::SourceUnavailable(error.to_string()))?;
    let top = Path::new(top.trim())
        .canonicalize()
        .map_err(|error| IngestError::SourceUnavailable(error.to_string()))?;
    if source_root
        .canonicalize()
        .map_err(|error| IngestError::SourceUnavailable(error.to_string()))?
        != top
    {
        return Err(IngestError::SourceUnavailable(
            "source root is not a Git checkout root".into(),
        ));
    }
    let status = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(source_root)
        .output()
        .map_err(|e| IngestError::SourceUnavailable(e.to_string()))?;
    if !status.status.success() || !status.stdout.is_empty() {
        return Err(IngestError::SourceUnavailable(
            "source Git checkout is unavailable or dirty".into(),
        ));
    }
    let head = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(source_root)
        .output()
        .map_err(|e| IngestError::SourceUnavailable(e.to_string()))?;
    if !head.status.success() {
        return Err(IngestError::SourceUnavailable(
            "source Git revision unavailable".into(),
        ));
    }
    String::from_utf8(head.stdout)
        .map(|value| value.trim().to_owned())
        .map_err(|e| IngestError::SourceUnavailable(e.to_string()))
}
