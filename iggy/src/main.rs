use clap::Parser as ClapParser;
use iggy::{
    parse_tree::{IggyParseTreeBuilder, create_parse_tree, to_sexpr},
    parser::IggyParser,
};
#[cfg(feature = "debug-trace")]
use iguana::trace::TraceEvent;
use iguana::{
    grammar::symbols::NonterminalNodeKind,
    input::Input,
    parser::{ParseResult, Parser},
    visualization::{
        dot::write_svg,
        gss::{build_gss_dot_graph, render_gss},
        sppf::{build_sppf_graph, write_sppf_dot},
    },
};
use std::{
    fs::File,
    io::{self, BufWriter, Write},
    path::PathBuf,
};
#[derive(Clone, Copy, Default, clap::ValueEnum)]
enum TraceFormat {
    #[default]
    Text,
    Json,
}
#[derive(Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
enum VisTarget {
    Sppf,
    Gss,
}
#[derive(ClapParser)]
#[command(name = "parser")]
#[command(about = "Parse a file and generate visualization")]
#[command(arg_required_else_help = true)]
struct Cli {
    /// Input file to parse (required unless --list-nonterminals is used)
    file: Option<PathBuf>,
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
    #[arg(long, value_enum, default_value_t = TraceFormat::Text, requires = "trace")]
    format: TraceFormat,
    /// Generate visualization as SVG (sppf or gss)
    #[arg(long, value_enum)]
    vis: Option<VisTarget>,
    /// Write SPPF as JSON to the specified file
    #[arg(long, value_name = "FILE")]
    write_sppf: Option<PathBuf>,
    /// Write GSS as JSON to the specified file
    #[arg(long, value_name = "FILE")]
    write_gss: Option<PathBuf>,
}
#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;
fn main() -> Result<(), io::Error> {
    #[cfg(feature = "dhat-heap")]
    let _profiler = dhat::Profiler::new_heap();
    let cli = Cli::parse();
    if cli.list_nonterminals {
        for nt in IggyParser::nonterminals() {
            if nt.kind == NonterminalNodeKind::Simple {
                println!("{}", nt.name);
            }
        }
        return Ok(());
    }
    if let Some(ref path) = cli.write_symbols {
        let nonterminals: Vec<&str> = IggyParser::nonterminals()
            .map(|nt| nt.name.as_str())
            .collect();
        let terminals: Vec<&str> = IggyParser::terminals().map(|t| t.name.as_str()).collect();
        let slots: Vec<&str> = IggyParser::slots().map(|s| s.name.as_str()).collect();
        let symbols = serde_json::json!(
            { "nonterminals" : nonterminals, "terminals" : terminals, "slots" : slots }
        );
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);
        writeln!(
            writer,
            "{}",
            serde_json::to_string_pretty(&symbols).unwrap()
        )?;
        return Ok(());
    }
    let file = cli.file.ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "Input file is required for parsing",
        )
    })?;
    let start_nonterminal_name = cli.start_nonterminal.ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "--start is required for parsing",
        )
    })?;
    #[cfg(not(feature = "debug-trace"))]
    if cli.trace.is_some() {
        eprintln!(
            "Warning: --trace flag ignored. Recompile with `--features debug-trace` to enable tracing."
        );
    }
    let input = Input::try_from(file.as_path())?;
    let start_nonterminal_id =
        IggyParser::nonterminal_id(&start_nonterminal_name).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Unknown nonterminal: '{}'", start_nonterminal_name),
            )
        })?;
    let mut parser = IggyParser::new(&input, start_nonterminal_id);
    #[cfg(feature = "debug-trace")]
    if cli.trace.is_some() {
        parser.trace_events = Some(vec![]);
    }
    let parse_tree_builder = IggyParseTreeBuilder;
    match parser.run() {
        ParseResult::Success(parse_success) => {
            let node_id = parse_success.sppf_node_id;
            if let Some(ref path) = cli.write_sppf {
                let sppf = build_sppf_graph(&parser, node_id);
                let file = File::create(path)?;
                let mut writer = BufWriter::new(file);
                writeln!(writer, "{}", serde_json::to_string(&sppf).unwrap())?;
            }
            if let Some(ref path) = cli.write_gss {
                let gss = build_gss_dot_graph(&parser);
                let file = File::create(path)?;
                let mut writer = BufWriter::new(file);
                writeln!(writer, "{}", serde_json::to_string(&gss).unwrap())?;
            }
            match cli.vis {
                Some(VisTarget::Gss) => {
                    let path = std::path::Path::new("gss.dot");
                    render_gss(&parser, path)?;
                    write_svg(path)?;
                    println!("GSS visualization generated: gss.svg");
                }
                Some(VisTarget::Sppf) => {
                    let path = std::path::Path::new("sppf.dot");
                    write_sppf_dot(&parser, node_id, path)?;
                    write_svg(path)?;
                    println!("SPPF visualization generated: sppf.svg");
                }
                None => {}
            }
            if cli.write_sppf.is_none() && cli.write_gss.is_none() && cli.vis.is_none() {
                println!("Parse success.");
                let parse_tree = create_parse_tree(
                    node_id,
                    &start_nonterminal_name,
                    &parser,
                    &parse_tree_builder,
                );
                println!("{}", to_sexpr(parse_tree.as_parse_tree_ref()));
            }
        }
        ParseResult::Failure() => {
            println!("Parse failed");
        }
    }
    #[cfg(feature = "debug-trace")]
    if let Some(ref trace_events) = parser.trace_events {
        write_trace_events(trace_events, &parser, &cli.trace, cli.format)?;
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
                    for event in trace_events {
                        writeln!(writer, "{}", serde_json::to_string(event).unwrap())?;
                    }
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
                for event in trace_events {
                    println!("{}", serde_json::to_string(event).unwrap());
                }
            }
        },
        None => {}
    }
    Ok(())
}

