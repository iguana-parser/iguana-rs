use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use crate::{
    arena::Arena,
    ids::NonterminalId,
    input::Input,
    parse_tree::is_ambiguous,
    parser::{GLLResult, Parser},
};

use super::{
    Color, args::Args, collect_files, corpus, start_nonterminal_id, start_nonterminal_name,
};

/// Runs a benchmark (the `--benchmark` flag). A benchmark has two modes:
/// - Single: a single file (the positional argument) is parsed `--iters`
///   times (100 by default), after `--warmup` runs that are discarded (10 by
///   default).
/// - Batch: every file in a directory (`--dir`) or in a file list
///   (`--files-from`, in order) is parsed, or every file in the corpus when
///   neither is given. The corpus is the list of
///   repositories in the `repos.txt` file of the corpus directory
///   (`--corpus-dir`, `corpus` by default). An iteration in the batch mode
///   is a pass over all the files. By default, the number of iterations is
///   3 and the number of warmup passes is 0. The files are reported in two
///   groups: Success, for the files that are parsed to a single parse tree,
///   and Failure, for the files that have a parse error, are ambiguous, or
///   cannot be read because of an I/O error.
pub(super) fn run<'i, 'arena, P: Parser<'i, 'arena>>(args: &Args) -> io::Result<()> {
    if let Some(file) = args.file.as_ref() {
        let start_nonterminal_id =
            start_nonterminal_id::<P::Grammar>(start_nonterminal_name(args)?)?;
        // A file that cannot be read has no time to report.
        Input::try_from(file.as_path())
            .map_err(|e| io::Error::new(e.kind(), format!("{}: {}", file.display(), e)))?;
        let config = BenchConfig {
            mode: BenchMode::Single,
            iters: args.iters.unwrap_or(100) as usize,
            warmup: args.warmup.unwrap_or(10) as usize,
            save: args.save.clone(),
            baseline: args.baseline.clone(),
        };
        eprintln!(
            "Benchmarking {} over {} {}...",
            file.display(),
            config.iters,
            if config.iters == 1 {
                "iteration"
            } else {
                "iterations"
            },
        );
        let file_path = file.clone();
        let mut tree_arena = Arena::new();
        let mut parser_arena = Arena::new();
        return run_benchmark(config, move || {
            let mut stats = Pass::default();
            bench_parse_file::<P>(
                &file_path,
                start_nonterminal_id,
                &mut tree_arena,
                &mut parser_arena,
                &mut stats,
            );
            stats
        });
    }
    let config = BenchConfig {
        mode: BenchMode::Batch,
        iters: args.iters.unwrap_or(3) as usize,
        warmup: args.warmup.unwrap_or(0) as usize,
        save: args.save.clone(),
        baseline: args.baseline.clone(),
    };
    let mut groups: Vec<(String, Vec<(PathBuf, NonterminalId)>)> = Vec::new();
    if let Some(list) = args.files_from.as_ref() {
        let start_nonterminal_id =
            start_nonterminal_id::<P::Grammar>(start_nonterminal_name(args)?)?;
        let text = fs::read_to_string(list)
            .map_err(|e| io::Error::new(e.kind(), format!("{}: {}", list.display(), e)))?;
        let paired = file_list(&text)
            .into_iter()
            .map(|p| (p, start_nonterminal_id))
            .collect();
        groups.push((list.display().to_string(), paired));
    } else if let Some(dir) = args.dir.as_ref() {
        let start_nonterminal_id =
            start_nonterminal_id::<P::Grammar>(start_nonterminal_name(args)?)?;
        let mut files = Vec::new();
        collect_files(dir, args.ext.as_deref(), &mut files)?;
        files.sort();
        let paired = files
            .into_iter()
            .map(|p| (p, start_nonterminal_id))
            .collect();
        groups.push((dir.display().to_string(), paired));
    } else {
        let repos_path = args.corpus_dir.join("repos.txt");
        if corpus::init_corpus_dir(&args.corpus_dir)? {
            println!(
                "Created {}. List your repos there and re-run.",
                repos_path.display()
            );
            return Ok(());
        }
        for entry in corpus::read_repos(&repos_path)? {
            let start_nonterminal_id = start_nonterminal_id::<P::Grammar>(&entry.start)?;
            let checkout = args.corpus_dir.join(".cache").join(&entry.name);
            corpus::fetch_corpus(&checkout, &entry.repo, &entry.git_ref)?;
            let mut files = Vec::new();
            collect_files(&checkout, Some(&entry.ext), &mut files)?;
            files.sort();
            let paired = files
                .into_iter()
                .map(|p| (p, start_nonterminal_id))
                .collect();
            groups.push((entry.name.clone(), paired));
        }
    }
    let total_files: usize = groups.iter().map(|(_, files)| files.len()).sum();
    if total_files == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "no files to benchmark",
        ));
    }
    let total = group_digits(&total_files.to_string());
    let iterations_word = if config.iters == 1 {
        "iteration"
    } else {
        "iterations"
    };
    if args.dir.is_none() && args.files_from.is_none() {
        eprintln!(
            "Running the corpus ({} files), {} {}:",
            total, config.iters, iterations_word
        );
        let width = groups
            .iter()
            .map(|(label, _)| label.len() + 1)
            .max()
            .unwrap_or(0);
        for (label, files) in &groups {
            eprintln!(
                "  {:<width$} {} files",
                format!("{}:", label),
                group_digits(&files.len().to_string()),
                width = width,
            );
        }
    } else {
        eprintln!(
            "Running {} ({} files), {} {}...",
            groups[0].0, total, config.iters, iterations_word
        );
    }
    eprintln!();
    let iters = config.iters;
    let warmup = config.warmup;
    let max_label = groups
        .iter()
        .map(|(label, _)| label.len())
        .max()
        .unwrap_or(0);
    let mut pass = 0usize;
    let mut tree_arena = Arena::new();
    let mut parser_arena = Arena::new();
    run_benchmark(config, move || {
        if pass > 0 {
            eprintln!();
        }
        let is_warmup = pass < warmup;
        let run_num = if is_warmup {
            pass + 1
        } else {
            pass - warmup + 1
        };
        let kind = if is_warmup { "warmup" } else { "run" };
        eprintln!(
            "{} {}/{}",
            kind,
            run_num,
            if is_warmup { warmup } else { iters }
        );
        pass += 1;
        let mut stats = Pass::default();
        for (label, files) in &groups {
            eprint!("  {}...", label);
            let _ = io::stderr().flush();
            let time_before = total_time(&stats);
            for (path, start_nonterminal_id) in files {
                bench_parse_file::<P>(
                    path,
                    *start_nonterminal_id,
                    &mut tree_arena,
                    &mut parser_arena,
                    &mut stats,
                );
            }
            let source_time = total_time(&stats) - time_before;
            let pad = " ".repeat(max_label - label.len() + 1);
            eprintln!(
                "{}{} ms",
                pad,
                group_digits(&format!("{:.0}", source_time.as_secs_f64() * 1000.0))
            );
        }
        let run_ms = as_ms(total_time(&stats));
        eprintln!(
            "{} {} completed in {} ms",
            kind,
            run_num,
            group_digits(&format!("{:.0}", run_ms))
        );
        stats
    })
}

/// The paths in a file list (--files-from), in order, without blank lines.
fn file_list(text: &str) -> Vec<PathBuf> {
    text.lines()
        .filter(|line| !line.trim().is_empty())
        .map(PathBuf::from)
        .collect()
}

/// Parses one file under a benchmark and records the file in `pass`, under
/// each phase the file goes through (`Phase`). The input is reloaded so the
/// `prepare` phase is measured. The caller's arenas are reused across files
/// and reset between them, so they keep their chunks and teardown is a bulk
/// reset rather than a per-file free, the pattern the arena is built for.
fn bench_parse_file<'i, 'arena, P: Parser<'i, 'arena>>(
    path: &Path,
    start_nonterminal_id: NonterminalId,
    tree_arena: &mut Arena,
    parser_arena: &mut Arena,
    pass: &mut Pass,
) {
    // Reading the file is deliberately not included in benchmarks. It
    // measures the file system rather than the parser, and its time varies
    // between runs because of factors like the page cache.
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(_) => {
            let bytes = fs::metadata(path).map(|m| m.len()).unwrap_or(0);
            record(pass, Phase::IoError, Duration::ZERO, bytes);
            return;
        }
    };
    let prepare_start = Instant::now();
    let input = Input::from(text);
    let prepare_time = prepare_start.elapsed();
    let bytes = input.byte_len() as u64;
    let init_start = Instant::now();
    let mut parser = P::ConcreteParser::<'_, '_>::new(&input, parser_arena);
    let init = init_start.elapsed();
    // `GLLResult::Failure` has no duration, so the benchmark also times the
    // run itself. This time includes computing the failure to report.
    let run_start = Instant::now();
    let result = parser.run(start_nonterminal_id);
    let run_time = run_start.elapsed();
    let success = match result {
        GLLResult::Success(success) => success,
        GLLResult::Failure(_) => {
            drop(parser);
            parser_arena.reset();
            record(pass, Phase::Error, run_time, bytes);
            return;
        }
    };
    let parse = success.duration;
    if is_ambiguous(&parser, success.sppf_node_id) {
        drop(parser);
        parser_arena.reset();
        record(pass, Phase::Amb, parse, bytes);
        return;
    }
    let tree_start = Instant::now();
    {
        let tree = parser.build_tree(success.sppf_node_id, tree_arena);
        std::hint::black_box(tree);
    }
    let tree = tree_start.elapsed();
    let drop_start = Instant::now();
    drop(parser);
    parser_arena.reset();
    tree_arena.reset();
    drop(input);
    let drop = drop_start.elapsed();
    record(pass, Phase::Prepare, prepare_time, bytes);
    record(pass, Phase::Init, init, bytes);
    record(pass, Phase::Parse, parse, bytes);
    record(pass, Phase::Tree, tree, bytes);
    record(pass, Phase::Drop, drop, bytes);
}

/// The mode of a benchmark: a single file, or every file of a directory, a
/// file list or the corpus.
#[derive(Clone, Copy, PartialEq, Eq)]
enum BenchMode {
    Single,
    Batch,
}

struct BenchConfig {
    mode: BenchMode,
    iters: usize,
    warmup: usize,
    save: Option<PathBuf>,
    baseline: Option<PathBuf>,
}

struct BenchSummary {
    n: usize,
    min: f64,
    mean: f64,
    median: f64,
    p90: f64,
    max: f64,
    variance: f64,
    stddev: f64,
}

/// A part of a benchmark that is timed on its own. The phases form two
/// groups. A file that is parsed to a single parse tree goes through the
/// five phases of the Success group. Every other file is in one phase of the
/// Failure group.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Phase {
    /// Input preparation: decoding the source into characters. Reading the
    /// file is not timed, because it measures the file system rather than
    /// the parser.
    Prepare,
    /// Arenas + parse tree builder + Parser allocation.
    Init,
    /// The GLL parse itself.
    Parse,
    /// SPPF → parse tree extraction.
    Tree,
    /// Teardown of all the above.
    Drop,
    /// A file with a parse error. The time of the phase is the time until the
    /// parser reported the error.
    Error,
    /// An ambiguous file. The time of the phase is the time of the parse.
    Amb,
    /// A file that could not be read. The file does not reach the parser, so
    /// the phase has no time.
    IoError,
}

impl Phase {
    const COUNT: usize = 8;
    const SUCCESS: [Phase; 5] = [
        Self::Prepare,
        Self::Init,
        Self::Parse,
        Self::Tree,
        Self::Drop,
    ];
    const FAILURE: [Phase; 3] = [Self::Error, Self::Amb, Self::IoError];

    fn name(self) -> &'static str {
        match self {
            Self::Prepare => "prepare",
            Self::Init => "init",
            Self::Parse => "parse",
            Self::Tree => "tree",
            Self::Drop => "drop",
            Self::Error => "error",
            Self::Amb => "amb",
            Self::IoError => "ioerror",
        }
    }
}

/// The time, the number of files, and the size of the files of one phase in
/// one pass.
#[derive(Clone, Copy, Default)]
struct PhaseStats {
    time: Duration,
    files: usize,
    bytes: u64,
}

/// The stats of one pass over the files of a benchmark, indexed by `Phase`.
type Pass = [PhaseStats; Phase::COUNT];

/// Adds a file of `bytes` bytes that took `time` in `phase` to `pass`.
fn record(pass: &mut Pass, phase: Phase, time: Duration, bytes: u64) {
    let stats = &mut pass[phase as usize];
    stats.time += time;
    stats.files += 1;
    stats.bytes += bytes;
}

/// Returns the time of the Success group in `pass`.
fn success_time(pass: &Pass) -> Duration {
    Phase::SUCCESS
        .iter()
        .map(|phase| pass[*phase as usize].time)
        .sum()
}

/// Returns the time of all phases in `pass`.
fn total_time(pass: &Pass) -> Duration {
    pass.iter().map(|stats| stats.time).sum()
}

fn as_ms(time: Duration) -> f64 {
    time.as_secs_f64() * 1000.0
}

/// Runs `parse_once` `warmup + iters` times, collecting per-phase timings.
/// Warmup samples are discarded. Prints summary stats for each phase and for
/// the total, which covers every phase but input preparation, then the
/// throughput (MB/s) at the median total. In the batch mode, the phases cover
/// the Success group, and the Failure group follows: its files and time by
/// status, then the time of all files. In the single mode, a file with a
/// parse error or an ambiguity is printed as its status and the stats of its
/// time, with no phases. If `config.save` is set, writes the raw samples as
/// JSON. If `config.baseline` is set, loads a prior run and reports the mean
/// delta on the reported total with a 95% CI on the difference (Welch's SE
/// for unequal variances).
fn run_benchmark(config: BenchConfig, mut parse_once: impl FnMut() -> Pass) -> io::Result<()> {
    // The time of each phase in each measured pass, indexed by `Phase`.
    let mut samples: [Vec<f64>; Phase::COUNT] = Default::default();
    // The time of the parsed files and of all files in each measured pass.
    // Neither counts input preparation, so the two reconcile: the time of
    // all files is the total plus the time of the files that failed.
    let mut total_samples = Vec::with_capacity(config.iters);
    let mut all_files_samples = Vec::with_capacity(config.iters);
    // The file counts and the sizes come from the last measured pass. Every
    // pass parses the same files.
    let mut last_pass = Pass::default();

    for i in 0..(config.warmup + config.iters) {
        let pass = parse_once();
        if i >= config.warmup {
            for (phase_samples, stats) in samples.iter_mut().zip(&pass) {
                phase_samples.push(as_ms(stats.time));
            }
            // Input preparation is measured but reported on its own, so it
            // is subtracted from both totals rather than from one of them.
            let prepare = pass[Phase::Prepare as usize].time;
            total_samples.push(as_ms(success_time(&pass) - prepare));
            all_files_samples.push(as_ms(total_time(&pass) - prepare));
            last_pass = pass;
        }
    }
    // A file of the Success group goes through all five phases of the group,
    // so each of the five has the files and the size of the group.
    let success = last_pass[Phase::Parse as usize];
    let failure_files: usize = Phase::FAILURE
        .iter()
        .map(|phase| last_pass[*phase as usize].files)
        .sum();

    // The Failure phase of the file of the single mode, when the file is in
    // one.
    let single_failure = match config.mode {
        BenchMode::Single => Phase::FAILURE
            .into_iter()
            .find(|phase| last_pass[*phase as usize].files > 0),
        BenchMode::Batch => None,
    };
    // The status of the run, the samples of the time it measures, and its
    // size. They are those of the reported phases, or those of the Failure
    // phase of the file in the single mode. `--save` writes them, and
    // `--baseline` compares the status and the samples.
    let (status, measured_samples, bytes_per_iter) = match single_failure {
        None => ("ok", &total_samples, success.bytes),
        Some(phase) => (
            phase.name(),
            &samples[phase as usize],
            last_pass[phase as usize].bytes,
        ),
    };
    let measured = summarize(measured_samples);

    let color = Color::for_stdout();
    let iterations = format!(
        "{} iteration{}{}",
        config.iters,
        if config.iters == 1 { "" } else { "s" },
        if config.warmup > 0 {
            format!(" (+{} warmup)", config.warmup)
        } else {
            String::new()
        },
    );
    println!();
    match config.mode {
        BenchMode::Single => println!(
            "Benchmark: {}, {} bytes per iteration (times in ms)",
            iterations,
            group_digits(&bytes_per_iter.to_string()),
        ),
        BenchMode::Batch => {
            println!("Benchmark: {}, times in ms", iterations);
            println!();
            println!(
                "Success: {} file{}, {} bytes per iteration",
                group_digits(&success.files.to_string()),
                if success.files == 1 { "" } else { "s" },
                group_digits(&bytes_per_iter.to_string()),
            );
        }
    }
    println!();
    if single_failure.is_some() {
        println!("Status: {}", status);
        println!();
        print_phase_header(&color);
        print_phase_row(status, &measured, ms_decimals(measured.max));
    } else {
        // Header, then a phase per row, a rule, and the total last since it
        // is the sum of the phases above it. Input preparation stands apart
        // below, since it is the one phase the total leaves out.
        print_phase_header(&color);
        let decimals = ms_decimals(measured.max);
        for phase in Phase::SUCCESS.into_iter().filter(|p| *p != Phase::Prepare) {
            let summary = summarize(&samples[phase as usize]);
            print_phase_row(phase.name(), &summary, decimals);
        }
        println!("  {}", "-".repeat(8 + 6 * 14));
        print_phase_row("total", &measured, decimals);
        println!();
        let prepare = summarize(&samples[Phase::Prepare as usize]);
        print_phase_row(Phase::Prepare.name(), &prepare, decimals);
        println!();
        println!(
            "The prepare phase (UTF-8 decoding) is not included in the total: the total\n\
             measures parsing from the decoded text in memory."
        );
        println!();
        let parse = summarize(&samples[Phase::Parse as usize]);
        println!(
            "Throughput (median): {:.2} MB/s total at {} ms, {:.2} MB/s parse-only",
            mb_per_s(bytes_per_iter, measured.median),
            fmt_ms(measured.median, decimals),
            mb_per_s(bytes_per_iter, parse.median),
        );
    }

    if config.mode == BenchMode::Batch && failure_files > 0 {
        let failure_bytes: u64 = Phase::FAILURE
            .iter()
            .map(|phase| last_pass[*phase as usize].bytes)
            .sum();
        let summaries = Phase::FAILURE.map(|phase| summarize(&samples[phase as usize]));
        let all_files = summarize(&all_files_samples);
        println!();
        println!(
            "Failure: {} file{}, {} bytes per iteration",
            group_digits(&failure_files.to_string()),
            if failure_files == 1 { "" } else { "s" },
            group_digits(&failure_bytes.to_string()),
        );
        println!();
        println!(
            "  {}{:<8}{:>8}{:>14}{:>14}{:>14}{:>14}{:>14}{:>14}{}",
            color.bold,
            "status",
            "files",
            "min",
            "mean",
            "stddev",
            "median",
            "p90",
            "max",
            color.reset
        );
        let decimals = ms_decimals(summaries.iter().map(|s| s.max).fold(0.0, f64::max));
        for (phase, summary) in Phase::FAILURE.into_iter().zip(&summaries) {
            print_failure_row(phase, last_pass[phase as usize].files, summary, decimals);
        }
        println!();
        println!(
            "All files (median): {} ms for {} files",
            fmt_ms(all_files.median, ms_decimals(all_files.max)),
            group_digits(&(success.files + failure_files).to_string()),
        );
    }

    if let Some(ref path) = config.save {
        let mut json = serde_json::json!({
            "files": success.files,
            "bytes": bytes_per_iter,
            "samples_ms": measured_samples,
        });
        for phase in Phase::SUCCESS {
            json[format!("{}_samples_ms", phase.name())] = samples[phase as usize].clone().into();
        }
        for phase in Phase::FAILURE {
            json[format!("{}_files", phase.name())] = last_pass[phase as usize].files.into();
            json[format!("{}_samples_ms", phase.name())] = samples[phase as usize].clone().into();
        }
        if config.mode == BenchMode::Single {
            json["status"] = status.into();
        }
        fs::write(path, serde_json::to_string_pretty(&json).unwrap())?;
        eprintln!("Saved baseline to {}", path.display());
    }

    if let Some(ref path) = config.baseline {
        let text = fs::read_to_string(path)?;
        let parsed: serde_json::Value = serde_json::from_str(&text).map_err(io::Error::other)?;
        // A baseline without a status is a batch run or predates the status.
        let baseline_status = parsed["status"].as_str().unwrap_or("ok");
        if baseline_status != status {
            return Err(io::Error::other(format!(
                "the baseline {} has the status {}, and this run has the status {}, so the two are not comparable",
                path.display(),
                baseline_status,
                status
            )));
        }
        // A baseline written by another build can hold the same keys with
        // different meanings, so require the phases this build writes. The
        // comparison reads only `samples_ms`, but a file missing a phase
        // came from a format whose `samples_ms` is a different quantity.
        for phase in Phase::SUCCESS.into_iter().chain(Phase::FAILURE) {
            let key = format!("{}_samples_ms", phase.name());
            if !parsed[&key].is_array() {
                return Err(io::Error::other(format!(
                    "the baseline {} has no {}, so it was written by another build and is not comparable",
                    path.display(),
                    key
                )));
            }
        }
        let baseline_samples: Vec<f64> = parsed["samples_ms"]
            .as_array()
            .ok_or_else(|| io::Error::other("baseline missing samples_ms array"))?
            .iter()
            .map(|v| v.as_f64().unwrap())
            .collect();
        let baseline = summarize(&baseline_samples);
        let delta = measured.mean - baseline.mean;
        let se =
            (measured.variance / measured.n as f64 + baseline.variance / baseline.n as f64).sqrt();
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

/// Prints the header of a phase table, bold on a terminal.
fn print_phase_header(color: &Color) {
    println!(
        "  {}{:<8}{:>14}{:>14}{:>14}{:>14}{:>14}{:>14}{}",
        color.bold, "phase", "min", "mean", "stddev", "median", "p90", "max", color.reset
    );
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

/// Prints the row of `phase` in the Failure table: the number of files, then
/// the stats of the time a pass spent on them. The stats are dashes for a
/// phase with no files and for `IoError`, which has no time.
fn print_failure_row(phase: Phase, files: usize, s: &BenchSummary, decimals: usize) {
    let stats = if files > 0 && phase != Phase::IoError {
        [s.min, s.mean, s.stddev, s.median, s.p90, s.max].map(|v| fmt_ms(v, decimals))
    } else {
        [(); 6].map(|_| "-".to_string())
    };
    println!(
        "  {:<8}{:>8}{:>14}{:>14}{:>14}{:>14}{:>14}{:>14}",
        phase.name(),
        group_digits(&files.to_string()),
        stats[0],
        stats[1],
        stats[2],
        stats[3],
        stats[4],
        stats[5],
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
pub(super) fn group_digits(digits: &str) -> String {
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
pub(super) fn mb_per_s(bytes: u64, ms: f64) -> f64 {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_list_keeps_the_order_and_skips_blank_lines() {
        assert_eq!(
            file_list("b.sql\n\na.sql\n  \nsub/c.sql\n"),
            [
                PathBuf::from("b.sql"),
                PathBuf::from("a.sql"),
                PathBuf::from("sub/c.sql"),
            ]
        );
    }

    #[test]
    fn phase_groups_cover_every_phase_once() {
        let mut indexes: Vec<usize> = Phase::SUCCESS
            .into_iter()
            .chain(Phase::FAILURE)
            .map(|phase| phase as usize)
            .collect();
        indexes.sort();
        assert_eq!(indexes, (0..Phase::COUNT).collect::<Vec<_>>());
    }

    #[test]
    fn record_adds_a_file_to_its_phase() {
        let ms = Duration::from_millis;
        let mut pass = Pass::default();
        // Two files of the Success group, 10 bytes each.
        for _ in 0..2 {
            record(&mut pass, Phase::Prepare, ms(1), 10);
            record(&mut pass, Phase::Init, ms(2), 10);
            record(&mut pass, Phase::Parse, ms(3), 10);
            record(&mut pass, Phase::Tree, ms(4), 10);
            record(&mut pass, Phase::Drop, ms(5), 10);
        }
        record(&mut pass, Phase::Error, ms(7), 30);
        record(&mut pass, Phase::Error, ms(8), 40);
        record(&mut pass, Phase::Amb, ms(9), 50);
        record(&mut pass, Phase::IoError, Duration::ZERO, 60);

        for phase in Phase::SUCCESS {
            assert_eq!(pass[phase as usize].files, 2, "{phase:?}");
            assert_eq!(pass[phase as usize].bytes, 20, "{phase:?}");
        }
        assert_eq!(pass[Phase::Parse as usize].time, ms(6));
        assert_eq!(success_time(&pass), ms(30));

        let error = pass[Phase::Error as usize];
        assert_eq!((error.time, error.files, error.bytes), (ms(15), 2, 70));
        let amb = pass[Phase::Amb as usize];
        assert_eq!((amb.time, amb.files, amb.bytes), (ms(9), 1, 50));
        let ioerror = pass[Phase::IoError as usize];
        assert_eq!(
            (ioerror.time, ioerror.files, ioerror.bytes),
            (Duration::ZERO, 1, 60)
        );

        // The time of all phases is the Success time plus the Failure times.
        assert_eq!(total_time(&pass), ms(54));
    }
}
