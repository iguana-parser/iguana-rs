use std::{env, path::Path};

// The binary embeds `viewer-dist` via `include_dir!`, which is a hard
// compile error if the directory is missing. The dist is committed and lives
// inside this crate so `cargo package` ships it and a registry build works
// without npm. It only goes missing if deleted; check it here for an
// actionable message rather than a macro error.
fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is set");
    let dist = Path::new(&manifest_dir).join("viewer-dist");
    if !dist.join("index.html").exists() {
        panic!(
            "iguana/viewer-dist is missing. Rebuild it with \
             `npm run build --workspace web-viewer` (run `npm install` in the \
             repo root first if the viewer dependencies are not installed)."
        );
    }
    println!("cargo:rerun-if-changed={}", dist.display());
}
