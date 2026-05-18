use std::fs;
use std::io;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

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
///   - `init`: ParseContext + parse tree builder + Parser allocation
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

    println!(
        "Benchmark: {} samples ({} warmup), {} bytes",
        config.iters, config.warmup, bytes_per_iter
    );
    println!(
        "  {:<6} {:>8} {:>8} {:>8} {:>8} {:>8} {:>8}",
        "phase", "min", "mean", "stddev", "median", "p90", "max"
    );
    print_phase_row("total", &total);
    print_phase_row("input", &input);
    print_phase_row("init", &init);
    print_phase_row("parse", &parse);
    print_phase_row("tree", &tree);
    print_phase_row("drop", &drop);
    println!(
        "  throughput: {:.2} MB/s (median total), {:.2} MB/s (median parse only)",
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

fn print_phase_row(name: &str, s: &BenchSummary) {
    println!(
        "  {:<6} {:>8.3} {:>8.3} {:>8.3} {:>8.3} {:>8.3} {:>8.3}",
        name, s.min, s.mean, s.stddev, s.median, s.p90, s.max
    );
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
