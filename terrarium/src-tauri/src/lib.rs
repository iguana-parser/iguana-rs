mod trace_replay;

use std::{fs, io::Write, path::Path, path::PathBuf, process::Command, sync::Mutex, thread};

use iguana::visualization::{gss::GSS, sppf::SPPF};
use serde::Serialize;
use specta::Type;
use tauri::Emitter;
use tauri_specta::{collect_commands, Builder};
use tempfile::{NamedTempFile, TempDir};
use toml::Value;

use trace_replay::{DebugGSSInfo, DebugSPPFNode, ErrorInfo, EventLogEntry, TraceReplay};

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

/// Debug SPPF info returned to the frontend.
#[derive(Clone, Serialize, Type)]
struct DebugSPPFInfo {
    nodes: Vec<DebugSPPFNode>,
    /// The current SPPF node ID from the descriptor being processed
    current_node_id: Option<u32>,
}

#[derive(Clone, Serialize, Type)]
struct BuildProgress {
    stage: String,
    message: String,
}

#[derive(Clone, Serialize, Type)]
struct BuildResult {
    success: bool,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    duration_ms: Option<u64>,
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

/// Cached grammar analysis state.
/// Stores the parse result and a fallback token cache for when parsing fails.
struct GrammarState {
    parse_result: Option<iggy_ls::ParseResult>,
    cached_tokens: Vec<SemanticTokenData>,
}

#[derive(Default)]
struct ParseState {
    _temp_dir: Option<TempDir>,
    sppf_path: Option<PathBuf>,
    gss_path: Option<PathBuf>,
    parse_tree_path: Option<PathBuf>,
}

/// Result of a parse operation, indicating which outputs are available.
#[derive(Clone, Serialize, Type)]
struct ParseOutput {
    success: bool,
    error: Option<String>,
    duration_ms: Option<u32>,
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
    /// Total number of error steps (MatchFailed events)
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

fn get_parser_binary_path(directory: &str) -> Result<PathBuf, String> {
    let parser_name = read_parser_name(directory)?;
    let dir_path = Path::new(directory);

    // First, check local target directory (standalone project)
    let local_path = dir_path.join("target").join("debug").join(&parser_name);
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
                    let workspace_path = parent.join("target").join("debug").join(&parser_name);
                    if workspace_path.exists() {
                        return Ok(workspace_path);
                    }
                }
            }
        }
        current = parent.to_path_buf();
    }

    Err(format!(
        "Parser binary '{}' not found. Please build first.",
        parser_name
    ))
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
fn build_parser(directory: String, app: tauri::AppHandle) {
    // Spawn blocking work in a separate thread
    thread::spawn(move || {
        let _ = app.emit(
            "build-progress",
            BuildProgress {
                stage: "compile".into(),
                message: "Compiling parser...".into(),
            },
        );

        let build_output = Command::new("cargo")
            .args(["build", "--features", "debug-trace"])
            .current_dir(&directory)
            .output();

        match build_output {
            Ok(output) if output.status.success() => {
                let _ = app.emit(
                    "build-result",
                    BuildResult {
                        success: true,
                        message: "Build successful".into(),
                        duration_ms: None,
                    },
                );
            }
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                let _ = app.emit(
                    "build-result",
                    BuildResult {
                        success: false,
                        message: stderr.into_owned(),
                        duration_ms: None,
                    },
                );
            }
            Err(e) => {
                let _ = app.emit(
                    "build-result",
                    BuildResult {
                        success: false,
                        message: format!("Failed to run cargo build: {}", e),
                        duration_ms: None,
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
        .output()
        .map_err(|e| format!("Failed to run parser: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
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

    // Parse duration from stdout (format: "Parse success in <ms>ms")
    let duration_ms = stdout
        .lines()
        .find_map(|line| {
            line.strip_prefix("Parse success in ")
                .and_then(|rest| rest.strip_suffix("ms"))
                .and_then(|ms| ms.parse::<u32>().ok())
        });

    // Determine success/error status
    if stdout.lines().any(|line| line.trim() == "Parse failed") {
        return Ok(ParseOutput {
            success: false,
            error: Some("Parse error".to_string()),
            duration_ms: None,
            has_sppf,
            has_gss,
            has_parse_tree,
        });
    }

    if !output.status.success() {
        return Ok(ParseOutput {
            success: false,
            error: Some(format!("Parser error: {}", stderr.trim())),
            duration_ms: None,
            has_sppf,
            has_gss,
            has_parse_tree,
        });
    }

    Ok(ParseOutput {
        success: true,
        error: None,
        duration_ms,
        has_sppf,
        has_gss,
        has_parse_tree,
    })
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
fn generate_parser(directory: String, app: tauri::AppHandle) {
    // Get the workspace root (parent of terrarium directory)
    let workspace_root = match std::env::current_dir() {
        Ok(dir) => match dir.parent() {
            Some(parent) => parent.to_path_buf(),
            None => {
                let _ = app.emit(
                    "generate-result",
                    BuildResult {
                        success: false,
                        message: "Failed to find workspace root".into(),
                        duration_ms: None,
                    },
                );
                return;
            }
        },
        Err(e) => {
            let _ = app.emit(
                "generate-result",
                BuildResult {
                    success: false,
                    message: format!("Failed to get current directory: {}", e),
                    duration_ms: None,
                },
            );
            return;
        }
    };

    // Spawn blocking work in a separate thread
    thread::spawn(move || {
        let _ = app.emit(
            "generate-progress",
            BuildProgress {
                stage: "generate".into(),
                message: "Generating parser...".into(),
            },
        );

        let generate_output = Command::new("cargo")
            .args([
                "run",
                "--package",
                "iguana",
                "--",
                "generate",
                "--output",
                &directory,
                "--json",
            ])
            .current_dir(&workspace_root)
            .output();

        match generate_output {
            Ok(output) if output.status.success() => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let duration_ms = serde_json::from_str::<serde_json::Value>(stdout.trim())
                    .ok()
                    .and_then(|v| v["total_duration_ms"].as_u64());
                let _ = app.emit(
                    "generate-result",
                    BuildResult {
                        success: true,
                        message: "Generation successful".into(),
                        duration_ms,
                    },
                );
            }
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                let _ = app.emit(
                    "generate-result",
                    BuildResult {
                        success: false,
                        message: format!("Generation failed:\n{}", stderr),
                        duration_ms: None,
                    },
                );
            }
            Err(e) => {
                let _ = app.emit(
                    "generate-result",
                    BuildResult {
                        success: false,
                        message: format!("Failed to run generator: {}", e),
                        duration_ms: None,
                    },
                );
            }
        }
    });
}

// ============ Language Intelligence Commands ============

/// Parse the grammar source and cache the result. All other language intelligence
/// commands (semantic tokens, diagnostics, etc.) read from this cache.
#[tauri::command]
#[specta::specta]
fn analyze_grammar(source: String, state: tauri::State<Mutex<GrammarState>>) -> AnalyzeResult {
    let result = iggy_ls::parse(&source);
    let parse_duration_ms = result.parse_duration.as_millis() as u32;
    let tree_construction_duration_ms = result.tree_construction_duration.as_millis() as u32;
    let success = result.tree.is_some();

    let mut st = state.lock().unwrap();
    st.parse_result = Some(result);

    AnalyzeResult {
        success,
        parse_duration_ms,
        tree_construction_duration_ms,
    }
}

/// Return semantic tokens from the cached parse result.
#[tauri::command]
#[specta::specta]
fn get_semantic_tokens(state: tauri::State<Mutex<GrammarState>>) -> Vec<SemanticTokenData> {
    let mut st = state.lock().unwrap();

    let tokens: Vec<SemanticTokenData> = st
        .parse_result
        .as_ref()
        .map(|r| {
            iggy_ls::semantic_tokens::semantic_tokens(r)
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
        .unwrap_or_default();

    if !tokens.is_empty() {
        st.cached_tokens = tokens.clone();
        tokens
    } else {
        // Fallback: return cached tokens so highlighting stays stable while typing
        st.cached_tokens.clone()
    }
}

#[tauri::command]
#[specta::specta]
fn get_semantic_tokens_legend() -> SemanticTokensLegendData {
    let legend = iggy_ls::semantic_tokens::legend();
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = Builder::<tauri::Wry>::new().commands(collect_commands![
        get_parser_name,
        load_grammar,
        save_grammar,
        build_parser,
        generate_parser,
        parse,
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
        get_semantic_tokens,
        get_semantic_tokens_legend
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
        .manage(Mutex::new(DebugState::default()))
        .manage(Mutex::new(GrammarState {
            parse_result: None,
            cached_tokens: Vec::new(),
        }))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(builder.invoke_handler())
        .setup(move |app| {
            builder.mount_events(app);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
