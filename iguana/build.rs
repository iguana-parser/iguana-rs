use std::{env, path::Path};

// The binary embeds `web-viewer/dist` via `include_dir!`, which is a hard
// compile error if the directory is missing. The dist is committed, so this
// only trips if it was deleted; check it here for an actionable message rather
// than a macro error.
fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is set");
    let dist = Path::new(&manifest_dir).join("../web-viewer/dist");
    if !dist.join("index.html").exists() {
        panic!(
            "web-viewer/dist is missing. Rebuild it with `cargo xtask viewer` \
             (or `npm --prefix web-viewer install && npm run build --workspace web-viewer`)."
        );
    }
    println!("cargo:rerun-if-changed={}", dist.display());
}
