mod server;

use std::error::Error;

use lsp_server::Connection;
use lsp_types::{
    OneOf, SemanticTokensFullOptions, SemanticTokensOptions, ServerCapabilities,
};

use crate::server::main_loop;

fn main() -> Result<(), Box<dyn Error + Sync + Send>> {
    eprintln!("starting iguana LSP server");

    let (connection, io_threads) = Connection::stdio();

    let server_capabilities = serde_json::to_value(&ServerCapabilities {
        semantic_tokens_provider: Some(
            SemanticTokensOptions {
                legend: lsp::semantic_tokens::legend(),
                full: Some(SemanticTokensFullOptions::Bool(true)),
                range: None,
                work_done_progress_options: Default::default(),
            }
            .into(),
        ),
        document_formatting_provider: Some(OneOf::Left(true)),
        definition_provider: Some(OneOf::Left(true)),
        references_provider: Some(OneOf::Left(true)),
        ..Default::default()
    })
    .unwrap();

    let initialization_params = match connection.initialize(server_capabilities) {
        Ok(it) => it,
        Err(e) => {
            if e.channel_is_disconnected() {
                io_threads.join()?;
            }
            return Err(e.into());
        }
    };
    main_loop(connection, initialization_params)?;
    io_threads.join()?;

    eprintln!("shutting down iguana LSP server");
    Ok(())
}
