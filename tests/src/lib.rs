pub mod grammars;

use std::path::Path;

/// Check a parse tree against a golden file.
/// If REGENERATE env var is set, update the golden file instead.
pub fn check_parse_tree(actual: &str, golden_path: &Path) {
    let regenerate = std::env::var("REGENERATE").is_ok();

    if regenerate {
        std::fs::write(golden_path, actual).expect("Failed to write golden file");
        println!("Updated: {}", golden_path.display());
    } else {
        let expected = std::fs::read_to_string(golden_path)
            .unwrap_or_else(|_| panic!("Golden file not found: {}\nRun with REGENERATE=1 to create it", golden_path.display()));

        if actual != expected {
            panic!(
                "Parse tree mismatch for {}\n\nExpected:\n{}\n\nActual:\n{}\n\nRun with REGENERATE=1 to update",
                golden_path.display(),
                expected,
                actual
            );
        }
    }
}
