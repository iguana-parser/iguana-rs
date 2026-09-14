use std::{io, path::Path, time::Instant};

use crate::{
    arena::Arena,
    ids::NonterminalId,
    input::Input,
    parse_tree::is_ambiguous,
    parser::{GLLResult, Parser},
};

use super::{
    Color, args::Args, benchmark::mb_per_s, collect_files, start_nonterminal_id,
    start_nonterminal_name,
};

/// Runs `--dir`: parses every file under the directory, prints a status line
/// per file, and exits with failure when a file failed to parse, parsed
/// ambiguously, or could not be read.
pub(super) fn run<'i, 'arena, P: Parser<'i, 'arena>>(args: &Args) -> io::Result<()> {
    let dir = args.dir.as_deref().expect("--dir is set");
    let start_nonterminal_id = start_nonterminal_id::<P::Grammar>(start_nonterminal_name(args)?)?;
    let passed = run_batch::<P>(dir, args.ext.as_deref(), start_nonterminal_id, args.hist)?;
    if !passed {
        std::process::exit(1);
    }
    Ok(())
}

fn run_batch<'i, 'arena, P: Parser<'i, 'arena>>(
    dir: &Path,
    ext: Option<&str>,
    start_nonterminal_id: NonterminalId,
    hist_only: bool,
) -> io::Result<bool> {
    let color = Color::for_stdout();
    let mut files = Vec::new();
    collect_files(dir, ext, &mut files)?;
    files.sort();
    if !hist_only {
        println!(
            "{:<6}  {:<42}  {:<36}  PATH",
            "STATUS", "TIME (input, init, parse, tree, drop)", "REASON"
        );
    }
    let mut ok = 0usize;
    let mut ambiguous = 0usize;
    let mut failed = 0usize;
    let mut errs = 0usize;
    let mut total_input_ms: f64 = 0.0;
    let mut total_init_ms: f64 = 0.0;
    let mut total_parse_ms: f64 = 0.0;
    let mut total_tree_ms: f64 = 0.0;
    let mut total_drop_ms: f64 = 0.0;
    let mut max_total_ms: f64 = 0.0;
    let mut total_bytes: u64 = 0;
    let mut per_file: Vec<(u64, f64)> = Vec::new();
    #[cfg(feature = "instrument")]
    let mut corpus_stats = crate::instrument::Stats::new();
    for path in &files {
        let rel = path.strip_prefix(dir).unwrap_or(path.as_path());
        let input_start = Instant::now();
        let input = match Input::try_from(path.as_path()) {
            Ok(input) => input,
            Err(e) => {
                errs += 1;
                if !hist_only {
                    let reason = format!("IO Error: {}", e);
                    println!(
                        "{}{:<6}{}  {:<42}  {:<36}  {}",
                        color.red,
                        "ERR",
                        color.reset,
                        "-",
                        reason,
                        rel.display()
                    );
                }
                continue;
            }
        };
        let input_ms = input_start.elapsed().as_secs_f64() * 1000.0;
        let bytes = input.len() as u64;
        let init_start = Instant::now();
        let tree_arena = Arena::new();
        let parser_arena = Arena::new();
        let mut parser = P::ConcreteParser::<'_, '_>::new(&input, &parser_arena);
        let init_ms = init_start.elapsed().as_secs_f64() * 1000.0;
        match parser.run(start_nonterminal_id) {
            GLLResult::Success(success) => {
                let parse_ms = success.duration.as_secs_f64() * 1000.0;
                let ambig = is_ambiguous(&parser, success.sppf_node_id);
                let tc_start = Instant::now();
                {
                    let tree = parser.build_tree(success.sppf_node_id, &tree_arena);
                    std::hint::black_box(tree);
                }
                let tree_ms = tc_start.elapsed().as_secs_f64() * 1000.0;
                #[cfg(feature = "instrument")]
                corpus_stats.merge(parser.record_stats());
                let drop_start = Instant::now();
                drop(parser);
                drop(parser_arena);
                drop(tree_arena);
                drop(input);
                let drop_ms = drop_start.elapsed().as_secs_f64() * 1000.0;
                let total_ms = input_ms + init_ms + parse_ms + tree_ms + drop_ms;
                if ambig {
                    ambiguous += 1;
                } else {
                    ok += 1;
                }
                total_input_ms += input_ms;
                total_init_ms += init_ms;
                total_parse_ms += parse_ms;
                total_tree_ms += tree_ms;
                total_drop_ms += drop_ms;
                total_bytes += bytes;
                per_file.push((bytes, parse_ms));
                if total_ms > max_total_ms {
                    max_total_ms = total_ms;
                }
                if !hist_only {
                    let time = format!(
                        "{} ms ({} ms, {} ms, {} ms, {} ms, {} ms)",
                        total_ms as u128,
                        input_ms as u128,
                        init_ms as u128,
                        parse_ms as u128,
                        tree_ms as u128,
                        drop_ms as u128
                    );
                    let (label, code) = if ambig {
                        ("AMB", color.red)
                    } else {
                        ("OK", color.green)
                    };
                    println!(
                        "{}{:<6}{}  {:<42}  {:<36}  {}",
                        code,
                        label,
                        color.reset,
                        time,
                        "-",
                        rel.display()
                    );
                }
            }
            GLLResult::Failure(error) => {
                let (line, column) = input.line_column(error.input_index);
                failed += 1;
                if !hist_only {
                    let reason = format!("Parse error at line {}, column {}", line + 1, column + 1);
                    println!(
                        "{}{:<6}{}  {:<42}  {:<36}  {}",
                        color.red,
                        "FAIL",
                        color.reset,
                        "-",
                        reason,
                        rel.display()
                    );
                }
            }
        }
    }
    let total_ms = total_input_ms + total_init_ms + total_parse_ms + total_tree_ms + total_drop_ms;
    let parsed = ok + ambiguous;
    let avg_ms = if parsed > 0 {
        total_ms / parsed as f64
    } else {
        0.0
    };
    let throughput = mb_per_s(total_bytes, total_parse_ms);
    let throughput_total = mb_per_s(total_bytes, total_ms);
    if !hist_only {
        println!();
        println!(
            "Parsed {} files: {} OK, {} ambiguous, {} failed, {} errors",
            files.len(),
            ok,
            ambiguous,
            failed,
            errs
        );
        println!(
            "Total {:.0} ms (input {:.0}, init {:.0}, parse {:.0}, tree {:.0}, drop {:.0}); avg {:.1} ms, max {:.0} ms",
            total_ms,
            total_input_ms,
            total_init_ms,
            total_parse_ms,
            total_tree_ms,
            total_drop_ms,
            avg_ms,
            max_total_ms
        );
        println!(
            "Throughput on {} successful parses ({} bytes): {:.2} MB/s parse only, {:.2} MB/s total",
            parsed, total_bytes, throughput, throughput_total
        );
        if !per_file.is_empty() {
            let buckets: &[(&str, u64, u64)] = &[
                ("< 1 KB", 0, 1024),
                ("1-4 KB", 1024, 4 * 1024),
                ("4-16 KB", 4 * 1024, 16 * 1024),
                ("16-64 KB", 16 * 1024, 64 * 1024),
                ("64-256 KB", 64 * 1024, 256 * 1024),
                ("> 256 KB", 256 * 1024, u64::MAX),
            ];
            println!();
            println!("[throughput by file size]");
            println!(
                "  {:<12} {:>6} {:>12} {:>12} {:>8} {:>12} {:>10}",
                "size class", "files", "bytes", "parse ms", "MB/s", "median ms", "p90 ms"
            );
            for (label, lo, hi) in buckets {
                let mut bucket: Vec<&(u64, f64)> = per_file
                    .iter()
                    .filter(|(b, _)| *b >= *lo && *b < *hi)
                    .collect();
                if bucket.is_empty() {
                    continue;
                }
                let count = bucket.len();
                let bucket_bytes: u64 = bucket.iter().map(|(b, _)| *b).sum();
                let bucket_ms: f64 = bucket.iter().map(|(_, ms)| *ms).sum();
                let bucket_mbs = mb_per_s(bucket_bytes, bucket_ms);
                bucket.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
                let median_ms = bucket[count / 2].1;
                let p90_idx = (((count as f64) * 0.9) as usize).min(count - 1);
                let p90_ms = bucket[p90_idx].1;
                println!(
                    "  {:<12} {:>6} {:>12} {:>12.1} {:>8.2} {:>12.3} {:>10.3}",
                    label, count, bucket_bytes, bucket_ms, bucket_mbs, median_ms, p90_ms
                );
            }
        }
    }
    #[cfg(feature = "instrument")]
    {
        if !hist_only {
            println!();
        }
        println!("[stats] aggregated across {} successful parses", parsed);
        println!("{}", corpus_stats);
    }
    Ok(failed == 0 && errs == 0 && ambiguous == 0)
}
