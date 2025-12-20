use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::{generator::utils::to_first_uppercase, grammar::grammar::Grammar};

pub fn generate(grammar: &Grammar) -> TokenStream {
    let grammar_name = format_ident!("{}", grammar.name.to_lowercase());
    let parse_tree_builder = format_ident!("{}ParseTreeBuilder", to_first_uppercase(&grammar.name));
    let parser = format_ident!("{}Parser", to_first_uppercase(&grammar.name));
    quote! {
        use std::{
            env,
            fs::File,
            io::{self, BufWriter},
            path::Path,
        };

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

        #[cfg(feature = "dhat-heap")]
        #[global_allocator]
        static ALLOC: dhat::Alloc = dhat::Alloc;

        fn main() -> Result<(), io::Error> {
            #[cfg(feature = "dhat-heap")]
            let _profiler = dhat::Profiler::new_heap();

            let args: Vec<String> = env::args().collect();
            if args.len() < 2 {
                eprintln!("Usage: {} <file>", args[0]);
                std::process::exit(1);
            }
            let file_path = Path::new(&args[1]);
            let input = Input::try_from(file_path)?;
            let mut parser = #parser::new(&input);
            let parse_tree_builder = #parse_tree_builder;
            if let Some(node_id) = parser.run(NonterminalId(0)) {
                println!("Parse success.");
                let path = Path::new("gss.dot");
                let file = File::create(path)?;
                let mut gss_dot_file = BufWriter::new(file);
                render_gss(&parser, &mut gss_dot_file)?;
                write_svg(path);

                let path = Path::new("sppf.dot");
                write_sppf_dot(&parser, node_id, path)?;
                write_svg(path);
                let parse_tree = create_parse_tree(node_id, &parser, &parse_tree_builder);
                println!("{}", to_sexpr(parse_tree.as_parse_tree_ref()));
            } else {
                println!("Parse failed");
            }
            Ok(())
        }
    }
}
