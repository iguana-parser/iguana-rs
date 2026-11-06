use std::error::Error;

use crate::documents::{DocumentStore, end_position};
use iguana_lsp::analysis::Analysis;
use iguana_lsp::format::format_source;
use lsp_server::{Connection, Message, Notification, Request, RequestId, Response};
use lsp_types::notification::Notification as LspNotification;
use lsp_types::request::{
    DocumentSymbolRequest, FoldingRangeRequest, Formatting, GotoDefinition, References,
    Request as LspRequest, SemanticTokensFullRequest,
};
use lsp_types::{
    Diagnostic, DocumentSymbolResponse, GotoDefinitionResponse, Location, Position,
    PublishDiagnosticsParams, Range, SemanticTokens, SemanticTokensResult, TextEdit, Uri,
};
use serde::Serialize;

pub fn main_loop(
    connection: Connection,
    params: serde_json::Value,
) -> Result<(), Box<dyn Error + Sync + Send>> {
    let _params: lsp_types::InitializeParams = serde_json::from_value(params).unwrap();
    eprintln!("starting main loop");
    let mut documents = DocumentStore::default();
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
                        respond(&connection, &documents, id, uri, |analysis| {
                            SemanticTokensResult::Tokens(SemanticTokens {
                                result_id: None,
                                data: analysis.semantic_tokens().to_vec(),
                            })
                        })?;
                    }
                    Formatting::METHOD => {
                        let (id, params) = cast::<Formatting>(req);
                        let uri = &params.text_document.uri;
                        respond(&connection, &documents, id, uri, |analysis| {
                            formatting_edits(analysis.source())
                        })?;
                    }
                    References::METHOD => {
                        let (id, params) = cast::<References>(req);
                        let uri = &params.text_document_position.text_document.uri;
                        let position = params.text_document_position.position;
                        let include_declaration = params.context.include_declaration;
                        respond(&connection, &documents, id, uri, |analysis| {
                            analysis
                                .references(position, include_declaration)
                                .into_iter()
                                .map(|range| Location::new(uri.clone(), range))
                                .collect::<Vec<_>>()
                        })?;
                    }
                    GotoDefinition::METHOD => {
                        let (id, params) = cast::<GotoDefinition>(req);
                        let uri = &params.text_document_position_params.text_document.uri;
                        let position = params.text_document_position_params.position;
                        respond(&connection, &documents, id, uri, |analysis| {
                            analysis.definition(position).map(|range| {
                                GotoDefinitionResponse::Scalar(Location::new(uri.clone(), range))
                            })
                        })?;
                    }
                    DocumentSymbolRequest::METHOD => {
                        let (id, params) = cast::<DocumentSymbolRequest>(req);
                        let uri = &params.text_document.uri;
                        respond(&connection, &documents, id, uri, |analysis| {
                            DocumentSymbolResponse::Nested(analysis.document_symbols().to_vec())
                        })?;
                    }
                    FoldingRangeRequest::METHOD => {
                        let (id, params) = cast::<FoldingRangeRequest>(req);
                        let uri = &params.text_document.uri;
                        respond(&connection, &documents, id, uri, |analysis| {
                            analysis.folding_ranges().to_vec()
                        })?;
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
                    let uri = params.text_document.uri;
                    let analysis = Analysis::new(&params.text_document.text);
                    publish_diagnostics(&connection, uri.clone(), analysis.diagnostics())?;
                    documents.set(uri, analysis);
                }
                lsp_types::notification::DidChangeTextDocument::METHOD => {
                    let params: lsp_types::DidChangeTextDocumentParams =
                        serde_json::from_value(notif.params).unwrap();
                    if let Some(change) = params.content_changes.into_iter().last() {
                        let uri = params.text_document.uri;
                        let analysis = Analysis::new(&change.text);
                        publish_diagnostics(&connection, uri.clone(), analysis.diagnostics())?;
                        documents.set(uri, analysis);
                    }
                }
                lsp_types::notification::DidCloseTextDocument::METHOD => {
                    let params: lsp_types::DidCloseTextDocumentParams =
                        serde_json::from_value(notif.params).unwrap();
                    documents.close(&params.text_document.uri);
                }
                _ => {}
            },
        }
    }
    Ok(())
}

/// Sends the answer computed from the analysis of `uri`. The server does not
/// respond to a request for a file it cannot read.
fn respond<T: Serialize>(
    connection: &Connection,
    documents: &DocumentStore,
    id: RequestId,
    uri: &Uri,
    answer: impl FnOnce(&Analysis) -> T,
) -> Result<(), Box<dyn Error + Sync + Send>> {
    match documents.with_analysis(uri, answer) {
        Ok(result) => {
            let result = serde_json::to_value(&result).unwrap();
            let resp = Response::new_ok(id, result);
            connection.sender.send(Message::Response(resp))?;
        }
        Err(e) => eprintln!("failed to read {}: {}", uri.path().as_str(), e),
    }
    Ok(())
}

/// The edits that replace `source` with its formatted text. The list is empty
/// when the formatted text equals `source`, and the result is `None` when
/// `source` does not parse or is ambiguous.
fn formatting_edits(source: &str) -> Option<Vec<TextEdit>> {
    let formatted = format_source(source)?;
    if formatted == source {
        return Some(Vec::new());
    }
    Some(vec![TextEdit {
        range: Range {
            start: Position::new(0, 0),
            end: end_position(source),
        },
        new_text: formatted,
    }])
}

fn publish_diagnostics(
    connection: &Connection,
    uri: Uri,
    diagnostics: &[Diagnostic],
) -> Result<(), Box<dyn Error + Sync + Send>> {
    let params = PublishDiagnosticsParams {
        uri,
        diagnostics: diagnostics.to_vec(),
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
