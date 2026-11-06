use std::{
    env, fs,
    io::{BufRead, BufReader, Read, Write},
    path::PathBuf,
    process::{self, Child, ChildStdin, ChildStdout, Command, Stdio},
    str::FromStr,
    time::{SystemTime, UNIX_EPOCH},
};

use lsp_server::{Message, Notification, Request, RequestId, Response};
use lsp_types::{
    Diagnostic, DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
    DocumentFormattingParams, FormattingOptions, GotoDefinitionParams, GotoDefinitionResponse,
    InitializeParams, InitializedParams, PartialResultParams, Position, PublishDiagnosticsParams,
    Range, ReferenceContext, ReferenceParams, TextDocumentContentChangeEvent,
    TextDocumentIdentifier, TextDocumentItem, TextDocumentPositionParams, TextEdit, Uri,
    VersionedTextDocumentIdentifier, WorkDoneProgressParams,
    notification::{
        DidChangeTextDocument, DidCloseTextDocument, DidOpenTextDocument, Exit, Initialized,
        Notification as _, PublishDiagnostics,
    },
    request::{Formatting, GotoDefinition, Initialize, References, Shutdown},
};

struct LspProcess {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

impl LspProcess {
    fn start() -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_iguana-lsp"))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let stdin = child.stdin.take().unwrap();
        let stdout = BufReader::new(child.stdout.take().unwrap());
        let mut process = Self {
            child,
            stdin,
            stdout,
        };

        process.request::<Initialize>(InitializeParams::default());
        process.notify::<Initialized>(InitializedParams {});
        process
    }

    fn send(&mut self, message: Message) {
        let payload = serde_json::to_vec(&message).unwrap();
        write!(self.stdin, "Content-Length: {}\r\n\r\n", payload.len()).unwrap();
        self.stdin.write_all(&payload).unwrap();
        self.stdin.flush().unwrap();
    }

    fn receive(&mut self) -> Message {
        let mut content_length = None;
        loop {
            let mut header = String::new();
            assert_ne!(self.stdout.read_line(&mut header).unwrap(), 0);
            if header == "\r\n" {
                break;
            }
            if let Some(value) = header.strip_prefix("Content-Length:") {
                content_length = Some(value.trim().parse::<usize>().unwrap());
            }
        }

        let mut payload = vec![0; content_length.unwrap()];
        self.stdout.read_exact(&mut payload).unwrap();
        serde_json::from_slice(&payload).unwrap()
    }

    fn receive_response(&mut self, id: &RequestId) -> Response {
        loop {
            match self.receive() {
                Message::Response(response) if response.id == *id => return response,
                Message::Notification(_) => {}
                other => panic!("unexpected server message: {other:?}"),
            }
        }
    }

    fn request<R: lsp_types::request::Request>(&mut self, params: R::Params) -> R::Result {
        let id = RequestId::from(1);
        self.send(Message::Request(Request::new(
            id.clone(),
            R::METHOD.to_string(),
            params,
        )));
        let response = self.receive_response(&id);
        serde_json::from_value(response.response_result.unwrap()).unwrap()
    }

    fn notify<N: lsp_types::notification::Notification>(&mut self, params: N::Params) {
        self.send(Message::Notification(Notification::new(
            N::METHOD.to_string(),
            params,
        )));
    }

    fn open(&mut self, uri: &Uri, text: &str) {
        self.notify::<DidOpenTextDocument>(DidOpenTextDocumentParams {
            text_document: TextDocumentItem::new(
                uri.clone(),
                "iggy".to_string(),
                1,
                text.to_string(),
            ),
        });
    }

    fn change(&mut self, uri: &Uri, text: &str) {
        self.notify::<DidChangeTextDocument>(DidChangeTextDocumentParams {
            text_document: VersionedTextDocumentIdentifier::new(uri.clone(), 2),
            content_changes: vec![TextDocumentContentChangeEvent {
                range: None,
                range_length: None,
                text: text.to_string(),
            }],
        });
    }

    fn close(&mut self, uri: &Uri) {
        self.notify::<DidCloseTextDocument>(DidCloseTextDocumentParams {
            text_document: TextDocumentIdentifier::new(uri.clone()),
        });
    }

    fn format(&mut self, uri: &Uri) -> Option<Vec<TextEdit>> {
        self.request::<Formatting>(DocumentFormattingParams {
            text_document: TextDocumentIdentifier::new(uri.clone()),
            options: FormattingOptions {
                tab_size: 2,
                insert_spaces: true,
                ..FormattingOptions::default()
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
        })
    }

    fn definition(&mut self, uri: &Uri, position: Position) -> Option<Range> {
        let result = self.request::<GotoDefinition>(GotoDefinitionParams {
            text_document_position_params: TextDocumentPositionParams::new(
                TextDocumentIdentifier::new(uri.clone()),
                position,
            ),
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        });
        result.map(|result| match result {
            GotoDefinitionResponse::Scalar(location) => location.range,
            other => panic!("unexpected definition response: {other:?}"),
        })
    }

    fn references(&mut self, uri: &Uri, position: Position) -> Vec<Range> {
        let locations = self.request::<References>(ReferenceParams {
            text_document_position: TextDocumentPositionParams::new(
                TextDocumentIdentifier::new(uri.clone()),
                position,
            ),
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
            context: ReferenceContext {
                include_declaration: false,
            },
        });
        locations
            .unwrap_or_default()
            .into_iter()
            .map(|location| location.range)
            .collect()
    }

    fn receive_diagnostics(&mut self) -> Vec<Diagnostic> {
        match self.receive() {
            Message::Notification(notification)
                if notification.method == PublishDiagnostics::METHOD =>
            {
                let params: PublishDiagnosticsParams =
                    serde_json::from_value(notification.params).unwrap();
                params.diagnostics
            }
            other => panic!("unexpected server message: {other:?}"),
        }
    }

    fn shutdown(mut self) {
        self.request::<Shutdown>(());
        self.notify::<Exit>(());
        assert!(self.child.wait().unwrap().success());
    }
}

impl Drop for LspProcess {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// A file path under the temporary directory that does not exist yet, and its
/// URI.
fn temporary_file(name: &str) -> (PathBuf, Uri) {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = env::temp_dir().join(format!("iguana-lsp-{name}-{}-{unique}.iggy", process::id()));
    let uri = Uri::from_str(&format!("file://{}", path.display())).unwrap();
    (path, uri)
}

/// The URI of a document that exists only in the editor.
fn editor_uri() -> Uri {
    Uri::from_str("file:///live-document.iggy").unwrap()
}

/// `A` refers to `B` on line 3, and the head of `B` is on line `5 + blank_lines`.
fn grammar_with_gap(blank_lines: usize) -> String {
    format!(
        "grammar T\n\nA\n  = B\n\n{}B\n  = \"x\"\n",
        "\n".repeat(blank_lines)
    )
}

#[test]
fn formatting_uses_text_from_the_latest_document_change() {
    let (path, uri) = temporary_file("live-document");
    let saved = "grammar Test\n\nA = \"a\"\n\nB = \"b\"\n";
    let live = "grammar Test\n\nA = \"a\"\n\n// middle comment\n\nB = \"😀\"";
    fs::write(&path, saved).unwrap();

    let mut server = LspProcess::start();
    server.open(&uri, saved);
    server.change(&uri, live);

    assert_eq!(
        server.format(&uri),
        Some(vec![TextEdit {
            range: Range::new(Position::new(0, 0), Position::new(6, 8)),
            new_text: "grammar Test\n\nA\n  = \"a\"\n\n// middle comment\n\nB\n  = \"😀\"\n"
                .to_string(),
        }])
    );

    server.shutdown();
    fs::remove_file(path).unwrap();
}

#[test]
fn unchanged_formatting_returns_no_edits() {
    let (path, uri) = temporary_file("unchanged-formatting");
    let source = "grammar Test\n\nS=(\"a\" // inside group\n| \"b\")\n";
    fs::write(&path, source).unwrap();

    let mut server = LspProcess::start();
    assert_eq!(server.format(&uri), Some(Vec::new()));

    server.shutdown();
    assert_eq!(fs::read_to_string(&path).unwrap(), source);
    fs::remove_file(path).unwrap();
}

#[test]
fn formatting_range_includes_a_trailing_newline() {
    let (path, uri) = temporary_file("trailing-newline");
    let source = "grammar Test\n\nS=\"a\"\n";
    fs::write(&path, source).unwrap();

    let mut server = LspProcess::start();
    assert_eq!(
        server.format(&uri),
        Some(vec![TextEdit {
            range: Range::new(Position::new(0, 0), Position::new(3, 0)),
            new_text: "grammar Test\n\nS\n  = \"a\"\n".to_string(),
        }])
    );

    server.shutdown();
    assert_eq!(fs::read_to_string(&path).unwrap(), source);
    fs::remove_file(path).unwrap();
}

#[test]
fn requests_answer_from_the_latest_document_change() {
    let uri = editor_uri();
    let reference = Position::new(3, 4);

    let mut server = LspProcess::start();
    server.open(&uri, &grammar_with_gap(0));
    assert_eq!(
        server.definition(&uri, reference).unwrap().start,
        Position::new(5, 0)
    );

    server.change(&uri, &grammar_with_gap(3));
    assert_eq!(
        server.definition(&uri, reference).unwrap().start,
        Position::new(8, 0)
    );
    assert_eq!(
        server.references(&uri, Position::new(8, 0))[0].start,
        reference
    );

    server.shutdown();
}

#[test]
fn closed_document_answers_from_the_file_on_disk() {
    let (path, uri) = temporary_file("closed-document");
    let reference = Position::new(3, 4);
    fs::write(&path, grammar_with_gap(3)).unwrap();

    let mut server = LspProcess::start();
    server.open(&uri, &grammar_with_gap(0));
    assert_eq!(
        server.definition(&uri, reference).unwrap().start,
        Position::new(5, 0)
    );

    server.close(&uri);
    assert_eq!(
        server.definition(&uri, reference).unwrap().start,
        Position::new(8, 0)
    );

    server.shutdown();
    fs::remove_file(path).unwrap();
}

#[test]
fn unopened_document_follows_the_file_on_disk() {
    let (path, uri) = temporary_file("unopened-document");
    let reference = Position::new(3, 4);
    fs::write(&path, grammar_with_gap(0)).unwrap();

    let mut server = LspProcess::start();
    assert_eq!(
        server.definition(&uri, reference).unwrap().start,
        Position::new(5, 0)
    );

    fs::write(&path, grammar_with_gap(3)).unwrap();
    assert_eq!(
        server.definition(&uri, reference).unwrap().start,
        Position::new(8, 0)
    );

    server.shutdown();
    fs::remove_file(path).unwrap();
}

#[test]
fn position_after_the_last_line_has_no_answer() {
    let uri = editor_uri();
    let source = grammar_with_gap(0);
    let after_last_line = Position::new(source.lines().count() as u32, 0);

    let mut server = LspProcess::start();
    server.open(&uri, &source);
    assert_eq!(server.definition(&uri, after_last_line), None);
    assert_eq!(server.references(&uri, after_last_line), vec![]);

    server.shutdown();
}

#[test]
fn diagnostics_follow_each_document_change() {
    let uri = editor_uri();

    let mut server = LspProcess::start();
    server.open(&uri, "grammar");
    let diagnostics = server.receive_diagnostics();
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].range.start, Position::new(0, 7));

    server.change(&uri, &grammar_with_gap(0));
    assert_eq!(server.receive_diagnostics(), vec![]);

    server.change(&uri, "grammar T\n\nA\n  = Missing\n");
    let diagnostics = server.receive_diagnostics();
    assert_eq!(diagnostics.len(), 1);
    assert!(diagnostics[0].message.contains("Missing"));

    server.shutdown();
}

#[test]
fn multiline_literal_publishes_diagnostics() {
    let uri = editor_uri();

    let mut server = LspProcess::start();
    server.open(&uri, "grammar T\nS = \"abc\nx\"\n");
    assert_eq!(server.receive_diagnostics(), vec![]);

    server.shutdown();
}
