use std::{
    ffi::OsStr,
    io,
    path::Path,
    process::{Command, Stdio},
    sync::Once,
    time::{Duration, Instant},
};

static MISSING_RUSTFMT_WARNING: Once = Once::new();

/// Formats the existing `files` with rustfmt and returns the elapsed time.
///
/// Missing paths are ignored. An unavailable rustfmt warns once and leaves the
/// files unchanged. A formatter failure includes stderr in the returned error.
pub fn format_files(files: &[&Path]) -> io::Result<Duration> {
    format_files_with(OsStr::new("rustfmt"), files)
}

fn format_files_with(program: &OsStr, files: &[&Path]) -> io::Result<Duration> {
    let files: Vec<&Path> = files.iter().copied().filter(|f| f.exists()).collect();
    if files.is_empty() {
        return Ok(Duration::ZERO);
    }
    if !is_available(program) {
        MISSING_RUSTFMT_WARNING.call_once(|| {
            eprintln!(
                "Warning: rustfmt is not available, leaving the generated sources unformatted."
            );
        });
        return Ok(Duration::ZERO);
    }

    let start = Instant::now();
    // Captured, so what rustfmt reports arrives in the error rather than on
    // the caller's terminal.
    let output = Command::new(program)
        .arg("--edition")
        .arg("2024")
        .arg("--quiet")
        .args(&files)
        .output()?;
    if !output.status.success() {
        let message = String::from_utf8_lossy(&output.stderr);
        return Err(io::Error::other(format!(
            "rustfmt failed on the generated sources: {}",
            message.trim()
        )));
    }
    Ok(start.elapsed())
}

/// Whether rustfmt can run. Rustup installs a proxy that starts even when the
/// component is missing from the active toolchain, so the exit status decides,
/// not the spawn.
fn is_available(program: &OsStr) -> bool {
    Command::new(program)
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn unformatted_file(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(name);
        fs::create_dir_all(&dir).unwrap();
        let file = dir.join("lib.rs");
        fs::write(&file, "fn main ( ) { }").unwrap();
        file
    }

    #[test]
    fn a_program_that_cannot_be_spawned_counts_as_missing() {
        let file = unformatted_file("iguana-format-missing");

        let elapsed = format_files_with(OsStr::new("rustfmt-that-does-not-exist"), &[&file])
            .expect("a missing formatter leaves the sources as they are");
        assert_eq!(elapsed, Duration::ZERO);
        assert_eq!(fs::read_to_string(&file).unwrap(), "fn main ( ) { }");
    }

    /// Rustup's proxy spawns even when the component is missing and then fails.
    /// The test binary stands in for it: it starts, and it rejects `--version`.
    #[test]
    fn a_program_that_spawns_but_fails_its_version_probe_counts_as_missing() {
        let file = unformatted_file("iguana-format-proxy");
        let program = std::env::current_exe().unwrap();

        let elapsed = format_files_with(program.as_os_str(), &[&file])
            .expect("a formatter that reports no version is treated as absent");
        assert_eq!(elapsed, Duration::ZERO);
        assert_eq!(fs::read_to_string(&file).unwrap(), "fn main ( ) { }");
    }

    #[test]
    fn a_file_rustfmt_cannot_parse_is_an_error() {
        if !is_available(OsStr::new("rustfmt")) {
            return;
        }
        let dir = std::env::temp_dir().join("iguana-format-invalid");
        fs::create_dir_all(&dir).unwrap();
        let file = dir.join("lib.rs");
        fs::write(&file, "fn main( {").unwrap();

        assert!(format_files_with(OsStr::new("rustfmt"), &[&file]).is_err());
    }

    #[test]
    fn formatting_rewrites_a_generated_file() {
        if !is_available(OsStr::new("rustfmt")) {
            return;
        }
        let file = unformatted_file("iguana-format-ok");

        format_files_with(OsStr::new("rustfmt"), &[&file]).expect("rustfmt should format the file");
        assert_eq!(fs::read_to_string(&file).unwrap(), "fn main() {}\n");
    }
}
