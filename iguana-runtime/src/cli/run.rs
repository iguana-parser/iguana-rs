use std::{
    fs::File,
    io::{self, BufWriter, Write},
    process::ExitCode,
    time::Instant,
};

use clap::{CommandFactory, FromArgMatches};

#[cfg(feature = "debug-trace")]
use crate::trace::TraceEvent;
use crate::{
    arena::Arena,
    grammar::Grammar,
    input::Input,
    parse_tree::{DisplayOptions, to_json, to_sexpr_with},
    parser::{GLLResult, Parser},
    visualization::{dot::write_graph, gss::build_gss_dot_graph, sppf::build_sppf_graph},
};
#[cfg(feature = "profile")]
use pprof::ProfilerGuardBuilder;

use super::{
    Symbols,
    args::{Args, Format},
    batch, benchmark, corpus, golden_file, repl, start_nonterminal_id, start_nonterminal_name,
};

#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

/// Runs the command-line interface of the parser `P` over the process
/// arguments and returns the exit code. A generated crate's `main` calls this
/// with its parser type.
pub fn main<'i, 'arena, P: Parser<'i, 'arena>>() -> ExitCode {
    #[cfg(feature = "dhat-heap")]
    let _profiler = dhat::Profiler::new_heap();
    match run::<P>() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("Error: {error}");
            ExitCode::FAILURE
        }
    }
}

/// Parses the process arguments. The about line names the grammar, which the
/// derive cannot do, so the command is built here.
fn parse_args<G: Grammar>() -> Args {
    let command = Args::command().about(format!("Parser for the {} grammar", G::NAME));
    Args::from_arg_matches(&command.get_matches()).unwrap_or_else(|error| error.exit())
}

fn run<'i, 'arena, P: Parser<'i, 'arena>>() -> io::Result<()> {
    let args = parse_args::<P::Grammar>();
    #[cfg(not(feature = "instrument"))]
    if args.hist {
        eprintln!(
            "Error: --hist requires the `instrument` feature. Recompile with --features instrument."
        );
        std::process::exit(1);
    }
    if let Some(format) = args.format {
        let writes_graph = args.write_sppf.is_some() || args.write_gss.is_some();
        let traces = args.trace.is_some();
        match format {
            Format::Text if writes_graph => {
                eprintln!(
                    "Error: --format text applies to --trace only; --write-sppf and --write-gss support json or svg."
                );
                std::process::exit(1);
            }
            Format::Svg if traces => {
                eprintln!(
                    "Error: --format svg applies to --write-sppf and --write-gss only; --trace supports text or json."
                );
                std::process::exit(1);
            }
            _ if !writes_graph && !traces => {
                eprintln!(
                    "Error: --format has no effect without --trace, --write-sppf, or --write-gss."
                );
                std::process::exit(1);
            }
            _ => {}
        }
    }
    if args.list_nonterminals {
        for name in P::Grammar::DISPLAY_ORDER {
            println!("{}", name);
        }
        return Ok(());
    }
    if let Some(ref path) = args.write_symbols {
        let symbols = Symbols {
            nonterminals: P::Grammar::NONTERMINALS
                .iter()
                .map(|nt| nt.display_name.to_string())
                .collect(),
            terminals: P::Grammar::TERMINALS
                .iter()
                .map(|t| t.name.to_string())
                .collect(),
            slots: P::Grammar::SLOTS
                .iter()
                .map(|s| s.display_name.to_string())
                .collect(),
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
        return golden_file::run::<P>(&args);
    }
    if args.corpus_test.is_some() {
        return corpus::run::<P>(&args);
    }
    if args.benchmark {
        return benchmark::run::<P>(&args);
    }
    if args.dir.is_some() {
        return batch::run::<P>(&args);
    }
    if args.repl {
        return repl::run::<P>(&args);
    }
    parse_file::<P>(args)
}

/// Parses the input file and writes whatever the output flags ask for. With
/// no output flag, the parse tree goes to stdout.
fn parse_file<'i, 'arena, P: Parser<'i, 'arena>>(args: Args) -> io::Result<()> {
    let file = args.file.as_ref().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "Input file is required for parsing",
        )
    })?;
    let start_nonterminal_id = start_nonterminal_id::<P::Grammar>(start_nonterminal_name(&args)?)?;
    #[cfg(not(feature = "debug-trace"))]
    if args.trace.is_some() {
        eprintln!(
            "Warning: --trace flag ignored. Recompile with `--features debug-trace` to enable tracing."
        );
    }
    let input = Input::try_from(file.as_path())?;
    #[cfg(feature = "profile")]
    if let Some(iterations) = args.profile {
        let guard = ProfilerGuardBuilder::default()
            .frequency(999)
            .build()
            .unwrap();
        let mut tree_arena = Arena::new();
        let mut parser_arena = Arena::new();
        for _ in 0..iterations {
            let mut parser = P::ConcreteParser::<'_, '_>::new(&input, &parser_arena);
            let result = parser.run(start_nonterminal_id);
            if let GLLResult::Success(success) = result {
                let _ = parser.build_tree(success.sppf_node_id, &tree_arena);
            }
            drop(parser);
            parser_arena.reset();
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
        eprintln!(
            "Warning: --profile flag ignored. Recompile with `--features profile` to enable profiling."
        );
    }
    let tree_arena = Arena::new();
    let parser_arena = Arena::new();
    let mut parser = P::ConcreteParser::<'_, '_>::new(&input, &parser_arena);
    #[cfg(feature = "debug-trace")]
    if args.trace.is_some() {
        parser.enable_trace();
    }
    let result = parser.run(start_nonterminal_id);
    #[cfg(feature = "debug-trace")]
    if let Some(ref target) = args.trace {
        let as_json = matches!(args.format, Some(Format::Json));
        write_trace_events(parser.trace_events(), &parser, target.as_deref(), as_json)?;
    }
    match result {
        GLLResult::Success(parse_success) => {
            let node_id = parse_success.sppf_node_id;
            let as_svg = matches!(args.format, Some(Format::Svg));
            if let Some(ref path) = args.write_sppf {
                let sppf = build_sppf_graph(&parser, node_id);
                write_graph(&sppf, path, as_svg)?;
            }
            if let Some(ref path) = args.write_gss {
                let gss = build_gss_dot_graph(&parser);
                write_graph(&gss, path, as_svg)?;
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
                || (args.write_sppf.is_none() && args.write_gss.is_none() && args.trace.is_none())
            {
                Some(parser.build_tree(node_id, &tree_arena))
            } else {
                None
            };
            let tree_construction_ms = parse_tree_opt
                .as_ref()
                .map(|_| tc_start.elapsed().as_millis());
            if let (Some(path), Some(parse_tree)) =
                (args.write_parse_tree.as_ref(), parse_tree_opt.as_ref())
            {
                let json = to_json(*parse_tree, P::Grammar::LAYOUT_NAME);
                let file = File::create(path)?;
                let mut writer = BufWriter::new(file);
                writeln!(writer, "{}", json)?;
            }
            if let Some(ref path) = args.write_result {
                let result = super::ParseOutput {
                    error: None,
                    parse_ms: Some(parse_success.duration.as_millis() as u32),
                    tree_construction_ms: tree_construction_ms.map(|ms| ms as u32),
                    parse_tree: None,
                };
                let file = File::create(path)?;
                let mut writer = BufWriter::new(file);
                writeln!(writer, "{}", serde_json::to_string(&result).unwrap())?;
            }
            if !args.hist {
                eprintln!("Parse success in {}ms", parse_success.duration.as_millis());
            }
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
                    println!(
                        "{}",
                        to_sexpr_with(*parse_tree, P::Grammar::LAYOUT_NAME, display_options)
                    );
                }
            }
        }
        GLLResult::Failure(error) => {
            let error = parser.to_parse_error(&error);
            eprintln!("{}", error.render(&input));
            if let Some(ref path) = args.write_result {
                let result = super::ParseOutput {
                    error: Some(error),
                    parse_ms: None,
                    tree_construction_ms: None,
                    parse_tree: None,
                };
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

/// Writes the trace events to `target`, or to stdout when `--trace` was given
/// without a file, as one JSON array or as one message per line.
#[cfg(feature = "debug-trace")]
fn write_trace_events<'i, 'arena>(
    trace_events: &[TraceEvent],
    parser: &impl Parser<'i, 'arena>,
    target: Option<&std::path::Path>,
    as_json: bool,
) -> io::Result<()> {
    match target {
        Some(path) => {
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
        None => {
            if as_json {
                println!("{}", serde_json::to_string(trace_events).unwrap());
            } else {
                for event in trace_events {
                    println!("{}", event.message(parser));
                }
            }
        }
    }
    Ok(())
}
