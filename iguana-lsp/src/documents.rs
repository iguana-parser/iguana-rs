use std::{fs, io};

use iguana_lsp::analysis::Analysis;
use lsp_types::{Position, Uri};
use rustc_hash::FxHashMap;

/// A map from open document URIs to their analyses.
#[derive(Default)]
pub struct DocumentStore {
    open: FxHashMap<Uri, Analysis>,
}

impl DocumentStore {
    pub fn set(&mut self, uri: Uri, analysis: Analysis) {
        self.open.insert(uri, analysis);
    }

    pub fn close(&mut self, uri: &Uri) {
        self.open.remove(uri);
    }

    /// Calls `f` with the analysis of `uri`. A file the client has not opened
    /// is read and analyzed for this call only, because the server does not
    /// hear about later changes to the file.
    pub fn with_analysis<T>(&self, uri: &Uri, f: impl FnOnce(&Analysis) -> T) -> io::Result<T> {
        match self.open.get(uri) {
            Some(analysis) => Ok(f(analysis)),
            None => {
                let source = fs::read_to_string(uri.path().as_str())?;
                Ok(f(&Analysis::new(&source)))
            }
        }
    }
}

/// The LSP position after the document's final character.
/// LSP character offsets use UTF-16 code units rather than UTF-8 bytes.
pub fn end_position(source: &str) -> Position {
    let line = source.chars().filter(|c| *c == '\n').count() as u32;
    let character = source.rsplit('\n').next().unwrap().encode_utf16().count() as u32;
    Position::new(line, character)
}
