mod trace_replay;

use std::{
    fs,
    fs::OpenOptions,
    io::{self, Write},
    panic,
    path::{Path, PathBuf},
    process::Command,
    sync::{Mutex, MutexGuard, OnceLock},
    thread,
};

use iguana_runtime::arena::Arena;
use iguana_runtime::cli;
use iguana_runtime::input::Input;
use iguana_runtime::visualization::{gss::GSS, sppf::SPPF};
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
#[derive(Clone, Default, Serialize, Type)]
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
    grammar_def: Option<iguana_lsp::GrammarDef>,
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
        let tree_arena = Arena::new();
        let result = iguana_lsp::build(&input, &tree_arena);
        match &result {
            iguana_lsp::BuildResult::Success {
                tree,
                parse_duration,
                tree_construction_duration,
            } => {
                self.grammar_def = Some(iguana_lsp::build_grammar_def(tree, &input));
                self.parse_ms = parse_duration.as_millis() as u32;
                self.tree_ms = tree_construction_duration.as_millis() as u32;
                self.success = true;
            }
            iguana_lsp::BuildResult::Error(_) | iguana_lsp::BuildResult::Ambiguous => {
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
    fn parse<'a>(&self, tree_arena: &'a Arena) -> Option<iguana_lsp::BuildResult<'a>> {
        let input = self.input.as_ref()?;
        match iguana_lsp::build(input, tree_arena) {
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
}

/// Returned by `parse()`: the runtime's parse report plus what only the host
/// knows. `result` is the parser's --write-result report with `parse_tree`
/// filled in from the tree file, read inline so a parse is a single command.
/// It is `None` when no valid report was available (a panic, a signal, or an
/// exit, clean or not, without a readable report), and `unexpected_error`
/// then holds the detail. A parse failure is not that case: the parser also
/// exits non-zero, but it writes a report whose `error` is set.
#[derive(Clone, Serialize, Type)]
struct ParseRun {
    result: Option<cli::ParseOutput>,
    unexpected_error: Option<String>,
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
    /// Total number of error steps
    total_errors: u32,
    /// Current error index (1-indexed) if at an error step, None otherwise
    current_error_index: Option<u32>,
    /// Only set on initial load
    input_path: Option<String>,
    symbols_path: Option<String>,
    trace_path: Option<String>,
}

/// The parser binary's target name from `Cargo.toml`: an explicit `[[bin]]` name
/// (set via `iguana generate --bin-name`) when present, otherwise the package
/// name. The binary on disk is named after the bin target, so this is what the
/// build output and the debug config must point at.
fn read_parser_name(directory: &str) -> Result<String, String> {
    let cargo_path = Path::new(directory).join("Cargo.toml");
    let content = fs::read_to_string(&cargo_path)
        .map_err(|_| "No valid Iguana parser found in this directory.".to_string())?;

    let toml: Value = toml::from_str(&content)
        .map_err(|_| "No valid Iguana parser found in this directory.".to_string())?;

    let bin_name = toml
        .get("bin")
        .and_then(|b| b.as_array())
        .and_then(|bins| bins.first())
        .and_then(|bin| bin.get("name"))
        .and_then(|n| n.as_str());
    let package_name = toml
        .get("package")
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str());
    bin_name
        .or(package_name)
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
                    let workspace_path = parent.join("target").join(profile).join(&parser_name);
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
    find_parser_binary(directory, "release").or_else(|_| find_parser_binary(directory, "debug"))
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
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "iggy"))
        .collect();

    match iggy_files.len() {
        0 => Err("No .iggy file found in this directory.".to_string()),
        1 => {
            let path = iggy_files[0].path();
            let content =
                fs::read_to_string(&path).map_err(|e| format!("Cannot read grammar file: {e}"))?;
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
    let built_features = BuildFeatures {
        instrument,
        debug_trace,
    };

    // Mark current binary as stale until rebuild completes.
    lock_state(&state).features = None;

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
                    lock_state(&state).features = Some(built_features);
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
) -> Result<ParseRun, String> {
    // An Err here means the parser could not be run at all (missing binary, spawn
    // failure): an unexpected error like a crash. Log it to the file so a saved
    // log carries it; the frontend surfaces it through the same banner.
    parse_inner(directory, input, start_nonterminal, state).inspect_err(|e| {
        log_to_file("error", &format!("Parser could not run: {e}"));
    })
}

fn parse_inner(
    directory: String,
    input: String,
    start_nonterminal: String,
    state: tauri::State<Mutex<ParseState>>,
) -> Result<ParseRun, String> {
    let parser_path = get_parser_binary_path(&directory)?;

    let mut input_file =
        NamedTempFile::new().map_err(|e| format!("Failed to create temp file: {}", e))?;
    input_file
        .write_all(input.as_bytes())
        .map_err(|e| format!("Failed to write input: {}", e))?;

    let temp_dir = TempDir::new().map_err(|e| format!("Failed to create temp directory: {}", e))?;
    let sppf_path = temp_dir.path().join("sppf.json");
    let gss_path = temp_dir.path().join("gss.json");
    let parse_tree_path = temp_dir.path().join("parse_tree.json");
    let result_path = temp_dir.path().join("result.json");

    let output = Command::new(&parser_path)
        .env("RUST_BACKTRACE", "1") // Always show backtraces for debugging
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
    // Read the parse tree inline so a parse is a single command.
    let parse_tree = parse_tree_path
        .exists()
        .then(|| fs::read_to_string(&parse_tree_path).ok())
        .flatten();

    // Store paths for files that exist
    let mut parse_state = lock_state(&state);
    parse_state._temp_dir = Some(temp_dir);
    parse_state.sppf_path = if has_sppf { Some(sppf_path) } else { None };
    parse_state.gss_path = if has_gss { Some(gss_path) } else { None };

    // Read parse result from JSON file written by --write-result
    let result = result_path
        .exists()
        .then(|| {
            fs::read_to_string(&result_path)
                .ok()
                .and_then(|s| serde_json::from_str::<cli::ParseOutput>(&s).ok())
        })
        .flatten();

    // Parser did not run to a result (panic, signal, non-zero exit): an
    // unexpected error, not a parse failure. Record the detail in the log file so
    // a saved log carries it; the frontend also shows it in the output panel.
    if !output.status.success() && result.is_none() {
        let detail = format!(
            "Parser exited unexpectedly ({}).\n{}",
            output.status,
            stderr.trim()
        );
        log_to_file("error", &detail);
        return Ok(ParseRun {
            result: None,
            unexpected_error: Some(detail),
        });
    }

    match result {
        Some(mut result) => {
            result.parse_tree = parse_tree;
            Ok(ParseRun {
                result: Some(result),
                unexpected_error: None,
            })
        }
        // The parser exited cleanly but wrote no result file: an unexpected error,
        // not a parse failure. Should not happen with the current generator.
        None => {
            let detail = "Parser produced no result.".to_string();
            log_to_file("error", &detail);
            Ok(ParseRun {
                result: None,
                unexpected_error: Some(detail),
            })
        }
    }
}

#[tauri::command]
#[specta::specta]
fn get_sppf(state: tauri::State<Mutex<ParseState>>) -> Result<SPPF, String> {
    let parse_state = lock_state(&state);
    let sppf_path = parse_state
        .sppf_path
        .as_ref()
        .ok_or("No parse result available. Run parse first.")?;

    let content =
        fs::read_to_string(sppf_path).map_err(|e| format!("Failed to read SPPF file: {}", e))?;

    serde_json::from_str(&content).map_err(|e| format!("Failed to parse SPPF JSON: {}", e))
}

#[tauri::command]
#[specta::specta]
fn get_gss(state: tauri::State<Mutex<ParseState>>) -> Result<GSS, String> {
    let parse_state = lock_state(&state);
    let gss_path = parse_state
        .gss_path
        .as_ref()
        .ok_or("No parse result available. Run parse first.")?;

    let content =
        fs::read_to_string(gss_path).map_err(|e| format!("Failed to read GSS file: {}", e))?;

    serde_json::from_str(&content).map_err(|e| format!("Failed to parse GSS JSON: {}", e))
}

/// Returns the cargo features the current parser binary was built with,
/// or `None` if no successful build has happened yet.
#[tauri::command]
#[specta::specta]
fn get_build_features(state: tauri::State<Mutex<BuildState>>) -> Option<BuildFeatures> {
    lock_state(&state).features
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

    let mut input_file =
        NamedTempFile::new().map_err(|e| format!("Failed to create temp file: {}", e))?;
    input_file
        .write_all(input.as_bytes())
        .map_err(|e| format!("Failed to write input: {}", e))?;

    let temp_dir = TempDir::new().map_err(|e| format!("Failed to create temp directory: {}", e))?;
    let stats_path = temp_dir.path().join("stats.json");

    // --quiet because none of the write flags here suppress the printed tree,
    // and rendering a whole parse tree to a stdout we discard is not free.
    let output = Command::new(&parser_path)
        .arg(input_file.path())
        .arg("--start")
        .arg(&start_nonterminal)
        .arg("--write-stats")
        .arg(&stats_path)
        .arg("--quiet")
        .output()
        .map_err(|e| format!("Failed to run parser: {}", e))?;

    if !stats_path.exists() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "Parser did not write stats. Was it built with --features instrument? {}",
            stderr.trim()
        ));
    }

    let content =
        fs::read_to_string(&stats_path).map_err(|e| format!("Failed to read stats file: {}", e))?;
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

    // The binary name (honors an explicit [[bin]]), so the debug config points
    // at the binary the build actually produces.
    let parser_name = read_parser_name(&directory)?;

    // Create .vscode directory if it doesn't exist
    let vscode_dir = dir_path.join(".vscode");
    fs::create_dir_all(&vscode_dir)
        .map_err(|e| format!("Failed to create .vscode directory: {}", e))?;

    // Write input to .vscode/debug-input.txt (keeps parser directory clean)
    let input_path = vscode_dir.join("debug-input.txt");
    fs::write(&input_path, &input).map_err(|e| format!("Failed to write debug input: {}", e))?;

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
        let ll1_flag = if no_ll1 { " --ll1 false" } else { "" };
        log_event(
            &app,
            "command",
            &format!("iguana generate --output {directory} --json{ll1_flag}"),
        );

        let mut cmd = Command::new("iguana");
        cmd.arg("generate")
            .arg("--output")
            .arg(&directory)
            .arg("--json");
        // The flag is only passed when the option is off, so a gen.toml beside
        // the grammar still decides when the checkbox is on.
        if no_ll1 {
            cmd.args(["--ll1", "false"]);
        }

        let output = match cmd.output() {
            Ok(output) => output,
            Err(e) => {
                let message = if e.kind() == io::ErrorKind::NotFound {
                    "iguana binary not found on PATH. Install with `cargo install iguana` \
                     or `brew install iguana`."
                        .to_string()
                } else {
                    format!("Failed to run iguana: {e}")
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
                return;
            }
        };

        if output.status.success() {
            let duration_ms = parse_total_duration_ms(&output.stdout);
            let suffix = duration_ms.map(|d| format!(" ({d}ms)")).unwrap_or_default();
            log_event(
                &app,
                "output",
                &format!("Parser generated successfully{suffix}"),
            );
            let _ = app.emit(
                "generate-result",
                BuildResult {
                    success: true,
                    message: "Generation successful".into(),
                    duration_ms,
                    features: None,
                },
            );
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let message = stderr.trim();
            let message = if message.is_empty() {
                format!("iguana exited with status {}", output.status)
            } else {
                message.to_string()
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
        }
    });
}

/// Probe whether the `iguana` binary is reachable on PATH. Returns `Ok(())`
/// on success and an actionable install hint when missing.
#[tauri::command]
#[specta::specta]
fn check_iguana() -> Result<(), String> {
    match Command::new("iguana").arg("help").output() {
        Ok(_) => Ok(()),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Err(
            "iguana CLI not found on PATH. Install with `cargo install iguana` or `brew install iguana`."
                .into(),
        ),
        Err(e) => Err(format!("Failed to run iguana: {e}")),
    }
}

fn parse_total_duration_ms(stdout: &[u8]) -> Option<u64> {
    let s = std::str::from_utf8(stdout).ok()?;
    let value: serde_json::Value = serde_json::from_str(s.trim()).ok()?;
    value.get("total_duration_ms")?.as_u64()
}

// ============ Language Intelligence Commands ============

/// Run an LSP query behind a panic boundary. Tauri command handlers run on
/// worker threads with no unwind boundary above them, so an uncaught panic in
/// grammar analysis would cross the FFI edge and abort the whole app. Catching
/// it degrades the query to an empty result instead.
fn lsp_guard<T: Default>(f: impl FnOnce() -> T) -> T {
    panic::catch_unwind(panic::AssertUnwindSafe(f)).unwrap_or_default()
}

/// Lock one of the managed states, recovering from a poisoned mutex. Every one
/// of them holds a cache the next user action refills (a parse result, a set of
/// output paths, the build features, a trace replay), so a panic that poisoned
/// one left nothing worth keeping. Propagating the poison instead would turn a
/// single panic into an abort on every later command.
fn lock_state<T>(state: &Mutex<T>) -> MutexGuard<'_, T> {
    state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Parse the grammar source and cache the result. Re-parses only if the source
/// text has changed since the last parse.
#[tauri::command]
#[specta::specta]
fn analyze_grammar(source: String, state: tauri::State<Mutex<GrammarState>>) -> AnalyzeResult {
    lsp_guard(|| {
        let mut st = lock_state(&state);
        let (parse_duration_ms, tree_construction_duration_ms) = st.ensure_built(&source);

        AnalyzeResult {
            success: st.success,
            parse_duration_ms,
            tree_construction_duration_ms,
        }
    })
}

/// Format the grammar using the cached parse result.
/// Falls back to parsing the source if no cached result exists.
/// Returns None if parsing fails (grammar cannot be formatted).
#[tauri::command]
#[specta::specta]
fn format_grammar(source: String, state: tauri::State<Mutex<GrammarState>>) -> Option<String> {
    lsp_guard(|| {
        let mut st = lock_state(&state);
        st.ensure_built(&source);
        let tree_arena = Arena::new();
        let iguana_lsp::BuildResult::Success { tree, .. } = st.parse(&tree_arena)? else {
            return None;
        };
        let input = st.input.as_ref()?;
        Some(iguana_lsp::format::format(tree, input))
    })
}

/// Return semantic tokens from the cached parse result.
#[tauri::command]
#[specta::specta]
fn get_semantic_tokens(state: tauri::State<Mutex<GrammarState>>) -> Vec<SemanticTokenData> {
    lsp_guard(|| {
        let st = lock_state(&state);
        let tree_arena = Arena::new();
        let Some(iguana_lsp::BuildResult::Success { tree, .. }) = st.parse(&tree_arena) else {
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
    })
}

/// Return document symbols (rule heads + alternative labels).
/// Ensures the parse result is fresh for the given source.
#[tauri::command]
#[specta::specta]
fn get_document_symbols(
    source: String,
    state: tauri::State<Mutex<GrammarState>>,
) -> Vec<DocumentSymbolData> {
    lsp_guard(|| {
        let mut st = lock_state(&state);
        st.ensure_built(&source);
        let tree_arena = Arena::new();
        let Some(iguana_lsp::BuildResult::Success { tree, .. }) = st.parse(&tree_arena) else {
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
    })
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
    lsp_guard(|| {
        let mut st = lock_state(&state);
        st.ensure_built(&source);
        let tree_arena = Arena::new();
        let iguana_lsp::BuildResult::Success { tree, .. } = st.parse(&tree_arena)? else {
            return None;
        };
        let input = st.input.as_ref()?;
        let grammar_def = st.grammar_def.as_ref()?;
        let spans = iguana_lsp::build_spans(grammar_def, tree, input);
        let uri: lsp_types::Uri = "file:///terrarium".parse().unwrap();
        let offset = input.offset(line, column);
        let loc = iguana_lsp::references::definition(grammar_def, &spans, input, &uri, offset)?;
        Some(LocationData {
            range: RangeData {
                start_line: loc.range.start.line,
                start_char: loc.range.start.character,
                end_line: loc.range.end.line,
                end_char: loc.range.end.character,
            },
        })
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
    lsp_guard(|| {
        let mut st = lock_state(&state);
        st.ensure_built(&source);
        let tree_arena = Arena::new();
        let Some(iguana_lsp::BuildResult::Success { tree, .. }) = st.parse(&tree_arena) else {
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
        iguana_lsp::references::references(
            grammar_def,
            &spans,
            input,
            &uri,
            offset,
            include_declaration,
        )
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
    })
}

/// Return folding ranges for the grammar.
#[tauri::command]
#[specta::specta]
fn get_folding_ranges(
    source: String,
    state: tauri::State<Mutex<GrammarState>>,
) -> Vec<FoldingRangeData> {
    lsp_guard(|| {
        let mut st = lock_state(&state);
        st.ensure_built(&source);
        let tree_arena = Arena::new();
        let Some(iguana_lsp::BuildResult::Success { tree, .. }) = st.parse(&tree_arena) else {
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
    })
}

/// Return diagnostics (e.g. unresolved references) for the grammar.
#[tauri::command]
#[specta::specta]
fn get_diagnostics(
    source: String,
    state: tauri::State<Mutex<GrammarState>>,
) -> Vec<DiagnosticData> {
    lsp_guard(|| {
        let mut st = lock_state(&state);
        st.ensure_built(&source);
        let Some(ref input) = st.input else {
            return vec![];
        };
        let tree_arena = Arena::new();
        let diagnostics = match iguana_lsp::build(input, &tree_arena) {
            iguana_lsp::BuildResult::Success { tree, .. } => {
                let Some(ref grammar_def) = st.grammar_def else {
                    return vec![];
                };
                let spans = iguana_lsp::build_spans(grammar_def, tree, input);
                iguana_lsp::diagnostics::diagnostics(grammar_def, &spans, input)
            }
            iguana_lsp::BuildResult::Ambiguous => vec![],
            iguana_lsp::BuildResult::Error(error) => {
                vec![iguana_lsp::diagnostics::to_diagnostic(error, input)]
            }
        };
        diagnostics
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
    })
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
        token_types: legend
            .token_types
            .iter()
            .map(|t| t.as_str().to_string())
            .collect(),
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
    let temp_dir = TempDir::new().map_err(|e| format!("Failed to create temp directory: {}", e))?;
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

    // A failing parse is the one most worth stepping through, and the parser
    // writes its trace before exiting non-zero, so the exit status is not a
    // reason to give up. Only a missing trace file is: the binary was built
    // without `debug-trace`, or it died before writing.
    if !trace_path.exists() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Parser wrote no trace: {}", stderr.trim()));
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

    let mut debug_state = lock_state(&state);
    debug_state.replay = Some(replay);

    Ok(info)
}

#[tauri::command]
#[specta::specta]
fn debug_step_forward(state: tauri::State<Mutex<DebugState>>) -> Result<DebugInfo, String> {
    let mut debug_state = lock_state(&state);
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
    let mut debug_state = lock_state(&state);
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
    let debug_state = lock_state(&state);
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
    let debug_state = lock_state(&state);
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
    let debug_state = lock_state(&state);
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
    let debug_state = lock_state(&state);
    let replay = debug_state
        .replay
        .as_ref()
        .ok_or("No debug session. Load a trace first.")?;

    Ok(replay.get_debug_gss_info())
}

#[tauri::command]
#[specta::specta]
fn debug_go_to_furthest_error(state: tauri::State<Mutex<DebugState>>) -> Result<DebugInfo, String> {
    let mut debug_state = lock_state(&state);
    let replay = debug_state
        .replay
        .as_mut()
        .ok_or("No debug session. Load a trace first.")?;

    let target = replay.furthest_error_step().ok_or("No errors in trace.")?;

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
    let debug_state = lock_state(&state);
    let replay = debug_state
        .replay
        .as_ref()
        .ok_or("No debug session. Load a trace first.")?;

    Ok(replay.get_errors_list())
}

#[tauri::command]
#[specta::specta]
fn debug_next_error(state: tauri::State<Mutex<DebugState>>) -> Result<DebugInfo, String> {
    let mut debug_state = lock_state(&state);
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
    let debug_state = lock_state(&state);
    let replay = debug_state
        .replay
        .as_ref()
        .ok_or("No debug session. Load a trace first.")?;

    Ok(replay.build_event_log())
}

#[tauri::command]
#[specta::specta]
fn debug_prev_error(state: tauri::State<Mutex<DebugState>>) -> Result<DebugInfo, String> {
    let mut debug_state = lock_state(&state);
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

/// Copies the session log file to a user-chosen path. The frontend supplies the
/// destination from a native save dialog.
#[tauri::command]
#[specta::specta]
fn save_log(destination: String) -> Result<(), String> {
    let src = log_file_path().ok_or("Log file path is not available")?;
    fs::copy(&src, &destination)
        .map(|_| ())
        .map_err(|e| format!("Failed to save log: {e}"))
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
        get_log_path,
        save_log,
        check_iguana
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
                log_to_file("panic", &formatted);
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
