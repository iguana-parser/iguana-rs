use std::error::Error;

use lsp_server::{Connection, ExtractError, Message, Request, RequestId, Response};
use lsp_types::request::{Formatting, SemanticTokensFullRequest};
use lsp_types::{Position, Range, SemanticTokens, SemanticTokensResult, TextEdit};

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
                match cast::<SemanticTokensFullRequest>(req) {
                    Ok((id, params)) => {
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
                    Err(err @ ExtractError::JsonError { .. }) => panic!("{err:?}"),
                    Err(ExtractError::MethodMismatch(req)) => match cast::<Formatting>(req) {
                        Ok((id, params)) => {
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
                                let last_line_len = source.lines().last().map_or(0, |l| l.len()) as u32;
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
                        Err(err @ ExtractError::JsonError { .. }) => panic!("{err:?}"),
                        Err(ExtractError::MethodMismatch(_)) => {}
                    },
                }
            }
            Message::Response(resp) => {
                eprintln!("got response: {resp:?}");
            }
            Message::Notification(_) => {}
        }
    }
    Ok(())
}

fn cast<R>(req: Request) -> Result<(RequestId, R::Params), ExtractError<Request>>
where
    R: lsp_types::request::Request,
    R::Params: serde::de::DeserializeOwned,
{
    req.extract(R::METHOD)
}
