use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::{
    generator::grammar_utils::{parse_tree_builder_ident, parser_ident},
    grammar::def::Grammar,
    utils::to_snake_case,
};

pub fn generate(grammar: &Grammar) -> TokenStream {
    let grammar_name = format_ident!("{}", to_snake_case(&grammar.name));
    let parse_tree_builder = parse_tree_builder_ident(&grammar.name);
    let parser = parser_ident(&grammar.name);
    let about = format!("Parser for the {} grammar", grammar.name);
    let repl_help = format!("Run the {} parser in REPL mode", grammar.name);
    quote! {
        use std::{
            fs::{self, File},
            io::{self, BufWriter, Write},
            path::{Path, PathBuf},
            time::{Duration, Instant},
        };

        use clap::{Parser as ClapParser, ValueEnum as ClapValueEnum};
        use iguana_runtime::{
            arena::Arena,
            cli,
            ids::NonterminalId,
            input::Input,
            parse_tree::{DisplayOptions, is_ambiguous},
            parser::{ParseResult, Parser},
            visualization::{dot::write_graph, gss::build_gss_dot_graph, sppf::build_sppf_graph},
        };

        #[cfg(feature = "profile")]
        use pprof::ProfilerGuardBuilder;
        use #grammar_name::{
            parse_tree::{#parse_tree_builder, create_parse_tree, to_json, to_sexpr_with},
            grammar_data::{nonterminal_id, NONTERMINALS, NONTERMINAL_DISPLAY_ORDER, SLOTS, TERMINALS},
            parser::#parser,
        };

        #[cfg(feature = "debug-trace")]
        use iguana_runtime::trace::TraceEvent;

        #[derive(Clone, Copy, ClapValueEnum)]
        enum Format {
            Text,
            Json,
            Svg,
        }

        #[derive(ClapParser)]
        #[command(about = #about)]
        #[command(arg_required_else_help = true)]
        struct Cli {
            /// Input file to parse
            ///
            /// Required unless --list-nonterminals, --dir, or --benchmark is used
            file: Option<PathBuf>,

            /// Don't print the parse tree to stdout
            ///
            /// Status messages (timing, errors) still go to stderr; redirect with 2>/dev/null to silence them too
            #[arg(short, long)]
            quiet: bool,

            /// Nonterminal to start parsing from
            ///
            /// Required unless --list-nonterminals is used
            #[arg(long = "start", value_name = "NAME", help_heading = "Parsing")]
            start_nonterminal: Option<String>,

            /// Directory to recursively parse all files in
            ///
            /// Reports per-file success or failure with timings, plus a summary at the end
            #[arg(long, value_name = "DIR", conflicts_with = "file", help_heading = "Parsing")]
            dir: Option<PathBuf>,

            /// File extension filter for --dir (e.g. "java")
            ///
            /// Without it, all files are parsed
            #[arg(long, value_name = "EXT", requires = "dir", help_heading = "Parsing")]
            ext: Option<String>,

            #[doc = #repl_help]
            ///
            /// Reads inputs from stdin and prints each parse tree. Requires --start; no input file is used
            #[arg(long, help_heading = "Parsing")]
            repl: bool,

            /// List the nonterminals declared in the grammar
            ///
            /// The valid values for --start, one per line, then exits
            #[arg(long, help_heading = "Grammar info")]
            list_nonterminals: bool,

            /// Write all nonterminals, terminals, and slots as JSON to a file
            ///
            /// Includes the derived nonterminals that --list-nonterminals hides
            #[arg(long, value_name = "FILE", help_heading = "Grammar info")]
            write_symbols: Option<PathBuf>,

            /// Show layout (whitespace, comments) nodes in the parse tree (false by default)
            #[arg(long, help_heading = "Parse-tree output")]
            show_layout: bool,

            /// Show empty optionals and repetitions (X?, X* that matched nothing) in the parse tree (false by default)
            #[arg(long, help_heading = "Parse-tree output")]
            show_empty: bool,

            /// Show wrapper nodes (start, optionals, groups, alternations) in the parse tree (false by default)
            #[arg(long, help_heading = "Parse-tree output")]
            show_wrappers: bool,

            /// Write the parse tree as JSON
            #[arg(long, value_name = "FILE", help_heading = "Output files")]
            write_parse_tree: Option<PathBuf>,

            /// Write the SPPF as JSON
            #[arg(long, value_name = "FILE", help_heading = "Output files")]
            write_sppf: Option<PathBuf>,

            /// Write the GSS graph as JSON
            ///
            /// Nodes with labels, plus edges
            #[arg(long, value_name = "FILE", help_heading = "Output files")]
            write_gss: Option<PathBuf>,

            /// Write GSS nodes as JSON
            ///
            /// Normalized with IDs so trace replay can map each GSS node to its nonterminal and input position
            #[arg(long, value_name = "FILE", help_heading = "Output files")]
            write_gss_nodes: Option<PathBuf>,

            /// Write the parse result as JSON
            ///
            /// On success includes timings; on failure includes the error location and message
            #[arg(long, value_name = "FILE", help_heading = "Output files")]
            write_result: Option<PathBuf>,

            /// Output format for --trace, --write-sppf, and --write-gss
            ///
            /// text: --trace only. json: any of the three, and the default for each. svg: --write-sppf and --write-gss only, rendered from DOT by the graphviz `dot` binary. A format that does not apply to the requested output is rejected.
            #[arg(long, value_enum, help_heading = "Output files")]
            format: Option<Format>,

            /// Enable trace output (writes to stdout, or a file if given)
            #[arg(long, value_name = "FILE", help_heading = "Tracing")]
            trace: Option<Option<PathBuf>>,

            /// Run the parser many times and report timing statistics
            ///
            /// Benchmarks a single file (the positional argument) or a --dir; otherwise the corpus listed in repos.txt. Reports min, mean, median, p90, max, stddev (in ms) for each phase: input (file read), init (allocation), parse (input characters to the SPPF), tree (SPPF to parse tree), drop (teardown); total is their sum
            #[arg(long, help_heading = "Benchmarking and profiling")]
            benchmark: bool,

            /// Number of measured iterations for --benchmark
            ///
            /// Defaults to 100 for a single file, 3 for a directory or corpus (one iteration is a full pass over every file)
            #[arg(long, value_name = "N", requires = "benchmark", help_heading = "Benchmarking and profiling")]
            iters: Option<u32>,

            /// Number of warmup iterations before measurement
            ///
            /// Defaults to 10 for a single file, 0 for a directory or corpus (a whole-corpus pass self-warms, so a cold first pass is just the median's slow outlier)
            #[arg(long, value_name = "N", requires = "benchmark", help_heading = "Benchmarking and profiling")]
            warmup: Option<u32>,

            /// Save benchmark samples to a JSON file
            ///
            /// Pairs with --baseline for A/B comparison across runs
            #[arg(long, value_name = "FILE", requires = "benchmark", help_heading = "Benchmarking and profiling")]
            save: Option<PathBuf>,

            /// Compare benchmark results against a saved baseline JSON
            ///
            /// Reports the mean delta with a 95% CI (confidence interval) on the difference; flags the run as improved/regressed/no-change
            #[arg(long, value_name = "FILE", requires = "benchmark", help_heading = "Benchmarking and profiling")]
            baseline: Option<PathBuf>,

            /// Profile the parser and write a flamegraph SVG
            ///
            /// Runs the parser N times in a loop under a sampling profiler. Requires the "profile" feature: cargo build --features profile
            #[arg(long, value_name = "N", help_heading = "Benchmarking and profiling")]
            profile: Option<u32>,

            /// Output path for the flamegraph SVG (used with --profile)
            #[arg(long, value_name = "FILE", default_value = "flamegraph.svg", help_heading = "Benchmarking and profiling")]
            profile_output: PathBuf,

            /// Compare each input's output against its sibling X.sexpr
            ///
            /// Works with a single file or --dir (which then requires --ext). Goldens hold the parse-tree s-expression on success or a "Parse error at ..." line on failure
            #[arg(long, conflicts_with = "benchmark", conflicts_with = "profile", help_heading = "Golden-file testing")]
            check_sexpr: bool,

            /// Write each input's output to its sibling X.sexpr, overwriting
            ///
            /// Same input rules as --check-sexpr
            #[arg(long, conflicts_with = "benchmark", conflicts_with = "profile", conflicts_with = "check_sexpr", help_heading = "Golden-file testing")]
            regenerate_sexpr: bool,

            /// Print golden diffs in full instead of truncating past 200 lines
            #[arg(long, requires = "check_sexpr", help_heading = "Golden-file testing")]
            full_diff: bool,

            /// Compare each corpus against its committed baseline
            ///
            /// Parses the corpora listed in <corpus-dir>/repos.txt. With a NAME, restrict to that corpus; otherwise run all. Add --update to rewrite the baselines instead of checking
            #[arg(long, value_name = "NAME", conflicts_with_all = ["benchmark", "profile", "check_sexpr", "regenerate_sexpr", "repl", "dir"], help_heading = "Corpus testing")]
            corpus_test: Option<Option<String>>,

            /// Rewrite corpus baselines instead of checking them
            ///
            /// Use with --corpus-test. Refuses to rewrite (and fails) when a file regressed (ok -> error) or parsed ambiguously
            #[arg(long, requires = "corpus_test", help_heading = "Corpus testing")]
            update: bool,

            /// Directory holding repos.txt, the per-corpus baselines, and the .cache/ checkouts
            ///
            /// Used with --corpus-test
            #[arg(long, value_name = "DIR", default_value = "corpus", help_heading = "Corpus testing")]
            corpus_dir: PathBuf,

            /// Write parser stats (counters and histograms) as JSON
            ///
            /// Requires the "instrument" feature
            #[arg(long, value_name = "FILE", help_heading = "Stats")]
            write_stats: Option<PathBuf>,

            /// Print only the parser stats histogram and exit
            ///
            /// Suppresses the printed parse tree, the "Parse success" line, and (with --dir) the per-file timing lines and the aggregate timing summary. Requires the "instrument" feature
            #[arg(long, help_heading = "Stats")]
            hist: bool,
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
                eprintln!("Error: --hist requires the `instrument` feature. Recompile with --features instrument.");
                std::process::exit(1);
            }

            // --format governs --trace (text, json) and --write-sppf / --write-gss (json, svg).
            // Reject a value that does not apply to the requested output.
            if let Some(format) = args.format {
                let writes_graph = args.write_sppf.is_some() || args.write_gss.is_some();
                let traces = args.trace.is_some();
                match format {
                    Format::Text if writes_graph => {
                        eprintln!("Error: --format text applies to --trace only; --write-sppf and --write-gss support json or svg.");
                        std::process::exit(1);
                    }
                    Format::Svg if traces => {
                        eprintln!("Error: --format svg applies to --write-sppf and --write-gss only; --trace supports text or json.");
                        std::process::exit(1);
                    }
                    _ if !writes_graph && !traces => {
                        eprintln!("Error: --format has no effect without --trace, --write-sppf, or --write-gss.");
                        std::process::exit(1);
                    }
                    _ => {}
                }
            }

            // Handle --list-nonterminals: print user-declared nonterminals in grammar source order.
            // The list is pre-computed at codegen time (filtering and sorting happen there).
            if args.list_nonterminals {
                for name in NONTERMINAL_DISPLAY_ORDER.iter() {
                    println!("{}", name);
                }
                return Ok(());
            }

            // Handle --write-symbols: write all nonterminals, terminals, and slots as JSON and exit
            if let Some(ref path) = args.write_symbols {
                let symbols = cli::Symbols {
                    nonterminals: NONTERMINALS.iter().map(|nt| nt.display.to_string()).collect(),
                    terminals: TERMINALS.iter().map(|t| t.name.to_string()).collect(),
                    slots: SLOTS.iter().map(|s| s.display_name.to_string()).collect(),
                };
                let file = File::create(path)?;
                let mut writer = BufWriter::new(file);
                writeln!(writer, "{}", serde_json::to_string_pretty(&symbols).unwrap())?;
                return Ok(());
            }

            // Golden-file testing: compare (or rewrite) each input's output against
            // its sibling X.sexpr. Takes precedence over batch mode so --dir works for
            // both. The closure produces the golden content; cli::run_golden owns the
            // pairing, diffing, summary, and exit signal.
            if args.check_sexpr || args.regenerate_sexpr {
                let mode = if args.regenerate_sexpr {
                    cli::GoldenMode::Regenerate
                } else {
                    cli::GoldenMode::Check
                };

                let start_nonterminal_name = args.start_nonterminal.as_ref().ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidInput, "--start is required for parsing")
                })?;
                let start_nonterminal_id = resolve_start_nonterminal(start_nonterminal_name)?;

                if args.write_parse_tree.is_some()
                    || args.write_sppf.is_some()
                    || args.write_gss.is_some()
                    || args.write_gss_nodes.is_some()
                    || args.write_result.is_some()
                    || args.write_stats.is_some()
                    || args.trace.is_some()
                {
                    eprintln!("Warning: output flags (--write-parse-tree, --write-sppf, --write-gss, --write-gss-nodes, --write-result, --write-stats, --trace) are ignored in golden mode.");
                }

                let mut inputs = Vec::new();
                if let Some(dir) = args.dir.as_ref() {
                    let ext = args.ext.as_deref().ok_or_else(|| io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "--ext is required with --dir for golden testing (so .sexpr goldens are not parsed as inputs)",
                    ))?;
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

                let display_options = DisplayOptions {
                    show_layout: args.show_layout,
                    show_empty: args.show_empty,
                    show_wrappers: args.show_wrappers,
                };

                let passed = cli::run_golden(mode, inputs, args.dir.as_deref(), args.quiet, args.full_diff, |path| {
                    let input = Input::try_from(path)?;
                    let tree_arena = Arena::new();
                    let parse_tree_builder = #parse_tree_builder::new(&tree_arena);
                    let vec_arena = Arena::new();
                    let mut parser = #parser::new(&input, start_nonterminal_id, &vec_arena);
                    let content = match parser.run() {
                        ParseResult::Success(success) => {
                            let tree = create_parse_tree(
                                success.sppf_node_id,
                                start_nonterminal_id,
                                &parser,
                                &parse_tree_builder,
                            );
                            to_sexpr_with(tree, display_options)
                        }
                        ParseResult::Failure(error) => {
                            let (line, column, message) = parser.format_error(&error);
                            let len = parser.error_span_len(error.input_index);
                            format!("Parse error at line {}, col {}: {}\n{}\n", line + 1, column + 1, message, input.line_and_caret(error.input_index, len))
                        }
                    };
                    Ok(content)
                })?;

                if !passed {
                    std::process::exit(1);
                }
                return Ok(());
            }

            // Corpus regression testing: parse each corpus listed in
            // <corpus-dir>/repos.txt and check it against (or rewrite) its
            // baseline. Config-driven, so it ignores --dir; each corpus's
            // checkout is found at <corpus-dir>/.cache/<name>.
            if let Some(filter) = &args.corpus_test {
                let only = filter.as_deref();
                if args.start_nonterminal.is_some() {
                    eprintln!("Note: --start is ignored with --corpus-test; the start nonterminal comes from repos.txt.");
                }
                let mode = if args.update { cli::CorpusMode::Update } else { cli::CorpusMode::Check };
                let repos_path = args.corpus_dir.join("repos.txt");
                // Scaffold the corpus dir on first use; stop after creating the
                // template so the user can fill in their repos.
                if cli::init_corpus_dir(&args.corpus_dir)? {
                    println!("Created {}. List your repos there and re-run.", repos_path.display());
                    return Ok(());
                }
                let entries = cli::read_repos(&repos_path)?;
                if entries.is_empty() {
                    eprintln!("Warning: no repos listed in {}", repos_path.display());
                }

                let mut ran = 0usize;
                let mut passed = 0usize;
                // Reused across all files; the reset after each parse frees the
                // arena in bulk (same pattern as bench_parse_file).
                let mut vec_arena = Arena::new();
                for entry in &entries {
                    if let Some(name) = only {
                        if entry.name != name {
                            continue;
                        }
                    }
                    ran += 1;

                    let start_nonterminal_id = resolve_start_nonterminal(&entry.start)?;
                    let checkout = args.corpus_dir.join(".cache").join(&entry.name);
                    if let Err(e) = cli::fetch_corpus(&checkout, &entry.repo, &entry.git_ref) {
                        eprintln!("Corpus '{}': {}", entry.name, e);
                        continue;
                    }

                    let mut inputs = Vec::new();
                    collect_files(&checkout, Some(&entry.ext), &mut inputs)?;
                    inputs.sort();

                    let baseline_path = args.corpus_dir.join(format!("{}.txt", entry.name));
                    let report = cli::run_corpus(
                        &entry.name,
                        inputs,
                        &checkout,
                        &baseline_path,
                        cli::CorpusConfig {
                            mode,
                            quiet: args.quiet,
                        },
                        |path| {
                            let input = match Input::try_from(path) {
                                Ok(input) => input,
                                Err(e) => return cli::CorpusOutcome::IoError { message: e.to_string() },
                            };
                            let mut parser = #parser::new(&input, start_nonterminal_id, &vec_arena);
                            let outcome = match parser.run() {
                                ParseResult::Success(success) => cli::CorpusOutcome::Ok {
                                    ambiguous: is_ambiguous(&parser, success.sppf_node_id),
                                },
                                ParseResult::Failure(error) => {
                                    let (line, column, message) = parser.format_error(&error);
                                    cli::CorpusOutcome::Error {
                                        message: format!(
                                            "Parse error at line {}, col {}: {}",
                                            line + 1, column + 1, message
                                        ),
                                    }
                                }
                            };
                            drop(parser);
                            vec_arena.reset();
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
                        cli::CorpusMode::Check => println!(
                            "{} {} checked: {} passed, {} failed", ran, noun, passed, ran - passed
                        ),
                        cli::CorpusMode::Update => println!(
                            "{} {} updated: {} written, {} refused", ran, noun, passed, ran - passed
                        ),
                    }
                }

                if passed != ran {
                    std::process::exit(1);
                }
                return Ok(());
            }

            // Benchmarking: run repeated parses and report timing statistics. The
            // source is implicit: a positional file benchmarks that file (default
            // 100 iterations), a --dir benchmarks every file under it, otherwise the
            // corpus in repos.txt. For a directory or corpus one iteration is a full
            // pass over every file (default 3 iterations), so each sample is one
            // whole-corpus run.
            if args.benchmark {
                if let Some(file) = args.file.as_ref() {
                    let start_nonterminal_name = args.start_nonterminal.as_ref().ok_or_else(|| {
                        io::Error::new(io::ErrorKind::InvalidInput, "--start is required for parsing")
                    })?;
                    let start_nonterminal_id = resolve_start_nonterminal(start_nonterminal_name)?;
                    let config = cli::BenchConfig {
                        iters: args.iters.unwrap_or(100) as usize,
                        warmup: args.warmup.unwrap_or(10) as usize,
                        save: args.save.clone(),
                        baseline: args.baseline.clone(),
                    };
                    eprintln!(
                        "Benchmarking {} over {} {}...",
                        file.display(),
                        config.iters,
                        if config.iters == 1 { "iteration" } else { "iterations" },
                    );
                    let file_path = file.clone();
                    let mut tree_arena = Arena::new();
                    let mut vec_arena = Arena::new();
                    return cli::run_benchmark(config, move || {
                        bench_parse_file(&file_path, start_nonterminal_id, &mut tree_arena, &mut vec_arena)
                            .expect("benchmark input could not be read, failed to parse, or is ambiguous")
                    });
                }

                // Directory or corpus: collect each source's files with its start
                // nonterminal, kept in per-source groups so every run can report
                // which source it is on. The progress prints sit between
                // bench_parse_file calls, so their time lands in no phase and the
                // reported timings stay exact; the closure sums each run into one
                // sample.
                let config = cli::BenchConfig {
                    iters: args.iters.unwrap_or(3) as usize,
                    warmup: args.warmup.unwrap_or(0) as usize,
                    save: args.save.clone(),
                    baseline: args.baseline.clone(),
                };
                let mut groups: Vec<(String, Vec<(PathBuf, NonterminalId)>)> = Vec::new();
                if let Some(dir) = args.dir.as_ref() {
                    let start_nonterminal_name = args.start_nonterminal.as_ref().ok_or_else(|| {
                        io::Error::new(io::ErrorKind::InvalidInput, "--start is required for parsing")
                    })?;
                    let start_nonterminal_id = resolve_start_nonterminal(start_nonterminal_name)?;
                    let mut files = Vec::new();
                    collect_files(dir, args.ext.as_deref(), &mut files)?;
                    files.sort();
                    let paired = files.into_iter().map(|p| (p, start_nonterminal_id)).collect();
                    groups.push((dir.display().to_string(), paired));
                } else {
                    let repos_path = args.corpus_dir.join("repos.txt");
                    // Scaffold the corpus dir on first use, same as --corpus-test.
                    if cli::init_corpus_dir(&args.corpus_dir)? {
                        println!("Created {}. List your repos there and re-run.", repos_path.display());
                        return Ok(());
                    }
                    for entry in cli::read_repos(&repos_path)? {
                        let start_nonterminal_id = resolve_start_nonterminal(&entry.start)?;
                        let checkout = args.corpus_dir.join(".cache").join(&entry.name);
                        cli::fetch_corpus(&checkout, &entry.repo, &entry.git_ref)?;
                        let mut files = Vec::new();
                        collect_files(&checkout, Some(&entry.ext), &mut files)?;
                        files.sort();
                        let paired = files.into_iter().map(|p| (p, start_nonterminal_id)).collect();
                        groups.push((entry.name.clone(), paired));
                    }
                }

                let total_files: usize = groups.iter().map(|(_, files)| files.len()).sum();
                if total_files == 0 {
                    return Err(io::Error::new(io::ErrorKind::InvalidInput, "no files to benchmark"));
                }

                // Header, then a per-source file breakdown for a bare corpus (with the
                // counts aligned in a column). A single --dir names the directory instead.
                let total = cli::group_digits(&total_files.to_string());
                let iterations_word = if config.iters == 1 { "iteration" } else { "iterations" };
                if args.dir.is_none() {
                    eprintln!("Running the corpus ({} files), {} {}:", total, config.iters, iterations_word);
                    let width = groups.iter().map(|(label, _)| label.len() + 1).max().unwrap_or(0);
                    for (label, files) in &groups {
                        eprintln!(
                            "  {:<width$} {} files",
                            format!("{}:", label),
                            cli::group_digits(&files.len().to_string()),
                            width = width,
                        );
                    }
                } else {
                    eprintln!("Running {} ({} files), {} {}...", groups[0].0, total, config.iters, iterations_word);
                }
                eprintln!();

                let iters = config.iters;
                let warmup = config.warmup;
                // Longest source label, used to align the per-source time column.
                let max_label = groups.iter().map(|(label, _)| label.len()).max().unwrap_or(0);
                let mut pass = 0usize;
                let mut tree_arena = Arena::new();
                let mut vec_arena = Arena::new();
                return cli::run_benchmark(config, move || {
                    // A blank line separates consecutive runs. Then announce the run (or
                    // warmup pass), each source with its time, and the whole run's time.
                    // Every printed duration is a sum of bench_parse_file timings, so the
                    // prints add nothing to any phase.
                    if pass > 0 {
                        eprintln!();
                    }
                    let is_warmup = pass < warmup;
                    let run_num = if is_warmup { pass + 1 } else { pass - warmup + 1 };
                    let kind = if is_warmup { "warmup" } else { "run" };
                    eprintln!("{} {}/{}", kind, run_num, if is_warmup { warmup } else { iters });
                    pass += 1;

                    let mut input = Duration::ZERO;
                    let mut init = Duration::ZERO;
                    let mut parse = Duration::ZERO;
                    let mut tree = Duration::ZERO;
                    let mut drop = Duration::ZERO;
                    let mut bytes = 0u64;
                    // Files that fail to parse or parse ambiguously are skipped, so
                    // each sample times only the clean parses.
                    for (label, files) in &groups {
                        // Print the label first so the cursor waits right after it; the
                        // time is padded to a shared column once the source is done.
                        eprint!("  {}...", label);
                        let _ = io::stderr().flush();
                        let mut source_time = Duration::ZERO;
                        for (path, start_nonterminal_id) in files {
                            if let Some(t) = bench_parse_file(path, *start_nonterminal_id, &mut tree_arena, &mut vec_arena) {
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
                        eprintln!("{}{} ms", pad, cli::group_digits(&format!("{:.0}", source_time.as_secs_f64() * 1000.0)));
                    }
                    let run_ms = (input + init + parse + tree + drop).as_secs_f64() * 1000.0;
                    eprintln!("{} {} completed in {} ms", kind, run_num, cli::group_digits(&format!("{:.0}", run_ms)));
                    cli::PhaseTimings { input, init, parse, tree, drop, bytes }
                });
            }

            // Batch mode: parse every file under --dir, report per-file results and a summary.
            if let Some(dir) = args.dir.as_ref() {
                let start_nonterminal_name = args.start_nonterminal.as_ref().ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidInput, "--start is required for parsing")
                })?;
                let start_nonterminal_id = resolve_start_nonterminal(start_nonterminal_name)?;
                let passed = run_batch(dir, args.ext.as_deref(), start_nonterminal_id, args.hist)?;
                if !passed {
                    std::process::exit(1);
                }
                return Ok(());
            }

            // Interactive REPL: parse inputs read from stdin. Needs --start but no
            // input file, so resolve the start nonterminal and loop here.
            if args.repl {
                let start_nonterminal_name = args.start_nonterminal.as_ref().ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidInput, "--start is required for parsing")
                })?;
                let start_nonterminal_id = resolve_start_nonterminal(start_nonterminal_name)?;
                let display_options = DisplayOptions {
                    show_layout: args.show_layout,
                    show_empty: args.show_empty,
                    show_wrappers: args.show_wrappers,
                };
                cli::run_repl(display_options, |text, display_options| {
                    let input = Input::from(text);
                    let tree_arena = Arena::new();
                    let parse_tree_builder = #parse_tree_builder::new(&tree_arena);
                    let vec_arena = Arena::new();
                    let mut parser = #parser::new(&input, start_nonterminal_id, &vec_arena);
                    match parser.run() {
                        ParseResult::Success(success) => {
                            let node_id = success.sppf_node_id;
                            let ambiguous = is_ambiguous(&parser, node_id);
                            let tree = create_parse_tree(node_id, start_nonterminal_id, &parser, &parse_tree_builder);
                            cli::ReplOutcome::Parsed { tree: to_sexpr_with(tree, display_options), ambiguous }
                        }
                        ParseResult::Failure(error) => {
                            let (line, column, message) = parser.format_error(&error);
                            let len = parser.error_span_len(error.input_index);
                            cli::ReplOutcome::Failed {
                                message: format!("Parse error at line {}, col {}: {}\n{}", line + 1, column + 1, message, input.line_and_caret(error.input_index, len)),
                            }
                        }
                    }
                });
                return Ok(());
            }

            // For parsing, file and start_nonterminal are required
            let file = args.file.ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "Input file is required for parsing")
            })?;
            let start_nonterminal_name = args.start_nonterminal.ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "--start is required for parsing")
            })?;

            #[cfg(not(feature = "debug-trace"))]
            if args.trace.is_some() {
                eprintln!("Warning: --trace flag ignored. Recompile with `--features debug-trace` to enable tracing.");
            }

            let input = Input::try_from(file.as_path())?;

            let start_nonterminal_id = resolve_start_nonterminal(&start_nonterminal_name)?;

            // Profiling mode: run the parser N times under a sampling profiler
            // and write a flamegraph SVG. Short-circuits all other output.
            #[cfg(feature = "profile")]
            if let Some(iterations) = args.profile {
                let guard = ProfilerGuardBuilder::default()
                    .frequency(999)
                    .build()
                    .unwrap();

                // The arenas are reset per iteration instead of dropped, the
                // same lifecycle as --benchmark, so the profile measures the
                // warm-arena work the benchmark times.
                let mut tree_arena = Arena::new();
                let mut vec_arena = Arena::new();
                for _ in 0..iterations {
                    let mut parser = #parser::new(&input, start_nonterminal_id, &vec_arena);
                    let result = parser.run();
                    if let ParseResult::Success(success) = result {
                        let parse_tree_builder = #parse_tree_builder::new(&tree_arena);
                        let _ = create_parse_tree(
                            success.sppf_node_id,
                            start_nonterminal_id,
                            &parser,
                            &parse_tree_builder,
                        );
                    }
                    drop(parser);
                    vec_arena.reset();
                    tree_arena.reset();
                }

                let report = guard.report().build().unwrap();
                let file = File::create(&args.profile_output)?;
                report.flamegraph(&file).unwrap();
                eprintln!("Flamegraph written to {}", args.profile_output.display());
                return Ok(());
            }

            #[cfg(not(feature = "profile"))]
            if args.profile.is_some() {
                eprintln!("Warning: --profile flag ignored. Recompile with `--features profile` to enable profiling.");
            }

            let tree_arena = Arena::new();
            let vec_arena = Arena::new();
            let mut parser = #parser::new(&input, start_nonterminal_id, &vec_arena);

            #[cfg(feature = "debug-trace")]
            if args.trace.is_some() {
                parser.trace_events = Some(vec![]);
            }

            let parse_tree_builder = #parse_tree_builder::new(&tree_arena);
            let result = parser.run();

            // Write trace events immediately after parsing (before any visualization that might panic)
            #[cfg(feature = "debug-trace")]
            if let Some(ref trace_events) = parser.trace_events {
                let as_json = matches!(args.format, Some(Format::Json));
                write_trace_events(trace_events, &parser, &args.trace, as_json)?;
            }

            match result {
                ParseResult::Success(parse_success) => {
                    let node_id = parse_success.sppf_node_id;
                    // SPPF/GSS write as JSON by default, or SVG with --format svg.
                    let as_svg = matches!(args.format, Some(Format::Svg));

                    // Handle --write-sppf (write SPPF as JSON or SVG)
                    if let Some(ref path) = args.write_sppf {
                        let sppf = build_sppf_graph(&parser, node_id);
                        write_graph(&sppf, path, as_svg)?;
                    }

                    // Handle --write-gss (write GSS graph as JSON or SVG)
                    if let Some(ref path) = args.write_gss {
                        let gss = build_gss_dot_graph(&parser);
                        write_graph(&gss, path, as_svg)?;
                    }

                    // Handle --write-gss-nodes (write GSS nodes as JSON for trace replay)
                    if let Some(ref path) = args.write_gss_nodes {
                        let gss_nodes: Vec<_> = parser.gss_nodes().collect();
                        let file = File::create(path)?;
                        let mut writer = BufWriter::new(file);
                        writeln!(writer, "{}", serde_json::to_string(&gss_nodes).unwrap())?;
                    }

                    // Time tree construction once (separately from --write-parse-tree)
                    // so we can report it via --write-result even if no parse-tree file is requested.
                    let tc_start = Instant::now();
                    let parse_tree_opt = if args.write_parse_tree.is_some()
                        || args.write_result.is_some()
                        || (args.write_sppf.is_none() && args.write_gss.is_none() && args.trace.is_none())
                    {
                        Some(create_parse_tree(node_id, start_nonterminal_id, &parser, &parse_tree_builder))
                    } else {
                        None
                    };
                    let tree_construction_ms = parse_tree_opt.as_ref().map(|_| tc_start.elapsed().as_millis());

                    // Handle --write-parse-tree (write parse tree as JSON for visualization)
                    if let (Some(path), Some(parse_tree)) = (args.write_parse_tree.as_ref(), parse_tree_opt.as_ref()) {
                        let json = to_json(*parse_tree);
                        let file = File::create(path)?;
                        let mut writer = BufWriter::new(file);
                        writeln!(writer, "{}", json)?;
                    }

                    // Handle --write-result (write parse result as JSON)
                    if let Some(ref path) = args.write_result {
                        let result = cli::ParseResult::Success(cli::ParseSuccess {
                            parse_ms: parse_success.duration.as_millis() as u64,
                            tree_construction_ms: tree_construction_ms.map(|ms| ms as u64),
                        });
                        let file = File::create(path)?;
                        let mut writer = BufWriter::new(file);
                        writeln!(writer, "{}", serde_json::to_string(&result).unwrap())?;
                    }

                    if !args.hist {
                        eprintln!("Parse success in {}ms", parse_success.duration.as_millis());
                    }

                    // Print the parse tree on stdout unless the user opted out
                    // (`--quiet` / `--hist`), already wrote it elsewhere (`--write-parse-tree`),
                    // or selected another output mode (`--write-sppf`, `--write-gss`,
                    // `--trace`).
                    if !args.quiet
                        && !args.hist
                        && args.write_parse_tree.is_none()
                        && args.write_sppf.is_none()
                        && args.write_gss.is_none()
                        && args.trace.is_none()
                    {
                        if let Some(ref parse_tree) = parse_tree_opt {
                            let display_options = DisplayOptions {
                                show_layout: args.show_layout,
                                show_empty: args.show_empty,
                                show_wrappers: args.show_wrappers,
                            };
                            println!("{}", to_sexpr_with(*parse_tree, display_options));
                        }
                    }
                }
                ParseResult::Failure(error) => {
                    let (line, column, message) = parser.format_error(&error);
                    let len = parser.error_span_len(error.input_index);
                    eprintln!("Parse error at line {}, col {}: {}\n{}", line + 1, column + 1, message, input.line_and_caret(error.input_index, len));

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
                eprintln!("Warning: --write-stats flag ignored. Recompile with `--features instrument` to enable stats.");
            }
            Ok(())
        }

        /// Resolves a user-supplied start nonterminal name to the id of its
        /// generated StartA wrapper. Every source nonterminal gets a wrapper as
        /// its entry point, so `-n A` resolves to StartA. A name with no wrapper
        /// (a typo, or a nonterminal introduced by desugaring) is not an entry
        /// point and is an error.
        fn resolve_start_nonterminal(name: &str) -> io::Result<NonterminalId> {
            nonterminal_id(&format!("Start{}", name))
                .ok_or_else(|| io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("Unknown nonterminal: '{}'", name),
                ))
        }

        fn run_batch(dir: &Path, ext: Option<&str>, start_nonterminal_id: NonterminalId, hist_only: bool) -> io::Result<bool> {
            let color = cli::Color::for_stdout();

            let mut files = Vec::new();
            collect_files(dir, ext, &mut files)?;
            files.sort();

            if !hist_only {
                println!("{:<6}  {:<42}  {:<36}  PATH",
                    "STATUS", "TIME (input, init, parse, tree, drop)", "REASON");
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
                            println!("{}{:<6}{}  {:<42}  {:<36}  {}",
                                color.red, "ERR", color.reset, "-", reason, rel.display());
                        }
                        continue;
                    }
                };
                let input_ms = input_start.elapsed().as_secs_f64() * 1000.0;
                let bytes = input.len() as u64;

                let init_start = Instant::now();
                let tree_arena = Arena::new();
                let parse_tree_builder = #parse_tree_builder::new(&tree_arena);
                let vec_arena = Arena::new();
                let mut parser = #parser::new(&input, start_nonterminal_id, &vec_arena);
                let init_ms = init_start.elapsed().as_secs_f64() * 1000.0;

                match parser.run() {
                    ParseResult::Success(success) => {
                        let parse_ms = success.duration.as_secs_f64() * 1000.0;
                        let ambig = is_ambiguous(&parser, success.sppf_node_id);
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
                        // The builder borrows the tree arena and owns no heap data,
                        // so release its borrow before timing teardown of the
                        // structures that actually free memory.
                        let _ = parse_tree_builder;
                        let drop_start = Instant::now();
                        drop(parser);
                        drop(vec_arena);
                        drop(tree_arena);
                        drop(input);
                        let drop_ms = drop_start.elapsed().as_secs_f64() * 1000.0;
                        let total_ms = input_ms + init_ms + parse_ms + tree_ms + drop_ms;
                        // An ambiguous parse still parsed, so its time and bytes feed
                        // the throughput totals; only the status bucket and the exit
                        // gate treat it apart from a clean OK.
                        if ambig { ambiguous += 1; } else { ok += 1; }
                        total_input_ms += input_ms;
                        total_init_ms += init_ms;
                        total_parse_ms += parse_ms;
                        total_tree_ms += tree_ms;
                        total_drop_ms += drop_ms;
                        total_bytes += bytes;
                        per_file.push((bytes, parse_ms));
                        if total_ms > max_total_ms { max_total_ms = total_ms; }
                        if !hist_only {
                            let time = format!("{} ms ({} ms, {} ms, {} ms, {} ms, {} ms)",
                                total_ms as u128, input_ms as u128, init_ms as u128,
                                parse_ms as u128, tree_ms as u128, drop_ms as u128);
                            let (label, code) = if ambig {
                                ("AMB", color.red)
                            } else {
                                ("OK", color.green)
                            };
                            println!("{}{:<6}{}  {:<42}  {:<36}  {}",
                                code, label, color.reset, time, "-", rel.display());
                        }
                    }
                    ParseResult::Failure(error) => {
                        let (line, column, _) = parser.format_error(&error);
                        failed += 1;
                        if !hist_only {
                            let reason = format!("Parse error at line {}, col {}", line + 1, column + 1);
                            println!("{}{:<6}{}  {:<42}  {:<36}  {}",
                                color.red, "FAIL", color.reset, "-", reason, rel.display());
                        }
                    }
                }
            }

            let total_ms = total_input_ms + total_init_ms + total_parse_ms
                + total_tree_ms + total_drop_ms;
            let parsed = ok + ambiguous;
            let avg_ms = if parsed > 0 { total_ms / parsed as f64 } else { 0.0 };
            let throughput = cli::mb_per_s(total_bytes, total_parse_ms);
            let throughput_total = cli::mb_per_s(total_bytes, total_ms);
            if !hist_only {
                println!();
                println!("Parsed {} files: {} OK, {} ambiguous, {} failed, {} errors",
                    files.len(), ok, ambiguous, failed, errs);
                println!("Total {:.0} ms (input {:.0}, init {:.0}, parse {:.0}, tree {:.0}, drop {:.0}); avg {:.1} ms, max {:.0} ms",
                    total_ms, total_input_ms, total_init_ms, total_parse_ms, total_tree_ms,
                    total_drop_ms, avg_ms, max_total_ms);
                println!("Throughput on {} successful parses ({} bytes): {:.2} MB/s parse only, {:.2} MB/s total",
                    parsed, total_bytes, throughput, throughput_total);

                if !per_file.is_empty() {
                    let buckets: &[(&str, u64, u64)] = &[
                        ("< 1 KB",     0,                 1024),
                        ("1-4 KB",     1024,              4 * 1024),
                        ("4-16 KB",    4 * 1024,          16 * 1024),
                        ("16-64 KB",   16 * 1024,         64 * 1024),
                        ("64-256 KB",  64 * 1024,         256 * 1024),
                        ("> 256 KB",   256 * 1024,        u64::MAX),
                    ];
                    println!();
                    println!("[throughput by file size]");
                    println!("  {:<12} {:>6} {:>12} {:>12} {:>8} {:>12} {:>10}",
                        "size class", "files", "bytes", "parse ms", "MB/s", "median ms", "p90 ms");
                    for (label, lo, hi) in buckets {
                        let mut bucket: Vec<&(u64, f64)> = per_file.iter()
                            .filter(|(b, _)| *b >= *lo && *b < *hi)
                            .collect();
                        if bucket.is_empty() { continue; }
                        let count = bucket.len();
                        let bucket_bytes: u64 = bucket.iter().map(|(b, _)| *b).sum();
                        let bucket_ms: f64 = bucket.iter().map(|(_, ms)| *ms).sum();
                        let bucket_mbs = cli::mb_per_s(bucket_bytes, bucket_ms);
                        bucket.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
                        let median_ms = bucket[count / 2].1;
                        let p90_idx = (((count as f64) * 0.9) as usize).min(count - 1);
                        let p90_ms = bucket[p90_idx].1;
                        println!("  {:<12} {:>6} {:>12} {:>12.1} {:>8.2} {:>12.3} {:>10.3}",
                            label, count, bucket_bytes, bucket_ms, bucket_mbs, median_ms, p90_ms);
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

        /// Parses one file under a benchmark, returning its per-phase timings, or
        /// `None` when the file should not count toward the measurement: it cannot
        /// be read, it fails to parse, or it parses ambiguously. A benchmark times
        /// clean single-tree parses, so a whole-corpus run skips the rest rather
        /// than mixing their work in. The input is reloaded so the `input` phase is
        /// measured. The caller's arenas are reused across files and reset
        /// between them, so they keep their chunks and teardown is a bulk
        /// reset rather than a per-file free, the pattern the arena is built for.
        fn bench_parse_file(path: &Path, start_nonterminal_id: NonterminalId, tree_arena: &mut Arena, vec_arena: &mut Arena) -> Option<cli::PhaseTimings> {
            let input_start = Instant::now();
            let input = Input::try_from(path).ok()?;
            let input_time = input_start.elapsed();
            let bytes = input.len() as u64;

            let init_start = Instant::now();
            let mut parser = #parser::new(&input, start_nonterminal_id, vec_arena);
            let init = init_start.elapsed();

            // A skipped file still resets the parser's arena, so its allocations
            // do not carry into the next file's measurement.
            let ParseResult::Success(success) = parser.run() else {
                drop(parser);
                vec_arena.reset();
                return None;
            };
            let parse = success.duration;
            if is_ambiguous(&parser, success.sppf_node_id) {
                drop(parser);
                vec_arena.reset();
                return None;
            }

            let tree_start = Instant::now();
            {
                let parse_tree_builder = #parse_tree_builder::new(tree_arena);
                let tree = create_parse_tree(
                    success.sppf_node_id,
                    start_nonterminal_id,
                    &parser,
                    &parse_tree_builder,
                );
                std::hint::black_box(tree);
            }
            let tree = tree_start.elapsed();

            // Dropping the parser runs each collection's Drop, but every spilled
            // buffer lives in an arena, so those are no-op deallocs; the resets
            // then free both arenas in bulk and keep their chunks for the next file.
            let drop_start = Instant::now();
            drop(parser);
            vec_arena.reset();
            tree_arena.reset();
            drop(input);
            let drop = drop_start.elapsed();

            Some(cli::PhaseTimings { input: input_time, init, parse, tree, drop, bytes })
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
        fn write_trace_events<'i, 'arena>(
            trace_events: &[TraceEvent],
            parser: &impl Parser<'i, 'arena>,
            trace_option: &Option<Option<PathBuf>>,
            as_json: bool,
        ) -> io::Result<()> {
            match trace_option {
                Some(Some(path)) => {
                    let file = File::create(path)?;
                    let mut writer = BufWriter::new(file);
                    if as_json {
                        writeln!(writer, "{}", serde_json::to_string(trace_events).unwrap())?;
                    } else {
                        for event in trace_events {
                            writeln!(writer, "{}", event.message(parser))?;
                        }
                    }
                }
                Some(None) => {
                    if as_json {
                        println!("{}", serde_json::to_string(trace_events).unwrap());
                    } else {
                        for event in trace_events {
                            println!("{}", event.message(parser));
                        }
                    }
                }
                None => {}
            }
            Ok(())
        }
    }
}
