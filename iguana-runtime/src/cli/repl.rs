use std::io;

use crate::{
    arena::Arena,
    grammar::Grammar,
    input::Input,
    parse_tree::{DisplayOptions, is_ambiguous, to_sexpr_with},
    parser::{GLLResult, Parser},
};

use super::{args::Args, start_nonterminal_id, start_nonterminal_name};

/// Runs `--repl`: reads inputs from stdin and prints the parse tree of each.
pub(super) fn run<'i, 'arena, P: Parser<'i, 'arena>>(args: &Args) -> io::Result<()> {
    let start_nonterminal_id = start_nonterminal_id::<P::Grammar>(start_nonterminal_name(args)?)?;
    let display_options = DisplayOptions {
        show_layout: args.show_layout,
        show_empty: args.show_empty,
        show_wrappers: args.show_wrappers,
    };
    run_repl(display_options, |text, display_options| {
        let input = Input::from(text);
        let tree_arena = Arena::new();
        let parser_arena = Arena::new();
        let mut parser = P::ConcreteParser::<'_, '_>::new(&input, &parser_arena);
        match parser.run(start_nonterminal_id) {
            GLLResult::Success(success) => {
                let node_id = success.sppf_node_id;
                let ambiguous = is_ambiguous(&parser, node_id);
                let tree = parser.build_tree(node_id, &tree_arena);
                ReplOutcome::Parsed {
                    tree: to_sexpr_with(tree, P::Grammar::LAYOUT_NAME, display_options),
                    ambiguous,
                }
            }
            GLLResult::Failure(error) => ReplOutcome::Failed {
                message: parser.to_parse_error(&error).render(&input),
            },
        }
    });
    Ok(())
}

enum ReplOutcome {
    Parsed { tree: String, ambiguous: bool },
    Failed { message: String },
}

/// Reads inputs from stdin and prints the parse tree for each. One input is a
/// block of lines terminated by a blank line; `Ctrl-D` exits. A line starting
/// with `:` at a fresh prompt is a meta-command (`:set`, `:help`), not parse
/// input. Prompts and diagnostics go to stderr and parse trees to stdout, so
/// the output stays usable when stdout is redirected to a file.
///
/// `options` is the initial rendering state, taken from the parser's flags;
/// `:set` mutates it during the session and each parse renders with the
/// current value.
fn run_repl<F>(mut options: DisplayOptions, mut parse_fn: F)
where
    F: FnMut(&str, DisplayOptions) -> ReplOutcome,
{
    use io::{BufRead, Write};

    eprintln!("Enter input, blank line to parse. :help for commands, Ctrl-D to exit.");
    let stdin = io::stdin();
    let mut handle = stdin.lock();
    loop {
        eprint!("> ");
        io::stderr().flush().ok();

        // Accumulate lines until a blank line submits the block.
        let mut block = String::new();
        let mut command = false;
        loop {
            let mut line = String::new();
            if handle.read_line(&mut line).unwrap_or(0) == 0 {
                // EOF: parse a pending block, then exit on the next prompt.
                if block.is_empty() {
                    eprintln!();
                    return;
                }
                break;
            }
            // A `:` line at a fresh prompt is a command; mid-block it is input.
            if block.is_empty() && line.trim_start().starts_with(':') {
                run_repl_command(line.trim(), &mut options);
                command = true;
                break;
            }
            if line.trim_end_matches(['\n', '\r']).is_empty() {
                break;
            }
            block.push_str(&line);
            eprint!(". ");
            io::stderr().flush().ok();
        }
        if command {
            continue;
        }

        let input = block.trim_end();
        if input.is_empty() {
            continue;
        }
        match parse_fn(input, options) {
            ReplOutcome::Parsed { tree, ambiguous } => {
                if ambiguous {
                    eprintln!("// ambiguous");
                }
                print!("{}", tree);
                io::stdout().flush().ok();
            }
            ReplOutcome::Failed { message } => eprintln!("{}", message),
        }
    }
}

/// Runs a REPL meta-command (the line still carries its leading `:`). Feedback
/// goes to stderr, like the prompt.
fn run_repl_command(line: &str, options: &mut DisplayOptions) {
    let mut parts = line.split_whitespace();
    match parts.next() {
        Some(":help") | Some(":h") | Some(":?") => {
            eprintln!("commands:");
            eprintln!("  :set                       list settings");
            eprintln!("  :set <name> [true|false]   toggle, or set, a setting");
            eprintln!("  settings: show-layout, show-empty, show-wrappers");
            eprintln!("  :help                      show this help");
            eprintln!("  Ctrl-D                     exit");
        }
        Some(":set") => {
            let name = parts.next();
            let value = parts.next();
            match name {
                None => {
                    eprintln!("show-layout   = {}", options.show_layout);
                    eprintln!("show-empty    = {}", options.show_empty);
                    eprintln!("show-wrappers = {}", options.show_wrappers);
                }
                Some("show-layout") => {
                    set_repl_bool("show-layout", value, &mut options.show_layout)
                }
                Some("show-empty") => set_repl_bool("show-empty", value, &mut options.show_empty),
                Some("show-wrappers") => {
                    set_repl_bool("show-wrappers", value, &mut options.show_wrappers)
                }
                Some(other) => eprintln!("unknown setting: {} (try :help)", other),
            }
        }
        Some(other) => eprintln!("unknown command: {} (try :help)", other),
        None => {}
    }
}

/// Applies a `:set <name> [value]` to a boolean setting, echoing the result.
/// A missing value toggles, so `:set show-layout` flips it.
fn set_repl_bool(name: &str, value: Option<&str>, slot: &mut bool) {
    match parse_repl_bool(value, *slot) {
        Ok(value) => {
            *slot = value;
            eprintln!("{name} = {value}");
        }
        Err(()) => eprintln!("usage: :set {name} [true|false]"),
    }
}

/// Parses a boolean setting value. A missing value toggles the current state,
/// so the common `:set show-layout` flips it without typing true or false.
fn parse_repl_bool(value: Option<&str>, current: bool) -> Result<bool, ()> {
    match value {
        None => Ok(!current),
        Some("true") | Some("on") => Ok(true),
        Some("false") | Some("off") => Ok(false),
        Some(_) => Err(()),
    }
}
