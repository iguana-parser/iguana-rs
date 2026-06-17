use clap::{Parser as ClapParser, ValueEnum as ClapValueEnum};
use follow_restriction_multiple::{
    grammar_data::{NONTERMINAL_DISPLAY_ORDER, NONTERMINALS, SLOTS, TERMINALS, nonterminal_id},
    parse_tree::{
        FollowRestrictionMultipleParseTreeBuilder, create_parse_tree, to_json, to_sexpr_with,
    },
    parser::FollowRestrictionMultipleParser,
};
#[cfg(feature = "debug-trace")]
use iguana_runtime::trace::TraceEvent;
use iguana_runtime::{
    cli,
    ids::NonterminalId,
    input::Input,
    parse_tree::{ParseContext, SexprOptions, is_ambiguous},
    parser::{ParseResult, Parser},
    visualization::{
        dot::write_svg,
        gss::{build_gss_dot_graph, render_gss},
        sppf::{build_sppf_graph, write_sppf_dot},
    },
};
#[cfg(feature = "profile")]
use pprof::ProfilerGuardBuilder;
use std::{
    fs::{self, File},
    io::{self, BufWriter, Write},
    path::{Path, PathBuf},
    time::Instant,
};
#[derive(Clone, Copy, Default, ClapValueEnum)]
enum TraceFormat {
    #[default]
    Text,
    Json,
}
#[derive(Clone, Copy, PartialEq, Eq, ClapValueEnum)]
enum VisTarget {
    Sppf,
    Gss,
}
#[derive(ClapParser)]
#[command(name = "parser")]
#[command(about = "Parse a file and generate visualization")]
#[command(arg_required_else_help = true)]
struct Cli {
    /// Input file to parse (required unless --list-nonterminals or --dir is used)
    file: Option<PathBuf>,
    /// Directory to recursively parse all files in. Reports per-file
    /// success/failure with timings, plus a summary at the end.
    #[arg(long, value_name = "DIR", conflicts_with = "file")]
    dir: Option<PathBuf>,
    /// File extension filter for --dir (e.g. "java"). Without it, all files are parsed.
    #[arg(long, value_name = "EXT", requires = "dir")]
    ext: Option<String>,
    /// The nonterminal to start parsing from (required unless --list-nonterminals is used)
    #[arg(short = 'n', long = "nonterminal", value_name = "NAME")]
    start_nonterminal: Option<String>,
    /// List simple nonterminals (one per line) and exit
    #[arg(long)]
    list_nonterminals: bool,
    /// Interactive mode: read inputs from stdin and print the parse tree
    /// for each. Requires --nonterminal; no input file is used.
    #[arg(long)]
    repl: bool,
    /// Write symbol table (all nonterminals) as JSON to the specified file
    #[arg(long, value_name = "FILE")]
    write_symbols: Option<PathBuf>,
    /// Enable trace output (writes to stdout or specified file)
    #[arg(long, value_name = "FILE")]
    trace: Option<Option<PathBuf>>,
    /// Output format for trace (text or json)
    #[arg(long, value_enum, default_value_t, requires = "trace")]
    format: TraceFormat,
    /// Generate visualization as SVG (sppf or gss)
    #[arg(long, value_enum)]
    vis: Option<VisTarget>,
    /// Write SPPF as JSON to the specified file
    #[arg(long, value_name = "FILE")]
    write_sppf: Option<PathBuf>,
    /// Write GSS graph as JSON for visualization (nodes with labels + edges).
    /// Used by Terrarium for Cytoscape.js graph rendering.
    #[arg(long, value_name = "FILE")]
    write_gss: Option<PathBuf>,
    /// Write GSS nodes as JSON for trace replay (normalized with IDs).
    /// Used by Terrarium debugger to resolve GssNodeId to (nonterminal, input_index).
    #[arg(long, value_name = "FILE")]
    write_gss_nodes: Option<PathBuf>,
    /// Write parse tree as JSON for visualization.
    /// Used by Terrarium for parse tree rendering.
    #[arg(long, value_name = "FILE")]
    write_parse_tree: Option<PathBuf>,
    /// Profile the parser by running it N times in a loop under a
    /// sampling profiler, then write a flamegraph SVG.
    /// Requires the "profile" feature: cargo build --features profile
    #[arg(long, value_name = "N")]
    profile: Option<u32>,
    /// Output path for the flamegraph SVG (used with --profile).
    #[arg(long, value_name = "FILE", default_value = "flamegraph.svg")]
    profile_output: PathBuf,
    /// Benchmark mode: run the parser many times and report timing
    /// statistics (min, mean, median, p90, max, stddev). In-process,
    /// per-iteration sampling — same shape as criterion.
    #[arg(long)]
    benchmark: bool,
    /// Number of measured iterations for --benchmark (default 100).
    #[arg(long, value_name = "N", default_value_t = 100)]
    iters: u32,
    /// Number of warmup iterations before measurement (default 10).
    #[arg(long, value_name = "N", default_value_t = 10)]
    warmup: u32,
    /// Save benchmark samples to a JSON file. Pairs with --baseline
    /// for A/B comparison across runs.
    #[arg(long, value_name = "FILE")]
    save: Option<PathBuf>,
    /// Compare benchmark results against a saved baseline JSON.
    /// Reports the mean delta with a 95% CI on the difference;
    /// flags the run as improved/regressed/no-change.
    #[arg(long, value_name = "FILE")]
    baseline: Option<PathBuf>,
    /// Write parser stats (counters + histograms) as JSON.
    /// Requires the "instrument" feature.
    #[arg(long, value_name = "FILE")]
    write_stats: Option<PathBuf>,
    /// Write parse result as JSON: on success includes timings,
    /// on failure includes error location and message.
    #[arg(long, value_name = "FILE")]
    write_result: Option<PathBuf>,
    /// Suppress the parse-tree dump on stdout. Status messages
    /// (timing, errors) still go to stderr; redirect with `2>/dev/null`
    /// to silence them too.
    #[arg(short, long)]
    quiet: bool,
    /// Include layout (whitespace, comments) nodes in the parse-tree
    /// output. False by default.
    #[arg(long)]
    show_layout: bool,
    /// Show empty optionals and repetitions (`X?`, `X*` that matched
    /// nothing) in the parse-tree output. False by default.
    #[arg(long)]
    show_empty: bool,
    /// Show wrapper nodes (the `@Start` wrapper, optionals, anonymous
    /// groups, and alternations) in the parse-tree output. False by
    /// default.
    #[arg(long)]
    show_wrappers: bool,
    /// Print only the parser stats histogram and exit. Suppresses
    /// the parse-tree dump, the "Parse success" line, and (with
    /// `--dir`) the per-file timing lines and the aggregate
    /// timing summary. Requires the `instrument` feature.
    #[arg(long)]
    hist: bool,
    /// Golden-file testing: compare each input's output against its
    /// sibling `X.sexpr`. Works with a single file or `--dir` (which
    /// then requires `--ext`). Goldens hold the parse-tree s-expression
    /// on success or a `Parse error at ...` line on failure.
    #[arg(long, conflicts_with = "benchmark", conflicts_with = "profile")]
    check_sexpr: bool,
    /// Golden-file testing: write each input's output to its sibling
    /// `X.sexpr`, overwriting. Same input rules as `--check-sexpr`.
    #[arg(
        long,
        conflicts_with = "benchmark",
        conflicts_with = "profile",
        conflicts_with = "check_sexpr"
    )]
    regenerate_sexpr: bool,
    /// Print golden diffs in full instead of truncating past 200 lines.
    #[arg(long, requires = "check_sexpr")]
    full_diff: bool,
}
#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;
fn main() -> Result<(), io::Error> {
    #[cfg(feature = "dhat-heap")]
    let _profiler = dhat::Profiler::new_heap();
    let args = Cli::parse();
    #[cfg(not(feature = "instrument"))]
    if args.hist {
        eprintln!(
            "Error: --hist requires the `instrument` feature. Recompile with --features instrument."
        );
        std::process::exit(1);
    }
    if args.list_nonterminals {
        for name in NONTERMINAL_DISPLAY_ORDER.iter() {
            println!("{}", name);
        }
        return Ok(());
    }
    if let Some(ref path) = args.write_symbols {
        let symbols = cli::Symbols {
            nonterminals: NONTERMINALS
                .iter()
                .map(|nt| nt.display.to_string())
                .collect(),
            terminals: TERMINALS.iter().map(|t| t.name.to_string()).collect(),
            slots: SLOTS.iter().map(|s| s.display_name.to_string()).collect(),
        };
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);
        writeln!(
            writer,
            "{}",
            serde_json::to_string_pretty(&symbols).unwrap()
        )?;
        return Ok(());
    }
    if args.check_sexpr || args.regenerate_sexpr {
        let mode = if args.regenerate_sexpr {
            cli::GoldenMode::Regenerate
        } else {
            cli::GoldenMode::Check
        };
        let start_nonterminal_name = args.start_nonterminal.as_ref().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "--nonterminal is required for parsing",
            )
        })?;
        let start_nonterminal_id = resolve_start_nonterminal(start_nonterminal_name)?;
        if args.write_parse_tree.is_some()
            || args.write_sppf.is_some()
            || args.write_gss.is_some()
            || args.write_gss_nodes.is_some()
            || args.write_result.is_some()
            || args.write_stats.is_some()
            || args.vis.is_some()
            || args.trace.is_some()
        {
            eprintln!(
                "Warning: output flags (--write-parse-tree, --write-sppf, --write-gss, --write-gss-nodes, --write-result, --write-stats, --vis, --trace) are ignored in golden mode."
            );
        }
        let mut inputs = Vec::new();
        if let Some(dir) = args.dir.as_ref() {
            let ext = args . ext . as_deref () . ok_or_else (|| io :: Error :: new (io :: ErrorKind :: InvalidInput , "--ext is required with --dir for golden testing (so .sexpr goldens are not parsed as inputs)" ,)) ? ;
            collect_files(dir, Some(ext), &mut inputs)?;
            inputs.sort();
        } else if let Some(file) = args.file.as_ref() {
            inputs.push(file.clone());
        } else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "golden testing requires an input file or --dir",
            ));
        }
        let sexpr_options = SexprOptions {
            show_layout: args.show_layout,
            show_empty: args.show_empty,
            show_wrappers: args.show_wrappers,
        };
        let passed = cli::run_golden(
            mode,
            inputs,
            args.dir.as_deref(),
            args.quiet,
            args.full_diff,
            |path| {
                let input = Input::try_from(path)?;
                let ctx = ParseContext::new();
                let parse_tree_builder = FollowRestrictionMultipleParseTreeBuilder::new(&ctx);
                let mut parser = FollowRestrictionMultipleParser::new(&input, start_nonterminal_id);
                let content = match parser.run() {
                    ParseResult::Success(success) => {
                        let tree = create_parse_tree(
                            success.sppf_node_id,
                            start_nonterminal_id,
                            &parser,
                            &parse_tree_builder,
                        );
                        to_sexpr_with(tree, sexpr_options)
                    }
                    ParseResult::Failure(error) => {
                        let (line, column, message) = parser.format_error(&error);
                        format!(
                            "Parse error at line {}, col {}: {}\n",
                            line + 1,
                            column + 1,
                            message
                        )
                    }
                };
                Ok(content)
            },
        )?;
        if !passed {
            std::process::exit(1);
        }
        return Ok(());
    }
    if let Some(dir) = args.dir.as_ref() {
        let start_nonterminal_name = args.start_nonterminal.as_ref().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "--nonterminal is required for parsing",
            )
        })?;
        let start_nonterminal_id = resolve_start_nonterminal(start_nonterminal_name)?;
        return run_batch(dir, args.ext.as_deref(), start_nonterminal_id, args.hist);
    }
    if args.repl {
        let start_nonterminal_name = args.start_nonterminal.as_ref().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "--nonterminal is required for parsing",
            )
        })?;
        let start_nonterminal_id = resolve_start_nonterminal(start_nonterminal_name)?;
        let sexpr_options = SexprOptions {
            show_layout: args.show_layout,
            show_empty: args.show_empty,
            show_wrappers: args.show_wrappers,
        };
        cli::run_repl(sexpr_options, |text, sexpr_options| {
            let input = Input::from(text);
            let ctx = ParseContext::new();
            let parse_tree_builder = FollowRestrictionMultipleParseTreeBuilder::new(&ctx);
            let mut parser = FollowRestrictionMultipleParser::new(&input, start_nonterminal_id);
            match parser.run() {
                ParseResult::Success(success) => {
                    let node_id = success.sppf_node_id;
                    let ambiguous = is_ambiguous(&parser, node_id);
                    let tree = create_parse_tree(
                        node_id,
                        start_nonterminal_id,
                        &parser,
                        &parse_tree_builder,
                    );
                    cli::ReplOutcome::Parsed {
                        tree: to_sexpr_with(tree, sexpr_options),
                        ambiguous,
                    }
                }
                ParseResult::Failure(error) => {
                    let (line, column, message) = parser.format_error(&error);
                    cli::ReplOutcome::Failed {
                        message: format!("Parse failed at line {line}, column {column}: {message}"),
                    }
                }
            }
        });
        return Ok(());
    }
    let file = args.file.ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "Input file is required for parsing",
        )
    })?;
    let start_nonterminal_name = args.start_nonterminal.ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "--nonterminal is required for parsing",
        )
    })?;
    #[cfg(not(feature = "debug-trace"))]
    if args.trace.is_some() {
        eprintln!(
            "Warning: --trace flag ignored. Recompile with `--features debug-trace` to enable tracing."
        );
    }
    let input = Input::try_from(file.as_path())?;
    let start_nonterminal_id = resolve_start_nonterminal(&start_nonterminal_name)?;
    if args.benchmark {
        let config = cli::BenchConfig {
            iters: args.iters as usize,
            warmup: args.warmup as usize,
            save: args.save.clone(),
            baseline: args.baseline.clone(),
        };
        let file_path = file.clone();
        return cli::run_benchmark(config, move || {
            let input_start = Instant::now();
            let input = Input::try_from(file_path.as_path()).expect("failed to load input");
            let input_time = input_start.elapsed();
            let bytes = input.len() as u64;
            let init_start = Instant::now();
            let ctx = ParseContext::new();
            let parse_tree_builder = FollowRestrictionMultipleParseTreeBuilder::new(&ctx);
            let mut parser = FollowRestrictionMultipleParser::new(&input, start_nonterminal_id);
            let init = init_start.elapsed();
            let (parse, tree) = match parser.run() {
                ParseResult::Success(success) => {
                    let parse = success.duration;
                    let tree_start = Instant::now();
                    {
                        let tree = create_parse_tree(
                            success.sppf_node_id,
                            start_nonterminal_id,
                            &parser,
                            &parse_tree_builder,
                        );
                        std::hint::black_box(tree);
                    }
                    (parse, tree_start.elapsed())
                }
                ParseResult::Failure(error) => {
                    let (line, column, message) = parser.format_error(&error);
                    panic!("Parse failed at line {line} column {column}: {message}");
                }
            };
            let drop_start = Instant::now();
            drop(parser);
            drop(parse_tree_builder);
            drop(ctx);
            drop(input);
            let drop = drop_start.elapsed();
            cli::PhaseTimings {
                input: input_time,
                init,
                parse,
                tree,
                drop,
                bytes,
            }
        });
    }
    #[cfg(feature = "profile")]
    if let Some(iterations) = args.profile {
        let guard = ProfilerGuardBuilder::default()
            .frequency(999)
            .build()
            .unwrap();
        for _ in 0..iterations {
            let ctx = ParseContext::new();
            let parse_tree_builder = FollowRestrictionMultipleParseTreeBuilder::new(&ctx);
            let mut parser = FollowRestrictionMultipleParser::new(&input, start_nonterminal_id);
            let result = parser.run();
            if let ParseResult::Success(success) = result {
                let _ = create_parse_tree(
                    success.sppf_node_id,
                    start_nonterminal_id,
                    &parser,
                    &parse_tree_builder,
                );
            }
        }
        let report = guard.report().build().unwrap();
        let file = File::create(&args.profile_output)?;
        report.flamegraph(&file).unwrap();
        eprintln!("Flamegraph written to {}", args.profile_output.display());
        return Ok(());
    }
    #[cfg(not(feature = "profile"))]
    if args.profile.is_some() {
        eprintln!(
            "Warning: --profile flag ignored. Recompile with `--features profile` to enable profiling."
        );
    }
    let ctx = ParseContext::new();
    let mut parser = FollowRestrictionMultipleParser::new(&input, start_nonterminal_id);
    #[cfg(feature = "debug-trace")]
    if args.trace.is_some() {
        parser.trace_events = Some(vec![]);
    }
    let parse_tree_builder = FollowRestrictionMultipleParseTreeBuilder::new(&ctx);
    let result = parser.run();
    #[cfg(feature = "debug-trace")]
    if let Some(ref trace_events) = parser.trace_events {
        write_trace_events(trace_events, &parser, &args.trace, args.format)?;
    }
    match result {
        ParseResult::Success(parse_success) => {
            let node_id = parse_success.sppf_node_id;
            if let Some(ref path) = args.write_sppf {
                let sppf = build_sppf_graph(&parser, node_id);
                let file = File::create(path)?;
                let mut writer = BufWriter::new(file);
                writeln!(writer, "{}", serde_json::to_string(&sppf).unwrap())?;
            }
            if let Some(ref path) = args.write_gss {
                let gss = build_gss_dot_graph(&parser);
                let file = File::create(path)?;
                let mut writer = BufWriter::new(file);
                writeln!(writer, "{}", serde_json::to_string(&gss).unwrap())?;
            }
            if let Some(ref path) = args.write_gss_nodes {
                let gss_nodes: Vec<_> = parser.gss_nodes().collect();
                let file = File::create(path)?;
                let mut writer = BufWriter::new(file);
                writeln!(writer, "{}", serde_json::to_string(&gss_nodes).unwrap())?;
            }
            let tc_start = Instant::now();
            let parse_tree_opt = if args.write_parse_tree.is_some()
                || args.write_result.is_some()
                || (args.write_sppf.is_none()
                    && args.write_gss.is_none()
                    && args.vis.is_none()
                    && args.trace.is_none())
            {
                Some(create_parse_tree(
                    node_id,
                    start_nonterminal_id,
                    &parser,
                    &parse_tree_builder,
                ))
            } else {
                None
            };
            let tree_construction_ms = parse_tree_opt
                .as_ref()
                .map(|_| tc_start.elapsed().as_millis());
            if let (Some(path), Some(parse_tree)) =
                (args.write_parse_tree.as_ref(), parse_tree_opt.as_ref())
            {
                let json = to_json(*parse_tree);
                let file = File::create(path)?;
                let mut writer = BufWriter::new(file);
                writeln!(writer, "{}", json)?;
            }
            if let Some(ref path) = args.write_result {
                let result = cli::ParseResult::Success(cli::ParseSuccess {
                    parse_ms: parse_success.duration.as_millis() as u64,
                    tree_construction_ms: tree_construction_ms.map(|ms| ms as u64),
                });
                let file = File::create(path)?;
                let mut writer = BufWriter::new(file);
                writeln!(writer, "{}", serde_json::to_string(&result).unwrap())?;
            }
            match args.vis {
                Some(VisTarget::Gss) => {
                    let path = Path::new("gss.dot");
                    render_gss(&parser, path)?;
                    write_svg(path)?;
                    eprintln!("GSS visualization generated: gss.svg");
                }
                Some(VisTarget::Sppf) => {
                    let path = Path::new("sppf.dot");
                    write_sppf_dot(&parser, node_id, path)?;
                    write_svg(path)?;
                    eprintln!("SPPF visualization generated: sppf.svg");
                }
                None => {}
            }
            if !args.hist {
                eprintln!("Parse success in {}ms", parse_success.duration.as_millis());
            }
            if !args.quiet
                && !args.hist
                && args.write_parse_tree.is_none()
                && args.write_sppf.is_none()
                && args.write_gss.is_none()
                && args.vis.is_none()
                && args.trace.is_none()
            {
                if let Some(ref parse_tree) = parse_tree_opt {
                    let sexpr_options = SexprOptions {
                        show_layout: args.show_layout,
                        show_empty: args.show_empty,
                        show_wrappers: args.show_wrappers,
                    };
                    println!("{}", to_sexpr_with(*parse_tree, sexpr_options));
                }
            }
        }
        ParseResult::Failure(error) => {
            let (line, column, message) = parser.format_error(&error);
            eprintln!("Parse failed at line {line}, column {column}: {message}");
            if let Some(ref path) = args.write_result {
                let result = cli::ParseResult::Failure(cli::ParseFailure {
                    line,
                    column,
                    message,
                });
                let file = File::create(path)?;
                let mut writer = BufWriter::new(file);
                writeln!(writer, "{}", serde_json::to_string(&result).unwrap())?;
            }
            std::process::exit(1);
        }
    }
    #[cfg(feature = "instrument")]
    {
        let stats = parser.record_stats();
        if let Some(ref path) = args.write_stats {
            let file = File::create(path)?;
            let mut writer = BufWriter::new(file);
            writeln!(writer, "{}", serde_json::to_string(&stats).unwrap())?;
        } else {
            eprintln!("{}", stats);
        }
    }
    #[cfg(not(feature = "instrument"))]
    if args.write_stats.is_some() {
        eprintln!(
            "Warning: --write-stats flag ignored. Recompile with `--features instrument` to enable stats."
        );
    }
    Ok(())
}
/// Resolves a user-supplied start nonterminal name to its id. A start
/// nonterminal A is generated as a StartA wrapper (handling layout and
/// EOF), so we try StartA first and fall back to A when A is not a start
/// nonterminal.
fn resolve_start_nonterminal(name: &str) -> io::Result<NonterminalId> {
    nonterminal_id(&format!("Start{}", name))
        .or_else(|| nonterminal_id(name))
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Unknown nonterminal: '{}'", name),
            )
        })
}
fn run_batch(
    dir: &Path,
    ext: Option<&str>,
    start_nonterminal_id: NonterminalId,
    hist_only: bool,
) -> io::Result<()> {
    let color = cli::Color::for_stdout();
    let mut files = Vec::new();
    collect_files(dir, ext, &mut files)?;
    files.sort();
    if !hist_only {
        println!(
            "{:<6}  {:<42}  {:<36}  {}",
            "STATUS", "TIME (input, init, parse, tree, drop)", "REASON", "PATH"
        );
    }
    let mut ok = 0usize;
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
    let mut corpus_stats = iguana_runtime::instrument::Stats::new();
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
        let ctx = ParseContext::new();
        let parse_tree_builder = FollowRestrictionMultipleParseTreeBuilder::new(&ctx);
        let mut parser = FollowRestrictionMultipleParser::new(&input, start_nonterminal_id);
        let init_ms = init_start.elapsed().as_secs_f64() * 1000.0;
        match parser.run() {
            ParseResult::Success(success) => {
                let parse_ms = success.duration.as_secs_f64() * 1000.0;
                let tc_start = Instant::now();
                {
                    let tree = create_parse_tree(
                        success.sppf_node_id,
                        start_nonterminal_id,
                        &parser,
                        &parse_tree_builder,
                    );
                    std::hint::black_box(tree);
                }
                let tree_ms = tc_start.elapsed().as_secs_f64() * 1000.0;
                #[cfg(feature = "instrument")]
                corpus_stats.merge(parser.record_stats());
                let drop_start = Instant::now();
                drop(parser);
                drop(parse_tree_builder);
                drop(ctx);
                drop(input);
                let drop_ms = drop_start.elapsed().as_secs_f64() * 1000.0;
                let total_ms = input_ms + init_ms + parse_ms + tree_ms + drop_ms;
                ok += 1;
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
                    println!(
                        "{}{:<6}{}  {:<42}  {:<36}  {}",
                        color.green,
                        "OK",
                        color.reset,
                        time,
                        "-",
                        rel.display()
                    );
                }
            }
            ParseResult::Failure(error) => {
                let (line, column, _) = parser.format_error(&error);
                failed += 1;
                if !hist_only {
                    let reason = format!("Parse Error at line {}, col {}", line, column);
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
    let avg_ms = if ok > 0 { total_ms / ok as f64 } else { 0.0 };
    let throughput = cli::mb_per_s(total_bytes, total_parse_ms);
    let throughput_total = cli::mb_per_s(total_bytes, total_ms);
    if !hist_only {
        println!();
        println!(
            "Parsed {} files: {} OK, {} failed, {} errors",
            files.len(),
            ok,
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
            ok, total_bytes, throughput, throughput_total
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
                let bucket_mbs = cli::mb_per_s(bucket_bytes, bucket_ms);
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
        println!("[stats] aggregated across {} successful parses", ok);
        println!("{}", corpus_stats);
    }
    Ok(())
}
fn collect_files(dir: &Path, ext: Option<&str>, out: &mut Vec<PathBuf>) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_files(&path, ext, out)?;
        } else if path.is_file() {
            if let Some(ext_filter) = ext {
                if path.extension().and_then(|e| e.to_str()) != Some(ext_filter) {
                    continue;
                }
            }
            out.push(path);
        }
    }
    Ok(())
}
#[cfg(feature = "debug-trace")]
fn write_trace_events<'i>(
    trace_events: &[TraceEvent],
    parser: &impl Parser<'i>,
    trace_option: &Option<Option<PathBuf>>,
    format: TraceFormat,
) -> io::Result<()> {
    match trace_option {
        Some(Some(path)) => {
            let file = File::create(path)?;
            let mut writer = BufWriter::new(file);
            match format {
                TraceFormat::Text => {
                    for event in trace_events {
                        writeln!(writer, "{}", event.message(parser))?;
                    }
                }
                TraceFormat::Json => {
                    writeln!(writer, "{}", serde_json::to_string(trace_events).unwrap())?;
                }
            }
        }
        Some(None) => match format {
            TraceFormat::Text => {
                for event in trace_events {
                    println!("{}", event.message(parser));
                }
            }
            TraceFormat::Json => {
                println!("{}", serde_json::to_string(trace_events).unwrap());
            }
        },
        None => {}
    }
    Ok(())
}
