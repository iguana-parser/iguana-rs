use std::{
    collections::{BTreeMap, BTreeSet},
    fs, io,
    path::{Path, PathBuf},
    process::Command,
};

use crate::{
    arena::Arena,
    input::Input,
    parse_tree::is_ambiguous,
    parser::{GLLResult, Parser},
};

use super::{Color, args::Args, collect_files, print_status, start_nonterminal_id};

/// Runs `--corpus-test`: checks every corpus listed in `repos.txt`, or the
/// named one, against its committed baseline, or rewrites the baselines under
/// `--update`. Exits with failure when a corpus did not pass.
pub(super) fn run<'i, 'arena, P: Parser<'i, 'arena>>(args: &Args) -> io::Result<()> {
    let only = args
        .corpus_test
        .as_ref()
        .expect("--corpus-test is set")
        .as_deref();
    if args.start_nonterminal.is_some() {
        eprintln!(
            "Note: --start is ignored with --corpus-test; the start nonterminal comes from repos.txt."
        );
    }
    let mode = if args.update {
        CorpusMode::Update
    } else {
        CorpusMode::Check
    };
    let repos_path = args.corpus_dir.join("repos.txt");
    if init_corpus_dir(&args.corpus_dir)? {
        println!(
            "Created {}. List your repos there and re-run.",
            repos_path.display()
        );
        return Ok(());
    }
    let entries = read_repos(&repos_path)?;
    if entries.is_empty() {
        eprintln!("Warning: no repos listed in {}", repos_path.display());
    }
    let mut ran = 0usize;
    let mut passed = 0usize;
    let mut parser_arena = Arena::new();
    for entry in &entries {
        if let Some(name) = only {
            if entry.name != name {
                continue;
            }
        }
        ran += 1;
        let start_nonterminal_id = start_nonterminal_id::<P::Grammar>(&entry.start)?;
        let checkout = args.corpus_dir.join(".cache").join(&entry.name);
        if let Err(e) = fetch_corpus(&checkout, &entry.repo, &entry.git_ref) {
            eprintln!("Corpus '{}': {}", entry.name, e);
            continue;
        }
        let mut inputs = Vec::new();
        collect_files(&checkout, Some(&entry.ext), &mut inputs)?;
        inputs.sort();
        let baseline_path = args.corpus_dir.join(format!("{}.txt", entry.name));
        let report = run_corpus(
            &entry.name,
            inputs,
            &checkout,
            &baseline_path,
            CorpusConfig {
                mode,
                quiet: args.quiet,
            },
            |path| {
                let input = match Input::try_from(path) {
                    Ok(input) => input,
                    Err(e) => {
                        return CorpusOutcome::IoError {
                            message: e.to_string(),
                        };
                    }
                };
                let mut parser = P::ConcreteParser::<'_, '_>::new(&input, &parser_arena);
                let outcome = match parser.run(start_nonterminal_id) {
                    GLLResult::Success(success) => CorpusOutcome::Ok {
                        ambiguous: is_ambiguous(&parser, success.sppf_node_id),
                    },
                    GLLResult::Failure(error) => {
                        let error = parser.to_parse_error(&error);
                        let (line, column) = input.line_column(error.span.left_extent);
                        CorpusOutcome::Error {
                            message: format!(
                                "Parse error at line {}, column {}: {}",
                                line + 1,
                                column + 1,
                                error.message
                            ),
                        }
                    }
                };
                drop(parser);
                parser_arena.reset();
                outcome
            },
        )?;
        if report.passed {
            passed += 1;
        }
    }
    if let Some(name) = only {
        if ran == 0 {
            eprintln!("No corpus named '{}' in {}", name, repos_path.display());
            std::process::exit(1);
        }
    }
    if !args.quiet {
        let noun = if ran == 1 { "repo" } else { "repos" };
        println!();
        match mode {
            CorpusMode::Check => println!(
                "{} {} checked: {} passed, {} failed",
                ran,
                noun,
                passed,
                ran - passed
            ),
            CorpusMode::Update => println!(
                "{} {} updated: {} written, {} refused",
                ran,
                noun,
                passed,
                ran - passed
            ),
        }
    }
    if passed != ran {
        std::process::exit(1);
    }
    Ok(())
}

/// One line of `repos.txt`: a corpus to fetch and parse. The parser reads
/// `name`, `ext`, and `start`; `repo`/`git_ref` are consumed by the fetch step
/// and kept as provenance.
pub(super) struct CorpusEntry {
    pub(super) name: String,
    pub(super) ext: String,
    pub(super) start: String,
    pub(super) repo: String,
    pub(super) git_ref: String,
}

/// Reads `repos.txt`: one corpus per line as whitespace-separated
/// `name ext start repo ref`. Blank lines and `#` comments are skipped.
pub(super) fn read_repos(path: &Path) -> io::Result<Vec<CorpusEntry>> {
    let text = fs::read_to_string(path)?;
    parse_repos_text(&text).map_err(|msg| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("{}: {msg}", path.display()),
        )
    })
}

fn parse_repos_text(text: &str) -> Result<Vec<CorpusEntry>, String> {
    let mut entries = Vec::new();
    for (i, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() != 5 {
            return Err(format!(
                "line {}: expected `name ext start repo ref`, found {} fields",
                i + 1,
                fields.len()
            ));
        }
        entries.push(CorpusEntry {
            name: fields[0].to_string(),
            ext: fields[1].to_string(),
            start: fields[2].to_string(),
            repo: fields[3].to_string(),
            git_ref: fields[4].to_string(),
        });
    }
    Ok(entries)
}

/// Scaffolds the corpus directory on first use: creates `dir`, writes a
/// `.gitignore` for the `.cache/` checkouts when absent, and (when `repos.txt`
/// is missing) writes a commented template. Returns `true` when it just created
/// `repos.txt`, so the caller can stop and let the user fill it in.
pub(super) fn init_corpus_dir(dir: &Path) -> io::Result<bool> {
    fs::create_dir_all(dir)?;

    let gitignore = dir.join(".gitignore");
    if !gitignore.exists() {
        fs::write(gitignore, ".cache/\n")?;
    }

    let repos = dir.join("repos.txt");
    if repos.exists() {
        return Ok(false);
    }
    fs::write(repos, REPOS_TEMPLATE)?;
    Ok(true)
}

const REPOS_TEMPLATE: &str = concat!(
    "# Repos to parse, one per line: name ext start repo ref\n",
    "#   name  = baseline file (<name>.txt) + checkout dir (.cache/<name>)\n",
    "#   ext   = file extension to parse        start = start nonterminal\n",
    "#   repo  = git URL                        ref   = tag or branch (shallow-cloned)\n",
    "#\n",
    "# myproj java CompilationUnit https://github.com/owner/repo v1.0.0\n",
);

/// Ensures `repo` is checked out at `git_ref` in `dir`, shallow-cloning it when
/// the directory is absent. A present checkout is left as-is: refs are pinned
/// and immutable, so to repin you delete the directory and re-run. Errors if
/// `git` is not on PATH or the clone fails. `git_ref` must name a branch or tag
/// (a bare commit SHA is not valid for a shallow `--branch` clone).
pub(super) fn fetch_corpus(dir: &Path, repo: &str, git_ref: &str) -> io::Result<()> {
    if dir.exists() {
        return Ok(());
    }
    eprintln!("Cloning {repo} @ {git_ref} into {}", dir.display());
    let status = Command::new("git")
        .args([
            "clone",
            "-c",
            "advice.detachedHead=false",
            "--depth",
            "1",
            "--branch",
            git_ref,
            repo,
        ])
        .arg(dir)
        .status()
        .map_err(|e| io::Error::new(e.kind(), format!("failed to run git: {e}")))?;
    if !status.success() {
        return Err(io::Error::other(format!(
            "git clone of {repo} @ {git_ref} failed"
        )));
    }
    Ok(())
}

/// Whether the corpus harness checks results against the committed baseline or
/// rewrites it. A rewrite is refused when the new state would regress
/// (ok -> error/ioerror) or is ambiguous, so an update cannot record either.
#[derive(Clone, Copy, PartialEq, Eq)]
enum CorpusMode {
    Check,
    Update,
}

/// Knobs for a `run_corpus` run: check vs. rewrite, and whether to suppress the
/// per-corpus status line.
struct CorpusConfig {
    mode: CorpusMode,
    quiet: bool,
}

/// The outcome of parsing one corpus file. `ambiguous` marks a success with more
/// than one derivation; it records under the `amb` baseline status, which the
/// check treats exactly like an error.
enum CorpusOutcome {
    Ok { ambiguous: bool },
    Error { message: String },
    IoError { message: String },
}

/// Summary of one corpus run, returned so the caller can aggregate across
/// corpora and set the exit code. `passed` is false when a file regresses: it
/// parsed cleanly in the baseline and now fails, a newly ambiguous parse
/// included. On an `Update` this also means the baseline was left untouched,
/// since a rewrite is refused rather than record a regression.
struct CorpusReport {
    files: usize,
    ok: usize,
    ambiguous: usize,
    error: usize,
    ioerror: usize,
    passed: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Status {
    Ok,
    Ambiguous,
    Error,
    IoError,
}

/// One file's outcome as just parsed. `message` is already tab-sanitized.
struct CurrentFile {
    path: String,
    status: Status,
    message: Option<String>,
}

/// One per-file line read back from a baseline. `message` is present only for
/// `error`/`ioerror`.
struct BaselineRecord {
    status: Status,
    message: Option<String>,
}

/// Aggregate counts from a baseline header's `# files=…` line.
struct BaselineTotals {
    files: usize,
    ok: usize,
    ambiguous: usize,
    error: usize,
    ioerror: usize,
}

/// Runs corpus regression testing for one corpus. `parse_one` parses a single
/// file and reports `ok`/`error`/`ioerror`; `run_corpus` owns building the
/// per-file records, then either writing the baseline (`Update`) or comparing
/// against it (`Check`).
///
/// `root` is the corpus checkout; per-file paths are recorded relative to it,
/// normalized to `/`, and sorted, so the committed baseline is stable. The
/// baseline records correctness only (status and message), no timing, so a
/// re-run reproduces it byte for byte unless a parse outcome changed. Perf is
/// measured separately with `--benchmark`.
///
/// Returns a `CorpusReport`; on a `Check`, `passed` is false when a file
/// regresses: it parsed cleanly in the baseline (`ok`) and now fails with
/// `amb`, `error`, or `ioerror`. Recoveries (`fail -> ok`), message/status
/// drift between failing states, and added/removed files are reported but do
/// not fail the run, so a red check always means a genuine regression.
/// Ambiguity is diffed against the baseline like an error: a parse with more
/// than one derivation records under the `amb` status, so a file that is
/// ambiguous in both the baseline and the run is a known state, not a failure,
/// while a fresh `ok -> amb` fails. An `Update` applies the same rule: it
/// rewrites the baseline only when the run holds no regression, and otherwise
/// fails without writing.
fn run_corpus(
    name: &str,
    inputs: Vec<PathBuf>,
    root: &Path,
    baseline_path: &Path,
    config: CorpusConfig,
    mut parse_one: impl FnMut(&Path) -> CorpusOutcome,
) -> io::Result<CorpusReport> {
    let CorpusConfig { mode, quiet } = config;
    let color = Color::for_stdout();

    // Parse every file into a current record.
    let mut files: Vec<CurrentFile> = Vec::with_capacity(inputs.len());
    for input in &inputs {
        let path = normalize_rel(input, root);
        let (status, message) = match parse_one(input) {
            CorpusOutcome::Ok {
                ambiguous: is_ambig,
            } => {
                let status = if is_ambig {
                    Status::Ambiguous
                } else {
                    Status::Ok
                };
                (status, None)
            }
            CorpusOutcome::Error { message } => (Status::Error, Some(sanitize(&message))),
            CorpusOutcome::IoError { message } => (Status::IoError, Some(sanitize(&message))),
        };
        files.push(CurrentFile {
            path,
            status,
            message,
        });
    }
    files.sort_by(|a, b| a.path.cmp(&b.path));

    let ok = files.iter().filter(|f| f.status == Status::Ok).count();
    let ambiguous = files
        .iter()
        .filter(|f| f.status == Status::Ambiguous)
        .count();
    let error = files.iter().filter(|f| f.status == Status::Error).count();
    let ioerror = files.iter().filter(|f| f.status == Status::IoError).count();
    let report = CorpusReport {
        files: files.len(),
        ok,
        ambiguous,
        error,
        ioerror,
        passed: true,
    };

    match mode {
        CorpusMode::Update => {
            // An update must not bake a regression into the committed baseline, so
            // hold the rewrite to the same condition as a Check: no file that
            // parsed cleanly in the baseline may now fail, a new ambiguity
            // included. A missing baseline is fine on a first run, since there is
            // nothing to regress from.
            let baseline = match read_baseline(baseline_path) {
                Ok((_, baseline)) => baseline,
                Err(e) if e.kind() == io::ErrorKind::NotFound => BTreeMap::new(),
                Err(e) => return Err(e),
            };
            let diffs = diff_records(&files, &baseline);
            let passed = diffs.regressions.is_empty();
            if !passed {
                // A refusal prints even under --quiet: it is the point of the run.
                print_status(&color, "FAIL", false, &corpus_counts(name, &report));
                print_diffs(&color, &diffs);
                println!("  baseline left unchanged (an update records no regression)");
                return Ok(CorpusReport {
                    passed: false,
                    ..report
                });
            }
            fs::write(baseline_path, serialize_baseline(name, &files))?;
            if !quiet {
                print_status(&color, "WRITE", true, &corpus_counts(name, &report));
            }
            Ok(report)
        }
        CorpusMode::Check => {
            let (totals, baseline) = match read_baseline(baseline_path) {
                Ok(baseline) => baseline,
                Err(e) if e.kind() == io::ErrorKind::NotFound => {
                    print_status(
                        &color,
                        "FAIL",
                        false,
                        &format!(
                            "{name}: no baseline at {} (run --corpus-test --update)",
                            baseline_path.display()
                        ),
                    );
                    return Ok(CorpusReport {
                        passed: false,
                        ..report
                    });
                }
                Err(e) => return Err(e),
            };

            // Compare every file against its baseline record. A regression
            // (a file that parsed cleanly in the baseline now fails, a new
            // ambiguity included) fails the check; recoveries, drift, and
            // added/removed are soft.
            let diffs = diff_records(&files, &baseline);
            let passed = diffs.regressions.is_empty();

            if !passed {
                // A failure prints even under --quiet: it is the point of the run.
                print_status(&color, "FAIL", false, &corpus_counts(name, &report));
                print_baseline_counts(&totals);
                print_diffs(&color, &diffs);
            } else if !quiet {
                if diffs.is_clean() {
                    print_status(&color, "PASS", true, &corpus_counts(name, &report));
                } else {
                    // No regression, but the baseline does not match this run.
                    print_status(&color, "DRIFT", true, &corpus_counts(name, &report));
                    print_baseline_counts(&totals);
                    print_diffs(&color, &diffs);
                    println!(
                        "  no regressions; run `--corpus-test --update` to refresh the baseline"
                    );
                }
            }

            Ok(CorpusReport { passed, ..report })
        }
    }
}

/// `"<name>: N files (A ok, B ambiguous, C error, D ioerror)"`, the per-corpus
/// status tail.
fn corpus_counts(name: &str, report: &CorpusReport) -> String {
    format!(
        "{name}: {} files ({} ok, {} ambiguous, {} error, {} ioerror)",
        report.files, report.ok, report.ambiguous, report.error, report.ioerror
    )
}

/// Strips `root` and normalizes separators to `/` so the recorded path is the
/// same on every platform.
fn normalize_rel(path: &Path, root: &Path) -> String {
    let rel = path.strip_prefix(root).unwrap_or(path);
    rel.to_string_lossy().replace('\\', "/")
}

/// Collapses tabs and newlines in a value to spaces, so it never splits a record
/// or spills onto the next line.
fn sanitize(s: &str) -> String {
    s.replace(['\t', '\n', '\r'], " ")
}

/// Renders the baseline: the `# corpus`/`# files=…` header, then one sorted line
/// per file. Lines carry status and message only, no timing, so the committed
/// file stays stable across runs.
fn serialize_baseline(name: &str, files: &[CurrentFile]) -> String {
    let ok = files.iter().filter(|f| f.status == Status::Ok).count();
    let ambiguous = files
        .iter()
        .filter(|f| f.status == Status::Ambiguous)
        .count();
    let error = files.iter().filter(|f| f.status == Status::Error).count();
    let ioerror = files.iter().filter(|f| f.status == Status::IoError).count();

    let mut out = String::new();
    out.push_str(&format!("# corpus: {name}\n"));
    out.push_str(&format!(
        "# files={} ok={ok} amb={ambiguous} error={error} ioerror={ioerror}\n",
        files.len()
    ));
    // Per-file lines carry status only, and a message for the two failing states.
    // No time is recorded: a file's parse time drifts run to run and would churn
    // the committed baseline on every update. Perf is measured with `--benchmark`.
    for f in files {
        match f.status {
            Status::Ok => out.push_str(&format!("{}\tok\n", f.path)),
            Status::Ambiguous => out.push_str(&format!("{}\tamb\n", f.path)),
            Status::Error => {
                let message = f.message.as_deref().unwrap_or("");
                out.push_str(&format!("{}\terror\t{message}\n", f.path));
            }
            Status::IoError => {
                let message = f.message.as_deref().unwrap_or("");
                out.push_str(&format!("{}\tioerror\t{message}\n", f.path));
            }
        }
    }
    out
}

/// Reads a baseline into its header totals and a path-keyed map of per-file
/// records.
fn read_baseline(path: &Path) -> io::Result<(BaselineTotals, BTreeMap<String, BaselineRecord>)> {
    let text = fs::read_to_string(path)?;
    let mut totals = None;
    let mut records = BTreeMap::new();
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("# ") {
            if rest.starts_with("files=") {
                totals = Some(parse_totals(rest)?);
            }
            continue;
        }
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }
        if let Some((path, record)) = parse_record_line(line) {
            records.insert(path, record);
        }
    }
    let totals = totals.ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("{}: missing totals header", path.display()),
        )
    })?;
    Ok((totals, records))
}

/// Parses `files=N ok=N amb=N error=N ioerror=N` (order-independent). A baseline
/// written before ambiguity tracking omits `amb=`, so it defaults to 0; a
/// `parse_ms=N` left by an older baseline is ignored.
fn parse_totals(line: &str) -> io::Result<BaselineTotals> {
    let map: BTreeMap<&str, &str> = line
        .split_whitespace()
        .filter_map(|t| t.split_once('='))
        .collect();
    let get = |key: &str| -> io::Result<u64> {
        map.get(key).and_then(|v| v.parse().ok()).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("malformed totals header: missing {key}"),
            )
        })
    };
    let get_opt = |key: &str| -> usize { map.get(key).and_then(|v| v.parse().ok()).unwrap_or(0) };
    Ok(BaselineTotals {
        files: get("files")? as usize,
        ok: get("ok")? as usize,
        ambiguous: get_opt("amb"),
        error: get("error")? as usize,
        ioerror: get("ioerror")? as usize,
    })
}

/// Parses one `path \t status \t [message]` record. A baseline written before
/// per-file times were dropped carried a time field before the message; it is
/// ignored, so the message is read as the last field and either layout parses.
fn parse_record_line(line: &str) -> Option<(String, BaselineRecord)> {
    let mut fields = line.split('\t');
    let path = fields.next()?.to_string();
    let status = match fields.next()? {
        "ok" => Status::Ok,
        "amb" => Status::Ambiguous,
        "error" => Status::Error,
        "ioerror" => Status::IoError,
        _ => return None,
    };
    let message = match status {
        Status::Ok | Status::Ambiguous => None,
        Status::Error | Status::IoError => fields.next_back().map(|s| s.to_string()),
    };
    Some((path, BaselineRecord { status, message }))
}

/// The per-file divergences between a run and its baseline, split by kind so
/// only regressions decide pass or fail and the soft kinds stay out of the exit code.
#[derive(Default)]
struct Diffs {
    /// `ok -> error/ioerror`: a file that parsed now fails. The only failing case.
    regressions: Vec<(String, String)>,
    /// `error/ioerror -> ok`: a file that failed now parses.
    recoveries: Vec<String>,
    /// Still failing, but the status or (brittle) message text changed.
    drift: usize,
    /// In the current run but not the baseline (the corpus grew).
    added: usize,
    /// In the baseline but not the current run (the corpus shrank).
    removed: usize,
}

impl Diffs {
    /// Whether the run reproduced the baseline exactly.
    fn is_clean(&self) -> bool {
        self.regressions.is_empty()
            && self.recoveries.is_empty()
            && self.drift == 0
            && self.added == 0
            && self.removed == 0
    }
}

/// Classifies every file against its baseline record. Only `regressions`
/// fails the check; the rest are soft signals.
fn diff_records(current: &[CurrentFile], baseline: &BTreeMap<String, BaselineRecord>) -> Diffs {
    let mut diffs = Diffs::default();
    let mut seen = BTreeSet::new();
    for f in current {
        seen.insert(f.path.as_str());
        let Some(b) = baseline.get(&f.path) else {
            diffs.added += 1;
            continue;
        };
        match (b.status == Status::Ok, f.status == Status::Ok) {
            (true, true) => {}
            (true, false) => {
                // An ambiguous parse carries no message, so label it explicitly;
                // errors and io-errors bring their own.
                let detail = if f.status == Status::Ambiguous {
                    "ambiguous".to_string()
                } else {
                    f.message.clone().unwrap_or_default()
                };
                diffs.regressions.push((f.path.clone(), detail));
            }
            (false, true) => diffs.recoveries.push(f.path.clone()),
            (false, false) => {
                // Both failing: a status or message change is brittle drift.
                if b.status != f.status || f.message.as_deref() != b.message.as_deref() {
                    diffs.drift += 1;
                }
            }
        }
    }
    diffs.removed = baseline
        .keys()
        .filter(|p| !seen.contains(p.as_str()))
        .count();
    diffs
}

/// Prints the drill-down: regressions first (the signal), then recoveries, then
/// the brittle kinds collapsed to counts. Each list is capped so a sweeping
/// change can't flood the output.
fn print_diffs(color: &Color, diffs: &Diffs) {
    const CAP: usize = 50;
    if !diffs.regressions.is_empty() {
        println!(
            "  {}regressions (ok -> fail): {}{}",
            color.red,
            diffs.regressions.len(),
            color.reset
        );
        for (path, message) in diffs.regressions.iter().take(CAP) {
            let detail = if message.is_empty() {
                String::new()
            } else {
                format!(": {message}")
            };
            println!("    {path}{detail}");
        }
        if diffs.regressions.len() > CAP {
            println!("    +{} more", diffs.regressions.len() - CAP);
        }
    }
    if !diffs.recoveries.is_empty() {
        println!(
            "  {}recoveries (fail -> ok): {}{}",
            color.green,
            diffs.recoveries.len(),
            color.reset
        );
        for path in diffs.recoveries.iter().take(CAP) {
            println!("    {path}");
        }
        if diffs.recoveries.len() > CAP {
            println!("    +{} more", diffs.recoveries.len() - CAP);
        }
    }
    if diffs.drift > 0 {
        println!(
            "  drift (still failing, message/status changed): {}",
            diffs.drift
        );
    }
    if diffs.added > 0 || diffs.removed > 0 {
        println!("  added: {}   removed: {}", diffs.added, diffs.removed);
    }
}

/// Prints the baseline's recorded counts, the reference for the drill-down.
fn print_baseline_counts(totals: &BaselineTotals) {
    println!(
        "  baseline: {} files, {} ok, {} ambiguous, {} error, {} ioerror",
        totals.files, totals.ok, totals.ambiguous, totals.error, totals.ioerror
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_repos_text_reads_entries_and_skips_comments() {
        let cfg = "# name ext start repo ref\n\
                   openjdk9 java CompilationUnit https://x/jdk9 jdk-9+181\n\
                   \n\
                   spring java CompilationUnit https://x/spring v4\n";
        let entries = parse_repos_text(cfg).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].name, "openjdk9");
        assert_eq!(entries[0].start, "CompilationUnit");
        assert_eq!(entries[1].git_ref, "v4");
    }

    #[test]
    fn parse_repos_text_rejects_wrong_arity() {
        assert!(parse_repos_text("only three fields here\n").is_err());
    }

    #[test]
    fn parse_record_line_reads_each_status() {
        let (path, r) = parse_record_line("a/B.java\tok").unwrap();
        assert_eq!(path, "a/B.java");
        assert_eq!(r.status, Status::Ok);

        let (_, r) = parse_record_line("a/B.java\tamb").unwrap();
        assert_eq!(r.status, Status::Ambiguous);

        // Errors and io-errors carry their message as the last field.
        let (_, r) =
            parse_record_line("a/B.java\terror\tParse error at line 1, column 1: x").unwrap();
        assert_eq!(r.status, Status::Error);
        assert_eq!(
            r.message.as_deref(),
            Some("Parse error at line 1, column 1: x")
        );

        let (_, r) = parse_record_line("a/B.java\tioerror\tbad utf-8").unwrap();
        assert_eq!(r.status, Status::IoError);
        assert_eq!(r.message.as_deref(), Some("bad utf-8"));

        // A baseline written before per-file times were dropped still parses: the
        // leading time field is ignored, so the message stays the last field.
        let (_, r) = parse_record_line("a/B.java\tok\t12").unwrap();
        assert_eq!(r.status, Status::Ok);
        let (_, r) = parse_record_line("a/B.java\terror\t-\told format").unwrap();
        assert_eq!(r.message.as_deref(), Some("old format"));
    }

    #[test]
    fn parse_totals_reads_counts() {
        // A trailing parse_ms= from an older baseline is tolerated and ignored.
        let t = parse_totals("files=10 ok=7 amb=1 error=1 ioerror=1 parse_ms=42").unwrap();
        assert_eq!(
            (t.files, t.ok, t.ambiguous, t.error, t.ioerror),
            (10, 7, 1, 1, 1)
        );
        // A baseline written before ambiguity tracking omits amb=; it defaults to 0.
        let old = parse_totals("files=10 ok=8 error=1 ioerror=1").unwrap();
        assert_eq!(old.ambiguous, 0);
    }

    #[test]
    fn sanitize_strips_tabs_and_newlines() {
        assert_eq!(sanitize("a\tb\nc\rd"), "a b c d");
    }

    #[test]
    fn serialize_baseline_records_status_without_times() {
        let files = vec![
            CurrentFile {
                path: "a.java".into(),
                status: Status::Ok,
                message: None,
            },
            CurrentFile {
                path: "c.java".into(),
                status: Status::Error,
                message: Some("Parse error".into()),
            },
            CurrentFile {
                path: "e.java".into(),
                status: Status::Ambiguous,
                message: None,
            },
        ];
        let out = serialize_baseline("demo", &files);
        assert!(out.contains("# corpus: demo\n"));
        assert!(out.contains("# files=3 ok=1 amb=1 error=1 ioerror=0\n"));
        // Status only, no time; errors keep their message.
        assert!(out.contains("a.java\tok\n"));
        assert!(out.contains("e.java\tamb\n"));
        assert!(out.contains("c.java\terror\tParse error\n"));
    }

    #[test]
    fn diff_records_separates_regression_from_recovery() {
        // One file regresses ok->error while another recovers error->ok: counts
        // are unchanged, but the regression alone fails the check.
        let current = vec![
            CurrentFile {
                path: "a".into(),
                status: Status::Error,
                message: Some("boom".into()),
            },
            CurrentFile {
                path: "b".into(),
                status: Status::Ok,
                message: None,
            },
        ];
        let mut baseline = BTreeMap::new();
        baseline.insert(
            "a".to_string(),
            BaselineRecord {
                status: Status::Ok,
                message: None,
            },
        );
        baseline.insert(
            "b".to_string(),
            BaselineRecord {
                status: Status::Error,
                message: Some("boom".into()),
            },
        );

        let diffs = diff_records(&current, &baseline);
        assert_eq!(
            diffs.regressions,
            vec![("a".to_string(), "boom".to_string())]
        );
        assert_eq!(diffs.recoveries, vec!["b".to_string()]);
    }

    #[test]
    fn diff_records_treats_ambiguity_like_an_error() {
        // ok -> amb regresses (labeled "ambiguous"), amb -> amb is a known state
        // and stays clean, and amb -> ok recovers, exactly as error does.
        let current = vec![
            CurrentFile {
                path: "new".into(),
                status: Status::Ambiguous,
                message: None,
            },
            CurrentFile {
                path: "known".into(),
                status: Status::Ambiguous,
                message: None,
            },
            CurrentFile {
                path: "fixed".into(),
                status: Status::Ok,
                message: None,
            },
        ];
        let mut baseline = BTreeMap::new();
        baseline.insert(
            "new".to_string(),
            BaselineRecord {
                status: Status::Ok,
                message: None,
            },
        );
        baseline.insert(
            "known".to_string(),
            BaselineRecord {
                status: Status::Ambiguous,
                message: None,
            },
        );
        baseline.insert(
            "fixed".to_string(),
            BaselineRecord {
                status: Status::Ambiguous,
                message: None,
            },
        );

        let diffs = diff_records(&current, &baseline);
        assert_eq!(
            diffs.regressions,
            vec![("new".to_string(), "ambiguous".to_string())]
        );
        assert_eq!(diffs.recoveries, vec!["fixed".to_string()]);
        assert_eq!(diffs.drift, 0);
    }

    #[test]
    fn diff_records_message_change_is_drift_not_regression() {
        // A file that failed before and still fails, only with different error
        // text, is brittle drift, so it must not fail the check.
        let current = vec![CurrentFile {
            path: "a".into(),
            status: Status::Error,
            message: Some("expected TypeIdentifier".into()),
        }];
        let mut baseline = BTreeMap::new();
        baseline.insert(
            "a".to_string(),
            BaselineRecord {
                status: Status::Error,
                message: Some("expected Identifier".into()),
            },
        );

        let diffs = diff_records(&current, &baseline);
        assert!(diffs.regressions.is_empty());
        assert_eq!(diffs.drift, 1);
    }

    #[test]
    fn diff_records_is_clean_when_identical() {
        let current = vec![
            CurrentFile {
                path: "a".into(),
                status: Status::Ok,
                message: None,
            },
            CurrentFile {
                path: "b".into(),
                status: Status::Error,
                message: Some("boom".into()),
            },
        ];
        let mut baseline = BTreeMap::new();
        baseline.insert(
            "a".to_string(),
            BaselineRecord {
                status: Status::Ok,
                message: None,
            },
        );
        baseline.insert(
            "b".to_string(),
            BaselineRecord {
                status: Status::Error,
                message: Some("boom".into()),
            },
        );

        assert!(diff_records(&current, &baseline).is_clean());
    }
}
