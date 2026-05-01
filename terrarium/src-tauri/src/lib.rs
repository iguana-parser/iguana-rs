mod trace_replay;

use std::{
    cell::Cell,
    fs,
    fs::OpenOptions,
    io::Write,
    panic,
    path::{Path, PathBuf},
    process::Command,
    sync::{Mutex, OnceLock},
    thread,
};

use iguana::input::Input;
use iguana::parse_tree::ParseContext;
use iguana::visualization::{gss::GSS, sppf::SPPF};
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{Emitter, Manager};
use tauri_plugin_opener::OpenerExt;
use tauri_specta::{collect_commands, Builder};
use tempfile::{NamedTempFile, TempDir};
use toml::Value;

use trace_replay::{DebugGSSInfo, DebugSPPFNode, ErrorInfo, EventLogEntry, TraceReplay};

static LOG_DIR: OnceLock<PathBuf> = OnceLock::new();

fn log_file_path() -> Option<PathBuf> {
    LOG_DIR.get().map(|d| d.join("terrarium.log"))
}

// Thread-local panic capture: the panic hook stores the formatted message here
// when CAPTURING is true (inside catch_unwind). The catch_unwind caller reads
// it and routes through log_event so both file and UI get the same content.
thread_local! {
    static CAPTURING: Cell<bool> = const { Cell::new(false) };
    static LAST_PANIC: Cell<Option<String>> = const { Cell::new(None) };
}

fn log_to_file(kind: &str, message: &str) {
    let Some(path) = log_file_path() else { return };
    if let Some(dir) = path.parent() {
        let _ = fs::create_dir_all(dir);
    }
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&path) {
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
        let _ = writeln!(file, "[{timestamp}] [{kind}] {message}");
    }
}

fn log_event(app: &tauri::AppHandle, kind: &str, message: &str) {
    let timestamp = chrono::Local::now().format("%H:%M:%S").to_string();
    log_to_file(kind, message);
    let _ = app.emit(
        "log",
        LogEntry {
            kind: kind.to_string(),
            message: message.to_string(),
            timestamp,
        },
    );
}

/// Specta-compatible wrapper for lsp_types::SemanticToken
#[derive(Clone, Serialize, Type)]
struct SemanticTokenData {
    delta_line: u32,
    delta_start: u32,
    length: u32,
    token_type: u32,
    token_modifiers_bitset: u32,
}

/// Semantic token legend (token type names).
#[derive(Clone, Serialize, Type)]
struct SemanticTokensLegendData {
    token_types: Vec<String>,
}

#[derive(Clone, Serialize, Type)]
struct RangeData {
    start_line: u32,
    start_char: u32,
    end_line: u32,
    end_char: u32,
}

/// Specta-compatible wrapper for lsp_types::DocumentSymbol.
/// `kind` is the numeric LSP SymbolKind (e.g. 5 = Class, 9 = Constructor, 10 = Enum).
#[derive(Clone, Serialize, Type)]
struct DocumentSymbolData {
    name: String,
    kind: u32,
    range: RangeData,
    selection_range: RangeData,
    children: Vec<DocumentSymbolData>,
}

#[derive(Clone, Serialize, Type)]
struct LocationData {
    range: RangeData,
}

#[derive(Clone, Serialize, Type)]
struct FoldingRangeData {
    start_line: u32,
    end_line: u32,
}

#[derive(Clone, Serialize, Type)]
struct DiagnosticData {
    range: RangeData,
    severity: u32,
    message: String,
}

/// Debug SPPF info returned to the frontend.
#[derive(Clone, Serialize, Type)]
struct DebugSPPFInfo {
    nodes: Vec<DebugSPPFNode>,
    /// The current SPPF node ID from the descriptor being processed
    current_node_id: Option<u32>,
}

#[derive(Clone, Serialize, Type)]
struct LogEntry {
    kind: String,
    message: String,
    timestamp: String,
}

#[derive(Clone, Serialize, Type)]
struct BuildResult {
    success: bool,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    duration_ms: Option<u64>,
    /// Features the binary was built with. None when build failed or for non-build events.
    #[serde(skip_serializing_if = "Option::is_none")]
    features: Option<BuildFeatures>,
}

/// Cargo features the current parser binary was built with.
#[derive(Clone, Copy, Default, Serialize, Type)]
struct BuildFeatures {
    instrument: bool,
    debug_trace: bool,
}

/// Parser stats from --write-stats output (instrument feature).
#[derive(Clone, Serialize, Deserialize, Type)]
struct StatsData {
    descriptors_count: u32,
    gss_nodes_count: u32,
    gss_edges_count: u32,
    nonterminal_nodes_count: u32,
    intermediate_nodes_count: u32,
    terminal_nodes_count: u32,
    ambiguous_nodes_count: u32,
    /// Map from collection name (e.g. "Env::bindings: InlineVec") to recorded sizes.
    histograms: std::collections::BTreeMap<String, Vec<u32>>,
}

/// Result of analyzing/parsing a grammar source.
#[derive(Clone, Serialize, Type)]
struct AnalyzeResult {
    success: bool,
    /// Time spent in the GLL parsing algorithm (milliseconds).
    parse_duration_ms: u32,
    /// Time spent constructing the typed parse tree from the SPPF (milliseconds).
    tree_construction_duration_ms: u32,
}

/// Cached grammar state. Each command that needs the parse tree re-parses
/// (iggy grammars parse in a few milliseconds). `grammar_def` is cached
/// and rebuilt only when the source text changes.
struct GrammarState {
    source: Option<String>,
    input: Option<Input>,
    grammar_def: Option<iguana::grammar::def::GrammarDef>,
    /// Cached timings from the last parse.
    parse_ms: u32,
    tree_ms: u32,
    /// Whether the last parse succeeded.
    success: bool,
}

impl GrammarState {
    /// Ensure `grammar_def` is up-to-date for `source`. Re-parses only when
    /// the source text has changed. Returns the parse timings.
    fn ensure_built(&mut self, source: &str) -> (u32, u32) {
        if self.source.as_deref() == Some(source) {
            return (self.parse_ms, self.tree_ms);
        }
        let input = Input::from(source);
        let ctx = ParseContext::new();
        let result = iguana_lsp::build(&input, &ctx);
        match &result {
            iguana_lsp::BuildResult::Success {
                ref tree,
                parse_duration,
                tree_construction_duration,
            } => {
                self.grammar_def = iguana_lsp::build_grammar_def(tree, &input);
                self.parse_ms = parse_duration.as_millis() as u32;
                self.tree_ms = tree_construction_duration.as_millis() as u32;
                self.success = true;
            }
            iguana_lsp::BuildResult::Error { .. } => {
                self.grammar_def = None;
                self.parse_ms = 0;
                self.tree_ms = 0;
                self.success = false;
            }
        };
        self.source = Some(source.to_string());
        self.input = Some(input);
        (self.parse_ms, self.tree_ms)
    }

    /// Parse the current source and return the tree. Returns None if no source
    /// is set or parsing fails.
    fn parse<'a>(&self, ctx: &'a ParseContext) -> Option<iguana_lsp::BuildResult<'a>> {
        let input = self.input.as_ref()?;
        match iguana_lsp::build(input, ctx) {
            result @ iguana_lsp::BuildResult::Success { .. } => Some(result),
            _ => None,
        }
    }
}

/// Tracks the cargo features the current parser binary was built with.
/// `None` means no successful build yet (or the binary is stale).
#[derive(Default)]
struct BuildState {
    features: Option<BuildFeatures>,
}

#[derive(Default)]
struct ParseState {
    _temp_dir: Option<TempDir>,
    sppf_path: Option<PathBuf>,
    gss_path: Option<PathBuf>,
    parse_tree_path: Option<PathBuf>,
}

/// Parse error location and message, from the parser's --write-result JSON.
#[derive(Clone, Serialize, Type)]
struct ParseErrorInfo {
    line: u32,
    column: u32,
    message: String,
}

/// Result of a parse operation, indicating which outputs are available.
#[derive(Clone, Serialize, Type)]
struct ParseOutput {
    success: bool,
    error: Option<String>,
    error_info: Option<ParseErrorInfo>,
    duration_ms: Option<u32>,
    tree_construction_ms: Option<u32>,
    has_sppf: bool,
    has_gss: bool,
    has_parse_tree: bool,
}

#[derive(Default)]
struct DebugState {
    replay: Option<TraceReplay>,
}

/// Debug info returned to the frontend.
#[derive(Clone, Serialize, Type)]
struct DebugInfo {
    current_step: u32,
    total_steps: u32,
    /// The current action being displayed (formatted as string)
    current_action: Option<String>,
    /// Pending descriptors in the worklist (formatted as strings)
    descriptor_set: Vec<String>,
    /// Current position in the input (character index)
    input_index: Option<u32>,
    /// Total number of error steps (ParseError events)
    total_errors: u32,
    /// Current error index (1-indexed) if at an error step, None otherwise
    current_error_index: Option<u32>,
    /// Only set on initial load
    input_path: Option<String>,
    symbols_path: Option<String>,
    trace_path: Option<String>,
}

fn read_parser_name(directory: &str) -> Result<String, String> {
    let cargo_path = Path::new(directory).join("Cargo.toml");
    let content = fs::read_to_string(&cargo_path)
        .map_err(|_| "No valid Iguana parser found in this directory.".to_string())?;

    let toml: Value = content
        .parse()
        .map_err(|_| "No valid Iguana parser found in this directory.".to_string())?;

    toml["package"]["name"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| "No valid Iguana parser found in this directory.".to_string())
}

fn find_parser_binary(directory: &str, profile: &str) -> Result<PathBuf, String> {
    let parser_name = read_parser_name(directory)?;
    let dir_path = Path::new(directory);

    // First, check local target directory (standalone project)
    let local_path = dir_path.join("target").join(profile).join(&parser_name);
    if local_path.exists() {
        return Ok(local_path);
    }

    // For Cargo workspace members, the binary is in the workspace root's target directory.
    // Walk up to find a parent Cargo.toml with [workspace].
    let mut current = dir_path.to_path_buf();
    while let Some(parent) = current.parent() {
        let parent_cargo = parent.join("Cargo.toml");
        if parent_cargo.exists() {
            if let Ok(content) = fs::read_to_string(&parent_cargo) {
                if content.contains("[workspace]") {
                    let workspace_path =
                        parent.join("target").join(profile).join(&parser_name);
                    if workspace_path.exists() {
                        return Ok(workspace_path);
                    }
                }
            }
        }
        current = parent.to_path_buf();
    }

    Err(format!(
        "Parser binary '{}' not found in {} profile. Please build first.",
        parser_name, profile
    ))
}

fn get_parser_binary_path(directory: &str) -> Result<PathBuf, String> {
    find_parser_binary(directory, "release")
        .or_else(|_| find_parser_binary(directory, "debug"))
}

#[tauri::command]
#[specta::specta]
fn get_parser_name(directory: String) -> Result<String, String> {
    read_parser_name(&directory)
}

#[tauri::command]
#[specta::specta]
fn load_grammar(directory: String) -> Result<(String, String), String> {
    let dir = Path::new(&directory);
    let iggy_files: Vec<_> = fs::read_dir(dir)
        .map_err(|e| format!("Cannot read directory: {e}"))?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry
                .path()
                .extension()
                .is_some_and(|ext| ext == "iggy")
        })
        .collect();

    match iggy_files.len() {
        0 => Err("No .iggy file found in this directory.".to_string()),
        1 => {
            let path = iggy_files[0].path();
            let content = fs::read_to_string(&path)
                .map_err(|e| format!("Cannot read grammar file: {e}"))?;
            let filename = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            Ok((filename, content))
        }
        n => Err(format!("Found {n} .iggy files. Expected exactly one.")),
    }
}

#[tauri::command]
#[specta::specta]
fn save_grammar(directory: String, filename: String, content: String) -> Result<(), String> {
    let path = Path::new(&directory).join(&filename);
    fs::write(&path, &content).map_err(|e| format!("Cannot save grammar file: {e}"))
}

#[tauri::command]
#[specta::specta]
fn build_parser(
    directory: String,
    instrument: bool,
    debug_trace: bool,
    app: tauri::AppHandle,
    state: tauri::State<'_, Mutex<BuildState>>,
) {
    // Build feature list. `profile` is always on (no runtime cost when --profile not passed).
    let mut features = vec!["profile".to_string()];
    if instrument {
        features.push("instrument".into());
    }
    if debug_trace {
        features.push("debug-trace".into());
    }
    let features_arg = features.join(",");
    let built_features = BuildFeatures { instrument, debug_trace };

    // Mark current binary as stale until rebuild completes.
    state.lock().unwrap().features = None;

    thread::spawn(move || {
        log_event(
            &app,
            "command",
            &format!("cargo build --release --features {features_arg}"),
        );

        let build_output = Command::new("cargo")
            .args(["build", "--release", "--features", &features_arg])
            .current_dir(&directory)
            .output();

        match build_output {
            Ok(output) if output.status.success() => {
                if let Some(state) = app.try_state::<Mutex<BuildState>>() {
                    state.lock().unwrap().features = Some(built_features);
                }
                log_event(&app, "output", "Build successful");
                let _ = app.emit(
                    "build-result",
                    BuildResult {
                        success: true,
                        message: "Build successful".into(),
                        duration_ms: None,
                        features: Some(built_features),
                    },
                );
            }
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                log_event(&app, "error", &format!("Build failed\n{stderr}"));
                let _ = app.emit(
                    "build-result",
                    BuildResult {
                        success: false,
                        message: stderr.into_owned(),
                        duration_ms: None,
                        features: None,
                    },
                );
            }
            Err(e) => {
                let msg = format!("Failed to run cargo build: {e}");
                log_event(&app, "error", &msg);
                let _ = app.emit(
                    "build-result",
                    BuildResult {
                        success: false,
                        message: msg,
                        duration_ms: None,
                        features: None,
                    },
                );
            }
        }
    });
}

#[tauri::command]
#[specta::specta]
fn parse(
    directory: String,
    input: String,
    start_nonterminal: String,
    state: tauri::State<Mutex<ParseState>>,
) -> Result<ParseOutput, String> {
    let parser_path = get_parser_binary_path(&directory)?;

    let mut input_file = NamedTempFile::new()
        .map_err(|e| format!("Failed to create temp file: {}", e))?;
    input_file
        .write_all(input.as_bytes())
        .map_err(|e| format!("Failed to write input: {}", e))?;

    let temp_dir =
        TempDir::new().map_err(|e| format!("Failed to create temp directory: {}", e))?;
    let sppf_path = temp_dir.path().join("sppf.json");
    let gss_path = temp_dir.path().join("gss.json");
    let parse_tree_path = temp_dir.path().join("parse_tree.json");
    let result_path = temp_dir.path().join("result.json");

    let output = Command::new(&parser_path)
        .env("RUST_BACKTRACE", "1")  // Always show backtraces for debugging
        .arg(input_file.path())
        .arg("--start")
        .arg(&start_nonterminal)
        .arg("--write-sppf")
        .arg(&sppf_path)
        .arg("--write-gss")
        .arg(&gss_path)
        .arg("--write-parse-tree")
        .arg(&parse_tree_path)
        .arg("--write-result")
        .arg(&result_path)
        .output()
        .map_err(|e| format!("Failed to run parser: {}", e))?;

    let stderr = String::from_utf8_lossy(&output.stderr);

    // Check which output files were created (regardless of parser exit status)
    let has_sppf = sppf_path.exists();
    let has_gss = gss_path.exists();
    let has_parse_tree = parse_tree_path.exists();

    // Store paths for files that exist
    let mut parse_state = state.lock().unwrap();
    parse_state._temp_dir = Some(temp_dir);
    parse_state.sppf_path = if has_sppf { Some(sppf_path) } else { None };
    parse_state.gss_path = if has_gss { Some(gss_path) } else { None };
    parse_state.parse_tree_path = if has_parse_tree { Some(parse_tree_path) } else { None };

    // Read parse result from JSON file written by --write-result
    let result = result_path
        .exists()
        .then(|| {
            fs::read_to_string(&result_path)
                .ok()
                .and_then(|s| serde_json::from_str::<iguana::cli::ParseResult>(&s).ok())
        })
        .flatten();

    // Parser process failed to run (crash, not a parse error)
    if !output.status.success() && result.is_none() {
        return Ok(ParseOutput {
            success: false,
            error: Some(format!("Parser error: {}", stderr.trim())),
            error_info: None,
            duration_ms: None,
            tree_construction_ms: None,
            has_sppf,
            has_gss,
            has_parse_tree,
        });
    }

    match result {
        Some(iguana::cli::ParseResult::Success(s)) => Ok(ParseOutput {
            success: true,
            error: None,
            error_info: None,
            duration_ms: Some(s.parse_ms as u32),
            tree_construction_ms: s.tree_construction_ms.map(|n| n as u32),
            has_sppf,
            has_gss,
            has_parse_tree,
        }),
        Some(iguana::cli::ParseResult::Failure(f)) => Ok(ParseOutput {
            success: false,
            error: Some(format!(
                "Parse failed at line {}, column {}: {}",
                f.line, f.column, f.message
            )),
            error_info: Some(ParseErrorInfo {
                line: f.line,
                column: f.column,
                message: f.message,
            }),
            duration_ms: None,
            tree_construction_ms: None,
            has_sppf,
            has_gss,
            has_parse_tree,
        }),
        // Fallback: no result file (shouldn't happen with current generator)
        None => Ok(ParseOutput {
            success: false,
            error: Some("No parse result available".to_string()),
            error_info: None,
            duration_ms: None,
            tree_construction_ms: None,
            has_sppf,
            has_gss,
            has_parse_tree,
        }),
    }
}

#[tauri::command]
#[specta::specta]
fn get_sppf(state: tauri::State<Mutex<ParseState>>) -> Result<SPPF, String> {
    let parse_state = state.lock().unwrap();
    let sppf_path = parse_state
        .sppf_path
        .as_ref()
        .ok_or("No parse result available. Run parse first.")?;

    let content = fs::read_to_string(sppf_path)
        .map_err(|e| format!("Failed to read SPPF file: {}", e))?;

    serde_json::from_str(&content).map_err(|e| format!("Failed to parse SPPF JSON: {}", e))
}

#[tauri::command]
#[specta::specta]
fn get_gss(state: tauri::State<Mutex<ParseState>>) -> Result<GSS, String> {
    let parse_state = state.lock().unwrap();
    let gss_path = parse_state
        .gss_path
        .as_ref()
        .ok_or("No parse result available. Run parse first.")?;

    let content =
        fs::read_to_string(gss_path).map_err(|e| format!("Failed to read GSS file: {}", e))?;

    serde_json::from_str(&content).map_err(|e| format!("Failed to parse GSS JSON: {}", e))
}

/// Returns the parse tree JSON as a string.
/// The frontend will parse this JSON directly.
#[tauri::command]
#[specta::specta]
fn get_parse_tree(state: tauri::State<Mutex<ParseState>>) -> Result<String, String> {
    let parse_state = state.lock().unwrap();
    let parse_tree_path = parse_state
        .parse_tree_path
        .as_ref()
        .ok_or("No parse result available. Run parse first.")?;

    fs::read_to_string(parse_tree_path)
        .map_err(|e| format!("Failed to read parse tree file: {}", e))
}

/// Returns the cargo features the current parser binary was built with,
/// or `None` if no successful build has happened yet.
#[tauri::command]
#[specta::specta]
fn get_build_features(state: tauri::State<Mutex<BuildState>>) -> Option<BuildFeatures> {
    state.lock().unwrap().features
}

/// Run the parser with --write-stats and return the parsed Stats JSON.
/// Requires the binary to have been built with the `instrument` feature.
#[tauri::command]
#[specta::specta]
fn get_stats(
    directory: String,
    input: String,
    start_nonterminal: String,
) -> Result<StatsData, String> {
    let parser_path = get_parser_binary_path(&directory)?;

    let mut input_file = NamedTempFile::new()
        .map_err(|e| format!("Failed to create temp file: {}", e))?;
    input_file
        .write_all(input.as_bytes())
        .map_err(|e| format!("Failed to write input: {}", e))?;

    let temp_dir =
        TempDir::new().map_err(|e| format!("Failed to create temp directory: {}", e))?;
    let stats_path = temp_dir.path().join("stats.json");

    let output = Command::new(&parser_path)
        .arg(input_file.path())
        .arg("--start")
        .arg(&start_nonterminal)
        .arg("--write-stats")
        .arg(&stats_path)
        .output()
        .map_err(|e| format!("Failed to run parser: {}", e))?;

    if !stats_path.exists() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "Parser did not write stats. Was it built with --features instrument? {}",
            stderr.trim()
        ));
    }

    let content = fs::read_to_string(&stats_path)
        .map_err(|e| format!("Failed to read stats file: {}", e))?;
    serde_json::from_str(&content).map_err(|e| format!("Failed to parse stats JSON: {}", e))
}

/// Profile the parser by building with --features profile (release mode),
/// running the parser in a loop under a sampling profiler, and opening the
/// resulting flamegraph SVG in the default browser.
#[tauri::command]
#[specta::specta]
fn profile(
    directory: String,
    input: String,
    start_nonterminal: String,
    iterations: u32,
    app: tauri::AppHandle,
) {
    thread::spawn(move || {
        log_event(
            &app,
            "command",
            &format!("<parser> <input> --start {start_nonterminal} --profile {iterations}"),
        );

        let parser_path = match find_parser_binary(&directory, "release") {
            Ok(p) => p,
            Err(e) => {
                log_event(&app, "error", &e);
                let _ = app.emit(
                    "profile-result",
                    BuildResult {
                        success: false,
                        message: e,
                        duration_ms: None,
                        features: None,
                    },
                );
                return;
            }
        };

        // Write input to a temp file
        let mut input_file = match NamedTempFile::new() {
            Ok(f) => f,
            Err(e) => {
                let msg = format!("Failed to create temp file: {e}");
                log_event(&app, "error", &msg);
                let _ = app.emit(
                    "profile-result",
                    BuildResult {
                        success: false,
                        message: msg,
                        duration_ms: None,
                        features: None,
                    },
                );
                return;
            }
        };
        if let Err(e) = input_file.write_all(input.as_bytes()) {
            let msg = format!("Failed to write input: {e}");
            log_event(&app, "error", &msg);
            let _ = app.emit(
                "profile-result",
                BuildResult {
                    success: false,
                    message: msg,
                    duration_ms: None,
                    features: None,
                },
            );
            return;
        }

        let flamegraph_path = Path::new(&directory).join("flamegraph.svg");

        let output = Command::new(&parser_path)
            .arg(input_file.path())
            .arg("--start")
            .arg(&start_nonterminal)
            .arg("--profile")
            .arg(iterations.to_string())
            .arg("--profile-output")
            .arg(&flamegraph_path)
            .output();

        match output {
            Ok(output) if output.status.success() => {
                let url = format!("file://{}", flamegraph_path.display());
                let _ = app.opener().open_url(&url, None::<&str>);
                let msg = format!("Flamegraph written to {}", flamegraph_path.display());
                log_event(&app, "output", &msg);
                let _ = app.emit(
                    "profile-result",
                    BuildResult {
                        success: true,
                        message: msg,
                        duration_ms: None,
                        features: None,
                    },
                );
            }
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                let msg = format!("Profiling failed: {stderr}");
                log_event(&app, "error", &msg);
                let _ = app.emit(
                    "profile-result",
                    BuildResult {
                        success: false,
                        message: msg,
                        duration_ms: None,
                        features: None,
                    },
                );
            }
            Err(e) => {
                let msg = format!("Failed to run parser: {e}");
                log_event(&app, "error", &msg);
                let _ = app.emit(
                    "profile-result",
                    BuildResult {
                        success: false,
                        message: msg,
                        duration_ms: None,
                        features: None,
                    },
                );
            }
        }
    });
}

/// Sets up VS Code debug configuration for the current parser.
/// Writes input to a file and creates/updates .vscode/launch.json.
#[tauri::command]
#[specta::specta]
fn setup_vscode_debug(
    directory: String,
    input: String,
    start_nonterminal: String,
) -> Result<String, String> {
    let dir_path = Path::new(&directory);

    // Get parser name from Cargo.toml
    let cargo_toml_path = dir_path.join("Cargo.toml");
    let cargo_content = fs::read_to_string(&cargo_toml_path)
        .map_err(|e| format!("Failed to read Cargo.toml: {}", e))?;
    let cargo_toml: Value = cargo_content
        .parse()
        .map_err(|e| format!("Failed to parse Cargo.toml: {}", e))?;
    let parser_name = cargo_toml
        .get("package")
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str())
        .ok_or("Could not find package name in Cargo.toml")?;

    // Create .vscode directory if it doesn't exist
    let vscode_dir = dir_path.join(".vscode");
    fs::create_dir_all(&vscode_dir)
        .map_err(|e| format!("Failed to create .vscode directory: {}", e))?;

    // Write input to .vscode/debug-input.txt (keeps parser directory clean)
    let input_path = vscode_dir.join("debug-input.txt");
    fs::write(&input_path, &input)
        .map_err(|e| format!("Failed to write debug input: {}", e))?;

    // Generate launch.json content
    let launch_json = format!(
        r#"{{
  "version": "0.2.0",
  "configurations": [
    {{
      "type": "lldb",
      "request": "launch",
      "name": "Debug Parser",
      "cargo": {{
        "args": ["build", "--manifest-path", "${{workspaceFolder}}/Cargo.toml"]
      }},
      "program": "${{workspaceFolder}}/target/debug/{parser_name}",
      "args": ["${{workspaceFolder}}/.vscode/debug-input.txt", "--start", "{start_nonterminal}"],
      "cwd": "${{workspaceFolder}}"
    }}
  ]
}}"#,
        parser_name = parser_name,
        start_nonterminal = start_nonterminal
    );

    // Write launch.json
    let launch_json_path = vscode_dir.join("launch.json");
    fs::write(&launch_json_path, &launch_json)
        .map_err(|e| format!("Failed to write launch.json: {}", e))?;

    Ok(directory)
}

#[tauri::command]
#[specta::specta]
fn get_nonterminals(directory: String) -> Result<Vec<String>, String> {
    let parser_path = get_parser_binary_path(&directory)?;

    let output = Command::new(&parser_path)
        .arg("--list-nonterminals")
        .output()
        .map_err(|e| format!("Failed to run parser: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to list nonterminals: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let nonterminals: Vec<String> = stdout
        .lines()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    Ok(nonterminals)
}

#[tauri::command]
#[specta::specta]
fn generate_parser(directory: String, no_ll1: bool, app: tauri::AppHandle) {
    thread::spawn(move || {
        let ll1_flag = if no_ll1 { " --no-ll1" } else { "" };
        log_event(&app, "command", &format!("iguana generate --output .{ll1_flag}"));

        CAPTURING.set(true);
        let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
            let dir = Path::new(&directory);
            let iggy_file = find_iggy_file(dir).map_err(|e| e.to_string())?;
            let source = fs::read_to_string(&iggy_file).map_err(|e| e.to_string())?;
            let grammar_def = iguana::iggy::parse_grammar(&source)
                .map_err(|e| format!("{}", e))?;
            let grammar: iguana::grammar::def::Grammar = grammar_def.try_into().map_err(|names: Vec<String>| {
                format!("Unresolved identifiers: {}", names.join(", "))
            })?;
            iguana::generator::generate_scaffold(&grammar, dir).map_err(|e| e.to_string())?;
            let result = iguana::generator::generate_sources(
                &grammar,
                dir,
                iguana::generator::GenConfig {
                    ll1_optimization: !no_ll1,
                },
            )
            .map_err(|e| e.to_string())?;
            Ok::<u64, String>(result.total_duration_ms)
        }));
        CAPTURING.set(false);

        let message = match result {
            Ok(Ok(duration_ms)) => {
                log_event(
                    &app,
                    "output",
                    &format!("Parser generated successfully ({duration_ms}ms)"),
                );
                let _ = app.emit(
                    "generate-result",
                    BuildResult {
                        success: true,
                        message: "Generation successful".into(),
                        duration_ms: Some(duration_ms),
                        features: None,
                    },
                );
                return;
            }
            Ok(Err(message)) => message,
            Err(_) => {
                let detail = LAST_PANIC.take().unwrap_or_else(|| "unknown error".into());
                format!("Internal error during generation:\n{detail}")
            }
        };

        log_event(&app, "error", &message);
        let _ = app.emit(
            "generate-result",
            BuildResult {
                success: false,
                message,
                duration_ms: None,
                features: None,
            },
        );
    });
}

fn find_iggy_file(directory: &Path) -> Result<PathBuf, std::io::Error> {
    let iggy_files: Vec<_> = fs::read_dir(directory)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "iggy"))
        .collect();
    match iggy_files.len() {
        0 => Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("No .iggy file found in {}", directory.display()),
        )),
        1 => Ok(iggy_files[0].path()),
        n => Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!(
                "Found {n} .iggy files in {}. Expected exactly one.",
                directory.display()
            ),
        )),
    }
}

// ============ Language Intelligence Commands ============

/// Parse the grammar source and cache the result. Re-parses only if the source
/// text has changed since the last parse.
#[tauri::command]
#[specta::specta]
fn analyze_grammar(source: String, state: tauri::State<Mutex<GrammarState>>) -> AnalyzeResult {
    let mut st = state.lock().unwrap();
    let (parse_duration_ms, tree_construction_duration_ms) = st.ensure_built(&source);

    AnalyzeResult {
        success: st.success,
        parse_duration_ms,
        tree_construction_duration_ms,
    }
}

/// Format the grammar using the cached parse result.
/// Falls back to parsing the source if no cached result exists.
/// Returns None if parsing fails (grammar cannot be formatted).
#[tauri::command]
#[specta::specta]
fn format_grammar(source: String, state: tauri::State<Mutex<GrammarState>>) -> Option<String> {
    let mut st = state.lock().unwrap();
    st.ensure_built(&source);
    let ctx = ParseContext::new();
    let iguana_lsp::BuildResult::Success { ref tree, .. } = st.parse(&ctx)? else {
        return None;
    };
    let input = st.input.as_ref()?;
    Some(iguana_lsp::format::format(tree, input))
}

/// Return semantic tokens from the cached parse result.
#[tauri::command]
#[specta::specta]
fn get_semantic_tokens(state: tauri::State<Mutex<GrammarState>>) -> Vec<SemanticTokenData> {
    let st = state.lock().unwrap();
    let ctx = ParseContext::new();
    let Some(iguana_lsp::BuildResult::Success { ref tree, .. }) = st.parse(&ctx) else {
        return vec![];
    };
    let Some(ref input) = st.input else {
        return vec![];
    };
    iguana_lsp::semantic_tokens::semantic_tokens(tree, input)
        .into_iter()
        .map(|t| SemanticTokenData {
            delta_line: t.delta_line,
            delta_start: t.delta_start,
            length: t.length,
            token_type: t.token_type,
            token_modifiers_bitset: t.token_modifiers_bitset,
        })
        .collect()
}

/// Return document symbols (rule heads + alternative labels).
/// Ensures the parse result is fresh for the given source.
#[tauri::command]
#[specta::specta]
fn get_document_symbols(
    source: String,
    state: tauri::State<Mutex<GrammarState>>,
) -> Vec<DocumentSymbolData> {
    let mut st = state.lock().unwrap();
    st.ensure_built(&source);
    let ctx = ParseContext::new();
    let Some(iguana_lsp::BuildResult::Success { ref tree, .. }) = st.parse(&ctx) else {
        return vec![];
    };
    let Some(ref input) = st.input else {
        return vec![];
    };
    let Some(ref grammar_def) = st.grammar_def else {
        return vec![];
    };
    let spans = iguana_lsp::build_spans(grammar_def, tree, input);
    iguana_lsp::document_symbols::document_symbols(grammar_def, &spans, input)
        .into_iter()
        .map(convert_symbol)
        .collect()
}

/// Return the definition location of the symbol at the given position.
#[tauri::command]
#[specta::specta]
fn get_definition(
    source: String,
    line: u32,
    column: u32,
    state: tauri::State<Mutex<GrammarState>>,
) -> Option<LocationData> {
    let mut st = state.lock().unwrap();
    st.ensure_built(&source);
    let ctx = ParseContext::new();
    let iguana_lsp::BuildResult::Success { ref tree, .. } = st.parse(&ctx)? else {
        return None;
    };
    let input = st.input.as_ref()?;
    let grammar_def = st.grammar_def.as_ref()?;
    let spans = iguana_lsp::build_spans(grammar_def, tree, input);
    let uri: lsp_types::Uri = "file:///terrarium".parse().unwrap();
    let offset = input.offset(line, column);
    let loc = iguana_lsp::references::definition(&spans, input, &uri, offset)?;
    Some(LocationData {
        range: RangeData {
            start_line: loc.range.start.line,
            start_char: loc.range.start.character,
            end_line: loc.range.end.line,
            end_char: loc.range.end.character,
        },
    })
}

/// Return all references to the symbol at the given position.
#[tauri::command]
#[specta::specta]
fn get_references(
    source: String,
    line: u32,
    column: u32,
    include_declaration: bool,
    state: tauri::State<Mutex<GrammarState>>,
) -> Vec<LocationData> {
    let mut st = state.lock().unwrap();
    st.ensure_built(&source);
    let ctx = ParseContext::new();
    let Some(iguana_lsp::BuildResult::Success { ref tree, .. }) = st.parse(&ctx) else {
        return vec![];
    };
    let Some(ref input) = st.input else {
        return vec![];
    };
    let Some(ref grammar_def) = st.grammar_def else {
        return vec![];
    };
    let spans = iguana_lsp::build_spans(grammar_def, tree, input);
    let uri: lsp_types::Uri = "file:///terrarium".parse().unwrap();
    let offset = input.offset(line, column);
    iguana_lsp::references::references(&spans, input, &uri, offset, include_declaration)
        .into_iter()
        .map(|loc| LocationData {
            range: RangeData {
                start_line: loc.range.start.line,
                start_char: loc.range.start.character,
                end_line: loc.range.end.line,
                end_char: loc.range.end.character,
            },
        })
        .collect()
}

/// Return folding ranges for the grammar.
#[tauri::command]
#[specta::specta]
fn get_folding_ranges(
    source: String,
    state: tauri::State<Mutex<GrammarState>>,
) -> Vec<FoldingRangeData> {
    let mut st = state.lock().unwrap();
    st.ensure_built(&source);
    let ctx = ParseContext::new();
    let Some(iguana_lsp::BuildResult::Success { ref tree, .. }) = st.parse(&ctx) else {
        return vec![];
    };
    let Some(ref input) = st.input else {
        return vec![];
    };
    let Some(ref grammar_def) = st.grammar_def else {
        return vec![];
    };
    let spans = iguana_lsp::build_spans(grammar_def, tree, input);
    iguana_lsp::folding::folding_ranges(grammar_def, &spans, input)
        .into_iter()
        .map(|r| FoldingRangeData {
            start_line: r.start_line,
            end_line: r.end_line,
        })
        .collect()
}

/// Return diagnostics (e.g. unresolved references) for the grammar.
#[tauri::command]
#[specta::specta]
fn get_diagnostics(
    source: String,
    state: tauri::State<Mutex<GrammarState>>,
) -> Vec<DiagnosticData> {
    let mut st = state.lock().unwrap();
    st.ensure_built(&source);
    let Some(ref input) = st.input else {
        return vec![];
    };
    let ctx = ParseContext::new();
    match iguana_lsp::build(input, &ctx) {
        iguana_lsp::BuildResult::Success { ref tree, .. } => {
            let Some(ref grammar_def) = st.grammar_def else {
                return vec![];
            };
            let spans = iguana_lsp::build_spans(grammar_def, tree, input);
            iguana_lsp::diagnostics::diagnostics(grammar_def, &spans, input)
                .into_iter()
                .map(|d| {
                    let severity = match d.severity {
                        Some(lsp_types::DiagnosticSeverity::ERROR) => 8,
                        Some(lsp_types::DiagnosticSeverity::WARNING) => 4,
                        Some(lsp_types::DiagnosticSeverity::INFORMATION) => 2,
                        Some(lsp_types::DiagnosticSeverity::HINT) => 1,
                        _ => 8,
                    };
                    DiagnosticData {
                        range: RangeData {
                            start_line: d.range.start.line,
                            start_char: d.range.start.character,
                            end_line: d.range.end.line,
                            end_char: d.range.end.character,
                        },
                        severity,
                        message: d.message,
                    }
                })
                .collect()
        }
        iguana_lsp::BuildResult::Error { line, column, message } => {
            vec![DiagnosticData {
                range: RangeData {
                    start_line: line,
                    start_char: column,
                    end_line: line,
                    end_char: column,
                },
                severity: 8,
                message: format!("Parse error: {message}"),
            }]
        }
    }
}

/// Map the LSP SymbolKind constants we actually use to their numeric codes.
/// The `.0` field of `lsp_types::SymbolKind` is private, so we match by const.
fn symbol_kind_code(kind: lsp_types::SymbolKind) -> u32 {
    use lsp_types::SymbolKind;
    match kind {
        SymbolKind::CLASS => 5,
        SymbolKind::CONSTRUCTOR => 9,
        SymbolKind::ENUM => 10,
        SymbolKind::INTERFACE => 11,
        _ => 1, // FILE, harmless fallback
    }
}

fn convert_symbol(s: lsp_types::DocumentSymbol) -> DocumentSymbolData {
    DocumentSymbolData {
        name: s.name,
        kind: symbol_kind_code(s.kind),
        range: RangeData {
            start_line: s.range.start.line,
            start_char: s.range.start.character,
            end_line: s.range.end.line,
            end_char: s.range.end.character,
        },
        selection_range: RangeData {
            start_line: s.selection_range.start.line,
            start_char: s.selection_range.start.character,
            end_line: s.selection_range.end.line,
            end_char: s.selection_range.end.character,
        },
        children: s
            .children
            .unwrap_or_default()
            .into_iter()
            .map(convert_symbol)
            .collect(),
    }
}

#[tauri::command]
#[specta::specta]
fn get_semantic_tokens_legend() -> SemanticTokensLegendData {
    let legend = iguana_lsp::semantic_tokens::legend();
    SemanticTokensLegendData {
        token_types: legend.token_types.iter().map(|t| t.as_str().to_string()).collect(),
    }
}

// ============ Debug Commands ============

#[tauri::command]
#[specta::specta]
fn load_debug_trace(
    directory: String,
    input: String,
    start_nonterminal: String,
    state: tauri::State<Mutex<DebugState>>,
) -> Result<DebugInfo, String> {
    let parser_path = get_parser_binary_path(&directory)?;

    // Write input to temp file
    let mut input_file =
        NamedTempFile::new().map_err(|e| format!("Failed to create temp file: {}", e))?;
    input_file
        .write_all(input.as_bytes())
        .map_err(|e| format!("Failed to write input: {}", e))?;

    // Create temp directory for trace and symbols
    let temp_dir =
        TempDir::new().map_err(|e| format!("Failed to create temp directory: {}", e))?;
    let trace_path = temp_dir.path().join("trace.json");
    let symbols_path = temp_dir.path().join("symbols.json");

    // First, get the symbols (static, doesn't need parsing)
    let symbols_output = Command::new(&parser_path)
        .arg("--write-symbols")
        .arg(&symbols_path)
        .output()
        .map_err(|e| format!("Failed to run parser for symbols: {}", e))?;

    if !symbols_output.status.success() {
        let stderr = String::from_utf8_lossy(&symbols_output.stderr);
        return Err(format!("Failed to get symbols: {}", stderr));
    }

    // Then run parser with trace enabled
    let output = Command::new(&parser_path)
        .arg(input_file.path())
        .arg("--start")
        .arg(&start_nonterminal)
        .arg("--trace")
        .arg(&trace_path)
        .arg("--format")
        .arg("json")
        .output()
        .map_err(|e| format!("Failed to run parser: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Parser failed: {}", stderr));
    }

    // Load trace replay
    let replay = TraceReplay::load(&trace_path, &symbols_path)
        .map_err(|e| format!("Failed to load trace: {}", e))?;

    let info = DebugInfo {
        current_step: replay.current_step() as u32,
        total_steps: replay.total_steps() as u32,
        current_action: replay.current_action_string(),
        descriptor_set: replay.descriptor_set_strings(),
        input_index: replay.current_input_index().map(|i| i as u32),
        total_errors: replay.total_errors() as u32,
        current_error_index: replay.current_error_index().map(|i| i as u32),
        input_path: Some(input_file.path().to_string_lossy().to_string()),
        symbols_path: Some(symbols_path.to_string_lossy().to_string()),
        trace_path: Some(trace_path.to_string_lossy().to_string()),
    };

    let mut debug_state = state.lock().unwrap();
    debug_state.replay = Some(replay);

    Ok(info)
}

#[tauri::command]
#[specta::specta]
fn debug_step_forward(state: tauri::State<Mutex<DebugState>>) -> Result<DebugInfo, String> {
    let mut debug_state = state.lock().unwrap();
    let replay = debug_state
        .replay
        .as_mut()
        .ok_or("No debug session. Load a trace first.")?;

    replay.step_forward();

    Ok(DebugInfo {
        current_step: replay.current_step() as u32,
        total_steps: replay.total_steps() as u32,
        current_action: replay.current_action_string(),
        descriptor_set: replay.descriptor_set_strings(),
        input_index: replay.current_input_index().map(|i| i as u32),
        total_errors: replay.total_errors() as u32,
        current_error_index: replay.current_error_index().map(|i| i as u32),
        input_path: None,
        symbols_path: None,
        trace_path: None,
    })
}

#[tauri::command]
#[specta::specta]
fn debug_step_to(target: u32, state: tauri::State<Mutex<DebugState>>) -> Result<DebugInfo, String> {
    let mut debug_state = state.lock().unwrap();
    let replay = debug_state
        .replay
        .as_mut()
        .ok_or("No debug session. Load a trace first.")?;

    replay.step_to(target as usize);

    Ok(DebugInfo {
        current_step: replay.current_step() as u32,
        total_steps: replay.total_steps() as u32,
        current_action: replay.current_action_string(),
        descriptor_set: replay.descriptor_set_strings(),
        input_index: replay.current_input_index().map(|i| i as u32),
        total_errors: replay.total_errors() as u32,
        current_error_index: replay.current_error_index().map(|i| i as u32),
        input_path: None,
        symbols_path: None,
        trace_path: None,
    })
}

#[tauri::command]
#[specta::specta]
fn get_debug_info(state: tauri::State<Mutex<DebugState>>) -> Result<DebugInfo, String> {
    let debug_state = state.lock().unwrap();
    let replay = debug_state
        .replay
        .as_ref()
        .ok_or("No debug session. Load a trace first.")?;

    Ok(DebugInfo {
        current_step: replay.current_step() as u32,
        total_steps: replay.total_steps() as u32,
        current_action: replay.current_action_string(),
        descriptor_set: replay.descriptor_set_strings(),
        input_index: replay.current_input_index().map(|i| i as u32),
        total_errors: replay.total_errors() as u32,
        current_error_index: replay.current_error_index().map(|i| i as u32),
        input_path: None,
        symbols_path: None,
        trace_path: None,
    })
}

#[tauri::command]
#[specta::specta]
fn get_stack_trace(state: tauri::State<Mutex<DebugState>>) -> Result<Vec<String>, String> {
    let debug_state = state.lock().unwrap();
    let replay = debug_state
        .replay
        .as_ref()
        .ok_or("No debug session. Load a trace first.")?;

    replay
        .build_stack_trace()
        .ok_or_else(|| "No stack trace available at current step.".to_string())
}

#[tauri::command]
#[specta::specta]
fn get_debug_sppf(state: tauri::State<Mutex<DebugState>>) -> Result<DebugSPPFInfo, String> {
    let debug_state = state.lock().unwrap();
    let replay = debug_state
        .replay
        .as_ref()
        .ok_or("No debug session. Load a trace first.")?;

    Ok(DebugSPPFInfo {
        nodes: replay.sppf_nodes().to_vec(),
        current_node_id: replay.current_sppf_node_id(),
    })
}

#[tauri::command]
#[specta::specta]
fn get_debug_gss(state: tauri::State<Mutex<DebugState>>) -> Result<DebugGSSInfo, String> {
    let debug_state = state.lock().unwrap();
    let replay = debug_state
        .replay
        .as_ref()
        .ok_or("No debug session. Load a trace first.")?;

    Ok(replay.get_debug_gss_info())
}

#[tauri::command]
#[specta::specta]
fn debug_go_to_furthest_error(
    state: tauri::State<Mutex<DebugState>>,
) -> Result<DebugInfo, String> {
    let mut debug_state = state.lock().unwrap();
    let replay = debug_state
        .replay
        .as_mut()
        .ok_or("No debug session. Load a trace first.")?;

    let target = replay
        .furthest_error_step()
        .ok_or("No errors in trace.")?;

    replay.step_to(target);

    Ok(DebugInfo {
        current_step: replay.current_step() as u32,
        total_steps: replay.total_steps() as u32,
        current_action: replay.current_action_string(),
        descriptor_set: replay.descriptor_set_strings(),
        input_index: replay.current_input_index().map(|i| i as u32),
        total_errors: replay.total_errors() as u32,
        current_error_index: replay.current_error_index().map(|i| i as u32),
        input_path: None,
        symbols_path: None,
        trace_path: None,
    })
}

#[tauri::command]
#[specta::specta]
fn get_debug_errors(state: tauri::State<Mutex<DebugState>>) -> Result<Vec<ErrorInfo>, String> {
    let debug_state = state.lock().unwrap();
    let replay = debug_state
        .replay
        .as_ref()
        .ok_or("No debug session. Load a trace first.")?;

    Ok(replay.get_errors_list())
}

#[tauri::command]
#[specta::specta]
fn debug_next_error(state: tauri::State<Mutex<DebugState>>) -> Result<DebugInfo, String> {
    let mut debug_state = state.lock().unwrap();
    let replay = debug_state
        .replay
        .as_mut()
        .ok_or("No debug session. Load a trace first.")?;

    let target = replay
        .next_error_step()
        .ok_or("No more errors after current step.")?;

    replay.step_to(target);

    Ok(DebugInfo {
        current_step: replay.current_step() as u32,
        total_steps: replay.total_steps() as u32,
        current_action: replay.current_action_string(),
        descriptor_set: replay.descriptor_set_strings(),
        input_index: replay.current_input_index().map(|i| i as u32),
        total_errors: replay.total_errors() as u32,
        current_error_index: replay.current_error_index().map(|i| i as u32),
        input_path: None,
        symbols_path: None,
        trace_path: None,
    })
}

#[tauri::command]
#[specta::specta]
fn get_event_log(state: tauri::State<Mutex<DebugState>>) -> Result<Vec<EventLogEntry>, String> {
    let debug_state = state.lock().unwrap();
    let replay = debug_state
        .replay
        .as_ref()
        .ok_or("No debug session. Load a trace first.")?;

    Ok(replay.build_event_log())
}

#[tauri::command]
#[specta::specta]
fn debug_prev_error(state: tauri::State<Mutex<DebugState>>) -> Result<DebugInfo, String> {
    let mut debug_state = state.lock().unwrap();
    let replay = debug_state
        .replay
        .as_mut()
        .ok_or("No debug session. Load a trace first.")?;

    let target = replay
        .prev_error_step()
        .ok_or("No errors before current step.")?;

    replay.step_to(target);

    Ok(DebugInfo {
        current_step: replay.current_step() as u32,
        total_steps: replay.total_steps() as u32,
        current_action: replay.current_action_string(),
        descriptor_set: replay.descriptor_set_strings(),
        input_index: replay.current_input_index().map(|i| i as u32),
        total_errors: replay.total_errors() as u32,
        current_error_index: replay.current_error_index().map(|i| i as u32),
        input_path: None,
        symbols_path: None,
        trace_path: None,
    })
}

#[tauri::command]
#[specta::specta]
fn get_log_path() -> Option<String> {
    log_file_path().map(|p| p.display().to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = Builder::<tauri::Wry>::new().commands(collect_commands![
        get_parser_name,
        load_grammar,
        save_grammar,
        build_parser,
        generate_parser,
        parse,
        get_stats,
        get_build_features,
        profile,
        get_sppf,
        get_gss,
        get_parse_tree,
        setup_vscode_debug,
        get_nonterminals,
        load_debug_trace,
        debug_step_forward,
        debug_step_to,
        get_debug_info,
        get_stack_trace,
        get_debug_sppf,
        get_debug_gss,
        debug_go_to_furthest_error,
        debug_next_error,
        debug_prev_error,
        get_debug_errors,
        get_event_log,
        analyze_grammar,
        format_grammar,
        get_semantic_tokens,
        get_semantic_tokens_legend,
        get_document_symbols,
        get_definition,
        get_references,
        get_folding_ranges,
        get_diagnostics,
        get_log_path
    ]);

    #[cfg(debug_assertions)]
    builder
        .export(
            specta_typescript::Typescript::default(),
            "../src/bindings.ts",
        )
        .expect("Failed to export typescript bindings");

    tauri::Builder::default()
        .manage(Mutex::new(ParseState::default()))
        .manage(Mutex::new(BuildState::default()))
        .manage(Mutex::new(DebugState::default()))
        .manage(Mutex::new(GrammarState {
            source: None,
            input: None,
            grammar_def: None,
            parse_ms: 0,
            tree_ms: 0,
            success: false,
        }))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(builder.invoke_handler())
        .setup(move |app| {
            builder.mount_events(app);

            if let Ok(log_dir) = app.path().app_log_dir() {
                // Truncate the log file on startup so each session starts fresh
                let _ = fs::create_dir_all(&log_dir);
                let _ = fs::write(log_dir.join("terrarium.log"), b"");
                let _ = LOG_DIR.set(log_dir);
            }

            panic::set_hook(Box::new(|info| {
                let backtrace = std::backtrace::Backtrace::force_capture();
                let message = info
                    .payload()
                    .downcast_ref::<String>()
                    .map(|s| s.as_str())
                    .or_else(|| info.payload().downcast_ref::<&str>().copied())
                    .unwrap_or("unknown error");
                let location = info
                    .location()
                    .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
                    .unwrap_or_default();
                let formatted = format!("at {location}: {message}\n{backtrace}");
                if CAPTURING.get() {
                    // Inside catch_unwind — stash for the caller to route through log_event
                    LAST_PANIC.set(Some(formatted));
                } else {
                    // Uncaught panic — write to file directly as a safety net
                    log_to_file("panic", &formatted);
                }
            }));

            if std::env::var("TERRARIUM_SMOKE_TEST").is_ok() {
                println!("TERRARIUM_READY");
                std::process::exit(0);
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
