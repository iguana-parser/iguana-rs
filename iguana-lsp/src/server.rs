use std::error::Error;

use iguana_lsp::diagnostics::diagnostics;
use iguana_lsp::document_symbols::document_symbols;
use iguana_lsp::folding::folding_ranges;
use iguana_lsp::format::format;
use iguana_lsp::references::{definition, references};
use iguana_lsp::semantic_tokens::semantic_tokens;
use iguana_lsp::{BuildResult, build, build_grammar_def, build_spans};
use iguana_runtime::{arena::Arena, input::Input};
use lsp_server::{Connection, Message, Notification, Request, RequestId, Response};
use lsp_types::notification::Notification as LspNotification;
use lsp_types::request::{
    DocumentSymbolRequest, FoldingRangeRequest, Formatting, GotoDefinition, References,
    Request as LspRequest, SemanticTokensFullRequest,
};
use lsp_types::{
    Diagnostic, DocumentSymbolResponse, Position, PublishDiagnosticsParams, Range, SemanticTokens,
    SemanticTokensResult, TextEdit, Uri,
};

pub fn main_loop(
    connection: Connection,
    params: serde_json::Value,
) -> Result<(), Box<dyn Error + Sync + Send>> {
    let _params: lsp_types::InitializeParams = serde_json::from_value(params).unwrap();
    eprintln!("starting main loop");
    for msg in &connection.receiver {
        match msg {
            Message::Request(req) => {
                if connection.handle_shutdown(&req)? {
                    return Ok(());
                }
                match req.method.as_str() {
                    SemanticTokensFullRequest::METHOD => {
                        let (id, params) = cast::<SemanticTokensFullRequest>(req);
                        let uri = &params.text_document.uri;
                        let path = uri.path().as_str();
                        let source = match std::fs::read_to_string(path) {
                            Ok(s) => s,
                            Err(e) => {
                                eprintln!("failed to read {}: {}", path, e);
                                continue;
                            }
                        };
                        let input = Input::from(source.as_str());
                        let tree_arena = Arena::new();
                        let tokens = match build(&input, &tree_arena) {
                            BuildResult::Success { tree, .. } => semantic_tokens(tree, &input),
                            BuildResult::Error { .. } | BuildResult::Ambiguous => vec![],
                        };
                        let result = SemanticTokensResult::Tokens(SemanticTokens {
                            result_id: None,
                            data: tokens,
                        });
                        let result = serde_json::to_value(&result).unwrap();
                        let resp = Response::new_ok(id, result);
                        connection.sender.send(Message::Response(resp))?;
                    }
                    Formatting::METHOD => {
                        let (id, params) = cast::<Formatting>(req);
                        let uri = &params.text_document.uri;
                        let path = uri.path().as_str();
                        let source = match std::fs::read_to_string(path) {
                            Ok(s) => s,
                            Err(e) => {
                                eprintln!("failed to read {}: {}", path, e);
                                continue;
                            }
                        };
                        let input = Input::from(source.as_str());
                        let tree_arena = Arena::new();
                        let edits = match build(&input, &tree_arena) {
                            BuildResult::Success { tree, .. } => {
                                let formatted = format(tree, &input);
                                let line_count = source.lines().count() as u32;
                                let last_line_len =
                                    source.lines().last().map_or(0, |l| l.len()) as u32;
                                Some(vec![TextEdit {
                                    range: Range {
                                        start: Position::new(0, 0),
                                        end: Position::new(line_count, last_line_len),
                                    },
                                    new_text: formatted,
                                }])
                            }
                            BuildResult::Error { .. } | BuildResult::Ambiguous => None,
                        };
                        let result = serde_json::to_value(&edits).unwrap();
                        let resp = Response::new_ok(id, result);
                        connection.sender.send(Message::Response(resp))?;
                    }
                    References::METHOD => {
                        let (id, params) = cast::<References>(req);
                        let uri = &params.text_document_position.text_document.uri;
                        let path = uri.path().as_str();
                        let source = match std::fs::read_to_string(path) {
                            Ok(s) => s,
                            Err(e) => {
                                eprintln!("failed to read {}: {}", path, e);
                                continue;
                            }
                        };
                        let input = Input::from(source.as_str());
                        let tree_arena = Arena::new();
                        let locations = (|| {
                            let BuildResult::Success { tree, .. } = build(&input, &tree_arena)
                            else {
                                return None;
                            };
                            let grammar_def = build_grammar_def(tree, &input)?;
                            let spans = build_spans(&grammar_def, tree, &input);
                            let pos = params.text_document_position.position;
                            let offset = input.offset(pos.line, pos.character);
                            let include_declaration = params.context.include_declaration;
                            Some(references(&spans, &input, uri, offset, include_declaration))
                        })()
                        .unwrap_or_default();
                        let result = serde_json::to_value(&locations).unwrap();
                        let resp = Response::new_ok(id, result);
                        connection.sender.send(Message::Response(resp))?;
                    }
                    GotoDefinition::METHOD => {
                        let (id, params) = cast::<GotoDefinition>(req);
                        let uri = &params.text_document_position_params.text_document.uri;
                        let path = uri.path().as_str();
                        let source = match std::fs::read_to_string(path) {
                            Ok(s) => s,
                            Err(e) => {
                                eprintln!("failed to read {}: {}", path, e);
                                continue;
                            }
                        };
                        let input = Input::from(source.as_str());
                        let tree_arena = Arena::new();
                        let loc = (|| {
                            let BuildResult::Success { tree, .. } = build(&input, &tree_arena)
                            else {
                                return None;
                            };
                            let grammar_def = build_grammar_def(tree, &input)?;
                            let spans = build_spans(&grammar_def, tree, &input);
                            let pos = params.text_document_position_params.position;
                            let offset = input.offset(pos.line, pos.character);
                            definition(&spans, &input, uri, offset)
                        })();
                        let result = serde_json::to_value(
                            loc.map(lsp_types::GotoDefinitionResponse::Scalar),
                        )
                        .unwrap();
                        let resp = Response::new_ok(id, result);
                        connection.sender.send(Message::Response(resp))?;
                    }
                    DocumentSymbolRequest::METHOD => {
                        let (id, params) = cast::<DocumentSymbolRequest>(req);
                        let uri = &params.text_document.uri;
                        let path = uri.path().as_str();
                        let source = match std::fs::read_to_string(path) {
                            Ok(s) => s,
                            Err(e) => {
                                eprintln!("failed to read {}: {}", path, e);
                                continue;
                            }
                        };
                        let input = Input::from(source.as_str());
                        let tree_arena = Arena::new();
                        let symbols = (|| {
                            let BuildResult::Success { tree, .. } = build(&input, &tree_arena)
                            else {
                                return None;
                            };
                            let grammar_def = build_grammar_def(tree, &input)?;
                            let spans = build_spans(&grammar_def, tree, &input);
                            Some(document_symbols(&grammar_def, &spans, &input))
                        })()
                        .unwrap_or_default();
                        let result =
                            serde_json::to_value(DocumentSymbolResponse::Nested(symbols)).unwrap();
                        let resp = Response::new_ok(id, result);
                        connection.sender.send(Message::Response(resp))?;
                    }
                    FoldingRangeRequest::METHOD => {
                        let (id, params) = cast::<FoldingRangeRequest>(req);
                        let uri = &params.text_document.uri;
                        let path = uri.path().as_str();
                        let source = match std::fs::read_to_string(path) {
                            Ok(s) => s,
                            Err(e) => {
                                eprintln!("failed to read {}: {}", path, e);
                                continue;
                            }
                        };
                        let input = Input::from(source.as_str());
                        let tree_arena = Arena::new();
                        let ranges = (|| {
                            let BuildResult::Success { tree, .. } = build(&input, &tree_arena)
                            else {
                                return None;
                            };
                            let grammar_def = build_grammar_def(tree, &input)?;
                            let spans = build_spans(&grammar_def, tree, &input);
                            Some(folding_ranges(&grammar_def, &spans, &input))
                        })()
                        .unwrap_or_default();
                        let result = serde_json::to_value(&ranges).unwrap();
                        let resp = Response::new_ok(id, result);
                        connection.sender.send(Message::Response(resp))?;
                    }
                    _ => {}
                }
            }
            Message::Response(resp) => {
                eprintln!("got response: {resp:?}");
            }
            Message::Notification(notif) => match notif.method.as_str() {
                lsp_types::notification::DidOpenTextDocument::METHOD => {
                    let params: lsp_types::DidOpenTextDocumentParams =
                        serde_json::from_value(notif.params).unwrap();
                    publish_diagnostics(
                        &connection,
                        params.text_document.uri,
                        &params.text_document.text,
                    )?;
                }
                lsp_types::notification::DidChangeTextDocument::METHOD => {
                    let params: lsp_types::DidChangeTextDocumentParams =
                        serde_json::from_value(notif.params).unwrap();
                    if let Some(change) = params.content_changes.into_iter().last() {
                        publish_diagnostics(&connection, params.text_document.uri, &change.text)?;
                    }
                }
                _ => {}
            },
        }
    }
    Ok(())
}

fn publish_diagnostics(
    connection: &Connection,
    uri: Uri,
    source: &str,
) -> Result<(), Box<dyn std::error::Error + Sync + Send>> {
    let input = Input::from(source);
    let tree_arena = Arena::new();
    let diagnostics = match build(&input, &tree_arena) {
        BuildResult::Success { tree, .. } => build_grammar_def(tree, &input)
            .map(|grammar_def| {
                let spans = build_spans(&grammar_def, tree, &input);
                diagnostics(&grammar_def, &spans, &input)
            })
            .unwrap_or_default(),
        BuildResult::Ambiguous => vec![],
        BuildResult::Error {
            line,
            column,
            len,
            message,
        } => {
            vec![Diagnostic {
                range: Range {
                    start: Position::new(line, column),
                    end: Position::new(line, column + len),
                },
                severity: Some(lsp_types::DiagnosticSeverity::ERROR),
                message,
                ..Default::default()
            }]
        }
    };
    let params = PublishDiagnosticsParams {
        uri,
        diagnostics,
        version: None,
    };
    let notif = Notification {
        method: lsp_types::notification::PublishDiagnostics::METHOD.to_string(),
        params: serde_json::to_value(params).unwrap(),
    };
    connection.sender.send(Message::Notification(notif))?;
    Ok(())
}

fn cast<R>(req: Request) -> (RequestId, R::Params)
where
    R: lsp_types::request::Request,
    R::Params: serde::de::DeserializeOwned,
{
    req.extract(R::METHOD).unwrap()
}
