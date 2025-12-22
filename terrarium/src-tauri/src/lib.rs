use std::{fs, process::Command};

use toml::Value;

#[tauri::command]
fn get_parser_name() -> Result<String, String> {
    let content = fs::read_to_string("Cargo.toml")
        .map_err(|_| "No Cargo.toml found. Run Terrarium from a parser directory.")?;

    let toml: Value = content.parse().map_err(|_| "Invalid Cargo.toml")?;

    toml["package"]["name"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| "No package name in Cargo.toml".into())
}

#[tauri::command]
fn build_parser() -> Result<String, String> {
    let output = Command::new("cargo")
        .args(["build", "--features", "debug-trace"])
        .output()
        .map_err(|e| format!("Failed to run cargo: {}", e))?;

    if output.status.success() {
        Ok("Build successful".into())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).into_owned())
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![get_parser_name, build_parser])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
