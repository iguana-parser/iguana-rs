use std::{fs, io::Write, path::Path, path::PathBuf, process::Command, sync::Mutex, thread};

use iguana::visualization::{gss::GSS, sppf::SPPF};
use serde::Serialize;
use specta::Type;
use tauri::Emitter;
use tauri_specta::{collect_commands, Builder};
use tempfile::{NamedTempFile, TempDir};
use toml::Value;

#[derive(Clone, Serialize, Type)]
struct BuildProgress {
    stage: String,
    message: String,
}

#[derive(Clone, Serialize, Type)]
struct BuildResult {
    success: bool,
    message: String,
}

#[derive(Default)]
struct ParseState {
    _temp_dir: Option<TempDir>,
    sppf_path: Option<PathBuf>,
    gss_path: Option<PathBuf>,
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
    let parser_path = Path::new(directory)
        .join("target")
        .join("debug")
        .join(&parser_name);

    if !parser_path.exists() {
        return Err(format!(
            "Parser not found at {:?}. Please build first.",
            parser_path
        ));
    }

    Ok(parser_path)
}

#[tauri::command]
#[specta::specta]
fn get_parser_name(directory: String) -> Result<String, String> {
    read_parser_name(&directory)
}

#[tauri::command]
#[specta::specta]
fn build_parser(directory: String, app: tauri::AppHandle) {
    // Get the workspace root before spawning thread
    let workspace_root = match std::env::current_dir() {
        Ok(dir) => match dir.parent() {
            Some(parent) => parent.to_path_buf(),
            None => {
                let _ = app.emit(
                    "build-result",
                    BuildResult {
                        success: false,
                        message: "Failed to find workspace root".into(),
                    },
                );
                return;
            }
        },
        Err(e) => {
            let _ = app.emit(
                "build-result",
                BuildResult {
                    success: false,
                    message: format!("Failed to get current directory: {}", e),
                },
            );
            return;
        }
    };

    // Spawn blocking work in a separate thread
    thread::spawn(move || {
        // Step 1: Generate the parser
        let _ = app.emit(
            "build-progress",
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
            ])
            .current_dir(&workspace_root)
            .output();

        match generate_output {
            Ok(output) if output.status.success() => {}
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                let _ = app.emit(
                    "build-result",
                    BuildResult {
                        success: false,
                        message: format!("Generation failed:\n{}", stderr),
                    },
                );
                return;
            }
            Err(e) => {
                let _ = app.emit(
                    "build-result",
                    BuildResult {
                        success: false,
                        message: format!("Failed to run generator: {}", e),
                    },
                );
                return;
            }
        }

        // Step 2: Build the generated parser
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
                    },
                );
            }
            Err(e) => {
                let _ = app.emit(
                    "build-result",
                    BuildResult {
                        success: false,
                        message: format!("Failed to run cargo build: {}", e),
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
    state: tauri::State<Mutex<ParseState>>,
) -> Result<(), String> {
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

    let output = Command::new(&parser_path)
        .arg(input_file.path())
        .arg("--write-sppf")
        .arg(&sppf_path)
        .arg("--write-gss")
        .arg(&gss_path)
        .output()
        .map_err(|e| format!("Failed to run parser: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if stdout.trim() == "Parse failed" {
        return Err("Parse error".to_string());
    }

    if !output.status.success() {
        return Err(format!("Parser exited with error: {}", stderr));
    }

    let mut parse_state = state.lock().unwrap();
    parse_state._temp_dir = Some(temp_dir);
    parse_state.sppf_path = Some(sppf_path);
    parse_state.gss_path = Some(gss_path);

    Ok(())
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = Builder::<tauri::Wry>::new().commands(collect_commands![
        get_parser_name,
        build_parser,
        parse,
        get_sppf,
        get_gss,
        get_nonterminals
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
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(builder.invoke_handler())
        .setup(move |app| {
            builder.mount_events(app);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
