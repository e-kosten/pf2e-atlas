use std::{error::Error, path::PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args == ["--describe"] {
        let profiles = atlas_embedding::SEARCH_PROFILES
            .iter()
            .copied()
            .map(atlas_index::SourceSearchProfileIdentity::from_spec)
            .collect::<Vec<_>>();
        println!(
            "{}",
            serde_json::json!({
                "schema": atlas_runtime::WORKSPACE_SCHEMA_VERSION,
                "profiles": profiles,
            })
        );
        return Ok(());
    }
    let [path] = args.as_slice() else {
        return Err("usage: migrate-fixture-workspace EXISTING_DATABASE | --describe".into());
    };
    let path = PathBuf::from(path);
    if !path.is_file() {
        return Err(format!("Fixture database does not exist: {}", path.display()).into());
    }
    atlas_runtime::initialize_workspace(path)?;
    Ok(())
}
