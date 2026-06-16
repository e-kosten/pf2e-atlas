use std::path::Path;

fn main() {
    let dist = Path::new("../../web/atlas-ui/dist");
    let index = dist.join("index.html");

    println!("cargo:rerun-if-changed={}", dist.display());

    if !index.is_file() {
        eprintln!(
            "missing built Atlas web UI at {}; run `just web-ui-build` before compiling atlas-web",
            index.display()
        );
        std::process::exit(1);
    }
}
