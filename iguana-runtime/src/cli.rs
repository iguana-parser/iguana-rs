use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{self, IsTerminal};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use crate::parse_tree::SexprOptions;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Symbols {
    pub nonterminals: Vec<String>,
    pub terminals: Vec<String>,
    pub slots: Vec<String>,
}

pub struct BenchConfig {
    pub iters: usize,
    pub warmup: usize,
    pub save: Option<PathBuf>,
    pub baseline: Option<PathBuf>,
}

pub struct BenchSummary {
    pub n: usize,
    pub min: f64,
    pub mean: f64,
    pub median: f64,
    pub p90: f64,
    pub max: f64,
    pub variance: f64,
    pub stddev: f64,
}

/// Per-iteration phase breakdown. Total wall time is the sum of all phases.
///   - `input`: file read + line/column offset table construction
///   - `init`: arenas + parse tree builder + Parser allocation
///   - `parse`: the GLL parse itself
///   - `tree`: SPPF → parse tree extraction
///   - `drop`: teardown of all the above
pub struct PhaseTimings {
    pub input: Duration,
    pub init: Duration,
    pub parse: Duration,
    pub tree: Duration,
    pub drop: Duration,
    /// Bytes of input parsed in this iteration (used for throughput).
    pub bytes: u64,
}

/// Runs `parse_once` `warmup + iters` times, collecting per-phase timings.
/// Warmup samples are discarded. Prints summary stats for total and each
/// phase plus throughput (MB/s) at the median total. If `config.save` is
/// set, writes the raw samples as JSON. If `config.baseline` is set, loads
/// a prior run and reports the mean delta on total time with a 95% CI on
/// the difference (Welch's SE for unequal variances).
pub fn run_benchmark(
    config: BenchConfig,
    mut parse_once: impl FnMut() -> PhaseTimings,
) -> io::Result<()> {
    let mut input_samples = Vec::with_capacity(config.iters);
    let mut init_samples = Vec::with_capacity(config.iters);
    let mut parse_samples = Vec::with_capacity(config.iters);
    let mut tree_samples = Vec::with_capacity(config.iters);
    let mut drop_samples = Vec::with_capacity(config.iters);
    let mut total_samples = Vec::with_capacity(config.iters);
    let mut bytes_per_iter: u64 = 0;

    for i in 0..(config.warmup + config.iters) {
        let timings = parse_once();
        if i >= config.warmup {
            let inp = timings.input.as_secs_f64() * 1000.0;
            let ini = timings.init.as_secs_f64() * 1000.0;
            let p = timings.parse.as_secs_f64() * 1000.0;
            let t = timings.tree.as_secs_f64() * 1000.0;
            let d = timings.drop.as_secs_f64() * 1000.0;
            input_samples.push(inp);
            init_samples.push(ini);
            parse_samples.push(p);
            tree_samples.push(t);
            drop_samples.push(d);
            total_samples.push(inp + ini + p + t + d);
            bytes_per_iter = timings.bytes;
        }
    }

    let total = summarize(&total_samples);
    let input = summarize(&input_samples);
    let init = summarize(&init_samples);
    let parse = summarize(&parse_samples);
    let tree = summarize(&tree_samples);
    let drop = summarize(&drop_samples);

    let color = Color::for_stdout();
    let warmup_note = if config.warmup > 0 {
        format!(" (+{} warmup)", config.warmup)
    } else {
        String::new()
    };
    println!();
    println!(
        "Benchmark: {} iteration{}{}, {} bytes per iteration (times in ms)",
        config.iters,
        if config.iters == 1 { "" } else { "s" },
        warmup_note,
        group_digits(&bytes_per_iter.to_string()),
    );
    println!();
    // Header (bold on a terminal), then a phase per row, a rule, and the total
    // last since it is the sum of the phases above it.
    println!(
        "  {}{:<8}{:>14}{:>14}{:>14}{:>14}{:>14}{:>14}{}",
        color.bold, "phase", "min", "mean", "stddev", "median", "p90", "max", color.reset
    );
    let decimals = ms_decimals(total.max);
    print_phase_row("input", &input, decimals);
    print_phase_row("init", &init, decimals);
    print_phase_row("parse", &parse, decimals);
    print_phase_row("tree", &tree, decimals);
    print_phase_row("drop", &drop, decimals);
    println!("  {}", "-".repeat(8 + 6 * 14));
    print_phase_row("total", &total, decimals);
    println!();
    println!(
        "Throughput (median): {:.2} MB/s total, {:.2} MB/s parse-only",
        mb_per_s(bytes_per_iter, total.median),
        mb_per_s(bytes_per_iter, parse.median),
    );

    if let Some(ref path) = config.save {
        let json = serde_json::json!({
            "version": 2,
            "bytes": bytes_per_iter,
            "samples_ms": total_samples,
            "input_samples_ms": input_samples,
            "init_samples_ms": init_samples,
            "parse_samples_ms": parse_samples,
            "tree_samples_ms": tree_samples,
            "drop_samples_ms": drop_samples,
        });
        fs::write(path, serde_json::to_string_pretty(&json).unwrap())?;
        eprintln!("Saved baseline to {}", path.display());
    }

    if let Some(ref path) = config.baseline {
        let text = fs::read_to_string(path)?;
        let parsed: serde_json::Value = serde_json::from_str(&text).map_err(io::Error::other)?;
        let baseline_samples: Vec<f64> = parsed["samples_ms"]
            .as_array()
            .ok_or_else(|| io::Error::other("baseline missing samples_ms array"))?
            .iter()
            .map(|v| v.as_f64().unwrap())
            .collect();
        let baseline = summarize(&baseline_samples);
        let delta = total.mean - baseline.mean;
        let se = (total.variance / total.n as f64 + baseline.variance / baseline.n as f64).sqrt();
        let ci_half = 1.96 * se;
        let pct = 100.0 * delta / baseline.mean;
        println!();
        println!(
            "Compared to baseline {} ({} samples, mean {:.3} ms total):",
            path.display(),
            baseline.n,
            baseline.mean
        );
        println!("  delta  = {:+.3} ms ({:+.2}%)", delta, pct);
        println!(
            "  95% CI = [{:+.3}, {:+.3}] ms",
            delta - ci_half,
            delta + ci_half
        );
        if delta + ci_half < 0.0 {
            println!("  Result: IMPROVED (CI excludes 0)");
        } else if delta - ci_half > 0.0 {
            println!("  Result: REGRESSED (CI excludes 0)");
        } else {
            println!("  Result: no significant change (CI includes 0)");
        }
    }

    Ok(())
}

fn print_phase_row(name: &str, s: &BenchSummary, decimals: usize) {
    println!(
        "  {:<8}{:>14}{:>14}{:>14}{:>14}{:>14}{:>14}",
        name,
        fmt_ms(s.min, decimals),
        fmt_ms(s.mean, decimals),
        fmt_ms(s.stddev, decimals),
        fmt_ms(s.median, decimals),
        fmt_ms(s.p90, decimals),
        fmt_ms(s.max, decimals),
    );
}

/// Decimal places for a table whose largest value is `max`. Large values (a
/// whole-corpus pass) report whole or tenths of a millisecond, where finer
/// digits are noise; small values (a single file) keep three so sub-millisecond
/// timings stay legible.
fn ms_decimals(max: f64) -> usize {
    if max >= 10_000.0 {
        0
    } else if max >= 1_000.0 {
        1
    } else {
        3
    }
}

/// Inserts thousands separators into a run of digits: `"101454"` -> `"101,454"`.
/// Public so generated benchmark progress lines can comma-format their counts.
pub fn group_digits(digits: &str) -> String {
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    out
}

/// Formats a millisecond value with thousands separators and `decimals` decimal
/// places, e.g. `fmt_ms(101454.864, 0)` -> `"101,455"`.
fn fmt_ms(v: f64, decimals: usize) -> String {
    let s = format!("{:.*}", decimals, v);
    match s.split_once('.') {
        Some((int, frac)) => format!("{}.{}", group_digits(int), frac),
        None => group_digits(&s),
    }
}

/// Throughput in megabytes per second from `bytes` and `ms`.
/// 1 MB = 1_000_000 bytes (decimal, matching disk/network convention).
pub fn mb_per_s(bytes: u64, ms: f64) -> f64 {
    if ms <= 0.0 {
        return 0.0;
    }
    (bytes as f64) / (ms / 1000.0) / 1_000_000.0
}

fn summarize(samples_ms: &[f64]) -> BenchSummary {
    let mut sorted = samples_ms.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = sorted.len();
    let mean = sorted.iter().sum::<f64>() / n as f64;
    let variance = sorted
        .iter()
        .map(|x| {
            let v = x - mean;
            v * v
        })
        .sum::<f64>()
        / n as f64;
    BenchSummary {
        n,
        min: sorted[0],
        mean,
        median: sorted[n / 2],
        p90: sorted[(n as f64 * 0.9) as usize],
        max: sorted[n - 1],
        variance,
        stddev: variance.sqrt(),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ParseResult {
    Success(ParseSuccess),
    Failure(ParseFailure),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseSuccess {
    pub parse_ms: u64,
    pub tree_construction_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseFailure {
    pub line: u32,
    pub column: u32,
    pub message: String,
}

/// Outcome of parsing one REPL input.
pub enum ReplOutcome {
    Parsed { tree: String, ambiguous: bool },
    Failed { message: String },
}

/// Reads inputs from stdin and prints the parse tree for each. One input is a
/// block of lines terminated by a blank line; `Ctrl-D` exits. A line starting
/// with `:` at a fresh prompt is a meta-command (`:set`, `:help`), not parse
/// input. Prompts and diagnostics go to stderr and parse trees to stdout, so
/// the output stays usable when stdout is redirected to a file.
///
/// `options` is the initial rendering state, taken from the parser's flags;
/// `:set` mutates it during the session and each parse renders with the
/// current value.
pub fn run_repl<F>(mut options: SexprOptions, mut parse_fn: F)
where
    F: FnMut(&str, SexprOptions) -> ReplOutcome,
{
    use io::{BufRead, Write};

    eprintln!("Enter input, blank line to parse. :help for commands, Ctrl-D to exit.");
    let stdin = io::stdin();
    let mut handle = stdin.lock();
    loop {
        eprint!("> ");
        io::stderr().flush().ok();

        // Accumulate lines until a blank line submits the block.
        let mut block = String::new();
        let mut command = false;
        loop {
            let mut line = String::new();
            if handle.read_line(&mut line).unwrap_or(0) == 0 {
                // EOF: parse a pending block, then exit on the next prompt.
                if block.is_empty() {
                    eprintln!();
                    return;
                }
                break;
            }
            // A `:` line at a fresh prompt is a command; mid-block it is input.
            if block.is_empty() && line.trim_start().starts_with(':') {
                run_repl_command(line.trim(), &mut options);
                command = true;
                break;
            }
            if line.trim_end_matches(['\n', '\r']).is_empty() {
                break;
            }
            block.push_str(&line);
            eprint!(". ");
            io::stderr().flush().ok();
        }
        if command {
            continue;
        }

        let input = block.trim_end();
        if input.is_empty() {
            continue;
        }
        match parse_fn(input, options) {
            ReplOutcome::Parsed { tree, ambiguous } => {
                if ambiguous {
                    eprintln!("// ambiguous");
                }
                print!("{}", tree);
                io::stdout().flush().ok();
            }
            ReplOutcome::Failed { message } => eprintln!("{}", message),
        }
    }
}

/// Runs a REPL meta-command (the line still carries its leading `:`). Feedback
/// goes to stderr, like the prompt.
fn run_repl_command(line: &str, options: &mut SexprOptions) {
    let mut parts = line.split_whitespace();
    match parts.next() {
        Some(":help") | Some(":h") | Some(":?") => {
            eprintln!("commands:");
            eprintln!("  :set                       list settings");
            eprintln!("  :set <name> [true|false]   toggle, or set, a setting");
            eprintln!("  settings: show-layout, show-empty, show-wrappers");
            eprintln!("  :help                      show this help");
            eprintln!("  Ctrl-D                     exit");
        }
        Some(":set") => {
            let name = parts.next();
            let value = parts.next();
            match name {
                None => {
                    eprintln!("show-layout   = {}", options.show_layout);
                    eprintln!("show-empty    = {}", options.show_empty);
                    eprintln!("show-wrappers = {}", options.show_wrappers);
                }
                Some("show-layout") => {
                    set_repl_bool("show-layout", value, &mut options.show_layout)
                }
                Some("show-empty") => set_repl_bool("show-empty", value, &mut options.show_empty),
                Some("show-wrappers") => {
                    set_repl_bool("show-wrappers", value, &mut options.show_wrappers)
                }
                Some(other) => eprintln!("unknown setting: {} (try :help)", other),
            }
        }
        Some(other) => eprintln!("unknown command: {} (try :help)", other),
        None => {}
    }
}

/// Applies a `:set <name> [value]` to a boolean setting, echoing the result.
/// A missing value toggles, so `:set show-layout` flips it.
fn set_repl_bool(name: &str, value: Option<&str>, slot: &mut bool) {
    match parse_repl_bool(value, *slot) {
        Ok(value) => {
            *slot = value;
            eprintln!("{name} = {value}");
        }
        Err(()) => eprintln!("usage: :set {name} [true|false]"),
    }
}

/// Parses a boolean setting value. A missing value toggles the current state,
/// so the common `:set show-layout` flips it without typing true or false.
fn parse_repl_bool(value: Option<&str>, current: bool) -> Result<bool, ()> {
    match value {
        None => Ok(!current),
        Some("true") | Some("on") => Ok(true),
        Some("false") | Some("off") => Ok(false),
        Some(_) => Err(()),
    }
}

/// Whether the golden harness checks parser output against existing goldens or
/// rewrites them.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum GoldenMode {
    Check,
    Regenerate,
}

/// Runs golden-file testing over `inputs`. For each input file `X.<ext>`, the
/// golden is `X.sexpr` in the same directory. `golden_content` produces the text
/// to compare or write for one input: the parse-tree s-expression on success,
/// or a one-line `Parse error at line N, col M: <message>` on failure. It only
/// fails (`Err`) when the input itself is unreadable.
///
/// In `Check` mode each file is `OK`, `DIFF`, `MISS`, or `ERR`, and the run
/// passes only if every file is `OK`. In `Regenerate` mode each file is
/// `WRITE`, `unchanged`, or `ERR`, and the run passes unless an `ERR` occurred.
/// Returns whether the run passed so the caller can set the process exit code.
///
/// `quiet` suppresses the per-file status lines and the summary but never the
/// diff on a `DIFF` (whose header names the file). `full_diff` disables the
/// 200-line diff truncation. Paths are shown relative to `root` (the `--dir`
/// directory) when given, so the output reads cleanly; `None` shows them as
/// given (single-file mode).
pub fn run_golden(
    mode: GoldenMode,
    inputs: Vec<PathBuf>,
    root: Option<&Path>,
    quiet: bool,
    full_diff: bool,
    mut golden_content: impl FnMut(&Path) -> io::Result<String>,
) -> io::Result<bool> {
    let color = Color::for_stdout();
    let start = Instant::now();

    if inputs.is_empty() {
        eprintln!("Warning: no input files to check.");
    }

    let mut ok = 0usize;
    let mut diff = 0usize;
    let mut miss = 0usize;
    let mut written = 0usize;
    let mut unchanged = 0usize;
    let mut errs = 0usize;

    for input in &inputs {
        let golden_path = input.with_extension("sexpr");
        let shown_input = rel(input, root).display().to_string();
        let shown_golden = rel(&golden_path, root).display().to_string();
        let actual = match golden_content(input) {
            Ok(actual) => actual,
            Err(e) => {
                errs += 1;
                if !quiet {
                    print_status(&color, "ERR", false, &format!("{shown_input}: {e}"));
                }
                continue;
            }
        };

        match mode {
            GoldenMode::Check => match read_golden(&golden_path) {
                Ok(Some(golden)) if golden == actual => {
                    ok += 1;
                    if !quiet {
                        print_status(&color, "OK", true, &shown_input);
                    }
                }
                Ok(Some(golden)) => {
                    diff += 1;
                    if !quiet {
                        print_status(&color, "DIFF", false, &shown_input);
                    }
                    let body = unified_diff(&golden, &actual, &shown_golden, full_diff);
                    print_indented(&body);
                }
                Ok(None) => {
                    miss += 1;
                    if !quiet {
                        print_status(&color, "MISS", false, &shown_input);
                        println!("  (no golden; regenerate with --regenerate-sexpr)");
                    }
                }
                Err(e) => {
                    errs += 1;
                    if !quiet {
                        print_status(&color, "ERR", false, &format!("{shown_golden}: {e}"));
                    }
                }
            },
            GoldenMode::Regenerate => {
                // An unchanged golden gets no per-file line; only writes and
                // errors are worth reporting (the count lands in the summary).
                let same = matches!(read_golden(&golden_path), Ok(Some(ref g)) if *g == actual);
                if same {
                    unchanged += 1;
                } else if let Err(e) = fs::write(&golden_path, &actual) {
                    errs += 1;
                    if !quiet {
                        print_status(&color, "ERR", false, &format!("{shown_golden}: {e}"));
                    }
                } else {
                    written += 1;
                    if !quiet {
                        print_status(&color, "WRITE", true, &shown_input);
                    }
                }
            }
        }
    }

    let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;
    if !quiet {
        println!();
        match mode {
            GoldenMode::Check => println!(
                "Checked {} files: {} OK, {} DIFF, {} MISS, {} errors",
                inputs.len(),
                ok,
                diff,
                miss,
                errs
            ),
            GoldenMode::Regenerate => println!(
                "Regenerated {} files: {} written, {} unchanged, {} errors",
                inputs.len(),
                written,
                unchanged,
                errs
            ),
        }
        println!("Elapsed {:.0} ms", elapsed_ms);
    }

    let passed = match mode {
        GoldenMode::Check => diff == 0 && miss == 0 && errs == 0,
        GoldenMode::Regenerate => errs == 0,
    };
    Ok(passed)
}

/// Strips `root` from `path` for display, falling back to the full path when
/// `path` is not under `root` or `root` is `None` (single-file mode).
fn rel<'a>(path: &'a Path, root: Option<&Path>) -> &'a Path {
    root.and_then(|r| path.strip_prefix(r).ok()).unwrap_or(path)
}

/// Reads a golden file, normalizing CRLF to LF so goldens written on Windows
/// still compare equal. Returns `Ok(None)` when the golden does not exist (a
/// `MISS`), and `Err` only on a real I/O failure.
fn read_golden(path: &Path) -> io::Result<Option<String>> {
    match fs::read_to_string(path) {
        Ok(text) => Ok(Some(text.replace("\r\n", "\n"))),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e),
    }
}

/// ANSI colors, enabled only when stdout is a terminal so redirected output
/// stays plain. Shared by `run_golden` here and the generated `run_batch`.
pub struct Color {
    pub green: &'static str,
    pub red: &'static str,
    pub bold: &'static str,
    pub reset: &'static str,
}

impl Color {
    pub fn for_stdout() -> Self {
        if io::stdout().is_terminal() {
            Color {
                green: "\x1b[32m",
                red: "\x1b[31m",
                bold: "\x1b[1m",
                reset: "\x1b[0m",
            }
        } else {
            Color {
                green: "",
                red: "",
                bold: "",
                reset: "",
            }
        }
    }
}

/// Prints one per-file status line, coloring the status token green when `good`
/// and red otherwise.
fn print_status(color: &Color, status: &str, good: bool, rest: &str) {
    let code = if good { color.green } else { color.red };
    println!("{}{:<6}{}{}", code, status, color.reset, rest);
}

/// Prints `body` with every line indented two spaces, the way the diff sits
/// under a `DIFF` line.
fn print_indented(body: &str) {
    for line in body.lines() {
        println!("  {}", line);
    }
}

/// One line of a unified diff: a shared line, a line only in the golden, or a
/// line only in the actual output.
enum Edit<'a> {
    Eq(&'a str),
    Del(&'a str),
    Ins(&'a str),
}

/// Renders a unified diff of `golden` against `actual` with three lines of
/// context and `--- name (golden)` / `+++ name (actual)` headers. Lines are
/// split on `\n`, so a trailing-newline difference shows as a changed final
/// line. Unless `full_diff`, the body is truncated past 200 lines with a
/// `... (N more lines)` marker.
fn unified_diff(golden: &str, actual: &str, name: &str, full_diff: bool) -> String {
    let a: Vec<&str> = golden.split('\n').collect();
    let b: Vec<&str> = actual.split('\n').collect();
    let edits = diff_lines(&a, &b);

    let mut out = String::new();
    out.push_str(&format!("--- {name} (golden)\n"));
    out.push_str(&format!("+++ {name} (actual)\n"));
    out.push_str(&render_hunks(&edits));

    if !full_diff {
        // Keep the two headers plus 200 body lines before truncating. `out` is
        // newline-terminated, so the line count is its number of newlines.
        const LIMIT: usize = 202;
        let line_count = out.matches('\n').count();
        if line_count > LIMIT {
            let kept: String = out.split_inclusive('\n').take(LIMIT).collect();
            let remaining = line_count - LIMIT;
            return format!("{kept}... ({remaining} more lines)\n");
        }
    }
    out
}

/// Longest-common-subsequence line diff. Test goldens are small, so the
/// quadratic table is fine; an oversized input falls back to deleting the whole
/// golden and inserting the whole actual rather than allocating a huge table.
fn diff_lines<'a>(a: &[&'a str], b: &[&'a str]) -> Vec<Edit<'a>> {
    let (n, m) = (a.len(), b.len());
    if n.saturating_mul(m) > 4_000_000 {
        let mut edits = Vec::with_capacity(n + m);
        edits.extend(a.iter().map(|l| Edit::Del(l)));
        edits.extend(b.iter().map(|l| Edit::Ins(l)));
        return edits;
    }

    let mut dp = vec![vec![0u32; m + 1]; n + 1];
    for i in (0..n).rev() {
        for j in (0..m).rev() {
            dp[i][j] = if a[i] == b[j] {
                dp[i + 1][j + 1] + 1
            } else {
                dp[i + 1][j].max(dp[i][j + 1])
            };
        }
    }

    let mut edits = Vec::new();
    let (mut i, mut j) = (0, 0);
    while i < n && j < m {
        if a[i] == b[j] {
            edits.push(Edit::Eq(a[i]));
            i += 1;
            j += 1;
        } else if dp[i + 1][j] >= dp[i][j + 1] {
            edits.push(Edit::Del(a[i]));
            i += 1;
        } else {
            edits.push(Edit::Ins(b[j]));
            j += 1;
        }
    }
    edits.extend(a[i..].iter().map(|l| Edit::Del(l)));
    edits.extend(b[j..].iter().map(|l| Edit::Ins(l)));
    edits
}

/// Groups an edit script into unified-diff hunks: each changed run is padded
/// with up to three lines of context, and two runs separated by no more than
/// six shared lines merge into one hunk.
fn render_hunks(edits: &[Edit]) -> String {
    const CONTEXT: usize = 3;

    // Count of lines consumed in each side before edit `i`; the hunk header
    // adds 1 to turn these into 1-based line numbers.
    let mut old_pos = vec![0usize; edits.len() + 1];
    let mut new_pos = vec![0usize; edits.len() + 1];
    for (i, e) in edits.iter().enumerate() {
        let (d_old, d_new) = match e {
            Edit::Eq(_) => (1, 1),
            Edit::Del(_) => (1, 0),
            Edit::Ins(_) => (0, 1),
        };
        old_pos[i + 1] = old_pos[i] + d_old;
        new_pos[i + 1] = new_pos[i] + d_new;
    }

    let changed: Vec<usize> = edits
        .iter()
        .enumerate()
        .filter(|(_, e)| !matches!(e, Edit::Eq(_)))
        .map(|(i, _)| i)
        .collect();
    if changed.is_empty() {
        return String::new();
    }

    // Merge adjacent changes whose gap of shared lines is within twice the
    // context, then expand each group by the context on both ends.
    let mut out = String::new();
    let mut group_start = changed[0];
    let mut group_end = changed[0];
    let mut groups = Vec::new();
    for &c in &changed[1..] {
        if c - group_end - 1 <= 2 * CONTEXT {
            group_end = c;
        } else {
            groups.push((group_start, group_end));
            group_start = c;
            group_end = c;
        }
    }
    groups.push((group_start, group_end));

    for (gs, ge) in groups {
        let start = gs.saturating_sub(CONTEXT);
        let end = (ge + 1 + CONTEXT).min(edits.len());
        let old_len = old_pos[end] - old_pos[start];
        let new_len = new_pos[end] - new_pos[start];
        out.push_str(&format!(
            "@@ -{},{} +{},{} @@\n",
            old_pos[start] + 1,
            old_len,
            new_pos[start] + 1,
            new_len
        ));
        for e in &edits[start..end] {
            match e {
                Edit::Eq(line) => out.push_str(&format!(" {line}\n")),
                Edit::Del(line) => out.push_str(&format!("-{line}\n")),
                Edit::Ins(line) => out.push_str(&format!("+{line}\n")),
            }
        }
    }
    out
}

/// One line of `repos.txt`: a corpus to fetch and parse. The parser reads
/// `name`, `ext`, and `start`; `repo`/`git_ref` are consumed by the fetch step
/// and kept as provenance.
pub struct CorpusEntry {
    pub name: String,
    pub ext: String,
    pub start: String,
    pub repo: String,
    pub git_ref: String,
}

/// Reads `repos.txt`: one corpus per line as whitespace-separated
/// `name ext start repo ref`. Blank lines and `#` comments are skipped.
pub fn read_repos(path: &Path) -> io::Result<Vec<CorpusEntry>> {
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
pub fn init_corpus_dir(dir: &Path) -> io::Result<bool> {
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
pub fn fetch_corpus(dir: &Path, repo: &str, git_ref: &str) -> io::Result<()> {
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
pub enum CorpusMode {
    Check,
    Update,
}

/// Knobs for a `run_corpus` run: check vs. rewrite, the slow-file threshold, the
/// soft perf tolerance, and whether to suppress the per-corpus status line.
pub struct CorpusConfig {
    pub mode: CorpusMode,
    pub slow_ms: f64,
    pub perf_tolerance_pct: f64,
    pub quiet: bool,
}

/// The outcome of parsing one corpus file. `ms` is the parse time (the parser's
/// own `run()`), used for the slow-file tail and the aggregate `parse_ms`.
/// `ambiguous` marks a success with more than one derivation; it records under
/// the `amb` baseline status, which the check treats exactly like an error.
pub enum CorpusOutcome {
    Ok { ms: f64, ambiguous: bool },
    Error { ms: f64, message: String },
    IoError { message: String },
}

/// Summary of one corpus run, returned so the caller can aggregate across
/// corpora and set the exit code. `passed` is false when a file regresses: it
/// parsed cleanly in the baseline and now fails, a newly ambiguous parse
/// included. On an `Update` this also means the baseline was left untouched,
/// since a rewrite is refused rather than record a regression.
pub struct CorpusReport {
    pub name: String,
    pub files: usize,
    pub ok: usize,
    pub ambiguous: usize,
    pub error: usize,
    pub ioerror: usize,
    pub parse_ms: u64,
    pub passed: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Status {
    Ok,
    Ambiguous,
    Error,
    IoError,
}

/// One file's outcome as just parsed, with the precise parse time retained for
/// the aggregate and the slow tail. `message` is already tab-sanitized.
struct CurrentFile {
    path: String,
    status: Status,
    message: Option<String>,
    ms: f64,
}

/// One per-file line read back from a baseline. `message` is present only for
/// `error`/`ioerror`.
struct BaselineRecord {
    status: Status,
    message: Option<String>,
}

/// Aggregate counts from a baseline header's `# files=… parse_ms=…` line.
struct BaselineTotals {
    files: usize,
    ok: usize,
    ambiguous: usize,
    error: usize,
    ioerror: usize,
    parse_ms: u64,
}

/// Runs corpus regression testing for one corpus. `parse_one` parses a single
/// file and reports `ok`/`error`/`ioerror` plus the parse time; `run_corpus`
/// owns building the per-file records, then either writing the baseline
/// (`Update`) or comparing against it (`Check`).
///
/// `root` is the corpus checkout; per-file paths are recorded relative to it,
/// normalized to `/`, and sorted, so the committed baseline is stable. `slow_ms`
/// is the threshold above which a file's parse time is recorded (the ~99% fast
/// files carry no time, so the file does not churn). `perf_tolerance_pct` is the
/// soft band on the aggregate `parse_ms`.
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
/// fails without writing. Timing is a separate soft signal: a `parse_ms` drift
/// past `perf_tolerance_pct` is reported but never fails the run.
pub fn run_corpus(
    name: &str,
    inputs: Vec<PathBuf>,
    root: &Path,
    baseline_path: &Path,
    config: CorpusConfig,
    mut parse_one: impl FnMut(&Path) -> CorpusOutcome,
) -> io::Result<CorpusReport> {
    let CorpusConfig {
        mode,
        slow_ms,
        perf_tolerance_pct,
        quiet,
    } = config;
    let color = Color::for_stdout();

    // Parse every file into a current record, accumulating the precise
    // aggregate parse time.
    let mut files: Vec<CurrentFile> = Vec::with_capacity(inputs.len());
    let mut parse_ms_total = 0.0f64;
    for input in &inputs {
        let path = normalize_rel(input, root);
        let (status, message, ms) = match parse_one(input) {
            CorpusOutcome::Ok {
                ms,
                ambiguous: is_ambig,
            } => {
                let status = if is_ambig {
                    Status::Ambiguous
                } else {
                    Status::Ok
                };
                (status, None, ms)
            }
            CorpusOutcome::Error { ms, message } => (Status::Error, Some(sanitize(&message)), ms),
            CorpusOutcome::IoError { message } => (Status::IoError, Some(sanitize(&message)), 0.0),
        };
        parse_ms_total += ms;
        files.push(CurrentFile {
            path,
            status,
            message,
            ms,
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
    let parse_ms = parse_ms_total as u64;
    let report = CorpusReport {
        name: name.to_string(),
        files: files.len(),
        ok,
        ambiguous,
        error,
        ioerror,
        parse_ms,
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
            fs::write(baseline_path, serialize_baseline(name, &files, parse_ms))?;
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
            // added/removed are soft. Timing always reports below, regardless of
            // the verdict.
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

            if !quiet {
                report_perf(
                    totals.parse_ms,
                    parse_ms,
                    perf_tolerance_pct,
                    slow_ms,
                    &files,
                );
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
/// per file. The parse time is written only for files past `slow_ms`, so the
/// fast majority carry no time and the committed file stays stable.
fn serialize_baseline(name: &str, files: &[CurrentFile], parse_ms: u64) -> String {
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
        "# files={} ok={ok} amb={ambiguous} error={error} ioerror={ioerror} parse_ms={parse_ms}\n",
        files.len()
    ));
    // Per-file lines carry status only, and a message for the two failing states.
    // No per-file time is recorded: a file's parse time drifts run to run and
    // would churn the committed baseline on every update. Perf is tracked by the
    // aggregate `parse_ms` in the header, and `--dir` reports per-file times live.
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

/// Parses `files=N ok=N amb=N error=N ioerror=N parse_ms=N` (order-independent).
/// A baseline written before ambiguity tracking omits `amb=`, so it defaults to 0.
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
        parse_ms: get("parse_ms")?,
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
        Status::Error | Status::IoError => fields.last().map(|s| s.to_string()),
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

/// Soft perf signal: prints nothing while the aggregate `parse_ms` stays within
/// tolerance; past it, reports the delta and the top files by `|Δms|` (using the
/// slow tail recorded in the baseline).
fn report_perf(
    baseline_parse_ms: u64,
    parse_ms: u64,
    tolerance_pct: f64,
    slow_ms: f64,
    current: &[CurrentFile],
) {
    if baseline_parse_ms == 0 {
        return;
    }
    let pct = 100.0 * (parse_ms as f64 - baseline_parse_ms as f64) / baseline_parse_ms as f64;
    if pct.abs() <= tolerance_pct {
        return;
    }
    println!(
        "  perf: parse_ms {baseline_parse_ms} -> {parse_ms} ({pct:+.1}%, tolerance {tolerance_pct:.0}%)"
    );

    // The baseline keeps no per-file times, so point at the current slowest files
    // as the place to look; `--dir` and `--benchmark` give the per-file detail.
    let mut slow: Vec<&CurrentFile> = current.iter().filter(|f| f.ms > slow_ms).collect();
    slow.sort_by(|a, b| b.ms.partial_cmp(&a.ms).unwrap());
    for f in slow.iter().take(10) {
        println!("    {:.0} ms  {}", f.ms, f.path);
    }
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
        let (_, r) = parse_record_line("a/B.java\terror\tParse error at line 1, col 1: x").unwrap();
        assert_eq!(r.status, Status::Error);
        assert_eq!(
            r.message.as_deref(),
            Some("Parse error at line 1, col 1: x")
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
        let t = parse_totals("files=10 ok=7 amb=1 error=1 ioerror=1 parse_ms=42").unwrap();
        assert_eq!(
            (t.files, t.ok, t.ambiguous, t.error, t.ioerror, t.parse_ms),
            (10, 7, 1, 1, 1, 42)
        );
        // A baseline written before ambiguity tracking omits amb=; it defaults to 0.
        let old = parse_totals("files=10 ok=8 error=1 ioerror=1 parse_ms=42").unwrap();
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
                ms: 12.0,
            },
            CurrentFile {
                path: "c.java".into(),
                status: Status::Error,
                message: Some("Parse error".into()),
                ms: 0.5,
            },
            CurrentFile {
                path: "e.java".into(),
                status: Status::Ambiguous,
                message: None,
                ms: 9.0,
            },
        ];
        let out = serialize_baseline("demo", &files, 13);
        assert!(out.contains("# corpus: demo\n"));
        assert!(out.contains("# files=3 ok=1 amb=1 error=1 ioerror=0 parse_ms=13\n"));
        // Status only, no per-file time even for a slow file; errors keep their message.
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
                ms: 0.0,
            },
            CurrentFile {
                path: "b".into(),
                status: Status::Ok,
                message: None,
                ms: 0.0,
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
                ms: 0.0,
            },
            CurrentFile {
                path: "known".into(),
                status: Status::Ambiguous,
                message: None,
                ms: 0.0,
            },
            CurrentFile {
                path: "fixed".into(),
                status: Status::Ok,
                message: None,
                ms: 0.0,
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
            ms: 0.0,
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
                ms: 0.0,
            },
            CurrentFile {
                path: "b".into(),
                status: Status::Error,
                message: Some("boom".into()),
                ms: 0.0,
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

    #[test]
    fn unified_diff_one_changed_line() {
        let diff = unified_diff("a\nb\nc\n", "a\nB\nc\n", "g", false);
        assert_eq!(
            diff,
            "--- g (golden)\n\
             +++ g (actual)\n\
             @@ -1,4 +1,4 @@\n\
             \x20a\n\
             -b\n\
             +B\n\
             \x20c\n\
             \x20\n"
        );
    }

    #[test]
    fn unified_diff_flags_trailing_newline() {
        // The golden ends with a newline, the actual does not: the golden's
        // trailing blank line is deleted, which flags the difference.
        let diff = unified_diff("(S)\n", "(S)", "g", false);
        assert_eq!(
            diff,
            "--- g (golden)\n\
             +++ g (actual)\n\
             @@ -1,2 +1,1 @@\n\
             \x20(S)\n\
             -\n"
        );
    }

    #[test]
    fn unified_diff_truncates_past_limit() {
        let golden: String = (0..300).map(|i| format!("g{i}\n")).collect();
        let actual: String = (0..300).map(|i| format!("a{i}\n")).collect();
        let diff = unified_diff(&golden, &actual, "g", false);
        assert!(diff.contains("more lines)"));
        assert!(diff.lines().count() <= 203);

        let full = unified_diff(&golden, &actual, "g", true);
        assert!(!full.contains("more lines)"));
    }
}
