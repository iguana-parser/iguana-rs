use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::{generator::utils::to_first_uppercase, grammar::grammar::Grammar};

pub fn generate(grammar: &Grammar) -> TokenStream {
    let grammar_name = format_ident!("{}", grammar.name.to_lowercase());
    let parse_tree_builder = format_ident!("{}ParseTreeBuilder", to_first_uppercase(&grammar.name));
    let parser = format_ident!("{}Parser", to_first_uppercase(&grammar.name));
    quote! {
        use std::{
            fs::File,
            io::{self, BufWriter, Write},
            path::PathBuf,
        };

        use clap::Parser as ClapParser;
        use iguana::{
            ids::NonterminalId,
            input::Input,
            parser::Parser,
            visualization::{dot::write_svg, gss::render_gss, sppf::write_sppf_dot},
        };
        use #grammar_name::{
            parse_tree::{#parse_tree_builder, create_parse_tree, to_sexpr},
            parser::#parser,
        };

        #[cfg(feature = "debug-trace")]
        use iguana::trace::TraceEvent;
        #[cfg(feature = "debug-trace")]
        use serde_json;

        #[derive(Clone, Copy, Default, clap::ValueEnum)]
        enum TraceFormat {
            #[default]
            Text,
            Json,
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

            /// Generate SPPF (Shared Packed Parse Forest) visualization
            #[arg(long)]
            sppf: bool,

            /// Generate GSS (Graph Structured Stack) visualization
            #[arg(long)]
            gss: bool,
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
                eprintln!("Warning: --trace flag ignored. Recompile with `--features debug-trace` to enable tracing.");
            }

            let input = Input::try_from(cli.file.as_path())?;
            let mut parser = #parser::new(&input);

            #[cfg(feature = "debug-trace")]
            if cli.trace.is_some() {
                parser.trace_events = Some(vec![]);
            }

            let parse_tree_builder = #parse_tree_builder;
            if let Some(node_id) = parser.run(NonterminalId(0)) {
                println!("Parse success.");

                if cli.gss {
                    let path = std::path::Path::new("gss.dot");
                    render_gss(&parser, path)?;
                    write_svg(path)?;
                    println!("GSS visualization generated: gss.svg");
                }

                if cli.sppf {
                    let path = std::path::Path::new("sppf.dot");
                    write_sppf_dot(&parser, node_id, path)?;
                    write_svg(path)?;
                    println!("SPPF visualization generated: sppf.svg");
                }

                let parse_tree = create_parse_tree(node_id, &parser, &parse_tree_builder);
                println!("{}", to_sexpr(parse_tree.as_parse_tree_ref()));
            } else {
                println!("Parse failed");
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
                Some(None) => {
                    match format {
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
                    }
                }
                None => {}
            }
            Ok(())
        }
    }
}
