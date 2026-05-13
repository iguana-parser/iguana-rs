use std::fs;
use std::io;
use std::path::PathBuf;
use std::time::Instant;

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

/// Runs `parse_once` `warmup + iters` times, timing each call. Warmup
/// samples are discarded. Prints summary stats. If `config.save` is set,
/// writes the raw samples as JSON. If `config.baseline` is set, loads a
/// prior run and reports the mean delta with a 95% CI on the difference
/// (Welch's SE for unequal variances).
pub fn run_benchmark(config: BenchConfig, mut parse_once: impl FnMut()) -> io::Result<()> {
    let mut samples_ms = Vec::with_capacity(config.iters);
    for i in 0..(config.warmup + config.iters) {
        let start = Instant::now();
        parse_once();
        let elapsed = start.elapsed();
        if i >= config.warmup {
            samples_ms.push(elapsed.as_secs_f64() * 1000.0);
        }
    }

    let summary = summarize(&samples_ms);
    println!(
        "Benchmark: {} samples ({} warmup)",
        config.iters, config.warmup
    );
    println!("  min    = {:.3} ms", summary.min);
    println!(
        "  mean   = {:.3} ms (±{:.3} stddev)",
        summary.mean, summary.stddev
    );
    println!("  median = {:.3} ms", summary.median);
    println!("  p90    = {:.3} ms", summary.p90);
    println!("  max    = {:.3} ms", summary.max);

    if let Some(ref path) = config.save {
        let json = serde_json::json!({
            "version": 1,
            "samples_ms": samples_ms,
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
        let delta = summary.mean - baseline.mean;
        let se =
            (summary.variance / summary.n as f64 + baseline.variance / baseline.n as f64).sqrt();
        let ci_half = 1.96 * se;
        let pct = 100.0 * delta / baseline.mean;
        println!();
        println!(
            "Compared to baseline {} ({} samples, mean {:.3} ms):",
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
