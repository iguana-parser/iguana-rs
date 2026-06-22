//! The web viewer: the static parse-view app, embedded in the binary at build
//! time and written next to a built bundle to serve over HTTP.
//!
//! The viewer is grammar-independent, so it is built once from `web-viewer/`
//! and compiled into `iguana` verbatim. `iguana try` writes it alongside a
//! built bundle's wasm module and manifest and serves the directory, since a
//! wasm module loads over `http://` rather than `file://`.

use std::{fs, io, net::SocketAddr, path::Path, sync::Arc, thread};

use include_dir::{Dir, File, include_dir};
use tiny_http::{Header, Response, Server};

/// The viewer assets, built by Vite from `web-viewer/`. The grammar-specific
/// `manifest.json` and `wasm/` are not part of the viewer, so the build keeps
/// them out and `write_assets` skips them defensively.
static VIEWER: Dir = include_dir!("$CARGO_MANIFEST_DIR/../web-viewer/dist");

/// The fixed directory a wasm bundle is generated into and served from.
/// `iguana generate --wasm` writes the bundle here and `iguana try` serves it.
pub const WEBVIEW_DIR: &str = "webview";

/// Write the embedded viewer assets into `output_dir`, the root of a generated
/// wasm bundle. The grammar's own `manifest.json` and `wasm/pkg/` already live
/// there, so those paths are skipped rather than overwritten.
pub fn write_assets(output_dir: &Path) -> io::Result<()> {
    let mut files = Vec::new();
    collect_files(&VIEWER, &mut files);
    for file in files {
        let path = file.path();
        if path == Path::new("manifest.json") || path.starts_with("wasm") {
            continue;
        }
        let dest = output_dir.join(path);
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(dest, file.contents())?;
    }
    Ok(())
}

fn collect_files<'a>(dir: &'a Dir<'a>, out: &mut Vec<&'a File<'a>>) {
    out.extend(dir.files());
    for sub in dir.dirs() {
        collect_files(sub, out);
    }
}

/// Open the web viewer for a bundle built with `iguana generate --wasm`: write
/// the viewer next to the bundle's wasm module and manifest, then serve it.
///
/// The wasm module and manifest are the build's output, not the viewer's, so a
/// missing one means the bundle was never built; say so rather than serving a
/// viewer with nothing to load.
pub fn try_bundle(dir: &Path, port: u16) -> io::Result<()> {
    let wasm = dir.join("wasm").join("pkg").join("parser_bg.wasm");
    let manifest = dir.join("manifest.json");
    if !wasm.exists() || !manifest.exists() {
        return Err(io::Error::other(format!(
            "no wasm bundle in {}. Build it first with `iguana generate --wasm`.",
            dir.display()
        )));
    }
    write_assets(dir)?;
    serve(dir, port)
}

/// Serve the bundle at `dir` over HTTP on `127.0.0.1:<port>` until interrupted,
/// printing the URL for the user to open in a browser. The server does not open
/// the browser itself.
///
/// A few worker threads share the listener so the browser's parallel asset
/// requests do not queue behind one another.
pub fn serve(dir: &Path, port: u16) -> io::Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let server = Server::http(addr).map_err(|e| io::Error::other(e.to_string()))?;
    let server = Arc::new(server);

    let url = format!("http://{addr}/");
    match grammar_name(dir) {
        Some(name) => println!("Open the {name} grammar web view at {url}"),
        None => println!("Open the grammar web view at {url}"),
    }
    println!("Press Ctrl-C to stop");

    let mut workers = Vec::new();
    for _ in 0..4 {
        let server = Arc::clone(&server);
        let dir = dir.to_path_buf();
        workers.push(thread::spawn(move || {
            for request in server.incoming_requests() {
                let response = build_response(&dir, request.url());
                let _ = request.respond(response);
            }
        }));
    }
    for worker in workers {
        let _ = worker.join();
    }
    Ok(())
}

/// The grammar name from the bundle's `manifest.json`, or `None` when the file
/// is missing or cannot be parsed. The view message falls back to a generic
/// phrase in that case.
fn grammar_name(dir: &Path) -> Option<String> {
    let manifest = fs::read_to_string(dir.join("manifest.json")).ok()?;
    let value: serde_json::Value = serde_json::from_str(&manifest).ok()?;
    value.get("grammar")?.as_str().map(str::to_owned)
}

/// Resolve a request URL to a file under `dir` and build its response. `/`
/// serves `index.html`; a path that escapes `dir` or is missing returns 404.
fn build_response(dir: &Path, url: &str) -> Response<io::Cursor<Vec<u8>>> {
    let path = url.split('?').next().unwrap_or("/");
    let relative = path.trim_start_matches('/');
    if relative.split('/').any(|segment| segment == "..") {
        return not_found();
    }

    let mut file = if relative.is_empty() {
        dir.join("index.html")
    } else {
        dir.join(relative)
    };
    if file.is_dir() {
        file = file.join("index.html");
    }

    match fs::read(&file) {
        Ok(bytes) => Response::from_data(bytes).with_header(content_type(&file)),
        Err(_) => not_found(),
    }
}

fn not_found() -> Response<io::Cursor<Vec<u8>>> {
    Response::from_string("Not found").with_status_code(404)
}

fn content_type(path: &Path) -> Header {
    let mime = match path.extension().and_then(|e| e.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("js" | "mjs") => "text/javascript; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("json" | "map") => "application/json; charset=utf-8",
        Some("wasm") => "application/wasm",
        Some("svg") => "image/svg+xml",
        Some("ttf") => "font/ttf",
        Some("woff") => "font/woff",
        Some("woff2") => "font/woff2",
        _ => "application/octet-stream",
    };
    Header::from_bytes(&b"Content-Type"[..], mime.as_bytes()).expect("static header is valid")
}
