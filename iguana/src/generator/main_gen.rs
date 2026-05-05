use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::{
    grammar::def::Grammar,
    utils::{to_first_uppercase, to_snake_case},
};

pub fn generate(grammar: &Grammar) -> TokenStream {
    let grammar_name = format_ident!("{}", to_snake_case(&grammar.name));
    let parse_tree_builder = format_ident!("{}ParseTreeBuilder", to_first_uppercase(&grammar.name));
    let parser = format_ident!("{}Parser", to_first_uppercase(&grammar.name));
    quote! {
        use std::{
            fs::{self, File},
            io::{self, BufWriter, IsTerminal, Write},
            path::{Path, PathBuf},
            time::Instant,
        };

        use clap::{Parser as ClapParser, ValueEnum as ClapValueEnum};
        use iguana_runtime::{
            cli,
            ids::NonterminalId,
            input::Input,
            parse_tree::ParseContext,
            parser::{ParseResult, Parser},
            visualization::{dot::write_svg, gss::{build_gss_dot_graph, render_gss}, sppf::{build_sppf_graph, write_sppf_dot}},
        };

        #[cfg(feature = "profile")]
        use pprof::ProfilerGuardBuilder;
        use #grammar_name::{
            parse_tree::{#parse_tree_builder, create_parse_tree, to_json, to_sexpr},
            grammar_data::{nonterminal_id, NONTERMINALS, NONTERMINAL_DISPLAY_ORDER, SLOTS, TERMINALS},
            parser::#parser,
        };

        #[cfg(feature = "debug-trace")]
        use iguana_runtime::trace::TraceEvent;

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
            #[arg(long = "start")]
            start_nonterminal: Option<String>,

            /// List simple nonterminals (one per line) and exit
            #[arg(long)]
            list_nonterminals: bool,

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
        }

        #[cfg(feature = "dhat-heap")]
        #[global_allocator]
        static ALLOC: dhat::Alloc = dhat::Alloc;

        fn main() -> Result<(), io::Error> {
            #[cfg(feature = "dhat-heap")]
            let _profiler = dhat::Profiler::new_heap();

            let args = Cli::parse();

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

            // Batch mode: parse every file under --dir, report per-file results and a summary.
            // Ambiguity is not yet reported here; that lands when non-panicking detection is in place.
            if let Some(dir) = args.dir.as_ref() {
                let start_nonterminal_name = args.start_nonterminal.as_ref().ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidInput, "--start is required for parsing")
                })?;
                let start_nonterminal_id = nonterminal_id(&format!("Start{}", start_nonterminal_name))
                    .or_else(|| nonterminal_id(start_nonterminal_name))
                    .ok_or_else(|| io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!("Unknown nonterminal: '{}'", start_nonterminal_name)
                    ))?;
                return run_batch(dir, args.ext.as_deref(), start_nonterminal_id);
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

            // A nonterminal A can be a start nonterminal or not. If it is, the
            // generator produces a StartA wrapper that handles layout and EOF.
            // We first try to resolve StartA; if it doesn't exist, A is not a
            // start nonterminal so we use A directly.
            let start_nonterminal_id = nonterminal_id(&format!("Start{}", start_nonterminal_name))
                .or_else(|| nonterminal_id(&start_nonterminal_name))
                .ok_or_else(|| io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("Unknown nonterminal: '{}'", start_nonterminal_name)
                ))?;

            // Profiling mode: run the parser N times under a sampling profiler
            // and write a flamegraph SVG. Short-circuits all other output.
            #[cfg(feature = "profile")]
            if let Some(iterations) = args.profile {
                let guard = ProfilerGuardBuilder::default()
                    .frequency(999)
                    .build()
                    .unwrap();

                for _ in 0..iterations {
                    let ctx = ParseContext::new();
                    let parse_tree_builder = #parse_tree_builder::new(&ctx);
                    let mut parser = #parser::new(&input, start_nonterminal_id);
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
                eprintln!("Warning: --profile flag ignored. Recompile with `--features profile` to enable profiling.");
            }

            let ctx = ParseContext::new();
            let mut parser = #parser::new(&input, start_nonterminal_id);

            #[cfg(feature = "debug-trace")]
            if args.trace.is_some() {
                parser.trace_events = Some(vec![]);
            }

            let parse_tree_builder = #parse_tree_builder::new(&ctx);
            let result = parser.run();

            // Write trace events immediately after parsing (before any visualization that might panic)
            #[cfg(feature = "debug-trace")]
            if let Some(ref trace_events) = parser.trace_events {
                write_trace_events(trace_events, &parser, &args.trace, args.format)?;
            }

            match result {
                ParseResult::Success(parse_success) => {
                    let node_id = parse_success.sppf_node_id;

                    // Handle --write-sppf (write SPPF as JSON to file)
                    if let Some(ref path) = args.write_sppf {
                        let sppf = build_sppf_graph(&parser, node_id);
                        let file = File::create(path)?;
                        let mut writer = BufWriter::new(file);
                        writeln!(writer, "{}", serde_json::to_string(&sppf).unwrap())?;
                    }

                    // Handle --write-gss (write GSS graph as JSON for visualization)
                    if let Some(ref path) = args.write_gss {
                        let gss = build_gss_dot_graph(&parser);
                        let file = File::create(path)?;
                        let mut writer = BufWriter::new(file);
                        writeln!(writer, "{}", serde_json::to_string(&gss).unwrap())?;
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
                        || (args.write_sppf.is_none() && args.write_gss.is_none() && args.vis.is_none() && args.trace.is_none())
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

                    // Handle --vis (visualization as SVG)
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

                    eprintln!("Parse success in {}ms", parse_success.duration.as_millis());

                    // Print the parse tree on stdout unless the user opted out
                    // (`--quiet`), already wrote it elsewhere (`--write-parse-tree`),
                    // or selected another output mode (`--write-sppf`, `--write-gss`,
                    // `--vis`, `--trace`).
                    if !args.quiet
                        && args.write_parse_tree.is_none()
                        && args.write_sppf.is_none()
                        && args.write_gss.is_none()
                        && args.vis.is_none()
                        && args.trace.is_none()
                    {
                        if let Some(ref parse_tree) = parse_tree_opt {
                            println!("{}", to_sexpr(*parse_tree));
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
                eprintln!("Warning: --write-stats flag ignored. Recompile with `--features instrument` to enable stats.");
            }
            Ok(())
        }

        fn run_batch(dir: &Path, ext: Option<&str>, start_nonterminal_id: NonterminalId) -> io::Result<()> {
            let use_color = io::stdout().is_terminal();
            let green = if use_color { "\x1b[32m" } else { "" };
            let red = if use_color { "\x1b[31m" } else { "" };
            let reset = if use_color { "\x1b[0m" } else { "" };

            let mut files = Vec::new();
            collect_files(dir, ext, &mut files)?;
            files.sort();

            println!("{:<6}  {:<26}  {:<36}  {}", "STATUS", "TIME (parse, tree)", "REASON", "PATH");

            let mut ok = 0usize;
            let mut failed = 0usize;
            let mut errs = 0usize;
            let mut total_parse_ms: u128 = 0;
            let mut total_tree_ms: u128 = 0;
            let mut max_total_ms: u128 = 0;

            for path in &files {
                let rel = path.strip_prefix(dir).unwrap_or(path.as_path());
                let input = match Input::try_from(path.as_path()) {
                    Ok(input) => input,
                    Err(e) => {
                        errs += 1;
                        let reason = format!("IO Error: {}", e);
                        println!("{}{:<6}{}  {:<26}  {:<36}  {}",
                            red, "ERR", reset, "-", reason, rel.display());
                        continue;
                    }
                };
                let ctx = ParseContext::new();
                let parse_tree_builder = #parse_tree_builder::new(&ctx);
                let mut parser = #parser::new(&input, start_nonterminal_id);
                match parser.run() {
                    ParseResult::Success(success) => {
                        let parse_ms = success.duration.as_millis();
                        let tc_start = Instant::now();
                        create_parse_tree(success.sppf_node_id, start_nonterminal_id, &parser, &parse_tree_builder);
                        let tree_ms = tc_start.elapsed().as_millis();
                        let total_ms = parse_ms + tree_ms;
                        ok += 1;
                        total_parse_ms += parse_ms;
                        total_tree_ms += tree_ms;
                        if total_ms > max_total_ms { max_total_ms = total_ms; }
                        let time = format!("{} ms ({} ms, {} ms)", total_ms, parse_ms, tree_ms);
                        println!("{}{:<6}{}  {:<26}  {:<36}  {}",
                            green, "OK", reset, time, "-", rel.display());
                    }
                    ParseResult::Failure(error) => {
                        let (line, column, _) = parser.format_error(&error);
                        failed += 1;
                        let reason = format!("Parse Error at line {}, col {}", line, column);
                        println!("{}{:<6}{}  {:<26}  {:<36}  {}",
                            red, "FAIL", reset, "-", reason, rel.display());
                    }
                }
            }

            let total_ms = total_parse_ms + total_tree_ms;
            let avg_ms = if ok > 0 { total_ms / ok as u128 } else { 0 };
            println!();
            println!("Parsed {} files: {} OK, {} failed, {} errors",
                files.len(), ok, failed, errs);
            println!("Total {} ms (parse {} ms, tree {} ms); avg {} ms, max {} ms",
                total_ms, total_parse_ms, total_tree_ms, avg_ms, max_total_ms);
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
                Some(None) => {
                    match format {
                        TraceFormat::Text => {
                            for event in trace_events {
                                println!("{}", event.message(parser));
                            }
                        }
                        TraceFormat::Json => {
                            println!("{}", serde_json::to_string(trace_events).unwrap());
                        }
                    }
                }
                None => {}
            }
            Ok(())
        }
    }
}
