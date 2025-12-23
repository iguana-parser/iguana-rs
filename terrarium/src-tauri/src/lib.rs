use std::{fs, path::Path, process::Command, thread};

use serde::Serialize;
use tauri::Emitter;
use toml::Value;

#[derive(Clone, Serialize)]
struct BuildProgress {
    stage: String,
    message: String,
}

#[derive(Clone, Serialize)]
struct BuildResult {
    success: bool,
    message: String,
}

#[tauri::command]
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![get_parser_name, build_parser])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
