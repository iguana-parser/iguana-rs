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

/// Runs `--benchmark`. A single file (the positional argument) is parsed
/// `--iters` times after `--warmup` discarded runs, with defaults of 100 and
/// 10. A `--dir` or, without one, the corpora listed in `repos.txt` are
/// parsed as whole passes, with defaults of 3 and 0.
pub(super) fn run<'i, 'arena, P: Parser<'i, 'arena>>(args: &Args) -> io::Result<()> {
    if let Some(file) = args.file.as_ref() {
        let start_nonterminal_id =
            start_nonterminal_id::<P::Grammar>(start_nonterminal_name(args)?)?;
        let config = BenchConfig {
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
            bench_parse_file::<P>(
                &file_path,
                start_nonterminal_id,
                &mut tree_arena,
                &mut parser_arena,
            )
            .expect("benchmark input could not be read, failed to parse, or is ambiguous")
        });
    }
    let config = BenchConfig {
        iters: args.iters.unwrap_or(3) as usize,
        warmup: args.warmup.unwrap_or(0) as usize,
        save: args.save.clone(),
        baseline: args.baseline.clone(),
    };
    let mut groups: Vec<(String, Vec<(PathBuf, NonterminalId)>)> = Vec::new();
    if let Some(dir) = args.dir.as_ref() {
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
    if args.dir.is_none() {
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
        let mut input = Duration::ZERO;
        let mut init = Duration::ZERO;
        let mut parse = Duration::ZERO;
        let mut tree = Duration::ZERO;
        let mut drop = Duration::ZERO;
        let mut bytes = 0u64;
        for (label, files) in &groups {
            eprint!("  {}...", label);
            let _ = io::stderr().flush();
            let mut source_time = Duration::ZERO;
            for (path, start_nonterminal_id) in files {
                if let Some(t) = bench_parse_file::<P>(
                    path,
                    *start_nonterminal_id,
                    &mut tree_arena,
                    &mut parser_arena,
                ) {
                    input += t.input;
                    init += t.init;
                    parse += t.parse;
                    tree += t.tree;
                    drop += t.drop;
                    bytes += t.bytes;
                    source_time += t.input + t.init + t.parse + t.tree + t.drop;
                }
            }
            let pad = " ".repeat(max_label - label.len() + 1);
            eprintln!(
                "{}{} ms",
                pad,
                group_digits(&format!("{:.0}", source_time.as_secs_f64() * 1000.0))
            );
        }
        let run_ms = (input + init + parse + tree + drop).as_secs_f64() * 1000.0;
        eprintln!(
            "{} {} completed in {} ms",
            kind,
            run_num,
            group_digits(&format!("{:.0}", run_ms))
        );
        PhaseTimings {
            input,
            init,
            parse,
            tree,
            drop,
            bytes,
        }
    })
}

/// Parses one file under a benchmark, returning its per-phase timings, or
/// `None` when the file should not count toward the measurement: it cannot
/// be read, it fails to parse, or it parses ambiguously. A benchmark times
/// clean single-tree parses, so a whole-corpus run skips the rest rather
/// than mixing their work in. The input is reloaded so the `input` phase is
/// measured. The caller's arenas are reused across files and reset
/// between them, so they keep their chunks and teardown is a bulk
/// reset rather than a per-file free, the pattern the arena is built for.
fn bench_parse_file<'i, 'arena, P: Parser<'i, 'arena>>(
    path: &Path,
    start_nonterminal_id: NonterminalId,
    tree_arena: &mut Arena,
    parser_arena: &mut Arena,
) -> Option<PhaseTimings> {
    let input_start = Instant::now();
    let input = Input::try_from(path).ok()?;
    let input_time = input_start.elapsed();
    let bytes = input.len() as u64;
    let init_start = Instant::now();
    let mut parser = P::ConcreteParser::<'_, '_>::new(&input, parser_arena);
    let init = init_start.elapsed();
    let GLLResult::Success(success) = parser.run(start_nonterminal_id) else {
        drop(parser);
        parser_arena.reset();
        return None;
    };
    let parse = success.duration;
    if is_ambiguous(&parser, success.sppf_node_id) {
        drop(parser);
        parser_arena.reset();
        return None;
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
    Some(PhaseTimings {
        input: input_time,
        init,
        parse,
        tree,
        drop,
        bytes,
    })
}

struct BenchConfig {
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

/// Per-iteration phase breakdown. Total wall time is the sum of all phases.
///   - `input`: file read + line/column offset table construction
///   - `init`: arenas + parse tree builder + Parser allocation
///   - `parse`: the GLL parse itself
///   - `tree`: SPPF → parse tree extraction
///   - `drop`: teardown of all the above
struct PhaseTimings {
    input: Duration,
    init: Duration,
    parse: Duration,
    tree: Duration,
    drop: Duration,
    /// Bytes of input parsed in this iteration (used for throughput).
    bytes: u64,
}

/// Runs `parse_once` `warmup + iters` times, collecting per-phase timings.
/// Warmup samples are discarded. Prints summary stats for total and each
/// phase plus throughput (MB/s) at the median total. If `config.save` is
/// set, writes the raw samples as JSON. If `config.baseline` is set, loads
/// a prior run and reports the mean delta on total time with a 95% CI on
/// the difference (Welch's SE for unequal variances).
fn run_benchmark(
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
