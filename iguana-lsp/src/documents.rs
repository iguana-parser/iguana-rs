use std::{borrow::Cow, fs, io};

use lsp_types::{Position, Uri};
use rustc_hash::FxHashMap;

/// A map from open document URIs to their current text.
///
/// The server uses the live editor text when a document is open and falls back to
/// the file on disk for URIs the client has not opened.
#[derive(Default)]
pub struct DocumentStore {
    open: FxHashMap<Uri, String>,
}

impl DocumentStore {
    pub fn set(&mut self, uri: Uri, text: String) {
        self.open.insert(uri, text);
    }

    pub fn close(&mut self, uri: &Uri) {
        self.open.remove(uri);
    }

    pub fn source<'a>(&'a self, uri: &Uri) -> io::Result<Cow<'a, str>> {
        match self.open.get(uri) {
            Some(source) => Ok(Cow::Borrowed(source)),
            None => fs::read_to_string(uri.path().as_str()).map(Cow::Owned),
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
