use clap::Parser as ClapParser;
use iggy::{
    parse_tree::{IggyParseTreeBuilder, create_parse_tree, to_sexpr},
    parser::IggyParser,
};
#[cfg(feature = "debug-trace")]
use iguana::trace::TraceEvent;
use iguana::{
    ids::NonterminalId,
    input::Input,
    parser::{ParseResult, Parser},
    visualization::{
        dot::write_svg,
        gss::render_gss,
        sppf::{build_sppf_graph, write_sppf_dot},
    },
};
use serde_json;
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
#[derive(Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
enum EmitTarget {
    Sppf,
    Gss,
}
#[derive(ClapParser)]
#[command(name = "parser")]
#[command(about = "Parse a file and generate visualization")]
struct Cli {
    /// Input file to parse
    file: PathBuf,
    /// Enable trace output (writes to stdout or specified file)
    #[arg(long, value_name = "FILE")]
    trace: Option<Option<PathBuf>>,
    /// Output format for trace (text or json)
    #[arg(long, value_enum, default_value_t = TraceFormat::Text, requires = "trace")]
    format: TraceFormat,
    /// Generate visualization as SVG (sppf or gss)
    #[arg(long, value_enum)]
    vis: Option<VisTarget>,
    /// Emit data as JSON to stdout (sppf or gss)
    #[arg(long, value_enum)]
    emit: Option<EmitTarget>,
}
#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;
fn main() -> Result<(), io::Error> {
    #[cfg(feature = "dhat-heap")]
    let _profiler = dhat::Profiler::new_heap();
    let cli = Cli::parse();
    #[cfg(not(feature = "debug-trace"))]
    if cli.trace.is_some() {
        eprintln!(
            "Warning: --trace flag ignored. Recompile with `--features debug-trace` to enable tracing."
        );
    }
    let input = Input::try_from(cli.file.as_path())?;
    let mut parser = IggyParser::new(&input);
    #[cfg(feature = "debug-trace")]
    if cli.trace.is_some() {
        parser.trace_events = Some(vec![]);
    }
    let parse_tree_builder = IggyParseTreeBuilder;
    match parser.run(NonterminalId(0)) {
        ParseResult::Success(parse_success) => {
            let node_id = parse_success.sppf_node_id;
            println!("Parse success.");
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
            match cli.emit {
                Some(EmitTarget::Sppf) => {
                    let sppf = build_sppf_graph(&parser, node_id);
                    println!("{}", serde_json::to_string(&sppf).unwrap());
                }
                Some(EmitTarget::Gss) => {
                    eprintln!("GSS JSON emission not yet implemented");
                }
                None => {}
            }
            let parse_tree = create_parse_tree(node_id, &parser, &parse_tree_builder);
            println!("{}", to_sexpr(parse_tree.as_parse_tree_ref()));
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

