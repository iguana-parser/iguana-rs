use std::{
    fs::File,
    io::{self, BufWriter, Write},
    path::Path,
    process::{Command, Stdio},
};

use serde::Serialize;

/// A graph that can serialize itself to Graphviz DOT text.
pub trait ToDot {
    fn write_dot<W: Write>(&self, w: &mut W) -> io::Result<()>;
}

/// Write `graph` to `path` as JSON, or as SVG when `svg` is set.
///
/// SVG is rendered from the graph's DOT text by the graphviz `dot` binary.
pub fn write_graph<G: Serialize + ToDot>(graph: &G, path: &Path, svg: bool) -> io::Result<()> {
    if svg {
        let mut dot = Vec::new();
        graph.write_dot(&mut dot)?;
        dot_to_svg(&dot, path)
    } else {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);
        writeln!(writer, "{}", serde_json::to_string(graph).unwrap())
    }
}

/// Render DOT source to an SVG file by piping it through the graphviz `dot` binary.
fn dot_to_svg(dot_source: &[u8], path: &Path) -> io::Result<()> {
    let mut child = Command::new("dot")
        .arg("-Tsvg")
        .arg("-o")
        .arg(path)
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|e| {
            io::Error::new(
                e.kind(),
                format!("could not run graphviz `dot` (is graphviz installed?): {e}"),
            )
        })?;
    child.stdin.take().unwrap().write_all(dot_source)?;
    let status = child.wait()?;
    if !status.success() {
        return Err(io::Error::other(format!(
            "graphviz `dot` exited with {status}"
        )));
    }
    Ok(())
}

/// Escape a string for use inside a double-quoted DOT label.
pub(crate) fn escape_label(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}
