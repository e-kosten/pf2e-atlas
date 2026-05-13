use std::fs;
use std::path::Path;

pub(super) fn migration_rename_pairs_from_root(source_root: &Path) -> Vec<(String, String)> {
    let migration_root = source_root.join("src/module/migration/migrations");
    let Ok(entries) = fs::read_dir(migration_root) else {
        return Vec::new();
    };

    let mut paths = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "ts"))
        .collect::<Vec<_>>();
    paths.sort();

    let mut pairs = Vec::new();
    for path in paths {
        let Ok(source) = fs::read_to_string(path) else {
            continue;
        };
        pairs.extend(migration_rename_pairs(&source));
    }
    pairs
}

pub(super) fn migration_rename_pairs(source: &str) -> Vec<(String, String)> {
    let mut pairs = Vec::new();
    let mut rest = source;
    while let Some(start) = rest.find("Rename all uses and mentions of \"") {
        rest = &rest[start + "Rename all uses and mentions of \"".len()..];
        let Some(old_end) = rest.find('"') else {
            break;
        };
        let legacy_name = rest[..old_end].trim().to_string();
        rest = &rest[old_end + 1..];
        let Some(to_start) = rest.find(" to \"") else {
            continue;
        };
        rest = &rest[to_start + " to \"".len()..];
        let Some(new_end) = rest.find('"') else {
            break;
        };
        let remaster_name = rest[..new_end].trim().to_string();
        rest = &rest[new_end + 1..];
        if !legacy_name.is_empty() && !remaster_name.is_empty() {
            pairs.push((legacy_name, remaster_name));
        }
    }
    pairs
}
