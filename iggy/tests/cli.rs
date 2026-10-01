use std::{
    env, fs,
    path::PathBuf,
    process::{self, Command, Output},
    time::{SystemTime, UNIX_EPOCH},
};

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_iggy"))
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .args(args)
        .output()
        .unwrap()
}

const PARSING_MODES: &[&[&str]] = &[
    &["iggy.iggy"],
    &["--dir", "."],
    &["--repl"],
    &["--benchmark", "iggy.iggy"],
    &["--benchmark", "--dir", "."],
    &["--check-sexpr", "iggy.iggy"],
    &["--regenerate-sexpr", "iggy.iggy"],
];

#[test]
fn missing_start_is_a_readable_usage_error() {
    for args in PARSING_MODES {
        let output = run(args);
        assert_eq!(output.status.code(), Some(1), "{args:?}");
        assert!(output.stdout.is_empty(), "{args:?}");
        assert_eq!(
            String::from_utf8(output.stderr).unwrap(),
            "Error: --start is required for parsing\n",
            "{args:?}"
        );
    }
}

#[test]
fn unknown_start_points_to_list_nonterminals() {
    let listing = run(&["--list-nonterminals"]);
    assert!(listing.status.success());
    assert!(listing.stderr.is_empty());
    let names = String::from_utf8(listing.stdout).unwrap();
    assert!(names.starts_with("Grammar\nRule\n"));
    assert!(!names.contains("StartGrammar"));

    // The error points to --list-nonterminals for both a typo and a generated wrapper name.
    for name in ["Gramar", "StartGrammar"] {
        for mode in PARSING_MODES {
            let mut args = mode.to_vec();
            args.extend(["--start", name]);
            let output = run(&args);
            assert_eq!(output.status.code(), Some(1), "{args:?}");
            assert!(output.stdout.is_empty(), "{args:?}");
            assert_eq!(
                String::from_utf8(output.stderr).unwrap(),
                format!(
                    "Error: Unknown start nonterminal: '{name}'. Use --list-nonterminals to see the valid names.\n"
                ),
                "{args:?}"
            );
        }
    }
}

// 'é' takes two bytes, so the size of this grammar in bytes differs from its
// length in characters.
const VALID_GRAMMAR: &str = "grammar G\n\n// é\nS = \"a\"\n";
const INVALID_GRAMMAR: &str = "grammar G\n\nS = = \"a\"\n";
// These bytes are not valid UTF-8, so a file that holds them cannot be read.
const INVALID_UTF8: [u8; 2] = [0xff, 0xfe];

/// A directory with a valid, an invalid, and an unreadable grammar file for
/// a benchmark test. Dropping the value removes the directory, so a test
/// that fails also cleans up.
struct BenchmarkDir(PathBuf);

impl BenchmarkDir {
    fn new(name: &str) -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = env::temp_dir().join(format!("iggy-{name}-{}-{nonce}", process::id()));
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("valid.iggy"), VALID_GRAMMAR).unwrap();
        fs::write(dir.join("invalid.iggy"), INVALID_GRAMMAR).unwrap();
        fs::write(dir.join("unreadable.iggy"), INVALID_UTF8).unwrap();
        Self(dir)
    }

    fn path(&self) -> &str {
        self.0.to_str().unwrap()
    }

    /// Returns the path of the file `name` in the directory.
    fn file(&self, name: &str) -> String {
        self.0.join(name).to_str().unwrap().to_string()
    }
}

impl Drop for BenchmarkDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// Runs a benchmark of two iterations with the start nonterminal of the
/// test grammars.
fn benchmark(args: &[&str]) -> Output {
    let mut all = vec!["--benchmark", "--start", "Grammar", "--iters", "2"];
    all.extend(args);
    run(&all)
}

/// Returns the standard output of a run that must succeed. The CLI writes
/// its errors to the standard error, so a failed check shows both.
fn stdout_of_success(output: Output) -> String {
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        output.status.success(),
        "stdout:\n{stdout}\nstderr:\n{stderr}"
    );
    stdout
}

/// Returns the standard error of a run that must fail with exit code 1.
fn stderr_of_failure(output: Output) -> String {
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert_eq!(
        output.status.code(),
        Some(1),
        "stdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(stdout.is_empty(), "stdout:\n{stdout}\nstderr:\n{stderr}");
    stderr
}

fn read_samples(path: &str) -> serde_json::Value {
    serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap()
}

#[test]
fn batch_benchmark_reports_failures_apart_from_successes() {
    let dir = BenchmarkDir::new("batch-benchmark");
    let saved = dir.file("samples.json");

    let stdout = stdout_of_success(benchmark(&[
        "--dir",
        dir.path(),
        "--ext",
        "iggy",
        "--save",
        &saved,
    ]));
    assert!(
        stdout.contains(&format!(
            "Success: 1 file, {} bytes per iteration",
            VALID_GRAMMAR.len()
        )),
        "{stdout}"
    );
    assert!(
        stdout.contains(&format!(
            "Failure: 2 files, {} bytes per iteration",
            INVALID_GRAMMAR.len() + INVALID_UTF8.len()
        )),
        "{stdout}"
    );
    assert!(stdout.contains("All files (median): "), "{stdout}");
    assert!(stdout.contains(" for 3 files"), "{stdout}");

    let samples = read_samples(&saved);
    assert_eq!(samples["version"], 3);
    assert!(samples.get("status").is_none());
    assert_eq!(samples["files"], 1);
    assert_eq!(samples["bytes"], VALID_GRAMMAR.len());
    assert_eq!(samples["samples_ms"].as_array().unwrap().len(), 2);
    for phase in ["input", "init", "parse", "tree", "drop", "error", "amb"] {
        let key = format!("{phase}_samples_ms");
        assert_eq!(samples[&key].as_array().unwrap().len(), 2, "{key}");
    }
    assert_eq!(samples["error_files"], 1);
    assert_eq!(samples["amb_files"], 0);
    assert_eq!(samples["ioerror_files"], 1);
}

#[test]
fn single_benchmark_reports_the_phases_of_a_parsed_file() {
    let dir = BenchmarkDir::new("single-benchmark-parsed");
    let saved = dir.file("samples.json");

    let stdout = stdout_of_success(benchmark(&[&dir.file("valid.iggy"), "--save", &saved]));
    assert!(
        stdout.contains(&format!(
            "{} bytes per iteration (times in ms)",
            VALID_GRAMMAR.len()
        )),
        "{stdout}"
    );
    assert!(stdout.contains("\n  parse "), "{stdout}");
    assert!(stdout.contains("Throughput (median): "), "{stdout}");
    // A single file has no status line and no group.
    assert!(!stdout.contains("Status: "), "{stdout}");
    assert!(!stdout.contains("Success: "), "{stdout}");

    let samples = read_samples(&saved);
    assert_eq!(samples["status"], "ok");
    assert_eq!(samples["bytes"], VALID_GRAMMAR.len());
}

#[test]
fn single_benchmark_reports_the_status_of_a_parse_error() {
    let dir = BenchmarkDir::new("single-benchmark-error");
    let saved = dir.file("samples.json");

    let stdout = stdout_of_success(benchmark(&[&dir.file("invalid.iggy"), "--save", &saved]));
    assert!(
        stdout.contains(&format!(
            "{} bytes per iteration (times in ms)",
            INVALID_GRAMMAR.len()
        )),
        "{stdout}"
    );
    assert!(stdout.contains("Status: error\n"), "{stdout}");
    assert!(stdout.contains("\n  error "), "{stdout}");
    assert!(!stdout.contains("Throughput (median): "), "{stdout}");
    assert!(!stdout.contains("Failure: "), "{stdout}");

    let samples = read_samples(&saved);
    assert_eq!(samples["status"], "error");
    assert_eq!(samples["bytes"], INVALID_GRAMMAR.len());
    assert_eq!(samples["samples_ms"], samples["error_samples_ms"]);
}

#[test]
fn benchmark_baseline_must_have_the_status_of_the_run() {
    let dir = BenchmarkDir::new("benchmark-baseline");
    let valid_samples = dir.file("valid.json");
    let invalid_samples = dir.file("invalid.json");
    stdout_of_success(benchmark(&[
        &dir.file("valid.iggy"),
        "--save",
        &valid_samples,
    ]));
    stdout_of_success(benchmark(&[
        &dir.file("invalid.iggy"),
        "--save",
        &invalid_samples,
    ]));

    let stdout = stdout_of_success(benchmark(&[
        &dir.file("invalid.iggy"),
        "--baseline",
        &invalid_samples,
    ]));
    assert!(stdout.contains("Compared to baseline "), "{stdout}");

    let output = benchmark(&[&dir.file("invalid.iggy"), "--baseline", &valid_samples]);
    // The run prints its own result before it reads the baseline.
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("has the status ok, and this run has the status error"),
        "{stderr}"
    );
}

#[test]
fn single_benchmark_rejects_a_file_that_cannot_be_read() {
    let dir = BenchmarkDir::new("single-benchmark-unreadable");

    let stderr = stderr_of_failure(benchmark(&[&dir.file("unreadable.iggy")]));
    assert!(
        stderr.ends_with("unreadable.iggy: stream did not contain valid UTF-8\n"),
        "{stderr}"
    );
}
