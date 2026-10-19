//! Acquire only the immutable reference revision paired with the generated model.
use atlas_foundry_model::generated::{REFERENCE_SOURCE_COMMIT, REFERENCE_SOURCE_REPOSITORY};
use std::{path::Path, process::Command};

pub(crate) fn validate(path: &Path) -> Result<(), String> {
    validate_revision(path, REFERENCE_SOURCE_COMMIT)
}

fn git(path: &Path, args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(path)
        .args(args)
        .output()
        .map_err(|e| format!("cannot run Git: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "source Git operation failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    String::from_utf8(output.stdout).map_err(|e| e.to_string())
}

fn validate_revision(path: &Path, commit: &str) -> Result<(), String> {
    if !path.join(".git").is_dir() {
        return Err(format!(
            "managed source {} is not a Git checkout; expected {commit}",
            path.display()
        ));
    }
    let actual = git(path, &["rev-parse", "HEAD"])?;
    if actual.trim() != commit {
        return Err(format!(
            "managed source revision mismatch: expected {commit}, detected {}",
            actual.trim()
        ));
    }
    if !git(
        path,
        &[
            "status",
            "--porcelain",
            "--untracked-files=all",
            "--ignored",
        ],
    )?
    .trim()
    .is_empty()
    {
        return Err("managed source contains local changes; preserve them and use a clean pinned cache or an explicit --source directory".into());
    }
    Ok(())
}

pub(crate) fn acquire(path: &Path) -> Result<(), String> {
    acquire_revision(path, REFERENCE_SOURCE_REPOSITORY, REFERENCE_SOURCE_COMMIT)
}

fn acquire_revision(path: &Path, repository: &str, commit: &str) -> Result<(), String> {
    if path.exists() {
        return validate_revision(path, commit);
    }
    let parent = path
        .parent()
        .ok_or("source cache has no parent directory")?;
    std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    let temporary = tempfile::Builder::new()
        .prefix(".pf2e-fetch-")
        .tempdir_in(parent)
        .map_err(|e| e.to_string())?;
    let checkout = temporary.path().join("checkout");
    std::fs::create_dir(&checkout).map_err(|e| e.to_string())?;
    git(&checkout, &["init", "--quiet"])?;
    git(
        &checkout,
        &[
            "fetch",
            "--quiet",
            "--depth",
            "1",
            "--no-tags",
            repository,
            commit,
        ],
    )?;
    git(
        &checkout,
        &[
            "-c",
            "advice.detachedHead=false",
            "checkout",
            "--quiet",
            "--detach",
            commit,
        ],
    )?;
    validate_revision(&checkout, commit)?;
    // A competing setup may have published the same immutable cache. Never replace it.
    match std::fs::rename(&checkout, path) {
        Ok(()) => Ok(()),
        Err(_) if path.exists() => validate_revision(path, commit),
        Err(error) => Err(format!("cannot publish pinned source cache: {error}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn repository(root: &Path) -> (std::path::PathBuf, String) {
        let path = root.join("upstream");
        fs::create_dir(&path).unwrap();
        git(&path, &["init", "--quiet"]).unwrap();
        fs::write(path.join("record.json"), "pinned").unwrap();
        git(&path, &["add", "."]).unwrap();
        git(
            &path,
            &[
                "-c",
                "user.name=Atlas Test",
                "-c",
                "user.email=test@example.invalid",
                "-c",
                "commit.gpgsign=false",
                "commit",
                "--quiet",
                "-m",
                "pinned",
            ],
        )
        .unwrap();
        let first = git(&path, &["rev-parse", "HEAD"])
            .unwrap()
            .trim()
            .to_owned();
        fs::write(path.join("record.json"), "later").unwrap();
        git(&path, &["add", "."]).unwrap();
        git(
            &path,
            &[
                "-c",
                "user.name=Atlas Test",
                "-c",
                "user.email=test@example.invalid",
                "-c",
                "commit.gpgsign=false",
                "commit",
                "--quiet",
                "-m",
                "later",
            ],
        )
        .unwrap();
        (path, first)
    }

    #[test]
    fn acquisition_uses_exact_commit_and_reuses_it_without_network() {
        let temp = tempfile::tempdir().unwrap();
        let (upstream, first) = repository(temp.path());
        let destination = temp.path().join("cache").join(&first);
        acquire_revision(&destination, upstream.to_str().unwrap(), &first).unwrap();
        assert_eq!(
            fs::read_to_string(destination.join("record.json")).unwrap(),
            "pinned"
        );
        assert!(git(&destination, &["symbolic-ref", "HEAD"]).is_err());
        acquire_revision(&destination, "nonexistent-upstream", &first).unwrap();
        assert_eq!(
            fs::read_to_string(upstream.join("record.json")).unwrap(),
            "later"
        );
    }

    #[test]
    fn acquisition_preserves_dirty_or_wrong_revision_directories() {
        let temp = tempfile::tempdir().unwrap();
        let (upstream, first) = repository(temp.path());
        assert!(
            acquire_revision(&upstream, "unused", &first)
                .unwrap_err()
                .contains("revision mismatch")
        );
        assert_eq!(
            fs::read_to_string(upstream.join("record.json")).unwrap(),
            "later"
        );
        let destination = temp.path().join("cache");
        acquire_revision(&destination, upstream.to_str().unwrap(), &first).unwrap();
        fs::write(destination.join("record.json"), "user changes").unwrap();
        assert!(
            acquire_revision(&destination, "unused", &first)
                .unwrap_err()
                .contains("local changes")
        );
        assert_eq!(
            fs::read_to_string(destination.join("record.json")).unwrap(),
            "user changes"
        );
    }

    #[test]
    fn failed_fetch_never_publishes_partial_source() {
        let temp = tempfile::tempdir().unwrap();
        let (upstream, _) = repository(temp.path());
        let parent = temp.path().join("cache");
        let destination = parent.join("missing");
        assert!(
            acquire_revision(
                &destination,
                upstream.to_str().unwrap(),
                "1111111111111111111111111111111111111111"
            )
            .is_err()
        );
        assert!(!destination.exists());
        assert_eq!(fs::read_dir(parent).unwrap().count(), 0);
    }
}
