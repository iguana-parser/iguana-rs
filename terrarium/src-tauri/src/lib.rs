use std::{fs, io::Write, path::Path, process::Command, thread};

use iguana::visualization::sppf::SPPF;
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::Emitter;
use tauri_specta::{collect_commands, Builder};
use tempfile::NamedTempFile;
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

#[tauri::command]
#[specta::specta]
fn get_parser_name(directory: String) -> Result<String, String> {
    let cargo_path = Path::new(&directory).join("Cargo.toml");
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
fn parse(directory: String, input: String) -> Result<SPPF, String> {
    // Get parser name from Cargo.toml
    let parser_name = get_parser_name(directory.clone())?;

    // Write input to temp file
    let mut temp_file = NamedTempFile::new()
        .map_err(|e| format!("Failed to create temp file: {}", e))?;
    temp_file
        .write_all(input.as_bytes())
        .map_err(|e| format!("Failed to write input: {}", e))?;

    // Build path to parser binary
    let parser_path = Path::new(&directory)
        .join("target")
        .join("debug")
        .join(&parser_name);

    // Run parser with --emit sppf
    let output = Command::new(&parser_path)
        .arg(temp_file.path())
        .args(["--emit", "sppf"])
        .output()
        .map_err(|e| format!("Failed to run parser: {}", e))?;

    if output.status.success() {
        let json = String::from_utf8(output.stdout)
            .map_err(|e| format!("Invalid UTF-8 in output: {}", e))?;
        serde_json::from_str(&json)
            .map_err(|e| format!("Failed to parse SPPF JSON: {}", e))
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("Parse failed: {}", stderr))
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = Builder::<tauri::Wry>::new()
        .commands(collect_commands![get_parser_name, build_parser, parse]);

    #[cfg(debug_assertions)]
    builder
        .export(
            specta_typescript::Typescript::default(),
            "../src/bindings.ts",
        )
        .expect("Failed to export typescript bindings");

    tauri::Builder::default()
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
