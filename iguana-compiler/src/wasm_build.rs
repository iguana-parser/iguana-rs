use std::{io, path::Path, process::Command};

/// Build a generated wasm bundle's `wasm/` wrapper crate with wasm-pack.
///
/// Probes for wasm-pack and the `wasm32-unknown-unknown` target first,
/// reporting an actionable install hint when either is missing, then runs
/// `wasm-pack build --target web --out-name parser` in `wasm_dir`. wasm-pack
/// drives cargo build, wasm-bindgen, and wasm-opt, and pins a wasm-bindgen CLI
/// matching the crate, so the version skew of invoking those tools by hand does
/// not arise.
///
/// The output lands in `<wasm_dir>/pkg/` as `parser.js` and `parser_bg.wasm`,
/// the fixed module name the grammar-independent viewer loads.
pub fn build(wasm_dir: &Path) -> io::Result<()> {
    let have_wasm_pack = Command::new("wasm-pack")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    if !have_wasm_pack {
        return Err(io::Error::other(
            "wasm-pack not found. Install it with `cargo install wasm-pack`.",
        ));
    }

    // No rustup means we can't check the target; let wasm-pack report it.
    let have_target = Command::new("rustup")
        .args(["target", "list", "--installed"])
        .output()
        .map(|o| {
            o.status.success()
                && String::from_utf8_lossy(&o.stdout).contains("wasm32-unknown-unknown")
        })
        .unwrap_or(true);
    if !have_target {
        return Err(io::Error::other(
            "wasm32-unknown-unknown target not installed. \
             Add it with `rustup target add wasm32-unknown-unknown`.",
        ));
    }

    println!("Building wasm module with wasm-pack (target web)...");
    let status = Command::new("wasm-pack")
        .current_dir(wasm_dir)
        .args(["build", "--target", "web", "--out-name", "parser"])
        .status()?;
    if !status.success() {
        return Err(io::Error::other("wasm-pack build failed"));
    }
    Ok(())
}
