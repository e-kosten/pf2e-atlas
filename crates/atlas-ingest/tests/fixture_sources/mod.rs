use std::fs;
use std::path::{Path, PathBuf};

pub(crate) fn write_generated_affliction_fixture_source(
    root: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    copy_fixture_source("generated-afflictions", root)
}

pub(crate) fn write_family_fixture_source(root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    copy_fixture_source("families", root)
}

pub(crate) fn write_remaster_fixture_source(root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    copy_fixture_source("remaster-links", root)
}

pub(crate) fn write_fixture_source(root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    copy_fixture_source("core-record-types", root)
}

fn copy_fixture_source(name: &str, root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let source = fixture_source_dir(name);
    copy_dir_recursive(&source, root)
}

fn fixture_source_dir(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/foundry-source")
        .join(name)
}

fn copy_dir_recursive(source: &Path, target: &Path) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(target)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let entry_source = entry.path();
        let entry_target = target.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir_recursive(&entry_source, &entry_target)?;
        } else {
            fs::copy(&entry_source, &entry_target)?;
        }
    }
    Ok(())
}

pub(crate) fn fixture_root(name: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!(
        "atlas-ingest-{name}-{}-{}",
        std::process::id(),
        std::thread::current().name().unwrap_or("test")
    ));
    let _ = fs::remove_dir_all(&path);
    path
}
