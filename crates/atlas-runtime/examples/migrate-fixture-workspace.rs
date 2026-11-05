use std::{error::Error, path::PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let [path] = args.as_slice() else {
        return Err("usage: migrate-fixture-workspace EXISTING_DATABASE".into());
    };
    let path = PathBuf::from(path);
    if !path.is_file() {
        return Err(format!("Fixture database does not exist: {}", path.display()).into());
    }
    atlas_runtime::initialize_workspace(path)?;
    Ok(())
}
