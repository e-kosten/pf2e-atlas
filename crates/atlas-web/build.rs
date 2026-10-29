mod build_contract;

use std::{fs, path::Path};

fn main() {
    if let Err(error) = build() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn build() -> Result<(), Box<dyn std::error::Error>> {
    let root = Path::new("../..");
    println!("cargo:rerun-if-changed=build_contract.rs");
    let contract = build_contract::load(root)?;
    for path in &contract.watched_paths {
        println!("cargo:rerun-if-changed={}", path.display());
    }
    let contract = contract.identifier;
    println!("cargo:rustc-env=ATLAS_API_CONTRACT={contract}");
    let dist = Path::new("../../web/atlas-ui/dist");
    let index = dist.join("index.html");

    println!("cargo:rerun-if-changed={}", dist.display());

    if !index.is_file() {
        eprintln!(
            "missing built Atlas web UI at {}; run `just web-ui-build` before compiling atlas-web",
            index.display()
        );
        return Err("Build the frontend before compiling atlas-web".into());
    }
    let frontend = fs::read_to_string(dist.join("api-contract.txt"))
        .map_err(|error| format!("Missing built frontend API identifier ({error}); run `just web-ui-build` before compiling atlas-web"))?;
    if frontend.trim() != contract {
        return Err("Built Atlas frontend uses another API contract; run `just web-ui-build` before compiling atlas-web".into());
    }
    Ok(())
}
