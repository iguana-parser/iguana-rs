use std::path::PathBuf;

use clap::{Parser, ValueEnum};

/// The output format `--format` selects for `--trace`, `--write-sppf`, and
/// `--write-gss`.
#[derive(Clone, Copy, ValueEnum)]
pub enum Format {
    Text,
    Json,
    Svg,
}

/// The command-line arguments of a generated parser. The about line names the
/// grammar, so `main` sets it when it builds the command.
#[derive(Parser)]
#[command(arg_required_else_help = true)]
pub struct Args {
    /// Input file to parse
    ///
    /// Required unless --list-nonterminals, --dir, or --benchmark is used
    pub file: Option<PathBuf>,
    /// Don't print the parse tree to stdout
    ///
    /// Status messages (timing, errors) still go to stderr; redirect with 2>/dev/null to silence them too
    #[arg(short, long)]
    pub quiet: bool,
    /// Nonterminal to start parsing from
    ///
    /// Required unless --list-nonterminals is used
    #[arg(long = "start", value_name = "NAME", help_heading = "Parsing")]
    pub start_nonterminal: Option<String>,
    /// Directory to recursively parse all files in
    ///
    /// Reports per-file success or failure with timings, plus a summary at the end
    #[arg(
        long,
        value_name = "DIR",
        conflicts_with = "file",
        help_heading = "Parsing"
    )]
    pub dir: Option<PathBuf>,
    /// File extension filter for --dir (e.g. "java")
    ///
    /// Without it, all files are parsed
    #[arg(long, value_name = "EXT", requires = "dir", help_heading = "Parsing")]
    pub ext: Option<String>,
    /// Run the parser in REPL mode
    ///
    /// Reads inputs from stdin and prints each parse tree. Requires --start; no input file is used
    #[arg(long, help_heading = "Parsing")]
    pub repl: bool,
    /// List the nonterminals declared in the grammar
    ///
    /// The valid values for --start, one per line, then exits
    #[arg(long, help_heading = "Grammar info")]
    pub list_nonterminals: bool,
    /// Write all nonterminals, terminals, and slots as JSON to a file
    ///
    /// Includes the derived nonterminals that --list-nonterminals hides
    #[arg(long, value_name = "FILE", help_heading = "Grammar info")]
    pub write_symbols: Option<PathBuf>,
    /// Show layout (whitespace, comments) nodes in the parse tree (false by default)
    #[arg(long, help_heading = "Parse-tree output")]
    pub show_layout: bool,
    /// Show empty optionals and repetitions (X?, X* that matched nothing) in the parse tree (false by default)
    #[arg(long, help_heading = "Parse-tree output")]
    pub show_empty: bool,
    /// Show wrapper nodes (start, optionals, groups, alternations) in the parse tree (false by default)
    #[arg(long, help_heading = "Parse-tree output")]
    pub show_wrappers: bool,
    /// Write the parse tree as JSON
    #[arg(long, value_name = "FILE", help_heading = "Output files")]
    pub write_parse_tree: Option<PathBuf>,
    /// Write the SPPF as JSON
    #[arg(long, value_name = "FILE", help_heading = "Output files")]
    pub write_sppf: Option<PathBuf>,
    /// Write the GSS graph as JSON
    ///
    /// Nodes with labels, plus edges
    #[arg(long, value_name = "FILE", help_heading = "Output files")]
    pub write_gss: Option<PathBuf>,
    /// Write GSS nodes as JSON
    ///
    /// Normalized with IDs so trace replay can map each GSS node to its nonterminal and input position
    #[arg(long, value_name = "FILE", help_heading = "Output files")]
    pub write_gss_nodes: Option<PathBuf>,
    /// Write the parse result as JSON
    ///
    /// On success includes timings; on failure includes the error span and message
    #[arg(long, value_name = "FILE", help_heading = "Output files")]
    pub write_result: Option<PathBuf>,
    /// Output format for --trace, --write-sppf, and --write-gss
    ///
    /// text: --trace only. json: any of the three, and the default for each. svg: --write-sppf and --write-gss only, rendered from DOT by the graphviz `dot` binary. A format that does not apply to the requested output is rejected.
    #[arg(long, value_enum, help_heading = "Output files")]
    pub format: Option<Format>,
    /// Enable trace output (writes to stdout, or a file if given)
    #[arg(long, value_name = "FILE", help_heading = "Tracing")]
    pub trace: Option<Option<PathBuf>>,
    /// Run the parser many times and report timing statistics
    ///
    /// Benchmarks a single file (the positional argument) or a --dir; otherwise the corpus listed in repos.txt. Reports min, mean, median, p90, max, stddev (in ms) for each phase: input (file read), init (allocation), parse (input characters to the SPPF), tree (SPPF to parse tree), drop (teardown); total is their sum
    #[arg(long, help_heading = "Benchmarking and profiling")]
    pub benchmark: bool,
    /// Number of measured iterations for --benchmark
    ///
    /// Defaults to 100 for a single file, 3 for a directory or corpus (one iteration is a full pass over every file)
    #[arg(
        long,
        value_name = "N",
        requires = "benchmark",
        help_heading = "Benchmarking and profiling"
    )]
    pub iters: Option<u32>,
    /// Number of warmup iterations before measurement
    ///
    /// Defaults to 10 for a single file, 0 for a directory or corpus (a whole-corpus pass self-warms, so a cold first pass is just the median's slow outlier)
    #[arg(
        long,
        value_name = "N",
        requires = "benchmark",
        help_heading = "Benchmarking and profiling"
    )]
    pub warmup: Option<u32>,
    /// Save benchmark samples to a JSON file
    ///
    /// Pairs with --baseline for A/B comparison across runs
    #[arg(
        long,
        value_name = "FILE",
        requires = "benchmark",
        help_heading = "Benchmarking and profiling"
    )]
    pub save: Option<PathBuf>,
    /// Compare benchmark results against a saved baseline JSON
    ///
    /// Reports the mean delta with a 95% CI (confidence interval) on the difference; flags the run as improved/regressed/no-change
    #[arg(
        long,
        value_name = "FILE",
        requires = "benchmark",
        help_heading = "Benchmarking and profiling"
    )]
    pub baseline: Option<PathBuf>,
    /// Profile the parser and write a flamegraph SVG
    ///
    /// Runs the parser N times in a loop under a sampling profiler. Requires the "profile" feature: cargo build --features profile
    #[arg(long, value_name = "N", help_heading = "Benchmarking and profiling")]
    pub profile: Option<u32>,
    /// Output path for the flamegraph SVG (used with --profile)
    #[arg(
        long,
        value_name = "FILE",
        default_value = "flamegraph.svg",
        help_heading = "Benchmarking and profiling"
    )]
    pub profile_output: PathBuf,
    /// Compare each input's output against its sibling X.sexpr
    ///
    /// Works with a single file or --dir (which then requires --ext). Golden files hold the parse-tree s-expression on success or the rendered parse error on failure
    #[arg(
        long,
        conflicts_with = "benchmark",
        conflicts_with = "profile",
        help_heading = "Golden-file testing"
    )]
    pub check_sexpr: bool,
    /// Write each input's output to its sibling X.sexpr, overwriting
    ///
    /// Same input rules as --check-sexpr
    #[arg(
        long,
        conflicts_with = "benchmark",
        conflicts_with = "profile",
        conflicts_with = "check_sexpr",
        help_heading = "Golden-file testing"
    )]
    pub regenerate_sexpr: bool,
    /// Print golden diffs in full instead of truncating past 200 lines
    #[arg(long, requires = "check_sexpr", help_heading = "Golden-file testing")]
    pub full_diff: bool,
    /// Compare each corpus against its committed baseline
    ///
    /// Parses the corpora listed in <corpus-dir>/repos.txt. With a NAME, restrict to that corpus; otherwise run all. Add --update to rewrite the baselines instead of checking
    #[arg(
        long,
        value_name = "NAME",
        conflicts_with_all = ["benchmark", "profile", "check_sexpr", "regenerate_sexpr", "repl", "dir"],
        help_heading = "Corpus testing"
    )]
    pub corpus_test: Option<Option<String>>,
    /// Rewrite corpus baselines instead of checking them
    ///
    /// Use with --corpus-test. Refuses to rewrite (and fails) when a file regressed (ok -> error) or parsed ambiguously
    #[arg(long, requires = "corpus_test", help_heading = "Corpus testing")]
    pub update: bool,
    /// Directory holding repos.txt, the per-corpus baselines, and the .cache/ checkouts
    ///
    /// Used with --corpus-test
    #[arg(
        long,
        value_name = "DIR",
        default_value = "corpus",
        help_heading = "Corpus testing"
    )]
    pub corpus_dir: PathBuf,
    /// Write parser stats (counters and histograms) as JSON
    ///
    /// Requires the "instrument" feature
    #[arg(long, value_name = "FILE", help_heading = "Stats")]
    pub write_stats: Option<PathBuf>,
    /// Print only the parser stats histogram and exit
    ///
    /// Suppresses the printed parse tree, the "Parse success" line, and (with --dir) the per-file timing lines and the aggregate timing summary. Requires the "instrument" feature
    #[arg(long, help_heading = "Stats")]
    pub hist: bool,
}
