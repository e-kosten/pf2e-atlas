use sha2::{Digest, Sha256};
use std::{fs, path::Path};

fn main() {
    if let Err(error) = build() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn build() -> Result<(), Box<dyn std::error::Error>> {
    let root = Path::new("../..");
    let manifest = Path::new("api-contract-inputs.json");
    println!("cargo:rerun-if-changed={}", manifest.display());
    let inputs: Vec<String> = serde_json::from_slice(&fs::read(manifest)?)?;
    let mut files = Vec::new();
    for input in inputs {
        collect(root, &root.join(input), &mut files)?;
    }
    files.sort();
    let mut hash = Sha256::new();
    for file in files {
        let path = root.join(&file);
        println!("cargo:rerun-if-changed={}", path.display());
        hash.update(file.as_bytes());
        hash.update([0]);
        hash.update(fs::read_to_string(path)?.replace("\r\n", "\n").as_bytes());
        hash.update([0]);
    }
    let contract = format!("atlas-web-v1-{:x}", hash.finalize());
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

fn collect(root: &Path, path: &Path, files: &mut Vec<String>) -> std::io::Result<()> {
    if path.is_dir() {
        println!("cargo:rerun-if-changed={}", path.display());
        for entry in fs::read_dir(path)? {
            collect(root, &entry?.path(), files)?;
        }
    } else if path
        .extension()
        .is_some_and(|ext| ext == "rs" || ext == "ts")
    {
        files.push(
            path.strip_prefix(root)
                .map_err(std::io::Error::other)?
                .components()
                .map(|part| part.as_os_str().to_string_lossy())
                .collect::<Vec<_>>()
                .join("/"),
        );
    }
    Ok(())
}
