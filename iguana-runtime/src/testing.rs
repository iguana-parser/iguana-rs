use std::path::{Path, PathBuf};

pub fn golden_path(manifest_dir: &str, name: &str) -> PathBuf {
    let mut path = PathBuf::from(manifest_dir);
    path.push("parse_trees");
    path.push(format!("{}.txt", name));
    path
}

pub fn check_golden_file(actual: &str, golden_path: &Path) {
    let regenerate = std::env::var("REGENERATE").is_ok();

    if regenerate {
        std::fs::write(golden_path, actual).expect("Failed to write golden file");
        println!("Updated: {}", golden_path.display());
    } else {
        let expected = std::fs::read_to_string(golden_path).unwrap_or_else(|_| {
            panic!(
                "Golden file not found: {}\nRun with REGENERATE=1 to create it",
                golden_path.display()
            )
        });

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
