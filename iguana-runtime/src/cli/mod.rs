//! The command-line interface of a generated parser. A generated crate's
//! `main.rs` calls `main` with its parser type, and the runtime owns the
//! arguments, the modes, and the reporting. The interface is behind the `cli`
//! feature, which pulls in `clap`. The JSON reports `ParseOutput` and
//! `Symbols` are always compiled: Terrarium reads both, and the WebAssembly
//! wrapper returns a `ParseOutput`.

use serde::{Deserialize, Serialize};
use specta::Type;

use crate::result::ParseError;

#[cfg(feature = "cli")]
mod args;
#[cfg(feature = "cli")]
mod batch;
#[cfg(feature = "cli")]
mod benchmark;
#[cfg(feature = "cli")]
mod corpus;
#[cfg(feature = "cli")]
mod golden_file;
#[cfg(feature = "cli")]
mod repl;
#[cfg(feature = "cli")]
mod run;

#[cfg(feature = "cli")]
pub use run::main;

/// The names of every nonterminal, terminal, and slot of a grammar, written
/// as JSON by `--write-symbols`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Symbols {
    pub nonterminals: Vec<String>,
    pub terminals: Vec<String>,
    pub slots: Vec<String>,
}

/// The report of one parse, written as JSON by `--write-result` and returned
/// by the WebAssembly wrapper. A failure sets `error`; a success sets the
/// timings. `parse_tree` is inline only in the WebAssembly envelope, where the
/// host has no file to read the tree from; `--write-result` leaves it `None`,
/// since the tree has its own flag.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ParseOutput {
    pub error: Option<ParseError>,
    pub parse_ms: Option<u32>,
    pub tree_construction_ms: Option<u32>,
    pub parse_tree: Option<String>,
}

/// ANSI colors, enabled only when stdout is a terminal so redirected output
/// stays plain.
#[cfg(feature = "cli")]
struct Color {
    green: &'static str,
    red: &'static str,
    bold: &'static str,
    reset: &'static str,
}

#[cfg(feature = "cli")]
impl Color {
    fn for_stdout() -> Self {
        use std::io::IsTerminal;

        if std::io::stdout().is_terminal() {
            Color {
                green: "\x1b[32m",
                red: "\x1b[31m",
                bold: "\x1b[1m",
                reset: "\x1b[0m",
            }
        } else {
            Color {
                green: "",
                red: "",
                bold: "",
                reset: "",
            }
        }
    }
}

/// Prints one per-file status line, coloring the status token green when `good`
/// and red otherwise.
#[cfg(feature = "cli")]
fn print_status(color: &Color, status: &str, good: bool, rest: &str) {
    let code = if good { color.green } else { color.red };
    println!("{}{:<6}{}{}", code, status, color.reset, rest);
}

/// Collects the files under `dir`, recursively, into `out`. With `ext`, only
/// files with that extension are collected.
#[cfg(feature = "cli")]
fn collect_files(
    dir: &std::path::Path,
    ext: Option<&str>,
    out: &mut Vec<std::path::PathBuf>,
) -> std::io::Result<()> {
    for entry in std::fs::read_dir(dir)? {
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

/// Reads `--start`, which every parsing mode requires.
#[cfg(feature = "cli")]
fn start_nonterminal_name(args: &args::Args) -> std::io::Result<&str> {
    args.start_nonterminal.as_deref().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "--start is required for parsing",
        )
    })
}

/// Resolves a user-supplied start nonterminal name to the id of its
/// generated `StartA` wrapper, so `--start A` resolves to `StartA`. A name
/// with no wrapper (a typo, the layout nonterminal, or a nonterminal that
/// desugaring introduced) is not an entry point and is an error.
#[cfg(feature = "cli")]
fn start_nonterminal_id<G: crate::grammar::Grammar>(
    name: &str,
) -> std::io::Result<crate::ids::NonterminalId> {
    G::start_nonterminal_id(name).ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!(
                "Unknown start nonterminal: '{}'. Use --list-nonterminals to see the valid names.",
                name,
            ),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::input::Span;

    #[test]
    fn parse_output_success_round_trips_as_json() {
        let output = ParseOutput {
            error: None,
            parse_ms: Some(12),
            tree_construction_ms: Some(3),
            parse_tree: None,
        };
        let json = serde_json::to_string(&output).unwrap();
        assert_eq!(
            json,
            r#"{"error":null,"parse_ms":12,"tree_construction_ms":3,"parse_tree":null}"#
        );
        let back: ParseOutput = serde_json::from_str(&json).unwrap();
        assert!(back.error.is_none());
        assert_eq!(back.parse_ms, Some(12));
        assert_eq!(back.tree_construction_ms, Some(3));
    }

    #[test]
    fn parse_output_failure_round_trips_as_span_and_message_json() {
        let output = ParseOutput {
            error: Some(ParseError {
                span: Span::new(4, 7),
                message: "Expected Id".to_string(),
            }),
            parse_ms: None,
            tree_construction_ms: None,
            parse_tree: None,
        };
        let json = serde_json::to_string(&output).unwrap();
        assert_eq!(
            json,
            r#"{"error":{"span":{"left_extent":4,"right_extent":7},"message":"Expected Id"},"parse_ms":null,"tree_construction_ms":null,"parse_tree":null}"#
        );
        let error = serde_json::from_str::<ParseOutput>(&json)
            .unwrap()
            .error
            .unwrap();
        assert_eq!(error.span, Span::new(4, 7));
        assert_eq!(error.message, "Expected Id");
    }

    #[test]
    fn parse_output_inlines_the_parse_tree_json() {
        let output = ParseOutput {
            error: None,
            parse_ms: Some(2),
            tree_construction_ms: Some(1),
            parse_tree: Some(r#"{"id":0}"#.to_string()),
        };
        let json = serde_json::to_string(&output).unwrap();
        assert_eq!(
            json,
            r#"{"error":null,"parse_ms":2,"tree_construction_ms":1,"parse_tree":"{\"id\":0}"}"#
        );
        let back: ParseOutput = serde_json::from_str(&json).unwrap();
        assert_eq!(back.parse_tree.as_deref(), Some(r#"{"id":0}"#));
    }
}
