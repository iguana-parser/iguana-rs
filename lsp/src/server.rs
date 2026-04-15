use std::error::Error;

use lsp_server::{Connection, Message, Notification, Request, RequestId, Response};
use lsp_types::notification::Notification as LspNotification;
use lsp_types::request::{
    DocumentSymbolRequest, FoldingRangeRequest, Formatting, GotoDefinition, References,
    Request as LspRequest, SemanticTokensFullRequest,
};
use lsp_types::{
    DocumentSymbolResponse, Position, PublishDiagnosticsParams, Range, SemanticTokens,
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
                        let parse_result = lsp::parse(&source);
                        if parse_result.tree.is_none() {
                            eprintln!("parse error: grammar has syntax errors");
                        }
                        let tokens = lsp::semantic_tokens::semantic_tokens(&parse_result);
                        let result = SemanticTokensResult::Tokens(SemanticTokens {
                            result_id: None,
                            data: tokens,
                        });
                        let result = serde_json::to_value(&result).unwrap();
                        let resp = Response {
                            id,
                            result: Some(result),
                            error: None,
                        };
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
                        let parse_result = lsp::parse(&source);
                        if parse_result.tree.is_none() {
                            eprintln!("parse error: grammar has syntax errors, cannot format");
                        }
                        let edits = lsp::format::format(&parse_result).map(|formatted| {
                            let line_count = source.lines().count() as u32;
                            let last_line_len =
                                source.lines().last().map_or(0, |l| l.len()) as u32;
                            vec![TextEdit {
                                range: Range {
                                    start: Position::new(0, 0),
                                    end: Position::new(line_count, last_line_len),
                                },
                                new_text: formatted,
                            }]
                        });
                        let result = serde_json::to_value(&edits).unwrap();
                        let resp = Response {
                            id,
                            result: Some(result),
                            error: None,
                        };
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
                        let parse_result = lsp::parse(&source);
                        let locations = (|| {
                            let grammar_def = lsp::build_grammar_def(&parse_result)?;
                            let spans = lsp::build_spans(&grammar_def, &parse_result)?;
                            let pos = params.text_document_position.position;
                            let offset = parse_result.input.offset(pos.line, pos.character);
                            let include_declaration = params.context.include_declaration;
                            Some(lsp::references::references(
                                &spans,
                                &parse_result.input,
                                uri,
                                offset,
                                include_declaration,
                            ))
                        })()
                        .unwrap_or_default();
                        let result = serde_json::to_value(&locations).unwrap();
                        let resp = Response {
                            id,
                            result: Some(result),
                            error: None,
                        };
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
                        let parse_result = lsp::parse(&source);
                        let loc = (|| {
                            let grammar_def = lsp::build_grammar_def(&parse_result)?;
                            let spans = lsp::build_spans(&grammar_def, &parse_result)?;
                            let pos = params.text_document_position_params.position;
                            let offset = parse_result.input.offset(pos.line, pos.character);
                            lsp::references::definition(
                                &spans,
                                &parse_result.input,
                                uri,
                                offset,
                            )
                        })();
                        let result = serde_json::to_value(
                            &loc.map(lsp_types::GotoDefinitionResponse::Scalar),
                        )
                        .unwrap();
                        let resp = Response {
                            id,
                            result: Some(result),
                            error: None,
                        };
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
                        let parse_result = lsp::parse(&source);
                        let symbols = (|| {
                            let grammar_def = lsp::build_grammar_def(&parse_result)?;
                            let spans = lsp::build_spans(&grammar_def, &parse_result)?;
                            Some(lsp::document_symbols::document_symbols(
                                &grammar_def,
                                &spans,
                                &parse_result.input,
                            ))
                        })()
                        .unwrap_or_default();
                        let result = serde_json::to_value(
                            &DocumentSymbolResponse::Nested(symbols),
                        )
                        .unwrap();
                        let resp = Response {
                            id,
                            result: Some(result),
                            error: None,
                        };
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
                        let parse_result = lsp::parse(&source);
                        let ranges = (|| {
                            let grammar_def = lsp::build_grammar_def(&parse_result)?;
                            let spans = lsp::build_spans(&grammar_def, &parse_result)?;
                            Some(lsp::folding::folding_ranges(
                                &grammar_def,
                                &spans,
                                &parse_result.input,
                            ))
                        })()
                        .unwrap_or_default();
                        let result = serde_json::to_value(&ranges).unwrap();
                        let resp = Response {
                            id,
                            result: Some(result),
                            error: None,
                        };
                        connection.sender.send(Message::Response(resp))?;
                    }
                    _ => {}
                }
            }
            Message::Response(resp) => {
                eprintln!("got response: {resp:?}");
            }
            Message::Notification(notif) => {
                match notif.method.as_str() {
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
                            publish_diagnostics(
                                &connection,
                                params.text_document.uri,
                                &change.text,
                            )?;
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    Ok(())
}

fn publish_diagnostics(
    connection: &Connection,
    uri: Uri,
    source: &str,
) -> Result<(), Box<dyn std::error::Error + Sync + Send>> {
    let parse_result = lsp::parse(source);
    let diagnostics = (|| {
        let grammar_def = lsp::build_grammar_def(&parse_result)?;
        let spans = lsp::build_spans(&grammar_def, &parse_result)?;
        Some(lsp::diagnostics::diagnostics(
            &grammar_def,
            &spans,
            &parse_result.input,
        ))
    })()
    .unwrap_or_default();
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
