use std::{
    fs,
    io::{BufRead, BufReader, Read, Write},
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
    str::FromStr,
    time::{SystemTime, UNIX_EPOCH},
};

use lsp_server::{Message, Notification, Request, RequestId, Response};
use lsp_types::{
    DidChangeTextDocumentParams, DidOpenTextDocumentParams, DocumentFormattingParams,
    FormattingOptions, InitializeParams, Position, Range, TextDocumentContentChangeEvent,
    TextDocumentIdentifier, TextDocumentItem, TextEdit, Uri, VersionedTextDocumentIdentifier,
    WorkDoneProgressParams,
    notification::{DidChangeTextDocument, DidOpenTextDocument, Exit},
    request::{Formatting, Shutdown},
};
use lsp_types::{notification::Notification as _, request::Request as _};

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

        let initialize_id = RequestId::from(0);
        process.send(Message::Request(Request::new(
            initialize_id.clone(),
            lsp_types::request::Initialize::METHOD.to_string(),
            InitializeParams::default(),
        )));
        process.receive_response(&initialize_id);
        process.send(Message::Notification(Notification::new(
            lsp_types::notification::Initialized::METHOD.to_string(),
            lsp_types::InitializedParams {},
        )));
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

    fn format(&mut self, uri: Uri) -> Option<Vec<TextEdit>> {
        let formatting_id = RequestId::from(1);
        self.send(Message::Request(Request::new(
            formatting_id.clone(),
            Formatting::METHOD.to_string(),
            DocumentFormattingParams {
                text_document: TextDocumentIdentifier::new(uri),
                options: FormattingOptions {
                    tab_size: 2,
                    insert_spaces: true,
                    ..FormattingOptions::default()
                },
                work_done_progress_params: WorkDoneProgressParams::default(),
            },
        )));

        let response = self.receive_response(&formatting_id);
        serde_json::from_value(response.response_result.unwrap()).unwrap()
    }

    fn shutdown(mut self) {
        let shutdown_id = RequestId::from(2);
        self.send(Message::Request(Request::new(
            shutdown_id.clone(),
            Shutdown::METHOD.to_string(),
            (),
        )));
        self.receive_response(&shutdown_id);
        self.send(Message::Notification(Notification::new(
            Exit::METHOD.to_string(),
            (),
        )));
        assert!(self.child.wait().unwrap().success());
    }
}

impl Drop for LspProcess {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[test]
fn formatting_uses_text_from_the_latest_document_change() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "iguana-lsp-live-document-{}-{unique}.iggy",
        std::process::id()
    ));
    let saved = "grammar Test\n\nA = \"a\"\n\nB = \"b\"\n";
    let live = "grammar Test\n\nA = \"a\"\n\n// middle comment\n\nB = \"😀\"";
    fs::write(&path, saved).unwrap();
    let uri = Uri::from_str(&format!("file://{}", path.display())).unwrap();

    let mut server = LspProcess::start();

    server.send(Message::Notification(Notification::new(
        DidOpenTextDocument::METHOD.to_string(),
        DidOpenTextDocumentParams {
            text_document: TextDocumentItem::new(
                uri.clone(),
                "iggy".to_string(),
                1,
                saved.to_string(),
            ),
        },
    )));

    server.send(Message::Notification(Notification::new(
        DidChangeTextDocument::METHOD.to_string(),
        DidChangeTextDocumentParams {
            text_document: VersionedTextDocumentIdentifier::new(uri.clone(), 2),
            content_changes: vec![TextDocumentContentChangeEvent {
                range: None,
                range_length: None,
                text: live.to_string(),
            }],
        },
    )));

    let edits = server.format(uri);
    assert_eq!(
        edits,
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
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "iguana-lsp-unchanged-formatting-{}-{unique}.iggy",
        std::process::id()
    ));
    let source = "grammar Test\n\nS=(\"a\" // inside group\n| \"b\")\n";
    fs::write(&path, source).unwrap();
    let uri = Uri::from_str(&format!("file://{}", path.display())).unwrap();

    let mut server = LspProcess::start();
    assert_eq!(server.format(uri), Some(Vec::new()));

    server.shutdown();
    assert_eq!(fs::read_to_string(&path).unwrap(), source);
    fs::remove_file(path).unwrap();
}

#[test]
fn formatting_range_includes_a_trailing_newline() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "iguana-lsp-trailing-newline-{}-{unique}.iggy",
        std::process::id()
    ));
    let source = "grammar Test\n\nS=\"a\"\n";
    fs::write(&path, source).unwrap();
    let uri = Uri::from_str(&format!("file://{}", path.display())).unwrap();

    let mut server = LspProcess::start();
    assert_eq!(
        server.format(uri),
        Some(vec![TextEdit {
            range: Range::new(Position::new(0, 0), Position::new(3, 0)),
            new_text: "grammar Test\n\nS\n  = \"a\"\n".to_string(),
        }])
    );

    server.shutdown();
    assert_eq!(fs::read_to_string(&path).unwrap(), source);
    fs::remove_file(path).unwrap();
}
